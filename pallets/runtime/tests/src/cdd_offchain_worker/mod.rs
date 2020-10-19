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

//! Tests for the CddOffchainWorker pallet.
mod mock;
use chrono::prelude::Utc;
use codec::Decode;
use frame_support::{assert_err, assert_ok};
use frame_system::offchain::{SignedPayload, SigningTypes};
use mock::{
    account_from, add_secondary_key, make_account_with_balance, AccountId, Balance, Call,
    ExtBuilder, Extrinsic, Moment, Origin, Test, TestAppCryptoId,
};
use pallet_cdd_offchain_worker::{crypto, Payload};
use pallet_identity as identity;
use pallet_staking::RewardDestination;
use pallet_timestamp;
use polymesh_primitives::{IdentityId, InvestorUid};
use sp_core::{
    offchain::{testing, OffchainExt, TransactionPoolExt},
    testing::KeyStore,
    traits::KeystoreExt,
};
use sp_runtime::{transaction_validity::InvalidTransaction, RuntimeAppPublic};
use test_client::AccountKeyring;

type CddOffchainWorker = pallet_cdd_offchain_worker::Module<Test>;
type Staking = pallet_staking::Module<Test>;
type Identity = identity::Module<Test>;
pub type System = frame_system::Module<Test>;
pub type Timestamp = pallet_timestamp::Module<Test>;
type CddWorkerError = pallet_cdd_offchain_worker::Error<Test>;

fn add_nominator(stash: u64, value: Balance, expiry: Option<Moment>) {
    let uid = InvestorUid::from(format!("uid_{}", stash).as_bytes());
    let (signed, _) = make_account_with_balance(account_from(stash), uid, expiry, 200000).unwrap();
    add_secondary_key(account_from(stash), account_from(stash - 1));
    assert_ok!(Staking::bond(
        signed,
        account_from(stash - 1),
        value,
        RewardDestination::Controller
    ));
    assert_ok!(Staking::nominate(
        Origin::signed(account_from(stash - 1)),
        vec![account_from(11)]
    ));
}

#[test]
fn check_the_initial_nominators_of_chain() {
    ExtBuilder::default()
        .nominate(true)
        .build()
        .execute_with(|| {
            // Check the initial nominator status and expiry
            assert!(Staking::nominators(account_from(21)).is_some());
            // Check the identity expiry
            assert!(Identity::has_valid_cdd(IdentityId::from(2)));
        });
}

#[test]
fn should_submit_solution_from_runtime() {
    const PHRASE: &str =
        "news slush supreme milk chapter athlete soap sausage put clutch what kitten";
    let (offchain, _) = testing::TestOffchainExt::new();
    let (pool, pool_state) = testing::TestTransactionPoolExt::new();

    let keystore = KeyStore::new();

    keystore
        .write()
        .sr25519_generate_new(
            crypto::sr25519_app::Public::ID,
            Some(&format!("{}/hunter1", PHRASE)),
        )
        .unwrap();

    let mut txn = ExtBuilder::default()
        .nominate(true)
        .cdd_providers(vec![AccountId::from(AccountKeyring::Alice.public())])
        .build();
    txn.register_extension(OffchainExt::new(offchain));
    txn.register_extension(TransactionPoolExt::new(pool));
    txn.register_extension(KeystoreExt(keystore.clone()));

    let public_key = keystore
        .read()
        .sr25519_public_keys(crypto::sr25519_app::Public::ID)
        .get(0)
        .unwrap()
        .clone();

    let payload = Payload {
        block_number: 4,
        nominators: vec![account_from(601), account_from(701), account_from(501)],
        public: <Test as SigningTypes>::Public::from(public_key),
    };

    txn.execute_with(|| {

        let block = 1;
        System::set_block_number(block);

        let bonding_time = Staking::get_bonding_duration_period();
        let now = (Utc::now()).timestamp() as u64;
        Timestamp::set_timestamp(now * 1000);
        let expiry = now * 1000 + bonding_time * 1000 + 10000_u64; // in MS

        // Add 3 or 4 nominators in the system which has some expired cdd.
        add_nominator(501, 10000, Some(expiry));
        System::set_block_number(block + 1);
        add_nominator(601, 10000, Some(expiry + 1000));
        System::set_block_number(block + 2);
        add_nominator(701, 10000, Some(expiry + 2000));
        System::set_block_number(block + 3);
        add_nominator(801, 10000, Some(expiry + 3000));

        System::set_block_number(block + 4);
        // Set new timestamp
        Timestamp::set_timestamp(expiry + 2000);

        assert_ok!(CddOffchainWorker::remove_invalidate_nominators(4));


        // then
		let tx = pool_state.write().transactions.pop().unwrap();
		let tx = Extrinsic::decode(&mut &*tx).unwrap();
		assert_eq!(tx.signature, None);
		if let Call::CddOffchainWorker(pallet_cdd_offchain_worker::Call::submit_unsigned_invalidate_nominators_with_signed_payload(body, signature)) = tx.call {
            assert_eq!(body, payload);

			let signature_valid = <Payload<
				<Test as SigningTypes>::Public,
				<Test as frame_system::Trait>::BlockNumber, AccountId
					> as SignedPayload<Test>>::verify::<TestAppCryptoId>(&payload, signature.clone());

            assert!(signature_valid);
            // Execute the transaction with the same params
            assert_ok!(CddOffchainWorker::submit_unsigned_invalidate_nominators_with_signed_payload(Origin::none(), body, signature));
        }

        // validate the no. of nominators
        assert!(Staking::nominators(account_from(501)).is_none());
        assert!(Staking::nominators(account_from(601)).is_none());
        assert!(Staking::nominators(account_from(701)).is_none());
    });
}

#[test]
fn should_submit_raw_unsigned_transaction_on_chain() {
    const PHRASE: &str =
        "news slush supreme milk chapter athlete soap sausage put clutch what kitten";
    let (offchain, _) = testing::TestOffchainExt::new();
    let (pool, pool_state) = testing::TestTransactionPoolExt::new();

    let keystore = KeyStore::new();

    keystore
        .write()
        .sr25519_generate_new(
            crypto::sr25519_app::Public::ID,
            Some(&format!("{}/hunter1", PHRASE)),
        )
        .unwrap();

    let mut txn = ExtBuilder::default()
        .nominate(true)
        .cdd_providers(vec![AccountId::from(AccountKeyring::Alice.public())])
        .build();
    txn.register_extension(OffchainExt::new(offchain));
    txn.register_extension(TransactionPoolExt::new(pool));
    txn.register_extension(KeystoreExt(keystore.clone()));

    let public_key = keystore
        .read()
        .sr25519_public_keys(crypto::sr25519_app::Public::ID)
        .get(0)
        .unwrap()
        .clone();

    let payload = Payload {
        block_number: 5,
        nominators: vec![account_from(601), account_from(701), account_from(501)],
        public: <Test as SigningTypes>::Public::from(public_key),
    };

    txn.execute_with(|| {

        let block = 1;
        System::set_block_number(block);

        let bonding_time = Staking::get_bonding_duration_period();
        let now = (Utc::now()).timestamp() as u64;
        Timestamp::set_timestamp(now * 1000);
        let expiry = now * 1000 + bonding_time * 1000 + 10000_u64; // in MS

        // Add 3 or 4 nominators in the system which has some expired cdd.
        add_nominator(501, 10000, Some(expiry));
        System::set_block_number(block + 1);
        add_nominator(601, 10000, Some(expiry + 1000));
        System::set_block_number(block + 2);
        add_nominator(701, 10000, Some(expiry + 2000));
        System::set_block_number(block + 3);
        add_nominator(801, 10000, Some(expiry + 3000));
        System::set_block_number(block + 4);
        add_nominator(901, 10000, Some(expiry + 3000));
        System::set_block_number(block + 5);
        // Set new timestamp
        Timestamp::set_timestamp(expiry + 2001);

        assert_ok!(CddOffchainWorker::remove_invalidate_nominators(5));


        // then
		let tx = pool_state.write().transactions.pop().unwrap();
		let tx = Extrinsic::decode(&mut &*tx).unwrap();
		assert_eq!(tx.signature, None);
		if let Call::CddOffchainWorker(pallet_cdd_offchain_worker::Call::submit_unsigned_invalidate_nominators_with_signed_payload(body, signature)) = tx.call {
            assert_eq!(body, payload);

			let signature_valid = <Payload<
				<Test as SigningTypes>::Public,
				<Test as frame_system::Trait>::BlockNumber, AccountId
					> as SignedPayload<Test>>::verify::<TestAppCryptoId>(&payload, signature.clone());

            assert!(signature_valid);

            let new_payload_with_corrupted_block_number = Payload {
                block_number: 60,
                nominators: vec![account_from(601), account_from(701), account_from(501)],
                public: <Test as SigningTypes>::Public::from(public_key),
            };
            use frame_support::unsigned::ValidateUnsigned;
            // Execute the transaction with the different body and check for the validate unsigned
            let call = pallet_cdd_offchain_worker::Call::submit_unsigned_invalidate_nominators_with_signed_payload(new_payload_with_corrupted_block_number, signature.clone());
            assert_err!(CddOffchainWorker::pre_dispatch(&call), InvalidTransaction::Future);

            let new_payload_with_wrong_data_set = Payload {
                block_number: 5,
                nominators: vec![account_from(601), account_from(701)],
                public: <Test as SigningTypes>::Public::from(public_key),
            };

            let call = pallet_cdd_offchain_worker::Call::submit_unsigned_invalidate_nominators_with_signed_payload(new_payload_with_wrong_data_set, signature.clone());
            assert_err!(CddOffchainWorker::pre_dispatch(&call), InvalidTransaction::BadProof);

            let new_payload_with_zero_length_data_set = Payload {
                block_number: 5,
                nominators: vec![],
                public: <Test as SigningTypes>::Public::from(public_key),
            };

            // directly call the extrinsic with zero length of the data.
            assert_err!(
                CddOffchainWorker::submit_unsigned_invalidate_nominators_with_signed_payload(Origin::none(), new_payload_with_zero_length_data_set, signature),
                CddWorkerError::EmptyTargetList
            );
        }
    });
}

#[test]
#[should_panic = "No `keystore` associated for the current context!"]
fn check_for_local_accounts() {
    const PHRASE: &str =
        "news slush supreme milk chapter athlete soap sausage put clutch what kitten";
    let (offchain, _) = testing::TestOffchainExt::new();
    let (pool, _) = testing::TestTransactionPoolExt::new();

    let mut txn = ExtBuilder::default()
        .nominate(true)
        .cdd_providers(vec![AccountId::from(AccountKeyring::Alice.public())])
        .build();
    txn.register_extension(OffchainExt::new(offchain));
    txn.register_extension(TransactionPoolExt::new(pool));

    txn.execute_with(|| {
        let block = 1;
        System::set_block_number(block);

        let bonding_time = Staking::get_bonding_duration_period();
        let now = (Utc::now()).timestamp() as u64;
        Timestamp::set_timestamp(now * 1000);
        let expiry = now * 1000 + bonding_time * 1000 + 10000_u64; // in MS

        // Add 3 or 4 nominators in the system which has some expired cdd.
        add_nominator(501, 10000, Some(expiry));
        System::set_block_number(block + 1);
        add_nominator(601, 10000, Some(expiry + 1000));
        System::set_block_number(block + 2);

        // Set new timestamp
        Timestamp::set_timestamp(expiry + 2001);

        assert_ok!(CddOffchainWorker::remove_invalidate_nominators(5));
    });
}
