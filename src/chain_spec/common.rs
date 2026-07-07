use pallet_im_online::sr25519::AuthorityId as ImOnlineId;
use pallet_staking::StakerStatus;
use sc_chain_spec::ChainSpecExtension;
use sc_consensus_grandpa::AuthorityId as GrandpaId;
use sc_service::Properties;
use serde::{Deserialize, Serialize};
use sp_authority_discovery::AuthorityId as AuthorityDiscoveryId;
use sp_consensus_babe::AuthorityId as BabeId;
use sp_consensus_beefy::ecdsa_crypto::AuthorityId as BeefyId;
use sp_core::{sr25519, Pair, Public};
use sp_runtime::traits::{IdentifyAccount, Verify};
use sp_runtime::Perbill;

use pallet_asset::TickerRegistrationConfig;
use polymesh_primitives::asset_metadata::{AssetMetadataName, AssetMetadataSpec};
use polymesh_primitives::calendar::{CalendarPeriod, CalendarUnit};
use polymesh_primitives::constants::currency::ONE_POLY;
use polymesh_primitives::identity_id::GenesisIdentityRecord;
use polymesh_primitives::protocol_fee::ProtocolOp;
use polymesh_primitives::{AccountId, Balance, BlockNumber, IdentityId, Moment, Signature, Ticker};
use polymesh_primitives::{MaybeBlock, PosRatio};

// The URL for the telemetry server.
pub(crate) const STAGING_TELEMETRY_URL: &str = "wss://telemetry.polymesh.network/submit/";
pub(crate) const BOOTSTRAP_KEYS: u128 = 6_000 * ONE_POLY;
pub(crate) const BOOTSTRAP_TREASURY: u128 = 17_500_000 * ONE_POLY;
pub(crate) const DEV_KEYS: u128 = 30_000_000 * ONE_POLY;
pub(crate) const DEV_TREASURY: u128 = 50_000_000 * ONE_POLY;
pub(crate) const INITIAL_BOND: u128 = 500 * ONE_POLY;

pub(crate) type InitialAuth = (
    AccountId,
    AccountId,
    GrandpaId,
    BabeId,
    ImOnlineId,
    AuthorityDiscoveryId,
    BeefyId,
);
pub(crate) type AccountPublic = <Signature as Verify>::Signer;

/// Node `ChainSpec` extensions.
///
/// Additional parameters for some Substrate core modules,
/// customizable from the chain spec.
#[derive(Default, Clone, Serialize, Deserialize, ChainSpecExtension)]
#[serde(rename_all = "camelCase")]
pub struct Extensions {
    /// The light sync state extension used by the sync-state rpc.
    pub light_sync_state: sc_sync_state_rpc::LightSyncStateExtension,
}

/// Specialized `ChainSpec`.
pub(crate) type ChainSpec = sc_service::GenericChainSpec<Extensions>;

pub(crate) fn polymesh_properties(ss58_format: u8) -> Properties {
    let mut properties = Properties::new();
    properties.insert("ss58Format".to_string(), ss58_format.into());
    properties.insert("tokenDecimals".to_string(), 6.into());
    properties.insert("tokenSymbol".to_string(), "POLYX".into());

    properties
}

/// Generate an account ID from seed.
pub fn get_account_id_from_seed<TPublic: Public>(seed: &str) -> AccountId
where
    AccountPublic: From<<TPublic::Pair as Pair>::Public>,
{
    AccountPublic::from(get_from_seed::<TPublic>(seed)).into_account()
}

pub(crate) fn seeded_acc_id(seed: &str) -> AccountId {
    get_account_id_from_seed::<sr25519::Public>(seed)
}

/// Generate a crypto pair from seed.
pub(crate) fn get_from_seed<TPublic: Public>(seed: &str) -> <TPublic::Pair as Pair>::Public {
    TPublic::Pair::from_string(&format!("//{}", seed), None)
        .expect("static values are valid; qed")
        .public()
}

/// Generate an Aura authority key.
pub(crate) fn get_authority_keys_from_seed(s: &str, uniq: bool) -> InitialAuth {
    let stash_acc_id = seeded_acc_id(&format!("{}//stash", s));
    let acc_id = seeded_acc_id(s);

    let (grandpa_id, babe_id, im_online_id, discovery_id, beefy_id) = if uniq {
        (
            get_from_seed::<GrandpaId>(&format!("{}//gran", s)),
            get_from_seed::<BabeId>(&format!("{}//babe", s)),
            get_from_seed::<ImOnlineId>(&format!("{}//imon", s)),
            get_from_seed::<AuthorityDiscoveryId>(&format!("{}//auth", s)),
            get_from_seed::<BeefyId>(&format!("{}//beef", s)),
        )
    } else {
        (
            get_from_seed::<GrandpaId>(s),
            get_from_seed::<BabeId>(s),
            get_from_seed::<ImOnlineId>(s),
            get_from_seed::<AuthorityDiscoveryId>(s),
            get_from_seed::<BeefyId>(s),
        )
    };

    (
        stash_acc_id,
        acc_id,
        grandpa_id,
        babe_id,
        im_online_id,
        discovery_id,
        beefy_id,
    )
}

pub(crate) fn adjust_last(bytes: &mut [u8], n: u8) -> &str {
    bytes[bytes.len() - 1] = n + b'0';
    core::str::from_utf8(bytes).unwrap()
}

//==========================================================================
// Common genesis config uses across different runtimes
//==========================================================================

pub(crate) fn asset_genesis_config() -> serde_json::Value {
    let ticker_reg_config = TickerRegistrationConfig::<Moment> {
        max_ticker_length: 12,
        registration_length: Some(5_184_000_000),
    };

    serde_json::json!({
        "tickerRegistrationConfig": ticker_reg_config,
        "reservedCountryCurrencyCodes": currency_codes(),
        "assetMetadata": asset_metadata(),
    })
}

pub(crate) fn checkpoint_genesis_config() -> serde_json::Value {
    let period = CalendarPeriod {
        unit: CalendarUnit::Week,
        amount: 1,
    };

    serde_json::json!({
        "schedulesMaxComplexity": period.complexity(),
    })
}

pub(crate) fn validators_genesis_config(
    initial_stakers: &[StakersData],
    validator_commission_cap: Perbill,
) -> serde_json::Value {
    serde_json::json!({
        "validators": initial_stakers.iter().filter_map(|x| {
            if let StakerStatus::Validator = x.status {
                Some(x.identity_id)
            } else {
                None
            }
        })
        .collect::<Vec<_>>(),
        "validatorCommissionCap": validator_commission_cap,
    })
}

pub(crate) fn staking_genesis_config(initial_stakers: &[StakersData]) -> serde_json::Value {
    serde_json::json!({
        "validatorCount": 40,
        "minimumValidatorCount": 1,
        "stakers": initial_stakers.iter().map(|x| {
            (
                x.stash_id.clone(),
                x.controller_id.clone(),
                x.bonded_amount,
                x.status.clone(),
            )
        })
        .collect::<Vec<_>>(),
        "slashRewardFraction": Perbill::from_percent(10),
    })
}

pub(crate) fn pips_genesis_config(
    enactment_period: BlockNumber,
    pending_pip_expiry: MaybeBlock<BlockNumber>,
    active_pip_limit: u32,
) -> serde_json::Value {
    serde_json::json!({
        "pruneHistoricalPips": false,
        "minProposalDeposit": 2_000_000_000,
        "defaultEnactmentPeriod": enactment_period,
        "pendingPipExpiry": pending_pip_expiry,
        "maxPipSkipCount": 2,
        "activePipLimit": active_pip_limit,
    })
}

pub(crate) fn group_genesis_config(active_members_ids: Vec<IdentityId>) -> serde_json::Value {
    serde_json::json!({
       "activeMembersLimit": 20,
       "activeMembers": active_members_ids,
    })
}

pub(crate) fn committee_genesis_config(
    vote_threshold: (u32, u32),
    release_coordinator: IdentityId,
) -> serde_json::Value {
    serde_json::json!({
        "voteThreshold": vote_threshold,
        "releaseCoordinator": release_coordinator,
    })
}

pub(crate) fn protocol_fee_genesis_config() -> serde_json::Value {
    let protocol_fee: Vec<(ProtocolOp, u128)> = vec![
        (ProtocolOp::AssetCreateAsset, 2_500 * 1_000_000),
        (ProtocolOp::AssetRegisterTicker, 500 * 1_000_000),
    ];

    serde_json::json!({
        "baseFees": protocol_fee,
        "coefficient": PosRatio(1, 1)
    })
}

pub(crate) fn corporate_actions_genesis_config() -> serde_json::Value {
    serde_json::json!({
        "maxDetailsLength": 1_024,
    })
}

fn currency_codes() -> Vec<Ticker> {
    #[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
    pub struct FiatCurrency<String> {
        pub codes: Vec<String>,
    }

    let currency_file = include_str!("../data/currency_symbols.json");
    let currency_data: FiatCurrency<String> =
        serde_json::from_str(&currency_file).expect("unable do parse/deserialize currency file");

    currency_data
        .codes
        .into_iter()
        .map(|y| Ticker::from_slice_truncated(y.as_bytes()))
        .collect()
}

fn asset_metadata() -> Vec<(AssetMetadataName, AssetMetadataSpec)> {
    let asset_metadata_file = include_str!("../data/asset_metadata.json");
    serde_json::from_str(&asset_metadata_file)
        .expect("unable do parse/deserialize asset_metadata file")
}

//==========================================================================
// Types used for generating the genesis config
//==========================================================================

/// The `ChainSpec` setup mode.
pub(crate) enum ChainSpecMode {
    Bootstrap,
    Development,
    Local,
}

/// Data required for setting up a staker in genesis.
#[derive(Serialize, Deserialize)]
pub(crate) struct StakersData {
    identity_id: IdentityId,
    stash_id: AccountId,
    controller_id: AccountId,
    bonded_amount: Balance,
    status: StakerStatus<AccountId>,
}

impl StakersData {
    /// Creates a new [`StakersData`] instance.
    pub(crate) fn new(
        identity_id: IdentityId,
        stash_id: AccountId,
        controller_id: AccountId,
        bonded_amount: Balance,
        status: StakerStatus<AccountId>,
    ) -> Self {
        Self {
            identity_id,
            stash_id,
            controller_id,
            bonded_amount,
            status,
        }
    }
}

/// Data required for setting up the initial genesis state.
pub(crate) struct GenesisData {
    pub(crate) identities_record: Vec<GenesisIdentityRecord<AccountId>>,
    pub(crate) stakers_data: Vec<StakersData>,
    pub(crate) identities_balance: Vec<(AccountId, Balance)>,
}

impl GenesisData {
    /// Creates a new [`GenesisData`] instance.
    pub(crate) fn new(
        identities_record: Vec<GenesisIdentityRecord<AccountId>>,
        stakers_data: Vec<StakersData>,
        identities_balance: Vec<(AccountId, Balance)>,
    ) -> Self {
        Self {
            identities_record,
            stakers_data,
            identities_balance,
        }
    }
}
