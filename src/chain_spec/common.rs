use std::convert::TryInto;

use grandpa::AuthorityId as GrandpaId;
use pallet_im_online::sr25519::AuthorityId as ImOnlineId;
use pallet_staking::StakerStatus;
use sc_chain_spec::ChainSpecExtension;
use sc_network::config::MultiaddrWithPeerId;
use sc_service::{ChainType, Properties};
use sc_telemetry::TelemetryEndpoints;
use serde::{Deserialize, Serialize};
use sp_authority_discovery::AuthorityId as AuthorityDiscoveryId;
use sp_consensus_babe::AuthorityId as BabeId;
use sp_core::{sr25519, Pair, Public};
use sp_runtime::traits::{AccountIdConversion, IdentifyAccount, Verify};

use polymesh_primitives::asset_metadata::{AssetMetadataName, AssetMetadataSpec};
use polymesh_primitives::calendar::{CalendarPeriod, CalendarUnit};
use polymesh_primitives::constants::currency::ONE_POLY;
use polymesh_primitives::identity_id::GenesisIdentityRecord;
use polymesh_primitives::{ticker, AccountId, Balance, IdentityId, Moment, Signature, Ticker};

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
pub type ChainSpec = sc_service::GenericChainSpec<Extensions>;

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

    let (grandpa_id, babe_id, im_online_id, discovery_id) = if uniq {
        (
            get_from_seed::<GrandpaId>(&format!("{}//gran", s)),
            get_from_seed::<BabeId>(&format!("{}//babe", s)),
            get_from_seed::<ImOnlineId>(&format!("{}//imon", s)),
            get_from_seed::<AuthorityDiscoveryId>(&format!("{}//auth", s)),
        )
    } else {
        (
            get_from_seed::<GrandpaId>(s),
            get_from_seed::<BabeId>(s),
            get_from_seed::<ImOnlineId>(s),
            get_from_seed::<AuthorityDiscoveryId>(s),
        )
    };

    (
        stash_acc_id,
        acc_id,
        grandpa_id,
        babe_id,
        im_online_id,
        discovery_id,
    )
}

pub(crate) fn adjust_last(bytes: &mut [u8], n: u8) -> &str {
    bytes[bytes.len() - 1] = n + b'0';
    core::str::from_utf8(bytes).unwrap()
}

/// The `ChainSpec` setup mode.
pub(crate) enum ChainSpecMode {
    Bootstrap,
    Development,
    Local,
}

/// Data required for setting up a staker in genesis.
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
    identities_record: Vec<GenesisIdentityRecord<AccountId>>,
    stakers_data: Vec<StakersData>,
    identities_balance: Vec<(AccountId, Balance)>,
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

pub(crate) fn asset_genesis_config() -> serde_json::Value {
    serde_json::json!({
        "ticker_registration_config": ticker_registration_config(),
        "reserved_country_currency_codes": currency_codes(),
        "asset_metadata": asset_metadata(),
    })
}

pub(crate) fn checkpoint_genesis_config() -> serde_json::Value {
    let period = CalendarPeriod {
        unit: CalendarUnit::Week,
        amount: 1,
    };

    serde_json::json!({
        "schedules_max_complexity": period.complexity(),
    })
}

fn ticker_registration_config() -> pallet_asset::TickerRegistrationConfig<Moment> {
    pallet_asset::TickerRegistrationConfig {
        max_ticker_length: 12,
        registration_length: Some(5_184_000_000),
    }
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
        .expect("unable do parse/deserialize asset metadata file")
}
