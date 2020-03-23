#![cfg(test)]

use super::*;
use crate::mock::*;

#[test]
fn mint_nft_from_basic_contract() {
    ExtBuilder::default().build().execute_with(|| {
        // define all ids
        let account_id = ALICE;
        let token_id: H256 = H256::from_low_u64_be(0);
        let anchor_id: H256 = H256::from_low_u64_be(0);
        let registry_id = 0;

        let triple = get_valid_proof();
        let doc_root = triple.1;

        create_account_test(account_id);
        insert_anchor_test(anchor_id, doc_root);

        let (bytecode, codehash) = get_smart_contract(account_id);
        let contract_address =
            register_validation_fn_test::<NftRegistryTest>(account_id, &bytecode, &codehash);

        create_account_test(contract_address);
        register_validation_test(account_id, contract_address, Ok(()));
        create_nft_test(
            registry_id,
            account_id,
            contract_address,
            token_id,
            anchor_id,
            get_valid_metadata(),
            triple,
            Ok(()),
        );
    });
}

#[test]
fn valiation_function_already_exsits() {
    ExtBuilder::default().build().execute_with(|| {
        // define all ids
        let account_id = ALICE;
        let token_id: H256 = H256::from_low_u64_be(0);
        let anchor_id: H256 = H256::from_low_u64_be(0);
        let registry_id = 0;

        let triple = get_valid_proof();
        let doc_root = triple.1;

        create_account_test(account_id);
        insert_anchor_test(anchor_id, doc_root);

        let (bytecode, codehash) = get_smart_contract(account_id);
        let contract_address =
            register_validation_fn_test::<NftRegistryTest>(account_id, &bytecode, &codehash);

        create_account_test(contract_address);
        register_validation_test(account_id, contract_address, Ok(()));

        register_validation_test(
            account_id,
            contract_address,
            Err(Error::<NftRegistryTest>::ValidationFunctionAlreadyExists.into()),
        );
    });
}

#[test]
fn mint_validation_not_exist() {
    ExtBuilder::default().build().execute_with(|| {
        // define all ids
        let account_id = ALICE;
        let token_id: H256 = H256::from_low_u64_be(0);
        let anchor_id: H256 = H256::from_low_u64_be(0);
        let registry_id = 0;

        let triple = get_valid_proof();
        let doc_root = triple.1;

        create_account_test(account_id);
        insert_anchor_test(anchor_id, doc_root);

        let (bytecode, codehash) = get_smart_contract(account_id);
        let contract_address =
            register_validation_fn_test::<NftRegistryTest>(account_id, &bytecode, &codehash);

        create_account_test(contract_address);
        register_validation_test(account_id, contract_address, Ok(()));
        create_nft_test(
            INVALID_UID,
            account_id,
            contract_address,
            token_id,
            anchor_id,
            get_valid_metadata(),
            triple,
            Err(Error::<NftRegistryTest>::ValidationFunctionNotRegistered.into()),
        );
    });
}

#[test]
fn document_not_anchored() {
    ExtBuilder::default().build().execute_with(|| {
        // define all ids
        let account_id = ALICE;
        let token_id: H256 = H256::from_low_u64_be(0);
        let anchor_id: H256 = H256::from_low_u64_be(0);
        let registry_id = 0;

        let triple = get_valid_proof();
        let doc_root = triple.1;

        create_account_test(account_id);

        let (bytecode, codehash) = get_smart_contract(account_id);
        let contract_address =
            register_validation_fn_test::<NftRegistryTest>(account_id, &bytecode, &codehash);

        create_account_test(contract_address);
        register_validation_test(account_id, contract_address, Ok(()));

        create_nft_test(
            registry_id,
            account_id,
            contract_address,
            token_id,
            anchor_id,
            get_valid_metadata(),
            triple,
            Err(Error::<NftRegistryTest>::DocumentNotAnchored.into()),
        );
    });
}

#[test]
fn invalid_proof() {
    ExtBuilder::default().build().execute_with(|| {
        // define all ids
        let account_id = ALICE;
        let token_id: H256 = H256::from_low_u64_be(0);
        let anchor_id: H256 = H256::from_low_u64_be(0);
        let registry_id = 0;

        let triple = get_invalid_proof();
        let doc_root = triple.1;

        create_account_test(account_id);
        insert_anchor_test(anchor_id, doc_root);

        let (bytecode, codehash) = get_smart_contract(account_id);
        let contract_address =
            register_validation_fn_test::<NftRegistryTest>(account_id, &bytecode, &codehash);

        create_account_test(contract_address);
        register_validation_test(account_id, contract_address, Ok(()));

        create_nft_test(
            registry_id,
            account_id,
            contract_address,
            token_id,
            anchor_id,
            get_valid_metadata(),
            triple,
            Err(Error::<NftRegistryTest>::ProofValidationFailure.into()),
        );
    });
}

#[test]
fn transfer_token_successful() {
    ExtBuilder::default().build().execute_with(|| {
        // define all ids
        let account_id = ALICE;
        let token_id: H256 = H256::from_low_u64_be(0);
        let anchor_id: H256 = H256::from_low_u64_be(0);
        let registry_id = 0;

        let triple = get_valid_proof();
        let doc_root = triple.1;

        create_account_test(account_id);
        create_account_test(DJANGO);
        insert_anchor_test(anchor_id, doc_root);

        let (bytecode, codehash) = get_smart_contract(account_id);
        let contract_address =
            register_validation_fn_test::<NftRegistryTest>(account_id, &bytecode, &codehash);

        create_account_test(contract_address);
        register_validation_test(account_id, contract_address, Ok(()));
        create_nft_test(
            registry_id,
            account_id,
            contract_address,
            token_id,
            anchor_id,
            get_valid_metadata(),
            triple,
            Ok(()),
        );

        transfer_token_test(token_id, account_id, account_id, DJANGO, Ok(()));
    });
}

#[test]
fn burn_token_successful() {
    ExtBuilder::default().build().execute_with(|| {
        // define all ids
        let account_id = ALICE;
        let token_id: H256 = H256::from_low_u64_be(0);
        let anchor_id: H256 = H256::from_low_u64_be(0);
        let registry_id = 0;

        let triple = get_valid_proof();
        let doc_root = triple.1;

        create_account_test(account_id);
        create_account_test(DJANGO);
        insert_anchor_test(anchor_id, doc_root);

        let (bytecode, codehash) = get_smart_contract(account_id);
        let contract_address =
            register_validation_fn_test::<NftRegistryTest>(account_id, &bytecode, &codehash);

        create_account_test(contract_address);
        register_validation_test(account_id, contract_address, Ok(()));
        create_nft_test(
            registry_id,
            account_id,
            contract_address,
            token_id,
            anchor_id,
            get_valid_metadata(),
            triple,
            Ok(()),
        );

        burn_token_test(account_id, token_id, Ok(()));
    });
}

#[test]
fn finish_mint_not_from_contract() {
    ExtBuilder::default().build().execute_with(|| {
        // define all ids
        let account_id = ALICE;
        let token_id: H256 = H256::from_low_u64_be(0);
        let anchor_id: H256 = H256::from_low_u64_be(0);
        let registry_id = 0;

        let triple = get_valid_proof();
        let doc_root = triple.1;

        create_account_test(account_id);
        insert_anchor_test(anchor_id, doc_root);

        let (bytecode, codehash) = get_smart_contract(account_id);
        let contract_address =
            register_validation_fn_test::<NftRegistryTest>(account_id, &bytecode, &codehash);

        create_account_test(contract_address);
        register_validation_test(account_id, contract_address, Ok(()));

        let null_contract_address = NULL_CONTRACT;
        finish_mint_test(
            null_contract_address,
            registry_id,
            token_id,
            account_id,
            get_valid_metadata(),
            Err(Error::<NftRegistryTest>::NotValidationFunction.into()),
        );
    });
}
