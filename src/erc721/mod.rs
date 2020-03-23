#![cfg_attr(not(feature = "std"), no_std)]
use sp_runtime::traits::Hash as HashT;
use support::{
    decl_error, decl_event, decl_module, decl_storage, dispatch::DispatchError,
    dispatch::DispatchResult, traits::Randomness, Parameter,
};
use system::ensure_signed;

use sp_runtime::traits::{MaybeSerialize, Member, One, SimpleArithmetic};

// Encoding library
use codec::{Codec, Encode};

#[cfg(test)]
mod mock;
#[cfg(test)]
mod tests;

pub trait Trait: system::Trait {
    type Event: From<Event<Self>> + Into<<Self as system::Trait>::Event>;

    // Something that provides randomness generation in the runtime.
    type Randomness: Randomness<Self::Hash>;

    // Token index used for token enumeration, set as parameter in Trait to make it
    // more flexible to binding during building runtime
    type TokenIndex: Parameter
        + Member
        + SimpleArithmetic
        + Codec
        + Default
        + Copy
        + MaybeSerialize
        + PartialEq;
}

decl_error! {
    pub enum Error for Module<T: Trait> {
        // Token not existed in storage
        TokenNotExisted,

        // Token already exists
        TokenAlreadyExists,

        // No owner set for token
        TokenOwnerNotSet,

        // Account not token's owner
        NotTokenOwner,

        // Owner not needed to be approver
        OwnerAlwaysCanApprove,

        // Not token's owner or approver
        NotOwnerOrApprover,
    }
}

decl_event!(
    pub enum Event<T>
        where
        <T as system::Trait>::AccountId,
        <T as system::Trait>::Hash {
        // Token transfer event
        Transfer(Option<AccountId>, Option<AccountId>, Hash),

        // One token approved to an account
        Approval(AccountId, AccountId, Hash),

        // All tokens owned are approved to other account
        ApprovalForAll(AccountId, AccountId, bool),
    }
);

decl_storage! {
    trait Store for Module<T: Trait> as ERC721 {
        // Token count owned by an account
        pub OwnedTokensCount get(balance_of): map T::AccountId => T::TokenIndex;

        // Token owner's account id
        pub TokenOwner get(owner_of): map T::Hash => Option<T::AccountId>;

        // Token delegate's account id
        pub TokenApprovals get(get_approved): map T::Hash => Option<T::AccountId>;

        // Account delegate's account id
        pub OperatorApprovals get(is_approved_for_all): double_map T::AccountId, twox_128(T::AccountId) => bool;

        // Total count of minted token
        pub TotalSupply get(total_supply): T::TokenIndex;

        // Map of token index to token id
        pub AllTokens get(token_by_index): map T::TokenIndex => T::Hash;

        // Last token index
        pub AllTokensIndex: map T::Hash => T::TokenIndex;

        // Map of account and index to token id
        pub OwnedTokens get(token_of_owner_by_index): double_map T::AccountId, twox_128(T::TokenIndex)  => T::Hash;

        // Token id to index
        pub OwnedTokensIndex: map T::Hash => T::TokenIndex;

        // Value use to generate random value
        pub Nonce: u64;
    }
}

decl_module! {
    pub struct Module<T: Trait> for enum Call where origin: T::Origin  {
        fn deposit_event() = default;

        // Mint a new token
        fn mint(origin, token_id: T::Hash) -> DispatchResult {
            let sender = ensure_signed(origin)?;

            Self::_mint(&sender, &token_id)?;

            Ok(())
        }

        // Burn a token
        fn burn(origin, token_id: T::Hash) -> DispatchResult {
            let sender = ensure_signed(origin)?;

            Self::_burn(&sender, &token_id)?;

            Ok(())
        }

        // Approve a token to an account
        fn approve(origin, to: T::AccountId, token_id: T::Hash) -> DispatchResult {
            let sender = ensure_signed(origin)?;

            Self::_approve(&sender, &to, &token_id)?;

            Ok(())
        }

        // Set if an operator can transfer an owner's token
        fn set_approval_for_all(origin, to: T::AccountId, approved: bool) -> DispatchResult {
            let sender = ensure_signed(origin)?;

            Self::_set_approval_for_all(&sender, &to, approved)?;

            Ok(())
        }

        // Transfer token
        fn transfer_from(origin, from: T::AccountId, to: T::AccountId, token_id: T::Hash) -> DispatchResult {
            let sender = ensure_signed(origin)?;

            Self::_transfer_from(&sender, &from, &to, &token_id)?;

            Ok(())
        }

        // Create a new token, not part of ECR721 specification but useful to get a token
        fn create_token(origin) -> DispatchResult {
            let sender = ensure_signed(origin)?;

            Self::_create_token(&sender)?;

            Ok(())
        }
    }
}

impl<T: Trait> Module<T> {
    // Create a new token and token id from random generator
    pub fn _create_token(
        account_id: &T::AccountId,
    ) -> sp_std::result::Result<T::Hash, DispatchError> {
        // Nonce used to generate random hash value
        let nonce = Nonce::get();

        // Generate random hash based on account id and nonce
        let random_hash = (
            <T as Trait>::Randomness::random_seed(),
            account_id.clone(),
            nonce,
        )
            .using_encoded(<T as system::Trait>::Hashing::hash);

        // Mint the token
        Self::_mint(account_id, &random_hash.into())?;

        // Update nonce
        Nonce::mutate(|n| *n = n.saturating_add(1));

        Ok(random_hash)
    }

    // Mint a new token
    pub fn _mint(to: &T::AccountId, token_id: &T::Hash) -> DispatchResult {
        // Check if token id already minted
        Self::ensure_token_not_existed(token_id)?;

        // Add token id to storage
        Self::add_token_to_all_tokens_enumeration(token_id);
        Self::add_token_to_owner_enumeration(to, token_id);

        <TokenOwner<T>>::insert(token_id, to);
        <OwnedTokensCount<T>>::mutate(to, |value| *value += One::one());

        // Store event
        Self::deposit_event(RawEvent::Transfer(None, Some(to.clone()), *token_id));

        Ok(())
    }

    // Burn a token
    pub fn _burn(sender: &T::AccountId, token_id: &T::Hash) -> DispatchResult {
        // Ensure token existed
        Self::ensure_token_exists(token_id)?;

        // Ensure get token owner
        let owner = Self::ensure_get_token_owner(token_id)?;

        // Ensure sender is token's owner
        if *sender != owner {
            return Err(Error::<T>::NotTokenOwner.into());
        }

        // Remove token id from storage
        Self::remove_token_from_all_tokens_enumeration(token_id);
        Self::remove_token_from_owner_enumeration(&owner, token_id);

        <OwnedTokensIndex<T>>::remove(token_id);
        <TokenApprovals<T>>::remove(token_id);
        <TokenOwner<T>>::remove(token_id);

        // Store event
        Self::deposit_event(RawEvent::Transfer(Some(owner), None, *token_id));

        Ok(())
    }

    // Transfer token from one account to other
    pub fn _transfer_from(
        sender: &T::AccountId,
        from: &T::AccountId,
        to: &T::AccountId,
        token_id: &T::Hash,
    ) -> DispatchResult {
        // Ensure token existed
        Self::ensure_token_exists(token_id)?;

        // Ensure get token owner
        let owner = Self::ensure_get_token_owner(token_id)?;

        // Ensure from account is owner
        if owner != *from {
            return Err(Error::<T>::NotTokenOwner.into());
        }

        // Ensure sender can transfer token
        Self::ensure_approver_or_owner(&sender, token_id)?;

        Self::remove_token_from_owner_enumeration(from, token_id);
        Self::add_token_to_owner_enumeration(to, token_id);

        <TokenApprovals<T>>::remove(token_id);
        <TokenOwner<T>>::insert(token_id, to);

        <OwnedTokensCount<T>>::mutate(from, |value| *value -= One::one());
        <OwnedTokensCount<T>>::mutate(to, |value| *value += One::one());

        Self::deposit_event(RawEvent::Transfer(
            Some(from.clone()),
            Some(to.clone()),
            *token_id,
        ));

        Ok(())
    }

    // Approve a token to an account
    pub fn _approve(
        sender: &T::AccountId,
        to: &T::AccountId,
        token_id: &T::Hash,
    ) -> DispatchResult {
        // Ensure token existed
        Self::ensure_token_exists(token_id)?;

        // Ensure token owner exist
        let owner = Self::ensure_get_token_owner(token_id)?;

        // Unnecessary to approve for self
        if to.clone() == owner {
            return Err(Error::<T>::OwnerAlwaysCanApprove.into());
        }

        // Ensure sender can approve
        Self::ensure_owner_or_approved_for_all(&sender, &owner)?;

        // Update data in storage
        <TokenApprovals<T>>::insert(token_id, to);

        // Stroe the event
        Self::deposit_event(RawEvent::Approval(sender.clone(), to.clone(), *token_id));

        Ok(())
    }

    // Set if an operator can transfer an owner's token
    pub fn _set_approval_for_all(
        sender: &T::AccountId,
        to: &T::AccountId,
        approved: bool,
    ) -> DispatchResult {
        // Unnecessary to approve for self
        if to == sender {
            return Err(Error::<T>::OwnerAlwaysCanApprove.into());
        }

        // Insert operator approval into storage
        <OperatorApprovals<T>>::insert(sender, to, approved);

        // Store the event
        Self::deposit_event(RawEvent::ApprovalForAll(
            sender.clone(),
            to.clone(),
            approved,
        ));

        Ok(())
    }

    // Get all tokens owned by account
    pub fn get_tokens_owned_account(account_id: T::AccountId) -> Vec<T::Hash> {
        <OwnedTokens<T>>::iter_prefix(account_id).collect::<Vec<_>>()
    }

    // Token owner or token approval or owner's delegate
    fn ensure_approver_or_owner(sender: &T::AccountId, token_id: &T::Hash) -> DispatchResult {
        // Get token owner and approver
        let token_owner = Self::owner_of(token_id);
        let approved_or_owner = match token_owner {
            Some(owner) => owner == *sender || Self::is_approved_for_all(owner, sender),
            None => false,
        };

        if approved_or_owner {
            return Ok(());
        }

        // Check if sender is approved user
        let approved_user = Self::get_approved(token_id);
        let approved_as_user = match approved_user {
            Some(user) => user == *sender,
            None => false,
        };

        if approved_as_user {
            Ok(())
        } else {
            Err(Error::<T>::NotOwnerOrApprover.into())
        }
    }

    // Add token index to its owner's enumeration
    fn add_token_to_owner_enumeration(to: &T::AccountId, token_id: &T::Hash) {
        let new_token_index = Self::balance_of(to);
        <OwnedTokensIndex<T>>::insert(token_id, new_token_index);
        <OwnedTokens<T>>::insert(to, new_token_index, token_id);
        <OwnedTokensCount<T>>::mutate(to, |value| *value += One::one());
    }

    // Add token index to all token's enumeration
    fn add_token_to_all_tokens_enumeration(token_id: &T::Hash) {
        let new_token_index = <TotalSupply<T>>::get();
        <AllTokensIndex<T>>::insert(token_id, new_token_index);
        <AllTokens<T>>::insert(new_token_index, token_id);
        <TotalSupply<T>>::mutate(|value| *value += One::one());
    }

    // Remove token info from owner's enumeration
    fn remove_token_from_owner_enumeration(from: &T::AccountId, token_id: &T::Hash) {
        // Sub owner's token count
        <OwnedTokensCount<T>>::mutate(from, |value| *value -= One::one());

        // Get last owned token index
        let last_token_index = <OwnedTokensCount<T>>::get(from);

        // exchange token index and last token index
        let token_index = <OwnedTokensIndex<T>>::get(token_id);

        if token_index != last_token_index {
            let last_token_id = <OwnedTokens<T>>::get(from.clone(), last_token_index);
            <OwnedTokens<T>>::insert(from.clone(), token_index, last_token_id);
            <OwnedTokensIndex<T>>::insert(last_token_id, token_index);
        }

        <OwnedTokens<T>>::remove(from, last_token_index);
        <OwnedTokensIndex<T>>::remove(token_id);
    }

    // Remove token info from all token's enumeration
    fn remove_token_from_all_tokens_enumeration(token_id: &T::Hash) {
        // Update total supply
        <TotalSupply<T>>::mutate(|value| *value -= One::one());

        let last_token_index = <TotalSupply<T>>::get();

        // exchange token index and last token index
        let token_index = <AllTokensIndex<T>>::get(token_id);

        if token_index != last_token_index {
            let last_token_id = <AllTokens<T>>::get(last_token_index);
            <AllTokens<T>>::insert(token_index, last_token_id);
            <AllTokensIndex<T>>::insert(last_token_id, token_index);
        }

        <AllTokens<T>>::remove(last_token_index);
        <AllTokensIndex<T>>::remove(token_id);
    }

    // Ensure get token owner
    fn ensure_get_token_owner(token_id: &T::Hash) -> Result<T::AccountId, DispatchError> {
        match Self::owner_of(token_id) {
            Some(owner) => Ok(owner),
            None => Err(Error::<T>::TokenOwnerNotSet.into()),
        }
    }

    // Ensure sender can transfer owner's token
    fn ensure_owner_or_approved_for_all(
        sender: &T::AccountId,
        owner: &T::AccountId,
    ) -> DispatchResult {
        if sender == owner || Self::is_approved_for_all(sender, owner) {
            Ok(())
        } else {
            Err(Error::<T>::OwnerAlwaysCanApprove.into())
        }
    }

    // Ensure token exists
    pub fn ensure_token_exists(token_id: &T::Hash) -> DispatchResult {
        if <TokenOwner<T>>::exists(token_id) {
            Ok(())
        } else {
            Err(Error::<T>::TokenNotExisted.into())
        }
    }

    // Ensure token not existed
    pub fn ensure_token_not_existed(token_id: &T::Hash) -> DispatchResult {
        if <TokenOwner<T>>::exists(token_id) {
            Err(Error::<T>::TokenAlreadyExists.into())
        } else {
            Ok(())
        }
    }
}
