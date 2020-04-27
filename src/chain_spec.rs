use grandpa::AuthorityId as GrandpaId;
use im_online::sr25519::AuthorityId as ImOnlineId;
use polymesh_primitives::{AccountId, IdentityId, PosRatio, Signature};
use polymesh_runtime::{
    asset::TickerRegistrationConfig,
    config::{
        AssetConfig, BalancesConfig, BridgeConfig, ContractsConfig, GenesisConfig, IdentityConfig,
        ImOnlineConfig, IndicesConfig, PipsConfig, SessionConfig, SimpleTokenConfig, StakingConfig,
        SudoConfig, SystemConfig,
    },
    runtime::{
        CddServiceProvidersConfig, CommitteeMembershipConfig, PolymeshCommitteeConfig,
        ProtocolFeeConfig, TreasuryConfig,
    },
    Commission, OfflineSlashingParams, Perbill, SessionKeys, StakerStatus, WASM_BINARY,
};
use polymesh_runtime_common::{
    constants::{
        currency::{MILLICENTS, POLY},
        time::{DAYS, HOURS},
    },
    protocol_fee::ProtocolOp,
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

/// Specialized `ChainSpec`. This is a specialization of the general Substrate ChainSpec type.
pub type ChainSpec = sc_service::ChainSpec<GenesisConfig>;

/// The chain specification option. This is expected to come in from the CLI and
/// is little more than one of a number of alternatives which can easily be converted
/// from a string (`--chain=...`) into a `ChainSpec`.
#[derive(Clone, Debug)]
pub enum Alternative {
    /// Whatever the current runtime is, with just Alice as an auth.
    Development,
    /// Whatever the current runtime is, with simple Alice/Bob auths.
    LocalTestnet,
    /// The stats collector testnet
    StatsTestnet,
}

/// Helper function to generate a crypto pair from seed
pub fn get_from_seed<TPublic: Public>(seed: &str) -> <TPublic::Pair as Pair>::Public {
    TPublic::Pair::from_string(&format!("//{}", seed), None)
        .expect("static values are valid; qed")
        .public()
}

type AccountPublic = <Signature as Verify>::Signer;

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

impl Alternative {
    /// Get an actual chain config from one of the alternatives.
    pub(crate) fn load(self) -> Result<ChainSpec, String> {
        Ok(match self {
            Alternative::Development => ChainSpec::from_genesis(
                "Development",
                "dev",
                || {
                    testnet_genesis(
                        vec![get_authority_keys_from_seed("Alice")],
                        get_account_id_from_seed::<sr25519::Public>("Alice"),
                        vec![
                            get_account_id_from_seed::<sr25519::Public>("Alice"),
                            get_account_id_from_seed::<sr25519::Public>("Bob"),
                            get_account_id_from_seed::<sr25519::Public>("Alice//stash"),
                        ],
                        true,
                    )
                },
                vec![],
                None,
                None,
                Some(polymath_props()),
                None,
            ),
            Alternative::LocalTestnet => ChainSpec::from_genesis(
                "Local Testnet",
                "local_testnet",
                || {
                    testnet_genesis(
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
                },
                vec![],
                None,
                None,
                Some(polymath_props()),
                None,
            ),
            Alternative::StatsTestnet => ChainSpec::from_genesis(
                "Stats Testnet",
                "stats-testnet",
                || {
                    testnet_genesis(
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
                        true,
                    )
                },
                vec![],
                None,
                None,
                Some(polymath_props()),
                None,
            ),
        })
    }

    pub(crate) fn from(s: &str) -> Option<Self> {
        match s {
            "dev" => Some(Alternative::Development),
            "" | "local" => Some(Alternative::LocalTestnet),
            "stats-testnet" => Some(Alternative::StatsTestnet),
            _ => None,
        }
    }
}

fn polymath_props() -> Properties {
    json!({"tokenDecimals": 6, "tokenSymbol": "POLYX" })
        .as_object()
        .unwrap()
        .clone()
}

fn session_keys(
    grandpa: GrandpaId,
    babe: BabeId,
    im_online: ImOnlineId,
    authority_discovery: AuthorityDiscoveryId,
) -> SessionKeys {
    SessionKeys {
        babe,
        grandpa,
        im_online,
        authority_discovery,
    }
}

fn testnet_genesis(
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
    const STASH: u128 = 30_000_000_000 * POLY; //30G Poly
    let _desired_seats = (endowed_accounts.len() / 2 - initial_authorities.len()) as u32;
    GenesisConfig {
        frame_system: Some(SystemConfig {
            code: WASM_BINARY.to_vec(),
            changes_trie_config: Default::default(),
        }),
        asset: Some(AssetConfig {
            asset_creation_fee: 250,
            ticker_registration_fee: 250,
            ticker_registration_config: TickerRegistrationConfig {
                max_ticker_length: 12,
                registration_length: Some(5_184_000_000),
            },
            fee_collector: get_account_id_from_seed::<sr25519::Public>("Dave"),
        }),
        bridge: Some(BridgeConfig {
            admin: get_account_id_from_seed::<sr25519::Public>("Alice"),
            creator: get_account_id_from_seed::<sr25519::Public>("Alice"),
            signatures_required: 0,
            signers: vec![],
            timelock: 10,
        }),
        identity: Some(IdentityConfig {
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
        simple_token: Some(SimpleTokenConfig { creation_fee: 1000 }),
        balances: Some(BalancesConfig {
            balances: endowed_accounts
                .iter()
                .cloned()
                .map(|k| (k, 1 << 55))
                .collect(),
        }),
        treasury: Some(TreasuryConfig { balance: 0 }),
        pallet_indices: Some(IndicesConfig { indices: vec![] }),
        pallet_sudo: Some(SudoConfig { key: root_key }),
        pallet_session: Some(SessionConfig {
            keys: initial_authorities
                .iter()
                .map(|x| {
                    (
                        x.0.clone(),
                        x.0.clone(),
                        session_keys(x.2.clone(), x.3.clone(), x.4.clone(), x.5.clone()),
                    )
                })
                .collect::<Vec<_>>(),
        }),
        pallet_staking: Some(StakingConfig {
            minimum_validator_count: 1,
            validator_count: 2,
            stakers: initial_authorities
                .iter()
                .map(|x| (x.0.clone(), x.1.clone(), STASH, StakerStatus::Validator))
                .collect(),
            invulnerables: initial_authorities.iter().map(|x| x.0.clone()).collect(),
            slash_reward_fraction: Perbill::from_percent(10),
            validator_commission: Commission::Global(PerThing::from_rational_approximation(
                1u64, 4u64,
            )),
            min_bond_threshold: 0,
            ..Default::default()
        }),
        pallet_pips: Some(PipsConfig {
            prune_historical_pips: false,
            min_proposal_deposit: 5000,
            quorum_threshold: 100_000,
            proposal_duration: 50,
            proposal_cool_off_period: HOURS * 6,
            default_enactment_period: DAYS * 7,
        }),
        pallet_im_online: Some(ImOnlineConfig {
            slashing_params: OfflineSlashingParams {
                max_offline_percent: 10u32,
                constant: 3u32,
                max_slash_percent: 7u32,
            },
            ..Default::default()
        }),
        pallet_authority_discovery: Some(Default::default()),
        pallet_babe: Some(Default::default()),
        pallet_grandpa: Some(Default::default()),
        pallet_contracts: Some(ContractsConfig {
            current_schedule: contracts::Schedule {
                enable_println, // this should only be enabled on development chains
                ..Default::default()
            },
            gas_price: 1 * MILLICENTS,
        }),
        group_Instance1: Some(CommitteeMembershipConfig {
            active_members: vec![],
            phantom: Default::default(),
        }),
        committee_Instance1: Some(PolymeshCommitteeConfig {
            vote_threshold: (1, 2),
            members: vec![
                IdentityId::from(3),
                IdentityId::from(4),
                IdentityId::from(5),
            ],
            phantom: Default::default(),
        }),
        group_Instance2: Some(CddServiceProvidersConfig {
            // sp1, sp2, alice
            active_members: vec![
                IdentityId::from(1),
                IdentityId::from(2),
                IdentityId::from(42),
            ],
            phantom: Default::default(),
        }),
        protocol_fee: Some(ProtocolFeeConfig {
            base_fees: vec![
                (ProtocolOp::AssetCreateToken, 10_000 * 1_000_000),
                (ProtocolOp::AssetRegisterTicker, 2_500 * 1_000_000),
            ],
            coefficient: PosRatio(1, 1),
        }),
    }
}

pub fn load_spec(id: &str) -> Result<Option<ChainSpec>, String> {
    Ok(match Alternative::from(id) {
        Some(spec) => Some(spec.load()?),
        None => None,
    })
}
