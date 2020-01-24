use crate::{balances, group, identity};
use codec::Encode;
use frame_support::{
    dispatch::DispatchResult, impl_outer_origin, parameter_types, traits::Currency,
};
use frame_system::{self as system, EnsureSignedBy};
use primitives::{IdentityId, Key};
use sp_core::{crypto::Pair as PairTrait, sr25519::Pair, H256};
use sp_io::TestExternalities;
use sp_runtime::{
    testing::Header,
    traits::{BlakeTwo256, IdentityLookup, Verify},
    AnySignature, Perbill,
};
use std::convert::TryFrom;
use test_client::AccountKeyring;

impl_outer_origin! {
    pub enum Origin for TestStorage {}
}

// For testing the module, we construct most of a mock runtime. This means
// first constructing a configuration type (`Test`) which `impl`s each of the
// configuration traits of modules we want to use.
#[derive(Clone, Eq, PartialEq)]
pub struct TestStorage;
type AccountId = <AnySignature as Verify>::Signer;

parameter_types! {
    pub const BlockHashCount: u32 = 250;
    pub const MaximumBlockWeight: u32 = 4096;
    pub const MaximumBlockLength: u32 = 4096;
    pub const AvailableBlockRatio: Perbill = Perbill::from_percent(75);
}

impl frame_system::Trait for TestStorage {
    type Origin = Origin;
    type Index = u64;
    type BlockNumber = u64;
    type Call = ();
    type Hash = H256;
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
    type DustRemoval = ();
    type TransferPayment = ();
    type ExistentialDeposit = ExistentialDeposit;
    type TransferFee = TransferFee;
    type CreationFee = CreationFee;
    type Identity = crate::identity::Module<TestStorage>;
}

parameter_types! {
    pub const MinimumPeriod: u64 = 3;
}

impl pallet_timestamp::Trait for TestStorage {
    type Moment = u64;
    type OnTimestampSet = ();
    type MinimumPeriod = MinimumPeriod;
}

#[derive(codec::Encode, codec::Decode, Debug, Clone, Eq, PartialEq)]
pub struct IdentityProposal {
    pub dummy: u8,
}

impl sp_runtime::traits::Dispatchable for IdentityProposal {
    type Origin = Origin;
    type Trait = TestStorage;

    fn dispatch(self, _origin: Self::Origin) -> DispatchResult {
        Ok(())
    }
}

parameter_types! {
    pub const One: AccountId = AccountId::from(AccountKeyring::Dave);
    pub const Two: AccountId = AccountId::from(AccountKeyring::Dave);
    pub const Three: AccountId = AccountId::from(AccountKeyring::Dave);
    pub const Four: AccountId = AccountId::from(AccountKeyring::Dave);
    pub const Five: AccountId = AccountId::from(AccountKeyring::Dave);
}

impl group::Trait<group::Instance1> for TestStorage {
    type Event = ();
    type AddOrigin = EnsureSignedBy<One, AccountId>;
    type RemoveOrigin = EnsureSignedBy<Two, AccountId>;
    type SwapOrigin = EnsureSignedBy<Three, AccountId>;
    type ResetOrigin = EnsureSignedBy<Four, AccountId>;
    type MembershipInitialized = ();
    type MembershipChanged = ();
}

impl identity::Trait for TestStorage {
    type Event = ();
    type Proposal = IdentityProposal;
    type AcceptTransferTarget = TestStorage;
}

impl crate::asset::AcceptTransfer for TestStorage {
    fn accept_ticker_transfer(_: IdentityId, _: u64) -> DispatchResult {
        Ok(())
    }
    fn accept_token_ownership_transfer(_: IdentityId, _: u64) -> DispatchResult {
        Ok(())
    }
}

// Publish type alias for each module
pub type Identity = identity::Module<TestStorage>;
pub type Balances = balances::Module<TestStorage>;

/// Create externalities
pub fn build_ext() -> TestExternalities {
    let mut storage = frame_system::GenesisConfig::default()
        .build_storage::<TestStorage>()
        .unwrap();

    identity::GenesisConfig::<TestStorage> {
        owner: AccountKeyring::Alice.public().into(),
        did_creation_fee: 250,
    }
    .assimilate_storage(&mut storage)
    .unwrap();

    sp_io::TestExternalities::new(storage)
}

pub fn make_account(
    id: AccountId,
) -> Result<(<TestStorage as frame_system::Trait>::Origin, IdentityId), &'static str> {
    make_account_with_balance(id, 1_000)
}

/// It creates an Account and registers its DID.
pub fn make_account_with_balance(
    id: AccountId,
    balance: <TestStorage as balances::Trait>::Balance,
) -> Result<(<TestStorage as frame_system::Trait>::Origin, IdentityId), &'static str> {
    let signed_id = Origin::signed(id.clone());
    Balances::make_free_balance_be(&id, balance);

    Identity::register_did(signed_id.clone(), vec![]);
    let did = Identity::get_identity(&Key::try_from(id.encode())?).unwrap();

    Ok((signed_id, did))
}

pub fn register_keyring_account(acc: AccountKeyring) -> Result<IdentityId, &'static str> {
    register_keyring_account_with_balance(acc, 10_000)
}

pub fn register_keyring_account_with_balance(
    acc: AccountKeyring,
    balance: <TestStorage as balances::Trait>::Balance,
) -> Result<IdentityId, &'static str> {
    Balances::make_free_balance_be(&acc.public(), balance);

    let acc_pub = acc.public();
    Identity::register_did(Origin::signed(acc_pub.clone()), vec![]);

    let acc_key = Key::from(acc_pub.0);
    let did =
        Identity::get_identity(&acc_key).ok_or_else(|| "Key cannot be generated from account")?;

    Ok(did)
}

pub fn account_from(id: u64) -> AccountId {
    let mut enc_id_vec = id.encode();
    enc_id_vec.resize_with(32, Default::default);

    let mut enc_id = [0u8; 32];
    enc_id.copy_from_slice(enc_id_vec.as_slice());

    Pair::from_seed(&enc_id).public()
}
