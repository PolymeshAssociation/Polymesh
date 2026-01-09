use sc_chain_spec::ChainType;
use sp_runtime::traits::AccountIdConversion;
use sp_runtime::PerThing;

use pallet_staking::StakerStatus;
use polymesh_primitives::constants::TREASURY_PALLET_ID;
use polymesh_primitives::identity_id::GenesisIdentityRecord;
use polymesh_primitives::{AccountId, Balance, IdentityId, MaybeBlock, SecondaryKey};
use polymesh_runtime_develop::constants::time::MINUTES;
use polymesh_runtime_develop::runtime::{SessionKeys, BABE_GENESIS_EPOCH_CONFIG};

use crate::chain_spec::common::asset_genesis_config;
use crate::chain_spec::common::{checkpoint_genesis_config, committee_genesis_config};
use crate::chain_spec::common::{corporate_actions_genesis_config, staking_genesis_config};
use crate::chain_spec::common::{get_authority_keys_from_seed, pips_genesis_config};
use crate::chain_spec::common::{group_genesis_config, polymesh_properties};
use crate::chain_spec::common::{polymesh_contracts_genesis_config, seeded_acc_id};
use crate::chain_spec::common::{protocol_fee_genesis_config, validators_genesis_config};
use crate::chain_spec::common::{ChainSpec, DEV_KEYS, DEV_TREASURY, INITIAL_BOND};
use crate::chain_spec::common::{ChainSpecMode, GenesisData, InitialAuth, StakersData};

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
        // ETH dev alith
        array_bytes::hex_n_into_unchecked(
            "f24ff3a9cf04c71dbc94d0b566f7a27b94566caceeeeeeeeeeeeeeeeeeeeeeee",
        ),
        // ETH dev baltathar
        array_bytes::hex_n_into_unchecked(
            "3cd0a705a2dc65e5b1e1205896baa2be8a07c6e0eeeeeeeeeeeeeeeeeeeeeeee",
        ),
        // ETH dev charleth
        array_bytes::hex_n_into_unchecked(
            "798d4ba9baf0064ec19eb4f0a1a45785ae9d6dfceeeeeeeeeeeeeeeeeeeeeeee",
        ),
        // ETH dev dorothy
        array_bytes::hex_n_into_unchecked(
            "773539d4ac0e786233d90a233654ccee26a613d9eeeeeeeeeeeeeeeeeeeeeeee",
        ),
        // ETH dev ethan
        array_bytes::hex_n_into_unchecked(
            "ff64d3f6efe2317ee2807d223a0bdc4c0c49dfdbeeeeeeeeeeeeeeeeeeeeeeee",
        ),
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

    let session_keys = session_keys(&initial_authorities);

    let (vote_threshold, identity_1) = ((1, 2), IdentityId::from(1));
    let group_genesis_config = group_genesis_config(vec![identity_1]);
    let committee_genesis_config = committee_genesis_config(vote_threshold, identity_1);

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
        "validators": validators_genesis_config(&genesis_data.stakers_data, PerThing::from_rational(1u64, 4u64)),
        "staking": staking_genesis_config(&genesis_data.stakers_data),
        "pips": pips_genesis_config(MINUTES, MaybeBlock::None, 25),
        "babe": {
            "epochConfig": Some(BABE_GENESIS_EPOCH_CONFIG),
        },
        "committeeMembership": group_genesis_config,
        "polymeshCommittee": committee_genesis_config,
        "didRegistrars": group_genesis_config,
        "technicalCommitteeMembership": group_genesis_config,
        "technicalCommittee": committee_genesis_config,
        "upgradeCommitteeMembership": group_genesis_config,
        "upgradeCommittee": committee_genesis_config,
        "protocolFee": protocol_fee_genesis_config(),
        "corporateAction": corporate_actions_genesis_config(),
        "polymeshContracts": polymesh_contracts_genesis_config(root_key),
    })
}

/// Returns [`GenesisData`] given the initial authorities and other funded accounts.
pub fn genesis_data(
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

/// Returns the initial list of validator at genesis representing by their `(AccountId, ValidatorId, Keys)`.
pub(crate) fn session_keys(
    init_authorities: &[InitialAuth],
) -> Vec<(AccountId, AccountId, SessionKeys)> {
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
