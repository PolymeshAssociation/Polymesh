use crate::{balances, identity};
use primitives::{IdentityId, Key};

use codec::Encode;
use sr_io::TestExternalities;
use sr_primitives::{
    testing::Header,
    traits::{BlakeTwo256, ConvertInto, IdentityLookup},
    Perbill,
};
use srml_support::{
    dispatch::{DispatchError, DispatchResult},
    impl_outer_origin, parameter_types,
    traits::Currency,
};
use std::convert::TryFrom;
use substrate_primitives::{Blake2Hasher, H256};

impl_outer_origin! {
    pub enum Origin for TestStorage {}
}

// For testing the module, we construct most of a mock runtime. This means
// first constructing a configuration type (`Test`) which `impl`s each of the
// configuration traits of modules we want to use.
#[derive(Clone, Eq, PartialEq)]
pub struct TestStorage;

parameter_types! {
    pub const BlockHashCount: u32 = 250;
    pub const MaximumBlockWeight: u32 = 4096;
    pub const MaximumBlockLength: u32 = 4096;
    pub const AvailableBlockRatio: Perbill = Perbill::from_percent(75);
}

impl system::Trait for TestStorage {
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

impl balances::Trait for TestStorage {
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
    type Identity = crate::identity::Module<TestStorage>;
}

parameter_types! {
    pub const MinimumPeriod: u64 = 3;
}

impl timestamp::Trait for TestStorage {
    type Moment = u64;
    type OnTimestampSet = ();
    type MinimumPeriod = MinimumPeriod;
}

#[derive(codec::Encode, codec::Decode, Debug, Clone, Eq, PartialEq)]
pub struct IdentityProposal {
    pub dummy: u8,
}

impl sr_primitives::traits::Dispatchable for IdentityProposal {
    type Origin = Origin;
    type Trait = TestStorage;
    type Error = DispatchError;

    fn dispatch(self, _origin: Self::Origin) -> DispatchResult<Self::Error> {
        Ok(())
    }
}

impl identity::Trait for TestStorage {
    type Event = ();
    type Proposal = IdentityProposal;
}

// Publish type alias for each module
pub type Identity = identity::Module<TestStorage>;
pub type Balances = balances::Module<TestStorage>;

/// Create externalities
pub fn build_ext() -> TestExternalities<Blake2Hasher> {
    system::GenesisConfig::default()
        .build_storage::<TestStorage>()
        .unwrap()
        .into()
}

/// It creates an Account and registers its DID.
pub fn make_account_with_balance(
    id: u64,
    balance: <TestStorage as balances::Trait>::Balance,
) -> Result<(<TestStorage as system::Trait>::Origin, IdentityId), &'static str> {
    let signed_id = Origin::signed(id);
    Balances::make_free_balance_be(&id, balance);

    Identity::register_did(signed_id.clone(), vec![])?;
    let did = Identity::get_identity(&Key::try_from(id.encode())?).unwrap();

    Ok((signed_id, did))
}
