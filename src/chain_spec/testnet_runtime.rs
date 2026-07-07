use sc_chain_spec::ChainType;
use sc_network::config::MultiaddrWithPeerId;
use sc_telemetry::TelemetryEndpoints;
use sp_runtime::PerThing;

use polymesh_primitives::{AccountId, IdentityId, MaybeBlock};
use polymesh_runtime_testnet::constants::time::DAYS;
use polymesh_runtime_testnet::runtime::{SessionKeys, BABE_GENESIS_EPOCH_CONFIG};

use crate::chain_spec::common::pips_genesis_config;
use crate::chain_spec::common::{asset_genesis_config, group_genesis_config};
use crate::chain_spec::common::{checkpoint_genesis_config, committee_genesis_config};
use crate::chain_spec::common::{corporate_actions_genesis_config, staking_genesis_config};
use crate::chain_spec::common::{get_authority_keys_from_seed, polymesh_properties, seeded_acc_id};
use crate::chain_spec::common::{protocol_fee_genesis_config, validators_genesis_config};
use crate::chain_spec::common::{ChainSpec, ChainSpecMode, InitialAuth, STAGING_TELEMETRY_URL};

pub fn testnet_chain_spec(chain_spec_mode: ChainSpecMode) -> ChainSpec {
    let code = polymesh_runtime_testnet::runtime::WASM_BINARY
        .expect("Mainnet wasm binary is not available.");

    match chain_spec_mode {
        ChainSpecMode::Bootstrap => bootstap_chain_spec(code),
        ChainSpecMode::Development => dev_chain_spec(code),
        ChainSpecMode::Local => local_chain_spec(code),
    }
}

fn bootstap_chain_spec(code: &[u8]) -> ChainSpec {
    let root_key = seeded_acc_id("polymesh_5");

    let initial_authorities = vec![
        get_authority_keys_from_seed("Alice", false),
        get_authority_keys_from_seed("Bob", false),
        get_authority_keys_from_seed("Charlie", false),
    ];

    let testnet_telemetry = TelemetryEndpoints::new(vec![(STAGING_TELEMETRY_URL.to_string(), 0)])
        .expect("Testnet bootstrap telemetry url is valid; qed");

    ChainSpec::builder(code, Default::default())
        .with_name("Polymesh Testnet")
        .with_id("testnet")
        .with_chain_type(ChainType::Live)
        .with_boot_nodes(testnet_boot_nodes())
        .with_telemetry_endpoints(testnet_telemetry)
        .with_protocol_id("/polymesh/testnet")
        .with_properties(polymesh_properties(42))
        .with_genesis_config_patch(testnet_genesis_config(initial_authorities, root_key))
        .build()
}

fn dev_chain_spec(code: &[u8]) -> ChainSpec {
    let root_key = seeded_acc_id("Eve");

    let initial_authorities = vec![get_authority_keys_from_seed("Alice", false)];

    ChainSpec::builder(code, Default::default())
        .with_name("Polymesh Testnet Develop")
        .with_id("dev_testnet")
        .with_chain_type(ChainType::Development)
        .with_properties(polymesh_properties(42))
        .with_genesis_config_patch(testnet_genesis_config(initial_authorities, root_key))
        .build()
}

fn local_chain_spec(code: &[u8]) -> ChainSpec {
    let root_key = seeded_acc_id("Eve");

    let initial_authorities = vec![
        get_authority_keys_from_seed("Alice", false),
        get_authority_keys_from_seed("Bob", false),
        get_authority_keys_from_seed("Charlie", false),
    ];

    ChainSpec::builder(code, Default::default())
        .with_name("Polymesh Testnet Local")
        .with_id("local_testnet")
        .with_chain_type(ChainType::Local)
        .with_properties(polymesh_properties(42))
        .with_genesis_config_patch(testnet_genesis_config(initial_authorities, root_key))
        .build()
}

fn testnet_genesis_config(
    initial_authorities: Vec<InitialAuth>,
    root_key: AccountId,
) -> serde_json::Value {
    let genesis_data =
        crate::chain_spec::mainnet_runtime::genesis_data(&initial_authorities, root_key.clone());

    let session_keys = session_keys(&initial_authorities);

    let (identity_1, identity_2, identity_3) = (
        IdentityId::from(1),
        IdentityId::from(2),
        IdentityId::from(3),
    );

    let (identity_4, identity_5) = (IdentityId::from(4), IdentityId::from(5));

    serde_json::json!({
        "asset": asset_genesis_config(),
        "checkpoint": checkpoint_genesis_config(),
        "identity": {
            "identities": genesis_data.identities_record,
        },
        "balances": {
            "balances": genesis_data.identities_balance,
        },
        "session": {
            "keys": session_keys,
        },
        "validators": validators_genesis_config(&genesis_data.stakers_data, PerThing::from_rational(1u64, 4u64)),
        "staking": staking_genesis_config(&genesis_data.stakers_data),
        "pips": pips_genesis_config(DAYS * 30, MaybeBlock::None, 1_000),
        "babe": {
            "epochConfig": Some(BABE_GENESIS_EPOCH_CONFIG),
        },
        // Governing council
        "committeeMembership": group_genesis_config(vec![identity_1, identity_2, identity_3]),  // three GC members
        "polymeshCommittee": committee_genesis_config((2, 3), identity_1), // RC = 1, 2/3 votes required
        // DID registrars
        "didRegistrars": group_genesis_config(vec![identity_1]),
        // Technical Committee
        "technicalCommitteeMembership": group_genesis_config(vec![identity_3, identity_4, identity_5]), // One GC member + genesis operator + Bridge Multisig
        "technicalCommittee": committee_genesis_config((1, 2), identity_3), // RC = 3, 1/2 votes required
        // Upgrade Committee
        "upgradeCommitteeMembership": group_genesis_config(vec![identity_1]), // One GC member
        "upgradeCommittee": committee_genesis_config((1, 2), identity_1), // 1/2 votes required
        "protocolFee": protocol_fee_genesis_config(),
        "corporateAction": corporate_actions_genesis_config(),
    })
}

fn testnet_boot_nodes() -> Vec<MultiaddrWithPeerId> {
    vec![
        "/dns4/testnet-bootnode-001.polymesh.live/tcp/443/wss/p2p/12D3KooWNG4hedmYixq3Vx4crj5VFxHLFWjqYfbAZwFekHJ8Y7du".parse().expect("Unable to parse bootnode"),
        "/dns4/testnet-bootnode-002.polymesh.live/tcp/443/wss/p2p/12D3KooW9uY8zFnHB5UKyLuwUpZLpPUSJYT2tYfFvpfNCd2K1ceZ".parse().expect("Unable to parse bootnode"),
        "/dns4/testnet-bootnode-003.polymesh.live/tcp/443/wss/p2p/12D3KooWB7AyqsmerKTmcMoyMJJw6ddwWUJ7nFBDGw2viNGN2DBX".parse().expect("Unable to parse bootnode"),
        "/dns4/testnet-bootnode-001.polymesh.live/tcp/30333/p2p/12D3KooWNG4hedmYixq3Vx4crj5VFxHLFWjqYfbAZwFekHJ8Y7du".parse().expect("Unable to parse bootnode"),
        "/dns4/testnet-bootnode-002.polymesh.live/tcp/30333/p2p/12D3KooW9uY8zFnHB5UKyLuwUpZLpPUSJYT2tYfFvpfNCd2K1ceZ".parse().expect("Unable to parse bootnode"),
        "/dns4/testnet-bootnode-003.polymesh.live/tcp/30333/p2p/12D3KooWB7AyqsmerKTmcMoyMJJw6ddwWUJ7nFBDGw2viNGN2DBX".parse().expect("Unable to parse bootnode"),
    ]
}

/// Returns the initial list of validator at genesis representing by their `(AccountId, ValidatorId, Keys)`.
fn session_keys(init_authorities: &[InitialAuth]) -> Vec<(AccountId, AccountId, SessionKeys)> {
    let mut initial_session_keys = Vec::new();

    for initial_auth in init_authorities {
        initial_session_keys.push((
            initial_auth.0.clone(),
            initial_auth.0.clone(),
            SessionKeys {
                grandpa: initial_auth.2.clone(),
                babe: initial_auth.3.clone(),
                im_online: initial_auth.4.clone(),
                authority_discovery: initial_auth.5.clone(),
                beefy: initial_auth.6.clone(),
            },
        ))
    }

    initial_session_keys
}
