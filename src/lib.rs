#![cfg_attr(not(feature = "std"), no_std)]

use proofs::Proof;
use sp_core::H256;
use sp_runtime::traits::{Saturating, StaticLookup};
use sp_std::collections::btree_set::BTreeSet;
use sp_std::{result::Result, vec::Vec};
use support::{
    decl_error, decl_event, decl_module, decl_storage,
    dispatch::{DispatchError, DispatchResult},
    ensure,
    traits::{Currency, Get, LockableCurrency, Randomness, ReservableCurrency},
    weights::SimpleDispatchInfo,
};
use system::{ensure_signed, RawOrigin};

// Encoding library
use codec::Encode;

mod anchor;
mod erc721;
mod proofs;

#[cfg(test)]
mod mock;
#[cfg(test)]
mod tests;

#[derive(Encode)]
struct ContractParameter<Hash, Account> {
    uid: RegistryUid,
    token_id: Hash,
    token_owner: Account,
    metadata: Vec<u8>,
    proof_leaves: Vec<H256>,
}

type RegistryUid = u64;
type BalanceOf<T> = <<T as Trait>::Currency as Currency<<T as system::Trait>::AccountId>>::Balance;

pub trait Trait: system::Trait + contracts::Trait + erc721::Trait + anchor::Trait {
    type Event: From<Event<Self>> + Into<<Self as system::Trait>::Event>;
    /// Something that provides randomness in the runtime.
    type Randomness: Randomness<Self::Hash>;

    /// The amount of balance that must be deposited for metadata stored once.
    type NFTDepositBase: Get<BalanceOf<Self>>;

    /// The amount of balance that must be deposited per byte of metadata stored.
    type NFTDepositPerByte: Get<BalanceOf<Self>>;

    /// The amount of balance that must be deposited per validation function registry.
    type NFTValidationRegistryDeposit: Get<BalanceOf<Self>>;

    /// Currency type for this module.
    type Currency: ReservableCurrency<Self::AccountId>
        + LockableCurrency<Self::AccountId, Moment = Self::BlockNumber>;
}

decl_error! {
    pub enum Error for Module<T: Trait> {
        // Validation function not registered
        ValidationFunctionNotRegistered,

        // Validation function already exists
        ValidationFunctionAlreadyExists,

        // Not validation function
        NotValidationFunction,

        // Proof validation failed
        ProofValidationFailure,

        // Proof validation failed
        DocumentNotAnchored,
    }
}

decl_event!(
    pub enum Event<T>
        where
        <T as system::Trait>::AccountId,
        <T as system::Trait>::Hash {
        // Account register a new Uid with smart contract
        NewRegistry(AccountId, RegistryUid),

        // New NFT created
        MintNft(RegistryUid, Hash),
    }
);

decl_storage! {
    trait Store for Module<T: Trait> as NftRegistry {
        // Each registry uid include a validation function in smart contract
        pub ValidationFn get(validator_fn): map hasher(blake2_256) RegistryUid => Option<T::AccountId>;

        // Next Registry id
        pub NextRegistryId: RegistryUid;

        // Registry Metadata
        pub RegistryUidForTokenId get(registry_uid_for_token_id): map T::Hash => RegistryUid;

        // Metadata for each token id
        pub TokenMetadata get(token_metadata): map T::Hash => Vec<u8>;

        // Reserved currency for each token
        pub DepositByTokenId get(deposit_by_token_id): map T::Hash => BalanceOf<T>;

        // Validation function map to avoid register again
        pub ValidationFunctionMap get(validation_function_map): map T::AccountId => bool;
    }
}

decl_module! {
    pub struct Module<T: Trait> for enum Call where origin: T::Origin  {
        fn deposit_event() = default;

        // Register validation function
        fn new_registry(origin, validation_fn_addr: T::AccountId) -> DispatchResult {
            let sender = ensure_signed(origin)?;

            // Ensure function not registered before
            if <ValidationFunctionMap<T>>::exists(&validation_fn_addr) {
                return Err(Error::<T>::ValidationFunctionAlreadyExists.into());
            }

            // Reserve fee for validation function
            <T as Trait>::Currency::reserve(&sender, T::NFTValidationRegistryDeposit::get())?;

            // Keep old value for event
            let uid = NextRegistryId::get();

            // Write state
            <ValidationFn<T>>::insert(&uid, &validation_fn_addr);

            // Update next registry id
            NextRegistryId::mutate(|value| *value += 1);

            // Put the validation function in set
            <ValidationFunctionMap<T>>::insert(&validation_fn_addr, true);

            // Store event
            Self::deposit_event(RawEvent::NewRegistry(sender, uid));

            Ok(())
        }

        // Mint a new token
        // Value: account can transfer some currency to smart contract via calling
        // Gas limit: set the maximum gas usage for smart contract execution in WASM
        fn mint(origin,
            registry_uid: RegistryUid,
            token_id: T::Hash,
            metadata: Vec<u8>,
            anchor_id: T::Hash,
            proofs: Vec<Proof>,
            static_proofs: [H256;3],
            value: contracts::BalanceOf<T>,
            gas_limit: contracts::Gas,
        ) -> DispatchResult {
            let sender = ensure_signed(origin)?;

            // Contract registered for the uid
            let validation_function = Self::ensure_validation_fn_exists(registry_uid)?;

            // Ensure token id not existed
            <erc721::Module<T>>::ensure_token_not_existed(&token_id)?;

            // Get the doc root
            let doc_root = Self::get_document_root(&anchor_id)?;

            // Verify the proof against document root
            Self::validate_proofs(&doc_root, &proofs, &static_proofs)?;

            // Get the hash of validate method in contract
            let keccak = ink_utils::hash::keccak256("validate".as_bytes());

            // Encode 4 bytes of the method name's hash
            let selector = [keccak[0], keccak[1], keccak[2], keccak[3]];
            let mut call = selector.encode();

            // Collect all leaves in proofs
            let proof_leaves = proofs.iter().map(|proof| proof.leaf_hash).collect();

            // Put parameters into single struct.
            let contract_parameter = ContractParameter::<T::Hash, T::AccountId> {
                uid: registry_uid,
                token_id: token_id,
                token_owner: sender.clone(),
                metadata: metadata.clone(),
                proof_leaves: proof_leaves,
            };

            // Append the parameter after method
            call.append(&mut Encode::encode(&contract_parameter));

            // Call the contract via bare call
            <contracts::Module<T>>::bare_call(
                sender,
                validation_function,
                value,
                gas_limit,
                call);

            Ok(())
        }

        // Call back interface for smart contract
        fn finish_mint(origin, uid: RegistryUid, token_id: T::Hash, token_owner: T::AccountId, metadata: Vec<u8>) -> DispatchResult {
            let sender = ensure_signed(origin)?;

            // Ensure uid is existed
            Self::ensure_sender_is_validation_function(uid, &sender)?;

            // Get storage fee for metadata
            let total_deposit = Self::compute_metadata_fee(metadata.len() as u32);

            // Reserve fee for token
            <T as Trait>::Currency::reserve(&token_owner, total_deposit)?;

            // Use the uid to create a new ERC721 token, BUG owner is not sender.
            <erc721::Module<T>>::_mint(&token_owner, &token_id)?;

            // Insert token id to registry id map
            <RegistryUidForTokenId<T>>::insert(&token_id, uid);

            // Insert token metadata
            <TokenMetadata<T>>::insert(&token_id, metadata);

            // Insert deposit into storage
            <DepositByTokenId<T>>::insert(&token_id, total_deposit);

            // Just emit an event
            Self::deposit_event(RawEvent::MintNft(uid, token_id));

            Ok(())
        }

        // transfer_from will transfer to addresses even without a balance
        fn transfer_from(origin, from: T::AccountId, to: T::AccountId, token_id: T::Hash) -> DispatchResult {
            let sender = ensure_signed(origin)?;

            // Get the deposited fee
            let total_deposit = Self::deposit_by_token_id(&token_id);

            // Reserve the currency
            <T as Trait>::Currency::repatriate_reserved(&from, &to, total_deposit)?;

            // Transfer token
            <erc721::Module<T>>::_transfer_from(&sender, &from, &to, &token_id)?;

            Ok(())
        }

        // Burn the token and unreserve the currency for metadata
        fn burn(origin, token_id: T::Hash) -> DispatchResult {
            let sender = ensure_signed(origin)?;

            // Burn the token
            <erc721::Module<T>>::_burn(&sender, &token_id)?;

            // Get the total deposit fee
            let total_deposit = Self::deposit_by_token_id(&token_id);

            // Repay deposit currency to token owner
            <T as Trait>::Currency::unreserve(&sender, total_deposit);

            // Remove storage related to burned token
            <TokenMetadata<T>>::remove(&token_id);
            <DepositByTokenId<T>>::remove(&token_id);
            <RegistryUidForTokenId<T>>::remove(&token_id);

            Ok(())
        }

        // Approve a token to an account
        fn approve(origin, to: T::AccountId, token_id: T::Hash) -> DispatchResult {
            let sender = ensure_signed(origin)?;

            <erc721::Module<T>>::_approve(&sender, &to, &token_id)?;

            Ok(())
        }

        // Set if an operator can transfer an owner's token
        fn set_approval_for_all(origin, to: T::AccountId, approved: bool) -> DispatchResult {
            let sender = ensure_signed(origin)?;

            <erc721::Module<T>>::_set_approval_for_all(&sender, &to, approved)?;

            Ok(())
        }
    }
}

impl<T: Trait> Module<T> {
    // Get the document root via anchor id
    fn get_document_root(anchor_id: &T::Hash) -> Result<T::Hash, DispatchError> {
        match <anchor::Module<T>>::get_anchor_by_id(*anchor_id) {
            Some(anchor_data) => Ok(anchor_data.doc_root),
            None => Err(Error::<T>::DocumentNotAnchored.into()),
        }
    }

    // Compute deposit fee according to length
    fn compute_metadata_fee(metadata_length: u32) -> BalanceOf<T> {
        // Deposit for metadata bytes fee
        let bytes_deposit =
            <BalanceOf<T>>::from(metadata_length).saturating_mul(T::NFTDepositPerByte::get());

        // Get the fee
        bytes_deposit.saturating_add(T::NFTDepositBase::get())
    }

    // Ensure sender is uid's validation smart contract
    fn ensure_sender_is_validation_function(
        uid: RegistryUid,
        sender: &T::AccountId,
    ) -> DispatchResult {
        let validation_function = Self::ensure_validation_fn_exists(uid)?;

        if validation_function == *sender {
            Ok(())
        } else {
            Err(Error::<T>::NotValidationFunction.into())
        }
    }

    // Ensure validataion function exists in storage
    fn ensure_validation_fn_exists(
        uid: RegistryUid,
    ) -> sp_std::result::Result<T::AccountId, DispatchError> {
        match <ValidationFn<T>>::get(uid) {
            Some(validation_function) => Ok(validation_function),
            None => Err(Error::<T>::ValidationFunctionNotRegistered.into()),
        }
    }

    // Validate proof via merkle tree
    fn validate_proofs(
        doc_root: &T::Hash,
        proofs: &Vec<Proof>,
        static_proofs: &[H256; 3],
    ) -> DispatchResult {
        if proofs::validate_proofs(H256::from_slice(doc_root.as_ref()), proofs, *static_proofs) {
            Ok(())
        } else {
            Err(Error::<T>::ProofValidationFailure.into())
        }
    }
}
