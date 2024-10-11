use frame_support::assert_ok;

use polymesh_primitives::asset::AssetId;
use polymesh_primitives::settlement::{VenueDetails, VenueId, VenueType};

use crate::asset_pallet::setup::create_and_issue_sample_asset;
use crate::storage::User;
use crate::TestStorage;

type Asset = pallet_asset::Module<TestStorage>;
type Nft = pallet_nft::Module<TestStorage>;
type Settlement = pallet_settlement::Module<TestStorage>;

/// Calls [`create_and_issue_sample_asset`] and creates a venue for `asset_owner`.
pub fn create_and_issue_sample_asset_with_venue(asset_owner: &User) -> (AssetId, Option<VenueId>) {
    let asset_id = create_and_issue_sample_asset(&asset_owner);

    let venue_id = Settlement::venue_counter();
    assert_ok!(Settlement::create_venue(
        asset_owner.origin(),
        VenueDetails::default(),
        vec![asset_owner.acc()],
        VenueType::Other
    ));

    (asset_id, Some(venue_id))
}
