use sc_chain_spec::ChainType;
use serde_json::json;

use crate::chain_spec::common::{polymesh_properties, ChainSpec};

pub fn develop_genesis_config() -> serde_json::Value {
    unimplemented!()
}

pub fn develop_chain_spec() -> ChainSpec {
    let code = polymesh_runtime_develop::runtime::WASM_BINARY
        .expect("Development wasm binary is not available.");

    ChainSpec::builder(code, Default::default())
        .with_name("Polymesh CI Develop")
        .with_id("dev_ci")
        .with_chain_type(ChainType::Development)
        .with_genesis_config_patch(develop_genesis_config())
        .with_properties(polymesh_properties(42))
        .build()
}
