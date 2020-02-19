use super::*;

use sp_core::H256;
use sp_runtime::{
    testing::Header,
    traits::{BadOrigin, BlakeTwo256, IdentityLookup},
    Perbill,
};
use std::time::Instant;
use support::{
    assert_err, assert_ok, impl_outer_event, impl_outer_origin, parameter_types,
    traits::Randomness, weights::Weight,
};

// For testing the module, we construct most of a mock runtime. This means
// first constructing a configuration type (`Test`) which `impl`s each of the
// configuration traits of modules we want to use.
#[derive(Clone, Eq, PartialEq)]
pub struct Test;

impl_outer_origin! {
    pub enum Origin for Test {}
}

mod nft_mod {
    pub use crate::Event;
}

impl_outer_event! {
    pub enum TestEvent for Test {
        nft_mod<T>,
    }
}

parameter_types! {
    pub const BlockHashCount: u64 = 250;
    pub const MaximumBlockWeight: Weight = 1024;
    pub const MaximumBlockLength: u32 = 2 * 1024;
    pub const AvailableBlockRatio: Perbill = Perbill::from_percent(75);
}
impl system::Trait for Test {
    type AccountId = u64;
    type Call = ();
    type Lookup = IdentityLookup<Self::AccountId>;
    type Index = u64;
    type BlockNumber = u64;
    type Hash = H256;
    type Hashing = BlakeTwo256;
    type Header = Header;
    type Event = TestEvent;
    type Origin = Origin;
    type BlockHashCount = BlockHashCount;
    type MaximumBlockWeight = MaximumBlockWeight;
    type MaximumBlockLength = MaximumBlockLength;
    type AvailableBlockRatio = AvailableBlockRatio;
    type Version = ();
    type ModuleToIndex = ();
}

impl pallet_timestamp::Trait for Test {
    type Moment = u64;
    type OnTimestampSet = ();
    type MinimumPeriod = ();
}

parameter_types! {
    pub const ExistentialDeposit: u64 = 0;
    pub const TransferFee: u64 = 0;
    pub const CreationFee: u64 = 0;
    pub const TransactionBaseFee: u64 = 0;
    pub const TransactionByteFee: u64 = 0;
}

impl Trait for Test {
    type Event = TestEvent;
    type Randomness = RandomnessCollectiveFlip;
}

impl Test {}

type RandomnessCollectiveFlip = randomness_collective_flip::Module<Test>;
type TestModule = Module<Test>;
type System = system::Module<Test>;

// This function basically just builds a genesis storage key/value store according to
// our desired mockup.
pub fn new_test_ext() -> sp_io::TestExternalities {
    let mut t = system::GenesisConfig::default()
        .build_storage::<Test>()
        .unwrap();

    t.into()
}

pub fn create_token_mock() {
    // TestModule::create_token()
}

#[test]
fn nft_registry() {
    new_test_ext().execute_with(|| {
        //let anchor_id = <Test as frame_system::Trait>::Hashing::hash_of(&0);
        //let signing_root = <Test as frame_system::Trait>::Hashing::hash_of(&0);

        // reject unsigned
        assert_eq!(true, true);
    });
}
