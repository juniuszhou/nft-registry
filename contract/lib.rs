#![cfg_attr(not(feature = "std"), no_std)]

use ink_lang as ink;
use ink_lang::contract;
// use ink_types_node_runtime::{calls, AccountIndex, NodeRuntimeTypes};

#[ink::contract(version = "0.1.0")]
mod contract {
    use ink_core::storage;

    /// Defines the storage of your contract.
    /// Add new fields to the below struct in order
    /// to add new static storage fields to your contract.
    #[ink(storage)]
    struct Contract {
        /// Stores a single `bool` value on the storage.
        value: storage::Value<bool>,
    }

    impl Contract {
        /// Constructor that initializes the `bool` value to the given `init_value`.
        #[ink(constructor)]
        fn new(&mut self, init_value: bool) {
            self.value.set(init_value);
        }

        /// Constructor that initializes the `bool` value to `false`.
        ///
        /// Constructors can delegate to other constructors.
        #[ink(constructor)]
        fn default(&mut self) {
            self.new(false)
        }

        /// A message that can be called on instantiated contracts.
        /// This one flips the value of the stored `bool` from `true`
        /// to `false` and vice versa.
        #[ink(message)]
        fn flip(&mut self) {
            *self.value = !self.get();
        }

        /// Simply returns the current value of our `bool`.
        #[ink(message)]
        fn get(&self) -> bool {
            *self.value
        }

        pub fn balance_transfer(&mut self, dest: AccountId, value: Balance) {
            let dest_addr = calls::Address::Id(dest);
            let transfer_call =
                calls::Balances::<NodeRuntimeTypes, AccountIndex>::transfer(dest_addr, value);
            env.dispatch_call(transfer_call);
            env.invoke_runtime();
        }
    }

    /// Unit tests in Rust are normally defined within such a `#[cfg(test)]`
    /// module and test functions are marked with a `#[test]` attribute.
    /// The below code is technically just normal Rust code.
    #[cfg(test)]
    mod tests {
        /// Imports all the definitions from the outer scope so we can use them here.
        use super::*;

        /// We test if the default constructor does its job.
        #[test]
        fn default_works() {
            // Note that even though we defined our `#[ink(constructor)]`
            // above as `&mut self` functions that return nothing we can call
            // them in test code as if they were normal Rust constructors
            // that take no `self` argument but return `Self`.
            let contract = Contract::default();
            assert_eq!(contract.get(), false);
        }

        /// We test a simple use case of our contract.
        #[test]
        fn it_works() {
            let mut contract = Contract::new(false);
            assert_eq!(contract.get(), false);
            contract.flip();
            assert_eq!(contract.get(), true);
        }
    }
}
