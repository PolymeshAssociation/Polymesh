// This file is part of Substrate.

// Copyright (C) 2019-2020 Parity Technologies (UK) Ltd.
// SPDX-License-Identifier: GPL-3.0-or-later WITH Classpath-exception-2.0

// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with this program. If not, see <https://www.gnu.org/licenses/>.

//! Genesis Configuration.

use crate::keyring::*;
use node_primitives::{AccountId, IdentityId, InvestorUid};
use node_runtime::{config::*, StakerStatus};
use polymesh_common_utilities::constants::currency::*;
use sp_core::ChangesTrieConfiguration;
use sp_keyring::{Ed25519Keyring, Sr25519Keyring};
use sp_runtime::Perbill;

/// Create genesis runtime configuration for tests.
pub fn config(support_changes_trie: bool) -> GenesisConfig {
    config_endowed(support_changes_trie, Default::default())
}

/// Create genesis runtime configuration for tests with some extra
/// endowed accounts.
pub fn config_endowed(support_changes_trie: bool, extra_endowed: Vec<AccountId>) -> GenesisConfig {
    let mut endowed = vec![
        (alice(), 111 * DOLLARS),
        (bob(), 100 * DOLLARS),
        (charlie(), 100_000_000 * DOLLARS),
        (dave(), 111 * DOLLARS),
        (eve(), 101 * DOLLARS),
        (ferdie(), 100 * DOLLARS),
    ];

    endowed.extend(
        extra_endowed
            .into_iter()
            .map(|endowed| (endowed, 100 * DOLLARS)),
    );

    GenesisConfig {
        frame_system: Some(SystemConfig {
            changes_trie_config: if support_changes_trie {
                Some(ChangesTrieConfiguration {
                    digest_interval: 2,
                    digest_levels: 2,
                })
            } else {
                None
            },
            code: node_runtime::WASM_BINARY.to_vec(),
        }),
        pallet_indices: Some(IndicesConfig { indices: vec![] }),
        balances: Some(BalancesConfig { balances: endowed }),
        pallet_session: Some(SessionConfig {
            keys: vec![
                (
                    dave(),
                    alice(),
                    to_session_keys(&Ed25519Keyring::Alice, &Sr25519Keyring::Alice),
                ),
                (
                    eve(),
                    bob(),
                    to_session_keys(&Ed25519Keyring::Bob, &Sr25519Keyring::Bob),
                ),
                (
                    ferdie(),
                    charlie(),
                    to_session_keys(&Ed25519Keyring::Charlie, &Sr25519Keyring::Charlie),
                ),
            ],
        }),
        pallet_staking: Some(StakingConfig {
            stakers: vec![
                (dave(), alice(), 111 * DOLLARS, StakerStatus::Validator),
                (eve(), bob(), 100 * DOLLARS, StakerStatus::Validator),
                (ferdie(), charlie(), 100 * DOLLARS, StakerStatus::Validator),
            ],
            validator_count: 3,
            minimum_validator_count: 0,
            slash_reward_fraction: Perbill::from_percent(10),
            invulnerables: vec![alice(), bob(), charlie()],
            ..Default::default()
        }),
        pallet_contracts: Some(ContractsConfig {
            current_schedule: Default::default(),
        }),
        pallet_babe: Some(Default::default()),
        pallet_grandpa: Some(GrandpaConfig {
            authorities: vec![],
        }),
        pallet_im_online: Some(Default::default()),
        pallet_authority_discovery: Some(Default::default()),
        pallet_sudo: Some(Default::default()),
        asset: Some(Default::default()),
        identity: {
			let initial_identities = endowed.enumerate().map(|(account, i)| (
				alice(),
				IdentityId::from(1),
				IdentityId::from(1),
				InvestorUid::from(b"uid1".as_ref()),
				None,
			)).collect::<Vec<_>>();
			for (account, i) in endowed.enumerate() {

			}
            let initial_identities = vec![
                // (primary_account_id, service provider did, target did, expiry time of CustomerDueDiligence claim i.e 10 days is ms)
                // Service providers
                (
                    alice(),
                    IdentityId::from(1),
                    IdentityId::from(1),
                    InvestorUid::from(b"uid1".as_ref()),
                    None,
                ),
                (
                    bob(),
                    IdentityId::from(2),
                    IdentityId::from(2),
                    InvestorUid::from(b"uid2".as_ref()),
                    None,
                ),
                (
                    charlie(),
                    IdentityId::from(3),
                    IdentityId::from(3),
                    InvestorUid::from(b"uid3".as_ref()),
                    None,
                ),
                // Governance committee members
                (
                    dave(),
                    IdentityId::from(1),
                    IdentityId::from(4),
                    InvestorUid::from(b"uid4".as_ref()),
                    None,
                ),
                (
                    eve(),
                    IdentityId::from(2),
                    IdentityId::from(5),
                    InvestorUid::from(b"uid5".as_ref()),
                    None,
                ),
                (
                    ferdie(),
                    IdentityId::from(3),
                    IdentityId::from(6),
                    InvestorUid::from(b"uid6".as_ref()),
                    None,
                ),
            ];

            Some(IdentityConfig {
                identities: initial_identities,
                ..Default::default()
            })
        },
        bridge: Some(Default::default()),
        pallet_pips: Some(Default::default()),
        group_Instance1: Some(node_runtime::runtime::CommitteeMembershipConfig {
            active_members_limit: 20,
            active_members: vec![
                IdentityId::from(3),
                IdentityId::from(4),
                IdentityId::from(5),
                IdentityId::from(6),
            ],
            phantom: Default::default(),
        }),
        committee_Instance1: Some(PolymeshCommitteeConfig {
            vote_threshold: (1, 2),
            members: vec![],
            release_coordinator: IdentityId::from(6),
            phantom: Default::default(),
        }),
        group_Instance2: Some(node_runtime::runtime::CddServiceProvidersConfig {
            active_members_limit: u32::MAX,
            // sp1, sp2, first authority
            active_members: vec![
                IdentityId::from(1),
                IdentityId::from(2),
                IdentityId::from(6),
            ],
            phantom: Default::default(),
        }),
        // Technical Committee:
        group_Instance3: Some(Default::default()),
        committee_Instance3: Some(Default::default()),
        // Upgrade Committee:
        group_Instance4: Some(Default::default()),
        committee_Instance4: Some(Default::default()),
        protocol_fee: Some(Default::default()),
        settlement: Some(Default::default()),
    }
}
