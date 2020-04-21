use grandpa::AuthorityId as GrandpaId;
use im_online::sr25519::AuthorityId as ImOnlineId;
use polymesh_common_utilities::constants::{
    currency::{MILLICENTS, POLY}
};
use polymesh_primitives::{AccountId, IdentityId, Signature};
use polymesh_runtime_common::asset::TickerRegistrationConfig;
use polymesh_runtime_develop::{self as general, config as GeneralConfig, constants::time::HOURS as DevelopHours};
use polymesh_runtime_testnet_v1::{self as v1, config as v1Config, constants::time::HOURS as v1Hours};
use sc_service::Properties;
use serde_json::json;
use sp_authority_discovery::AuthorityId as AuthorityDiscoveryId;
use sc_chain_spec::ChainType;
use sp_consensus_babe::AuthorityId as BabeId;
use sp_core::{sr25519, Pair, Public};
use sp_runtime::{
    traits::{IdentifyAccount, Verify},
    PerThing,
};

/// Specialized `ChainSpec` for develop chain
pub type GeneralChainSpec = sc_service::ChainSpec<general::runtime::GenesisConfig>;

/// Specialized `ChainSpec` for testnet-v1 chain
pub type V1ChainSpec = sc_service::ChainSpec<v1::runtime::GenesisConfig>;

type AccountPublic = <Signature as Verify>::Signer;

fn v1_session_keys(
    grandpa: GrandpaId,
    babe: BabeId,
    im_online: ImOnlineId,
    authority_discovery: AuthorityDiscoveryId,
) -> v1::SessionKeys {
    general::SessionKeys {
        babe,
        grandpa,
        im_online,
        authority_discovery,
    }
}

fn general_session_keys(
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
) -> (
    AccountId,
    AccountId,
    GrandpaId,
    BabeId,
    ImOnlineId,
    AuthorityDiscoveryId,
) {
    (
        get_account_id_from_seed::<sr25519::Public>(&format!("{}//stash", seed)),
        get_account_id_from_seed::<sr25519::Public>(seed),
        get_from_seed::<GrandpaId>(seed),
        get_from_seed::<BabeId>(seed),
        get_from_seed::<ImOnlineId>(seed),
        get_from_seed::<AuthorityDiscoveryId>(seed),
    )
}

fn polymath_props() -> Properties {
    json!({"tokenDecimals": 6, "tokenSymbol": "POLY" })
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
) -> GeneralConfig::GenesisConfig {

    const STASH: u128 = 30_000_000_000 * POLY; //30G Poly

    GeneralConfig::GenesisConfig {
        frame_system: Some(GeneralConfig::SystemConfig {
            code: general::WASM_BINARY.to_vec(),
            changes_trie_config: Default::default(),
        }),
        asset: Some(GeneralConfig::AssetConfig {
            asset_creation_fee: 250,
            ticker_registration_fee: 250,
            ticker_registration_config: TickerRegistrationConfig {
                max_ticker_length: 12,
                registration_length: Some(5_184_000_000),
            },
            fee_collector: get_account_id_from_seed::<sr25519::Public>("Dave"),
        }),
        bridge: Some(GeneralConfig::BridgeConfig {
            admin: get_account_id_from_seed::<sr25519::Public>("Alice"),
            creator: get_account_id_from_seed::<sr25519::Public>("Alice"),
            signatures_required: 0,
            signers: vec![],
            timelock: 10,
        }),
        identity: Some(GeneralConfig::IdentityConfig {
            owner: get_account_id_from_seed::<sr25519::Public>("Dave"),
            identities: vec![
                // (master_account_id, service provider did, target did, expiry time of CustomerDueDiligence claim i.e 10 days is ms)
                // Service providers
                (
                    get_account_id_from_seed::<sr25519::Public>("service_provider_1"),
                    IdentityId::from(1),
                    IdentityId::from(1),
                    None,
                ),
                (
                    get_account_id_from_seed::<sr25519::Public>("service_provider_2"),
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
                    IdentityId::from(2),
                    IdentityId::from(5),
                    None,
                ),
                // Validators
                (
                    get_account_id_from_seed::<sr25519::Public>("Alice//stash"),
                    IdentityId::from(2),
                    IdentityId::from(6),
                    None,
                ),
                (
                    get_account_id_from_seed::<sr25519::Public>("Bob//stash"),
                    IdentityId::from(1),
                    IdentityId::from(7),
                    None,
                ),
                (
                    get_account_id_from_seed::<sr25519::Public>("Charlie//stash"),
                    IdentityId::from(1),
                    IdentityId::from(8),
                    None,
                ),
                // Alice and bob
                (
                    get_account_id_from_seed::<sr25519::Public>("Alice"),
                    IdentityId::from(42),
                    IdentityId::from(42),
                    None,
                ),
                (
                    get_account_id_from_seed::<sr25519::Public>("Bob"),
                    IdentityId::from(42),
                    IdentityId::from(1337),
                    None,
                ),
            ],
            ..Default::default()
        }),
        simple_token: Some(GeneralConfig::SimpleTokenConfig { creation_fee: 1000 }),
        balances: Some(GeneralConfig::BalancesConfig {
            balances: endowed_accounts
                .iter()
                .cloned()
                .map(|k| (k, 1 << 55))
                .collect(),
        }),
        pallet_treasury: Some(Default::default()),
        pallet_indices: Some(GeneralConfig::IndicesConfig { indices: vec![] }),
        pallet_sudo: Some(GeneralConfig::SudoConfig { key: root_key }),
        pallet_session: Some(GeneralConfig::SessionConfig {
            keys: initial_authorities
                .iter()
                .map(|x| {
                    (
                        x.0.clone(),
                        x.0.clone(),
                        general_session_keys(x.2.clone(), x.3.clone(), x.4.clone(), x.5.clone()),
                    )
                })
                .collect::<Vec<_>>(),
        }),
        pallet_staking: Some(GeneralConfig::StakingConfig {
            minimum_validator_count: 1,
            validator_count: 2,
            stakers: initial_authorities
                .iter()
                .map(|x| (x.0.clone(), x.1.clone(), STASH, general::StakerStatus::Validator))
                .collect(),
            invulnerables: initial_authorities.iter().map(|x| x.0.clone()).collect(),
            slash_reward_fraction: general::Perbill::from_percent(10),
            validator_commission: general::Commission::Global(PerThing::from_rational_approximation(
                1u64, 4u64,
            )),
            min_bond_threshold: 0,
            ..Default::default()
        }),
        pallet_mips: Some(GeneralConfig::MipsConfig {
            min_proposal_deposit: 5000,
            quorum_threshold: 100_000,
            proposal_duration: 50,
            proposal_cool_off_period: DevelopHours * 6,
        }),
        pallet_im_online: Some(GeneralConfig::ImOnlineConfig {
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
        pallet_contracts: Some(GeneralConfig::ContractsConfig {
            current_schedule: contracts::Schedule {
                enable_println, // this should only be enabled on development chains
                ..Default::default()
            },
            gas_price: 1 * MILLICENTS,
        }),
        group_Instance1: Some(general::runtime::CommitteeMembershipConfig {
            active_members: vec![],
            phantom: Default::default(),
        }),
        committee_Instance1: Some(GeneralConfig::PolymeshCommitteeConfig {
            vote_threshold: (1, 2),
            members: vec![
                IdentityId::from(3),
                IdentityId::from(4),
                IdentityId::from(5),
            ],
            phantom: Default::default(),
        }),
        group_Instance2: Some(general::runtime::CddServiceProvidersConfig {
            // sp1, sp2, alice
            active_members: vec![
                IdentityId::from(1),
                IdentityId::from(2),
                IdentityId::from(42),
            ],
            phantom: Default::default(),
        }),
        protocol_fee: Some(GeneralConfig::ProtocolFeeConfig {
            ..Default::default()
        }),
    }
}

fn general_development_genesis() -> GeneralConfig::GenesisConfig {
    general_testnet_genesis(
        vec![get_authority_keys_from_seed("Alice")],
        get_account_id_from_seed::<sr25519::Public>("Alice"),
        vec![
            get_account_id_from_seed::<sr25519::Public>("Alice"),
            get_account_id_from_seed::<sr25519::Public>("Bob"),
            get_account_id_from_seed::<sr25519::Public>("Alice//stash"),
        ],
        true,
    )
}


pub fn general_development_testnet_config() -> GeneralChainSpec {
    GeneralChainSpec::from_genesis(
		"Development",
		"dev",
		ChainType::Development,
		general_development_genesis,
		vec![],
		None,
        None,
		Some(polymath_props()),
        None,
	)
}

fn general_local_genesis() -> GeneralConfig::GenesisConfig {
    general_testnet_genesis(
        vec![
            get_authority_keys_from_seed("Alice"),
            get_authority_keys_from_seed("Bob"),
        ],
        get_account_id_from_seed::<sr25519::Public>("Alice"),
        vec![
            get_account_id_from_seed::<sr25519::Public>("Alice"),
            get_account_id_from_seed::<sr25519::Public>("Bob"),
            get_account_id_from_seed::<sr25519::Public>("Charlie"),
            get_account_id_from_seed::<sr25519::Public>("Dave"),
            get_account_id_from_seed::<sr25519::Public>("Alice//stash"),
            get_account_id_from_seed::<sr25519::Public>("Bob//stash"),
            get_account_id_from_seed::<sr25519::Public>("Charlie//stash"),
        ],
        true,
    )
}

pub fn general_local_testnet_config() -> GeneralChainSpec {
    GeneralChainSpec::from_genesis(
		"Local Testnet",
        "local_testnet",
		ChainType::Local,
		general_local_genesis,
		vec![],
		None,
        None,
		Some(polymath_props()),
        None,
	)
}

fn general_live_genesis() -> GeneralConfig::GenesisConfig {
    general_testnet_genesis(
        vec![
            get_authority_keys_from_seed("Alice"),
            get_authority_keys_from_seed("Bob"),
            get_authority_keys_from_seed("Charlie"),
        ],
        get_account_id_from_seed::<sr25519::Public>("Alice"),
        vec![
            get_account_id_from_seed::<sr25519::Public>("Alice"),
            get_account_id_from_seed::<sr25519::Public>("Bob"),
            get_account_id_from_seed::<sr25519::Public>("Charlie"),
            get_account_id_from_seed::<sr25519::Public>("Dave"),
            get_account_id_from_seed::<sr25519::Public>("Eve"),
            get_account_id_from_seed::<sr25519::Public>("Ferdie"),
            get_account_id_from_seed::<sr25519::Public>("Alice//stash"),
            get_account_id_from_seed::<sr25519::Public>("Bob//stash"),
            get_account_id_from_seed::<sr25519::Public>("Charlie//stash"),
            get_account_id_from_seed::<sr25519::Public>("Dave//stash"),
            get_account_id_from_seed::<sr25519::Public>("Eve//stash"),
            get_account_id_from_seed::<sr25519::Public>("Ferdie//stash"),
        ],
        false,
    )
}

pub fn general_live_testnet_config() -> GeneralChainSpec {
    GeneralChainSpec::from_genesis(
		"Live Testnet",
        "live-testnet",
		ChainType::Live,
		general_live_genesis,
		vec![],
		None,
        None,
		Some(polymath_props()),
        None,
	)
}

fn v1_live_testnet_genesis() -> v1Config::GenesisConfig {

    // Need to provide authorities
    let initial_authorities: Vec<(
        AccountId,
        AccountId,
        GrandpaId,
        BabeId,
        ImOnlineId,
        AuthorityDiscoveryId,
    )> = vec![()];
    let root_key: AccountId;
    // Need endowed accounts address
    let endowed_accounts: Vec<AccountId> = vec![];

    const STASH: u128 = 300 * POLY; //300 Poly
    const ENDOWMENT: u128 = 1_00_000 * POLY;

    v1Config::GenesisConfig {
        frame_system: Some(v1Config::SystemConfig {
            code: v1::WASM_BINARY.to_vec(),
            changes_trie_config: Default::default(),
        }),
        asset: Some(v1Config::AssetConfig {
            asset_creation_fee: 250,
            ticker_registration_fee: 250,
            ticker_registration_config: TickerRegistrationConfig {
                max_ticker_length: 12,
                registration_length: Some(5_184_000_000),
            },
            fee_collector: get_account_id_from_seed::<sr25519::Public>("Dave"),
        }),
        bridge: Some(v1Config::BridgeConfig {
            admin: get_account_id_from_seed::<sr25519::Public>("Alice"),
            creator: get_account_id_from_seed::<sr25519::Public>("Alice"),
            signatures_required: 0,
            signers: vec![],
            timelock: 10,
        }),
        identity: Some(v1Config::IdentityConfig {
            owner: get_account_id_from_seed::<sr25519::Public>("Dave"),
            identities: vec![
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
                    IdentityId::from(2),
                    IdentityId::from(2),
                    None,
                ),
                // Governance committee members
                (
                    get_account_id_from_seed::<sr25519::Public>("polymath_1"),
                    IdentityId::from(1),
                    IdentityId::from(3),
                    None,
                ),
                (
                    get_account_id_from_seed::<sr25519::Public>("polymath_2"),
                    IdentityId::from(1),
                    IdentityId::from(4),
                    None,
                ),
                (
                    get_account_id_from_seed::<sr25519::Public>("polymath_3"),
                    IdentityId::from(2),
                    IdentityId::from(5),
                    None,
                ),
                // Validators
                (
                    get_account_id_from_seed::<sr25519::Public>("operator_1//stash"),
                    IdentityId::from(2),
                    IdentityId::from(6),
                    None,
                ),
                (
                    get_account_id_from_seed::<sr25519::Public>("operator_2//stash"),
                    IdentityId::from(1),
                    IdentityId::from(7),
                    None,
                ),
                (
                    get_account_id_from_seed::<sr25519::Public>("operator_3//stash"),
                    IdentityId::from(1),
                    IdentityId::from(8),
                    None,
                ),
                (
                    get_account_id_from_seed::<sr25519::Public>("operator_4//stash"),
                    IdentityId::from(1),
                    IdentityId::from(9),
                    None,
                ),
                (
                    get_account_id_from_seed::<sr25519::Public>("operator_5//stash"),
                    IdentityId::from(1),
                    IdentityId::from(10),
                    None,
                )
            ],
            ..Default::default()
        }),
        simple_token: Some(v1Config::SimpleTokenConfig { creation_fee: 1000 }),
        balances: Some(v1Config::BalancesConfig {
            balances: endowed_accounts
                .iter()
                .cloned()
                .map(|k| (k, 1 << 55))
                .collect(),
        }),
        pallet_treasury: Some(Default::default()),
        pallet_indices: Some(v1Config::IndicesConfig { indices: vec![] }),
        pallet_sudo: Some(v1Config::SudoConfig { key: root_key }),
        pallet_session: Some(v1Config::SessionConfig {
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
        pallet_staking: Some(v1Config::StakingConfig {
            minimum_validator_count: 1,
            validator_count: 2,
            stakers: initial_authorities
                .iter()
                .map(|x| (x.0.clone(), x.1.clone(), STASH, v1::StakerStatus::Validator))
                .collect(),
            invulnerables: initial_authorities.iter().map(|x| x.0.clone()).collect(),
            slash_reward_fraction: v1::Perbill::from_percent(10),
            validator_commission: v1::Commission::Global(PerThing::from_rational_approximation(
                1u64, 4u64,
            )),
            min_bond_threshold: 0,
            ..Default::default()
        }),
        pallet_mips: Some(v1Config::MipsConfig {
            min_proposal_deposit: 5000,
            quorum_threshold: 100_000,
            proposal_duration: 50,
            proposal_cool_off_period: v1Hours * 6,
        }),
        pallet_im_online: Some(v1Config::ImOnlineConfig {
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
        pallet_contracts: Some(v1Config::ContractsConfig {
            current_schedule: contracts::Schedule {
                ..Default::default()
            },
            gas_price: 1 * MILLICENTS,
        }),
        group_Instance1: Some(v1::runtime::CommitteeMembershipConfig {
            active_members: vec![],
            phantom: Default::default(),
        }),
        committee_Instance1: Some(v1Config::PolymeshCommitteeConfig {
            vote_threshold: (1, 2),
            members: vec![
                IdentityId::from(3),
                IdentityId::from(4),
                IdentityId::from(5),
            ],
            phantom: Default::default(),
        }),
        group_Instance2: Some(v1::runtime::CddServiceProvidersConfig {
            // sp1, sp2, alice
            active_members: vec![
                IdentityId::from(1),
                IdentityId::from(2),
                IdentityId::from(42),
            ],
            phantom: Default::default(),
        }),
        protocol_fee: Some(v1Config::ProtocolFeeConfig {
            ..Default::default()
        }),
    }
}

fn v1_live_testnet_config() -> V1ChainSpec {
    // provide boot nodes
    let boot_nodes = vec![];
    V1ChainSpec::from_genesis(
		"Polymesh Live V1 Testnet",
        "live-testnet",
		ChainType::Live,
		v1_live_testnet_genesis,
		boot_nodes,
		None, // TODO: Need to provide telemetry URL where every validator telemetry can be seen
        None,
		Some(polymath_props()),
        Default::default(),
	)
}

fn v1_develop_testnet_genesis() -> v1Config::GenesisConfig {
    v1_testnet_genesis(
        vec![get_authority_keys_from_seed("Alice")],
        get_account_id_from_seed::<sr25519::Public>("Alice"),
        vec![
            get_account_id_from_seed::<sr25519::Public>("Alice"),
            get_account_id_from_seed::<sr25519::Public>("Bob"),
            get_account_id_from_seed::<sr25519::Public>("Alice//stash"),
        ],
        true,
    )
}

pub fn v1_develop_testnet_config() -> V1ChainSpec {
    // provide boot nodes
    let boot_nodes = vec![];
    V1ChainSpec::from_genesis(
		"Polymesh Develop V1 Testnet",
        "development-testnet",
		ChainType::Development,
		v1_develop_testnet_genesis,
		boot_nodes,
		None,
        None,
		Some(polymath_props()),
        Default::default(),
	)
}

fn v1_local_testnet_genesis() -> v1Config::GenesisConfig {
    v1_testnet_genesis(
        vec![
            get_authority_keys_from_seed("Alice"),
            get_authority_keys_from_seed("Bob"),
        ],
        get_account_id_from_seed::<sr25519::Public>("Alice"),
        vec![
            get_account_id_from_seed::<sr25519::Public>("Alice"),
            get_account_id_from_seed::<sr25519::Public>("Bob"),
            get_account_id_from_seed::<sr25519::Public>("Charlie"),
            get_account_id_from_seed::<sr25519::Public>("Dave"),
            get_account_id_from_seed::<sr25519::Public>("Alice//stash"),
            get_account_id_from_seed::<sr25519::Public>("Bob//stash"),
            get_account_id_from_seed::<sr25519::Public>("Charlie//stash"),
        ],
        true,
    )
}

pub fn v1_local_testnet_config() -> V1ChainSpec {
    // provide boot nodes
    let boot_nodes = vec![];
    V1ChainSpec::from_genesis(
		"Polymesh Local V1 Testnet",
        "local-testnet",
		ChainType::Local,
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
) -> v1Config::GenesisConfig {

    const STASH: u128 = 300 * POLY; //300 Poly
    const ENDOWMENT: u128 = 1_00_000 * POLY;

    v1Config::GenesisConfig {
        frame_system: Some(v1Config::SystemConfig {
            code: v1::WASM_BINARY.to_vec(),
            changes_trie_config: Default::default(),
        }),
        asset: Some(v1Config::AssetConfig {
            asset_creation_fee: 250,
            ticker_registration_fee: 250,
            ticker_registration_config: TickerRegistrationConfig {
                max_ticker_length: 12,
                registration_length: Some(5_184_000_000),
            },
            fee_collector: get_account_id_from_seed::<sr25519::Public>("Dave"),
        }),
        bridge: Some(v1Config::BridgeConfig {
            admin: get_account_id_from_seed::<sr25519::Public>("Alice"),
            creator: get_account_id_from_seed::<sr25519::Public>("Alice"),
            signatures_required: 0,
            signers: vec![],
            timelock: 10,
        }),
        identity: Some(v1Config::IdentityConfig {
            owner: get_account_id_from_seed::<sr25519::Public>("Dave"),
            identities: vec![
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
                    IdentityId::from(2),
                    IdentityId::from(2),
                    None,
                ),
                // Governance committee members
                (
                    get_account_id_from_seed::<sr25519::Public>("polymath_1"),
                    IdentityId::from(1),
                    IdentityId::from(3),
                    None,
                ),
                (
                    get_account_id_from_seed::<sr25519::Public>("polymath_2"),
                    IdentityId::from(1),
                    IdentityId::from(4),
                    None,
                ),
                (
                    get_account_id_from_seed::<sr25519::Public>("polymath_3"),
                    IdentityId::from(2),
                    IdentityId::from(5),
                    None,
                ),
                // Validators
                (
                    get_account_id_from_seed::<sr25519::Public>("operator_1//stash"),
                    IdentityId::from(2),
                    IdentityId::from(6),
                    None,
                ),
                (
                    get_account_id_from_seed::<sr25519::Public>("operator_2//stash"),
                    IdentityId::from(1),
                    IdentityId::from(7),
                    None,
                ),
                (
                    get_account_id_from_seed::<sr25519::Public>("operator_3//stash"),
                    IdentityId::from(1),
                    IdentityId::from(8),
                    None,
                ),
                (
                    get_account_id_from_seed::<sr25519::Public>("operator_4//stash"),
                    IdentityId::from(1),
                    IdentityId::from(9),
                    None,
                ),
                (
                    get_account_id_from_seed::<sr25519::Public>("operator_5//stash"),
                    IdentityId::from(1),
                    IdentityId::from(10),
                    None,
                )
            ],
            ..Default::default()
        }),
        simple_token: Some(v1Config::SimpleTokenConfig { creation_fee: 1000 }),
        balances: Some(v1Config::BalancesConfig {
            balances: endowed_accounts
                .iter()
                .cloned()
                .map(|k| (k, 1 << 55))
                .collect(),
        }),
        pallet_treasury: Some(Default::default()),
        pallet_indices: Some(v1Config::IndicesConfig { indices: vec![] }),
        pallet_sudo: Some(v1Config::SudoConfig { key: root_key }),
        pallet_session: Some(v1Config::SessionConfig {
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
        pallet_staking: Some(v1Config::StakingConfig {
            minimum_validator_count: 1,
            validator_count: 2,
            stakers: initial_authorities
                .iter()
                .map(|x| (x.0.clone(), x.1.clone(), STASH, v1::StakerStatus::Validator))
                .collect(),
            invulnerables: initial_authorities.iter().map(|x| x.0.clone()).collect(),
            slash_reward_fraction: v1::Perbill::from_percent(10),
            validator_commission: v1::Commission::Global(PerThing::from_rational_approximation(
                1u64, 4u64,
            )),
            min_bond_threshold: 0,
            ..Default::default()
        }),
        pallet_mips: Some(v1Config::MipsConfig {
            min_proposal_deposit: 5000,
            quorum_threshold: 100_000,
            proposal_duration: 50,
            proposal_cool_off_period: v1Hours * 6,
        }),
        pallet_im_online: Some(v1Config::ImOnlineConfig {
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
        pallet_contracts: Some(v1Config::ContractsConfig {
            current_schedule: contracts::Schedule {
                enable_println, // this should only be enabled on development chains
                ..Default::default()
            },
            gas_price: 1 * MILLICENTS,
        }),
        group_Instance1: Some(v1::runtime::CommitteeMembershipConfig {
            active_members: vec![],
            phantom: Default::default(),
        }),
        committee_Instance1: Some(v1::runtime::PolymeshCommitteeConfig {
            vote_threshold: (1, 2),
            members: vec![
                IdentityId::from(3),
                IdentityId::from(4),
                IdentityId::from(5),
            ],
            phantom: Default::default(),
        }),
        group_Instance2: Some(v1::runtime::CddServiceProvidersConfig {
            // sp1, sp2, alice
            active_members: vec![
                IdentityId::from(1),
                IdentityId::from(2),
                IdentityId::from(42),
            ],
            phantom: Default::default(),
        }),
        protocol_fee: Some(v1Config::ProtocolFeeConfig {
            ..Default::default()
        }),
    }
}