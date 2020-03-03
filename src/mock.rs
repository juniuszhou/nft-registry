use super::*;

use contracts::{AccountCounter, ComputeDispatchFee, ContractAddressFor, TrieId, TrieIdGenerator};
use ink_core::env::*;
// use node_primitives::Balance;
use node_runtime::constants::currency::*;
use sp_core::H256;
use sp_core::{sr25519, Blake2Hasher};
use sp_runtime::{
    testing::{Digest, DigestItem, Header},
    traits::{BlakeTwo256, Hash, IdentityLookup},
    BuildStorage, Perbill,
};
use std::cell::RefCell;
use std::time::Instant;
use support::{
    assert_ok, impl_outer_dispatch, impl_outer_event, impl_outer_origin, parameter_types,
    traits::Currency, weights::Weight,
};
pub mod nftregistry {
    // Re-export contents of the root. This basically
    // needs to give a name for the current crate.
    // This hack is required for `impl_outer_event!`.
    pub use super::super::*;
    use support::impl_outer_event;
}

#[derive(Eq, Clone, PartialEq)]
pub struct NftRegistryTest;

impl_outer_origin! {
    pub enum Origin for NftRegistryTest {}
}

impl_outer_dispatch! {
    pub enum Call for NftRegistryTest where origin: Origin {
        balances::Balances,
        contracts::Contract,
        nftregistry::NftRegistry,
    }
}

impl_outer_event! {
    pub enum MetaEvent for NftRegistryTest {
        balances<T>, contracts<T>, nftregistry<T>, erc721<T>,
    }
}

type BlockNumber = u64;

parameter_types! {
    pub const BlockHashCount: BlockNumber = 250;
    pub const MaximumBlockWeight: Weight = 1_000_000;
    pub const AvailableBlockRatio: Perbill = Perbill::from_percent(75);
    pub const MaximumBlockLength: u32 = 5 * 1024 * 1024;
}

impl system::Trait for NftRegistryTest {
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

parameter_types! {
    pub const ExistentialDeposit: u64 = 500;
    pub const TransferFee: u64 = 0;
    pub const CreationFee: u64 = 0;
}

impl balances::Trait for NftRegistryTest {
    type Balance = u64;
    type OnFreeBalanceZero = contracts::Module<NftRegistryTest>;
    type OnNewAccount = ();
    type Event = MetaEvent;
    type DustRemoval = ();
    type TransferPayment = ();
    type ExistentialDeposit = ExistentialDeposit;
    type TransferFee = TransferFee;
    type CreationFee = CreationFee;
}

parameter_types! {
    pub const MinimumPeriod: u64 = 1;
}

impl pallet_timestamp::Trait for NftRegistryTest {
    type Moment = u64;
    type OnTimestampSet = ();
    type MinimumPeriod = MinimumPeriod;
}

impl erc721::Trait for NftRegistryTest {
    type Event = MetaEvent;
    type Randomness = RandomnessCollectiveFlip;
}

impl anchor::Trait for NftRegistryTest {}

parameter_types! {
    pub const SignedClaimHandicap: u32 = 2;
    pub const TombstoneDeposit: u64 = 16;
    pub const StorageSizeOffset: u32 = 8;
    pub const RentByteFee: u64 = 4;
    pub const RentDepositOffset: u64 = 10_000;
    pub const SurchargeReward: u64 = 150;
    pub const TransactionBaseFee: u64 = 2;
    pub const TransactionByteFee: u64 = 6;
    pub const ContractFee: u64 = 21;
    pub const CallBaseFee: u64 = 135;
    pub const InstantiateBaseFee: u64 = 175;
    pub const MaxDepth: u32 = 100;
    pub const MaxValueSize: u32 = 16_384;
    pub const BlockGasLimit: u64 = 100_000;
}

pub struct ExtBuilder {
    existential_deposit: u64,
    gas_price: u64,
    block_gas_limit: u64,
    transfer_fee: u64,
    instantiation_fee: u64,
}
impl Default for ExtBuilder {
    fn default() -> Self {
        Self {
            existential_deposit: 0,
            gas_price: 2,
            block_gas_limit: 100_000_000,
            transfer_fee: 0,
            instantiation_fee: 0,
        }
    }
}
thread_local! {
    static EXISTENTIAL_DEPOSIT: RefCell<u64> = RefCell::new(0);
    static TRANSFER_FEE: RefCell<u64> = RefCell::new(0);
    static INSTANTIATION_FEE: RefCell<u64> = RefCell::new(0);
    static BLOCK_GAS_LIMIT: RefCell<u64> = RefCell::new(0);
}

pub fn create_genesis_config() -> crate::GenesisConfig {
    crate::GenesisConfig {
        min_token_metadata_length: 10,
        max_token_metadata_length: 100,
    }
}

impl ExtBuilder {
    pub fn existential_deposit(mut self, existential_deposit: u64) -> Self {
        self.existential_deposit = existential_deposit;
        self
    }
    pub fn gas_price(mut self, gas_price: u64) -> Self {
        self.gas_price = gas_price;
        self
    }
    pub fn block_gas_limit(mut self, block_gas_limit: u64) -> Self {
        self.block_gas_limit = block_gas_limit;
        self
    }
    pub fn transfer_fee(mut self, transfer_fee: u64) -> Self {
        self.transfer_fee = transfer_fee;
        self
    }
    pub fn instantiation_fee(mut self, instantiation_fee: u64) -> Self {
        self.instantiation_fee = instantiation_fee;
        self
    }
    pub fn set_associated_consts(&self) {
        EXISTENTIAL_DEPOSIT.with(|v| *v.borrow_mut() = self.existential_deposit);
        TRANSFER_FEE.with(|v| *v.borrow_mut() = self.transfer_fee);
        INSTANTIATION_FEE.with(|v| *v.borrow_mut() = self.instantiation_fee);
        BLOCK_GAS_LIMIT.with(|v| *v.borrow_mut() = self.block_gas_limit);
    }
    pub fn build(self) -> sp_io::TestExternalities {
        self.set_associated_consts();
        let mut t = system::GenesisConfig::default()
            // let mut t = create_genesis_config()
            .build_storage::<NftRegistryTest>()
            .unwrap();
        balances::GenesisConfig::<NftRegistryTest> {
            balances: vec![],
            vesting: vec![],
        }
        .assimilate_storage(&mut t)
        .unwrap();
        contracts::GenesisConfig::<NftRegistryTest> {
            current_schedule: contracts::Schedule {
                enable_println: true,
                ..Default::default()
            },
            gas_price: self.gas_price.into(),
        }
        .assimilate_storage(&mut t)
        .unwrap();

        let nft_config = create_genesis_config();
        nft_config.assimilate_storage(&mut t).unwrap();

        sp_io::TestExternalities::new(t)
    }
}

pub type RandomnessCollectiveFlip = randomness_collective_flip::Module<NftRegistryTest>;
pub type Timestamp = pallet_timestamp::Module<NftRegistryTest>;
pub type NftReg = super::Module<NftRegistryTest>;
pub type Balances = balances::Module<NftRegistryTest>;
pub type Contract = contracts::Module<NftRegistryTest>;
pub type NftRegistry = super::Module<NftRegistryTest>;

impl contracts::Trait for NftRegistryTest {
    type Currency = Balances;
    type Time = Timestamp;
    type Randomness = randomness_collective_flip::Module<NftRegistryTest>;
    type Call = Call;
    type Event = MetaEvent;
    type DetermineContractAddress = DummyContractAddressFor;
    type ComputeDispatchFee = DummyComputeDispatchFee;
    type TrieIdGenerator = DummyTrieIdGenerator;
    type GasPayment = ();
    type RentPayment = ();
    type SignedClaimHandicap = SignedClaimHandicap;
    type TombstoneDeposit = TombstoneDeposit;
    type StorageSizeOffset = StorageSizeOffset;
    type RentByteFee = RentByteFee;
    type RentDepositOffset = RentDepositOffset;
    type SurchargeReward = SurchargeReward;
    type TransferFee = TransferFee;
    type CreationFee = CreationFee;
    type TransactionBaseFee = TransactionBaseFee;
    type TransactionByteFee = TransactionByteFee;
    type ContractFee = ContractFee;
    type CallBaseFee = CallBaseFee;
    type InstantiateBaseFee = InstantiateBaseFee;
    type MaxDepth = MaxDepth;
    type MaxValueSize = MaxValueSize;
    type BlockGasLimit = BlockGasLimit;
}

parameter_types! {
    pub const NFTDepositBase: u64 = 1_000 * CENTS as u64;
    pub const NFTDepositPerByte: u64 = 1_000 * CENTS as u64;
}

impl super::Trait for NftRegistryTest {
    type Event = MetaEvent;
    // type Balance = u64;
    type Randomness = RandomnessCollectiveFlip;
    type NFTDepositBase = NFTDepositBase;
    type NFTDepositPerByte = NFTDepositPerByte;
    type Currency = Balances;
}

pub struct DummyContractAddressFor;
impl ContractAddressFor<H256, u64> for DummyContractAddressFor {
    fn contract_address_for(_code_hash: &H256, _data: &[u8], origin: &u64) -> u64 {
        *origin + 1
    }
}

pub struct DummyTrieIdGenerator;
impl TrieIdGenerator<u64> for DummyTrieIdGenerator {
    fn trie_id(account_id: &u64) -> TrieId {
        use sp_core::storage::well_known_keys;

        let new_seed = contracts::AccountCounter::mutate(|v| {
            *v = v.wrapping_add(1);
            *v
        });

        let mut res = vec![];
        res.extend_from_slice(well_known_keys::CHILD_STORAGE_KEY_PREFIX);
        res.extend_from_slice(b"default:");
        res.extend_from_slice(&new_seed.to_le_bytes());
        res.extend_from_slice(&account_id.to_le_bytes());
        res
    }
}

pub struct DummyComputeDispatchFee;
impl ComputeDispatchFee<Call, u64> for DummyComputeDispatchFee {
    fn compute_dispatch_fee(call: &Call) -> u64 {
        69
    }
}

pub const ALICE: u64 = 1;
pub const BOB: u64 = 2;
pub const CHARLIE: u64 = 3;
pub const DJANGO: u64 = 4;
pub const NULL_CONTRACT: u64 = 100;

pub const CODE_DISPATCH_CALL: &str = r#"
(module
    (import "env" "ext_dispatch_call" (func $ext_dispatch_call (param i32 i32)))
    (import "env" "memory" (memory 1 1))
    (func (export "call")
        (call $ext_dispatch_call
            (i32.const 8) ;; Pointer to the start of encoded call buffer
            (i32.const 10) ;; Length of the buffer
        )
    )
    (func (export "deploy"))
    ;; Generated by codec::Encode::encode(&Call::NftRegistry(nftregistry::Call::finish_mint(registry_id))))
    (data (i32.const 8) "\02\02\00\00\00\00\00\00\00\00")
)
"#;

pub fn compile_module<T>(
    wabt_module: &str,
) -> std::result::Result<(Vec<u8>, <T::Hashing as Hash>::Output), wabt::Error>
where
    T: system::Trait,
{
    let wasm = wabt::wat2wasm(wabt_module)?;
    let code_hash = T::Hashing::hash(&wasm);
    Ok((wasm, code_hash))
}

pub fn get_wasm_bytecode() -> std::result::Result<Vec<u8>, &'static str> {
    use std::path::Path;
    use std::{fs::File, io, io::prelude::*};

    // Get the wasm contract byte code from a file
    let mut f =
        File::open(Path::new("./testcontract.wasm")).map_err(|_| "Failed to open contract file")?;
    let mut bytecode = Vec::<u8>::new();
    f.read_to_end(&mut bytecode)
        .map(|_| bytecode)
        .map_err(|_| "Didn't read to end of file")
}

pub fn create_account_mock() {
    Balances::deposit_creating(&ALICE, 100_000_000_000_000_000);
    Balances::deposit_creating(&BOB, 100_000_000_000_000_000);
    Balances::deposit_creating(&CHARLIE, 100_000_000_000_000_000);
    Balances::deposit_creating(&DJANGO, 100_000_000_000_000_000);
}

// Compile smart contract from string
pub fn compile_smart_contract<T>() -> (Vec<u8>, H256)
where
    T: system::Trait,
{
    compile_module::<NftRegistryTest>(CODE_DISPATCH_CALL).unwrap()
}

pub fn register_validation_fn_mock<T>(account_id: u64, bytecode: &Vec<u8>, codehash: &H256) -> u64
where
    T: system::Trait,
{
    let origin = Origin::signed(account_id);
    // Store code on chain
    assert_ok!(<contracts::Module<NftRegistryTest>>::put_code(
        origin.clone(),
        100_000,
        bytecode.clone()
    )
    .and_then(|_| <contracts::Module<NftRegistryTest>>::instantiate(
        origin.clone(),
        1_000,
        100_000,
        *codehash,
        codec::Encode::encode(&account_id)
    )));

    <NftRegistryTest as contracts::Trait>::DetermineContractAddress::contract_address_for(
        &codehash,
        &codec::Encode::encode(&account_id),
        &account_id,
    )
}

pub fn register_validation_mock(account_id: u64, contract_address: u64) {
    let origin = Origin::signed(account_id);

    // Create registry and mint nft
    assert_ok!(NftReg::new_registry(origin.clone(), contract_address));

    // Check event logs to see that validation function registered
    assert!(<system::Module<NftRegistryTest>>::events()
        .iter()
        .find(|e| match e.event {
            MetaEvent::nftregistry(RawEvent::NewRegistry(_, _)) => true,
            _ => false,
        })
        .is_some());
}

pub fn create_nft_mock(registry_id: u64, account_id: u64, result: DispatchResult) {
    let origin = Origin::signed(account_id);
    // Mint a nft
    assert_eq!(
        NftReg::mint(
            origin,
            registry_id,
            vec![],
            b"valid metadata".to_vec(),
            0,
            100_000
        ),
        result
    );

    // Check event logs to see that nft was minted
    assert_eq!(
        <system::Module<NftRegistryTest>>::events()
            .iter()
            .find(|e| match e.event {
                MetaEvent::nftregistry(RawEvent::MintNft(_, _, _)) => true,
                _ => false,
            }).is_some(), 
        result.is_ok(),
    );
}
