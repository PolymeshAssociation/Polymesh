use grandpa::AuthorityId as GrandpaId;
use im_online::sr25519::AuthorityId as ImOnlineId;
use pallet_asset::TickerRegistrationConfig;
use polymesh_common_utilities::{
    constants::currency::{MILLICENTS, POLY},
    protocol_fee::ProtocolOp,
};
use polymesh_primitives::{AccountId, AccountKey, IdentityId, PosRatio, Signatory, Signature};
use std::convert::TryFrom;

use polymesh_runtime_develop::{self as general, constants::time as GeneralTime};
use polymesh_runtime_testnet_v1::{
    self as v1,
    config::{self as V1Config, GenesisConfig},
    constants::time as V1Time,
};
use sc_service::Properties;
use serde_json::json;
use sp_authority_discovery::AuthorityId as AuthorityDiscoveryId;
use sp_consensus_babe::AuthorityId as BabeId;
use sp_core::{sr25519, Pair, Public};
use sp_runtime::{
    traits::{IdentifyAccount, Verify},
    PerThing,
};

//use substrate_telemetry::TelemetryEndpoints;
use sc_telemetry::TelemetryEndpoints;

const STAGING_TELEMETRY_URL: &str = "wss://telemetry.polymesh.live/submit/";

// TODO: Different chainspec can be used once we have new version of substrate
pub type ChainSpec = sc_service::ChainSpec<GenesisConfig>;
//pub type GeneralChainSpec = sc_service::ChainSpec<V1Config::GenesisConfig>;

type AccountPublic = <Signature as Verify>::Signer;

pub trait IsV1Network {
    fn is_v1_network(&self) -> bool;
}

impl IsV1Network for ChainSpec {
    fn is_v1_network(&self) -> bool {
        self.name().starts_with("Polymesh Aldebaran")
    }
}

fn v1_session_keys(
    grandpa: GrandpaId,
    babe: BabeId,
    im_online: ImOnlineId,
    authority_discovery: AuthorityDiscoveryId,
) -> v1::SessionKeys {
    v1::SessionKeys {
        babe,
        grandpa,
        im_online,
        authority_discovery,
    }
}

fn _general_session_keys(
    grandpa: GrandpaId,
    babe: BabeId,
    im_online: ImOnlineId,
    authority_discovery: AuthorityDiscoveryId,
) -> general::SessionKeys {
    general::SessionKeys {
        babe,
        grandpa,
        im_online,
        authority_discovery,
    }
}

/// Helper function to generate a crypto pair from seed
pub fn get_from_seed<TPublic: Public>(seed: &str) -> <TPublic::Pair as Pair>::Public {
    TPublic::Pair::from_string(&format!("//{}", seed), None)
        .expect("static values are valid; qed")
        .public()
}

/// Helper function to generate an account ID from seed
pub fn get_account_id_from_seed<TPublic: Public>(seed: &str) -> AccountId
where
    AccountPublic: From<<TPublic::Pair as Pair>::Public>,
{
    AccountPublic::from(get_from_seed::<TPublic>(seed)).into_account()
}

/// Helper function to generate stash, controller and session key from seed
pub fn get_authority_keys_from_seed(
    seed: &str,
    uniq: bool,
) -> (
    AccountId,
    AccountId,
    GrandpaId,
    BabeId,
    ImOnlineId,
    AuthorityDiscoveryId,
) {
    if uniq {
        (
            get_account_id_from_seed::<sr25519::Public>(&format!("{}//stash", seed)),
            get_account_id_from_seed::<sr25519::Public>(seed),
            get_from_seed::<GrandpaId>(&format!("{}//gran", seed)),
            get_from_seed::<BabeId>(&format!("{}//babe", seed)),
            get_from_seed::<ImOnlineId>(&format!("{}//imon", seed)),
            get_from_seed::<AuthorityDiscoveryId>(&format!("{}//auth", seed)),
        )
    } else {
        (
            get_account_id_from_seed::<sr25519::Public>(&format!("{}//stash", seed)),
            get_account_id_from_seed::<sr25519::Public>(seed),
            get_from_seed::<GrandpaId>(seed),
            get_from_seed::<BabeId>(seed),
            get_from_seed::<ImOnlineId>(seed),
            get_from_seed::<AuthorityDiscoveryId>(seed),
        )
    }
}

fn polymath_props() -> Properties {
    json!({"tokenDecimals": 6, "tokenSymbol": "POLYX" })
        .as_object()
        .unwrap()
        .clone()
}

fn general_testnet_genesis(
    initial_authorities: Vec<(
        AccountId,
        AccountId,
        GrandpaId,
        BabeId,
        ImOnlineId,
        AuthorityDiscoveryId,
    )>,
    root_key: AccountId,
    endowed_accounts: Vec<AccountId>,
    enable_println: bool,
) -> GenesisConfig {
    const STASH: u128 = 5_000_000 * POLY;
    const ENDOWMENT: u128 = 100_000_000 * POLY;

    GenesisConfig {
        frame_system: Some(V1Config::SystemConfig {
            code: general::WASM_BINARY.to_vec(),
            changes_trie_config: Default::default(),
        }),
        asset: Some(V1Config::AssetConfig {
            ticker_registration_config: TickerRegistrationConfig {
                max_ticker_length: 12,
                registration_length: Some(5_184_000_000),
            },
        }),
        identity: {
            let initial_identities = vec![
                // (master_account_id, service provider did, target did, expiry time of CustomerDueDiligence claim i.e 10 days is ms)
                // Service providers
                (
                    get_account_id_from_seed::<sr25519::Public>("cdd_provider_1"),
                    IdentityId::from(1),
                    IdentityId::from(1),
                    None,
                ),
                (
                    get_account_id_from_seed::<sr25519::Public>("cdd_provider_2"),
                    IdentityId::from(2),
                    IdentityId::from(2),
                    None,
                ),
                // Governance committee members
                (
                    get_account_id_from_seed::<sr25519::Public>("governance_committee_1"),
                    IdentityId::from(1),
                    IdentityId::from(3),
                    None,
                ),
                (
                    get_account_id_from_seed::<sr25519::Public>("governance_committee_2"),
                    IdentityId::from(1),
                    IdentityId::from(4),
                    None,
                ),
                (
                    get_account_id_from_seed::<sr25519::Public>("governance_committee_3"),
                    IdentityId::from(1),
                    IdentityId::from(5),
                    None,
                ),
            ];
            let mut identity_counter = 5;
            let authority_identities = initial_authorities
                .iter()
                .map(|x| {
                    identity_counter = identity_counter + 1;
                    (
                        x.1.clone(),
                        IdentityId::from(1),
                        IdentityId::from(identity_counter),
                        None,
                    )
                })
                .collect::<Vec<_>>();

            let all_identities = initial_identities
                .iter()
                .cloned()
                .chain(authority_identities.iter().cloned())
                .collect::<Vec<_>>();
            identity_counter = 5;
            let signing_keys = initial_authorities
                .iter()
                .map(|x| {
                    identity_counter = identity_counter + 1;
                    (x.0.clone(), IdentityId::from(identity_counter))
                })
                .collect::<Vec<_>>();

            Some(V1Config::IdentityConfig {
                identities: all_identities,
                signing_keys: signing_keys,
                ..Default::default()
            })
        },
        bridge: Some(V1Config::BridgeConfig {
            admin: initial_authorities[0].0.clone(),
            creator: initial_authorities[0].0.clone(),
            signatures_required: 3,
            signers: vec![
                Signatory::AccountKey(
                    AccountKey::try_from(&get_from_seed::<sr25519::Public>("relay_1").to_vec())
                        .unwrap(),
                ),
                Signatory::AccountKey(
                    AccountKey::try_from(&get_from_seed::<sr25519::Public>("relay_2").to_vec())
                        .unwrap(),
                ),
                Signatory::AccountKey(
                    AccountKey::try_from(&get_from_seed::<sr25519::Public>("relay_3").to_vec())
                        .unwrap(),
                ),
                Signatory::AccountKey(
                    AccountKey::try_from(&get_from_seed::<sr25519::Public>("relay_4").to_vec())
                        .unwrap(),
                ),
                Signatory::AccountKey(
                    AccountKey::try_from(&get_from_seed::<sr25519::Public>("relay_5").to_vec())
                        .unwrap(),
                ),
            ],
            timelock: 10,
            bridge_limit: (100_000_000, 1000),
        }),
        balances: Some(V1Config::BalancesConfig {
            balances: endowed_accounts
                .iter()
                .map(|k: &AccountId| (k.clone(), ENDOWMENT))
                .chain(initial_authorities.iter().map(|x| (x.1.clone(), ENDOWMENT)))
                .chain(initial_authorities.iter().map(|x| (x.0.clone(), STASH)))
                .collect(),
        }),
        pallet_indices: Some(V1Config::IndicesConfig { indices: vec![] }),
        pallet_sudo: Some(V1Config::SudoConfig { key: root_key }),
        pallet_session: Some(V1Config::SessionConfig {
            keys: initial_authorities
                .iter()
                .map(|x| {
                    (
                        x.0.clone(),
                        x.0.clone(),
                        v1_session_keys(x.2.clone(), x.3.clone(), x.4.clone(), x.5.clone()),
                    )
                })
                .collect::<Vec<_>>(),
        }),
        pallet_staking: Some(V1Config::StakingConfig {
            minimum_validator_count: 1,
            validator_count: 2,
            validator_commission: v1::Commission::Global(PerThing::from_rational_approximation(
                1u64, 4u64,
            )),
            stakers: initial_authorities
                .iter()
                .map(|x| {
                    (
                        x.0.clone(),
                        x.1.clone(),
                        STASH,
                        general::StakerStatus::Validator,
                    )
                })
                .collect(),
            invulnerables: initial_authorities.iter().map(|x| x.0.clone()).collect(),
            slash_reward_fraction: general::Perbill::from_percent(10),
            min_bond_threshold: 5_000_000_000_000,
            ..Default::default()
        }),
        pallet_pips: Some(V1Config::PipsConfig {
            prune_historical_pips: false,
            min_proposal_deposit: 5_000 * POLY,
            quorum_threshold: 100_000,
            proposal_duration: 50,
            proposal_cool_off_period: GeneralTime::DAYS * 0,
            default_enactment_period: GeneralTime::DAYS * 7,
        }),
        pallet_im_online: Some(V1Config::ImOnlineConfig {
            slashing_params: general::OfflineSlashingParams {
                max_offline_percent: 10u32,
                constant: 3u32,
                max_slash_percent: 7u32,
            },
            ..Default::default()
        }),
        pallet_authority_discovery: Some(Default::default()),
        pallet_babe: Some(Default::default()),
        pallet_grandpa: Some(Default::default()),
        pallet_contracts: Some(V1Config::ContractsConfig {
            current_schedule: contracts::Schedule {
                enable_println, // this should only be enabled on development chains
                ..Default::default()
            },
            gas_price: 1 * MILLICENTS,
        }),
        group_Instance1: Some(v1::runtime::CommitteeMembershipConfig {
            active_members: vec![
                IdentityId::from(3),
                IdentityId::from(4),
                IdentityId::from(5),
            ],
            phantom: Default::default(),
        }),
        committee_Instance1: Some(V1Config::PolymeshCommitteeConfig {
            vote_threshold: (1, 2),
            members: vec![],
            phantom: Default::default(),
        }),
        group_Instance2: Some(v1::runtime::CddServiceProvidersConfig {
            // sp1, sp2, first authority
            active_members: vec![
                IdentityId::from(1),
                IdentityId::from(2),
                IdentityId::from(6),
            ],
            phantom: Default::default(),
        }),
        protocol_fee: Some(V1Config::ProtocolFeeConfig {
            base_fees: vec![
                (ProtocolOp::AssetCreateAsset, 10_000 * 1_000_000),
                (ProtocolOp::AssetRegisterTicker, 2_500 * 1_000_000),
            ],
            coefficient: PosRatio(1, 1),
        }),
    }
}

fn general_development_genesis() -> GenesisConfig {
    general_testnet_genesis(
        vec![get_authority_keys_from_seed("Alice", false)],
        get_account_id_from_seed::<sr25519::Public>("Alice"),
        vec![get_account_id_from_seed::<sr25519::Public>("Bob")],
        true,
    )
}

pub fn general_development_testnet_config() -> ChainSpec {
    ChainSpec::from_genesis(
        "Development",
        "dev",
        general_development_genesis,
        vec![],
        None,
        None,
        Some(polymath_props()),
        None,
    )
}

fn general_local_genesis() -> GenesisConfig {
    general_testnet_genesis(
        vec![
            get_authority_keys_from_seed("Alice", false),
            get_authority_keys_from_seed("Bob", false),
        ],
        get_account_id_from_seed::<sr25519::Public>("Alice"),
        vec![
            get_account_id_from_seed::<sr25519::Public>("Charlie"),
            get_account_id_from_seed::<sr25519::Public>("Dave"),
            get_account_id_from_seed::<sr25519::Public>("Charlie//stash"),
        ],
        true,
    )
}

pub fn general_local_testnet_config() -> ChainSpec {
    ChainSpec::from_genesis(
        "Local Development",
        "local_dev",
        general_local_genesis,
        vec![],
        None,
        None,
        Some(polymath_props()),
        None,
    )
}

fn general_live_genesis() -> GenesisConfig {
    general_testnet_genesis(
        vec![
            get_authority_keys_from_seed("Alice", false),
            get_authority_keys_from_seed("Bob", false),
            get_authority_keys_from_seed("Charlie", false),
        ],
        get_account_id_from_seed::<sr25519::Public>("Alice"),
        vec![
            get_account_id_from_seed::<sr25519::Public>("Dave"),
            get_account_id_from_seed::<sr25519::Public>("Eve"),
            get_account_id_from_seed::<sr25519::Public>("Ferdie"),
            get_account_id_from_seed::<sr25519::Public>("Dave//stash"),
            get_account_id_from_seed::<sr25519::Public>("Eve//stash"),
            get_account_id_from_seed::<sr25519::Public>("Ferdie//stash"),
        ],
        false,
    )
}

pub fn general_live_testnet_config() -> ChainSpec {
    ChainSpec::from_genesis(
        "Live Development",
        "live_dev",
        general_live_genesis,
        vec![],
        None,
        None,
        Some(polymath_props()),
        None,
    )
}

fn v1_live_testnet_genesis() -> GenesisConfig {
    v1_testnet_genesis(
        vec![
            get_authority_keys_from_seed("operator_1", true),
            get_authority_keys_from_seed("operator_2", true),
            get_authority_keys_from_seed("operator_3", true),
            get_authority_keys_from_seed("operator_4", true),
            get_authority_keys_from_seed("operator_5", true),
        ],
        get_account_id_from_seed::<sr25519::Public>("polymath_1"),
        vec![
            get_account_id_from_seed::<sr25519::Public>("polymath_1"),
            get_account_id_from_seed::<sr25519::Public>("polymath_2"),
            get_account_id_from_seed::<sr25519::Public>("polymath_3"),
            get_account_id_from_seed::<sr25519::Public>("relay_1"),
            get_account_id_from_seed::<sr25519::Public>("relay_2"),
            get_account_id_from_seed::<sr25519::Public>("relay_3"),
            get_account_id_from_seed::<sr25519::Public>("relay_4"),
            get_account_id_from_seed::<sr25519::Public>("relay_5"),
        ],
        false,
    )
}

pub fn v1_live_testnet_config() -> ChainSpec {
    // provide boot nodes
    let boot_nodes = vec![];
    ChainSpec::from_genesis(
        "Polymesh Aldebaran Testnet",
        "aldebaran",
        v1_live_testnet_genesis,
        boot_nodes,
        Some(TelemetryEndpoints::new(vec![(
            STAGING_TELEMETRY_URL.to_string(),
            0,
        )])),
        Some(&*"/polymath/aldebaran/0"),
        Some(polymath_props()),
        Default::default(),
    )
}

fn v1_develop_testnet_genesis() -> GenesisConfig {
    v1_testnet_genesis(
        vec![get_authority_keys_from_seed("Alice", false)],
        get_account_id_from_seed::<sr25519::Public>("Alice"),
        vec![
            get_account_id_from_seed::<sr25519::Public>("Bob"),
            get_account_id_from_seed::<sr25519::Public>("Bob//stash"),
            get_account_id_from_seed::<sr25519::Public>("relay_1"),
            get_account_id_from_seed::<sr25519::Public>("relay_2"),
            get_account_id_from_seed::<sr25519::Public>("relay_3"),
            get_account_id_from_seed::<sr25519::Public>("relay_4"),
            get_account_id_from_seed::<sr25519::Public>("relay_5"),
        ],
        true,
    )
}

pub fn v1_develop_testnet_config() -> ChainSpec {
    // provide boot nodes
    let boot_nodes = vec![];
    ChainSpec::from_genesis(
        "Polymesh Aldebaran Develop",
        "dev_aldebaran",
        v1_develop_testnet_genesis,
        boot_nodes,
        None,
        None,
        Some(polymath_props()),
        Default::default(),
    )
}

fn v1_local_testnet_genesis() -> GenesisConfig {
    v1_testnet_genesis(
        vec![
            get_authority_keys_from_seed("Alice", false),
            get_authority_keys_from_seed("Bob", false),
        ],
        get_account_id_from_seed::<sr25519::Public>("Alice"),
        vec![
            get_account_id_from_seed::<sr25519::Public>("Charlie"),
            get_account_id_from_seed::<sr25519::Public>("Dave"),
            get_account_id_from_seed::<sr25519::Public>("Charlie//stash"),
            get_account_id_from_seed::<sr25519::Public>("relay_1"),
            get_account_id_from_seed::<sr25519::Public>("relay_2"),
            get_account_id_from_seed::<sr25519::Public>("relay_3"),
            get_account_id_from_seed::<sr25519::Public>("relay_4"),
            get_account_id_from_seed::<sr25519::Public>("relay_5"),
        ],
        true,
    )
}

pub fn v1_local_testnet_config() -> ChainSpec {
    // provide boot nodes
    let boot_nodes = vec![];
    ChainSpec::from_genesis(
        "Polymesh Aldebaran Local",
        "local_aldebaran",
        v1_local_testnet_genesis,
        boot_nodes,
        None,
        None,
        Some(polymath_props()),
        Default::default(),
    )
}

fn v1_testnet_genesis(
    initial_authorities: Vec<(
        AccountId,
        AccountId,
        GrandpaId,
        BabeId,
        ImOnlineId,
        AuthorityDiscoveryId,
    )>,
    root_key: AccountId,
    endowed_accounts: Vec<AccountId>,
    enable_println: bool,
) -> GenesisConfig {
    const STASH: u128 = 5_000_000 * POLY;
    const ENDOWMENT: u128 = 100_000_000 * POLY;

    GenesisConfig {
        frame_system: Some(V1Config::SystemConfig {
            code: v1::WASM_BINARY.to_vec(),
            changes_trie_config: Default::default(),
        }),
        asset: Some(V1Config::AssetConfig {
            ticker_registration_config: TickerRegistrationConfig {
                max_ticker_length: 12,
                registration_length: Some(5_184_000_000),
            },
        }),
        identity: {
            let initial_identities = vec![
                // (master_account_id, service provider did, target did, expiry time of CustomerDueDiligence claim i.e 10 days is ms)
                // Service providers
                (
                    get_account_id_from_seed::<sr25519::Public>("cdd_provider_1"),
                    IdentityId::from(1),
                    IdentityId::from(1),
                    None,
                ),
                (
                    get_account_id_from_seed::<sr25519::Public>("cdd_provider_2"),
                    IdentityId::from(2),
                    IdentityId::from(2),
                    None,
                ),
                (
                    get_account_id_from_seed::<sr25519::Public>("cdd_provider_3"),
                    IdentityId::from(3),
                    IdentityId::from(3),
                    None,
                ),
                // Governance committee members
                (
                    get_account_id_from_seed::<sr25519::Public>("polymath_1"),
                    IdentityId::from(1),
                    IdentityId::from(4),
                    None,
                ),
                (
                    get_account_id_from_seed::<sr25519::Public>("polymath_2"),
                    IdentityId::from(2),
                    IdentityId::from(5),
                    None,
                ),
                (
                    get_account_id_from_seed::<sr25519::Public>("polymath_3"),
                    IdentityId::from(3),
                    IdentityId::from(6),
                    None,
                ),
            ];
            let mut identity_counter = 6;
            let authority_identities = initial_authorities
                .iter()
                .map(|x| {
                    identity_counter = identity_counter + 1;
                    (
                        x.1.clone(),
                        IdentityId::from(1),
                        IdentityId::from(identity_counter),
                        None,
                    )
                })
                .collect::<Vec<_>>();

            let all_identities = initial_identities
                .iter()
                .cloned()
                .chain(authority_identities.iter().cloned())
                .collect::<Vec<_>>();
            identity_counter = 5;
            let signing_keys = initial_authorities
                .iter()
                .map(|x| {
                    identity_counter = identity_counter + 1;
                    (x.0.clone(), IdentityId::from(identity_counter))
                })
                .collect::<Vec<_>>();

            Some(V1Config::IdentityConfig {
                identities: all_identities,
                signing_keys: signing_keys,
                ..Default::default()
            })
        },
        bridge: Some(V1Config::BridgeConfig {
            admin: get_account_id_from_seed::<sr25519::Public>("polymath_1"),
            creator: get_account_id_from_seed::<sr25519::Public>("polymath_1"),
            signatures_required: 3,
            signers: vec![
                Signatory::AccountKey(
                    AccountKey::try_from(&get_from_seed::<sr25519::Public>("relay_1").to_vec())
                        .unwrap(),
                ),
                Signatory::AccountKey(
                    AccountKey::try_from(&get_from_seed::<sr25519::Public>("relay_2").to_vec())
                        .unwrap(),
                ),
                Signatory::AccountKey(
                    AccountKey::try_from(&get_from_seed::<sr25519::Public>("relay_3").to_vec())
                        .unwrap(),
                ),
                Signatory::AccountKey(
                    AccountKey::try_from(&get_from_seed::<sr25519::Public>("relay_4").to_vec())
                        .unwrap(),
                ),
                Signatory::AccountKey(
                    AccountKey::try_from(&get_from_seed::<sr25519::Public>("relay_5").to_vec())
                        .unwrap(),
                ),
            ],
            timelock: V1Time::MINUTES * 15,
            bridge_limit: (25_000_000_000, V1Time::DAYS * 1),
        }),
        balances: Some(V1Config::BalancesConfig {
            balances: endowed_accounts
                .iter()
                .map(|k: &AccountId| (k.clone(), ENDOWMENT))
                .chain(initial_authorities.iter().map(|x| (x.1.clone(), ENDOWMENT)))
                .chain(initial_authorities.iter().map(|x| (x.0.clone(), STASH)))
                .collect(),
        }),
        pallet_indices: Some(V1Config::IndicesConfig { indices: vec![] }),
        pallet_sudo: Some(V1Config::SudoConfig { key: root_key }),
        pallet_session: Some(V1Config::SessionConfig {
            keys: initial_authorities
                .iter()
                .map(|x| {
                    (
                        x.0.clone(),
                        x.0.clone(),
                        v1_session_keys(x.2.clone(), x.3.clone(), x.4.clone(), x.5.clone()),
                    )
                })
                .collect::<Vec<_>>(),
        }),
        pallet_staking: Some(V1Config::StakingConfig {
            minimum_validator_count: 1,
            validator_count: initial_authorities.len() as u32,
            validator_commission: v1::Commission::Global(PerThing::zero()),
            stakers: initial_authorities
                .iter()
                .map(|x| (x.0.clone(), x.1.clone(), STASH, v1::StakerStatus::Validator))
                .collect(),
            invulnerables: initial_authorities.iter().map(|x| x.0.clone()).collect(),
            slash_reward_fraction: v1::Perbill::from_percent(10),
            min_bond_threshold: 5_000_000_000_000,
            ..Default::default()
        }),
        pallet_pips: Some(V1Config::PipsConfig {
            prune_historical_pips: false,
            min_proposal_deposit: 5_000 * POLY,
            quorum_threshold: 100_000_000_000,
            proposal_duration: V1Time::DAYS * 7,
            proposal_cool_off_period: V1Time::HOURS * 6,
            default_enactment_period: V1Time::DAYS * 7,
        }),
        pallet_im_online: Some(V1Config::ImOnlineConfig {
            slashing_params: v1::OfflineSlashingParams {
                max_offline_percent: 10u32,
                constant: 3u32,
                max_slash_percent: 7u32,
            },
            ..Default::default()
        }),
        pallet_authority_discovery: Some(Default::default()),
        pallet_babe: Some(Default::default()),
        pallet_grandpa: Some(Default::default()),
        pallet_contracts: Some(V1Config::ContractsConfig {
            current_schedule: contracts::Schedule {
                enable_println, // this should only be enabled on development chains
                ..Default::default()
            },
            gas_price: 1 * MILLICENTS,
        }),
        group_Instance1: Some(v1::runtime::CommitteeMembershipConfig {
            active_members: vec![
                IdentityId::from(4),
                IdentityId::from(5),
                IdentityId::from(6),
            ],
            phantom: Default::default(),
        }),
        committee_Instance1: Some(v1::runtime::PolymeshCommitteeConfig {
            vote_threshold: (2, 3),
            members: vec![],
            phantom: Default::default(),
        }),
        group_Instance2: Some(v1::runtime::CddServiceProvidersConfig {
            // sp1, sp2, sp3
            active_members: vec![
                IdentityId::from(1),
                IdentityId::from(2),
                IdentityId::from(3),
            ],
            phantom: Default::default(),
        }),
        protocol_fee: Some(V1Config::ProtocolFeeConfig {
            base_fees: vec![
                (ProtocolOp::AssetCreateAsset, 10_000 * 1_000_000),
                (ProtocolOp::AssetRegisterTicker, 2_500 * 1_000_000),
            ],
            coefficient: PosRatio(1, 1),
        }),
    }
}
