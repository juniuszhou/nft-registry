#![cfg_attr(not(feature = "std"), no_std)]

use ink_lang as ink;

#[ink::contract(version = "0.1.0", env = NodeRuntimeTypes)]

mod calls {
    use ink_core::env;
    use ink_core::storage;
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

        // Dispatches a validate call to the NFT module
        #[ink(message)]
        fn validate(&self, parameters: Vec<u8>) {
            let decoded =
                ContractParameter::<Hash, AccountId>::decode(&mut &parameters[..]).unwrap();

            if decoded.uid > 0 {
                let mint_call = runtime_calls::finish_mint(
                    decoded.uid,
                    decoded.token_id,
                    decoded.token_owner,
                    decoded.metadata,
                );

                // dispatch the call to the runtime
                let result = self.env().invoke_runtime(&mint_call);

                // report result to console
                // NOTE: println should only be used on a development chain
                env::println(&format!(
                    "Balance transfer invoke_runtime result {:?}",
                    result
                ));
            } else {
                env::println("uid is invalid.");
            }
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;
        use sp_core::crypto::AccountId32;
        use sp_keyring::AccountKeyring;

        #[test]
        fn dispatches_validate_call() {
            let calls = Calls::new();
            let token_owner: AccountId32 = [0 as u8; 32].into();
            let parameters = ContractParameter::<Hash, AccountId> {
                uid: 0,
                token_id: [0 as u8; 32].into(),
                token_owner: token_owner.into(),
                metadata: vec![],
                proof_leaves: vec![],
            };

            let _result = calls.validate(parameters.encode());
        }
    }
}
