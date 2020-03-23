#![cfg(test)]

use super::*;
use mock::*;
use sp_core::H256;

#[test]
fn create_token() {
    ExtBuilder::default().build().execute_with(|| {
        let account_id = ALICE;
        create_token_test(account_id);
    });
}

#[test]
fn transfer_token() {
    ExtBuilder::default().build().execute_with(|| {
        let account_id = ALICE;
        let to_account = BOB;

        let token_id = ERC721::_create_token(&account_id).unwrap();
        transfer_token_test(token_id, account_id, account_id, to_account, Ok(()));
    });
}

#[test]
fn burn_token() {
    ExtBuilder::default().build().execute_with(|| {
        let account_id = ALICE;

        let token_id = ERC721::_create_token(&account_id).unwrap();
        burn_token_test(account_id, token_id, Ok(()));
    });
}

#[test]
fn approve_token() {
    ExtBuilder::default().build().execute_with(|| {
        let account_id = ALICE;
        let to_account = BOB;

        let token_id = ERC721::_create_token(&account_id).unwrap();
        approve_token_test(account_id, to_account, token_id, Ok(()));
    });
}

#[test]
fn approve_for_all() {
    ExtBuilder::default().build().execute_with(|| {
        let account_id = ALICE;
        let to_account = BOB;
        approve_for_all_test(account_id, to_account, true, Ok(()));
        approve_for_all_test(account_id, to_account, false, Ok(()));
    });
}

#[test]
fn approve_then_transfer() {
    ExtBuilder::default().build().execute_with(|| {
        let account_id = ALICE;
        let approver_account = BOB;
        let to_account = CHARLIE;

        let token_id = ERC721::_create_token(&account_id).unwrap();
        approve_token_test(account_id, approver_account, token_id, Ok(()));
        transfer_token_test(token_id, approver_account, account_id, to_account, Ok(()));
    });
}

#[test]
fn approve_for_all_then_transfer() {
    ExtBuilder::default().build().execute_with(|| {
        let account_id = ALICE;
        let approver_account = BOB;
        let to_account = CHARLIE;

        let token_id = ERC721::_create_token(&account_id).unwrap();
        approve_for_all_test(account_id, approver_account, true, Ok(()));
        transfer_token_test(token_id, approver_account, account_id, to_account, Ok(()));
    });
}

#[test]
fn token_already_exists() {
    ExtBuilder::default().build().execute_with(|| {
        let account_id = ALICE;

        let token_id = ERC721::_create_token(&account_id).unwrap();
        mint_token_test(
            account_id,
            token_id,
            Err(Error::<ERC721Test>::TokenAlreadyExists.into()),
        );
    });
}

#[test]
fn token_not_existed() {
    ExtBuilder::default().build().execute_with(|| {
        let account_id = ALICE;
        let to_account = CHARLIE;

        let token_id: H256 = [0 as u8; 32].into();
        transfer_token_test(
            token_id,
            account_id,
            account_id,
            to_account,
            Err(Error::<ERC721Test>::TokenNotExisted.into()),
        );
    });
}

#[test]
fn not_token_owner_trasfer() {
    ExtBuilder::default().build().execute_with(|| {
        let account_id = ALICE;
        let to_account = CHARLIE;

        let token_id = ERC721::_create_token(&account_id).unwrap();
        transfer_token_test(
            token_id,
            account_id,
            to_account,
            to_account,
            Err(Error::<ERC721Test>::NotTokenOwner.into()),
        );
    });
}

#[test]
fn not_token_owner_or_approver_trasfer() {
    ExtBuilder::default().build().execute_with(|| {
        let account_id = ALICE;
        let to_account = CHARLIE;

        let token_id = ERC721::_create_token(&account_id).unwrap();
        transfer_token_test(
            token_id,
            to_account,
            account_id,
            to_account,
            Err(Error::<ERC721Test>::NotOwnerOrApprover.into()),
        );
    });
}

#[test]
fn approve_token_self() {
    ExtBuilder::default().build().execute_with(|| {
        let account_id = ALICE;

        let token_id = ERC721::_create_token(&account_id).unwrap();
        approve_token_test(
            account_id,
            account_id,
            token_id,
            Err(Error::<ERC721Test>::OwnerAlwaysCanApprove.into()),
        );
    });
}
