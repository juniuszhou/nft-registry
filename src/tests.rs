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

        create_account_mock(account_id);
        insert_anchor_mock(anchor_id, doc_root);

        let (bytecode, codehash) = get_smart_contract(account_id);
        let contract_address =
            register_validation_fn_mock::<NftRegistryTest>(account_id, &bytecode, &codehash);

        create_account_mock(contract_address);
        register_validation_mock(account_id, contract_address);
        create_nft_mock(
            registry_id,
            account_id,
            contract_address,
            token_id,
            anchor_id,
            get_valid_metadata(),
            true,
            Ok(()),
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

        create_account_mock(account_id);
        insert_anchor_mock(anchor_id, doc_root);

        let (bytecode, codehash) = get_smart_contract(account_id);
        let contract_address =
            register_validation_fn_mock::<NftRegistryTest>(account_id, &bytecode, &codehash);

        create_account_mock(contract_address);
        register_validation_mock(account_id, contract_address);
        create_nft_mock(
            INVALID_UID,
            account_id,
            contract_address,
            token_id,
            anchor_id,
            get_valid_metadata(),
            true,
            Err(Error::<NftRegistryTest>::ValidationFnNotExisted.into()),
        );
    });
}

#[test]
fn mint_token_already_registered() {
    ExtBuilder::default().build().execute_with(|| {
        // define all ids
        let account_id = ALICE;
        let token_id: H256 = H256::from_low_u64_be(0);
        let anchor_id: H256 = H256::from_low_u64_be(0);
        let registry_id = 0;

        let triple = get_valid_proof();
        let doc_root = triple.1;

        create_account_mock(account_id);
        insert_anchor_mock(anchor_id, doc_root);

        let (bytecode, codehash) = get_smart_contract(account_id);
        let contract_address =
            register_validation_fn_mock::<NftRegistryTest>(account_id, &bytecode, &codehash);

        create_account_mock(contract_address);
        register_validation_mock(account_id, contract_address);
        create_nft_mock(
            registry_id,
            account_id,
            contract_address,
            token_id,
            anchor_id,
            get_valid_metadata(),
            true,
            Ok(()),
        );

        create_nft_mock(
            registry_id,
            account_id,
            contract_address,
            token_id,
            anchor_id,
            get_valid_metadata(),
            true,
            Err(Error::<NftRegistryTest>::TokenAlreadyExisted.into()),
        );
    });
}

#[test]
fn mint_metadata_invalid() {
    ExtBuilder::default().build().execute_with(|| {
        // define all ids
        let account_id = ALICE;
        let token_id: H256 = H256::from_low_u64_be(0);
        let anchor_id: H256 = H256::from_low_u64_be(0);
        let registry_id = 0;

        let triple = get_valid_proof();
        let doc_root = triple.1;

        create_account_mock(account_id);
        insert_anchor_mock(anchor_id, doc_root);

        let (bytecode, codehash) = get_smart_contract(account_id);
        let contract_address =
            register_validation_fn_mock::<NftRegistryTest>(account_id, &bytecode, &codehash);

        create_account_mock(contract_address);
        register_validation_mock(account_id, contract_address);

        create_nft_mock(
            registry_id,
            account_id,
            contract_address,
            token_id,
            anchor_id,
            get_invalid_metadata(),
            true,
            Err(Error::<NftRegistryTest>::MetadataLengthInvalid.into()),
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

        create_account_mock(account_id);
        // insert_anchor_mock(anchor_id, doc_root);

        let (bytecode, codehash) = get_smart_contract(account_id);
        let contract_address =
            register_validation_fn_mock::<NftRegistryTest>(account_id, &bytecode, &codehash);

        create_account_mock(contract_address);
        register_validation_mock(account_id, contract_address);

        create_nft_mock(
            registry_id,
            account_id,
            contract_address,
            token_id,
            anchor_id,
            get_valid_metadata(),
            true,
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

        create_account_mock(account_id);
        // insert_anchor_mock(anchor_id, doc_root);

        let (bytecode, codehash) = get_smart_contract(account_id);
        let contract_address =
            register_validation_fn_mock::<NftRegistryTest>(account_id, &bytecode, &codehash);

        create_account_mock(contract_address);
        register_validation_mock(account_id, contract_address);

        create_nft_mock(
            registry_id,
            account_id,
            contract_address,
            token_id,
            anchor_id,
            get_valid_metadata(),
            true,
            Err(Error::<NftRegistryTest>::DocumentNotAnchored.into()),
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

        create_account_mock(account_id);
        create_account_mock(DJANGO);
        insert_anchor_mock(anchor_id, doc_root);

        let (bytecode, codehash) = get_smart_contract(account_id);
        let contract_address =
            register_validation_fn_mock::<NftRegistryTest>(account_id, &bytecode, &codehash);

        create_account_mock(contract_address);
        register_validation_mock(account_id, contract_address);
        create_nft_mock(
            registry_id,
            account_id,
            contract_address,
            token_id,
            anchor_id,
            get_valid_metadata(),
            true,
            Ok(()),
        );

        transfer_token_mock(token_id, account_id, account_id, DJANGO, Ok(()));
    });
}

#[test]
fn transfer_token_invalid_owner() {
    ExtBuilder::default().build().execute_with(|| {
        // define all ids
        let account_id = ALICE;
        let token_id: H256 = H256::from_low_u64_be(0);
        let anchor_id: H256 = H256::from_low_u64_be(0);
        let registry_id = 0;

        let triple = get_valid_proof();
        let doc_root = triple.1;

        create_account_mock(account_id);
        create_account_mock(DJANGO);
        insert_anchor_mock(anchor_id, doc_root);

        let (bytecode, codehash) = get_smart_contract(account_id);
        let contract_address =
            register_validation_fn_mock::<NftRegistryTest>(account_id, &bytecode, &codehash);

        create_account_mock(contract_address);
        register_validation_mock(account_id, contract_address);
        create_nft_mock(
            registry_id,
            account_id,
            contract_address,
            token_id,
            anchor_id,
            get_valid_metadata(),
            true,
            Ok(()),
        );

        transfer_token_mock(
            token_id,
            DJANGO,
            account_id,
            DJANGO,
            Err(Error::<NftRegistryTest>::WrongTokenOwner.into()),
        );
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

        create_account_mock(account_id);
        create_account_mock(DJANGO);
        insert_anchor_mock(anchor_id, doc_root);

        let (bytecode, codehash) = get_smart_contract(account_id);
        let contract_address =
            register_validation_fn_mock::<NftRegistryTest>(account_id, &bytecode, &codehash);

        create_account_mock(contract_address);
        register_validation_mock(account_id, contract_address);
        create_nft_mock(
            registry_id,
            account_id,
            contract_address,
            token_id,
            anchor_id,
            get_valid_metadata(),
            true,
            Ok(()),
        );

        burn_token_mock(account_id, token_id, Ok(()));
    });
}

#[test]
fn burn_token_invalid_owner() {
    ExtBuilder::default().build().execute_with(|| {
        // define all ids
        let account_id = ALICE;
        let token_id: H256 = H256::from_low_u64_be(0);
        let anchor_id: H256 = H256::from_low_u64_be(0);
        let registry_id = 0;

        let triple = get_valid_proof();
        let doc_root = triple.1;

        create_account_mock(account_id);
        create_account_mock(DJANGO);
        insert_anchor_mock(anchor_id, doc_root);

        let (bytecode, codehash) = get_smart_contract(account_id);
        let contract_address =
            register_validation_fn_mock::<NftRegistryTest>(account_id, &bytecode, &codehash);

        create_account_mock(contract_address);
        register_validation_mock(account_id, contract_address);
        create_nft_mock(
            registry_id,
            account_id,
            contract_address,
            token_id,
            anchor_id,
            get_valid_metadata(),
            true,
            Ok(()),
        );

        burn_token_mock(
            DJANGO,
            token_id,
            Err(Error::<NftRegistryTest>::WrongTokenOwner.into()),
        );
    });
}
