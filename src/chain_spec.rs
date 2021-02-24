use codec::{Decode, Encode};
use grandpa::AuthorityId as GrandpaId;
use pallet_asset::{ClassicTickerImport, TickerRegistrationConfig};
use pallet_im_online::sr25519::AuthorityId as ImOnlineId;
use pallet_staking::StakerStatus;
use polymesh_common_utilities::{constants::currency::POLY, protocol_fee::ProtocolOp, GC_DID};
use polymesh_primitives::{
    AccountId, IdentityId, InvestorUid, Moment, PosRatio, Signatory, Signature, SmartExtensionType,
    Ticker,
};
use sc_chain_spec::ChainType;
use sc_service::Properties;
use sc_telemetry::TelemetryEndpoints;
use serde_json::json;
use sp_authority_discovery::AuthorityId as AuthorityDiscoveryId;
use sp_consensus_babe::AuthorityId as BabeId;
use sp_core::{sr25519, Pair, Public};
use sp_runtime::{
    traits::{IdentifyAccount, Verify},
    PerThing,
};
#[cfg(feature = "std")]
use sp_runtime::{Deserialize, Serialize};

use std::convert::TryInto;

const STAGING_TELEMETRY_URL: &str = "wss://telemetry.polymesh.live/submit/";

type AccountPublic = <Signature as Verify>::Signer;

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

fn seeded_acc_id(seed: &str) -> AccountId {
    get_account_id_from_seed::<sr25519::Public>(seed)
}

/// Helper function to generate stash, controller and session key from seed
pub fn get_authority_keys_from_seed(seed: &str, uniq: bool) -> InitialAuth {
    if uniq {
        (
            seeded_acc_id(&format!("{}//stash", seed)),
            seeded_acc_id(seed),
            get_from_seed::<GrandpaId>(&format!("{}//gran", seed)),
            get_from_seed::<BabeId>(&format!("{}//babe", seed)),
            get_from_seed::<ImOnlineId>(&format!("{}//imon", seed)),
            get_from_seed::<AuthorityDiscoveryId>(&format!("{}//auth", seed)),
        )
    } else {
        (
            seeded_acc_id(&format!("{}//stash", seed)),
            seeded_acc_id(seed),
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
            versions: vec![
                (SmartExtensionType::TransferManager, 5000),
                (SmartExtensionType::Offerings, 5000),
                (SmartExtensionType::SmartWallet, 5000),
            ],
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
    let tickers_data: Vec<ClassicTickerImport> =
        serde_json::from_str(&reserved_tickers_file).unwrap();
    tickers_data
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

// (primary_account_id, service provider did, target did, expiry time of CDD claim i.e 10 days is ms)
type Identity = (
    AccountId,
    IdentityId,
    IdentityId,
    InvestorUid,
    Option<Moment>,
);

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

fn cdd_provider(n: u8) -> Identity {
    (
        seeded_acc_id(adjust_last(&mut { *b"cdd_provider_0" }, n)),
        IdentityId::from(n as u128),
        IdentityId::from(n as u128),
        InvestorUid::from(adjust_last(&mut { *b"uid0" }, n).as_bytes()),
        None,
    )
}

fn gc_mem(n: u8) -> Identity {
    (
        seeded_acc_id(adjust_last(&mut { *b"governance_committee_0" }, n)),
        IdentityId::from(1 as u128),
        IdentityId::from(2 + n as u128),
        InvestorUid::from(adjust_last(&mut { *b"uid3" }, n)),
        None,
    )
}

fn polymath_mem(n: u8) -> Identity {
    (
        seeded_acc_id(adjust_last(&mut { *b"polymath_0" }, n)),
        IdentityId::from(1 as u128),
        IdentityId::from(2 + n as u128),
        InvestorUid::from(adjust_last(&mut { *b"uid3" }, n)),
        None,
    )
}

const STASH: u128 = 5_000_000 * POLY;
const ENDOWMENT: u128 = 100_000_000 * POLY;

fn identities(
    initial_authorities: &[InitialAuth],
    initial_identities: &[Identity],
) -> (
    Vec<(
        IdentityId,
        AccountId,
        AccountId,
        u128,
        StakerStatus<AccountId>,
    )>,
    Vec<Identity>,
    Vec<(AccountId, IdentityId)>,
) {
    let num_initial_identities = initial_identities.len() as u128;
    let mut identity_counter = num_initial_identities;
    let authority_identities = initial_authorities
        .iter()
        .map(|x| {
            identity_counter += 1;
            let did = IdentityId::from(identity_counter);
            let investor_uid = InvestorUid::from(did.as_ref());
            (x.1.clone(), IdentityId::from(1), did, investor_uid, None)
        })
        .collect::<Vec<_>>();

    let all_identities = initial_identities
        .iter()
        .cloned()
        .chain(authority_identities.iter().cloned())
        .collect::<Vec<_>>();
    identity_counter = num_initial_identities;

    let secondary_keys = initial_authorities
        .iter()
        .map(|x| {
            identity_counter += 1;
            (x.0.clone(), IdentityId::from(identity_counter))
        })
        .collect::<Vec<_>>();

    let stakers = authority_identities
        .iter()
        .cloned()
        .zip(initial_authorities.iter().cloned())
        .map(|((_, _, did, ..), x)| {
            (
                did,
                x.0.clone(),
                x.1.clone(),
                STASH,
                pallet_staking::StakerStatus::Validator,
            )
        })
        .collect::<Vec<_>>();

    (stakers, all_identities, secondary_keys)
}

fn balances(inits: &[InitialAuth], endoweds: &[AccountId]) -> Vec<(AccountId, u128)> {
    endoweds
        .iter()
        .map(|k: &AccountId| (k.clone(), ENDOWMENT))
        .chain(inits.iter().map(|x| (x.1.clone(), ENDOWMENT)))
        .chain(inits.iter().map(|x| (x.0.clone(), STASH)))
        .collect()
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
            validator_count: $auths.len() as u32,
            validator_commission_cap: $cap,
            stakers: $stakers,
            invulnerables: $auths.iter().map(|x| x.0.clone()).collect(),
            slash_reward_fraction: sp_runtime::Perbill::from_percent(10),
            min_bond_threshold: 5_000_000_000_000,
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
            max_pip_skip_count: 1,
            active_pip_limit: $limit,
            pending_pip_expiry: <_>::default(),
        }
    };
}

macro_rules! cdd_membership {
    ($($member:expr),*) => {
        pallet_group::GenesisConfig {
            active_members_limit: u32::MAX,
            active_members: vec![$(IdentityId::from($member)),*, GC_DID],
            phantom: Default::default(),
        }
    };
}

macro_rules! committee_membership {
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
        (ProtocolOp::AssetCreateAsset, 10_000 * 1_000_000),
        (ProtocolOp::AssetRegisterTicker, 2_500 * 1_000_000),
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

pub mod general {
    use super::*;
    use polymesh_runtime_develop::{self as rt, constants::time};

    pub type ChainSpec = sc_service::GenericChainSpec<rt::runtime::GenesisConfig>;

    session_keys!();

    fn genesis(
        initial_authorities: Vec<InitialAuth>,
        root_key: AccountId,
        endowed_accounts: Vec<AccountId>,
        enable_println: bool,
    ) -> rt::runtime::GenesisConfig {
        let init_ids = [
            // Service providers
            cdd_provider(1),
            cdd_provider(2),
            // Governance committee members
            gc_mem(1),
            gc_mem(2),
            gc_mem(3),
        ];
        let (stakers, all_identities, secondary_keys) = identities(&initial_authorities, &init_ids);

        rt::runtime::GenesisConfig {
            frame_system: Some(frame(rt::WASM_BINARY)),
            pallet_asset: Some(asset!()),
            pallet_checkpoint: Some(checkpoint!()),
            pallet_identity: Some(pallet_identity::GenesisConfig {
                identities: all_identities,
                secondary_keys,
                ..Default::default()
            }),
            pallet_balances: Some(pallet_balances::GenesisConfig {
                balances: balances(&initial_authorities, &endowed_accounts),
            }),
            pallet_bridge: Some(pallet_bridge::GenesisConfig {
                admin: initial_authorities[0].1.clone(),
                creator: initial_authorities[0].1.clone(),
                signatures_required: 1,
                signers: bridge_signers(),
                timelock: 10,
                bridge_limit: (100_000_000 * POLY, 1000),
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
            pallet_contracts: Some(pallet_contracts::GenesisConfig {
                current_schedule: pallet_contracts::Schedule {
                    enable_println, // this should only be enabled on development chains
                    ..Default::default()
                },
            }),
            // Governance Council:
            pallet_group_Instance1: Some(committee_membership!(3, 4, 5, 6)),
            pallet_committee_Instance1: Some(committee!(6)),
            pallet_group_Instance2: Some(cdd_membership!(1, 2, 6)), // sp1, sp2, first authority
            // Technical Committee:
            pallet_group_Instance3: Some(committee_membership!(3)),
            pallet_committee_Instance3: Some(committee!(3)),
            // Upgrade Committee:
            pallet_group_Instance4: Some(committee_membership!(4)),
            pallet_committee_Instance4: Some(committee!(4)),
            pallet_protocol_fee: Some(protocol_fee!()),
            pallet_settlement: Some(Default::default()),
            pallet_multisig: Some(pallet_multisig::GenesisConfig {
                transaction_version: 1,
            }),
            pallet_corporate_actions: Some(corporate_actions!()),
        }
    }

    fn develop_genesis() -> rt::runtime::GenesisConfig {
        genesis(
            vec![get_authority_keys_from_seed("Alice", false)],
            seeded_acc_id("Alice"),
            vec![
                seeded_acc_id("Bob"),
                seeded_acc_id("relay_1"),
                seeded_acc_id("relay_2"),
                seeded_acc_id("relay_3"),
                seeded_acc_id("relay_4"),
                seeded_acc_id("relay_5"),
            ],
            true,
        )
    }

    fn config(
        name: &str,
        id: &str,
        ctype: ChainType,
        genesis: impl 'static + Sync + Send + Fn() -> rt::runtime::GenesisConfig,
    ) -> ChainSpec {
        let props = Some(polymath_props());
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
            vec![
                seeded_acc_id("Charlie"),
                seeded_acc_id("Dave"),
                seeded_acc_id("Charlie//stash"),
                seeded_acc_id("relay_1"),
                seeded_acc_id("relay_2"),
                seeded_acc_id("relay_3"),
                seeded_acc_id("relay_4"),
                seeded_acc_id("relay_5"),
            ],
            true,
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

pub mod alcyone_testnet {
    use super::*;
    use polymesh_runtime_testnet::{self as rt, constants::time};

    pub type ChainSpec = sc_service::GenericChainSpec<rt::runtime::GenesisConfig>;

    session_keys!();

    fn genesis(
        initial_authorities: Vec<InitialAuth>,
        root_key: AccountId,
        endowed_accounts: Vec<AccountId>,
        enable_println: bool,
    ) -> rt::runtime::GenesisConfig {
        let init_ids = [
            // Service providers
            cdd_provider(1),
            cdd_provider(2),
            // Governance committee members
            polymath_mem(1),
            polymath_mem(2),
            polymath_mem(3),
        ];
        let (stakers, all_identities, secondary_keys) = identities(&initial_authorities, &init_ids);

        rt::runtime::GenesisConfig {
            frame_system: Some(frame(rt::WASM_BINARY)),
            pallet_asset: Some(asset!()),
            pallet_checkpoint: Some(checkpoint!()),
            pallet_identity: Some(pallet_identity::GenesisConfig {
                identities: all_identities,
                secondary_keys,
                ..Default::default()
            }),
            pallet_balances: Some(pallet_balances::GenesisConfig {
                balances: balances(&initial_authorities, &endowed_accounts),
            }),
            pallet_bridge: Some(pallet_bridge::GenesisConfig {
                admin: seeded_acc_id("polymath_1"),
                creator: seeded_acc_id("polymath_1"),
                signatures_required: 3,
                signers: bridge_signers(),
                timelock: time::MINUTES * 15,
                bridge_limit: (30_000_000_000, time::DAYS),
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
            pallet_contracts: Some(pallet_contracts::GenesisConfig {
                current_schedule: pallet_contracts::Schedule {
                    enable_println, // this should only be enabled on development chains
                    ..Default::default()
                },
            }),
            // Governing council
            pallet_group_Instance1: Some(committee_membership!(3, 4, 5)), //admin, gc1, gc2
            pallet_committee_Instance1: Some(committee!(3, (2, 3))),
            // CDD providers
            pallet_group_Instance2: Some(cdd_membership!(1, 2, 3)), // sp1, sp2, admin
            // Technical Committee:
            pallet_group_Instance3: Some(committee_membership!(3)), //admin
            pallet_committee_Instance3: Some(committee!(3)),
            // Upgrade Committee:
            pallet_group_Instance4: Some(committee_membership!(3)), //admin
            pallet_committee_Instance4: Some(committee!(3)),
            pallet_protocol_fee: Some(protocol_fee!()),
            pallet_settlement: Some(Default::default()),
            pallet_multisig: Some(pallet_multisig::GenesisConfig {
                transaction_version: 1,
            }),
            pallet_corporate_actions: Some(corporate_actions!()),
        }
    }

    fn develop_genesis() -> rt::runtime::GenesisConfig {
        genesis(
            vec![get_authority_keys_from_seed("Alice", false)],
            seeded_acc_id("Alice"),
            vec![
                seeded_acc_id("Bob"),
                seeded_acc_id("Bob//stash"),
                seeded_acc_id("relay_1"),
                seeded_acc_id("relay_2"),
                seeded_acc_id("relay_3"),
                seeded_acc_id("relay_4"),
                seeded_acc_id("relay_5"),
            ],
            true,
        )
    }

    pub fn develop_config() -> ChainSpec {
        // provide boot nodes
        let boot_nodes = vec![];
        ChainSpec::from_genesis(
            "Polymesh Alcyone Develop",
            "dev_alcyone",
            ChainType::Development,
            develop_genesis,
            boot_nodes,
            None,
            None,
            Some(polymath_props()),
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
            vec![
                seeded_acc_id("Charlie"),
                seeded_acc_id("Dave"),
                seeded_acc_id("Charlie//stash"),
                seeded_acc_id("relay_1"),
                seeded_acc_id("relay_2"),
                seeded_acc_id("relay_3"),
                seeded_acc_id("relay_4"),
                seeded_acc_id("relay_5"),
            ],
            true,
        )
    }

    pub fn local_config() -> ChainSpec {
        // provide boot nodes
        let boot_nodes = vec![];
        ChainSpec::from_genesis(
            "Polymesh Alcyone Local",
            "local_alcyone",
            ChainType::Local,
            local_genesis,
            boot_nodes,
            None,
            None,
            Some(polymath_props()),
            Default::default(),
        )
    }
}

pub mod polymesh_mainnet {
    use super::*;
    use polymesh_runtime_mainnet::{self as rt, constants::time};

    pub type ChainSpec = sc_service::GenericChainSpec<rt::runtime::GenesisConfig>;

    session_keys!();

    fn genesis(
        initial_authorities: Vec<InitialAuth>,
        root_key: AccountId,
        endowed_accounts: Vec<AccountId>,
        enable_println: bool,
    ) -> rt::runtime::GenesisConfig {
        let init_ids = [
            // Service providers
            cdd_provider(1),
            cdd_provider(2),
            // Governance committee members
            polymath_mem(1),
            polymath_mem(2),
            polymath_mem(3),
        ];
        let (stakers, all_identities, secondary_keys) = identities(&initial_authorities, &init_ids);

        rt::runtime::GenesisConfig {
            frame_system: Some(frame(rt::WASM_BINARY)),
            pallet_asset: Some(asset!()),
            pallet_checkpoint: Some(checkpoint!()),
            pallet_identity: Some(pallet_identity::GenesisConfig {
                identities: all_identities,
                secondary_keys,
                ..Default::default()
            }),
            pallet_balances: Some(pallet_balances::GenesisConfig {
                balances: balances(&initial_authorities, &endowed_accounts),
            }),
            pallet_bridge: Some(pallet_bridge::GenesisConfig {
                admin: seeded_acc_id("polymath_1"),
                creator: seeded_acc_id("polymath_1"),
                signatures_required: 3,
                signers: bridge_signers(),
                timelock: time::MINUTES * 15,
                bridge_limit: (30_000_000_000, time::DAYS),
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
            pallet_contracts: Some(pallet_contracts::GenesisConfig {
                current_schedule: pallet_contracts::Schedule {
                    enable_println, // this should only be enabled on development chains
                    ..Default::default()
                },
            }),
            // Governing council
            pallet_group_Instance1: Some(committee_membership!(3, 4, 5)), //admin, gc1, gc2
            pallet_committee_Instance1: Some(committee!(3, (2, 3))),
            // CDD providers
            pallet_group_Instance2: Some(cdd_membership!(1, 2, 3)), // sp1, sp2, admin
            // Technical Committee:
            pallet_group_Instance3: Some(committee_membership!(3)), //admin
            pallet_committee_Instance3: Some(committee!(3)),
            // Upgrade Committee:
            pallet_group_Instance4: Some(committee_membership!(3)), //admin
            pallet_committee_Instance4: Some(committee!(3)),
            pallet_protocol_fee: Some(protocol_fee!()),
            pallet_settlement: Some(Default::default()),
            pallet_multisig: Some(pallet_multisig::GenesisConfig {
                transaction_version: 1,
            }),
            pallet_corporate_actions: Some(corporate_actions!()),
        }
    }

    fn bootstrap_genesis() -> rt::runtime::GenesisConfig {
        genesis(
            vec![
                get_authority_keys_from_seed("Alice", false),
                get_authority_keys_from_seed("Bob", false),
                get_authority_keys_from_seed("Charlie", false),
            ],
            seeded_acc_id("Alice"),
            vec![
                seeded_acc_id("cdd_provider_1"),
                seeded_acc_id("cdd_provider_2"),
                seeded_acc_id("polymath_1"),
                seeded_acc_id("polymath_2"),
                seeded_acc_id("polymath_3"),
                seeded_acc_id("relay_1"),
                seeded_acc_id("relay_2"),
                seeded_acc_id("relay_3"),
                seeded_acc_id("relay_4"),
                seeded_acc_id("relay_5"),
            ],
            false,
        )
    }

    pub fn bootstrap_config() -> ChainSpec {
        // provide boot nodes
        let boot_nodes = vec![
            "/dns4/buffron-bootnode-1.polymesh.live/tcp/30333/p2p/12D3KooWAhsJHrHJ5Wk5v6sensyjJu2afJFanq4acxbMqhWje2pw".parse().expect("Unable to parse bootnode"),
            "/dns4/buffron-bootnode-2.polymesh.live/tcp/30333/p2p/12D3KooWQZ1mfWzKAzK5eXMqk4qupQqTshtWFSiSbhKS5D6Ycz1M".parse().expect("Unable to parse bootnode"),
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
            Some(&*"/polymath/mainnet/1"),
            Some(polymath_props()),
            Default::default(),
        )
    }

    fn develop_genesis() -> rt::runtime::GenesisConfig {
        genesis(
            vec![get_authority_keys_from_seed("Alice", false)],
            seeded_acc_id("Alice"),
            vec![
                seeded_acc_id("Bob"),
                seeded_acc_id("Bob//stash"),
                seeded_acc_id("relay_1"),
                seeded_acc_id("relay_2"),
                seeded_acc_id("relay_3"),
                seeded_acc_id("relay_4"),
                seeded_acc_id("relay_5"),
            ],
            true,
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
            Some(polymath_props()),
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
            vec![
                seeded_acc_id("Charlie"),
                seeded_acc_id("Dave"),
                seeded_acc_id("Charlie//stash"),
                seeded_acc_id("relay_1"),
                seeded_acc_id("relay_2"),
                seeded_acc_id("relay_3"),
                seeded_acc_id("relay_4"),
                seeded_acc_id("relay_5"),
            ],
            true,
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
            Some(polymath_props()),
            Default::default(),
        )
    }
}
