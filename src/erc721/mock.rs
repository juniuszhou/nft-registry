use super::*;

use node_runtime::constants::currency::*;
use sp_core::H256;
use sp_core::{sr25519, Blake2Hasher};
use sp_runtime::{
    testing::{Digest, DigestItem, Header},
    traits::{BlakeTwo256, Hash, IdentityLookup},
    BuildStorage, Perbill,
};
use support::{
    assert_ok, impl_outer_dispatch, impl_outer_event, impl_outer_origin, parameter_types,
    traits::Currency, weights::Weight,
};

pub mod erc721 {
    pub use super::*;
    use support::impl_outer_event;
}

#[derive(Eq, Clone, PartialEq)]
pub struct ERC721Test;

impl_outer_origin! {
    pub enum Origin for ERC721Test {}
}

impl_outer_event! {
    pub enum MetaEvent for ERC721Test {
        erc721<T>,
    }
}

type BlockNumber = u64;

parameter_types! {
    pub const BlockHashCount: BlockNumber = 250;
    pub const MaximumBlockWeight: Weight = 1_000_000;
    pub const AvailableBlockRatio: Perbill = Perbill::from_percent(75);
    pub const MaximumBlockLength: u32 = 5 * 1024 * 1024;
}

impl system::Trait for ERC721Test {
    type Call = ();
    type AccountId = u64;
    type Lookup = IdentityLookup<Self::AccountId>;
    type Index = u64;
    type BlockNumber = BlockNumber;
    type Hash = H256;
    type Hashing = BlakeTwo256;
    type Header = Header;
    type Event = MetaEvent;
    type Origin = Origin;
    type BlockHashCount = BlockHashCount;
    type MaximumBlockWeight = MaximumBlockWeight;
    type MaximumBlockLength = MaximumBlockLength;
    type AvailableBlockRatio = AvailableBlockRatio;
    type Version = ();
    type ModuleToIndex = ();
}

#[derive(Default)]
pub struct ExtBuilder {}

impl ExtBuilder {
    pub fn build(self) -> sp_io::TestExternalities {
        let t = system::GenesisConfig::default()
            .build_storage::<ERC721Test>()
            .unwrap();

        sp_io::TestExternalities::new(t)
    }
}

pub type RandomnessCollectiveFlip = randomness_collective_flip::Module<ERC721Test>;
pub type Balances = balances::Module<ERC721Test>;
pub type ERC721 = Module<ERC721Test>;
pub type System = system::Module<ERC721Test>;

impl Trait for ERC721Test {
    type Event = MetaEvent;
    type Randomness = RandomnessCollectiveFlip;
    type TokenIndex = u64;
}

pub const ALICE: u64 = 1;
pub const BOB: u64 = 2;
pub const CHARLIE: u64 = 3;
pub const DJANGO: u64 = 4;

pub fn create_token_test(sender: u64) {
    let origin = Origin::signed(sender);

    assert_eq!(ERC721::create_token(origin), Ok(()));

    assert!(<system::Module<ERC721Test>>::events()
        .iter()
        .find(|e| match e.event {
            MetaEvent::erc721(RawEvent::Transfer(None, _, _)) => true,
            _ => false,
        })
        .is_some());
}

pub fn mint_token_test(sender: u64, token_id: H256, result: DispatchResult) {
    let origin = Origin::signed(sender);

    assert_eq!(ERC721::mint(origin, token_id), result);

    if result.is_ok() {
        assert!(<system::Module<ERC721Test>>::events()
            .iter()
            .find(|e| match e.event {
                MetaEvent::erc721(RawEvent::Transfer(None, _, _)) => true,
                _ => false,
            })
            .is_some());
    }
}

pub fn transfer_token_test(
    token_id: H256,
    sender: u64,
    from: u64,
    to: u64,
    result: DispatchResult,
) {
    let origin = Origin::signed(sender);

    assert_eq!(ERC721::transfer_from(origin, from, to, token_id), result);

    if result.is_ok() {
        // Check event
        assert!(<system::Module<ERC721Test>>::events()
            .iter()
            .find(|e| match e.event {
                MetaEvent::erc721(RawEvent::Transfer(_, _, _)) => true,
                _ => false,
            })
            .is_some());

        // Check the new owner of token
        assert_eq!(ERC721::owner_of(token_id).unwrap(), to);
    }
}

pub fn burn_token_test(sender: u64, token_id: H256, result: DispatchResult) {
    let origin = Origin::signed(sender);

    assert_eq!(ERC721::burn(origin, token_id), result);

    if result.is_ok() {
        assert!(<system::Module<ERC721Test>>::events()
            .iter()
            .find(|e| match e.event {
                MetaEvent::erc721(RawEvent::Transfer(_, None, _)) => true,
                _ => false,
            })
            .is_some());
        // Check the new owner of token
        assert_eq!(ERC721::owner_of(token_id), None);
    }
}

pub fn approve_token_test(sender: u64, to_account: u64, token_id: H256, result: DispatchResult) {
    let origin = Origin::signed(sender);

    assert_eq!(ERC721::approve(origin, to_account, token_id), result);

    if result.is_ok() {
        assert!(<system::Module<ERC721Test>>::events()
            .iter()
            .find(|e| match e.event {
                MetaEvent::erc721(RawEvent::Approval(_, _, token_id)) => true,
                _ => false,
            })
            .is_some());
        // Check the new owner of token
        assert_eq!(ERC721::get_approved(token_id).unwrap(), to_account);
    }
}

pub fn approve_for_all_test(sender: u64, to_account: u64, approved: bool, result: DispatchResult) {
    let origin = Origin::signed(sender);

    assert_eq!(
        ERC721::set_approval_for_all(origin, to_account, approved),
        result
    );

    if result.is_ok() {
        assert!(<system::Module<ERC721Test>>::events()
            .iter()
            .find(|e| match e.event {
                MetaEvent::erc721(RawEvent::ApprovalForAll(_, _, approved)) => true,
                _ => false,
            })
            .is_some());
        // Check the new owner of token
        assert_eq!(ERC721::is_approved_for_all(sender, to_account), approved);
    }
}
