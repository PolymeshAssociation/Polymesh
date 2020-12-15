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
use node_primitives::{AccountId, IdentifyAccount, IdentityId, InvestorUid, Signature};
use node_runtime::{config::*, StakerStatus};
use polymesh_common_utilities::constants::currency::*;
use sp_core::ChangesTrieConfiguration;
use sp_core::{sr25519, Pair, Public};
use sp_keyring::{Ed25519Keyring, Sr25519Keyring};
use sp_runtime::{traits::Verify, Perbill};
/// Create genesis runtime configuration for tests.
pub fn config(support_changes_trie: bool) -> GenesisConfig {
    config_endowed(support_changes_trie, Default::default())
}

type AccountPublic = <Signature as Verify>::Signer;

fn get_from_seed<TPublic: Public>(seed: &str) -> <TPublic::Pair as Pair>::Public {
    TPublic::Pair::from_string(&format!("//{}", seed), None)
        .expect("static values are valid; qed")
        .public()
}

fn get_account_id_from_seed<TPublic: Public>(seed: &str) -> AccountId
where
    AccountPublic: From<<TPublic::Pair as Pair>::Public>,
{
    AccountPublic::from(get_from_seed::<TPublic>(seed)).into_account()
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
        balances: Some(BalancesConfig {
            balances: endowed.clone(),
        }),
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
                (
                    IdentityId::from(3),
                    dave(),
                    alice(),
                    111 * DOLLARS,
                    StakerStatus::Validator,
                ),
                (
                    IdentityId::from(4),
                    eve(),
                    bob(),
                    100 * DOLLARS,
                    StakerStatus::Validator,
                ),
                (
                    IdentityId::from(5),
                    ferdie(),
                    charlie(),
                    100 * DOLLARS,
                    StakerStatus::Validator,
                ),
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
            let mut initial_identities = endowed
                .into_iter()
                .enumerate()
                .map(|(i, (account, _))| {
                    (
                        account,
                        IdentityId::from(1usize as u128),
                        IdentityId::from(i as u128),
                        InvestorUid::from([1u8; 16]),
                        None,
                    )
                })
                .collect::<Vec<_>>();

            let initial_len = initial_identities.len();
            initial_identities.reserve(initial_len);
            for i in 0..initial_len {
                initial_identities.push((
                    get_account_id_from_seed::<sr25519::Public>(&format!("random-user//{}", i)),
                    IdentityId::from(1usize as u128),
                    IdentityId::from((i + initial_len) as u128),
                    InvestorUid::from([1u8; 16]),
                    None,
                ))
            }

            Some(IdentityConfig {
                identities: initial_identities,
                ..Default::default()
            })
        },
        bridge: Some(Default::default()),
        pallet_pips: Some(Default::default()),
        group_Instance1: Some(Default::default()),
        committee_Instance1: Some(Default::default()),
        group_Instance2: Some(node_runtime::runtime::CddServiceProvidersConfig {
            active_members_limit: u32::MAX,
            active_members: vec![IdentityId::from(5u128), IdentityId::from(1usize as u128)],
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
        checkpoint: Some(Default::default()),
        multisig: Some(Default::default()),
    }
}
