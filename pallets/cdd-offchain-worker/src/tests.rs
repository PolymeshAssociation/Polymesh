// This file is part of the Polymesh distribution (https://github.com/PolymathNetwork/Polymesh).
// Copyright (c) 2020 Polymath

// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, version 3.

// This program is distributed in the hope that it will be useful, but
// WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the GNU
// General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with this program. If not, see <http://www.gnu.org/licenses/>.

//! Tests for the module.

use crate::*;
use chrono::prelude::Utc;
use codec::{Decode, Encode};
use frame_support::{assert_err, assert_noop, assert_ok, dispatch::DispatchError};
use mock::*;
use pallet_staking::RewardDestination;
use primitives::{IdentityId, Signatory};
use sp_core::{
    offchain::{testing, OffchainExt, TransactionPoolExt},
    testing::KeyStore,
    traits::KeystoreExt,
    H256,
};
use sp_runtime::testing::UintAuthorityId;
use sp_runtime::RuntimeAppPublic;
use test_client::AccountKeyring;

#[test]
fn check_the_initial_nominators_of_chain() {
    ExtBuilder::default()
        .nominate(true)
        .build()
        .execute_with(|| {
            // Check the initial nominator status and expiry
            assert!(Staking::nominators(account_from(101)).is_some());
            // Check the identity expiry
            assert!(Identity::has_valid_cdd(IdentityId::from(101)));
        });
}

#[test]
fn should_submit_unsigned_transaction_on_chain() {
    use sp_runtime::traits::OffchainWorker;

    const PHRASE: &str =
        "foster nation swing usage bread mind donor door whisper lyrics token enroll";

    let (offchain, offchain_state) = testing::TestOffchainExt::new();
    let (pool, pool_state) = testing::TestTransactionPoolExt::new();
    let keystore = KeyStore::new();
    keystore
        .write()
        .sr25519_generate_new(
            crate::crypto::SignerId::ID,
            Some(&format!("{}/worker1", PHRASE)),
        )
        .unwrap();

    let mut txn = ExtBuilder::default().nominate(true).build();
    txn.register_extension(OffchainExt::new(offchain));
    txn.register_extension(TransactionPoolExt::new(pool));
    txn.register_extension(KeystoreExt(keystore));

    txn.execute_with(|| {
        System::set_block_number(1); // set block 1
        UintAuthorityId::set_all_keys(vec![0, 1]);
        // Add nominators in the chain

        // 2 will nominate for 10, 20
        let nominator_stake = 500;
        assert_ok!(Staking::bond(
            Origin::signed(account_from(1)),
            account_from(2),
            nominator_stake,
            RewardDestination::default()
        ));

        let bonding_time = Staking::get_bonding_duration_period();
        let now = (Utc::now()).timestamp() as u64;
        Timestamp::set_timestamp(now * 1000);
        let expiry = now * 1000 + bonding_time * 1000 + 10000_u64; // in MS
                                                                   // Add identity to the stash 1
        create_did_and_add_claim(account_from(1), expiry);
        // nominate after did has the valid claim
        assert_ok!(Staking::nominate(
            Origin::signed(account_from(2)),
            vec![account_from(20), account_from(10)]
        ));

        System::set_block_number(2); // set block 2
                                     // 3 will nominate 30 & 20
        assert_ok!(Staking::bond(
            Origin::signed(account_from(3)),
            account_from(4),
            nominator_stake,
            RewardDestination::default()
        ));
        // Add identity to the stash 3
        create_did_and_add_claim(account_from(3), expiry);
        // nominate after did has the valid claim
        assert_ok!(Staking::nominate(
            Origin::signed(account_from(4)),
            vec![account_from(20), account_from(30)]
        ));

        System::set_block_number(3); // set block 3
        assert_eq!((Staking::fetch_invalid_cdd_nominators(0)).len(), 0);

        let now = (Utc::now()).timestamp() as u64;
        Timestamp::set_timestamp(now * 1000 + 110000); // increasing time of the chain

        assert_eq!((Staking::fetch_invalid_cdd_nominators(0)).len(), 2);
        let invalid_nominators = Staking::fetch_invalid_cdd_nominators(0);
        let block = 4;
        System::set_block_number(block); // set block 4

        CddOffchainWorker::offchain_worker(block);

        // Get the transaction
        let tx = pool_state.write().transactions.pop().unwrap();
        assert!(pool_state.read().transactions.is_empty());
        let ex = Extrinsic::decode(&mut &*tx).unwrap();
        let target =
            match ex.call {
                crate::mock::Call::CddOffchainWorker(
                    crate::Call::take_off_invalidate_nominators(_, t, _),
                ) => t,
                e => panic!("Unexpected call: {:?}", e),
            };
        assert_eq!(ex.signature, None);
        assert_eq!(target, invalid_nominators);
    });
}
