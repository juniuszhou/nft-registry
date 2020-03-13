#![cfg_attr(not(feature = "std"), no_std)]

use ink_lang as ink;

#[ink::contract(version = "0.1.0", env = NodeRuntimeTypes)]
// #[ink::contract(version = "0.1.0")]

mod calls {
    use ink_core::env;
    use ink_core::storage;

    // use ink_core::storage::Vec;
    use ink_prelude::vec::Vec;
    use ink_prelude::*;

    use ink_types_node_runtime::{calls as runtime_calls, NodeRuntimeTypes};
    use scale::{Decode, Encode};

    #[derive(Encode, Decode)]
    struct ContractParameter<Hash, AccountId> {
        uid: u64,
        token_id: Hash,
        token_owner: AccountId,
        metadata: Vec<u8>,
        proof_leaves: Vec<Hash>,
    }

    #[derive(Encode, Decode)]
    struct StaticProof<Hash> {
        basic_data_root: Hash,
        zk_data_root: Hash,
        signature_root: Hash,
    }

    #[derive(Encode, Decode)]
    pub struct Proof {
        leaf_hash: Hash,
        sorted_hashes: Vec<Hash>,
    }

    /// This simple dummy contract dispatches substrate runtime calls
    #[ink(storage)]
    struct Calls {
        value: storage::Value<bool>,
    }

    impl Calls {
        #[ink(constructor)]
        fn new(&mut self) {}

        // Dispatches a `transfer` call to the Balances srml module
        #[ink(message)]
        fn validate(&self, parameters: Vec<u8>) {
            let decoded =
                ContractParameter::<Hash, AccountId>::decode(&mut &parameters[..]).unwrap();

            let mint_call = runtime_calls::finish_mint(
                decoded.uid,
                decoded.token_id,
                decoded.token_owner,
                decoded.metadata,
            );

            // dispatch the call to the runtime
            let result = self.env().invoke_runtime(&mint_call);

            // report result to console
            // NOTE: println should only be used on a development chain)
            env::println(&format!(
                "Balance transfer invoke_runtime result {:?}",
                result
            ));
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;
        use sp_keyring::AccountKeyring;

        #[test]
        fn dispatches_balances_call() {
            let calls = Calls::new();
            // let alice = AccountId::from(AccountKeyring::Alice.to_account_id());
            // assert_eq!(calls.env().dispatched_calls().into_iter().count(), 0);
            // calls.balance_transfer(alice, 10000);
            // assert_eq!(calls.env().dispatched_calls().into_iter().count(), 1);
        }
    }
}
