use codec::{Decode, Encode};
use grandpa::AuthorityId as GrandpaId;
use pallet_asset::{ClassicTickerImport, TickerRegistrationConfig};
use pallet_bridge::BridgeTx;
use pallet_im_online::sr25519::AuthorityId as ImOnlineId;
use pallet_staking::StakerStatus;
use polymesh_common_utilities::{
    constants::{currency::POLY, REWARDS_PALLET_ID, TREASURY_PALLET_ID},
    protocol_fee::ProtocolOp,
    MaybeBlock, SystematicIssuers,
};
use polymesh_primitives::{
    identity_id::GenesisIdentityRecord, AccountId, Balance, HexAccountId, IdentityId, Moment,
    PosRatio, SecondaryKey, Signatory, Signature, Ticker,
};
use sc_chain_spec::{ChainSpecExtension, ChainType};
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

// Genesis POLYX distribution via bridge
const TREASURY_LOCK_HASH: &str =
    "0x1000000000000000000000000000000000000000000000000000000000000001";
const REWARDS_LOCK_HASH: &str =
    "0x1000000000000000000000000000000000000000000000000000000000000002";
const KEY_LOCK_HASH: &str = "0x1000000000000000000000000000000000000000000000000000000000000003";

const BOOTSTRAP_KEYS: u128 = 6_000 * POLY;
const BOOTSTRAP_TREASURY: u128 = 17_500_000 * POLY;

const DEV_KEYS: u128 = 30_000_000 * POLY;
const DEV_TREASURY: u128 = 50_000_000 * POLY;

const INITIAL_BOND: u128 = 500 * POLY;

// 1 in 4 blocks (on average, not counting collisions) will be primary babe blocks.
const PRIMARY_PROBABILITY: (u64, u64) = (1, 4);
const BABE_GENESIS_EPOCH_CONFIG: sp_consensus_babe::BabeEpochConfiguration =
    sp_consensus_babe::BabeEpochConfiguration {
        c: PRIMARY_PROBABILITY,
        allowed_slots: sp_consensus_babe::AllowedSlots::PrimaryAndSecondaryPlainSlots,
    };
const BABE_GENESIS: pallet_babe::GenesisConfig = pallet_babe::GenesisConfig {
    authorities: vec![],
    epoch_config: Some(BABE_GENESIS_EPOCH_CONFIG),
};

/// Node `ChainSpec` extensions.
///
/// Additional parameters for some Substrate core modules,
/// customizable from the chain spec.
#[derive(Default, Clone, Serialize, Deserialize, ChainSpecExtension)]
#[serde(rename_all = "camelCase")]
pub struct Extensions {
    /// The light sync state.
    ///
    /// This value will be set by the `sync-state rpc` implementation.
    pub light_sync_state: sc_sync_state_rpc::LightSyncStateExtension,
}

pub type GenericChainSpec<R> = sc_service::GenericChainSpec<R, Extensions>;

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
                // Reservations will expire at end of March 2022
                registration_length: Some(1648771199999),
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

#[derive(Clone)]
struct BridgeLockId {
    nonce: u32,
    amount: u128,
    tx_hash: H256,
}

impl BridgeLockId {
    fn new(nonce: u32, amount: u128, hash: &'static str) -> Self {
        let offset = if hash.starts_with("0x") { 2 } else { 0 };
        let stripped_hash = &hash[offset..];
        let hash_vec: Vec<u8> = rustc_hex::FromHex::from_hex(stripped_hash)
            .expect("Failed to decode transaction hash (Invalid hex Value)");
        let hash_array: [u8; 32] = hash_vec
            .try_into()
            .expect("Failed to decode transaction hash (Invalid hash length)");
        Self {
            nonce,
            amount,
            tx_hash: hash_array.into(),
        }
    }

    fn generate_bridge_locks(
        starting_nonce: u32,
        count: u32,
        amount: u128,
        hash: &'static str,
    ) -> Vec<Self> {
        (0..count)
            .map(|x| Self::new(starting_nonce + x, amount, hash))
            .collect()
    }
}

fn genesis_processed_data(
    initial_authorities: &Vec<InitialAuth>, //Alice, Bob, Charlie
    root_key: AccountId,                    //polymath_5
    treasury_bridge_lock: BridgeLockId,
    rewards_bridge_lock: BridgeLockId,
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
    // 1 = [Polymath] GenesisCouncil (1 of 3) + UpgradeCommittee (1 of 1) + TechnicalCommittee (1 of 1) + GCReleaseCoordinator
    // 2 = GenesisCouncil (2 of 3)
    // 3 = GenesisCouncil (3 of 3)
    // 4 = Operator
    // 5 = Bridge + Sudo

    // Identity_01
    // Primary Key: polymath_1

    // Identity_02
    // Primary Key: polymath_2

    // Identity_03
    // Primary Key: polymath_3

    // Identity_04
    // Primary Key: polymath_4
    // Secondary Keys: Alice, Alice//stash, Bob, Bob//stash, Charlie, Charlie//stash

    // Identity_05
    // Primary Key: polymath_5
    // Secondary Keys: bridge multisig (controller)

    let mut identities = Vec::with_capacity(5);
    let mut keys = Vec::with_capacity(5 + 2 * initial_authorities.len()); //11

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
            INITIAL_BOND,
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

    // Give CDD issuer to operator and bridge admin / sudo since it won't receive CDD from the group automatically
    identities[3]
        .issuers
        .push(SystematicIssuers::CDDProvider.as_id());

    // Give CDD issuer to operator and bridge admin / sudo since it won't receive CDD from the group automatically
    identities[4]
        .issuers
        .push(SystematicIssuers::CDDProvider.as_id());

    // Accumulate bridge transactions
    let mut complete_txs: Vec<_> = key_bridge_locks
        .iter()
        .cloned()
        .zip(keys.iter().cloned())
        .map(
            |(
                BridgeLockId {
                    nonce,
                    amount,
                    tx_hash,
                },
                recipient,
            )| BridgeTx {
                nonce,
                recipient,
                amount,
                tx_hash,
            },
        )
        .collect();

    complete_txs.push(BridgeTx {
        nonce: treasury_bridge_lock.nonce,
        recipient: TREASURY_PALLET_ID.into_account(),
        amount: treasury_bridge_lock.amount,
        tx_hash: treasury_bridge_lock.tx_hash,
    });

    complete_txs.push(BridgeTx {
        nonce: rewards_bridge_lock.nonce,
        recipient: REWARDS_PALLET_ID.into_account(),
        amount: rewards_bridge_lock.amount,
        tx_hash: rewards_bridge_lock.tx_hash,
    });

    (identities, stakers, complete_txs)
}

fn dev_genesis_processed_data(
    initial_authorities: &Vec<InitialAuth>,
    treasury_bridge_lock: BridgeLockId,
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
            INITIAL_BOND,
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
        .zip(identity.secondary_keys.iter().map(|sk| sk.key.clone()))
        .map(
            |(
                BridgeLockId {
                    nonce,
                    amount,
                    tx_hash,
                },
                recipient,
            )| BridgeTx {
                nonce,
                recipient,
                amount,
                tx_hash,
            },
        )
        .collect();

    complete_txs.push(BridgeTx {
        nonce: treasury_bridge_lock.nonce,
        recipient: TREASURY_PALLET_ID.into_account(),
        amount: BOOTSTRAP_TREASURY,
        tx_hash: treasury_bridge_lock.tx_hash,
    });

    complete_txs.push(BridgeTx {
        nonce: rewards_bridge_lock.nonce,
        recipient: REWARDS_PALLET_ID.into_account(),
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
            validator_count: 40,
            validator_commission_cap: $cap,
            stakers: $stakers,
            invulnerables: vec![],
            slash_reward_fraction: sp_runtime::Perbill::from_percent(10),
            min_bond_threshold: 0, // Will be updated to 50,000 POLYX once network has stabalised, before external operators join
            ..Default::default()
        }
    };
}

macro_rules! pips {
    ($period:expr, $expiry:expr, $limit:expr) => {
        pallet_pips::GenesisConfig {
            prune_historical_pips: false,
            min_proposal_deposit: 2_000_000_000,
            default_enactment_period: $period,
            max_pip_skip_count: 2,
            active_pip_limit: $limit,
            pending_pip_expiry: $expiry,
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

    pub type ChainSpec = GenericChainSpec<rt::runtime::GenesisConfig>;

    session_keys!();

    fn genesis(
        initial_authorities: Vec<InitialAuth>,
        root_key: AccountId,
        _enable_println: bool,
        treasury_bridge_lock: BridgeLockId,
        rewards_bridge_lock: BridgeLockId,
        key_bridge_locks: Vec<BridgeLockId>,
        other_funded_accounts: Vec<AccountId>,
    ) -> rt::runtime::GenesisConfig {
        let (identity, stakers, complete_txs) = dev_genesis_processed_data(
            &initial_authorities,
            treasury_bridge_lock,
            rewards_bridge_lock,
            key_bridge_locks,
            other_funded_accounts,
        );

        rt::runtime::GenesisConfig {
            system: frame(rt::WASM_BINARY),
            asset: asset!(),
            checkpoint: checkpoint!(),
            identity: pallet_identity::GenesisConfig {
                identities: vec![identity],
                ..Default::default()
            },
            balances: Default::default(),
            bridge: pallet_bridge::GenesisConfig {
                admin: initial_authorities[0].1.clone(),
                creator: initial_authorities[0].1.clone(),
                signatures_required: 1,
                signers: bridge_signers(),
                timelock: 10,
                bridge_limit: (100_000_000 * POLY, 1000),
                complete_txs,
            },
            indices: pallet_indices::GenesisConfig { indices: vec![] },
            sudo: pallet_sudo::GenesisConfig { key: root_key },
            session: session!(initial_authorities, session_keys),
            staking: staking!(
                initial_authorities,
                stakers,
                PerThing::from_rational(1u64, 4u64)
            ),
            pips: pips!(time::MINUTES, MaybeBlock::None, 25),
            im_online: Default::default(),
            authority_discovery: Default::default(),
            babe: BABE_GENESIS,
            grandpa: Default::default(),
            /*
            pallet_contracts: Some(pallet_contracts::GenesisConfig {
                current_schedule: pallet_contracts::Schedule {
                    enable_println, // this should only be enabled on development chains
                    ..Default::default()
                },
            }),
            */
            // Governance Council:
            committee_membership: group_membership!(1),
            polymesh_committee: committee!(1),
            // CDD providers
            cdd_service_providers: group_membership!(1),
            // Technical Committee:
            technical_committee_membership: group_membership!(1),
            technical_committee: committee!(1),
            // Upgrade Committee:
            upgrade_committee_membership: group_membership!(1),
            upgrade_committee: committee!(1),
            protocol_fee: protocol_fee!(),
            settlement: Default::default(),
            multi_sig: pallet_multisig::GenesisConfig {
                transaction_version: 1,
            },
            corporate_action: corporate_actions!(),
            rewards: rewards!(),
        }
    }

    fn develop_genesis() -> rt::runtime::GenesisConfig {
        genesis(
            vec![get_authority_keys_from_seed("Alice", false)],
            seeded_acc_id("Alice"),
            true,
            BridgeLockId::new(1, DEV_TREASURY, TREASURY_LOCK_HASH),
            BridgeLockId::new(
                2,
                itn_rewards().into_iter().map(|(_, b)| b + (1 * POLY)).sum(),
                REWARDS_LOCK_HASH,
            ),
            BridgeLockId::generate_bridge_locks(3, 20, DEV_KEYS, KEY_LOCK_HASH),
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
        ChainSpec::from_genesis(
            name,
            id,
            ctype,
            genesis,
            vec![],
            None,
            None,
            None,
            props,
            <_>::default(),
        )
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
                get_authority_keys_from_seed("Charlie", false),
            ],
            seeded_acc_id("Alice"),
            true,
            BridgeLockId::new(1, DEV_TREASURY, TREASURY_LOCK_HASH),
            BridgeLockId::new(
                2,
                itn_rewards().into_iter().map(|(_, b)| b + (1 * POLY)).sum(),
                REWARDS_LOCK_HASH,
            ),
            BridgeLockId::generate_bridge_locks(3, 20, DEV_KEYS, KEY_LOCK_HASH),
            vec![seeded_acc_id("Dave"), seeded_acc_id("Eve")],
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

    pub type ChainSpec = GenericChainSpec<rt::runtime::GenesisConfig>;

    session_keys!();

    fn genesis(
        initial_authorities: Vec<InitialAuth>,
        root_key: AccountId,
        _enable_println: bool,
        treasury_bridge_lock: BridgeLockId,
        rewards_bridge_lock: BridgeLockId,
        key_bridge_locks: Vec<BridgeLockId>,
    ) -> rt::runtime::GenesisConfig {
        let (identities, stakers, complete_txs) = genesis_processed_data(
            &initial_authorities,
            root_key.clone(),
            treasury_bridge_lock,
            rewards_bridge_lock,
            key_bridge_locks,
        );

        rt::runtime::GenesisConfig {
            system: frame(rt::WASM_BINARY),
            asset: asset!(),
            checkpoint: checkpoint!(),
            identity: pallet_identity::GenesisConfig {
                identities,
                ..Default::default()
            },
            balances: Default::default(),
            bridge: pallet_bridge::GenesisConfig {
                admin: root_key.clone(),
                creator: root_key.clone(),
                signatures_required: 3,
                signers: bridge_signers(),
                timelock: time::MINUTES * 15,
                bridge_limit: (30_000 * POLY, 1 * time::DAYS),
                complete_txs,
            },
            indices: pallet_indices::GenesisConfig { indices: vec![] },
            sudo: pallet_sudo::GenesisConfig { key: root_key },
            session: session!(initial_authorities, session_keys),
            staking: staking!(
                initial_authorities,
                stakers,
                PerThing::from_rational(1u64, 10u64)
            ),
            pips: pips!(time::DAYS * 30, MaybeBlock::None, 1000),
            im_online: Default::default(),
            authority_discovery: Default::default(),
            babe: BABE_GENESIS,
            grandpa: Default::default(),
            /*
            pallet_contracts: Some(pallet_contracts::GenesisConfig {
                current_schedule: pallet_contracts::Schedule {
                    enable_println, // this should only be enabled on development chains
                    ..Default::default()
                },
            }),
            */
            // Governing council
            committee_membership: group_membership!(1, 2, 3), // 3 GC members
            polymesh_committee: committee!(1, (2, 3)),        // RC = 1, 2/3 votes required
            // CDD providers
            cdd_service_providers: group_membership!(1),
            // Technical Committee:
            technical_committee_membership: group_membership!(3, 4, 5), // One GC member + genesis operator + Bridge Multisig
            technical_committee: committee!(3), // RC = 3, 1/2 votes required
            // Upgrade Committee:
            upgrade_committee_membership: group_membership!(1), // One GC member
            upgrade_committee: committee!(1),                   // RC = 1, 1/2 votes required
            protocol_fee: protocol_fee!(),
            settlement: Default::default(),
            multi_sig: pallet_multisig::GenesisConfig {
                transaction_version: 1,
            },
            corporate_action: corporate_actions!(),
            rewards: rewards!(),
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
            BridgeLockId::new(1, BOOTSTRAP_TREASURY, TREASURY_LOCK_HASH),
            BridgeLockId::new(
                2,
                itn_rewards().into_iter().map(|(_, b)| b + (1 * POLY)).sum(),
                REWARDS_LOCK_HASH,
            ),
            BridgeLockId::generate_bridge_locks(3, 20, BOOTSTRAP_KEYS, KEY_LOCK_HASH),
        )
    }

    pub fn bootstrap_config() -> ChainSpec {
        // provide boot nodes
        let boot_nodes = vec![
            "/dns4/testnet-bootnode-001.polymesh.live/tcp/30333/p2p/12D3KooWNG4hedmYixq3Vx4crj5VFxHLFWjqYfbAZwFekHJ8Y7du".parse().expect("Unable to parse bootnode"),
            "/dns4/testnet-bootnode-002.polymesh.live/tcp/30333/p2p/12D3KooW9uY8zFnHB5UKyLuwUpZLpPUSJYT2tYfFvpfNCd2K1ceZ".parse().expect("Unable to parse bootnode"),
            "/dns4/testnet-bootnode-003.polymesh.live/tcp/30333/p2p/12D3KooWB7AyqsmerKTmcMoyMJJw6ddwWUJ7nFBDGw2viNGN2DBX".parse().expect("Unable to parse bootnode"),
        ];
        ChainSpec::from_genesis(
            "Polymesh Testnet",
            "testnet",
            ChainType::Live,
            bootstrap_genesis,
            boot_nodes,
            Some(
                TelemetryEndpoints::new(vec![(STAGING_TELEMETRY_URL.to_string(), 0)])
                    .expect("Testnet bootstrap telemetry url is valid; qed"),
            ),
            Some(&*"/polymesh/testnet"),
            None,
            Some(polymath_props(42)),
            Default::default(),
        )
    }

    fn develop_genesis() -> rt::runtime::GenesisConfig {
        genesis(
            vec![get_authority_keys_from_seed("Alice", false)],
            seeded_acc_id("Eve"),
            true,
            BridgeLockId::new(1, BOOTSTRAP_TREASURY, TREASURY_LOCK_HASH),
            BridgeLockId::new(
                2,
                itn_rewards().into_iter().map(|(_, b)| b + (1 * POLY)).sum(),
                REWARDS_LOCK_HASH,
            ),
            BridgeLockId::generate_bridge_locks(3, 20, BOOTSTRAP_KEYS, KEY_LOCK_HASH),
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
                get_authority_keys_from_seed("Charlie", false),
            ],
            seeded_acc_id("Eve"),
            true,
            BridgeLockId::new(1, BOOTSTRAP_TREASURY, TREASURY_LOCK_HASH),
            BridgeLockId::new(
                2,
                itn_rewards().into_iter().map(|(_, b)| b + (1 * POLY)).sum(),
                REWARDS_LOCK_HASH,
            ),
            BridgeLockId::generate_bridge_locks(3, 20, BOOTSTRAP_KEYS, KEY_LOCK_HASH),
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
            None,
            Some(polymath_props(42)),
            Default::default(),
        )
    }
}

pub mod mainnet {
    use super::*;
    use polymesh_runtime_mainnet::{self as rt, constants::time};

    pub type ChainSpec = GenericChainSpec<rt::runtime::GenesisConfig>;

    session_keys!();

    fn genesis(
        initial_authorities: Vec<InitialAuth>,
        root_key: AccountId,
        _enable_println: bool,
        treasury_bridge_lock: BridgeLockId,
        reward_bridge_lock: BridgeLockId,
        key_bridge_locks: Vec<BridgeLockId>,
    ) -> rt::runtime::GenesisConfig {
        let (identities, stakers, complete_txs) = genesis_processed_data(
            &initial_authorities,
            root_key.clone(),
            treasury_bridge_lock,
            reward_bridge_lock,
            key_bridge_locks,
        );

        rt::runtime::GenesisConfig {
            system: frame(rt::WASM_BINARY),
            asset: asset!(),
            checkpoint: checkpoint!(),
            identity: pallet_identity::GenesisConfig {
                identities,
                ..Default::default()
            },
            balances: Default::default(),
            bridge: pallet_bridge::GenesisConfig {
                admin: root_key.clone(),
                creator: root_key.clone(),
                signatures_required: 4,
                signers: bridge_signers(),
                timelock: time::HOURS * 24,
                bridge_limit: (1_000_000_000 * POLY, 365 * time::DAYS),
                complete_txs,
            },
            indices: pallet_indices::GenesisConfig { indices: vec![] },
            sudo: pallet_sudo::GenesisConfig { key: root_key },
            session: session!(initial_authorities, session_keys),
            staking: staking!(
                initial_authorities,
                stakers,
                PerThing::from_rational(1u64, 10u64)
            ),
            pips: pips!(time::DAYS * 30, MaybeBlock::Some(time::DAYS * 90), 1000),
            im_online: Default::default(),
            authority_discovery: Default::default(),
            babe: BABE_GENESIS,
            grandpa: Default::default(),
            /*
            pallet_contracts: Some(pallet_contracts::GenesisConfig {
                current_schedule: pallet_contracts::Schedule {
                    enable_println, // this should only be enabled on development chains
                    ..Default::default()
                },
            }),
            */
            // Governing council
            committee_membership: group_membership!(1, 2, 3), // 3 GC members
            polymesh_committee: committee!(1, (2, 3)),        // RC = 1, 2/3 votes required
            // CDD providers
            cdd_service_providers: group_membership!(1), // GC_1 is also a CDD provider
            // Technical Committee:
            technical_committee_membership: group_membership!(1), // One GC member
            technical_committee: committee!(1),                   // 1/2 votes required
            // Upgrade Committee:
            upgrade_committee_membership: group_membership!(1), // One GC member
            upgrade_committee: committee!(1),                   // 1/2 votes required
            protocol_fee: protocol_fee!(),
            settlement: Default::default(),
            multi_sig: pallet_multisig::GenesisConfig {
                transaction_version: 1,
            },
            corporate_action: corporate_actions!(),
            rewards: rewards!(),
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
            BridgeLockId::new(1, BOOTSTRAP_TREASURY, TREASURY_LOCK_HASH),
            BridgeLockId::new(
                2,
                itn_rewards().into_iter().map(|(_, b)| b + (1 * POLY)).sum(),
                REWARDS_LOCK_HASH,
            ),
            BridgeLockId::generate_bridge_locks(3, 20, BOOTSTRAP_KEYS, KEY_LOCK_HASH),
        )
    }

    pub fn bootstrap_config() -> ChainSpec {
        // provide boot nodes
        let boot_nodes = vec![
            "/dns4/mainnet-bootnode-001.polymesh.network/tcp/30333/p2p/12D3KooWDiaRBvzjt1p95mTqJETxJw3nz1E6fF2Yf62ojimEGJS7".parse().expect("Unable to parse bootnode"),
            "/dns4/mainnet-bootnode-002.polymesh.network/tcp/30333/p2p/12D3KooWN9E6gtgybnXwDVNMUGwSA82pzBj72ibGYfZuomyEDQTU".parse().expect("Unable to parse bootnode"),
            "/dns4/mainnet-bootnode-003.polymesh.network/tcp/30333/p2p/12D3KooWQ3K8jGadCQSVhihLEsJfSz3TJGgBHMU3vTtK3jd2Wq5E".parse().expect("Unable to parse bootnode"),
            "/dns4/mainnet-bootnode-004.polymesh.network/tcp/30333/p2p/12D3KooWAjLb7S2FKk1Bxyw3vkaqgcSpjfxHwpGvqcXACFYSK8Xq".parse().expect("Unable to parse bootnode"),
            "/dns4/mainnet-bootnode-005.polymesh.network/tcp/30333/p2p/12D3KooWKvXCP5b5PW4tHFAYyFVk3kRhwF3qXJbnVcPSGHP6Zmjg".parse().expect("Unable to parse bootnode"),
            "/dns4/mainnet-bootnode-006.polymesh.network/tcp/30333/p2p/12D3KooWBQhDAjfo13dM4nsogXD39F5TcN9iTVzjXgPqFn9Yaccz".parse().expect("Unable to parse bootnode"),
            "/dns4/mainnet-bootnode-007.polymesh.network/tcp/30333/p2p/12D3KooWMwFdYC53MqdyR9WYvJiPfxfYXh65NfY9QSuZeyKa53fg".parse().expect("Unable to parse bootnode"),
        ];
        ChainSpec::from_genesis(
            "Polymesh Mainnet",
            "mainnet",
            ChainType::Live,
            bootstrap_genesis,
            boot_nodes,
            Some(
                TelemetryEndpoints::new(vec![(STAGING_TELEMETRY_URL.to_string(), 0)])
                    .expect("Mainnet bootstrap telemetry url is valid; qed"),
            ),
            Some(&*"/polymesh/mainnet"),
            None,
            Some(polymath_props(12)),
            Default::default(),
        )
    }

    fn develop_genesis() -> rt::runtime::GenesisConfig {
        genesis(
            vec![get_authority_keys_from_seed("Alice", false)],
            seeded_acc_id("Eve"),
            true,
            BridgeLockId::new(1, BOOTSTRAP_TREASURY, TREASURY_LOCK_HASH),
            BridgeLockId::new(
                2,
                itn_rewards().into_iter().map(|(_, b)| b + (1 * POLY)).sum(),
                REWARDS_LOCK_HASH,
            ),
            BridgeLockId::generate_bridge_locks(3, 20, BOOTSTRAP_KEYS, KEY_LOCK_HASH),
        )
    }

    pub fn develop_config() -> ChainSpec {
        // provide boot nodes
        let boot_nodes = vec![];
        ChainSpec::from_genesis(
            "Polymesh Mainnet Develop",
            "dev_mainnet",
            ChainType::Development,
            develop_genesis,
            boot_nodes,
            None,
            None,
            None,
            Some(polymath_props(12)),
            Default::default(),
        )
    }

    fn local_genesis() -> rt::runtime::GenesisConfig {
        genesis(
            vec![
                get_authority_keys_from_seed("Alice", false),
                get_authority_keys_from_seed("Bob", false),
                get_authority_keys_from_seed("Charlie", false),
            ],
            seeded_acc_id("Eve"),
            true,
            BridgeLockId::new(1, BOOTSTRAP_TREASURY, TREASURY_LOCK_HASH),
            BridgeLockId::new(
                2,
                itn_rewards().into_iter().map(|(_, b)| b + (1 * POLY)).sum(),
                REWARDS_LOCK_HASH,
            ),
            BridgeLockId::generate_bridge_locks(3, 20, BOOTSTRAP_KEYS, KEY_LOCK_HASH),
        )
    }

    pub fn local_config() -> ChainSpec {
        // provide boot nodes
        let boot_nodes = vec![];
        ChainSpec::from_genesis(
            "Polymesh Mainnet Local",
            "local_mainnet",
            ChainType::Local,
            local_genesis,
            boot_nodes,
            None,
            None,
            None,
            Some(polymath_props(12)),
            Default::default(),
        )
    }
}

pub mod ci {
    use super::*;
    use polymesh_runtime_ci::{self as rt, constants::time};

    pub type ChainSpec = GenericChainSpec<rt::runtime::GenesisConfig>;

    session_keys!();

    fn genesis(
        initial_authorities: Vec<InitialAuth>,
        root_key: AccountId,
        _enable_println: bool,
        treasury_bridge_lock: BridgeLockId,
        reward_bridge_lock: BridgeLockId,
        key_bridge_locks: Vec<BridgeLockId>,
    ) -> rt::runtime::GenesisConfig {
        let (identities, stakers, complete_txs) = genesis_processed_data(
            &initial_authorities,
            root_key.clone(),
            treasury_bridge_lock,
            reward_bridge_lock,
            key_bridge_locks,
        );

        rt::runtime::GenesisConfig {
            system: frame(rt::WASM_BINARY),
            asset: asset!(),
            checkpoint: checkpoint!(),
            identity: pallet_identity::GenesisConfig {
                identities,
                ..Default::default()
            },
            balances: Default::default(),
            bridge: pallet_bridge::GenesisConfig {
                admin: seeded_acc_id("polymath_1"),
                creator: seeded_acc_id("polymath_1"),
                signatures_required: 3,
                signers: bridge_signers(),
                timelock: time::MINUTES * 15,
                bridge_limit: (30_000_000_000, time::DAYS),
                complete_txs,
            },
            indices: pallet_indices::GenesisConfig { indices: vec![] },
            sudo: pallet_sudo::GenesisConfig { key: root_key },
            session: session!(initial_authorities, session_keys),
            staking: staking!(initial_authorities, stakers, PerThing::zero()),
            pips: pips!(time::DAYS * 7, MaybeBlock::None, 1000),
            im_online: Default::default(),
            authority_discovery: Default::default(),
            babe: BABE_GENESIS,
            grandpa: Default::default(),
            /*
            pallet_contracts: Some(pallet_contracts::GenesisConfig {
                current_schedule: pallet_contracts::Schedule {
                    enable_println, // this should only be enabled on development chains
                    ..Default::default()
                },
            }),
            */
            // Governing council
            committee_membership: group_membership!(1, 2, 3, 5),
            polymesh_committee: committee!(1, (2, 4)),
            // CDD providers
            cdd_service_providers: group_membership!(1, 2, 3, 5),
            // Technical Committee:
            technical_committee_membership: group_membership!(3, 5),
            technical_committee: committee!(5),
            // Upgrade Committee:
            upgrade_committee_membership: group_membership!(1, 5),
            upgrade_committee: committee!(5),
            protocol_fee: protocol_fee!(),
            settlement: Default::default(),
            multi_sig: pallet_multisig::GenesisConfig {
                transaction_version: 1,
            },
            corporate_action: corporate_actions!(),
            rewards: rewards!(),
        }
    }

    fn develop_genesis() -> rt::runtime::GenesisConfig {
        genesis(
            vec![get_authority_keys_from_seed("Bob", false)],
            seeded_acc_id("Alice"),
            true,
            BridgeLockId::new(1, DEV_TREASURY, TREASURY_LOCK_HASH),
            BridgeLockId::new(
                2,
                itn_rewards().into_iter().map(|(_, b)| b + (1 * POLY)).sum(),
                REWARDS_LOCK_HASH,
            ),
            BridgeLockId::generate_bridge_locks(3, 20, DEV_KEYS, KEY_LOCK_HASH),
        )
    }

    pub fn develop_config() -> ChainSpec {
        // provide boot nodes
        let boot_nodes = vec![];
        ChainSpec::from_genesis(
            "Polymesh CI Develop",
            "dev_ci",
            ChainType::Development,
            develop_genesis,
            boot_nodes,
            None,
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
                get_authority_keys_from_seed("Charlie", false),
            ],
            seeded_acc_id("Alice"),
            true,
            BridgeLockId::new(1, DEV_TREASURY, TREASURY_LOCK_HASH),
            BridgeLockId::new(
                2,
                itn_rewards().into_iter().map(|(_, b)| b + (1 * POLY)).sum(),
                REWARDS_LOCK_HASH,
            ),
            BridgeLockId::generate_bridge_locks(3, 20, DEV_KEYS, KEY_LOCK_HASH),
        )
    }

    pub fn local_config() -> ChainSpec {
        // provide boot nodes
        let boot_nodes = vec![];
        ChainSpec::from_genesis(
            "Polymesh CI Local",
            "local_ci",
            ChainType::Local,
            local_genesis,
            boot_nodes,
            None,
            None,
            None,
            Some(polymath_props(42)),
            Default::default(),
        )
    }
}
