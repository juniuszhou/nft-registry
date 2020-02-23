#![cfg_attr(not(feature = "std"), no_std)]

use system::{ensure_signed, RawOrigin};

use node_primitives::{Balance, BlockNumber, Hash};
use support::{
    decl_error, decl_event, decl_module, decl_storage, dispatch::DispatchResult, ensure,
    traits::Randomness, weights::SimpleDispatchInfo,
};
// use sp_core::hashing::keccak_256;
use proofs::Proof;
use sp_core::H256;
use sp_io::hashing::{blake2_256, keccak_256};
use sp_runtime::traits::{BlakeTwo256, Hash as HashT, StaticLookup};
use sp_runtime::RandomNumberGenerator;
use sp_std::vec::Vec;

// Encoding library
use codec::{Codec, Decode, Encode};

mod anchor;
mod erc721;
mod proofs;

struct NftContract;
type RegistryUid = u64;
type NftUid = u64;

pub trait Trait:
    system::Trait + balances::Trait + contracts::Trait + erc721::Trait + anchor::Trait
{
    type Event: From<Event<Self>> + Into<<Self as system::Trait>::Event>;
    /// Something that provides randomness in the runtime.
    type Randomness: Randomness<Self::Hash>;
}

decl_error! {
    pub enum Error for Module<T: Trait> {
        /// No owner set for token
        Error
    }
}

decl_event!(
    pub enum Event<T> 
        where
        <T as system::Trait>::AccountId,
        <T as system::Trait>::Hash {
        NewRegistry(AccountId, RegistryUid),
        DepositAsset(Hash),
        MintNft(RegistryUid, u64),
    }
);

decl_storage! {
    trait Store for Module<T: Trait> as NftRegistry {
        pub ValidationFn get(validator_of): map hasher(blake2_256) RegistryUid => Option<T::AccountId>;
        pub RegistryNonce: map RegistryUid => u64;
        pub RegistryToken get(registry_token): map RegistryUid => T::Hash;
        pub Nonce: u64;
    }
}

decl_module! {
    pub struct Module<T: Trait> for enum Call where origin: T::Origin  {
        fn deposit_event() = default;

        fn new_registry(origin, validation_fn_addr: T::AccountId) -> DispatchResult {
            let sender = ensure_signed(origin)?;

            // Generate a uid and check that it's unique
            let nonce = Nonce::get();
            let uid = nonce;
            // let uid = (nonce).using_encoded(<T as system::Trait>::Hashing::hash);
            // ensure!(!<ValidationFn<T>>::exists(uid), "This new id for a registry already exists!");

            // Check for overflow on index
            let nplus1 = Nonce::get().checked_add(1)
                .ok_or("Nonce overflow when adding a new registry")?;

            // Write state
            <ValidationFn<T>>::insert(&uid, validation_fn_addr);
            Nonce::put( nplus1 );

            // Events
            Self::deposit_event(RawEvent::NewRegistry(sender, uid));

            Ok(())
        }

        fn mint(origin,
                uid: RegistryUid,
                parameters: Vec<u8>,
                value: contracts::BalanceOf<T>,
                gas_limit: contracts::Gas,
            ) -> DispatchResult {
            let sender = ensure_signed(origin)?;

            ensure!(<ValidationFn<T>>::exists(uid), "No registry with this uid exists");

            let validation_fn = Self::validator_of(uid)
                .ok_or("This should not happen bcs ensure above^")?;

            // Wasm contract should emit an event for success or failure
            <contracts::Module<T>>::call(
                T::Origin::from(RawOrigin::<T::AccountId>::Signed(sender)),
                T::Lookup::unlookup(validation_fn),
                value,
                gas_limit,
                parameters);

            Ok(())
        }

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
            Self::mint_nft(sender, uid);

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

            Self::deposit_event(RawEvent::DepositAsset(bundled_hash));

            Ok(())
        }
    }
}

impl<T: Trait> Module<T> {
    fn mint_nft(account_id: T::AccountId, uid: RegistryUid) -> DispatchResult {
        let token_id = <erc721::Module<T>>::create_token(account_id)?;
        <RegistryToken<T>>::insert(uid, token_id);
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

/// tests for nft_registry module
#[cfg(test)]
mod tests;
