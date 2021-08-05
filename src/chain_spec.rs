use codec::{Decode, Encode};
use grandpa::AuthorityId as GrandpaId;
use pallet_asset::{ClassicTickerImport, TickerRegistrationConfig};
use pallet_bridge::BridgeTx;
use pallet_im_online::sr25519::AuthorityId as ImOnlineId;
use pallet_staking::StakerStatus;
use polymesh_common_utilities::{
    constants::{currency::POLY, REWARDS_MODULE_ID, TREASURY_MODULE_ID},
    protocol_fee::ProtocolOp,
    SystematicIssuers,
};
use polymesh_primitives::{
    identity_id::GenesisIdentityRecord, AccountId, Balance, HexAccountId, IdentityId, Moment,
    PosRatio, SecondaryKey, Signatory, Signature, Ticker,
};
use sc_chain_spec::ChainType;
use sc_service::Properties;
use sc_telemetry::TelemetryEndpoints;
use serde_json::json;
use sp_authority_discovery::AuthorityId as AuthorityDiscoveryId;
use sp_consensus_babe::AuthorityId as BabeId;
use sp_core::{sr25519, Pair, Public, H256};
use sp_runtime::{
    traits::{AccountIdConversion, IdentifyAccount, Verify},
    PerThing,
};
#[cfg(feature = "std")]
use sp_runtime::{Deserialize, Serialize};
use std::convert::TryInto;

// The URL for the telemetry server.
const STAGING_TELEMETRY_URL: &str = "wss://telemetry.polymesh.live/submit/";
const BRIDGE_LOCK_HASH: &str = "0x1000000000000000000000000000000000000000000000000000000000000001";
const REWARDS_LOCK_HASH: &str =
    "0x1000000000000000000000000000000000000000000000000000000000000002";

/// Generate a crypto pair from seed.
pub fn get_from_seed<TPublic: Public>(seed: &str) -> <TPublic::Pair as Pair>::Public {
    TPublic::Pair::from_string(&format!("//{}", seed), None)
        .expect("static values are valid; qed")
        .public()
}

type AccountPublic = <Signature as Verify>::Signer;

/// Generate an account ID from seed.
pub fn get_account_id_from_seed<TPublic: Public>(seed: &str) -> AccountId
where
    AccountPublic: From<<TPublic::Pair as Pair>::Public>,
{
    AccountPublic::from(get_from_seed::<TPublic>(seed)).into_account()
}

fn seeded_acc_id(seed: &str) -> AccountId {
    get_account_id_from_seed::<sr25519::Public>(seed)
}

/// Generate an Aura authority key.
pub fn get_authority_keys_from_seed(s: &str, uniq: bool) -> InitialAuth {
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

fn polymath_props(ss58: u8) -> Properties {
    json!({ "ss58Format": ss58, "tokenDecimals": 6, "tokenSymbol": "POLYX" })
        .as_object()
        .unwrap()
        .clone()
}

macro_rules! session_keys {
    () => {
        fn session_keys(
            grandpa: GrandpaId,
            babe: BabeId,
            im_online: ImOnlineId,
            authority_discovery: AuthorityDiscoveryId,
        ) -> rt::SessionKeys {
            rt::SessionKeys {
                babe,
                grandpa,
                im_online,
                authority_discovery,
            }
        }
    };
}

macro_rules! asset {
    () => {
        pallet_asset::GenesisConfig {
            ticker_registration_config: ticker_registration_config(),
            classic_migration_tconfig: TickerRegistrationConfig {
                max_ticker_length: 12,
                // Reservations will expire at end of 2021
                registration_length: Some(1640995199999),
            },
            /*
            versions: vec![
                (SmartExtensionType::TransferManager, 5000),
                (SmartExtensionType::Offerings, 5000),
                (SmartExtensionType::SmartWallet, 5000),
            ],
            */
            // Always use the first id, whomever that may be.
            classic_migration_contract_did: IdentityId::from(1),
            classic_migration_tickers: classic_reserved_tickers(),
            reserved_country_currency_codes: currency_codes(),
        }
    };
}

fn ticker_registration_config() -> TickerRegistrationConfig<Moment> {
    TickerRegistrationConfig {
        max_ticker_length: 12,
        registration_length: Some(5_184_000_000),
    }
}

fn currency_codes() -> Vec<Ticker> {
    // Fiat Currency Struct
    #[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
    #[derive(Encode, Decode, Clone, Debug, PartialEq, Eq)]
    pub struct FiatCurrency<String> {
        pub codes: Vec<String>,
    }

    let currency_file = include_str!("data/currency_symbols.json");
    let currency_data: FiatCurrency<String> = serde_json::from_str(&currency_file).unwrap();
    currency_data
        .codes
        .into_iter()
        .map(|y| y.as_bytes().try_into().unwrap())
        .collect()
}

#[allow(unreachable_code)]
fn classic_reserved_tickers() -> Vec<ClassicTickerImport> {
    #[cfg(feature = "runtime-benchmarks")]
    return Vec::new();

    let reserved_tickers_file = include_str!("data/reserved_classic_tickers.json");
    serde_json::from_str(&reserved_tickers_file).unwrap()
}

macro_rules! checkpoint {
    () => {{
        // We use a weekly complexity. That is, >= 7 days apart per CP is OK.
        use polymesh_primitives::calendar::{CalendarPeriod, CalendarUnit::Week};
        let period = CalendarPeriod {
            unit: Week,
            amount: 1,
        };
        pallet_asset::checkpoint::GenesisConfig {
            schedules_max_complexity: period.complexity(),
        }
    }};
}

type InitialAuth = (
    AccountId,
    AccountId,
    GrandpaId,
    BabeId,
    ImOnlineId,
    AuthorityDiscoveryId,
);

fn adjust_last<'a>(bytes: &'a mut [u8], n: u8) -> &'a str {
    bytes[bytes.len() - 1] = n + b'0';
    core::str::from_utf8(bytes).unwrap()
}

const BOOTSTRAP_STASH: u128 = 10_000 * POLY;
const BOOTSTRAP_TREASURY: u128 = 30_000_000 * POLY;
// Needed for integration tests
const BOOTSTRAP_BIG: u128 = 30_000_000 * POLY;

#[derive(Clone)]
struct BridgeLockId {
    nonce: u32,
    tx_hash: H256,
}

impl BridgeLockId {
    fn new(nonce: u32, hash: &'static str) -> Self {
        let offset = if hash.starts_with("0x") { 2 } else { 0 };
        let stripped_hash = &hash[offset..];
        let hash_vec: Vec<u8> = rustc_hex::FromHex::from_hex(stripped_hash)
            .expect("Failed to decode transaction hash (Invalid hex Value)");
        let hash_array: [u8; 32] = hash_vec
            .try_into()
            .expect("Failed to decode transaction hash (Invalid hash length)");
        Self {
            nonce,
            tx_hash: hash_array.into(),
        }
    }

    fn generate_bridge_locks(count: u32) -> Vec<Self> {
        const HASH: &str = "0x000000000000000000000000000000000000000000000000000000000000dead";
        (0..count).map(|x| Self::new(x + 100, HASH)).collect()
    }
}

fn genesis_processed_data(
    initial_authorities: &Vec<InitialAuth>,
    root_key: AccountId,
    treasury_bridge_lock: BridgeLockId,
    key_bridge_locks: Vec<BridgeLockId>,
) -> (
    Vec<GenesisIdentityRecord<AccountId>>,
    Vec<(
        IdentityId,
        AccountId,
        AccountId,
        u128,
        StakerStatus<AccountId>,
    )>,
    Vec<BridgeTx<AccountId>>,
) {
    // Identities and their roles
    // 1 = GC + UC
    // 2 = GC
    // 3 = GC + TC
    // 4 = Operator
    // 5 = Bridge + Sudo
    let mut identities = Vec::with_capacity(5);
    let mut keys = Vec::with_capacity(5 + 2 * initial_authorities.len());

    let mut create_id = |nonce: u8, primary_key: AccountId| {
        keys.push(primary_key.clone());
        identities.push(GenesisIdentityRecord::new(nonce, primary_key));
    };

    // Creating Identities 1-4 (GC + Operators)
    for i in 1..5u8 {
        create_id(i, seeded_acc_id(adjust_last(&mut { *b"polymath_0" }, i)));
    }

    // Creating identity for sudo + bridge admin
    create_id(5u8, root_key);

    // 3 operators, all self staking at genesis
    let mut stakers = Vec::with_capacity(initial_authorities.len());
    for (stash, controller, ..) in initial_authorities {
        stakers.push((
            IdentityId::from(4), // All operators have the same Identity
            stash.clone(),
            controller.clone(),
            BOOTSTRAP_STASH / 2,
            pallet_staking::StakerStatus::Validator,
        ));
        // Make stash and controller 4th Identity's secondary keys.
        let mut push_key = |key: &AccountId| {
            identities[3]
                .secondary_keys
                .push(SecondaryKey::from_account_id_with_full_perms(key.clone()));
            keys.push(key.clone());
        };
        push_key(stash);
        push_key(controller);
    }

    // Give CDD issuer to operator since it won't receive CDD from the group automatically
    identities[3]
        .issuers
        .push(SystematicIssuers::CDDProvider.as_id());

    // Accumulate bridge transactions
    let mut complete_txs: Vec<_> = key_bridge_locks
        .iter()
        .cloned()
        .zip(keys.iter().cloned())
        .map(|(BridgeLockId { nonce, tx_hash }, recipient)| BridgeTx {
            nonce,
            recipient,
            amount: BOOTSTRAP_BIG,
            tx_hash,
        })
        .collect();
    complete_txs.push(BridgeTx {
        nonce: treasury_bridge_lock.nonce,
        recipient: TREASURY_MODULE_ID.into_account(),
        amount: BOOTSTRAP_TREASURY,
        tx_hash: treasury_bridge_lock.tx_hash,
    });
    (identities, stakers, complete_txs)
}

fn dev_genesis_processed_data(
    initial_authorities: &Vec<InitialAuth>,
    rewards_bridge_lock: BridgeLockId,
    key_bridge_locks: Vec<BridgeLockId>,
    other_funded_accounts: Vec<AccountId>,
) -> (
    GenesisIdentityRecord<AccountId>,
    Vec<(
        IdentityId,
        AccountId,
        AccountId,
        u128,
        StakerStatus<AccountId>,
    )>,
    Vec<BridgeTx<AccountId>>,
) {
    let mut identity = GenesisIdentityRecord::new(1u8, initial_authorities[0].0.clone());

    let mut stakers = Vec::with_capacity(initial_authorities.len());
    identity
        .secondary_keys
        .reserve(initial_authorities.len() * 2 + other_funded_accounts.len());
    let mut add_sk = |acc| {
        identity
            .secondary_keys
            .push(SecondaryKey::from_account_id_with_full_perms(acc))
    };
    for (stash, controller, ..) in initial_authorities {
        stakers.push((
            IdentityId::from(1),
            stash.clone(),
            controller.clone(),
            BOOTSTRAP_STASH / 2,
            pallet_staking::StakerStatus::Validator,
        ));
        add_sk(stash.clone());
        add_sk(controller.clone());
    }

    for account in other_funded_accounts {
        add_sk(account);
    }

    // Accumulate bridge transactions
    let mut complete_txs: Vec<_> = key_bridge_locks
        .iter()
        .cloned()
        .zip(
            identity
                .secondary_keys
                .iter()
                .map(|sk| sk.signer.as_account().unwrap().clone()),
        )
        .map(|(BridgeLockId { nonce, tx_hash }, recipient)| BridgeTx {
            nonce,
            recipient,
            amount: BOOTSTRAP_TREASURY,
            tx_hash,
        })
        .collect();

    complete_txs.push(BridgeTx {
        nonce: rewards_bridge_lock.nonce,
        recipient: REWARDS_MODULE_ID.into_account(),
        amount: itn_rewards().into_iter().map(|(_, b)| b + (1 * POLY)).sum(),
        tx_hash: rewards_bridge_lock.tx_hash,
    });

    // The 0th key is the primary key
    identity.secondary_keys.remove(0);

    (identity, stakers, complete_txs)
}

fn bridge_signers() -> Vec<Signatory<AccountId>> {
    let signer =
        |seed| Signatory::Account(AccountId::from(get_from_seed::<sr25519::Public>(seed).0));
    vec![
        signer("relay_1"),
        signer("relay_2"),
        signer("relay_3"),
        signer("relay_4"),
        signer("relay_5"),
    ]
}

fn frame(wasm_binary: Option<&[u8]>) -> frame_system::GenesisConfig {
    frame_system::GenesisConfig {
        code: wasm_binary.expect("WASM binary was not generated").to_vec(),
        changes_trie_config: Default::default(),
    }
}

macro_rules! session {
    ($inits:expr, $build:expr) => {
        pallet_session::GenesisConfig {
            keys: $inits
                .iter()
                .map(|x| {
                    let sks = $build(x.2.clone(), x.3.clone(), x.4.clone(), x.5.clone());
                    (x.0.clone(), x.0.clone(), sks)
                })
                .collect::<Vec<_>>(),
        }
    };
}

macro_rules! staking {
    ($auths:expr, $stakers:expr, $cap:expr) => {
        pallet_staking::GenesisConfig {
            minimum_validator_count: 1,
            validator_count: 20,
            validator_commission_cap: $cap,
            stakers: $stakers,
            invulnerables: vec![],
            slash_reward_fraction: sp_runtime::Perbill::from_percent(10),
            min_bond_threshold: 0,
            ..Default::default()
        }
    };
}

macro_rules! pips {
    ($period:expr, $limit: expr) => {
        pallet_pips::GenesisConfig {
            prune_historical_pips: false,
            min_proposal_deposit: 0,
            default_enactment_period: $period,
            max_pip_skip_count: 2,
            active_pip_limit: $limit,
            pending_pip_expiry: <_>::default(),
        }
    };
}

macro_rules! group_membership {
    ($($member:expr),*) => {
        pallet_group::GenesisConfig {
            active_members_limit: 20,
            active_members: vec![$(IdentityId::from($member)),*],
            phantom: Default::default(),
        }
    };
}

macro_rules! corporate_actions {
    () => {
        pallet_corporate_actions::GenesisConfig {
            max_details_length: 1024,
        }
    };
}

macro_rules! committee {
    ($rc:expr) => {
        committee!($rc, (1, 2))
    };
    ($rc:expr, $vote:expr) => {
        pallet_committee::GenesisConfig {
            vote_threshold: $vote,
            members: vec![],
            release_coordinator: IdentityId::from($rc),
            expires_after: <_>::default(),
            phantom: Default::default(),
        }
    };
}

fn protocol_fees() -> Vec<(ProtocolOp, u128)> {
    vec![
        (ProtocolOp::AssetCreateAsset, 2_500 * 1_000_000),
        (ProtocolOp::AssetRegisterTicker, 500 * 1_000_000),
    ]
}

macro_rules! protocol_fee {
    () => {
        pallet_protocol_fee::GenesisConfig {
            base_fees: protocol_fees(),
            coefficient: PosRatio(1, 1),
        }
    };
}

macro_rules! rewards {
    () => {
        pallet_rewards::GenesisConfig {
            itn_rewards: itn_rewards(),
        }
    };
}

#[allow(unreachable_code)]
fn itn_rewards() -> Vec<(AccountId, Balance)> {
    #[cfg(feature = "runtime-benchmarks")]
    return Vec::new();

    let itn_rewards_file = include_str!("data/itn_rewards.json");
    serde_json::from_str::<Vec<(HexAccountId, Balance)>>(&itn_rewards_file)
        .unwrap()
        .into_iter()
        .map(|(acc, bal)| (acc.0.into(), bal))
        .collect()
}

pub mod general {
    use super::*;
    use polymesh_runtime_develop::{self as rt, constants::time};

    pub type ChainSpec = sc_service::GenericChainSpec<rt::runtime::GenesisConfig>;

    session_keys!();

    fn genesis(
        initial_authorities: Vec<InitialAuth>,
        root_key: AccountId,
        _enable_println: bool,
        rewards_bridge_lock: BridgeLockId,
        key_bridge_locks: Vec<BridgeLockId>,
        other_funded_accounts: Vec<AccountId>,
    ) -> rt::runtime::GenesisConfig {
        let (identity, stakers, complete_txs) = dev_genesis_processed_data(
            &initial_authorities,
            rewards_bridge_lock,
            key_bridge_locks,
            other_funded_accounts,
        );

        rt::runtime::GenesisConfig {
            frame_system: Some(frame(rt::WASM_BINARY)),
            pallet_asset: Some(asset!()),
            pallet_checkpoint: Some(checkpoint!()),
            pallet_identity: Some(pallet_identity::GenesisConfig {
                identities: vec![identity],
                ..Default::default()
            }),
            pallet_balances: Some(Default::default()),
            pallet_bridge: Some(pallet_bridge::GenesisConfig {
                admin: initial_authorities[0].1.clone(),
                creator: initial_authorities[0].1.clone(),
                signatures_required: 1,
                signers: bridge_signers(),
                timelock: 10,
                bridge_limit: (100_000_000 * POLY, 1000),
                complete_txs,
            }),
            pallet_indices: Some(pallet_indices::GenesisConfig { indices: vec![] }),
            pallet_sudo: Some(pallet_sudo::GenesisConfig { key: root_key }),
            pallet_session: Some(session!(initial_authorities, session_keys)),
            pallet_staking: Some(staking!(
                initial_authorities,
                stakers,
                PerThing::from_rational_approximation(1u64, 4u64)
            )),
            pallet_pips: Some(pips!(time::MINUTES, 25)),
            pallet_im_online: Some(Default::default()),
            pallet_authority_discovery: Some(Default::default()),
            pallet_babe: Some(Default::default()),
            pallet_grandpa: Some(Default::default()),
            /*
            pallet_contracts: Some(pallet_contracts::GenesisConfig {
                current_schedule: pallet_contracts::Schedule {
                    enable_println, // this should only be enabled on development chains
                    ..Default::default()
                },
            }),
            */
            // Governance Council:
            pallet_group_Instance1: Some(group_membership!(1)),
            pallet_committee_Instance1: Some(committee!(1)),
            // CDD providers
            pallet_group_Instance2: Some(group_membership!(1)),
            // Technical Committee:
            pallet_group_Instance3: Some(group_membership!(1)),
            pallet_committee_Instance3: Some(committee!(1)),
            // Upgrade Committee:
            pallet_group_Instance4: Some(group_membership!(1)),
            pallet_committee_Instance4: Some(committee!(1)),
            pallet_protocol_fee: Some(protocol_fee!()),
            pallet_settlement: Some(Default::default()),
            pallet_multisig: Some(pallet_multisig::GenesisConfig {
                transaction_version: 1,
            }),
            pallet_corporate_actions: Some(corporate_actions!()),
            pallet_rewards: Some(rewards!()),
        }
    }

    fn develop_genesis() -> rt::runtime::GenesisConfig {
        genesis(
            vec![get_authority_keys_from_seed("Alice", false)],
            seeded_acc_id("Alice"),
            true,
            BridgeLockId::new(2, REWARDS_LOCK_HASH),
            BridgeLockId::generate_bridge_locks(20),
            vec![
                seeded_acc_id("Bob"),
                seeded_acc_id("Charlie"),
                seeded_acc_id("Dave"),
                seeded_acc_id("Eve"),
            ],
        )
    }

    fn config(
        name: &str,
        id: &str,
        ctype: ChainType,
        genesis: impl 'static + Sync + Send + Fn() -> rt::runtime::GenesisConfig,
    ) -> ChainSpec {
        let props = Some(polymath_props(42));
        ChainSpec::from_genesis(name, id, ctype, genesis, vec![], None, None, props, None)
    }

    pub fn develop_config() -> ChainSpec {
        config(
            "Development",
            "dev",
            ChainType::Development,
            develop_genesis,
        )
    }

    fn local_genesis() -> rt::runtime::GenesisConfig {
        genesis(
            vec![
                get_authority_keys_from_seed("Alice", false),
                get_authority_keys_from_seed("Bob", false),
            ],
            seeded_acc_id("Alice"),
            true,
            BridgeLockId::new(2, REWARDS_LOCK_HASH),
            BridgeLockId::generate_bridge_locks(20),
            vec![
                seeded_acc_id("Charlie"),
                seeded_acc_id("Dave"),
                seeded_acc_id("Eve"),
            ],
        )
    }

    pub fn local_config() -> ChainSpec {
        config(
            "Local Development",
            "local_dev",
            ChainType::Local,
            local_genesis,
        )
    }
}

pub mod testnet {
    use super::*;
    use polymesh_runtime_testnet::{self as rt, constants::time};

    pub type ChainSpec = sc_service::GenericChainSpec<rt::runtime::GenesisConfig>;

    session_keys!();

    fn genesis(
        initial_authorities: Vec<InitialAuth>,
        root_key: AccountId,
        _enable_println: bool,
        treasury_bridge_lock: BridgeLockId,
        key_bridge_locks: Vec<BridgeLockId>,
    ) -> rt::runtime::GenesisConfig {
        let (identities, stakers, complete_txs) = genesis_processed_data(
            &initial_authorities,
            root_key.clone(),
            treasury_bridge_lock,
            key_bridge_locks,
        );

        rt::runtime::GenesisConfig {
            frame_system: Some(frame(rt::WASM_BINARY)),
            pallet_asset: Some(asset!()),
            pallet_checkpoint: Some(checkpoint!()),
            pallet_identity: Some(pallet_identity::GenesisConfig {
                identities,
                ..Default::default()
            }),
            pallet_balances: Some(Default::default()),
            pallet_bridge: Some(pallet_bridge::GenesisConfig {
                admin: seeded_acc_id("polymath_1"),
                creator: seeded_acc_id("polymath_1"),
                signatures_required: 3,
                signers: bridge_signers(),
                timelock: time::MINUTES * 15,
                bridge_limit: (30_000_000_000, time::DAYS),
                complete_txs,
            }),
            pallet_indices: Some(pallet_indices::GenesisConfig { indices: vec![] }),
            pallet_sudo: Some(pallet_sudo::GenesisConfig { key: root_key }),
            pallet_session: Some(session!(initial_authorities, session_keys)),
            pallet_staking: Some(staking!(initial_authorities, stakers, PerThing::zero())),
            pallet_pips: Some(pips!(time::DAYS * 7, 1000)),
            pallet_im_online: Some(Default::default()),
            pallet_authority_discovery: Some(Default::default()),
            pallet_babe: Some(Default::default()),
            pallet_grandpa: Some(Default::default()),
            /*
            pallet_contracts: Some(pallet_contracts::GenesisConfig {
                current_schedule: pallet_contracts::Schedule {
                    enable_println, // this should only be enabled on development chains
                    ..Default::default()
                },
            }),
            */
            // Governing council
            pallet_group_Instance1: Some(group_membership!(1, 2, 3, 5)),
            pallet_committee_Instance1: Some(committee!(1, (2, 4))),
            // CDD providers
            pallet_group_Instance2: Some(group_membership!(1, 2, 3, 5)),
            // Technical Committee:
            pallet_group_Instance3: Some(group_membership!(3, 5)),
            pallet_committee_Instance3: Some(committee!(5)),
            // Upgrade Committee:
            pallet_group_Instance4: Some(group_membership!(1, 5)),
            pallet_committee_Instance4: Some(committee!(5)),
            pallet_protocol_fee: Some(protocol_fee!()),
            pallet_settlement: Some(Default::default()),
            pallet_multisig: Some(pallet_multisig::GenesisConfig {
                transaction_version: 1,
            }),
            pallet_corporate_actions: Some(corporate_actions!()),
            pallet_rewards: Some(rewards!()),
        }
    }

    fn develop_genesis() -> rt::runtime::GenesisConfig {
        genesis(
            vec![get_authority_keys_from_seed("Bob", false)],
            seeded_acc_id("Alice"),
            true,
            BridgeLockId::new(1, BRIDGE_LOCK_HASH),
            BridgeLockId::generate_bridge_locks(20),
        )
    }

    pub fn develop_config() -> ChainSpec {
        // provide boot nodes
        let boot_nodes = vec![];
        ChainSpec::from_genesis(
            "Polymesh Testnet Develop",
            "dev_testnet",
            ChainType::Development,
            develop_genesis,
            boot_nodes,
            None,
            None,
            Some(polymath_props(42)),
            Default::default(),
        )
    }

    fn local_genesis() -> rt::runtime::GenesisConfig {
        genesis(
            vec![
                get_authority_keys_from_seed("Alice", false),
                get_authority_keys_from_seed("Bob", false),
            ],
            seeded_acc_id("Alice"),
            true,
            BridgeLockId::new(1, BRIDGE_LOCK_HASH),
            BridgeLockId::generate_bridge_locks(20),
        )
    }

    pub fn local_config() -> ChainSpec {
        // provide boot nodes
        let boot_nodes = vec![];
        ChainSpec::from_genesis(
            "Polymesh Testnet Local",
            "local_testnet",
            ChainType::Local,
            local_genesis,
            boot_nodes,
            None,
            None,
            Some(polymath_props(42)),
            Default::default(),
        )
    }
}

pub mod polymesh_itn {
    use super::*;
    use polymesh_runtime_itn::{self as rt, constants::time};

    pub type ChainSpec = sc_service::GenericChainSpec<rt::runtime::GenesisConfig>;

    session_keys!();

    fn genesis(
        initial_authorities: Vec<InitialAuth>,
        root_key: AccountId,
        _enable_println: bool,
        treasury_bridge_lock: BridgeLockId,
        key_bridge_locks: Vec<BridgeLockId>,
    ) -> rt::runtime::GenesisConfig {
        let (identities, stakers, complete_txs) = genesis_processed_data(
            &initial_authorities,
            root_key.clone(),
            treasury_bridge_lock,
            key_bridge_locks,
        );

        rt::runtime::GenesisConfig {
            frame_system: Some(frame(rt::WASM_BINARY)),
            pallet_asset: Some(asset!()),
            pallet_checkpoint: Some(checkpoint!()),
            pallet_identity: Some(pallet_identity::GenesisConfig {
                identities,
                ..Default::default()
            }),
            pallet_balances: Some(Default::default()),
            pallet_bridge: Some(pallet_bridge::GenesisConfig {
                admin: root_key.clone(),
                creator: root_key.clone(),
                signatures_required: 3,
                signers: bridge_signers(),
                timelock: time::MINUTES * 15,
                bridge_limit: (100_000_000_000, 365 * time::DAYS),
                complete_txs,
            }),
            pallet_indices: Some(pallet_indices::GenesisConfig { indices: vec![] }),
            pallet_sudo: Some(pallet_sudo::GenesisConfig { key: root_key }),
            pallet_session: Some(session!(initial_authorities, session_keys)),
            pallet_staking: Some(staking!(
                initial_authorities,
                stakers,
                PerThing::from_rational_approximation(1u64, 10u64)
            )),
            pallet_pips: Some(pips!(time::DAYS * 30, 1000)),
            pallet_im_online: Some(Default::default()),
            pallet_authority_discovery: Some(Default::default()),
            pallet_babe: Some(Default::default()),
            pallet_grandpa: Some(Default::default()),
            /*
            pallet_contracts: Some(pallet_contracts::GenesisConfig {
                current_schedule: pallet_contracts::Schedule {
                    enable_println, // this should only be enabled on development chains
                    ..Default::default()
                },
            }),
            */
            // Governing council
            pallet_group_Instance1: Some(group_membership!(1, 2, 3)), // 3 GC members
            pallet_committee_Instance1: Some(committee!(1, (2, 3))),  // RC = 1, 2/3 votes required
            // CDD providers
            pallet_group_Instance2: Some(Default::default()), // No CDD provider
            // Technical Committee:
            pallet_group_Instance3: Some(group_membership!(3, 4, 5)), // One GC member + genesis operator + Bridge Multisig
            pallet_committee_Instance3: Some(committee!(3)),          // RC = 3, 1/2 votes required
            // Upgrade Committee:
            pallet_group_Instance4: Some(group_membership!(1)), // One GC member
            pallet_committee_Instance4: Some(committee!(1)),    // RC = 1, 1/2 votes required
            pallet_protocol_fee: Some(protocol_fee!()),
            pallet_settlement: Some(Default::default()),
            pallet_multisig: Some(pallet_multisig::GenesisConfig {
                transaction_version: 1,
            }),
            pallet_corporate_actions: Some(corporate_actions!()),
            pallet_rewards: Some(rewards!()),
        }
    }

    fn bootstrap_genesis() -> rt::runtime::GenesisConfig {
        genesis(
            vec![
                get_authority_keys_from_seed("Alice", false),
                get_authority_keys_from_seed("Bob", false),
                get_authority_keys_from_seed("Charlie", false),
            ],
            seeded_acc_id("polymath_5"),
            false,
            BridgeLockId::new(
                1,
                "0x000000000000000000000000000000000000000000000000000000000f0b41ae",
            ),
            BridgeLockId::generate_bridge_locks(20),
        )
    }

    pub fn bootstrap_config() -> ChainSpec {
        // provide boot nodes
        let boot_nodes = vec![
            "/dns4/itn-bootnode-1.polymesh.live/tcp/30333/p2p/12D3KooWAKwaVWS7BUypNyCDwCEeqSgn4vPUtJyMJesbrdkTnuBE".parse().expect("Unable to parse bootnode"),
            "/dns4/itn-bootnode-2.polymesh.live/tcp/30333/p2p/12D3KooWGqNUAnt1uRNjM5EP49wGN8eb6VnBUfpRLr1Ln8LMQjDe".parse().expect("Unable to parse bootnode"),
            "/dns4/itn-bootnode-3.polymesh.live/tcp/30333/p2p/12D3KooWFYsTF3oVu8jywC13hMFwzf9n8MFr2pBWRdyDYyWKiGnq".parse().expect("Unable to parse bootnode"),
        ];
        ChainSpec::from_genesis(
            "Polymesh ITN",
            "polymesh_itn",
            ChainType::Live,
            bootstrap_genesis,
            boot_nodes,
            Some(
                TelemetryEndpoints::new(vec![(STAGING_TELEMETRY_URL.to_string(), 0)])
                    .expect("ITN bootstrap telemetry url is valid; qed"),
            ),
            Some(&*"/polymath/itn"),
            Some(polymath_props(12)),
            Default::default(),
        )
    }

    fn develop_genesis() -> rt::runtime::GenesisConfig {
        genesis(
            vec![get_authority_keys_from_seed("Alice", false)],
            seeded_acc_id("Eve"),
            true,
            BridgeLockId::new(
                1,
                "0x000000000000000000000000000000000000000000000000000000000f0b41ae",
            ),
            BridgeLockId::generate_bridge_locks(20),
        )
    }

    pub fn develop_config() -> ChainSpec {
        // provide boot nodes
        let boot_nodes = vec![];
        ChainSpec::from_genesis(
            "Polymesh ITN Develop",
            "dev_itn",
            ChainType::Development,
            develop_genesis,
            boot_nodes,
            None,
            Some(&*"/polymath/develop/1"),
            Some(polymath_props(12)),
            Default::default(),
        )
    }

    fn local_genesis() -> rt::runtime::GenesisConfig {
        genesis(
            vec![
                get_authority_keys_from_seed("Alice", false),
                get_authority_keys_from_seed("Bob", false),
            ],
            seeded_acc_id("Eve"),
            true,
            BridgeLockId::new(
                1,
                "0x000000000000000000000000000000000000000000000000000000000f0b41ae",
            ),
            BridgeLockId::generate_bridge_locks(20),
        )
    }

    pub fn local_config() -> ChainSpec {
        // provide boot nodes
        let boot_nodes = vec![];
        ChainSpec::from_genesis(
            "Polymesh ITN Local",
            "local_itn",
            ChainType::Local,
            local_genesis,
            boot_nodes,
            None,
            None,
            Some(polymath_props(12)),
            Default::default(),
        )
    }
}
