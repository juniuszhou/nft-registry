#![cfg_attr(not(feature = "std"), no_std)]

use system::{ensure_signed, RawOrigin};
use support::{
    decl_error, decl_event, decl_module, decl_storage, dispatch::DispatchResult, ensure,
    traits::{Randomness, Get, Currency, ReservableCurrency, LockableCurrency,}, weights::SimpleDispatchInfo,
};
use proofs::Proof;
use sp_core::H256;
use sp_runtime::traits::{StaticLookup, Saturating,};
use sp_std::vec::Vec;
use node_primitives::Balance;

mod anchor;
mod erc721;
// mod mock;
mod proofs;
// mod tests;

type RegistryUid = u64;
type BalanceOf<T> =
    <<T as Trait>::Currency as Currency<<T as system::Trait>::AccountId>>::Balance;

pub trait Trait:
    system::Trait + contracts::Trait + erc721::Trait + anchor::Trait
{
    type Event: From<Event<Self>> + Into<<Self as system::Trait>::Event>;
    /// Something that provides randomness in the runtime.
    type Randomness: Randomness<Self::Hash>;

    /// The amount of balance that must be deposited per byte of preimage stored.
    type MetadataByteDeposit: Get<BalanceOf<Self>>;

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
        // Token URI length
        // TokenURILengthInvalid,
        // Now token owner
        WrongTokenOwner,
        // Metadata length invalid
        MetadataLengthInvalid,


    }
}

decl_event!(
    pub enum Event<T> 
        where
        <T as system::Trait>::AccountId {
        // <T as system::Trait>::Hash 
        
        // Account register a new Uid with smart contract
        NewRegistry(AccountId, RegistryUid),
        
        // DepositAsset(Hash),

        // New NFT created
        MintNft(RegistryUid, u64),
    }
);

decl_storage! {
    trait Store for Module<T: Trait> as NftRegistry {
        // Each registry uid include a validation function in smart contract
        pub ValidationFn get(validator_of): map hasher(blake2_256) RegistryUid => Option<T::AccountId>;
        
        // Each Uid can generate lots of token
        pub RegistryNonce: map hasher(blake2_256) RegistryUid => u64;
        // Uid and Nonce map to token id
        
        // Next Registry id
        pub NextRegistryId: RegistryUid;

        // Rigistry UID and Nonce mapping to token id
        pub RegistryToken get(registry_token): map hasher(blake2_256) (RegistryUid, u64) => T::Hash;
        
        // Metadata for each token id
        pub RegistryTokenMetadata get(registry_token_metadata): map hasher(blake2_256) T::Hash => Vec<u8>;
        
        // Token URI not needed yet.
        // Token's URI
        // pub RegistryTokenURI get(registry_token_uri): map hasher(blake2_256) T::Hash => Vec<u8>;
        // Token URI min length
        // pub MinTokenURILength get(min_token_uri_length) config(): u32;
        // Token URI max length
        // pub MaxTokenURILength get(max_token_uri_length) config(): u32;

        // Metadata min length
        pub MinMetadataLength get(min_token_uri_length) config(): u32;
        // Metadata max length
        pub MaxMetadataLength get(max_token_uri_length) config(): u32;

    }
}

decl_module! {
    pub struct Module<T: Trait> for enum Call where origin: T::Origin  {
        fn deposit_event() = default;

        const MetadataByteDeposit: BalanceOf<T> = T::MetadataByteDeposit::get();

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

            // Ensure 
            let deposit = <BalanceOf<T>>::from(metadata.len() as u32)
            .saturating_mul(T::MetadataByteDeposit::get());
            <T as Trait>::Currency::reserve(&sender, deposit)?;

            // Ensure metadata is valid
            Self::ensure_metadata_valid(&metadata)?;

            // Wasm contract should emit an event for success or failure
            <contracts::Module<T>>::call(
                T::Origin::from(RawOrigin::<T::AccountId>::Signed(sender)),
                T::Lookup::unlookup(Self::validator_of(uid).unwrap()),
                value,
                gas_limit,
                parameters)?;

            Ok(())
        }

        // Call back interface for smart contract
        fn finish_mint(origin, uid: RegistryUid) -> DispatchResult {
            let sender = ensure_signed(origin)?;

            // Ensure the caller is the validation contract for the corresponding NFT class
            ensure!(Self::validator_of(&uid)
                        .map_or(false, |validator_addr| validator_addr == sender),
                        "Sender must be validator contract for this Nft registry");

            // Assign Nft uid
            let nft_uid = RegistryNonce::get(&uid);

            let nplus1 = nft_uid.checked_add(1)
                .ok_or("Overflow when incrementing registry nonce")?;

            // Use the uid to create a new ERC721 token
            Self::mint_nft(sender, uid)?;

            // Increment nonce
            RegistryNonce::insert(uid, nplus1);

            // Just emit an event
            Self::deposit_event(RawEvent::MintNft(uid, nft_uid));

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

            // Self::deposit_event(RawEvent::DepositAsset(bundled_hash));

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
    // fn ensure_token_uri_valid(token_uri: &Vec<u8>) -> DispatchResult {
    //     let length = token_uri.len() as u32;
    //     if length > MaxTokenURILength::get() || length < MinTokenURILength::get() {
    //         Ok(())
    //     } else {
    //         Err(Error::<T>::TokenURILengthInvalid.into())
    //     }
    // }

    fn ensure_metadata_valid(metadata: &Vec<u8>) -> DispatchResult {
        let length = metadata.len() as u32;
        if length > MaxMetadataLength::get() || length < MinMetadataLength::get() {
            Ok(())
        } else {
            Err(Error::<T>::MetadataLengthInvalid.into())
        }
    }

    fn ensure_token_owner(token_id: &T::Hash, sender: &T::AccountId) -> DispatchResult {
        if  <erc721::Module<T>>::_is_approved_or_owner(sender.clone(), *token_id) {
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
        if <erc721::Module<T>>::_exists(*token_id) {
            Ok(())
        } else {
            Err(Error::<T>::TokenNotMinted.into())
        }
    }

    // Mint a NFT according to accound id
    fn mint_nft(account_id: T::AccountId, uid: RegistryUid) -> DispatchResult {
        let token_id = <erc721::Module<T>>::create_token(account_id)?;
        <RegistryToken<T>>::insert((uid, RegistryNonce::get(uid)), token_id);
        Ok(())
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
}
