use pallet_staking::StakerStatus;
use sc_chain_spec::ChainType;
use sc_network::config::MultiaddrWithPeerId;
use sc_telemetry::TelemetryEndpoints;
use sp_runtime::traits::AccountIdConversion;
use sp_runtime::PerThing;

use polymesh_primitives::constants::TREASURY_PALLET_ID;
use polymesh_primitives::identity_id::GenesisIdentityRecord;
use polymesh_primitives::{AccountId, MaybeBlock, SecondaryKey};
use polymesh_primitives::{Balance, IdentityId, SystematicIssuers};
use polymesh_runtime_mainnet::constants::time::DAYS;
use polymesh_runtime_mainnet::runtime::{SessionKeys, BABE_GENESIS_EPOCH_CONFIG};

use crate::chain_spec::common::asset_genesis_config;
use crate::chain_spec::common::pips_genesis_config;
use crate::chain_spec::common::{adjust_last, get_authority_keys_from_seed};
use crate::chain_spec::common::{checkpoint_genesis_config, committee_genesis_config};
use crate::chain_spec::common::{corporate_actions_genesis_config, staking_genesis_config};
use crate::chain_spec::common::{group_genesis_config, polymesh_properties, seeded_acc_id};
use crate::chain_spec::common::{protocol_fee_genesis_config, validators_genesis_config};
use crate::chain_spec::common::{ChainSpec, ChainSpecMode};
use crate::chain_spec::common::{GenesisData, InitialAuth, StakersData};
use crate::chain_spec::common::{BOOTSTRAP_KEYS, BOOTSTRAP_TREASURY};
use crate::chain_spec::common::{INITIAL_BOND, STAGING_TELEMETRY_URL};

pub fn mainnet_chain_spec(chain_spec_mode: ChainSpecMode) -> ChainSpec {
    let code = polymesh_runtime_mainnet::runtime::WASM_BINARY
        .expect("Mainnet wasm binary is not available.");

    match chain_spec_mode {
        ChainSpecMode::Bootstrap => bootstap_chain_spec(code),
        ChainSpecMode::Development => dev_chain_spec(code),
        ChainSpecMode::Local => local_chain_spec(code),
    }
}

/// Returns [`ChainSpec`] for bootstrapping mainnet.
fn bootstap_chain_spec(code: &[u8]) -> ChainSpec {
    let root_key = seeded_acc_id("polymesh_5");

    let initial_authorities = vec![
        get_authority_keys_from_seed("Alice", false),
        get_authority_keys_from_seed("Bob", false),
        get_authority_keys_from_seed("Charlie", false),
    ];

    let mainnet_telemetry = TelemetryEndpoints::new(vec![(STAGING_TELEMETRY_URL.to_string(), 0)])
        .expect("Mainnet bootstrap telemetry url is valid; qed");

    ChainSpec::builder(code, Default::default())
        .with_name("Polymesh Mainnet")
        .with_id("mainnet")
        .with_chain_type(ChainType::Live)
        .with_boot_nodes(mainnet_boot_nodes())
        .with_telemetry_endpoints(mainnet_telemetry)
        .with_protocol_id("/polymesh/mainnet")
        .with_properties(polymesh_properties(12))
        .with_genesis_config_patch(mainnet_genesis_config(initial_authorities, root_key))
        .build()
}

/// Returns [`ChainSpec`] for creating a dev mainnet chain.
fn dev_chain_spec(code: &[u8]) -> ChainSpec {
    let root_key = seeded_acc_id("Eve");

    let initial_authorities = vec![get_authority_keys_from_seed("Alice", false)];

    ChainSpec::builder(code, Default::default())
        .with_name("Polymesh Mainnet Develop")
        .with_id("dev_mainnet")
        .with_chain_type(ChainType::Development)
        .with_properties(polymesh_properties(12))
        .with_genesis_config_patch(mainnet_genesis_config(initial_authorities, root_key))
        .build()
}

/// Returns [`ChainSpec`] for creating a local mainnet chain.
fn local_chain_spec(code: &[u8]) -> ChainSpec {
    let root_key = seeded_acc_id("Eve");

    let initial_authorities = vec![
        get_authority_keys_from_seed("Alice", false),
        get_authority_keys_from_seed("Bob", false),
        get_authority_keys_from_seed("Charlie", false),
    ];

    ChainSpec::builder(code, Default::default())
        .with_name("Polymesh Mainnet Local")
        .with_id("local_mainnet")
        .with_chain_type(ChainType::Local)
        .with_properties(polymesh_properties(12))
        .with_genesis_config_patch(mainnet_genesis_config(initial_authorities, root_key))
        .build()
}

fn mainnet_genesis_config(
    initial_authorities: Vec<InitialAuth>,
    root_key: AccountId,
) -> serde_json::Value {
    let genesis_data = genesis_data(&initial_authorities, root_key.clone());

    let session_keys = session_keys(&initial_authorities);

    let (identity_1, identity_2, identity_3) = (
        IdentityId::from(1),
        IdentityId::from(2),
        IdentityId::from(3),
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
        "session": {
            "keys": session_keys,
        },
        "validators": validators_genesis_config(&genesis_data.stakers_data, PerThing::from_rational(1u64, 4u64)),
        "staking": staking_genesis_config(&genesis_data.stakers_data),
        "pips": pips_genesis_config(DAYS * 30, MaybeBlock::Some(DAYS * 90), 1_000),
        "babe": {
            "epochConfig": Some(BABE_GENESIS_EPOCH_CONFIG),
        },
        // Governing council
        "committeeMembership": group_genesis_config(vec![identity_1, identity_2, identity_3]),  // three GC members
        "polymeshCommittee": committee_genesis_config((2, 3), identity_1), // RC = 1, 2/3 votes required
        // DID registrars
        "didRegistrars": group_genesis_config(vec![identity_1]), // GC_1 is also a DID registrar
        // Technical Committee
        "technicalCommitteeMembership": group_genesis_config(vec![identity_1]), // One GC member
        "technicalCommittee": committee_genesis_config((1, 2), identity_1), // 1/2 votes required
        // Upgrade Committee
        "upgradeCommitteeMembership": group_genesis_config(vec![identity_1]), // One GC member
        "upgradeCommittee": committee_genesis_config((1, 2), identity_1), // 1/2 votes required
        "protocolFee": protocol_fee_genesis_config(),
        "corporateAction": corporate_actions_genesis_config(),
    })
}

pub(crate) fn genesis_data(
    initial_authorities: &[InitialAuth],
    root_key: AccountId,
) -> GenesisData {
    // Identities and their roles
    // 1 = [Polymesh] GenesisCouncil (1 of 3) + UpgradeCommittee (1 of 1) + TechnicalCommittee (1 of 1) + GCReleaseCoordinator
    // 2 = GenesisCouncil (2 of 3)
    // 3 = GenesisCouncil (3 of 3)
    // 4 = Operator
    // 5 = Sudo

    // Identity_01
    // Primary Key: polymesh_1

    // Identity_02
    // Primary Key: polymesh_2

    // Identity_03
    // Primary Key: polymesh_3

    // Identity_04
    // Primary Key: polymesh_4
    // Secondary Keys: Alice, Alice//stash, Bob, Bob//stash, Charlie, Charlie//stash

    // Identity_05
    // Primary Key: polymesh_5

    let mut identities_balance = Vec::new();
    let mut genesis_id_records = Vec::new();

    // Creating Identities 1-4 (GC + Operators)
    for i in 1..=4 {
        let new_acc = seeded_acc_id(adjust_last(&mut { *b"polymesh_0" }, i));
        create_identity(i, new_acc, &mut identities_balance, &mut genesis_id_records);
    }

    // Creating identity for sudo
    create_identity(
        5u8,
        root_key,
        &mut identities_balance,
        &mut genesis_id_records,
    );

    let mut stakers_data = Vec::new();
    for (stash_acc, controller_acc, ..) in initial_authorities {
        let staker = StakersData::new(
            IdentityId::from(4), // All operators have the same identity
            stash_acc.clone(),
            controller_acc.clone(),
            INITIAL_BOND,
            StakerStatus::Validator,
        );
        stakers_data.push(staker);
        add_secondary_key(
            stash_acc.clone(),
            &mut identities_balance,
            &mut genesis_id_records,
        );
        add_secondary_key(
            controller_acc.clone(),
            &mut identities_balance,
            &mut genesis_id_records,
        );
    }

    // Give CDD issuer to operator and sudo since it won't receive CDD from the group automatically
    genesis_id_records[3]
        .issuers
        .push(SystematicIssuers::CDDProvider.as_id());

    // Give CDD issuer to operator and sudo since it won't receive CDD from the group automatically
    genesis_id_records[4]
        .issuers
        .push(SystematicIssuers::CDDProvider.as_id());

    // Treasury
    identities_balance.push((
        TREASURY_PALLET_ID.into_account_truncating(),
        BOOTSTRAP_TREASURY,
    ));

    GenesisData::new(genesis_id_records, stakers_data, identities_balance)
}

fn create_identity(
    nonce: u8,
    primary_key: AccountId,
    account_balances: &mut Vec<(AccountId, Balance)>,
    genesis_identities: &mut Vec<GenesisIdentityRecord<AccountId>>,
) {
    account_balances.push((primary_key.clone(), BOOTSTRAP_KEYS));
    genesis_identities.push(GenesisIdentityRecord::new(nonce, primary_key));
}

fn add_secondary_key(
    account_id: AccountId,
    identities_balance: &mut Vec<(AccountId, Balance)>,
    genesis_id_record: &mut Vec<GenesisIdentityRecord<AccountId>>,
) {
    identities_balance.push((account_id.clone(), BOOTSTRAP_KEYS));
    genesis_id_record[3]
        .secondary_keys
        .push(SecondaryKey::from_account_id_with_full_perms(account_id));
}

fn mainnet_boot_nodes() -> Vec<MultiaddrWithPeerId> {
    vec![
        "/dns4/mainnet-bootnode-001.polymesh.network/tcp/443/wss/p2p/12D3KooWDiaRBvzjt1p95mTqJETxJw3nz1E6fF2Yf62ojimEGJS7".parse().expect("Unable to parse bootnode"),
        "/dns4/mainnet-bootnode-002.polymesh.network/tcp/443/wss/p2p/12D3KooWN9E6gtgybnXwDVNMUGwSA82pzBj72ibGYfZuomyEDQTU".parse().expect("Unable to parse bootnode"),
        "/dns4/mainnet-bootnode-003.polymesh.network/tcp/443/wss/p2p/12D3KooWQ3K8jGadCQSVhihLEsJfSz3TJGgBHMU3vTtK3jd2Wq5E".parse().expect("Unable to parse bootnode"),
        "/dns4/mainnet-bootnode-004.polymesh.network/tcp/443/wss/p2p/12D3KooWAjLb7S2FKk1Bxyw3vkaqgcSpjfxHwpGvqcXACFYSK8Xq".parse().expect("Unable to parse bootnode"),
        "/dns4/mainnet-bootnode-005.polymesh.network/tcp/443/wss/p2p/12D3KooWKvXCP5b5PW4tHFAYyFVk3kRhwF3qXJbnVcPSGHP6Zmjg".parse().expect("Unable to parse bootnode"),
        "/dns4/mainnet-bootnode-006.polymesh.network/tcp/443/wss/p2p/12D3KooWBQhDAjfo13dM4nsogXD39F5TcN9iTVzjXgPqFn9Yaccz".parse().expect("Unable to parse bootnode"),
        "/dns4/mainnet-bootnode-007.polymesh.network/tcp/443/wss/p2p/12D3KooWMwFdYC53MqdyR9WYvJiPfxfYXh65NfY9QSuZeyKa53fg".parse().expect("Unable to parse bootnode"),
        "/dns4/mainnet-bootnode-001.polymesh.network/tcp/30333/p2p/12D3KooWDiaRBvzjt1p95mTqJETxJw3nz1E6fF2Yf62ojimEGJS7".parse().expect("Unable to parse bootnode"),
        "/dns4/mainnet-bootnode-002.polymesh.network/tcp/30333/p2p/12D3KooWN9E6gtgybnXwDVNMUGwSA82pzBj72ibGYfZuomyEDQTU".parse().expect("Unable to parse bootnode"),
        "/dns4/mainnet-bootnode-003.polymesh.network/tcp/30333/p2p/12D3KooWQ3K8jGadCQSVhihLEsJfSz3TJGgBHMU3vTtK3jd2Wq5E".parse().expect("Unable to parse bootnode"),
        "/dns4/mainnet-bootnode-004.polymesh.network/tcp/30333/p2p/12D3KooWAjLb7S2FKk1Bxyw3vkaqgcSpjfxHwpGvqcXACFYSK8Xq".parse().expect("Unable to parse bootnode"),
        "/dns4/mainnet-bootnode-005.polymesh.network/tcp/30333/p2p/12D3KooWKvXCP5b5PW4tHFAYyFVk3kRhwF3qXJbnVcPSGHP6Zmjg".parse().expect("Unable to parse bootnode"),
        "/dns4/mainnet-bootnode-006.polymesh.network/tcp/30333/p2p/12D3KooWBQhDAjfo13dM4nsogXD39F5TcN9iTVzjXgPqFn9Yaccz".parse().expect("Unable to parse bootnode"),
        "/dns4/mainnet-bootnode-007.polymesh.network/tcp/30333/p2p/12D3KooWMwFdYC53MqdyR9WYvJiPfxfYXh65NfY9QSuZeyKa53fg".parse().expect("Unable to parse bootnode"),
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
