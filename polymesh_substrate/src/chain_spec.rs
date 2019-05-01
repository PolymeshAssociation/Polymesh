use hex_literal::{hex, hex_impl};
pub use node_template_runtime::GenesisConfig;
use node_template_runtime::{
    AccountId, AssetConfig, BalancesConfig, ConsensusConfig, CouncilSeatsConfig,
    CouncilVotingConfig, DemocracyConfig, GrandpaConfig, IdentityConfig, IndicesConfig, Perbill,
    Permill, SessionConfig, StakerStatus, StakingConfig, SudoConfig, TimestampConfig,
    TreasuryConfig,
};
use primitives::{crypto::UncheckedInto, ed25519, ed25519::Public as AuthorityId, sr25519, Pair};
use substrate_service;

// Note this is the URL for the telemetry server
//const STAGING_TELEMETRY_URL: &str = "wss://telemetry.polkadot.io/submit/";

/// Specialized `ChainSpec`. This is a specialization of the general Substrate ChainSpec type.
pub type ChainSpec = substrate_service::ChainSpec<GenesisConfig>;

/// The chain specification option. This is expected to come in from the CLI and
/// is little more than one of a number of alternatives which can easily be converted
/// from a string (`--chain=...`) into a `ChainSpec`.
#[derive(Clone, Debug)]
pub enum Alternative {
    /// Whatever the current runtime is, with just Alice as an auth.
    Development,
    /// Whatever the current runtime is, with simple Alice/Bob auths.
    LocalTestnet,
    Aws,
}

impl Alternative {
    /// Get an actual chain config from one of the alternatives.
    pub(crate) fn load(self) -> Result<ChainSpec, String> {
        Ok(match self {
            Alternative::Aws => ChainSpec::from_genesis(
                "AWS",
                "aws",
                || {
                    testnet_genesis(
                        vec![
                            get_authority_keys_from_seed("Alice"),
                            get_authority_keys_from_seed("Bob"),
                            get_authority_keys_from_seed("Charlie"),
                        ],
                        get_account_id_from_seed("Alice"),
                    )
                },
                vec![],
                None,
                None,
                None,
                None,
            ),
            Alternative::Development => ChainSpec::from_genesis(
                "Development",
                "dev",
                || {
                    testnet_genesis(
                        vec![get_authority_keys_from_seed("Alice")],
                        get_account_id_from_seed("Alice"),
                    )
                },
                vec![],
                None,
                None,
                None,
                None,
            ),
            Alternative::LocalTestnet => ChainSpec::from_genesis(
                "Local Testnet",
                "local_testnet",
                || {
                    testnet_genesis(
                        vec![
                            get_authority_keys_from_seed("Alice"),
                            get_authority_keys_from_seed("Bob"),
                            get_authority_keys_from_seed("Charlie"),
                        ],
                        get_account_id_from_seed("Alice"),
                    )
                },
                vec![],
                None,
                None,
                None,
                None,
            ),
        })
    }

    pub(crate) fn from(s: &str) -> Option<Self> {
        match s {
            "dev" => Some(Alternative::Development),
            "local2" => Some(Alternative::LocalTestnet),
            "" | "aws" => Some(Alternative::Aws),
            _ => None,
        }
    }
}

// fn testnet_genesis2(
//     endowed_accounts: Vec<AccountId>,
//     root_key: AccountId,
// ) -> GenesisConfig {
//     let initial_authorities: Vec<(AccountId, AccountId, AuthorityId)> = vec![(
// 		hex!["f8c3fe049c7ce8ad7387ec7ee31aa28790e1aa742e9b4d2b15b983dfb51cce29"].unchecked_into(),
// 		hex!["ae7f9412cb860d27303ed3296ddca201cdb3b24c9cf68bbf78923c99bb71e961"].unchecked_into(),
// 		hex!["6e151e7d42cf5953ce28f63f7ba60f46dade85c62564304bc8c1a0f6dcfb947c"].unchecked_into(),
// 	),(
// 		hex!["6e440d594f7e32a728004fab5ff5ec119b117ae023140ca94289c2d0a09cec0e"].unchecked_into(),
// 		hex!["62b365b29c91928b702f2af5f7828428ffff2d43c3d3ce3e1ed95c5c500a171a"].unchecked_into(),
// 		hex!["bef6b797203adb34bf688772b63e9a0b82bfa040c0e991bbcaf97f9121cec579"].unchecked_into(),
// 	),(
// 		hex!["ba20e735f1d1529c8fd1f190f6f83d27b6e9ebacfa40584ac3f6e1e3c129966f"].unchecked_into(),
// 		hex!["2008e0ab4d6efd47504a58aff3c7e39d209236ac79e8ad4dc93498e5cb45d71c"].unchecked_into(),
// 		hex!["403c6b6bde365a6b136bceb7972a189873e7d8b472d8007b3e14fa25b8d64d6f"].unchecked_into(),
// 	),(
// 		hex!["c4565fd29e8708cddbd33ce847cb2c7f266d3e438c64a150e7a18f0f4440427f"].unchecked_into(),
// 		hex!["ec5a499f08a8a61b9b0a123a7d7f904e50421cf3e17a848e33862a8328945525"].unchecked_into(),
// 		hex!["184f6a44548777a778d7ba9bf926db51fa16b6ae971450c6b760dc795cf22a1a"].unchecked_into(),
// 	)];
//     testnet_genesis(
// 		vec![
// 			get_authority_keys_from_seed("Alice"),
// 			get_authority_keys_from_seed("Bob"),
//             get_authority_keys_from_seed("Charlie"),
// 		],
// 		get_account_id_from_seed("Alice"),
// 		None,
// 	)
// }

/// Helper function to generate AccountId from seed
pub fn get_account_id_from_seed(seed: &str) -> AccountId {
    sr25519::Pair::from_string(&format!("//{}", seed), None)
        .expect("static values are valid; qed")
        .public()
}

/// Helper function to generate AuthorityId from seed
pub fn get_session_key_from_seed(seed: &str) -> AuthorityId {
    ed25519::Pair::from_string(&format!("//{}", seed), None)
        .expect("static values are valid; qed")
        .public()
}

/// Helper function to generate stash, controller and session key from seed
pub fn get_authority_keys_from_seed(seed: &str) -> (AccountId, AccountId, AuthorityId) {
    (
        get_account_id_from_seed(&format!("{}//stash", seed)),
        get_account_id_from_seed(seed),
        get_session_key_from_seed(seed),
    )
}

/// Helper function to create GenesisConfig for testing
pub fn testnet_genesis(
    initial_authorities: Vec<(AccountId, AccountId, AuthorityId)>,
    root_key: AccountId,
) -> GenesisConfig {
    let endowed_accounts: Vec<AccountId> = vec![
        // hex!["687f80444e7a32d6dd9cd7e8c8229cec5c9a504634f0f83a0572234826a95024"].unchecked_into(),
        // hex!["f8c3fe049c7ce8ad7387ec7ee31aa28790e1aa742e9b4d2b15b983dfb51cce29"].unchecked_into(),
        // hex!["ae7f9412cb860d27303ed3296ddca201cdb3b24c9cf68bbf78923c99bb71e961"].unchecked_into(),
        // hex!["11ed3fc1bd16dd030b695fb1a7a51cbd844bdc8465de856fb4e545e73e94d666"].unchecked_into(),
        get_account_id_from_seed("Alice"),
        get_account_id_from_seed("Bob"),
        get_account_id_from_seed("Charlie"),
        get_account_id_from_seed("Dave"),
        get_account_id_from_seed("Eve"),
        get_account_id_from_seed("Ferdie"),
        get_account_id_from_seed("Alice//stash"),
        get_account_id_from_seed("Bob//stash"),
        get_account_id_from_seed("Charlie//stash"),
        get_account_id_from_seed("Dave//stash"),
        get_account_id_from_seed("Eve//stash"),
        get_account_id_from_seed("Ferdie//stash"),
    ];

    const STASH: u128 = 1 << 20;
    const ENDOWMENT: u128 = 1 << 20;

    GenesisConfig {
        consensus: Some(ConsensusConfig {
            code: include_bytes!("../runtime/wasm/target/wasm32-unknown-unknown/release/node_template_runtime_wasm.compact.wasm").to_vec(),
            authorities: initial_authorities.iter().map(|x| x.2.clone()).collect(),
        }),
        system: None,
        indices: Some(IndicesConfig {
            ids: endowed_accounts.clone(),
        }),
        asset: Some(AssetConfig {
            asset_creation_fee: 250,
            fee_collector: get_account_id_from_seed("Dave"),
        }),
        identity: Some(IdentityConfig {
            owner: get_account_id_from_seed("Dave"),
        }),
        balances: Some(BalancesConfig {
            transaction_base_fee: 1,
            transaction_byte_fee: 0,
            existential_deposit: 500,
            transfer_fee: 0,
            creation_fee: 0,
            balances: endowed_accounts.iter().map(|k| (k.clone(), ENDOWMENT)).collect(),
            vesting: vec![],
        }),
        session: Some(SessionConfig {
            validators: initial_authorities.iter().map(|x| x.1.clone()).collect(),
            session_length: 10,
            keys: initial_authorities.iter().map(|x| (x.1.clone(), x.2.clone())).collect::<Vec<_>>(),
        }),
        staking: Some(StakingConfig {
            current_era: 0,
            minimum_validator_count: 1,
            validator_count: 2,
            sessions_per_era: 5,
            bonding_duration: 2 * 60 * 12,
            offline_slash: Perbill::zero(),
            session_reward: Perbill::zero(),
            current_session_reward: 0,
            offline_slash_grace: 0,
            stakers: initial_authorities.iter().map(|x| (x.0.clone(), x.1.clone(), STASH, StakerStatus::Validator)).collect(),
            invulnerables: initial_authorities.iter().map(|x| x.1.clone()).collect(),
        }),
        democracy: Some(DemocracyConfig {
            launch_period: 9,
            voting_period: 18,
            minimum_deposit: 10,
            public_delay: 0,
            max_lock_periods: 6,
        }),
        council_seats: Some(CouncilSeatsConfig {
            active_council: endowed_accounts.iter()
                .filter(|&endowed| initial_authorities.iter().find(|&(_, controller, _)| controller == endowed).is_none())
                .map(|a| (a.clone(), 1000000)).collect(),
            candidacy_bond: 10,
            voter_bond: 2,
            present_slash_per_voter: 1,
            carry_count: 4,
            presentation_duration: 10,
            approval_voting_period: 20,
            term_duration: 1000000,
            desired_seats: (endowed_accounts.len() / 2 - initial_authorities.len()) as u32,
            inactive_grace_period: 1,
        }),
        council_voting: Some(CouncilVotingConfig {
            cooloff_period: 75,
            voting_period: 20,
            enact_delay_period: 0,
        }),
        timestamp: Some(TimestampConfig {
            minimum_period: 5,                    // 5*2=10 second block time.
        }),
        treasury: Some(TreasuryConfig {
            proposal_bond: Permill::from_percent(5),
            proposal_bond_minimum: 1_000_000,
            spend_period: 12 * 60 * 24,
            burn: Permill::from_percent(50),
        }),
        sudo: Some(SudoConfig {
            key: root_key,
        }),
        grandpa: Some(GrandpaConfig {
            authorities: initial_authorities.iter().map(|x| (x.2.clone(), 1)).collect(),
        }),
    }
}
