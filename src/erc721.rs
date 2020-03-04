#![cfg_attr(not(feature = "std"), no_std)]
use sp_runtime::traits::Hash as HashT;
use support::{
    decl_error, decl_event, decl_module, decl_storage, dispatch::DispatchError,
    dispatch::DispatchResult, ensure, traits::Randomness,
};
use system::ensure_signed;

// Encoding library
use codec::Encode;

pub trait Trait: system::Trait + balances::Trait + contracts::Trait {
    type Event: From<Event<Self>> + Into<<Self as system::Trait>::Event>;

    // Something that provides randomness generation in the runtime.
    type Randomness: Randomness<Self::Hash>;
}

decl_error! {
    pub enum Error for Module<T: Trait> {
        // No owner set for token
        TokenNoOwner,

        // Got an overflow after adding
        Overflow,

        // Got an underflow after subing
        Underflow,
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
        pub OwnedTokensCount get(balance_of): map T::AccountId => u64;

        // Token owner's account id
        pub TokenOwner get(owner_of): map T::Hash => Option<T::AccountId>;

        // Token delegate's account id
        pub TokenApprovals get(get_approved): map T::Hash => Option<T::AccountId>;

        // Account deledate's account id
        pub OperatorApprovals get(is_approved_for_all): map (T::AccountId, T::AccountId) => bool;

        // Total count of minted token
        pub TotalSupply get(total_supply): u64;

        // Map of token index to token id
        pub AllTokens get(token_by_index): map u64 => T::Hash;

        // Last token index
        pub AllTokensIndex: map T::Hash => u64;

        // Map of account and index to token id
        pub OwnedTokens get(token_of_owner_by_index): map (T::AccountId, u64) => T::Hash;

        // Token id to index
        pub OwnedTokensIndex: map T::Hash => u64;

        // Value use to generate random value
        pub Nonce: u64;
    }
}

decl_module! {
    pub struct Module<T: Trait> for enum Call where origin: T::Origin  {
        fn deposit_event() = default;

        // Approve a token to an account
        fn approve(origin, to: T::AccountId, token_id: T::Hash) -> DispatchResult {
            let sender = ensure_signed(origin)?;
            let owner = match Self::owner_of(token_id) {
                Some(c) => c,
                None => return Err(Error::<T>::TokenNoOwner.into()),
            };

            ensure!(to != owner, "Owner is implicitly approved");
            ensure!(sender == owner || Self::is_approved_for_all((owner.clone(), sender.clone())), "You are not allowed to approve for this token");

            <TokenApprovals<T>>::insert(&token_id, &to);

            Self::deposit_event(RawEvent::Approval(owner, to, token_id));

            Ok(())
        }

        // Set if an operator can transfer an owner's token
        fn set_approval_for_all(origin, to: T::AccountId, approved: bool) -> DispatchResult {
            let sender = ensure_signed(origin)?;
            ensure!(to != sender, "You are already implicity approved for your own actions");
            <OperatorApprovals<T>>::insert((sender.clone(), to.clone()), approved);

            Self::deposit_event(RawEvent::ApprovalForAll(sender, to, approved));

            Ok(())
        }

        // transfer_from will transfer to addresses even without a balance
        // fn transfer_from(origin, from: T::AccountId, to: T::AccountId, token_id: T::Hash) -> DispatchResult {
        //     let sender = ensure_signed(origin)?;
        //     ensure!(Self::_is_approved_or_owner(sender, token_id), "You do not own this token");

        //     Self::_transfer_from(from, to, token_id)?;

        //     Ok(())
        // }

        // safe_transfer_from checks that the recieving address has enough balance to satisfy the ExistentialDeposit
        // This is not quite what it does on Ethereum, but in the same spirit...
        // fn safe_transfer_from(origin, from: T::AccountId, to: T::AccountId, token_id: T::Hash) -> DispatchResult {
        //     let to_balance = <balances::Module<T>>::free_balance(&to);
        //     ensure!(!to_balance.is_zero(), "'to' account does not satisfy the `ExistentialDeposit` requirement");

        //     Self::transfer_from(origin, from, to, token_id)?;

        //     Ok(())
        // }
    }
}

impl<T: Trait> Module<T> {
    // Create a new token
    pub(crate) fn create_token(
        account_id: &T::AccountId,
    ) -> sp_std::result::Result<T::Hash, DispatchError> {
        let nonce = Nonce::get();

        // generate random hash based on account id and nonce
        let random_hash = (
            <T as Trait>::Randomness::random_seed(),
            account_id.clone(),
            nonce,
        )
            .using_encoded(<T as system::Trait>::Hashing::hash);

        Self::_mint(account_id, &random_hash.into())?;
        Nonce::mutate(|n| *n += 1);

        Ok(random_hash)
    }

    // If token is existed
    pub(crate) fn _exists(token_id: &T::Hash) -> bool {
        return <TokenOwner<T>>::exists(token_id);
    }

    // Token owner or token approval or owner's delegate
    pub(crate) fn _is_approved_or_owner(spender: &T::AccountId, token_id: &T::Hash) -> bool {
        let owner = Self::owner_of(token_id);
        let approved_user = Self::get_approved(token_id);

        let approved_as_owner = match owner {
            Some(ref o) => o == spender,
            None => false,
        };

        let approved_as_delegate = match owner {
            Some(d) => Self::is_approved_for_all((d, spender.clone())),
            None => false,
        };

        let approved_as_user = match approved_user {
            Some(u) => u == *spender,
            None => false,
        };

        return approved_as_owner || approved_as_user || approved_as_delegate;
    }

    // Mint a new token
    pub(crate) fn _mint(to: &T::AccountId, token_id: &T::Hash) -> DispatchResult {
        ensure!(!Self::_exists(token_id), "Token already exists");

        let balance_of = Self::balance_of(to);

        let new_balance_of = match balance_of.checked_add(1) {
            Some(c) => c,
            None => return Err(Error::<T>::Overflow.into()),
        };

        Self::_add_token_to_all_tokens_enumeration(token_id)?;
        Self::_add_token_to_owner_enumeration(to, token_id)?;

        <TokenOwner<T>>::insert(token_id, &to);
        <OwnedTokensCount<T>>::insert(&to, new_balance_of);

        Self::deposit_event(RawEvent::Transfer(None, Some(to.clone()), *token_id));

        Ok(())
    }

    // Burn a token
    pub(crate) fn _burn(token_id: &T::Hash) -> DispatchResult {
        let owner = match Self::owner_of(token_id) {
            Some(c) => c,
            // None => return Err("No owner for this token"),
            None => return Err(Error::<T>::TokenNoOwner.into()),
        };

        let balance_of = Self::balance_of(&owner);

        let new_balance_of = match balance_of.checked_sub(1) {
            Some(c) => c,
            // None => return Err("Underflow subtracting a token to account balance"),
            None => return Err(Error::<T>::Underflow.into()),
        };

        // Writing to storage begins here
        Self::_remove_token_from_all_tokens_enumeration(token_id)?;
        Self::_remove_token_from_owner_enumeration(&owner, token_id)?;
        <OwnedTokensIndex<T>>::remove(token_id);

        Self::_clear_approval(token_id)?;

        <OwnedTokensCount<T>>::insert(&owner, new_balance_of);
        <TokenOwner<T>>::remove(token_id);

        Self::deposit_event(RawEvent::Transfer(Some(owner), None, *token_id));

        Ok(())
    }

    // Transfer token from one account to other
    pub(crate) fn _transfer_from(
        from: &T::AccountId,
        to: &T::AccountId,
        token_id: &T::Hash,
    ) -> DispatchResult {
        let owner = match Self::owner_of(token_id) {
            Some(c) => c,
            None => return Err(Error::<T>::TokenNoOwner.into()),
        };

        ensure!(owner == *from, "'from' account does not own this token");

        let balance_of_from = Self::balance_of(from);
        let balance_of_to = Self::balance_of(to);

        let new_balance_of_from = match balance_of_from.checked_sub(1) {
            Some(c) => c,
            None => return Err(Error::<T>::Underflow.into()),
        };

        let new_balance_of_to = match balance_of_to.checked_add(1) {
            Some(c) => c,
            None => return Err(Error::<T>::Overflow.into()),
        };

        Self::_remove_token_from_owner_enumeration(from, token_id)?;
        Self::_add_token_to_owner_enumeration(to, token_id)?;

        Self::_clear_approval(token_id)?;
        <OwnedTokensCount<T>>::insert(from, new_balance_of_from);
        <OwnedTokensCount<T>>::insert(to, new_balance_of_to);
        <TokenOwner<T>>::insert(token_id, to);

        Self::deposit_event(RawEvent::Transfer(
            Some(from.clone()),
            Some(to.clone()),
            *token_id,
        ));

        Ok(())
    }

    // Clear one token's delegate
    fn _clear_approval(token_id: &T::Hash) -> DispatchResult {
        <TokenApprovals<T>>::remove(token_id);
        Ok(())
    }

    // Add token index to its owner's enumeration
    fn _add_token_to_owner_enumeration(to: &T::AccountId, token_id: &T::Hash) -> DispatchResult {
        let new_token_index = Self::balance_of(to);
        <OwnedTokensIndex<T>>::insert(token_id, new_token_index);
        <OwnedTokens<T>>::insert((to, new_token_index), token_id);

        Ok(())
    }

    // Add token index to all token's enumeration
    fn _add_token_to_all_tokens_enumeration(token_id: &T::Hash) -> DispatchResult {
        let total_supply = Self::total_supply();

        let new_total_supply = match total_supply.checked_add(1) {
            Some(c) => c,
            None => return Err(Error::<T>::Overflow.into()),
        };

        let new_token_index = total_supply;

        <AllTokensIndex<T>>::insert(token_id, new_token_index);
        <AllTokens<T>>::insert(new_token_index, token_id);
        TotalSupply::put(new_total_supply);

        Ok(())
    }

    // Remove token info from owner
    fn _remove_token_from_owner_enumeration(
        from: &T::AccountId,
        token_id: &T::Hash,
    ) -> DispatchResult {
        let balance_of_from = Self::balance_of(from);
        let last_token_index = match balance_of_from.checked_sub(1) {
            Some(c) => c,
            None => return Err(Error::<T>::Underflow.into()),
        };

        // exchange token index and last token index
        let token_index = <OwnedTokensIndex<T>>::get(token_id);

        if token_index != last_token_index {
            let last_token_id = <OwnedTokens<T>>::get((from.clone(), last_token_index));
            <OwnedTokens<T>>::insert((from.clone(), token_index), last_token_id);
            <OwnedTokensIndex<T>>::insert(last_token_id, token_index);
        }

        <OwnedTokens<T>>::remove((from, last_token_index));
        <OwnedTokensIndex<T>>::remove(token_id);

        Ok(())
    }

    // Remove token info from all tokens and all token index
    fn _remove_token_from_all_tokens_enumeration(token_id: &T::Hash) -> DispatchResult {
        let total_supply = Self::total_supply();
        let new_total_supply = match total_supply.checked_sub(1) {
            Some(c) => c,
            None => return Err(Error::<T>::Underflow.into()),
        };

        let last_token_index = new_total_supply;

        // exchange token index and last token index
        let token_index = <AllTokensIndex<T>>::get(token_id);
        let last_token_id = <AllTokens<T>>::get(last_token_index);

        <AllTokens<T>>::insert(token_index, last_token_id);
        <AllTokensIndex<T>>::insert(last_token_id, token_index);

        <AllTokens<T>>::remove(last_token_index);
        <AllTokensIndex<T>>::remove(token_id);

        TotalSupply::put(new_total_supply);

        Ok(())
    }
}
