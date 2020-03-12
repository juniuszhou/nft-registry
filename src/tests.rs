#![cfg(test)]

use super::*;
use crate::mock::*;

#[test]
fn mint_nft_from_basic_contract() {
    ExtBuilder::default().build().execute_with(|| {
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
            Ok(()),
        );
    });
}

// #[test]
// fn mint_validation_not_exist() {
//     ExtBuilder::default().build().execute_with(|| {
//         create_account_mock();
//         let account_id = ALICE;
//         let registry_id = 0;
//         let token_id: H256 = H256::from_low_u64_be(0);

//         let contract_address = NULL_CONTRACT;
//         create_nft_mock(
//             registry_id,
//             account_id,
//             token_id,
//             Err(Error::<NftRegistryTest>::ValidationFnNotExisted.into()),
//         );
//     });
// }

// #[test]
// fn transfer_token() {
//     ExtBuilder::default().build().execute_with(|| {
//         create_account_mock();
//         let account_id = ALICE;
//         let to_account = DJANGO;
//         let token_id: H256 = H256::from_low_u64_be(0);

//         let registry_id = 0;
//         let (bytecode, codehash) = get_smart_contract(account_id);
//         let contract_address =
//             register_validation_fn_mock::<NftRegistryTest>(account_id, &bytecode, &codehash);
//         register_validation_mock(account_id, contract_address);
//         create_nft_mock(registry_id, account_id, token_id, Ok(()));
//         transfer_token_mock(token_id, account_id, account_id, to_account, Ok(()));
//     });
// }
