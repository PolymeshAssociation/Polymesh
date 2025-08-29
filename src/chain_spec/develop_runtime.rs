use pallet_staking::StakerStatus;
use polymesh_primitives::Balance;
use sc_chain_spec::ChainType;
use sc_rpc::chain;
use serde_json::json;
use sp_runtime::traits::AccountIdConversion;

use polymesh_primitives::constants::TREASURY_PALLET_ID;
use polymesh_primitives::identity_id::GenesisIdentityRecord;
use polymesh_primitives::{AccountId, IdentityId, SecondaryKey};

use crate::chain_spec::common::{get_authority_keys_from_seed, polymesh_properties};
use crate::chain_spec::common::{seeded_acc_id, GenesisData, InitialAuth, StakersData};
use crate::chain_spec::common::{ChainSpec, ChainSpecMode, DEV_KEYS, DEV_TREASURY, INITIAL_BOND};

pub(crate) fn develop_chain_spec(chain_spec_mode: ChainSpecMode) -> ChainSpec {
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
    let initial_authorities = vec![get_authority_keys_from_seed("Alice", false)];

    let other_funded_accounts = vec![
        seeded_acc_id("Bob"),
        seeded_acc_id("Charlie"),
        seeded_acc_id("Dave"),
        seeded_acc_id("Eve"),
    ];

    let root_key = seeded_acc_id("Alice");

    let genesis_json_config =
        develop_genesis_config(initial_authorities, other_funded_accounts, root_key);

    ChainSpec::builder(code, Default::default())
        .with_name("Development")
        .with_id("dev")
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

    let other_funded_accounts = vec![seeded_acc_id("Dave"), seeded_acc_id("Eve")];

    let root_key = seeded_acc_id("Alice");

    let genesis_json_config =
        develop_genesis_config(initial_authorities, other_funded_accounts, root_key);

    ChainSpec::builder(code, Default::default())
        .with_name("Local Development")
        .with_id("local_dev")
        .with_chain_type(ChainType::Local)
        .with_properties(polymesh_properties(42))
        .with_genesis_config_patch(genesis_json_config)
        .build()
}

fn develop_genesis_config(
    initial_authorities: Vec<InitialAuth>,
    other_funded_accounts: Vec<AccountId>,
    root_key: AccountId,
) -> serde_json::Value {
    let genesis_data = genesis_data(&initial_authorities, &other_funded_accounts);

    serde_json::json!({
        "asset": crate::chain_spec::common::asset_genesis_config(),
        "chekpoint": crate::chain_spec::common::checkpoint_genesis_config(),
        "identity": 
        "balances":
        "indices":
        "sudo":
        "session":
        "staking":
        "pips":
        "babe":
        "committee_membership":
        "polymesh_committee":
        "cdd_service_providers":
        "technical_committee_membership":
        "technical_committee":
        "upgrade_committee_membership":
        "upgrade_committee":
        "protocol_fee":
        "corporate_action": 
        "polymesh_contracts":
    })
}

/// Returns [`GenesisData`] given the initial authorities and other funded accounts.
fn genesis_data(
    initial_authorities: &[InitialAuth],
    other_funded_accounts: &[AccountId],
) -> GenesisData {
    let mut stakers_data = Vec::new();
    let mut identities_balance = Vec::new();
    let mut genesis_id_record = GenesisIdentityRecord::new(1u8, initial_authorities[0].0.clone());

    for (stash_acc, controller_acc, ..) in initial_authorities {
        let staker = StakersData::new(
            IdentityId::from(1),
            stash_acc.clone(),
            controller_acc.clone(),
            INITIAL_BOND,
            StakerStatus::Validator,
        );
        stakers_data.push(staker);
        add_secondary_key(
            stash_acc.clone(),
            &mut identities_balance,
            &mut genesis_id_record,
        );
        add_secondary_key(
            controller_acc.clone(),
            &mut identities_balance,
            &mut genesis_id_record,
        );
    }

    for acc_id in other_funded_accounts {
        add_secondary_key(
            acc_id.clone(),
            &mut identities_balance,
            &mut genesis_id_record,
        );
    }

    // Treasury
    identities_balance.push((TREASURY_PALLET_ID.into_account_truncating(), DEV_TREASURY));

    // The 0th key is the primary key
    genesis_id_record.secondary_keys.remove(0);

    GenesisData::new(vec![genesis_id_record], stakers_data, identities_balance)
}

/// Adds `account_id` as a secondary key with full permissions of `genesis_id_record`.
fn add_secondary_key(
    account_id: AccountId,
    identities_balance: &mut Vec<(AccountId, Balance)>,
    genesis_id_record: &mut GenesisIdentityRecord<AccountId>,
) {
    identities_balance.push((account_id.clone(), DEV_KEYS));
    genesis_id_record
        .secondary_keys
        .push(SecondaryKey::from_account_id_with_full_perms(account_id));
}
