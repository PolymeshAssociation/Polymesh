use chrono::prelude::Utc;
use frame_support::dispatch::{DispatchError, DispatchResult};
use frame_support::{assert_noop, assert_ok};
use frame_support::{StorageDoubleMap, StorageMap, StorageValue};
use hex_literal::hex;
use ink_primitives::hash as FunctionSelectorHasher;
use rand::Rng;
use sp_consensus_babe::Slot;
use sp_runtime::AnySignature;
use sp_std::collections::btree_set::BTreeSet;
use sp_std::convert::{From, TryFrom, TryInto};
use sp_std::iter;

use pallet_asset::{
    AssetDetails, AssetDocuments, AssetIdentifiers, AssetMetadataLocalKeyToName,
    AssetMetadataLocalNameToKey, AssetMetadataLocalSpecs, AssetMetadataValues, Assets,
    AssetsExemptFromAffirmation, BalanceOf, Config as AssetConfig, CustomTypeIdSequence,
    CustomTypes, CustomTypesInverse, MandatoryMediators, PreApprovedAsset,
    SecurityTokensOwnedByUser,
};
use pallet_portfolio::{
    NextPortfolioNumber, PortfolioAssetBalances, PortfolioAssetCount, PortfolioLockedAssets,
};
use pallet_statistics::AssetStats;
use polymesh_common_utilities::asset::AssetFnTrait;
use polymesh_common_utilities::constants::currency::ONE_UNIT;
use polymesh_common_utilities::traits::checkpoint::{
    NextCheckpoints, ScheduleCheckpoints, ScheduleId,
};
use polymesh_primitives::agent::AgentGroup;
use polymesh_primitives::asset::{
    AssetId, AssetName, AssetType, CheckpointId, CustomAssetTypeId, FundingRoundName,
    NonFungibleType,
};
use polymesh_primitives::asset_metadata::{
    AssetMetadataKey, AssetMetadataLocalKey, AssetMetadataLockStatus, AssetMetadataName,
    AssetMetadataSpec, AssetMetadataValue, AssetMetadataValueDetail,
};
use polymesh_primitives::calendar::{CalendarPeriod, CalendarUnit, FixedOrVariableCalendarUnit};
use polymesh_primitives::settlement::{
    InstructionId, Leg, SettlementType, VenueDetails, VenueId, VenueType,
};
use polymesh_primitives::statistics::StatType;
use polymesh_primitives::statistics::{Stat1stKey, Stat2ndKey};
use polymesh_primitives::{
    AssetIdentifier, AssetPermissions, AuthorizationData, AuthorizationError, Document, DocumentId,
    Fund, FundDescription, IdentityId, Memo, Moment, NFTCollectionKeys, Permissions, PortfolioId,
    PortfolioKind, PortfolioName, PortfolioNumber, Signatory, Ticker, WeightMeter,
};
use sp_keyring::AccountKeyring;

use crate::asset_pallet::setup::{
    create_and_issue_sample_asset, create_and_issue_sample_asset_linked_to_ticker, create_asset,
    ISSUE_AMOUNT,
};
use crate::ext_builder::ExtBuilder;
use crate::nft::create_nft_collection;
use crate::storage::{
    add_secondary_key, add_secondary_key_with_perms, register_keyring_account, root, Checkpoint,
    TestStorage, User,
};

type BaseError = pallet_base::Error<TestStorage>;
type Identity = pallet_identity::Module<TestStorage>;
type Balances = pallet_balances::Module<TestStorage>;
type Asset = pallet_asset::Module<TestStorage>;
type Timestamp = pallet_timestamp::Pallet<TestStorage>;
type ComplianceManager = pallet_compliance_manager::Module<TestStorage>;
type Portfolio = pallet_portfolio::Module<TestStorage>;
type AssetError = pallet_asset::Error<TestStorage>;
type OffChainSignature = AnySignature;
type Origin = <TestStorage as frame_system::Config>::RuntimeOrigin;
type DidRecords = pallet_identity::DidRecords<TestStorage>;
type Statistics = pallet_statistics::Module<TestStorage>;
type AssetGenesis = pallet_asset::GenesisConfig<TestStorage>;
type System = frame_system::Pallet<TestStorage>;
type ExternalAgents = pallet_external_agents::Module<TestStorage>;
type EAError = pallet_external_agents::Error<TestStorage>;
type FeeError = pallet_protocol_fee::Error<TestStorage>;
type PortfolioError = pallet_portfolio::Error<TestStorage>;
type StoreCallMetadata = pallet_permissions::StoreCallMetadata<TestStorage>;
type Settlement = pallet_settlement::Module<TestStorage>;

const TOTAL_SUPPLY: u128 = 1_000_000_000;

pub(crate) fn now() -> u64 {
    Utc::now().timestamp() as _
}

pub fn set_time_to_now() {
    set_timestamp(now());
}

fn slot_duration() -> u64 {
    pallet_babe::Pallet::<TestStorage>::slot_duration()
}

pub(crate) fn set_timestamp(n: u64) {
    pallet_babe::CurrentSlot::<TestStorage>::set(Slot::from(n / slot_duration()));
    Timestamp::set_timestamp(n);
}

pub(crate) fn max_len() -> u32 {
    <TestStorage as pallet_base::Config>::MaxLen::get()
}

pub(crate) fn max_len_bytes<R: From<Vec<u8>>>(add: u32) -> R {
    bytes_of_len(b'A', (max_len() + add) as usize)
}

macro_rules! assert_too_long {
    ($e:expr) => {
        let e_result = $e;
        assert_noop!(e_result, pallet_base::Error::<TestStorage>::TooLong);
    };
}

pub(crate) fn statistics_investor_count(asset_id: AssetId) -> u128 {
    AssetStats::get(
        Stat1stKey::investor_count(asset_id),
        Stat2ndKey::NoClaimStat,
    )
}

/// Returns the [`AssetDetails`] associated to the given `asset_id`.
pub(crate) fn get_asset_details(asset_id: &AssetId) -> AssetDetails {
    Assets::get(asset_id).unwrap()
}

/// Returns a [`AssetDetails`] where [`AssetDetails::total_supply`] is [`TOTAL_SUPPLY`] and the owner is `token_owner_did`.
pub(crate) fn sample_security_token(token_owner_did: IdentityId) -> AssetDetails {
    AssetDetails::new(TOTAL_SUPPLY, token_owner_did, true, AssetType::default())
}

fn enable_investor_count(asset_id: AssetId, owner: User) {
    assert_ok!(Statistics::set_active_asset_stats(
        owner.origin(),
        asset_id,
        [StatType::investor_count()].iter().cloned().collect(),
    ));
}

/// Transfers `amount` from `sender` default portfolio to `receiver` default portfolio.
pub(crate) fn transfer(
    asset_id: AssetId,
    sender: User,
    receiver: User,
    amount: u128,
) -> DispatchResult {
    let mut weight_meter = WeightMeter::max_limit_no_minimum();
    Asset::base_transfer(
        PortfolioId::default_portfolio(sender.did),
        PortfolioId::default_portfolio(receiver.did),
        asset_id,
        amount,
        None,
        None,
        IdentityId::default(),
        &mut weight_meter,
    )
}

/// Returns [`AssetIdentifier::cusip`].
fn cusip() -> AssetIdentifier {
    AssetIdentifier::cusip(*b"037833100").unwrap()
}

/// Returns a `FundingRoundName` which exceeds the maximum length defined in `AssetConfig`.
fn exceeded_funding_round_name() -> FundingRoundName {
    let funding_round_max_length =
        <TestStorage as AssetConfig>::FundingRoundNameMaxLength::get() + 1;

    iter::repeat(b'A')
        .take(funding_round_max_length as usize)
        .collect::<Vec<_>>()
        .into()
}

pub fn next_schedule_id(asset_id: AssetId) -> ScheduleId {
    let ScheduleId(id) = Checkpoint::schedule_id_sequence(asset_id);
    ScheduleId(id + 1)
}

#[track_caller]
pub fn check_schedules(asset_id: AssetId, schedules: &[(ScheduleId, ScheduleCheckpoints)]) {
    let mut cached = NextCheckpoints::default();
    for (id, schedule) in schedules {
        assert_eq!(
            Checkpoint::scheduled_checkpoints(asset_id, id).as_ref(),
            Some(schedule)
        );
        cached.add_schedule_next(*id, schedule.next().unwrap());
        cached.inc_total_pending(schedule.len() as u64);
    }
    if cached.is_empty() {
        assert_eq!(Checkpoint::cached_next_checkpoints(asset_id), None);
    } else {
        assert_eq!(Checkpoint::cached_next_checkpoints(asset_id), Some(cached));
    }
}

#[test]
fn check_the_test_hex() {
    ExtBuilder::default().build().execute_with(|| {
        let selector: [u8; 4] = (FunctionSelectorHasher::keccak256("verify_transfer".as_bytes())
            [0..4])
            .try_into()
            .unwrap();
        println!("{:#X}", u32::from_be_bytes(selector));
        let data = hex!("D9386E41");
        println!("{:?}", data);
    });
}

#[test]
fn issuers_can_create_and_rename_tokens() {
    ExtBuilder::default().build().execute_with(|| {
        let owner = User::new(AccountKeyring::Dave);
        let asset_id = Asset::generate_asset_id(owner.acc(), false);
        let funding_round_name: FundingRoundName = b"MyFundingRound".into();
        let sample_security_token = sample_security_token(owner.did);

        assert_ok!(Asset::create_asset(
            owner.origin(),
            b"MyAsset".into(),
            true,
            AssetType::default(),
            Vec::new(),
            Some(funding_round_name.clone()),
        ));
        enable_investor_count(asset_id, owner);

        let issue = |supply| Asset::issue(owner.origin(), asset_id, supply, PortfolioKind::Default);

        assert_noop!(
            issue(1_000_000_000_000_000_000_000_000),
            AssetError::TotalSupplyAboveLimit
        );
        // Sucessfully issue. Investor count should now be 1.
        assert_ok!(issue(sample_security_token.total_supply));
        assert_eq!(statistics_investor_count(asset_id), 1);

        // A correct entry is added
        assert_eq!(get_asset_details(&asset_id), sample_security_token);
        assert_eq!(SecurityTokensOwnedByUser::get(owner.did, asset_id), true);
        assert_eq!(Asset::funding_round(asset_id), funding_round_name.clone());

        // Unauthorized agents cannot rename the token.
        let eve = User::new(AccountKeyring::Eve);
        assert_noop!(
            Asset::rename_asset(eve.origin(), asset_id, vec![0xde, 0xad, 0xbe, 0xef].into()),
            EAError::UnauthorizedAgent
        );
        // The token should remain unchanged in storage.
        assert_eq!(get_asset_details(&asset_id), sample_security_token);
        // Rename the token and check storage has been updated.
        let new: AssetName = [0x42].into();
        assert_ok!(Asset::rename_asset(owner.origin(), asset_id, new.clone()));
        assert_eq!(Asset::asset_names(asset_id), Some(new));
        assert!(Asset::asset_identifiers(asset_id).is_empty());
    });
}

#[test]
fn valid_transfers_pass() {
    ExtBuilder::default().build().execute_with(|| {
        let owner = User::new(AccountKeyring::Dave);
        let alice = User::new(AccountKeyring::Alice);

        let asset_id = create_and_issue_sample_asset(&owner);

        // Should fail as sender matches receiver.
        let transfer = |from, to| transfer(asset_id, from, to, 500);

        assert_noop!(
            transfer(owner, owner),
            PortfolioError::InvalidTransferSenderIdMatchesReceiverId
        );
        assert_ok!(transfer(owner, alice));

        assert_eq!(Asset::balance_of(&asset_id, owner.did), ISSUE_AMOUNT - 500);
        assert_eq!(Asset::balance_of(&asset_id, alice.did), 500);
    })
}

#[test]
fn issuers_can_redeem_tokens() {
    ExtBuilder::default().build().execute_with(|| {
        let owner = User::new(AccountKeyring::Dave);
        let bob = User::new(AccountKeyring::Bob);
        let owner_portfolio_id = PortfolioId {
            did: owner.did,
            kind: PortfolioKind::Default,
        };

        let asset_id = create_and_issue_sample_asset(&owner);
        assert_eq!(PortfolioAssetCount::get(owner_portfolio_id), 1);

        assert_noop!(
            Asset::redeem(bob.origin(), asset_id, ISSUE_AMOUNT, PortfolioKind::Default),
            EAError::UnauthorizedAgent
        );

        assert_noop!(
            Asset::redeem(
                owner.origin(),
                asset_id,
                ISSUE_AMOUNT + 1,
                PortfolioKind::Default
            ),
            PortfolioError::InsufficientPortfolioBalance
        );

        assert_ok!(Asset::redeem(
            owner.origin(),
            asset_id,
            ISSUE_AMOUNT,
            PortfolioKind::Default
        ));

        assert_eq!(PortfolioAssetCount::get(owner_portfolio_id), 0);
        assert_eq!(Asset::balance_of(&asset_id, owner.did), 0);
        assert_eq!(get_asset_details(&asset_id).total_supply, 0);

        assert_noop!(
            Asset::redeem(owner.origin(), asset_id, 1, PortfolioKind::Default),
            PortfolioError::InsufficientPortfolioBalance
        );
    })
}

#[test]
fn checkpoints_fuzz_test() {
    println!("Starting");
    for _ in 0..10 {
        // When fuzzing in local, feel free to bump this number to add more fuzz runs.
        ExtBuilder::default().build().execute_with(|| {
            set_timestamp(now());
            let owner = User::new(AccountKeyring::Dave);
            let bob = User::new(AccountKeyring::Bob);

            let asset_id = create_and_issue_sample_asset(&owner);

            let mut owner_balance: [u128; 100] = [ISSUE_AMOUNT; 100];
            let mut bob_balance: [u128; 100] = [0; 100];
            let mut rng = rand::thread_rng();
            for j in 1..100 {
                let transfers = rng.gen_range(0, 10);
                owner_balance[j] = owner_balance[j - 1];
                bob_balance[j] = bob_balance[j - 1];
                for _k in 0..transfers {
                    if j == 1 {
                        owner_balance[0] -= 1;
                        bob_balance[0] += 1;
                    }
                    owner_balance[j] -= 1;
                    bob_balance[j] += 1;
                    transfer(asset_id, owner, bob, 1).unwrap();
                }
                assert_ok!(Checkpoint::create_checkpoint(owner.origin(), asset_id));
                let bal_at = |id, did| Asset::get_balance_at(asset_id, did, CheckpointId(id));
                let check = |id, idx| {
                    assert_eq!(bal_at(id, owner.did), owner_balance[idx]);
                    assert_eq!(bal_at(id, bob.did), bob_balance[idx]);
                };
                let x: u64 = u64::try_from(j).unwrap();
                check(0, j);
                check(0, j);
                check(1, 1);
                check(x - 1, j - 1);
                check(x, j);
                check(x + 1, j);
                check(1000, j);
            }
        });
    }
}

#[test]
fn controller_transfer() {
    ExtBuilder::default().build().execute_with(|| {
        let owner = User::new(AccountKeyring::Dave);
        let alice = User::new(AccountKeyring::Alice);

        let asset_id = create_and_issue_sample_asset(&owner);

        // Should fail as sender matches receiver.
        assert_noop!(
            transfer(asset_id, owner, owner, 500),
            PortfolioError::InvalidTransferSenderIdMatchesReceiverId
        );

        assert_ok!(transfer(asset_id, owner, alice, 500));

        let balance_of = |did| Asset::balance_of(&asset_id, did);
        let balance_alice = balance_of(alice.did);
        let balance_owner = balance_of(owner.did);
        assert_eq!(balance_owner, ISSUE_AMOUNT - 500);
        assert_eq!(balance_alice, 500);

        assert_ok!(Asset::controller_transfer(
            owner.origin(),
            asset_id,
            100,
            PortfolioId::default_portfolio(alice.did),
        ));
        assert_eq!(balance_of(owner.did), balance_owner + 100);
        assert_eq!(balance_of(alice.did), balance_alice - 100);
    })
}

#[test]
fn transfer_token_ownership() {
    ExtBuilder::default().build().execute_with(|| {
        set_timestamp(now());

        let owner = User::new(AccountKeyring::Dave);
        let alice = User::new(AccountKeyring::Alice);
        let bob = User::new(AccountKeyring::Bob);
        let ticker = Ticker::from_slice_truncated(b"MYTICKER");

        let asset_id = create_and_issue_sample_asset_linked_to_ticker(&owner, ticker);

        let auth_id_alice = Identity::add_auth(
            owner.did,
            Signatory::from(alice.did),
            AuthorizationData::TransferAssetOwnership(asset_id),
            None,
        )
        .unwrap();

        let auth_id_bob = Identity::add_auth(
            owner.did,
            Signatory::from(bob.did),
            AuthorizationData::TransferAssetOwnership(asset_id),
            None,
        )
        .unwrap();

        assert_eq!(get_asset_details(&asset_id).owner_did, owner.did);

        assert_noop!(
            Asset::accept_asset_ownership_transfer(alice.origin(), auth_id_alice + 1),
            "Authorization does not exist"
        );

        assert_eq!(SecurityTokensOwnedByUser::get(owner.did, asset_id), true);

        assert_ok!(Asset::accept_asset_ownership_transfer(
            alice.origin(),
            auth_id_alice
        ));
        assert_eq!(get_asset_details(&asset_id).owner_did, alice.did);
        assert_eq!(SecurityTokensOwnedByUser::get(owner.did, asset_id), false);
        assert_eq!(SecurityTokensOwnedByUser::get(alice.did, asset_id), true);

        assert_ok!(ExternalAgents::unchecked_add_agent(
            asset_id,
            alice.did,
            AgentGroup::Full
        ));
        assert_ok!(ExternalAgents::abdicate(owner.origin(), asset_id));
        assert_eq!(
            Asset::accept_asset_ownership_transfer(bob.origin(), auth_id_bob),
            Err(EAError::UnauthorizedAgent.into())
        );

        let mut auth_id = Identity::add_auth(
            alice.did,
            Signatory::from(bob.did),
            AuthorizationData::TransferAssetOwnership(asset_id),
            Some(now() - 100),
        )
        .unwrap();

        assert_noop!(
            Asset::accept_asset_ownership_transfer(bob.origin(), auth_id),
            "Authorization expired"
        );

        // Try accepting the wrong authorization type.
        auth_id = Identity::add_auth(
            alice.did,
            Signatory::from(bob.did),
            AuthorizationData::RotatePrimaryKey,
            Some(now() + 100),
        )
        .unwrap();

        assert_eq!(
            Asset::accept_asset_ownership_transfer(bob.origin(), auth_id),
            Err(AuthorizationError::BadType.into())
        );

        auth_id = Identity::add_auth(
            alice.did,
            Signatory::from(bob.did),
            AuthorizationData::TransferAssetOwnership([0; 16].into()),
            Some(now() + 100),
        )
        .unwrap();

        assert_eq!(
            Asset::accept_asset_ownership_transfer(bob.origin(), auth_id),
            Err(AssetError::NoSuchAsset.into())
        );

        auth_id = Identity::add_auth(
            alice.did,
            Signatory::from(bob.did),
            AuthorizationData::TransferAssetOwnership(asset_id),
            Some(now() + 100),
        )
        .unwrap();

        assert_ok!(Asset::accept_asset_ownership_transfer(
            bob.origin(),
            auth_id
        ));
        assert_eq!(get_asset_details(&asset_id).owner_did, bob.did);
    })
}

#[test]
fn update_identifiers() {
    ExtBuilder::default().build().execute_with(|| {
        let owner = User::new(AccountKeyring::Dave);

        // Create: A correct entry was added.
        let asset_ident = Some(vec![cusip()]);
        let asset_id = create_asset(&owner, None, None, None, asset_ident, None, false, None);
        assert_eq!(AssetIdentifiers::get(asset_id), vec![cusip()]);

        let update = |idents| Asset::update_identifiers(owner.origin(), asset_id, idents);

        // Create: A bad entry was rejected.
        assert_noop!(
            update(vec![cusip(), AssetIdentifier::CUSIP(*b"aaaa_aaaa")]),
            AssetError::InvalidAssetIdentifier
        );

        // Update: A bad entry was rejected.
        let mut updated_identifiers = vec![
            AssetIdentifier::cusip(*b"17275R102").unwrap(),
            AssetIdentifier::isin(*b"US0378331005").unwrap(),
            AssetIdentifier::CUSIP(*b"aaaa_aaaa"),
        ];
        assert_noop!(
            update(updated_identifiers.clone()),
            AssetError::InvalidAssetIdentifier
        );

        // Update: A correct entry was added.
        updated_identifiers.pop().unwrap();
        assert_ok!(update(updated_identifiers.clone()));
        assert_eq!(AssetIdentifiers::get(asset_id), updated_identifiers);
    });
}

#[test]
fn adding_removing_documents() {
    ExtBuilder::default().build().execute_with(|| {
        let owner = User::new(AccountKeyring::Dave);

        let asset_id = create_and_issue_sample_asset(&owner);

        let documents = vec![
            Document {
                name: b"A".into(),
                uri: b"www.a.com".into(),
                content_hash: [1u8; 64][..].try_into().unwrap(),
                doc_type: Some(b"foo".into()),
                filing_date: Some(42),
            },
            Document {
                name: b"B".into(),
                uri: b"www.b.com".into(),
                content_hash: [2u8; 64][..].try_into().unwrap(),
                doc_type: None,
                filing_date: None,
            },
        ];

        assert_ok!(Asset::add_documents(
            owner.origin(),
            documents.clone(),
            asset_id
        ));

        for (idx, doc) in documents.into_iter().enumerate() {
            assert_eq!(
                Some(doc),
                Asset::asset_documents(asset_id, DocumentId(idx as u32))
            );
        }

        assert_ok!(Asset::remove_documents(
            owner.origin(),
            (0..=1).map(DocumentId).collect(),
            asset_id
        ));

        assert_eq!(AssetDocuments::iter_prefix_values(asset_id).count(), 0);
    });
}

#[test]
fn freeze_unfreeze_asset() {
    ExtBuilder::default().build().execute_with(|| {
        let owner = User::new(AccountKeyring::Alice);
        let bob = User::new(AccountKeyring::Bob);

        let asset_id = create_and_issue_sample_asset(&owner);

        assert_noop!(
            Asset::freeze(bob.origin(), asset_id),
            EAError::UnauthorizedAgent
        );
        assert_noop!(
            Asset::unfreeze(owner.origin(), asset_id),
            AssetError::NotFrozen
        );
        assert_ok!(Asset::freeze(owner.origin(), asset_id));
        assert_noop!(
            Asset::freeze(owner.origin(), asset_id),
            AssetError::AlreadyFrozen
        );

        // Attempt to transfer token ownership.
        let auth_id = Identity::add_auth(
            owner.did,
            Signatory::from(bob.did),
            AuthorizationData::TransferAssetOwnership(asset_id),
            None,
        )
        .unwrap();
        assert_ok!(Asset::accept_asset_ownership_transfer(
            bob.origin(),
            auth_id
        ));

        // Not enough; bob needs to become an agent.
        assert_noop!(
            Asset::unfreeze(bob.origin(), asset_id),
            EAError::UnauthorizedAgent
        );

        assert_ok!(ExternalAgents::unchecked_add_agent(
            asset_id,
            bob.did,
            AgentGroup::Full
        ));
        assert_ok!(Asset::unfreeze(bob.origin(), asset_id));
        assert_noop!(
            Asset::unfreeze(bob.origin(), asset_id),
            AssetError::NotFrozen
        );
    });
}

#[test]
fn frozen_secondary_keys_create_asset_we() {
    ExtBuilder::default().build().execute_with(|| {
        // 0. Create identities.
        let alice = User::new(AccountKeyring::Alice);
        let bob = User::new_with(alice.did, AccountKeyring::Bob);
        let _charlie_id = register_keyring_account(AccountKeyring::Charlie).unwrap();

        // 1. Add Bob as signatory to Alice ID.
        add_secondary_key(alice.did, bob.acc());

        assert_ok!(Balances::transfer_with_memo(
            alice.origin(),
            bob.acc().into(),
            1_000,
            Some(Memo::from("Bob funding"))
        ));

        // 2. Bob can create token
        let asset_id = create_and_issue_sample_asset(&alice);
        assert_eq!(
            get_asset_details(&asset_id),
            sample_security_token(alice.did)
        );

        // 3. Alice freezes her secondary keys.
        assert_ok!(Identity::freeze_secondary_keys(alice.origin()));
    });
}

#[test]
fn next_checkpoint_is_updated_we() {
    ExtBuilder::default().build().execute_with(|| {
        // 14 November 2023, 22:13 UTC (millisecs)
        let start: u64 = 1_700_000_000_000;
        // A period of 42 hours.
        let period = CalendarPeriod {
            unit: CalendarUnit::Hour,
            amount: 42,
        };
        // The increment in seconds.
        let period_secs = match period.to_recurring().unwrap().as_fixed_or_variable() {
            FixedOrVariableCalendarUnit::Fixed(secs) => secs,
            _ => panic!("period should be fixed"),
        };
        let period_ms = period_secs * 1000;
        set_timestamp(start);
        assert_eq!(start, <TestStorage as AssetConfig>::UnixTime::now());

        let owner = User::new(AccountKeyring::Alice);
        let bob = User::new(AccountKeyring::Bob);

        let asset_id = create_and_issue_sample_asset(&owner);

        assert_eq!(Checkpoint::cached_next_checkpoints(asset_id), None);
        let schedule = ScheduleCheckpoints::from_period(start, period, 5);
        assert_ok!(Checkpoint::set_schedules_max_complexity(
            root(),
            period.complexity()
        ));
        assert_ok!(Checkpoint::create_schedule(
            owner.origin(),
            asset_id,
            schedule
        ));
        assert_ok!(Checkpoint::advance_update_balances(&asset_id, &[]));
        let id = CheckpointId(1);
        assert_eq!(id, Checkpoint::checkpoint_id_sequence(&asset_id));
        assert_eq!(start, Checkpoint::timestamps(asset_id, id));
        assert_eq!(ISSUE_AMOUNT, Checkpoint::total_supply_at(asset_id, id));
        assert_eq!(ISSUE_AMOUNT, Asset::get_balance_at(asset_id, owner.did, id));
        assert_eq!(0, Asset::get_balance_at(asset_id, bob.did, id));
        let checkpoint2 = start + period_ms;
        assert_eq!(vec![Some(checkpoint2)], next_checkpoints(asset_id));
        assert_eq!(vec![checkpoint2], checkpoint_ats(asset_id));

        let transfer = |at| {
            set_timestamp(at);
            transfer(asset_id, owner, bob, ISSUE_AMOUNT / 2).unwrap();
        };

        // Make a transaction before the next timestamp.
        transfer(checkpoint2 - 1000);
        // Make another transaction at the checkpoint.
        // The updates are applied after the checkpoint is recorded.
        // After this transfer Alice's balance is 0.
        transfer(checkpoint2);
        // The balance after checkpoint 2.
        assert_eq!(0, Asset::balance_of(&asset_id, owner.did));
        // Balances at checkpoint 2.
        let id = CheckpointId(2);
        assert_eq!(vec![start + 2 * period_ms], checkpoint_ats(asset_id));
        assert_eq!(id, Checkpoint::checkpoint_id_sequence(&asset_id));
        assert_eq!(start + period_ms, Checkpoint::timestamps(asset_id, id));
        assert_eq!(
            ISSUE_AMOUNT / 2,
            Asset::get_balance_at(asset_id, owner.did, id)
        );
        assert_eq!(
            ISSUE_AMOUNT / 2,
            Asset::get_balance_at(asset_id, bob.did, id)
        );
    });
}

#[test]
fn non_recurring_schedule_works_we() {
    ExtBuilder::default().build().execute_with(|| {
        // 14 November 2023, 22:13 UTC (millisecs)
        let start: u64 = 1_700_000_000_000;
        // Non-recuring schedule.
        let period = CalendarPeriod::default();
        set_timestamp(start);
        assert_eq!(start, <TestStorage as AssetConfig>::UnixTime::now());

        let owner = User::new(AccountKeyring::Alice);
        let bob = User::new(AccountKeyring::Bob);

        let asset_id = create_and_issue_sample_asset(&owner);

        assert_eq!(Checkpoint::cached_next_checkpoints(asset_id), None);
        let schedule = ScheduleCheckpoints::from_period(start, period, 10);
        assert_ok!(Checkpoint::set_schedules_max_complexity(
            root(),
            period.complexity()
        ));
        assert_ok!(Checkpoint::create_schedule(
            owner.origin(),
            asset_id,
            schedule
        ));
        assert_ok!(Checkpoint::advance_update_balances(&asset_id, &[]));
        let id = CheckpointId(1);
        assert_eq!(id, Checkpoint::checkpoint_id_sequence(&asset_id));
        assert_eq!(start, Checkpoint::timestamps(asset_id, id));
        assert_eq!(ISSUE_AMOUNT, Checkpoint::total_supply_at(asset_id, id));
        assert_eq!(ISSUE_AMOUNT, Asset::get_balance_at(asset_id, owner.did, id));
        assert_eq!(0, Asset::get_balance_at(asset_id, bob.did, id));
        // The schedule will not recur.
        assert_eq!(Checkpoint::cached_next_checkpoints(asset_id), None);
    });
}

fn checkpoint_ats(asset_id: AssetId) -> Vec<u64> {
    let cached = Checkpoint::cached_next_checkpoints(asset_id).unwrap_or_default();
    cached.schedules.values().copied().collect()
}

fn next_checkpoints(asset_id: AssetId) -> Vec<Option<u64>> {
    let ScheduleId(id) = Checkpoint::schedule_id_sequence(asset_id);
    (1..=id)
        .into_iter()
        .map(|id| {
            Checkpoint::scheduled_checkpoints(asset_id, ScheduleId(id)).and_then(|s| s.next())
        })
        .collect()
}

#[test]
fn schedule_remaining_works() {
    ExtBuilder::default().build().execute_with(|| {
        let start = 1_000;
        set_timestamp(start);

        let owner = User::new(AccountKeyring::Alice);
        let bob = User::new(AccountKeyring::Bob);

        let asset_id = create_and_issue_sample_asset(&owner);

        let transfer = |at: Moment| {
            set_timestamp(at * 1_000);
            transfer(asset_id, owner, bob, 1).unwrap();
        };
        let collect_ts = |sh_id| {
            Checkpoint::schedule_points(asset_id, sh_id)
                .into_iter()
                .map(|cp| Checkpoint::timestamps(asset_id, cp))
                .collect::<Vec<_>>()
        };

        // No schedules yet.
        assert_eq!(Checkpoint::cached_next_checkpoints(asset_id), None);

        // For simplicity, we use 1s = 1_000ms periods.
        let period = CalendarPeriod {
            unit: CalendarUnit::Second,
            amount: 1,
        };

        // Allow such a schedule to be added on-chain. Otherwise, we'll have errors.
        assert_ok!(Checkpoint::set_schedules_max_complexity(
            root(),
            period.complexity()
        ));

        // Create a schedule with one remaining and where `start == now`.
        let schedule = ScheduleCheckpoints::from_period(start, period, 1);
        assert_ok!(Checkpoint::create_schedule(
            owner.origin(),
            asset_id,
            schedule
        ));
        assert_ok!(Checkpoint::advance_update_balances(&asset_id, &[]));

        // We had `remaining == 1` and `start == now`,
        // so since a CP was created, hence `remaining => 0`,
        // the schedule was immediately evicted.
        assert_eq!(Checkpoint::cached_next_checkpoints(asset_id), None);
        assert_eq!(collect_ts(ScheduleId(1)), vec![start]);

        // This time, we set `remaining == 5`, but we still have `start == now`,
        // thus one CP is immediately created, so `remaining => 4`.
        let schedule = ScheduleCheckpoints::from_period(start, period, 5);
        let remaining = schedule.len();
        let id2 = ScheduleId(2);
        let assert_ts = |ticks| {
            assert_eq!(
                collect_ts(id2),
                (1..=ticks).map(|x| x * 1_000).collect::<Vec<_>>()
            );
        };
        let assert_sh = |at: Moment, remaining| {
            let now = (at - 1) * 1_000;
            let mut schedule = schedule.clone();
            schedule.remove_expired(now);
            let chain = Checkpoint::scheduled_checkpoints(asset_id, id2).unwrap_or_default();
            assert_eq!(chain, schedule);
            assert_eq!(chain.len(), remaining);
        };
        assert_ok!(Checkpoint::create_schedule(
            owner.origin(),
            asset_id,
            schedule.clone()
        ));
        assert_ok!(Checkpoint::advance_update_balances(&asset_id, &[]));
        assert_sh(2, 4);
        assert_ts(1);

        // Transfer and move through the 2nd to 4th recurrences.
        for i in 2..5 {
            transfer(i);
            assert_sh(i + 1, remaining - i as usize);
            assert_ts(i);
        }

        // Transfer and move to the 5th (last) recurrence.
        // We've to the point where there are no ticks left.
        transfer(5);
        assert_eq!(Checkpoint::cached_next_checkpoints(asset_id), None);
        assert_ts(5);
    });
}

#[test]
fn mesh_1531_ts_collission_regression_test() {
    ExtBuilder::default().build().execute_with(|| {
        // Create the assets.
        let owner = User::new(AccountKeyring::Alice);

        let asset_id = create_and_issue_sample_asset(&owner);
        let asset_id2 = create_and_issue_sample_asset(&owner);

        // First CP is made at 1s.
        let cp = CheckpointId(1);
        set_timestamp(1_000);
        let create = |asset_id| Checkpoint::create_checkpoint(owner.origin(), asset_id);
        assert_ok!(create(asset_id));
        assert_eq!(Checkpoint::timestamps(asset_id, cp), 1_000);

        // Second CP is for beta, using same ID.
        set_timestamp(2_000);
        assert_ok!(create(asset_id2));
        assert_eq!(Checkpoint::timestamps(asset_id, cp), 1_000);
        assert_eq!(Checkpoint::timestamps(asset_id2, cp), 2_000);
    });
}

#[test]
fn secondary_key_not_authorized_for_asset_test() {
    let charlie = vec![AccountKeyring::Charlie.to_account_id()];
    ExtBuilder::default()
        .cdd_providers(charlie)
        .build()
        .execute_with(|| {
            let alice = User::new(AccountKeyring::Alice);
            let bob = User::new_with(alice.did, AccountKeyring::Bob);
            let eve = User::new_with(alice.did, AccountKeyring::Eve);
            let asset_id = create_and_issue_sample_asset(&alice);
            let eve_permissions = Permissions {
                asset: AssetPermissions::elems(vec![AssetId::new([0; 16])]),
                ..Default::default()
            };

            add_secondary_key(alice.did, bob.acc());
            add_secondary_key_with_perms(alice.did, eve.acc(), eve_permissions);
            StoreCallMetadata::set_call_metadata("pallet_asset".into(), "issuer".into());

            assert_noop!(
                Asset::issue(eve.origin(), asset_id, 1_000, PortfolioKind::Default),
                pallet_external_agents::Error::<TestStorage>::SecondaryKeyNotAuthorizedForAsset
            );

            assert_ok!(Asset::issue(
                bob.origin(),
                asset_id,
                1_000,
                PortfolioKind::Default
            ));

            assert_eq!(
                get_asset_details(&asset_id).total_supply,
                ISSUE_AMOUNT + 1_000
            );
        });
}

#[test]
fn invalid_granularity_test() {
    test_with_owner(|owner| {
        let asset_id = create_asset(&owner, None, Some(false), None, None, None, false, None);
        assert_noop!(
            Asset::issue(owner.origin(), asset_id, 10_000, PortfolioKind::Default),
            AssetError::InvalidGranularity
        );
    })
}

fn bytes_of_len<R: From<Vec<u8>>>(e: u8, len: usize) -> R {
    iter::repeat(e).take(len).collect::<Vec<_>>().into()
}

#[test]
fn create_asset_errors() {
    ExtBuilder::default().build().execute_with(|| {
        let alice = User::new(AccountKeyring::Alice);

        let create = |name, is_divisible, funding_name| {
            Asset::create_asset(
                alice.origin(),
                name,
                is_divisible,
                AssetType::default(),
                vec![],
                funding_name,
            )
        };

        let max_length = <TestStorage as AssetConfig>::AssetNameMaxLength::get() + 1;
        assert_noop!(
            create(bytes_of_len(b'A', max_length as usize), true, None),
            AssetError::MaxLengthOfAssetNameExceeded
        );

        assert_noop!(
            create(b"MyAsset".into(), true, Some(exceeded_funding_round_name()),),
            AssetError::FundingRoundNameMaxLengthExceeded,
        );

        let asset_id = create_asset(&alice, None, Some(false), None, None, None, false, None);
        assert_noop!(
            Asset::issue(
                alice.origin().clone(),
                asset_id,
                1_000,
                PortfolioKind::Default
            ),
            AssetError::InvalidGranularity,
        );

        let asset_id = create_asset(&alice, None, None, None, None, None, false, None);
        assert_noop!(
            Asset::issue(
                alice.origin().clone(),
                asset_id,
                u128::MAX,
                PortfolioKind::Default
            ),
            AssetError::TotalSupplyAboveLimit,
        );
    });
}

#[test]
fn asset_type_custom_too_long() {
    ExtBuilder::default().build().execute_with(|| {
        let user = User::new(AccountKeyring::Alice);
        let case = |add| Asset::register_custom_asset_type(user.origin(), max_len_bytes(add));
        assert_too_long!(case(1));
        assert_ok!(case(0));
    });
}

#[test]
fn asset_type_custom_works() {
    ExtBuilder::default().build().execute_with(|| {
        let user = User::new(AccountKeyring::Alice);
        let register = |ty: &str| Asset::register_custom_asset_type(user.origin(), ty.into());
        let seq_is = |num| {
            assert_eq!(CustomTypeIdSequence::get().0, num);
        };
        let slot_has = |id, data: &str| {
            seq_is(id);
            let id = CustomAssetTypeId(id);
            let data = data.as_bytes();
            assert_eq!(CustomTypes::get(id), data);
            assert_eq!(CustomTypesInverse::get(data), Some(id));
        };

        // Nothing so far. Generator (G) at 0.
        seq_is(0);

        // Register first type. G -> 1.
        assert_ok!(register("foo"));
        slot_has(1, "foo");

        // Register same type. G unmoved.
        assert_ok!(register("foo"));
        slot_has(1, "foo");

        // Register different type. G -> 2.
        assert_ok!(register("bar"));
        slot_has(2, "bar");

        // Register same type. G unmoved.
        assert_ok!(register("bar"));
        slot_has(2, "bar");

        // Register different type. G -> 3.
        assert_ok!(register("foobar"));
        slot_has(3, "foobar");

        // Set G to max. Next registration fails.
        CustomTypeIdSequence::put(CustomAssetTypeId(u32::MAX));
        assert_noop!(register("qux"), BaseError::CounterOverflow);
    });
}

#[test]
fn invalid_custom_asset_type_check() {
    ExtBuilder::default().build().execute_with(|| {
        let owner = User::new(AccountKeyring::Dave);

        assert_noop!(
            Asset::create_asset(
                owner.origin(),
                b"MyAsset".into(),
                true,
                AssetType::Custom(CustomAssetTypeId(1_000_000)),
                Vec::new(),
                None,
            ),
            AssetError::InvalidCustomAssetTypeId
        );
    });
}

#[test]
fn asset_doc_field_too_long() {
    ExtBuilder::default().build().execute_with(|| {
        let owner = User::new(AccountKeyring::Alice);
        let asset_id = create_and_issue_sample_asset(&owner);
        let add_doc = |doc| Asset::add_documents(owner.origin(), vec![doc], asset_id);
        assert_too_long!(add_doc(Document {
            uri: max_len_bytes(1),
            ..<_>::default()
        }));
        assert_too_long!(add_doc(Document {
            name: max_len_bytes(1),
            ..<_>::default()
        }));
        assert_too_long!(add_doc(Document {
            doc_type: Some(max_len_bytes(1)),
            ..<_>::default()
        }));
        assert_ok!(add_doc(Document {
            uri: max_len_bytes(0),
            name: max_len_bytes(0),
            doc_type: Some(max_len_bytes(0)),
            ..<_>::default()
        }));
    });
}

#[track_caller]
fn test_with_owner(test: impl FnOnce(User)) {
    let owner = AccountKeyring::Alice;
    ExtBuilder::default()
        .add_regular_users_from_accounts(&[owner.to_account_id()])
        .cdd_providers(vec![AccountKeyring::Eve.to_account_id()])
        .build()
        .execute_with(|| test(User::existing(owner)));
}

#[test]
fn set_funding_round_test() {
    test_with_owner(|owner| {
        let asset_id = create_and_issue_sample_asset(&owner);
        assert_noop!(
            Asset::set_funding_round(owner.origin(), asset_id, exceeded_funding_round_name()),
            AssetError::FundingRoundNameMaxLengthExceeded
        );
        assert_ok!(Asset::set_funding_round(
            owner.origin(),
            asset_id,
            FundingRoundName(b"VIP round".to_vec())
        ));
    })
}

#[test]
fn update_identifiers_errors_test() {
    test_with_owner(|owner| {
        let asset_id = create_and_issue_sample_asset(&owner);
        let invalid_asset_ids = vec![
            AssetIdentifier::CUSIP(*b"037833108"),   // Invalid checksum.
            AssetIdentifier::CINS(*b"S08000AA7"),    // Invalid checksum.
            AssetIdentifier::ISIN(*b"US0378331004"), // Invalid checksum.
            AssetIdentifier::LEI(*b"549300GFX6WN7JDUSN37"), // Invalid checksum.
        ];

        invalid_asset_ids.into_iter().for_each(|identifier| {
            assert_noop!(
                Asset::update_identifiers(owner.origin(), asset_id, vec![identifier]),
                AssetError::InvalidAssetIdentifier
            );
        });

        let valid_asset_ids = vec![AssetIdentifier::CUSIP(*b"037833100")];
        assert_ok!(Asset::update_identifiers(
            owner.origin(),
            asset_id,
            valid_asset_ids
        ));
    })
}

#[test]
fn issuers_can_redeem_tokens_from_portfolio() {
    let alice = AccountKeyring::Alice.to_account_id();
    ExtBuilder::default()
        .cdd_providers(vec![alice.clone()])
        .build()
        .execute_with(|| {
            set_timestamp(now());

            let owner = User::new(AccountKeyring::Dave);
            let bob = User::new(AccountKeyring::Bob);

            // Create asset.
            let asset_id = create_and_issue_sample_asset(&owner);

            let portfolio_name = PortfolioName(vec![65u8; 5]);
            let next_portfolio_num = NextPortfolioNumber::get(&owner.did);
            let portfolio = PortfolioId::default_portfolio(owner.did);
            let user_portfolio = PortfolioId::user_portfolio(owner.did, next_portfolio_num.clone());
            Portfolio::create_portfolio(owner.origin(), portfolio_name.clone()).unwrap();

            Portfolio::move_portfolio_funds(
                owner.origin(),
                portfolio,
                user_portfolio,
                vec![Fund {
                    description: FundDescription::Fungible {
                        asset_id,
                        amount: ISSUE_AMOUNT,
                    },
                    memo: None,
                }],
            )
            .unwrap();

            assert_eq!(
                PortfolioAssetBalances::get(&portfolio, &asset_id),
                0u32.into()
            );
            assert_eq!(
                PortfolioAssetBalances::get(&user_portfolio, &asset_id),
                ISSUE_AMOUNT
            );

            assert_noop!(
                Asset::redeem(
                    bob.origin(),
                    asset_id,
                    ISSUE_AMOUNT,
                    PortfolioKind::User(next_portfolio_num)
                ),
                EAError::UnauthorizedAgent
            );

            assert_noop!(
                Asset::redeem(
                    owner.origin(),
                    asset_id,
                    ISSUE_AMOUNT + 1,
                    PortfolioKind::User(next_portfolio_num)
                ),
                PortfolioError::InsufficientPortfolioBalance
            );

            assert_ok!(Asset::redeem(
                owner.origin(),
                asset_id,
                ISSUE_AMOUNT / 2,
                PortfolioKind::User(next_portfolio_num)
            ));

            assert_eq!(Asset::balance_of(&asset_id, owner.did), ISSUE_AMOUNT / 2);
            assert_eq!(get_asset_details(&asset_id).total_supply, ISSUE_AMOUNT / 2);

            // Add auth for custody to be moved to bob
            let auth_id = Identity::add_auth(
                owner.did,
                Signatory::from(bob.did),
                AuthorizationData::PortfolioCustody(user_portfolio),
                None,
            )
            .unwrap();

            // Check that bob accepts auth
            assert_ok!(Portfolio::accept_portfolio_custody(bob.origin(), auth_id));

            assert_eq!(
                Portfolio::portfolio_custodian(user_portfolio),
                Some(bob.did)
            );

            // Check error is given when unauthorized custodian tries to redeem from portfolio
            assert_noop!(
                Asset::redeem(
                    owner.origin(),
                    asset_id,
                    ISSUE_AMOUNT,
                    PortfolioKind::User(next_portfolio_num)
                ),
                PortfolioError::UnauthorizedCustodian
            );

            // Remove bob as custodian
            assert_ok!(Portfolio::quit_portfolio_custody(
                bob.origin(),
                user_portfolio
            ));

            assert_ok!(Asset::redeem(
                owner.origin(),
                asset_id,
                ISSUE_AMOUNT / 2,
                PortfolioKind::User(next_portfolio_num)
            ));

            // Adds Bob as an external agent for the asset
            assert_ok!(ExternalAgents::unchecked_add_agent(
                asset_id,
                bob.did,
                AgentGroup::Full
            ));

            // Remove owner as agent
            assert_ok!(ExternalAgents::remove_agent(
                owner.origin(),
                asset_id,
                owner.did
            ));

            // Check error is given when unauthorized agent tries to redeem from portfolio
            assert_noop!(
                Asset::redeem(
                    owner.origin(),
                    asset_id,
                    1,
                    PortfolioKind::User(next_portfolio_num)
                ),
                EAError::UnauthorizedAgent
            );
        })
}

#[test]
fn issuers_can_change_asset_type() {
    ExtBuilder::default().build().execute_with(|| {
        let owner = User::new(AccountKeyring::Dave);
        let alice = User::new(AccountKeyring::Alice);

        // Create an asset
        let asset_id = create_and_issue_sample_asset(&owner);

        // Only the asset issuer is allowed to update the asset type
        assert_noop!(
            Asset::update_asset_type(alice.origin(), asset_id, AssetType::EquityPreferred),
            EAError::UnauthorizedAgent
        );
        // Invalid Custom type id must be rejected
        assert_noop!(
            Asset::update_asset_type(
                owner.origin(),
                asset_id,
                AssetType::Custom(CustomAssetTypeId(1))
            ),
            AssetError::InvalidCustomAssetTypeId
        );
        // Owner of the asset must be able to change the asset type, as long as it's not an invalid custom type
        assert_ok!(Asset::update_asset_type(
            owner.origin(),
            asset_id,
            AssetType::EquityPreferred
        ));
        assert_eq!(
            get_asset_details(&asset_id).asset_type,
            AssetType::EquityPreferred
        );
    })
}

/// Only metadata keys that already have a value set can be locked.
#[test]
fn prevent_locking_an_empty_key() {
    ExtBuilder::default().build().execute_with(|| {
        let alice = User::new(AccountKeyring::Alice);
        let asset_id = create_and_issue_sample_asset(&alice);
        let asset_metadata_name = AssetMetadataName(b"mylocalkey".to_vec());
        let asset_metadata_spec = AssetMetadataSpec {
            url: None,
            description: None,
            type_def: None,
        };
        assert_ok!(Asset::register_asset_metadata_local_type(
            alice.origin(),
            asset_id,
            asset_metadata_name.clone(),
            asset_metadata_spec
        ));
        let asset_metadata_detail = AssetMetadataValueDetail {
            expire: None,
            lock_status: AssetMetadataLockStatus::Locked,
        };
        let asset_metada_key = AssetMetadataKey::Local(AssetMetadataLocalKey(1));
        assert_noop!(
            Asset::set_asset_metadata_details(
                alice.origin(),
                asset_id,
                asset_metada_key,
                asset_metadata_detail
            ),
            AssetError::AssetMetadataValueIsEmpty
        );
    })
}

/// Only metadata keys that already exist can be deleted.
#[test]
fn remove_local_metadata_key_missing_key() {
    ExtBuilder::default().build().execute_with(|| {
        let alice = User::new(AccountKeyring::Alice);
        let asset_id = create_and_issue_sample_asset(&alice);
        let local_key = AssetMetadataLocalKey(1);
        assert_noop!(
            Asset::remove_local_metadata_key(alice.origin(), asset_id, local_key),
            AssetError::AssetMetadataKeyIsMissing
        );
    })
}

/// Only metadata keys that are not locked can be deleted.
#[test]
fn remove_local_metadata_key_locked_value() {
    ExtBuilder::default().build().execute_with(|| {
        let alice = User::new(AccountKeyring::Alice);
        let asset_id = create_and_issue_sample_asset(&alice);
        let asset_metadata_name = AssetMetadataName(b"mylocalkey".to_vec());
        let asset_metadata_spec = AssetMetadataSpec {
            url: None,
            description: None,
            type_def: None,
        };
        assert_ok!(Asset::register_asset_metadata_local_type(
            alice.origin(),
            asset_id,
            asset_metadata_name.clone(),
            asset_metadata_spec
        ));
        let asset_metadata_detail = AssetMetadataValueDetail {
            expire: None,
            lock_status: AssetMetadataLockStatus::Locked,
        };
        let asset_metada_key = AssetMetadataKey::Local(AssetMetadataLocalKey(1));
        assert_ok!(Asset::set_asset_metadata(
            alice.origin(),
            asset_id,
            asset_metada_key.clone(),
            AssetMetadataValue(b"randomvalue".to_vec()),
            None,
        ));
        assert_ok!(Asset::set_asset_metadata_details(
            alice.origin(),
            asset_id,
            asset_metada_key,
            asset_metadata_detail
        ));
        assert_noop!(
            Asset::remove_local_metadata_key(alice.origin(), asset_id, AssetMetadataLocalKey(1)),
            AssetError::AssetMetadataValueIsLocked
        );
    })
}

/// Only metadata keys that don't belong to NFT collections can be deleted.
#[test]
fn remove_nft_collection_metada_key() {
    ExtBuilder::default().build().execute_with(|| {
        let alice = User::new(AccountKeyring::Alice);
        let asset_metada_key = AssetMetadataKey::Local(AssetMetadataLocalKey(1));
        let collection_keys: NFTCollectionKeys = vec![asset_metada_key.clone()].into();
        let asset_id = create_nft_collection(
            alice,
            AssetType::NonFungible(NonFungibleType::Derivative),
            collection_keys,
        );
        assert_ok!(Asset::set_asset_metadata(
            alice.origin(),
            asset_id,
            asset_metada_key,
            AssetMetadataValue(b"randomvalue".to_vec()),
            None,
        ));
        assert_noop!(
            Asset::remove_local_metadata_key(alice.origin(), asset_id, AssetMetadataLocalKey(1)),
            AssetError::AssetMetadataKeyBelongsToNFTCollection
        );
    })
}

/// Successfully deletes a local metadata key.
#[test]
fn remove_local_metadata_key() {
    ExtBuilder::default().build().execute_with(|| {
        let alice = User::new(AccountKeyring::Alice);
        let asset_id = create_and_issue_sample_asset(&alice);
        let asset_metadata_name = AssetMetadataName(b"mylocalkey".to_vec());
        let asset_metadata_spec = AssetMetadataSpec {
            url: None,
            description: None,
            type_def: None,
        };
        assert_ok!(Asset::register_asset_metadata_local_type(
            alice.origin(),
            asset_id,
            asset_metadata_name.clone(),
            asset_metadata_spec
        ));
        let asset_metada_key = AssetMetadataKey::Local(AssetMetadataLocalKey(1));
        assert_ok!(Asset::set_asset_metadata(
            alice.origin(),
            asset_id,
            asset_metada_key.clone(),
            AssetMetadataValue(b"randomvalue".to_vec()),
            None,
        ));
        assert_ok!(Asset::remove_local_metadata_key(
            alice.origin(),
            asset_id,
            AssetMetadataLocalKey(1)
        ),);
        assert_eq!(
            AssetMetadataLocalKeyToName::get(&asset_id, AssetMetadataLocalKey(1)),
            None
        );
        assert_eq!(
            AssetMetadataLocalNameToKey::get(&asset_id, &asset_metadata_name),
            None
        );
        assert_eq!(
            AssetMetadataLocalSpecs::get(&asset_id, &AssetMetadataLocalKey(1)),
            None
        );
        assert_eq!(AssetMetadataValues::get(&asset_id, &asset_metada_key), None);
    })
}

/// Only metadata keys that already exist can have their value removed.
#[test]
fn remove_local_metadata_value_missing_key() {
    ExtBuilder::default().build().execute_with(|| {
        let alice = User::new(AccountKeyring::Alice);
        let asset_id = create_and_issue_sample_asset(&alice);
        assert_noop!(
            Asset::remove_metadata_value(
                alice.origin(),
                asset_id,
                AssetMetadataKey::Local(AssetMetadataLocalKey(1))
            ),
            AssetError::AssetMetadataKeyIsMissing
        );
    })
}

/// Only metadata keys that are no locked can have their value removed.
#[test]
fn remove_local_metadata_value_locked_value() {
    ExtBuilder::default().build().execute_with(|| {
        let alice = User::new(AccountKeyring::Alice);
        let asset_id = create_and_issue_sample_asset(&alice);
        let asset_metadata_name = AssetMetadataName(b"mylocalkey".to_vec());
        let asset_metadata_spec = AssetMetadataSpec {
            url: None,
            description: None,
            type_def: None,
        };
        assert_ok!(Asset::register_asset_metadata_local_type(
            alice.origin(),
            asset_id,
            asset_metadata_name.clone(),
            asset_metadata_spec
        ));
        let asset_metadata_detail = AssetMetadataValueDetail {
            expire: None,
            lock_status: AssetMetadataLockStatus::Locked,
        };
        let asset_metada_key = AssetMetadataKey::Local(AssetMetadataLocalKey(1));
        assert_ok!(Asset::set_asset_metadata(
            alice.origin(),
            asset_id,
            asset_metada_key.clone(),
            AssetMetadataValue(b"randomvalue".to_vec()),
            None,
        ));
        assert_ok!(Asset::set_asset_metadata_details(
            alice.origin(),
            asset_id,
            asset_metada_key,
            asset_metadata_detail
        ));
        assert_noop!(
            Asset::remove_metadata_value(alice.origin(), asset_id, asset_metada_key),
            AssetError::AssetMetadataValueIsLocked
        );
    })
}

/// Successfully removes a metadata value.
#[test]
fn remove_metadata_value() {
    ExtBuilder::default().build().execute_with(|| {
        let alice = User::new(AccountKeyring::Alice);
        let asset_id = create_and_issue_sample_asset(&alice);
        let asset_metadata_name = AssetMetadataName(b"mylocalkey".to_vec());
        let asset_metadata_spec = AssetMetadataSpec {
            url: None,
            description: None,
            type_def: None,
        };
        assert_ok!(Asset::register_asset_metadata_local_type(
            alice.origin(),
            asset_id,
            asset_metadata_name.clone(),
            asset_metadata_spec.clone()
        ));
        let asset_metada_key = AssetMetadataKey::Local(AssetMetadataLocalKey(1));
        assert_ok!(Asset::set_asset_metadata(
            alice.origin(),
            asset_id,
            asset_metada_key.clone(),
            AssetMetadataValue(b"randomvalue".to_vec()),
            None,
        ));
        assert_ok!(Asset::remove_metadata_value(
            alice.origin(),
            asset_id,
            asset_metada_key.clone(),
        ),);
        assert_eq!(
            AssetMetadataLocalKeyToName::get(&asset_id, AssetMetadataLocalKey(1)),
            Some(asset_metadata_name.clone())
        );
        assert_eq!(
            AssetMetadataLocalNameToKey::get(&asset_id, &asset_metadata_name),
            Some(AssetMetadataLocalKey(1))
        );
        assert_eq!(
            AssetMetadataLocalSpecs::get(&asset_id, &AssetMetadataLocalKey(1)),
            Some(asset_metadata_spec)
        );
        assert_eq!(AssetMetadataValues::get(&asset_id, &asset_metada_key), None);
    })
}

#[test]
fn issue_token_unassigned_custody() {
    ExtBuilder::default().build().execute_with(|| {
        let issued_amount = ONE_UNIT;
        let alice = User::new(AccountKeyring::Alice);
        let alice_user_portfolio = PortfolioKind::User(PortfolioNumber(1));

        assert_ok!(Portfolio::create_portfolio(
            alice.origin(),
            PortfolioName(b"AlicePortfolio".to_vec())
        ));

        let asset_id = Asset::generate_asset_id(alice.acc(), false);
        assert_ok!(Asset::create_asset(
            alice.origin(),
            b"MyAsset".into(),
            true,
            AssetType::default(),
            Vec::new(),
            None,
        ));
        assert_ok!(Asset::issue(
            alice.origin(),
            asset_id,
            issued_amount,
            alice_user_portfolio
        ));
        assert_eq!(BalanceOf::get(asset_id, alice.did), issued_amount);
    })
}

#[test]
fn redeem_token_unassigned_custody() {
    ExtBuilder::default().build().execute_with(|| {
        let alice = User::new(AccountKeyring::Alice);
        let asset_id = create_and_issue_sample_asset(&alice);
        assert_ok!(Asset::redeem(
            alice.origin(),
            asset_id,
            ISSUE_AMOUNT,
            PortfolioKind::Default
        ));
        assert_eq!(BalanceOf::get(asset_id, alice.did), 0);
    })
}

#[test]
fn redeem_token_assigned_custody() {
    ExtBuilder::default().build().execute_with(|| {
        let bob = User::new(AccountKeyring::Bob);
        let alice = User::new(AccountKeyring::Alice);
        let portfolio_id = PortfolioId::new(alice.did, PortfolioKind::Default);

        let asset_id = create_and_issue_sample_asset(&alice);
        // Change custody of the default portfolio
        let authorization_id = Identity::add_auth(
            alice.did,
            Signatory::from(bob.did),
            AuthorizationData::PortfolioCustody(portfolio_id),
            None,
        )
        .unwrap();
        assert_ok!(Portfolio::accept_portfolio_custody(
            bob.origin(),
            authorization_id
        ));

        assert_noop!(
            Asset::redeem(
                alice.origin(),
                asset_id,
                ISSUE_AMOUNT,
                PortfolioKind::Default
            ),
            PortfolioError::UnauthorizedCustodian
        );
    })
}

#[test]
fn pre_approve_asset() {
    ExtBuilder::default().build().execute_with(|| {
        let asset_id = AssetId::new([0; 16]);
        let alice = User::new(AccountKeyring::Alice);
        Asset::pre_approve_asset(alice.origin(), asset_id).unwrap();

        assert!(PreApprovedAsset::get(alice.did, asset_id));
        assert!(!AssetsExemptFromAffirmation::get(asset_id));
        assert!(Asset::skip_asset_affirmation(&alice.did, &asset_id));
    });
}

#[test]
fn remove_asset_pre_approval() {
    ExtBuilder::default().build().execute_with(|| {
        let asset_id = AssetId::new([0; 16]);
        let alice = User::new(AccountKeyring::Alice);
        Asset::pre_approve_asset(alice.origin(), asset_id).unwrap();
        Asset::remove_asset_pre_approval(alice.origin(), asset_id).unwrap();

        assert!(!PreApprovedAsset::get(alice.did, asset_id));
        assert!(!AssetsExemptFromAffirmation::get(asset_id));
        assert!(!Asset::skip_asset_affirmation(&alice.did, &asset_id));
    });
}

#[test]
fn unauthorized_custodian_ticker_exemption() {
    ExtBuilder::default().build().execute_with(|| {
        let asset_id = AssetId::new([0; 16]);
        let alice = User::new(AccountKeyring::Alice);

        assert_noop!(
            Asset::exempt_asset_affirmation(alice.origin(), asset_id),
            DispatchError::BadOrigin
        );
        assert_ok!(Asset::exempt_asset_affirmation(root(), asset_id,),);

        assert!(!PreApprovedAsset::get(alice.did, asset_id));
        assert!(AssetsExemptFromAffirmation::get(asset_id));
        assert!(Asset::skip_asset_affirmation(&alice.did, &asset_id));

        assert_noop!(
            Asset::remove_asset_affirmation_exemption(alice.origin(), asset_id),
            DispatchError::BadOrigin
        );
        assert_ok!(Asset::remove_asset_affirmation_exemption(root(), asset_id,),);

        assert!(!PreApprovedAsset::get(alice.did, asset_id));
        assert!(!AssetsExemptFromAffirmation::get(asset_id));
        assert!(!Asset::skip_asset_affirmation(&alice.did, &asset_id));
    });
}

#[test]
fn unauthorized_add_mandatory_mediators() {
    ExtBuilder::default().build().execute_with(|| {
        let bob = User::new(AccountKeyring::Bob);
        let alice = User::new(AccountKeyring::Alice);
        let max_mediators = <TestStorage as pallet_asset::Config>::MaxAssetMediators::get();
        let mediators: BTreeSet<IdentityId> = (0..max_mediators)
            .map(|i| IdentityId::from(i as u128))
            .collect();

        let asset_id = create_and_issue_sample_asset(&alice);
        assert_noop!(
            Asset::add_mandatory_mediators(bob.origin(), asset_id, mediators.try_into().unwrap()),
            EAError::UnauthorizedAgent
        );
    });
}

#[test]
fn successfully_add_mandatory_mediators() {
    ExtBuilder::default().build().execute_with(|| {
        let alice = User::new(AccountKeyring::Alice);
        let max_mediators = <TestStorage as pallet_asset::Config>::MaxAssetMediators::get();
        let mediators: BTreeSet<IdentityId> = (0..max_mediators)
            .map(|i| IdentityId::from(i as u128))
            .collect();

        let asset_id = create_and_issue_sample_asset(&alice);
        assert_ok!(Asset::add_mandatory_mediators(
            alice.origin(),
            asset_id,
            mediators.clone().try_into().unwrap()
        ));

        assert_eq!(
            MandatoryMediators::<TestStorage>::get(&asset_id).len(),
            mediators.len()
        );
        for mediator in mediators {
            assert!(MandatoryMediators::<TestStorage>::get(&asset_id).contains(&mediator));
        }
    });
}

#[test]
fn add_mandatory_mediators_exceed_limit() {
    ExtBuilder::default().build().execute_with(|| {
        let alice = User::new(AccountKeyring::Alice);
        let max_mediators = <TestStorage as pallet_asset::Config>::MaxAssetMediators::get();
        let mediators: BTreeSet<IdentityId> = (0..max_mediators)
            .map(|i| IdentityId::from(i as u128))
            .collect();

        let asset_id = create_and_issue_sample_asset(&alice);
        assert_ok!(Asset::add_mandatory_mediators(
            alice.origin(),
            asset_id,
            mediators.clone().try_into().unwrap()
        ));

        let new_mediator = BTreeSet::from([IdentityId::from(max_mediators as u128)]);
        assert_noop!(
            Asset::add_mandatory_mediators(
                alice.origin(),
                asset_id,
                new_mediator.try_into().unwrap()
            ),
            AssetError::NumberOfAssetMediatorsExceeded
        );
    });
}

#[test]
fn unauthorized_remove_mediators() {
    ExtBuilder::default().build().execute_with(|| {
        let bob = User::new(AccountKeyring::Bob);
        let alice = User::new(AccountKeyring::Alice);
        let max_mediators = <TestStorage as pallet_asset::Config>::MaxAssetMediators::get();
        let mediators: BTreeSet<IdentityId> = (0..max_mediators)
            .map(|i| IdentityId::from(i as u128))
            .collect();

        let asset_id = create_and_issue_sample_asset(&alice);
        assert_noop!(
            Asset::remove_mandatory_mediators(
                bob.origin(),
                asset_id,
                mediators.try_into().unwrap()
            ),
            EAError::UnauthorizedAgent
        );
    });
}

#[test]
fn successfully_remove_mediators() {
    ExtBuilder::default().build().execute_with(|| {
        let alice = User::new(AccountKeyring::Alice);
        let max_mediators = <TestStorage as pallet_asset::Config>::MaxAssetMediators::get();
        let mediators: BTreeSet<IdentityId> = (0..max_mediators)
            .map(|i| IdentityId::from(i as u128))
            .collect();

        let asset_id = create_and_issue_sample_asset(&alice);
        assert_ok!(Asset::add_mandatory_mediators(
            alice.origin(),
            asset_id,
            mediators.clone().try_into().unwrap()
        ));

        let remove_mediators = BTreeSet::from([IdentityId::from(0 as u128)]);
        assert_ok!(Asset::remove_mandatory_mediators(
            alice.origin(),
            asset_id,
            remove_mediators.clone().try_into().unwrap()
        ),);
        assert_eq!(
            MandatoryMediators::<TestStorage>::get(&asset_id).len(),
            mediators.len() - remove_mediators.len()
        );
        for mediator in remove_mediators {
            assert!(!MandatoryMediators::<TestStorage>::get(&asset_id).contains(&mediator));
        }
    });
}

#[test]
fn controller_transfer_locked_asset() {
    ExtBuilder::default().build().execute_with(|| {
        let bob = User::new(AccountKeyring::Bob);
        let alice = User::new(AccountKeyring::Alice);
        let bob_default_portfolio = PortfolioId {
            did: bob.did,
            kind: PortfolioKind::Default,
        };
        let alice_default_portfolio = PortfolioId {
            did: alice.did,
            kind: PortfolioKind::Default,
        };

        let asset_id = create_and_issue_sample_asset(&alice);
        let authorization_id = Identity::add_auth(
            alice.did,
            Signatory::from(bob.did),
            AuthorizationData::BecomeAgent(asset_id, AgentGroup::Full),
            None,
        )
        .unwrap();
        assert_ok!(ExternalAgents::accept_become_agent(
            bob.origin(),
            authorization_id
        ));
        // Lock the asset by creating and affirming an instruction
        assert_ok!(Settlement::create_venue(
            alice.origin(),
            VenueDetails::default(),
            vec![alice.acc()],
            VenueType::Other
        ));
        assert_ok!(Settlement::add_instruction(
            alice.origin(),
            Some(VenueId(0)),
            SettlementType::SettleManual(System::block_number() + 1),
            None,
            None,
            vec![Leg::Fungible {
                sender: alice_default_portfolio,
                receiver: bob_default_portfolio,
                asset_id,
                amount: ISSUE_AMOUNT,
            }],
            None,
        ));
        assert_ok!(Settlement::affirm_instruction(
            alice.origin(),
            InstructionId(0),
            vec![alice_default_portfolio]
        ),);

        // Controller transfer should fail since the tokens are locked
        assert_noop!(
            Asset::controller_transfer(bob.origin(), asset_id, 200, alice_default_portfolio),
            PortfolioError::InsufficientPortfolioBalance
        );
    });
}

#[test]
fn issue_tokens_user_portfolio() {
    ExtBuilder::default().build().execute_with(|| {
        let alice = User::new(AccountKeyring::Alice);
        let alice_user_portfolio = PortfolioId {
            did: alice.did,
            kind: PortfolioKind::User(PortfolioNumber(1)),
        };
        let alice_default_portfolio = PortfolioId {
            did: alice.did,
            kind: PortfolioKind::Default,
        };

        assert_ok!(Portfolio::create_portfolio(
            alice.origin(),
            PortfolioName(b"AliceUserPortfolio".to_vec())
        ));
        let asset_id = create_asset(
            &alice,
            None,
            None,
            None,
            None,
            None,
            true,
            Some(PortfolioKind::User(PortfolioNumber(1))),
        );

        assert_eq!(
            PortfolioAssetBalances::get(&alice_default_portfolio, &asset_id),
            0
        );
        assert_eq!(
            PortfolioAssetBalances::get(&alice_user_portfolio, &asset_id),
            ISSUE_AMOUNT
        );
        assert_eq!(
            PortfolioLockedAssets::get(&alice_user_portfolio, &asset_id),
            0
        );
        assert_eq!(BalanceOf::get(&asset_id, &alice.did), ISSUE_AMOUNT);
        assert_eq!(get_asset_details(&asset_id).total_supply, ISSUE_AMOUNT);
        assert_eq!(PortfolioAssetCount::get(alice_user_portfolio), 1);
        assert_eq!(PortfolioAssetCount::get(alice_default_portfolio), 0);
    });
}
