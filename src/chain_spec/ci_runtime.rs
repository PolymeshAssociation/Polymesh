use sc_chain_spec::ChainType;
use sp_runtime::PerThing;

use polymesh_primitives::{AccountId, IdentityId, MaybeBlock};
use polymesh_runtime_develop::constants::time::DAYS;
use polymesh_runtime_develop::runtime::BABE_GENESIS_EPOCH_CONFIG;

use crate::chain_spec::common::asset_genesis_config;
use crate::chain_spec::common::seeded_acc_id;
use crate::chain_spec::common::ChainSpec;
use crate::chain_spec::common::{checkpoint_genesis_config, committee_genesis_config};
use crate::chain_spec::common::{corporate_actions_genesis_config, staking_genesis_config};
use crate::chain_spec::common::{get_authority_keys_from_seed, pips_genesis_config};
use crate::chain_spec::common::{group_genesis_config, polymesh_properties};
use crate::chain_spec::common::{protocol_fee_genesis_config, validators_genesis_config};
use crate::chain_spec::common::{ChainSpecMode, InitialAuth};

pub(crate) fn ci_chain_spec(chain_spec_mode: ChainSpecMode) -> ChainSpec {
    let code = polymesh_runtime_develop::runtime::WASM_BINARY
        .expect("Development wasm binary is not available.");

    match chain_spec_mode {
        ChainSpecMode::Bootstrap => unimplemented!(),
        ChainSpecMode::Development => dev_chain_spec(code),
        ChainSpecMode::Local => local_chain_spec(code),
    }
}

/// Returns [`ChainSpec`] for creating a development chain.
fn dev_chain_spec(code: &[u8]) -> ChainSpec {
    let initial_authorities = vec![get_authority_keys_from_seed("Bob", false)];

    let root_key = seeded_acc_id("Alice");

    let genesis_json_config = ci_genesis_config(initial_authorities, root_key);

    ChainSpec::builder(code, Default::default())
        .with_name("Polymesh CI Develop")
        .with_id("dev_ci")
        .with_chain_type(ChainType::Development)
        .with_properties(polymesh_properties(42))
        .with_genesis_config_patch(genesis_json_config)
        .build()
}

/// Returns [`ChainSpec`] for creating a local development chain.
fn local_chain_spec(code: &[u8]) -> ChainSpec {
    let initial_authorities = vec![
        get_authority_keys_from_seed("Alice", false),
        get_authority_keys_from_seed("Bob", false),
        get_authority_keys_from_seed("Charlie", false),
    ];

    let root_key = seeded_acc_id("Alice");

    let genesis_json_config = ci_genesis_config(initial_authorities, root_key);

    ChainSpec::builder(code, Default::default())
        .with_name("Polymesh CI Local")
        .with_id("local_ci")
        .with_chain_type(ChainType::Local)
        .with_properties(polymesh_properties(42))
        .with_genesis_config_patch(genesis_json_config)
        .build()
}

fn ci_genesis_config(
    initial_authorities: Vec<InitialAuth>,
    root_key: AccountId,
) -> serde_json::Value {
    let genesis_data =
        crate::chain_spec::develop_runtime::genesis_data(&initial_authorities, &[root_key.clone()]);

    let session_keys = crate::chain_spec::develop_runtime::session_keys(&initial_authorities);

    let (identity_1, identity_2, identity_3, identity_5) = (
        IdentityId::from(1),
        IdentityId::from(2),
        IdentityId::from(3),
        IdentityId::from(5),
    );

    serde_json::json!({
        "asset": asset_genesis_config(),
        "checkpoint": checkpoint_genesis_config(),
        "identity": {
            "identities": genesis_data.identities_record,
        },
        "balances": {
            "balances": genesis_data.identities_balance,
        },
        "sudo": {
            "key": Some(root_key.clone()),
        },
        "session": {
            "keys": session_keys,
        },
        "validators": validators_genesis_config(&genesis_data.stakers_data, PerThing::zero()),
        "staking": staking_genesis_config(&genesis_data.stakers_data),
        "pips": pips_genesis_config(DAYS * 7, MaybeBlock::None, 1_000),
        "babe": {
            "epochConfig": Some(BABE_GENESIS_EPOCH_CONFIG),
        },
        // Governing council
        "committeeMembership": group_genesis_config(vec![identity_1, identity_2, identity_3, identity_5]),
        "polymeshCommittee": committee_genesis_config((2, 4), identity_1),
        // DID registrars
        "didRegistrars": group_genesis_config(vec![identity_1, identity_2, identity_3, identity_5]),
        // Technical Committee
        "technicalCommitteeMembership": group_genesis_config(vec![identity_3, identity_5]),
        "technicalCommittee": committee_genesis_config((1, 2), identity_5),
        // Upgrade Committee
        "upgradeCommitteeMembership": group_genesis_config(vec![identity_1, identity_5]),
        "upgradeCommittee": committee_genesis_config((1, 2), identity_5),
        "protocolFee": protocol_fee_genesis_config(),
        "corporateAction": corporate_actions_genesis_config(),
    })
}
