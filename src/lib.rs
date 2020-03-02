#![cfg_attr(not(feature = "std"), no_std)]

use node_primitives::Balance;
use proofs::Proof;
use sp_core::H256;
use sp_runtime::traits::{Saturating, StaticLookup, Zero};
use sp_std::vec::Vec;
use support::{
    decl_error, decl_event, decl_module, decl_storage,
    dispatch::{DispatchError, DispatchResult},
    ensure,
    traits::{Currency, Get, LockableCurrency, Randomness, ReservableCurrency},
    weights::SimpleDispatchInfo,
};
use system::{ensure_signed, RawOrigin};

// sp_std::result::Result<(), DispatchError>;

mod anchor;
mod erc721;
mod mock;
mod proofs;
mod tests;

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

        // Now token owner
        WrongTokenOwner,

        // Metadata length invalid
        MetadataLengthInvalid,

        // Not validation function
        NotValidationFunction,

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
        MintNft(RegistryUid, u64, Hash),
    }
);

decl_storage! {
    trait Store for Module<T: Trait> as NftRegistry {
        // Each registry uid include a validation function in smart contract
        pub ValidationFn get(validator_of): map hasher(blake2_256) RegistryUid => Option<T::AccountId>;

        // Each Uid can generate lots of token
        pub RegistryNonce: map hasher(blake2_256) RegistryUid => u64;

        // Next Registry id
        pub NextRegistryId: RegistryUid;

        // Rigistry UID and Nonce mapping to token id
        pub RegistryToken get(registry_token): map hasher(blake2_256) (RegistryUid, u64) => T::Hash;

        // Registry Metadata
        pub TokenByRegistryId get(token_by_registry_id): map hasher(blake2_256) T::Hash => (RegistryUid, u64);

        // Metadata for each token id
        pub RegistryTokenMetadata get(registry_token_metadata): map hasher(blake2_256) (RegistryUid, u64) => Vec<u8>;

        // Metadata min length
        pub MinMetadataLength get(min_token_uri_length) config(): u32;

        // Metadata max length
        pub MaxMetadataLength get(max_token_uri_length) config(): u32;

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

        // Register validation function
        fn new_registry(origin, validation_fn_addr: T::AccountId) -> DispatchResult {
            let sender = ensure_signed(origin)?;

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
        fn mint(origin,
                uid: RegistryUid,
                metadata: Vec<u8>,
                parameters: Vec<u8>,
                value: contracts::BalanceOf<T>,
                gas_limit: contracts::Gas,
            ) -> DispatchResult {
            let sender = ensure_signed(origin)?;

            // Contract registered for the uid
            Self::ensure_validation_fn_exists(uid)?;

            // Ensure metadata is valid
            Self::ensure_metadata_valid(&metadata)?;

            // Deposit for metadata bytes fee
            let bytes_deposit = <BalanceOf<T>>::from(metadata.len() as u32).saturating_mul(T::NFTDepositPerByte::get());

            // Compute the total deposit fee
            let total_deposit = bytes_deposit.saturating_add(T::NFTDepositBase::get());

            // Reserve the currency
            <T as Trait>::Currency::reserve(&sender, total_deposit)?;

            // Wasm contract should emit an event for success or failure
            <contracts::Module<T>>::call(
                T::Origin::from(RawOrigin::<T::AccountId>::Signed(sender)),
                T::Lookup::unlookup(Self::validator_of(uid).unwrap()),
                value,
                gas_limit,
                parameters)?;

            // Store metadata for token
            RegistryTokenMetadata::mutate((uid, RegistryNonce::get(uid)), |value| *value = metadata);

            Ok(())
        }

        // Call back interface for smart contract
        fn finish_mint(origin, uid: RegistryUid) -> DispatchResult {
            let sender = ensure_signed(origin)?;

            // Ensure uid is existed
            Self::ensure_sender_is_validation_function(uid, &sender)?;

            // Assign Nft uid
            let nft_uid = RegistryNonce::get(&uid);

            // Use the uid to create a new ERC721 token
            let token_id = Self::mint_nft(&sender, uid)?;

            // Insert token id to registry id map
            <TokenByRegistryId<T>>::insert(token_id, (uid, RegistryNonce::get(&uid)));

            // Insert registry id to token id map
            <RegistryToken<T>>::insert((uid, RegistryNonce::get(&uid)), token_id);

            // Increment nonce
            RegistryNonce::mutate(uid, |value| *value = nft_uid + 1);

            // Just emit an event
            Self::deposit_event(RawEvent::MintNft(uid, nft_uid, token_id));

            Ok(())
        }

        #[weight = SimpleDispatchInfo::FixedNormal(1_500_000)]
        fn validate_mint(origin, anchor_id: T::Hash, deposit_address: [u8; 20], pfs: Vec<Proof>, static_proofs: [H256;3]) -> DispatchResult {
            ensure_signed(origin)?;

            // get the anchor data from anchor ID
            let anchor_data = <anchor::Module<T>>::get_anchor_by_id(anchor_id).ok_or("Anchor doesn't exist")?;

            // validate proofs
            ensure!(Self::validate_proofs(anchor_data.get_doc_root(), &pfs, static_proofs), "Invalid proofs");

            // get the bundled hash
            let bundled_hash = Self::get_bundled_hash(pfs, deposit_address);

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
    // Compute metadata fee
    fn get_metadata_fee(token_id: &T::Hash) -> BalanceOf<T> {
        // Get registry id
        let (registry_id, nonce) = <TokenByRegistryId<T>>::get(token_id);

        // Transfer the deposit after token transfer done
        let metadata_length: u32 = RegistryTokenMetadata::get((registry_id, nonce)).len() as u32;

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
            Ok(())
        } else {
            Err(Error::<T>::MetadataLengthInvalid.into())
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
    fn _get_token_owner(token_id: &T::Hash) -> sp_std::result::Result<T::AccountId, DispatchError> {
        let owner = <erc721::Module<T>>::owner_of(token_id);
        match owner {
            Some(token_owner) => Ok(token_owner),
            None => Err(Error::<T>::WrongTokenOwner.into()),
        }
    }

    // Mint a NFT according to accound id
    fn mint_nft(
        account_id: &T::AccountId,
        _uid: RegistryUid,
    ) -> sp_std::result::Result<T::Hash, DispatchError> {
        let token_id = <erc721::Module<T>>::create_token(account_id)?;
        Ok(token_id)
    }

    fn validate_proofs(doc_root: T::Hash, pfs: &Vec<Proof>, static_proofs: [H256; 3]) -> bool {
        proofs::validate_proofs(H256::from_slice(doc_root.as_ref()), pfs, static_proofs)
    }

    /// Returns a Keccak hash of deposit_address + hash(keccak(name+value+salt)) of each proof provided.
    fn get_bundled_hash(pfs: Vec<Proof>, deposit_address: [u8; 20]) -> T::Hash {
        let bh = proofs::bundled_hash(pfs, deposit_address);
        let mut res: T::Hash = Default::default();
        res.as_mut().copy_from_slice(&bh[..]);
        res
    }

    // fn ensure_token_uri_valid(token_uri: &Vec<u8>) -> DispatchResult {
    //     let length = token_uri.len() as u32;
    //     if length > MaxTokenURILength::get() || length < MinTokenURILength::get() {
    //         Ok(())
    //     } else {
    //         Err(Error::<T>::TokenURILengthInvalid.into())
    //     }
    // }
}
