#[macro_use]
extern crate bencher;

extern crate polymesh_runtime as runtime;

use runtime::identity;
use runtime::balances;
use primitives::{ IdentityId, Key };

use bencher::Bencher;

use codec::{Decode, Encode};
use sr_io::{with_externalities, TestExternalities};
use sr_primitives::{
    testing::Header,
    traits::{BlakeTwo256, ConvertInto, IdentityLookup},
    Perbill,
};
use srml_support::{impl_outer_origin, parameter_types};
use substrate_primitives::{Blake2Hasher, H256};
use std::convert::TryFrom;

impl_outer_origin! {
    pub enum Origin for IdentityTest {}
}

// For testing the module, we construct most of a mock runtime. This means
// first constructing a configuration type (`Test`) which `impl`s each of the
// configuration traits of modules we want to use.
#[derive(Clone, Eq, PartialEq)]
pub struct IdentityTest;

parameter_types! {
    pub const BlockHashCount: u32 = 250;
    pub const MaximumBlockWeight: u32 = 4096;
    pub const MaximumBlockLength: u32 = 4096;
    pub const AvailableBlockRatio: Perbill = Perbill::from_percent(75);
}

impl system::Trait for IdentityTest {
    type Origin = Origin;
    type Index = u64;
    type BlockNumber = u64;
    type Hash = H256;
    type Hashing = BlakeTwo256;
    type AccountId = u64;
    type Lookup = IdentityLookup<Self::AccountId>;
    type Header = Header;
    type Event = ();

    type Call = ();
    type WeightMultiplierUpdate = ();
    type BlockHashCount = BlockHashCount;
    type MaximumBlockWeight = MaximumBlockWeight;
    type MaximumBlockLength = MaximumBlockLength;
    type AvailableBlockRatio = AvailableBlockRatio;
    type Version = ();
}

parameter_types! {
    pub const ExistentialDeposit: u64 = 0;
    pub const TransferFee: u64 = 0;
    pub const CreationFee: u64 = 0;
    pub const TransactionBaseFee: u64 = 0;
    pub const TransactionByteFee: u64 = 0;
}

impl balances::Trait for IdentityTest {
    type Balance = u128;
    type OnFreeBalanceZero = ();
    type OnNewAccount = ();
    type Event = ();
    type TransactionPayment = ();
    type DustRemoval = ();
    type TransferPayment = ();

    type ExistentialDeposit = ExistentialDeposit;
    type TransferFee = TransferFee;
    type CreationFee = CreationFee;
    type TransactionBaseFee = TransactionBaseFee;
    type TransactionByteFee = TransactionByteFee;
    type WeightToFee = ConvertInto;
    type Identity = identity::Module<IdentityTest>;
}

parameter_types! {
    pub const MinimumPeriod: u64 = 3;
}

impl timestamp::Trait for IdentityTest {
    type Moment = u64;
    type OnTimestampSet = ();
    type MinimumPeriod = MinimumPeriod;
}

impl identity::Trait for IdentityTest {
    type Event = ();
}

type Identity = identity::Module<IdentityTest>;


fn a(bench: &mut Bencher) {
    bench.iter(|| {
        (0..1000).fold(0, |x, y| x + y)
    })
}

fn b(bench: &mut Bencher) {
    const N: usize = 1024;
    bench.iter(|| {
        vec![0u8; N]
    });

    bench.bytes = N as u64;
}

/// Create externalities
fn build_ext() -> TestExternalities<Blake2Hasher> {
    system::GenesisConfig::default()
        .build_storage::<IdentityTest>()
        .unwrap()
        .into()
}

fn bench_register_did(b: &mut Bencher) {
    with_externalities(&mut build_ext(), || {
        let owner_id = Identity::owner();
        let owner_id_signed = Origin::signed(owner_id);
        let owner_key = Key::try_from(owner_id.encode()).unwrap();

        let dids = (0..1000)
            .map(|i| IdentityId::from(i as u128))
            .collect::<Vec<_>>();

        b.iter(|| {
            dids.iter().for_each(|did| {
                Identity::register_did(owner_id_signed.clone(), *did, vec![]);
            })
        });
    });
}

benchmark_group!(benches, bench_register_did);
benchmark_main!(benches);
