use crate::{asset, bridge, exemption, general_tm, multisig, percentage_tm, statistics, utils};

use polymesh_primitives::{AccountKey, IdentityId, Signatory};
use polymesh_runtime_balances as balances;
use polymesh_runtime_common::traits::{
    asset::AcceptTransfer, group::GroupTrait, identity::ClaimValue, multisig::AddSignerMultiSig,
    CommonTrait,
};
use polymesh_runtime_group as group;
use polymesh_runtime_identity as identity;

use codec::Encode;
use frame_support::{
    dispatch::DispatchResult, impl_outer_dispatch, impl_outer_origin, parameter_types,
    traits::Currency,
};
use frame_system::{self as system, EnsureSignedBy};
use sp_core::{
    crypto::{key_types, Pair as PairTrait},
    sr25519::{Pair, Public},
    H256,
};
use sp_runtime::{
    testing::{Header, UintAuthorityId},
    traits::{BlakeTwo256, ConvertInto, IdentityLookup, OpaqueKeys, Verify},
    AnySignature, KeyTypeId, Perbill,
};
use std::convert::TryFrom;
use test_client::AccountKeyring;

impl_outer_origin! {
    pub enum Origin for TestStorage {}
}

impl_outer_dispatch! {
    pub enum Call for TestStorage where origin: Origin {
        identity::Identity,
        multisig::MultiSig,
        pallet_contracts::Contracts,
        bridge::Bridge,
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
type Event = ();
type Version = ();

parameter_types! {
    pub const BlockHashCount: u32 = 250;
    pub const MaximumBlockWeight: u32 = 4096;
    pub const MaximumBlockLength: u32 = 4096;
    pub const AvailableBlockRatio: Perbill = Perbill::from_percent(75);
}

impl frame_system::Trait for TestStorage {
    type Origin = Origin;
    type Index = Index;
    type BlockNumber = BlockNumber;
    type Hash = Hash;
    type Hashing = Hashing;
    type AccountId = AccountId;
    type Lookup = Lookup;
    type Header = Header;
    type Event = Event;

    type Call = ();
    type BlockHashCount = BlockHashCount;
    type MaximumBlockWeight = MaximumBlockWeight;
    type MaximumBlockLength = MaximumBlockLength;
    type AvailableBlockRatio = AvailableBlockRatio;
    type Version = Version;
    type ModuleToIndex = ();
}

parameter_types! {
    pub const ExistentialDeposit: u64 = 0;
    pub const TransferFee: u64 = 0;
    pub const CreationFee: u64 = 0;
    pub const TransactionBaseFee: u64 = 0;
    pub const TransactionByteFee: u64 = 0;
}

impl CommonTrait for TestStorage {
    type Balance = u128;
    type CreationFee = CreationFee;
    type AcceptTransferTarget = TestStorage;
    type BlockRewardsReserve = balances::Module<TestStorage>;
}

impl balances::Trait for TestStorage {
    type OnFreeBalanceZero = ();
    type OnNewAccount = ();
    type Event = Event;
    type DustRemoval = ();
    type TransferPayment = ();
    type ExistentialDeposit = ExistentialDeposit;
    type TransferFee = TransferFee;
    type Identity = identity::Module<TestStorage>;
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
    type Event = ();
}

parameter_types! {
    pub const One: AccountId = AccountId::from(AccountKeyring::Dave);
    pub const Two: AccountId = AccountId::from(AccountKeyring::Dave);
    pub const Three: AccountId = AccountId::from(AccountKeyring::Dave);
    pub const Four: AccountId = AccountId::from(AccountKeyring::Dave);
    pub const Five: AccountId = AccountId::from(AccountKeyring::Dave);
}

impl group::Trait<group::Instance2> for TestStorage {
    type Event = ();
    type AddOrigin = EnsureSignedBy<One, AccountId>;
    type RemoveOrigin = EnsureSignedBy<Two, AccountId>;
    type SwapOrigin = EnsureSignedBy<Three, AccountId>;
    type ResetOrigin = EnsureSignedBy<Four, AccountId>;
    type MembershipInitialized = ();
    type MembershipChanged = ();
}

impl identity::Trait for TestStorage {
    type Event = Event;
    type Proposal = Call;
    type AddSignerMultiSigTarget = TestStorage;
    type KycServiceProviders = group::Module<TestStorage, group::Instance2>;
    type Balances = balances::Module<TestStorage>;
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
    type DetermineContractAddress = pallet_contracts::SimpleAddressDeterminator<TestStorage>;
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
    type TransferFee = ContractTransferFee;
    type CreationFee = ContractCreationFee;
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

impl asset::Trait for TestStorage {
    type Event = Event;
    type Currency = balances::Module<TestStorage>;
}

impl bridge::Trait for TestStorage {
    type Event = Event;
    type Proposal = Call;
}

impl exemption::Trait for TestStorage {
    type Event = Event;
    type Asset = asset::Module<TestStorage>;
}

impl utils::Trait for TestStorage {
    type Public = AccountId;
    type OffChainSignature = OffChainSignature;
    fn validator_id_to_account_id(
        v: <Self as pallet_session::Trait>::ValidatorId,
    ) -> Self::AccountId {
        v
    }
}

pub struct TestOnSessionEnding;
impl pallet_session::OnSessionEnding<AuthorityId> for TestOnSessionEnding {
    fn on_session_ending(_: SessionIndex, _: SessionIndex) -> Option<Vec<AuthorityId>> {
        None
    }
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
}

parameter_types! {
    pub const Period: BlockNumber = 1;
    pub const Offset: BlockNumber = 0;
    pub const DisabledValidatorsThreshold: Perbill = Perbill::from_percent(33);
}

impl pallet_session::Trait for TestStorage {
    type OnSessionEnding = TestOnSessionEnding;
    type Keys = UintAuthorityId;
    type ShouldEndSession = pallet_session::PeriodicSessions<Period, Offset>;
    type SessionHandler = TestSessionHandler;
    type Event = Event;
    type ValidatorId = AuthorityId;
    type ValidatorIdOf = ConvertInto;
    type SelectInitialValidators = ();
    type DisabledValidatorsThreshold = DisabledValidatorsThreshold;
}

// Publish type alias for each module
pub type Identity = identity::Module<TestStorage>;
pub type Balances = balances::Module<TestStorage>;
pub type Asset = asset::Module<TestStorage>;
pub type MultiSig = multisig::Module<TestStorage>;
pub type Randomness = pallet_randomness_collective_flip::Module<TestStorage>;
pub type Timestamp = pallet_timestamp::Module<TestStorage>;
pub type Contracts = pallet_contracts::Module<TestStorage>;
pub type Bridge = bridge::Module<TestStorage>;
pub type CDDServieProvider = group::Module<TestStorage, group::Instance2>;

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
    let cdd_providers = CDDServieProvider::get_members();
    let did_registration = if let Some(cdd_provider) = cdd_providers.into_iter().nth(0) {
        let cdd_acc = Public::from_raw(Identity::did_records(&cdd_provider).master_key.0);
        Identity::cdd_register_did(
            Origin::signed(cdd_acc),
            id,
            10,
            ClaimValue::default(),
            vec![],
        )
    } else {
        Identity::register_did(signed_id.clone(), vec![])
    };

    let _ = did_registration.map_err(|_| "Register DID failed")?;
    let did = Identity::get_identity(&AccountKey::try_from(id.encode())?).unwrap();

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

pub fn account_from(id: u64) -> AccountId {
    let mut enc_id_vec = id.encode();
    enc_id_vec.resize_with(32, Default::default);

    let mut enc_id = [0u8; 32];
    enc_id.copy_from_slice(enc_id_vec.as_slice());

    Pair::from_seed(&enc_id).public()
}
