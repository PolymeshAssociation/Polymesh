use polymesh_runtime::{
    asset, bridge, cdd_check::CddChecker, dividend, exemption, general_tm, multisig, percentage_tm,
    simple_token, statistics, voting,
};

use codec::Encode;
use frame_support::{
    assert_ok, dispatch::DispatchResult, impl_outer_dispatch, impl_outer_event, impl_outer_origin,
    parameter_types, traits::Currency, weights::DispatchInfo,
};
use frame_system::{self as system};
use pallet_committee as committee;
use pallet_mips as mips;
use polymesh_primitives::{AccountKey, AuthorizationData, IdentityId, Signatory};
use polymesh_protocol_fee as protocol_fee;
use polymesh_runtime_balances as balances;
use polymesh_runtime_common::traits::{
    asset::AcceptTransfer, balances::AccountData, group::GroupTrait, multisig::AddSignerMultiSig,
    CommonTrait,
};
use polymesh_runtime_group as group;
use polymesh_runtime_identity as identity;
use sp_core::{
    crypto::{key_types, Pair as PairTrait},
    sr25519::{Pair, Public},
    H256,
};
use sp_runtime::{
    impl_opaque_keys,
    testing::{Header, UintAuthorityId},
    traits::{BlakeTwo256, ConvertInto, IdentityLookup, OpaqueKeys, Verify},
    transaction_validity::{InvalidTransaction, TransactionValidity, ValidTransaction},
    AnySignature, KeyTypeId, Perbill,
};
use std::cell::RefCell;
use std::convert::TryFrom;
use test_client::AccountKeyring;

impl_opaque_keys! {
    pub struct MockSessionKeys {
        pub dummy: UintAuthorityId,
    }
}

impl From<UintAuthorityId> for MockSessionKeys {
    fn from(dummy: UintAuthorityId) -> Self {
        Self { dummy }
    }
}

impl_outer_origin! {
    pub enum Origin for TestStorage {}
}

impl_outer_dispatch! {
    pub enum Call for TestStorage where origin: Origin {
        identity::Identity,
        mips::Mips,
        multisig::MultiSig,
        pallet_contracts::Contracts,
        bridge::Bridge,
        asset::Asset,
    }
}

impl_outer_event! {
    pub enum EventTest for TestStorage {
        identity<T>,
        balances<T>,
        multisig<T>,
        percentage_tm<T>,
        bridge<T>,
        asset<T>,
        mips<T>,
        pallet_contracts<T>,
        pallet_session,
        general_tm,
        exemption,
        group Instance1<T>,
        group Instance2<T>,
        group DefaultInstance<T>,
        committee Instance1<T>,
        committee DefaultInstance<T>,
        voting<T>,
        dividend<T>,
        simple_token<T>,
        frame_system<T>,
        protocol_fee<T>,
    }
}

// For testing the module, we construct most of a mock runtime. This means
// first constructing a configuration type (`Test`) which `impl`s each of the
// configuration traits of modules we want to use.
#[derive(Clone, Eq, PartialEq)]
pub struct TestStorage;

type AccountId = <AnySignature as Verify>::Signer;
type Index = u64;
type BlockNumber = u64;
type Hash = H256;
type Hashing = BlakeTwo256;
type Lookup = IdentityLookup<AccountId>;
type OffChainSignature = AnySignature;
type SessionIndex = u32;
type AuthorityId = <AnySignature as Verify>::Signer;
type Event = EventTest;
type Version = ();

parameter_types! {
    pub const BlockHashCount: u32 = 250;
    pub const MaximumBlockWeight: u32 = 4096;
    pub const MaximumBlockLength: u32 = 4096;
    pub const AvailableBlockRatio: Perbill = Perbill::from_percent(75);
}

impl frame_system::Trait for TestStorage {
    type AccountId = AccountId;
    type Call = Call;
    type Lookup = Lookup;
    type Index = Index;
    type BlockNumber = BlockNumber;
    type Hash = Hash;
    type Hashing = Hashing;
    type Header = Header;
    type Event = Event;
    type Origin = Origin;
    type BlockHashCount = BlockHashCount;
    type MaximumBlockWeight = MaximumBlockWeight;
    type MaximumBlockLength = MaximumBlockLength;
    type AvailableBlockRatio = AvailableBlockRatio;
    type Version = Version;
    type ModuleToIndex = ();
    type OnNewAccount = ();
    type OnKilledAccount = ();
    type AccountData = AccountData<<TestStorage as CommonTrait>::Balance>;
}

parameter_types! {
    pub const ExistentialDeposit: u64 = 0;
    pub const TransactionBaseFee: u64 = 0;
    pub const TransactionByteFee: u64 = 0;
}

impl CommonTrait for TestStorage {
    type Balance = u128;
    type AcceptTransferTarget = TestStorage;
    type BlockRewardsReserve = balances::Module<TestStorage>;
}

impl balances::Trait for TestStorage {
    type DustRemoval = ();
    type Event = Event;
    type ExistentialDeposit = ExistentialDeposit;
    type AccountStore = frame_system::Module<TestStorage>;
    type Identity = identity::Module<TestStorage>;
    type CddChecker = CddChecker;
}

parameter_types! {
    pub const MinimumPeriod: u64 = 3;
}

impl pallet_timestamp::Trait for TestStorage {
    type Moment = u64;
    type OnTimestampSet = ();
    type MinimumPeriod = MinimumPeriod;
}

impl multisig::Trait for TestStorage {
    type Event = Event;
}

impl simple_token::Trait for TestStorage {
    type Event = Event;
}

impl pallet_transaction_payment::ChargeTxFee for TestStorage {
    fn charge_fee(_len: u32, _info: DispatchInfo) -> TransactionValidity {
        Ok(ValidTransaction::default())
    }
}

impl pallet_transaction_payment::CddAndFeeDetails<Call> for TestStorage {
    fn get_valid_payer(_: &Call, _: &Signatory) -> Result<Option<Signatory>, InvalidTransaction> {
        Ok(None)
    }
    fn clear_context() {}
    fn set_payer_context(_: Option<Signatory>) {}
    fn get_payer_from_context() -> Option<Signatory> {
        None
    }
    fn set_current_identity(_: &IdentityId) {}
}

parameter_types! {
    pub const One: AccountId = AccountId::from(AccountKeyring::Dave);
    pub const Two: AccountId = AccountId::from(AccountKeyring::Dave);
    pub const Three: AccountId = AccountId::from(AccountKeyring::Dave);
    pub const Four: AccountId = AccountId::from(AccountKeyring::Dave);
    pub const Five: AccountId = AccountId::from(AccountKeyring::Dave);
}

impl group::Trait<group::DefaultInstance> for TestStorage {
    type Event = Event;
    type AddOrigin = frame_system::EnsureRoot<AccountId>;
    type RemoveOrigin = frame_system::EnsureRoot<AccountId>;
    type SwapOrigin = frame_system::EnsureRoot<AccountId>;
    type ResetOrigin = frame_system::EnsureRoot<AccountId>;
    type PrimeOrigin = frame_system::EnsureRoot<AccountId>;
    type MembershipInitialized = committee::Module<TestStorage, committee::Instance1>;
    type MembershipChanged = committee::Module<TestStorage, committee::Instance1>;
}

/// PolymeshCommittee as an instance of group
impl group::Trait<group::Instance1> for TestStorage {
    type Event = Event;
    type AddOrigin = frame_system::EnsureRoot<AccountId>;
    type RemoveOrigin = frame_system::EnsureRoot<AccountId>;
    type SwapOrigin = frame_system::EnsureRoot<AccountId>;
    type ResetOrigin = frame_system::EnsureRoot<AccountId>;
    type PrimeOrigin = frame_system::EnsureRoot<AccountId>;
    type MembershipInitialized = committee::Module<TestStorage, committee::Instance1>;
    type MembershipChanged = committee::Module<TestStorage, committee::Instance1>;
}

impl group::Trait<group::Instance2> for TestStorage {
    type Event = Event;
    type AddOrigin = frame_system::EnsureRoot<AccountId>;
    type RemoveOrigin = frame_system::EnsureRoot<AccountId>;
    type SwapOrigin = frame_system::EnsureRoot<AccountId>;
    type ResetOrigin = frame_system::EnsureRoot<AccountId>;
    type PrimeOrigin = frame_system::EnsureRoot<AccountId>;
    type MembershipInitialized = identity::Module<TestStorage>;
    type MembershipChanged = identity::Module<TestStorage>;
}

pub type CommitteeOrigin<T, I> = committee::RawOrigin<<T as system::Trait>::AccountId, I>;

impl<I> From<CommitteeOrigin<TestStorage, I>> for Origin {
    fn from(_co: CommitteeOrigin<TestStorage, I>) -> Origin {
        Origin::system(frame_system::RawOrigin::Root)
    }
}

parameter_types! {
    pub const CommitteeRoot: AccountId = AccountId::from(AccountKeyring::Alice);
    pub const MotionDuration: BlockNumber = 0u64;
}

impl committee::Trait<committee::Instance1> for TestStorage {
    type Origin = Origin;
    type Proposal = Call;
    type CommitteeOrigin = frame_system::EnsureRoot<AccountId>;
    type Event = Event;
    type MotionDuration = MotionDuration;
}

impl committee::Trait<committee::DefaultInstance> for TestStorage {
    type Origin = Origin;
    type Proposal = Call;
    type CommitteeOrigin = frame_system::EnsureRoot<AccountId>;
    type Event = Event;
    type MotionDuration = MotionDuration;
}

impl identity::Trait for TestStorage {
    type Event = Event;
    type Proposal = Call;
    type AddSignerMultiSigTarget = TestStorage;
    type CddServiceProviders = group::Module<TestStorage, group::Instance2>;
    type Balances = balances::Module<TestStorage>;
    type ChargeTxFeeTarget = TestStorage;
    type CddHandler = TestStorage;
    type Public = AccountId;
    type OffChainSignature = OffChainSignature;
    type ProtocolFee = protocol_fee::Module<TestStorage>;
}

impl AddSignerMultiSig for TestStorage {
    fn accept_multisig_signer(_: Signatory, _: u64) -> DispatchResult {
        unimplemented!()
    }
}

impl AcceptTransfer for TestStorage {
    fn accept_ticker_transfer(_: IdentityId, _: u64) -> DispatchResult {
        Ok(())
    }
    fn accept_token_ownership_transfer(_: IdentityId, _: u64) -> DispatchResult {
        Ok(())
    }
}

parameter_types! {
    pub const SignedClaimHandicap: u64 = 2;
    pub const TombstoneDeposit: u64 = 16;
    pub const StorageSizeOffset: u32 = 8;
    pub const RentByteFee: u64 = 4;
    pub const RentDepositOffset: u64 = 10_000;
    pub const SurchargeReward: u64 = 150;
    pub const ContractTransactionBaseFee: u64 = 2;
    pub const ContractTransactionByteFee: u64 = 6;
    pub const ContractFee: u64 = 21;
    pub const CallBaseFee: u64 = 135;
    pub const InstantiateBaseFee: u64 = 175;
    pub const MaxDepth: u32 = 100;
    pub const MaxValueSize: u32 = 16_384;
    pub const ContractTransferFee: u64 = 50000;
    pub const ContractCreationFee: u64 = 50;
    pub const BlockGasLimit: u64 = 10000000;
}

impl pallet_contracts::Trait for TestStorage {
    type Currency = Balances;
    type Time = Timestamp;
    type Randomness = Randomness;
    type Call = Call;
    type Event = Event;
    type DetermineContractAddress = pallet_contracts::SimpleAddressDeterminer<TestStorage>;
    type ComputeDispatchFee = pallet_contracts::DefaultDispatchFeeComputor<TestStorage>;
    type TrieIdGenerator = pallet_contracts::TrieIdFromParentCounter<TestStorage>;
    type GasPayment = ();
    type RentPayment = ();
    type SignedClaimHandicap = SignedClaimHandicap;
    type TombstoneDeposit = TombstoneDeposit;
    type StorageSizeOffset = StorageSizeOffset;
    type RentByteFee = RentByteFee;
    type RentDepositOffset = RentDepositOffset;
    type SurchargeReward = SurchargeReward;
    type TransactionBaseFee = ContractTransactionBaseFee;
    type TransactionByteFee = ContractTransactionByteFee;
    type ContractFee = ContractFee;
    type CallBaseFee = CallBaseFee;
    type InstantiateBaseFee = InstantiateBaseFee;
    type MaxDepth = MaxDepth;
    type MaxValueSize = MaxValueSize;
    type BlockGasLimit = BlockGasLimit;
}

impl statistics::Trait for TestStorage {}

impl percentage_tm::Trait for TestStorage {
    type Event = Event;
}

impl general_tm::Trait for TestStorage {
    type Event = Event;
    type Asset = asset::Module<TestStorage>;
}

impl protocol_fee::Trait for TestStorage {
    type Event = Event;
    type Currency = Balances;
    type OnProtocolFeePayment = ();
}

impl asset::Trait for TestStorage {
    type Event = Event;
    type Currency = balances::Module<TestStorage>;
}

parameter_types! {
    pub const MaxTimelockedTxsPerBlock: u32 = 10;
    pub const BlockRangeForTimelock: BlockNumber = 1000;
}

impl bridge::Trait for TestStorage {
    type Event = Event;
    type Proposal = Call;
    type MaxTimelockedTxsPerBlock = MaxTimelockedTxsPerBlock;
    type BlockRangeForTimelock = BlockRangeForTimelock;
}

impl exemption::Trait for TestStorage {
    type Event = Event;
    type Asset = asset::Module<TestStorage>;
}

impl voting::Trait for TestStorage {
    type Event = Event;
    type Asset = asset::Module<TestStorage>;
}

thread_local! {
    pub static FORCE_SESSION_END: RefCell<bool> = RefCell::new(false);
    pub static SESSION_LENGTH: RefCell<u64> = RefCell::new(2);
}

pub struct TestSessionHandler;
impl pallet_session::SessionHandler<AuthorityId> for TestSessionHandler {
    const KEY_TYPE_IDS: &'static [KeyTypeId] = &[key_types::DUMMY];

    fn on_new_session<Ks: OpaqueKeys>(
        _changed: bool,
        _validators: &[(AuthorityId, Ks)],
        _queued_validators: &[(AuthorityId, Ks)],
    ) {
    }

    fn on_disabled(_validator_index: usize) {}

    fn on_genesis_session<Ks: OpaqueKeys>(_validators: &[(AuthorityId, Ks)]) {}
    fn on_before_session_ending() {}
}

pub struct TestShouldEndSession;
impl pallet_session::ShouldEndSession<BlockNumber> for TestShouldEndSession {
    fn should_end_session(now: BlockNumber) -> bool {
        let l = SESSION_LENGTH.with(|l| *l.borrow());
        now % l == 0
            || FORCE_SESSION_END.with(|l| {
                let r = *l.borrow();
                *l.borrow_mut() = false;
                r
            })
    }
}

pub struct TestSessionManager;
impl pallet_session::SessionManager<AccountId> for TestSessionManager {
    fn end_session(_: SessionIndex) {}
    fn start_session(_: SessionIndex) {}
    fn new_session(_: SessionIndex) -> Option<Vec<AccountId>> {
        None
    }
}

parameter_types! {
    pub const DisabledValidatorsThreshold: Perbill = Perbill::from_percent(33);
}

impl pallet_session::Trait for TestStorage {
    type Event = Event;
    type ValidatorId = AccountId;
    type ValidatorIdOf = ConvertInto;
    type ShouldEndSession = TestShouldEndSession;
    type SessionManager = TestSessionManager;
    type SessionHandler = TestSessionHandler;
    type Keys = MockSessionKeys;
    type DisabledValidatorsThreshold = DisabledValidatorsThreshold;
}

impl dividend::Trait for TestStorage {
    type Event = Event;
}

impl mips::Trait for TestStorage {
    type Currency = balances::Module<Self>;
    type CommitteeOrigin = frame_system::EnsureRoot<AccountId>;
    type VotingMajorityOrigin = frame_system::EnsureRoot<AccountId>;
    type GovernanceCommittee = Committee;
    type Event = Event;
}

// Publish type alias for each module
pub type Identity = identity::Module<TestStorage>;
pub type Mips = mips::Module<TestStorage>;
pub type Balances = balances::Module<TestStorage>;
pub type Asset = asset::Module<TestStorage>;
pub type MultiSig = multisig::Module<TestStorage>;
pub type Randomness = pallet_randomness_collective_flip::Module<TestStorage>;
pub type Timestamp = pallet_timestamp::Module<TestStorage>;
pub type Contracts = pallet_contracts::Module<TestStorage>;
pub type Bridge = bridge::Module<TestStorage>;
pub type GovernanceCommittee = group::Module<TestStorage, group::Instance1>;
pub type CddServiceProvider = group::Module<TestStorage, group::Instance2>;
pub type Committee = committee::Module<TestStorage, committee::Instance1>;

pub fn make_account(
    id: AccountId,
) -> Result<(<TestStorage as frame_system::Trait>::Origin, IdentityId), &'static str> {
    make_account_with_balance(id, 1_000_000)
}

/// It creates an Account and registers its DID.
pub fn make_account_with_balance(
    id: AccountId,
    balance: <TestStorage as CommonTrait>::Balance,
) -> Result<(<TestStorage as frame_system::Trait>::Origin, IdentityId), &'static str> {
    let signed_id = Origin::signed(id.clone());
    Balances::make_free_balance_be(&id, balance);

    // If we have CDD providers, first of them executes the registration.
    let cdd_providers = CddServiceProvider::get_members();
    let did_registration = if let Some(cdd_provider) = cdd_providers.into_iter().nth(0) {
        let cdd_acc = Public::from_raw(Identity::did_records(&cdd_provider).master_key.0);
        Identity::cdd_register_did(Origin::signed(cdd_acc), id, Some(10), vec![])
    } else {
        Identity::register_did(signed_id.clone(), vec![])
    };
    let _ = did_registration.map_err(|_| "Register DID failed")?;
    let did = Identity::get_identity(&AccountKey::try_from(id.encode())?).unwrap();

    Ok((signed_id, did))
}

pub fn make_account_without_cdd(
    id: AccountId,
) -> Result<(<TestStorage as frame_system::Trait>::Origin, IdentityId), &'static str> {
    let signed_id = Origin::signed(id.clone());
    Balances::make_free_balance_be(&id, 10_000_000);
    let did = Identity::_register_did(id.clone(), vec![], None).expect("did");
    Ok((signed_id, did))
}

pub fn register_keyring_account(acc: AccountKeyring) -> Result<IdentityId, &'static str> {
    register_keyring_account_with_balance(acc, 10_000_000)
}

pub fn register_keyring_account_with_balance(
    acc: AccountKeyring,
    balance: <TestStorage as CommonTrait>::Balance,
) -> Result<IdentityId, &'static str> {
    let acc_pub = acc.public();
    make_account_with_balance(acc_pub, balance).map(|(_, id)| id)
}

pub fn register_keyring_account_without_cdd(
    acc: AccountKeyring,
) -> Result<IdentityId, &'static str> {
    let acc_pub = acc.public();
    make_account_without_cdd(acc_pub).map(|(_, id)| id)
}

pub fn add_signing_item(did: IdentityId, signer: Signatory) {
    let master_key = Identity::did_records(&did).master_key;
    let auth_id = Identity::add_auth(
        Signatory::from(master_key),
        signer,
        AuthorizationData::JoinIdentity(did),
        None,
    );
    assert_ok!(Identity::join_identity(signer, auth_id));
}

pub fn account_from(id: u64) -> AccountId {
    let mut enc_id_vec = id.encode();
    enc_id_vec.resize_with(32, Default::default);

    let mut enc_id = [0u8; 32];
    enc_id.copy_from_slice(enc_id_vec.as_slice());

    Pair::from_seed(&enc_id).public()
}

pub fn get_identity_id(acc: AccountKeyring) -> Option<IdentityId> {
    let key = AccountKey::from(acc.public().0);
    Identity::get_identity(&key)
}
