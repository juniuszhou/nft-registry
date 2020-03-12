#![cfg_attr(not(feature = "std"), no_std)]

use proofs::Proof;
use sp_core::H256;
use sp_runtime::traits::{Saturating, StaticLookup};
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
mod mock;
mod proofs;
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
        /// Validation function not registered
        ValidationFnNotExisted,

        /// Uid not registered
        UidNotRegistered,

        /// Token not minted
        TokenNotMinted,

        /// Token already existed
        TokenAlreadyExisted,

        // Now token owner
        WrongTokenOwner,

        // Metadata length invalid
        MetadataLengthInvalid,

        // Not validation function
        NotValidationFunction,

        // Proof validation failed
        ProofValidationFailure,

        // Proof validation failed
        DocumentNotAnchored,

        // Token URI length
        // TokenURILengthInvalid,
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
        pub ValidationFn get(validator_of): map hasher(blake2_256) RegistryUid => Option<T::AccountId>;

        // Next Registry id
        pub NextRegistryId: RegistryUid;

        // Registry Metadata
        pub TokenByRegistryId get(token_by_registry_id): map T::Hash => RegistryUid;

        // Metadata for each token id
        pub RegistryTokenMetadata get(registry_token_metadata): map T::Hash => Vec<u8>;

        // Metadata min length
        pub MinMetadataLength get(min_token_metadata_length) config(): u32;

        // Metadata max length
        pub MaxMetadataLength get(max_token_metadata_length) config(): u32;

        // Token URI not needed yet.
        // Token's URI
        // pub RegistryTokenURI get(registry_token_uri): map hasher(blake2_256) T::Hash => Vec<u8>;
        // Token URI min length
        // pub MinTokenURILength get(min_token_uri_length) config(): u32;
        // Token URI max length
        // pub MaxTokenURILength get(max_token_uri_length) config(): u32;
    }
}

decl_module! {
    pub struct Module<T: Trait> for enum Call where origin: T::Origin  {
        fn deposit_event() = default;

        // Fee for store one byte
        const NFTDepositPerByte: BalanceOf<T> = T::NFTDepositPerByte::get();

        // Fee for one NFT metadata storage
        const NFTDepositBase: BalanceOf<T> = T::NFTDepositBase::get();

        // Fee for each NFT validation registry
        const NFTValidationRegistryDeposit: BalanceOf<T> = T::NFTValidationRegistryDeposit::get();

        // Register validation function
        fn new_registry(origin, validation_fn_addr: T::AccountId) -> DispatchResult {
            let sender = ensure_signed(origin)?;

            // Reserve fee for token
            <T as Trait>::Currency::reserve(&sender, T::NFTValidationRegistryDeposit::get())?;

            // Keep old value for event
            let uid = NextRegistryId::get();

            // Write state
            <ValidationFn<T>>::insert(&uid, validation_fn_addr);

            // Update next registry id
            NextRegistryId::mutate(|value| *value += 1);

            // Events
            Self::deposit_event(RawEvent::NewRegistry(sender, uid));

            Ok(())
        }

        // Mint a new token
        // Value: account can transfer some currency to smart contract via calling
        // Gas limit: set the maximum gas usage for smart contract execution in WASM
        fn mint(origin,
            uid: RegistryUid,
            token_id: T::Hash,
            metadata: Vec<u8>,
            anchor_id: T::Hash,
            pfs: Vec<Proof>,
            static_proofs: [H256;3],
            value: contracts::BalanceOf<T>,
            gas_limit: contracts::Gas,
        ) -> DispatchResult {
            let sender = ensure_signed(origin)?;

            // Contract registered for the uid
            Self::ensure_validation_fn_exists(uid)?;

            // Ensure token id not existed
            Self::ensure_token_not_existed(&token_id)?;

            // Ensure metadata is valid
            Self::ensure_metadata_valid(&metadata)?;

            // Get the doc root
            let doc_root = Self::get_document_root(&anchor_id)?;

            // Verify the proof against document root
            Self::validate_proofs(&doc_root, &pfs, &static_proofs)?;

            // Get the hash of validate method in contract
            let keccak = ink_utils::hash::keccak256("validate".as_bytes());

            // Encode 4 bytes of the method name's hash
            let selector = [keccak[0], keccak[1], keccak[2], keccak[3]];
            let mut call = selector.encode();

            // Compute the bundled hash
            let proof_leaves = pfs.iter().map(|proof| proof.leaf_hash).collect();

            // Put parameters into single struct.
            let contract_parameter = ContractParameter::<T::Hash, T::AccountId> {
                uid: uid,
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
                Self::validator_of(uid).unwrap(),
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

            // Use the uid to create a new ERC721 token, BUG owner is not sender.
            Self::mint_nft(&token_owner, &token_id)?;

            // Get storage fee for metadata
            let total_deposit = Self::compute_metadata_fee(metadata.len() as u32);

            // Reserve fee for token
            <T as Trait>::Currency::reserve(&token_owner, total_deposit)?;

            // Insert token id to registry id map
            <TokenByRegistryId<T>>::insert(&token_id, uid);

            // Insert token metadata
            <RegistryTokenMetadata<T>>::insert(&token_id, metadata);

            // Just emit an event
            Self::deposit_event(RawEvent::MintNft(uid, token_id));

            Ok(())
        }

        // transfer_from will transfer to addresses even without a balance
        fn transfer_from(origin, from: T::AccountId, to: T::AccountId, token_id: T::Hash) -> DispatchResult {
            let sender = ensure_signed(origin)?;

            // Check if sender can transfer token
            Self::ensure_is_approved_or_owner(&token_id, &sender)?;

            // Call transfer
            Self::_transfer_from(&from, &to, &token_id)?;

            // Compute the total deposit fee
            let total_deposit = Self::get_metadata_fee(&token_id);

            // Reserve the currency
            <T as Trait>::Currency::repatriate_reserved(&from, &to, total_deposit)?;

            Ok(())
        }

        // Burn the token and unreserve the currency for metadata
        fn burn(origin, token_id: T::Hash) -> DispatchResult {
            let sender = ensure_signed(origin)?;

            // Ensure token is existed
            Self::ensure_token_exists(&token_id)?;

            // Check if sender can transfer token
            Self::ensure_is_approved_or_owner(&token_id, &sender)?;

            // burn the token
            Self::_burn(&token_id)?;

            // Compute the total deposit fee
            let total_deposit = Self::get_metadata_fee(&token_id);

            // Repay deposit currency to token owner
            <T as Trait>::Currency::unreserve(&sender, total_deposit);

            Ok(())
        }

        // uri not necessary yet
        // fn register_token_uri(origin, token_id: T::Hash, token_uri: Vec<u8>)-> DispatchResult {
        //     // Get sender from signature
        //     let sender = ensure_signed(origin)?;

        //     // Ensure token existed in ERC 721 contract
        //     Self::ensure_token_exists(&token_id)?;

        //     // Only token's owner can update URI
        //     Self::ensure_token_owner(&token_id, &sender)?;

        //     // Check the length of token's URI
        //     Self::ensure_token_uri_valid(&token_uri)?;

        //     // Set token URI
        //     <RegistryTokenURI<T>>::mutate(token_id, |value| *value = token_uri);

        //     Ok(())
        // }
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

    // Compute metadata fee
    fn get_metadata_fee(token_id: &T::Hash) -> BalanceOf<T> {
        // Transfer the deposit after token transfer done
        let metadata_length: u32 = <RegistryTokenMetadata<T>>::get(token_id).len() as u32;

        Self::compute_metadata_fee(metadata_length)
    }

    // Compute deposit fee according to length
    fn compute_metadata_fee(metadata_length: u32) -> BalanceOf<T> {
        // Deposit for metadata bytes fee
        let bytes_deposit =
            <BalanceOf<T>>::from(metadata_length).saturating_mul(T::NFTDepositPerByte::get());

        // Get the fee
        bytes_deposit.saturating_add(T::NFTDepositBase::get())
    }

    // ensure sender is uid's validation smart contract
    fn ensure_sender_is_validation_function(
        uid: RegistryUid,
        sender: &T::AccountId,
    ) -> DispatchResult {
        Self::ensure_validation_fn_exists(uid)?;

        if <ValidationFn<T>>::get(uid).unwrap() == *sender {
            Ok(())
        } else {
            Err(Error::<T>::NotValidationFunction.into())
        }
    }

    fn ensure_metadata_valid(metadata: &Vec<u8>) -> DispatchResult {
        let length = metadata.len() as u32;
        if length > MaxMetadataLength::get() || length < MinMetadataLength::get() {
            Err(Error::<T>::MetadataLengthInvalid.into())
        } else {
            Ok(())
        }
    }

    fn ensure_is_approved_or_owner(token_id: &T::Hash, sender: &T::AccountId) -> DispatchResult {
        if <erc721::Module<T>>::_is_approved_or_owner(sender, token_id) {
            Ok(())
        } else {
            Err(Error::<T>::WrongTokenOwner.into())
        }
    }

    fn ensure_validation_fn_exists(uid: RegistryUid) -> DispatchResult {
        match <ValidationFn<T>>::get(uid) {
            Some(_) => Ok(()),
            None => Err(Error::<T>::ValidationFnNotExisted.into()),
        }
    }

    fn ensure_token_exists(token_id: &T::Hash) -> DispatchResult {
        if <erc721::Module<T>>::_exists(token_id) {
            Ok(())
        } else {
            Err(Error::<T>::TokenNotMinted.into())
        }
    }

    fn ensure_token_not_existed(token_id: &T::Hash) -> DispatchResult {
        if <erc721::Module<T>>::_exists(token_id) {
            Err(Error::<T>::TokenAlreadyExisted.into())
        } else {
            Ok(())
        }
    }

    fn _transfer_from(
        from: &T::AccountId,
        to: &T::AccountId,
        token_id: &T::Hash,
    ) -> DispatchResult {
        <erc721::Module<T>>::_transfer_from(from, to, token_id)
    }

    fn _burn(token_id: &T::Hash) -> DispatchResult {
        <erc721::Module<T>>::_burn(token_id)
    }

    // Get token owner
    fn _get_token_owner(token_id: &T::Hash) -> Result<T::AccountId, DispatchError> {
        let owner = <erc721::Module<T>>::owner_of(token_id);
        match owner {
            Some(token_owner) => Ok(token_owner),
            None => Err(Error::<T>::WrongTokenOwner.into()),
        }
    }

    // Mint a NFT according to accound id
    fn mint_nft(account_id: &T::AccountId, token_id: &T::Hash) -> DispatchResult {
        <erc721::Module<T>>::_mint(account_id, token_id)
    }

    // Validate proof via merkle tree
    fn validate_proofs(
        doc_root: &T::Hash,
        pfs: &Vec<Proof>,
        static_proofs: &[H256; 3],
    ) -> DispatchResult {
        if proofs::validate_proofs(H256::from_slice(doc_root.as_ref()), pfs, *static_proofs) {
            Ok(())
        } else {
            Err(Error::<T>::ProofValidationFailure.into())
        }
    }

    /// Returns a Keccak hash of deposit_address + hash(keccak(name+value+salt)) of each proof provided.
    fn get_bundled_hash(pfs: Vec<Proof>, deposit_address: [u8; 20]) -> T::Hash {
        let bh = proofs::bundled_hash(pfs, deposit_address);
        let mut res: T::Hash = Default::default();
        res.as_mut().copy_from_slice(&bh[..]);
        res
    }

    // uri not necessary yet
    // fn ensure_token_uri_valid(token_uri: &Vec<u8>) -> DispatchResult {
    //     let length = token_uri.len() as u32;
    //     if length > MaxTokenURILength::get() || length < MinTokenURILength::get() {
    //         Ok(())
    //     } else {
    //         Err(Error::<T>::TokenURILengthInvalid.into())
    //     }
    // }
}
