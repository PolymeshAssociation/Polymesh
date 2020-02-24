use crate::*;

use codec::{Decode, Encode};
use frame_support::{
    assert_ok, impl_outer_origin, parameter_types,
    weights::{GetDispatchInfo, Weight},
};
use sp_core::{
    crypto::Pair,
    offchain::{testing, OffchainExt, TransactionPoolExt},
    testing::KeyStore,
    traits::KeystoreExt,
    H256,
};
use sp_io::TestExternalities;
use sp_runtime::{
    testing::{Header, TestXt},
    traits::{BlakeTwo256, Extrinsic as ExtrinsicsT, IdentityLookup, Verify},
    AnySignature, Perbill, RuntimeAppPublic,
};
use std::cell::RefCell;
use test_client::AccountKeyring;

pub type BlockNumber = u64;
pub type AccountId = <AnySignature as Verify>::Signer;

impl_outer_origin! {
    pub enum Origin for Test  where system = frame_system {}
}

// For testing the module, we construct most of a mock runtime. This means
// first constructing a configuration type (`Test`) which `impl`s each of the
// configuration traits of modules we want to use.
#[derive(Clone, Eq, PartialEq)]
pub struct Test;
parameter_types! {
    pub const BlockHashCount: u64 = 250;
    pub const MaximumBlockWeight: Weight = 1024;
    pub const MaximumBlockLength: u32 = 2 * 1024;
    pub const AvailableBlockRatio: Perbill = Perbill::one();
}
impl frame_system::Trait for Test {
    type Origin = Origin;
    type Index = u64;
    type BlockNumber = BlockNumber;
    type Hash = H256;
    type Call = ();
    type Hashing = BlakeTwo256;
    type AccountId = AccountId;
    type Lookup = IdentityLookup<Self::AccountId>;
    type Header = Header;
    type Event = ();
    type BlockHashCount = BlockHashCount;
    type MaximumBlockWeight = MaximumBlockWeight;
    type MaximumBlockLength = MaximumBlockLength;
    type AvailableBlockRatio = AvailableBlockRatio;
    type Version = ();
    type ModuleToIndex = ();
    type OnReapAccount = ();
    type OnNewAccount = ();
    type AccountData = ();
}

pub type Extrinsic = TestXt<Call<Test>, ()>;
pub type SubmitTransaction =
    frame_system::offchain::TransactionSubmitter<crypto::Public, Test, Extrinsic>;

impl frame_system::offchain::CreateTransaction<Test, Extrinsic> for Test {
    type Public = sp_core::sr25519::Public;
    type Signature = sp_core::sr25519::Signature;

    fn create_transaction<F: frame_system::offchain::Signer<Self::Public, Self::Signature>>(
        call: <Extrinsic as ExtrinsicsT>::Call,
        _public: Self::Public,
        _account: <Test as frame_system::Trait>::AccountId,
        nonce: <Test as frame_system::Trait>::Index,
    ) -> Option<(
        <Extrinsic as ExtrinsicsT>::Call,
        <Extrinsic as ExtrinsicsT>::SignaturePayload,
    )> {
        Some((call, (nonce, ())))
    }
}

pub struct CoolingInterval;
impl Get<u64> for CoolingInterval {
    fn get() -> u64 {
        COOLING_INTERVAL.with(|v| *v.borrow())
    }
}

pub struct BufferInterval;
impl Get<u64> for BufferInterval {
    fn get() -> u64 {
        COOLING_INTERVAL.with(|v| *v.borrow())
    }
}

impl Trait for Test {
    type Event = ();
    type Call = Call<Test>;
    type SubmitSignedTransaction = SubmitTransaction;
    type CoolingInterval = CoolingInterval;
    type BufferInterval = BufferInterval;
}

pub struct ExtBuilder {
    cooling_interval: BlockNumber,
    buffer_interval: BlockNumber,
}

thread_local! {
    static COOLING_INTERVAL: RefCell<BlockNumber> = RefCell::new(0);
    static BUFFER_INTERVAL: RefCell<BlockNumber> = RefCell::new(0);
}

impl Default for ExtBuilder {
    fn default() -> Self {
        Self {
            cooling_interval: 3,
            buffer_interval: 0,
        }
    }
}

impl ExtBuilder {
    pub fn cooling_interval(mut self, cooling_interval: BlockNumber) -> Self {
        self.cooling_interval = cooling_interval;
        self
    }

    pub fn buffer_interval(mut self, buffer_interval: BlockNumber) -> Self {
        self.buffer_interval = buffer_interval;
        self
    }

    pub fn set_associated_consts(&self) {
        COOLING_INTERVAL.with(|v| *v.borrow_mut() = self.cooling_interval);
        BUFFER_INTERVAL.with(|v| *v.borrow_mut() = self.buffer_interval);
    }

    /// Create externalities
    pub fn build(self) -> TestExternalities {
        let mut storage = frame_system::GenesisConfig::default()
            .build_storage::<Test>()
            .unwrap();

        // Identity genesis.
        // identity::GenesisConfig::<Test> {
        //     owner: AccountKeyring::Alice.public().into(),
        //     did_creation_fee: 250,
        // }
        // .assimilate_storage(&mut storage)
        // .unwrap();

        // cdd_offchain_worker genesis.
        let _ = GenesisConfig::<Test> {
            stashIds: vec![
                AccountKeyring::Alice.public(),
                AccountKeyring::Dave.public(),
                AccountKeyring::Bob.public(),
            ],
        }
        .assimilate_storage(&mut storage)
        .unwrap();

        sp_io::TestExternalities::new(storage)
    }
}

pub type System = frame_system::Module<Test>;
pub type CddOffchainWorker = Module<Test>;
