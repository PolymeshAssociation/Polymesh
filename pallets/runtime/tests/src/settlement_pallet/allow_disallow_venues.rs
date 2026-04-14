use frame_support::{assert_noop, assert_ok};
use sp_keyring::Sr25519Keyring;
use sp_std::collections::btree_set::BTreeSet;

use pallet_settlement::{Error, VenueCounter};
use polymesh_primitives::settlement::{VenueDetails, VenueId, VenueType};

use crate::asset_pallet::setup::create_and_issue_sample_asset;
use crate::storage::User;
use crate::{ExtBuilder, TestStorage};

type Settlement = pallet_settlement::Pallet<TestStorage>;

#[test]
fn block_allowing_non_existing_venues() {
    ExtBuilder::default().build().execute_with(|| {
        let alice = User::new(Sr25519Keyring::Alice);

        let asset_id = create_and_issue_sample_asset(&alice);
        assert_eq!(VenueCounter::<TestStorage>::get(), VenueId(0));

        assert_noop!(
            Settlement::allow_venues(alice.origin(), asset_id, vec![VenueId(0)]),
            Error::<TestStorage>::InvalidVenue
        );
        assert_noop!(
            Settlement::allow_venues(alice.origin(), asset_id, vec![VenueId(1)]),
            Error::<TestStorage>::InvalidVenue
        );

        assert_ok!(Settlement::create_venue(
            alice.origin(),
            VenueDetails::default(),
            BTreeSet::from([alice.acc()]),
            VenueType::default()
        ));
        assert_eq!(VenueCounter::<TestStorage>::get(), VenueId(1));

        assert_noop!(
            Settlement::allow_venues(alice.origin(), asset_id, vec![VenueId(1)]),
            Error::<TestStorage>::InvalidVenue
        );
        assert_ok!(Settlement::allow_venues(
            alice.origin(),
            asset_id,
            vec![VenueId(0)]
        ),);
    });
}

#[test]
fn block_disallowing_non_existing_venues() {
    ExtBuilder::default().build().execute_with(|| {
        let alice = User::new(Sr25519Keyring::Alice);

        let asset_id = create_and_issue_sample_asset(&alice);
        assert_eq!(VenueCounter::<TestStorage>::get(), VenueId(0));

        assert_noop!(
            Settlement::disallow_venues(alice.origin(), asset_id, vec![VenueId(0)]),
            Error::<TestStorage>::InvalidVenue
        );
        assert_noop!(
            Settlement::disallow_venues(alice.origin(), asset_id, vec![VenueId(1)]),
            Error::<TestStorage>::InvalidVenue
        );

        assert_ok!(Settlement::create_venue(
            alice.origin(),
            VenueDetails::default(),
            BTreeSet::from([alice.acc()]),
            VenueType::default()
        ));
        assert_eq!(VenueCounter::<TestStorage>::get(), VenueId(1));

        assert_noop!(
            Settlement::disallow_venues(alice.origin(), asset_id, vec![VenueId(1)]),
            Error::<TestStorage>::InvalidVenue
        );
        assert_ok!(Settlement::disallow_venues(
            alice.origin(),
            asset_id,
            vec![VenueId(0)]
        ),);
    });
}
