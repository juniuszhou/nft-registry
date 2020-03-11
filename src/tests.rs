#![cfg(test)]

use super::*;
use crate::mock::*;

#[test]
fn mint_nft_from_ink_contract() {
    ExtBuilder::default().build().execute_with(|| {
        create_account_mock();
        let account_id = ALICE;
        let token_id: H256 = H256::from_low_u64_be(0);
        let registry_id = 0;

        let (bytecode, codehash) = get_smart_contract(account_id);
    });
}

#[test]
fn mint_nft_from_basic_contract() {
    ExtBuilder::default().build().execute_with(|| {
        create_account_mock();
        let account_id = ALICE;
        let token_id: H256 = H256::from_low_u64_be(0);

        let registry_id = 0;
        let (bytecode, codehash) = compile_smart_contract::<NftRegistryTest>();
        let contract_address =
            register_validation_fn_mock::<NftRegistryTest>(account_id, &bytecode, &codehash);
        register_validation_mock(account_id, contract_address);
        create_nft_mock(registry_id, account_id, token_id, Ok(()));
    });
}

#[test]
fn mint_validation_not_exist() {
    ExtBuilder::default().build().execute_with(|| {
        create_account_mock();
        let account_id = ALICE;
        let registry_id = 0;
        let token_id: H256 = H256::from_low_u64_be(0);

        let contract_address = NULL_CONTRACT;
        create_nft_mock(
            registry_id,
            account_id,
            token_id,
            Err(Error::<NftRegistryTest>::ValidationFnNotExisted.into()),
        );
    });
}
