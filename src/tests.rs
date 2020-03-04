#![cfg(test)]

use super::*;
use crate::mock::*;

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

#[test]
fn mint_nft_from_ink_contract() {
    ExtBuilder::default().build().execute_with(|| {
        // Balances::deposit_creating(&ALICE, 100_000_000);
        // let origin = Origin::signed(ALICE);
        // let registry_id = 0;

        // // Get wasm bytecode
        // let bytecode = get_wasm_bytecode().unwrap();
        // let codehash = <NftRegistryTest as system::Trait>::Hashing::hash(&bytecode);

        // // Store and instantiate contract
        // assert_ok!(<contracts::Module<NftRegistryTest>>::put_code(
        //     origin.clone(),
        //     100_000,
        //     bytecode
        // )
        // .and_then(|_| <contracts::Module<NftRegistryTest>>::instantiate(
        //     origin.clone(),
        //     1_000,
        //     100_000,
        //     codehash,
        //     codec::Encode::encode(&ALICE)
        // )));

        // Call validation contract method
        //let mut call = CallData::new( Selector::from_str("validate") );
        //let mut call = CallData::new( Selector::from_str("finish_mint") );
        //call.push_arg(&codec::Encode::encode(&2));
        //call.push_arg(&2);
        //call.push_arg(&registry_id);

        //let encoded = Encode::encode(&Call::Balances(pallet_balances::Call::transfer(CHARLIE, 50)));
        // println!(
        //     "ENCODED: {:?}",
        //     codec::Encode::encode(&Call::NftRegistry(nftregistry::Call::finish_mint(
        //         registry_id
        //     )))
        // );
        // let call = codec::Encode::encode(&Call::NftRegistry(nftregistry::Call::finish_mint(
        //     registry_id,
        // )));

        /*
        use codec::Encode;
        let keccak = ink_utils::hash::keccak256("finish_mint".as_bytes());
        let selector = [keccak[3], keccak[2], keccak[1], keccak[0]];
        let mut call = selector.encode();
        call.append( &mut Encode::encode(&registry_id) );
        */

        // let addr =
        //     <NftRegistryTest as contracts::Trait>::DetermineContractAddress::contract_address_for(
        //         &codehash,
        //         &codec::Encode::encode(&ALICE),
        //         &ALICE,
        //     );

        // println!("Contract address: {:?}", addr);

        // assert_ok!(NftReg::new_registry(origin.clone(), addr));

        //println!("Call: {:?}", call);

        // assert_ok!(
        //NftReg::mint(origin, registry_id, codec::Encode::encode(&call.to_bytes().to_vec()), 0, 100_000)
        //     NftReg::mint(origin, registry_id, vec![], 0, 100_000)
        // );
        /*
        let res = <contracts::Module<NftRegistryTest>>::bare_call(
            ALICE,
            addr,
            0,
            100_000,
            call);
            //codec::Encode::encode(&call.to_bytes().to_vec()));
            //call.to_bytes().to_vec());
            //vec![]);
        */

        //println!("Call result: {:?}", res.ok().map(|r| (r.is_success(), r.data)));
        //println!("Call result: {:?}", res.err().map(|e| (e.reason, e.buffer)));
        /*
        println!("Call: {:?}", call);
        assert_ok!(
            Contract::call(
                Origin::signed(ALICE),
                addr,
                0,
                100_000,
                //call)
                call.to_bytes().to_vec())
                //selector.to_vec())
        );
        */

        // println!("Event log:");
        // for e in &<system::Module<NftRegistryTest>>::events() {
        //     println!("{:?}", e);
        // }
    });
}

// This function basically just builds a genesis storage key/value store according to
// our desired mockup.
// pub fn new_test_ext() -> sp_io::TestExternalities {
//     let mut t = system::GenesisConfig::default()
//         .build_storage::<Test>()
//         .unwrap();

//     t.into()
// }

// pub fn create_token_mock() {
//     // TestModule::create_token()
// }

// #[test]
// fn nft_registry() {
//     new_test_ext().execute_with(|| {
//         //let anchor_id = <Test as frame_system::Trait>::Hashing::hash_of(&0);
//         //let signing_root = <Test as frame_system::Trait>::Hashing::hash_of(&0);

//         // reject unsigned
//         assert_eq!(true, true);
//     });
// }
