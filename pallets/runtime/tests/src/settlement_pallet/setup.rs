use frame_support::assert_ok;
use sp_std::collections::btree_set::BTreeSet;

use pallet_settlement::VenueCounter;
use polymesh_primitives::asset::AssetId;
use polymesh_primitives::settlement::{InstructionId, Leg};
use polymesh_primitives::settlement::{SettlementType, VenueDetails, VenueId, VenueType};
use polymesh_primitives::{BlockNumber, NFTId, NFTs, PortfolioId, WeightMeter};

use crate::asset_pallet::setup::create_and_issue_sample_asset;
use crate::asset_pallet::setup::create_and_issue_sample_nft;
use crate::storage::User;
use crate::TestStorage;

type Asset = pallet_asset::Pallet<TestStorage>;
type Nft = pallet_nft::Pallet<TestStorage>;
type Settlement = pallet_settlement::Pallet<TestStorage>;
type Timestamp = pallet_timestamp::Pallet<TestStorage>;

/// Calls [`create_and_issue_sample_asset`] and creates a venue for `asset_owner`.
pub fn create_and_issue_sample_asset_with_venue(asset_owner: &User) -> (AssetId, Option<VenueId>) {
    let asset_id = create_and_issue_sample_asset(&asset_owner);

    let venue_id = VenueCounter::<TestStorage>::get();
    assert_ok!(Settlement::create_venue(
        asset_owner.origin(),
        VenueDetails::default(),
        BTreeSet::from([asset_owner.acc()]),
        VenueType::Other,
    ));

    (asset_id, Some(venue_id))
}

/// 1. The mediator creates and issues one fungible and one non-fungible asset;
/// 2. The mediator transfers the assets to the sender;
/// 2. The mediator creates a settlement instruction with the assets;
/// 3. All parties affirm the instruction;
///
/// `Note:` The instruction transfers 1_000 tokens and 1 NFT from the sender's default portfolio to the receiver's default portfolio.
pub fn add_and_affirm_simple_instruction(
    sender: User,
    receiver: User,
    mediator: User,
    settlement_type: SettlementType<BlockNumber>,
) -> (AssetId, AssetId) {
    let med_default_portfolio = PortfolioId::default_portfolio(mediator.did);
    let rcv_default_portfolio = PortfolioId::default_portfolio(receiver.did);
    let sender_default_portfolio = PortfolioId::default_portfolio(sender.did);

    // The mediator creates both assets to allow controller transfer tests
    let (asset_id, venue_id) = create_and_issue_sample_asset_with_venue(&mediator);
    let nft_asset_id = create_and_issue_sample_nft(&mediator);
    Asset::simplified_fungible_transfer(
        asset_id,
        med_default_portfolio.clone().into(),
        sender_default_portfolio.clone().into(),
        1_000,
        InstructionId(0),
        None,
        mediator.did,
        &mut WeightMeter::max_limit_no_minimum(),
    )
    .unwrap();
    Nft::simplified_nft_transfer(
        med_default_portfolio.clone().into(),
        sender_default_portfolio.clone().into(),
        NFTs::new_unverified(nft_asset_id, vec![NFTId(1)]),
        Some(InstructionId(0)),
        None,
        mediator.did,
    )
    .unwrap();

    let legs = vec![
        Leg::Fungible {
            sender: sender_default_portfolio.clone().into(),
            receiver: rcv_default_portfolio.clone().into(),
            asset_id,
            amount: 1_000,
        },
        Leg::NonFungible {
            sender: sender_default_portfolio.clone().into(),
            receiver: rcv_default_portfolio.clone().into(),
            nfts: NFTs::new_unverified(nft_asset_id, vec![NFTId(1)]),
        },
    ];
    Settlement::add_instruction_with_mediators(
        mediator.origin(),
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
        sender.origin(),
        InstructionId(0),
        BTreeSet::from([sender_default_portfolio.clone().into()])
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

    (asset_id, nft_asset_id)
}
