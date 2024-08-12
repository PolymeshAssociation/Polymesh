use frame_support::{assert_noop, assert_ok};
use frame_support::{StorageDoubleMap, StorageMap, StorageValue};
use rand::Rng;
use sp_keyring::AccountKeyring;
use sp_std::collections::btree_set::BTreeSet;

use pallet_asset::{
    TickerConfig, TickerRegistration, TickersOwnedByUser, UniqueTickerRegistration,
};
use polymesh_primitives::asset::AssetType;
use polymesh_primitives::ticker::TICKER_LEN;
use polymesh_primitives::Ticker;

use crate::asset_test::{now, set_timestamp};
use crate::storage::User;
use crate::{ExtBuilder, TestStorage};

type Asset = pallet_asset::Module<TestStorage>;
type AssetError = pallet_asset::Error<TestStorage>;

#[test]
fn verify_ticker_characters() {
    let mut all_valid_characters = Vec::new();
    let mut valid_ascii_digits: Vec<u8> = (48..58).collect();
    let mut valid_ascii_letters: Vec<u8> = (65..91).collect();
    let mut valid_special_characters: Vec<u8> = Vec::from([b'-', b'.', b'/', b'_']);
    all_valid_characters.append(&mut valid_ascii_digits);
    all_valid_characters.append(&mut valid_ascii_letters);
    all_valid_characters.append(&mut valid_special_characters);

    let mut rng = rand::thread_rng();

    // Generates 10 random valid tickers
    for _ in 0..10 {
        let valid_ticker: Vec<u8> = (0..TICKER_LEN + 1)
            .map(|_| all_valid_characters[rng.gen_range(0, all_valid_characters.len())])
            .collect();
        assert_ok!(Asset::verify_ticker_characters(
            &Ticker::from_slice_truncated(&valid_ticker)
        ));
    }

    let valid_set: BTreeSet<&u8> = all_valid_characters.iter().collect();
    let mut all_invalid_characters: Vec<u8> = (1..=255).collect();
    all_invalid_characters.retain(|ascii_code| !valid_set.contains(ascii_code));

    // Generates 10 random invalid tickers
    for _ in 0..10 {
        let mut invalid_ticker: Vec<u8> = (0..TICKER_LEN - 1)
            .map(|_| all_valid_characters[rng.gen_range(0, all_valid_characters.len())])
            .collect();
        invalid_ticker.push(all_invalid_characters[rng.gen_range(0, all_invalid_characters.len())]);

        assert_eq!(
            Asset::verify_ticker_characters(&Ticker::from_slice_truncated(&invalid_ticker))
                .unwrap_err(),
            AssetError::InvalidTickerCharacter.into()
        );
    }

    assert_eq!(
        Asset::verify_ticker_characters(&Ticker::from_slice_truncated(&[0; TICKER_LEN]))
            .unwrap_err(),
        AssetError::TickerFirstByteNotValid.into()
    );
    assert_eq!(
        Asset::verify_ticker_characters(&Ticker::from_slice_truncated(&[b'A', 0, 0, 0, b'A']))
            .unwrap_err(),
        AssetError::InvalidTickerCharacter.into()
    );
}

#[test]
fn register_ticker() {
    ExtBuilder::default().build().execute_with(|| {
        let ticker: Ticker = Ticker::from_slice_truncated(b"TICKER");
        let alice = User::new(AccountKeyring::Alice);

        assert_ok!(Asset::register_unique_ticker(alice.origin(), ticker,));

        let ticker_registration_config = TickerConfig::<TestStorage>::get();
        assert_eq!(
            UniqueTickerRegistration::<TestStorage>::get(&ticker).unwrap(),
            TickerRegistration {
                owner: alice.did,
                expiry: ticker_registration_config.registration_length
            }
        );
        assert_eq!(TickersOwnedByUser::get(&alice.did, &ticker), true);
    });
}

/// This only makes sure register_ticker calls verify_ticker_characters.
/// The ticker validity tests are done in the `verify_ticker_characters` unit test.
#[test]
fn register_ticker_invalid_ticker_character() {
    ExtBuilder::default().build().execute_with(|| {
        let ticker: Ticker = Ticker::from_slice_truncated(b"TICKER+");
        let alice = User::new(AccountKeyring::Alice);

        assert_noop!(
            Asset::register_unique_ticker(alice.origin(), ticker,),
            AssetError::InvalidTickerCharacter
        );
    });
}

#[test]
fn register_ticker_already_linked() {
    ExtBuilder::default().build().execute_with(|| {
        let ticker: Ticker = Ticker::from_slice_truncated(b"TICKER");
        let alice = User::new(AccountKeyring::Alice);
        let asset_id = Asset::generate_asset_id(alice.acc(), false);

        assert_ok!(Asset::register_unique_ticker(alice.origin(), ticker,));

        assert_ok!(Asset::create_asset(
            alice.origin(),
            b"MyAsset".into(),
            true,
            AssetType::default(),
            Vec::new(),
            None,
        ));

        assert_ok!(Asset::link_ticker_to_asset_id(
            alice.origin(),
            ticker,
            asset_id
        ));

        assert_noop!(
            Asset::register_unique_ticker(alice.origin(), ticker,),
            AssetError::TickerIsAlreadyLinkedToAnAsset
        );
    });
}

#[test]
fn register_ticker_too_long() {
    ExtBuilder::default().build().execute_with(|| {
        let max_ticker_len = TickerConfig::<TestStorage>::get().max_ticker_length;
        let ticker_bytes: Vec<u8> = (65..65 + max_ticker_len + 1).collect();
        let ticker: Ticker = Ticker::from_slice_truncated(&ticker_bytes);
        let alice = User::new(AccountKeyring::Alice);

        assert_noop!(
            Asset::register_unique_ticker(alice.origin(), ticker,),
            AssetError::TickerTooLong
        );
    });
}

#[test]
fn register_ticker_already_expired() {
    ExtBuilder::default().build().execute_with(|| {
        let ticker: Ticker = Ticker::from_slice_truncated(b"TICKER00");
        let bob = User::new(AccountKeyring::Bob);
        let alice = User::new(AccountKeyring::Alice);

        assert_ok!(Asset::register_unique_ticker(alice.origin(), ticker,));
        assert_noop!(
            Asset::register_unique_ticker(bob.origin(), ticker,),
            AssetError::TickerAlreadyRegistered
        );
    });
}

#[test]
fn register_already_expired_ticker() {
    ExtBuilder::default().build().execute_with(|| {
        let ticker: Ticker = Ticker::from_slice_truncated(b"TICKER00");
        let bob = User::new(AccountKeyring::Bob);
        let alice = User::new(AccountKeyring::Alice);

        assert_ok!(Asset::register_unique_ticker(alice.origin(), ticker,));
        set_timestamp(now() + 10001);
        assert_ok!(Asset::register_unique_ticker(bob.origin(), ticker,));

        assert_eq!(
            UniqueTickerRegistration::<TestStorage>::get(&ticker)
                .unwrap()
                .owner,
            bob.did,
        );
        assert_eq!(TickersOwnedByUser::get(&alice.did, &ticker), false);
        assert_eq!(TickersOwnedByUser::get(&bob.did, &ticker), true);
    });
}

#[test]
fn register_ticker_renewal() {
    ExtBuilder::default().build().execute_with(|| {
        let ticker: Ticker = Ticker::from_slice_truncated(b"TICKER00");
        let alice = User::new(AccountKeyring::Alice);

        assert_ok!(Asset::register_unique_ticker(alice.origin(), ticker,));
        let ticker_registration = UniqueTickerRegistration::<TestStorage>::get(&ticker).unwrap();
        set_timestamp(10);
        assert_ok!(Asset::register_unique_ticker(alice.origin(), ticker,));
        let new_registration = UniqueTickerRegistration::<TestStorage>::get(&ticker).unwrap();
        assert_eq!(ticker_registration.owner, new_registration.owner);
        assert_eq!(
            ticker_registration.expiry.unwrap() + 10,
            new_registration.expiry.unwrap()
        );
    });
}
