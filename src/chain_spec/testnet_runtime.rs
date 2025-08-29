use sc_chain_spec::ChainType;
use sc_network::config::MultiaddrWithPeerId;
use sc_telemetry::TelemetryEndpoints;
use serde_json::json;

use crate::chain_spec::common::STAGING_TELEMETRY_URL;
use crate::chain_spec::common::{get_authority_keys_from_seed, polymesh_properties, seeded_acc_id};
use crate::chain_spec::common::{ChainSpec, ChainSpecMode};

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
        .with_genesis_config_patch(testnet_genesis_config())
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
        .with_genesis_config_patch(testnet_genesis_config())
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
        .with_genesis_config_patch(testnet_genesis_config())
        .build()
}

fn testnet_genesis_config() -> serde_json::Value {
    unimplemented!()
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
