use sp_std::collections::btree_set::BTreeSet;

use pallet_settlement::VenueCounter;
use polymesh_primitives::asset::AssetId;
use polymesh_primitives::settlement::{InstructionId, Leg};
use polymesh_primitives::settlement::{SettlementType, VenueDetails, VenueId, VenueType};
use polymesh_primitives::{BlockNumber, PortfolioId};

use crate::asset_pallet::setup::create_and_issue_sample_asset;
use crate::storage::User;
use crate::TestStorage;

type Settlement = pallet_settlement::Pallet<TestStorage>;
type Timestamp = pallet_timestamp::Pallet<TestStorage>;

/// Calls [`create_and_issue_sample_asset`] and creates a venue for `asset_owner`.
pub fn create_and_issue_sample_asset_with_venue(asset_owner: &User) -> (AssetId, Option<VenueId>) {
    let asset_id = create_and_issue_sample_asset(&asset_owner);

    let venue_id = VenueCounter::<TestStorage>::get();
    Settlement::create_venue(
        asset_owner.origin(),
        VenueDetails::default(),
        vec![asset_owner.acc()],
        VenueType::Other,
    )
    .unwrap();

    (asset_id, Some(venue_id))
}

/// 1. Creates and issues an asset with a venue;
/// 2. Creates a settlement instruction with the asset;
/// 3. Affirms the instruction;
///
/// `Note:` The instruction transfers 1_000 tokens from the sender's default portfolio to the receiver's default portfolio.
pub fn add_and_affirm_simple_instruction(
    sender: User,
    receiver: User,
    mediator: User,
    settlement_type: SettlementType<BlockNumber>,
) -> AssetId {
    let (asset_id, venue_id) = create_and_issue_sample_asset_with_venue(&sender);

    let rcv_default_portfolio = PortfolioId::default_portfolio(receiver.did);
    let sender_default_portfolio = PortfolioId::default_portfolio(sender.did);

    let legs = vec![Leg::Fungible {
        sender: sender_default_portfolio,
        receiver: rcv_default_portfolio,
        asset_id,
        amount: 1_000,
    }];

    Settlement::add_instruction_with_mediators(
        sender.origin(),
        venue_id,
        settlement_type,
        None,
        None,
        legs.clone(),
        None,
        BTreeSet::from([mediator.did]).try_into().unwrap(),
    )
    .unwrap();

    Settlement::affirm_instruction(
        receiver.origin(),
        InstructionId(0),
        BTreeSet::from([rcv_default_portfolio]).try_into().unwrap(),
    )
    .unwrap();

    Settlement::affirm_instruction(
        sender.origin(),
        InstructionId(0),
        BTreeSet::from([sender_default_portfolio])
            .try_into()
            .unwrap(),
    )
    .unwrap();

    Settlement::affirm_instruction_as_mediator(
        mediator.origin(),
        InstructionId(0),
        Some(Timestamp::get() + 1),
    )
    .unwrap();

    asset_id
}
