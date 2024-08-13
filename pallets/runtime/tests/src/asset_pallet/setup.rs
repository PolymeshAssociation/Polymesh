use frame_support::assert_ok;

use polymesh_primitives::asset::{AssetID, AssetType, FundingRoundName, NonFungibleType};
use polymesh_primitives::{AssetIdentifier, Balance, PortfolioKind, Ticker};

use crate::storage::User;
use crate::TestStorage;

type Asset = pallet_asset::Module<TestStorage>;
type Nft = pallet_nft::Module<TestStorage>;

/// The amount of tokens that will be issued for the sample asset.
pub const ISSUE_AMOUNT: Balance = 1_000_000_000;

/// Registers a unique ticker
pub fn register_unique_ticker(ticker_owner: &User, ticker: Ticker) {
    assert_ok!(Asset::register_unique_ticker(ticker_owner.origin(), ticker));
}

/// Creates a divisible asset where all values for its attributes are set to their default values.
/// The [`SecurityToken::total_supply`] will be set to [`ISSUE_AMOUNT`].
pub fn create_and_issue_sample_asset(asset_owner: &User) -> AssetID {
    let asset_id = Asset::generate_asset_id(asset_owner.acc(), false);

    assert_ok!(Asset::create_asset(
        asset_owner.origin(),
        b"MyAsset".into(),
        true,
        AssetType::default(),
        Vec::new(),
        None,
    ));

    assert_ok!(Asset::issue(
        asset_owner.origin(),
        asset_id,
        ISSUE_AMOUNT,
        PortfolioKind::Default
    ));

    asset_id
}

/// Calls [`register_unique_ticker`], [`create_and_issue_sample_asset`] and links `ticker` to the asset identifiers.
pub fn create_and_issue_sample_asset_linked_to_ticker(
    asset_owner: &User,
    ticker: Ticker,
) -> AssetID {
    register_unique_ticker(asset_owner, ticker);
    let asset_id = create_and_issue_sample_asset(asset_owner);

    assert_ok!(Asset::link_ticker_to_asset_id(
        asset_owner.origin(),
        ticker,
        asset_id
    ));

    asset_id
}

/// Creates an NFT collection and mints one token. All values for its attributes are set to their default values.
pub fn create_and_issue_sample_nft(asset_owner: &User) -> AssetID {
    let asset_id = Asset::generate_asset_id(asset_owner.acc(), false);

    assert_ok!(Asset::create_asset(
        asset_owner.origin(),
        b"MyNFTAsset".into(),
        false,
        AssetType::NonFungible(NonFungibleType::Derivative),
        Vec::new(),
        None,
    ));

    assert_ok!(Nft::create_nft_collection(
        asset_owner.origin(),
        Some(asset_id),
        None,
        Vec::new().into(),
    ));

    assert_ok!(Nft::issue_nft(
        asset_owner.origin(),
        asset_id,
        Vec::new(),
        PortfolioKind::Default
    ));

    asset_id
}

/// Creates an asset setting the attributes for the [`SecurityToken`] using the values in the parameters.
/// If `issue_tokens`` is `true` also mints [`ISSUE_AMOUNT`] tokens in the `issue_portfolio`.
pub fn create_asset(
    asset_owner: &User,
    asset_name: Option<&[u8]>,
    divisible: Option<bool>,
    asset_type: Option<AssetType>,
    asset_identifiers: Option<Vec<AssetIdentifier>>,
    funding_round_name: Option<FundingRoundName>,
    issue_tokens: bool,
    issue_portfolio: Option<PortfolioKind>,
) -> AssetID {
    let asset_id = Asset::generate_asset_id(asset_owner.acc(), false);

    assert_ok!(Asset::create_asset(
        asset_owner.origin(),
        asset_name.unwrap_or(b"MyAsset").into(),
        divisible.unwrap_or(true),
        asset_type.unwrap_or_default(),
        asset_identifiers.unwrap_or_default(),
        funding_round_name,
    ));

    if issue_tokens {
        assert_ok!(Asset::issue(
            asset_owner.origin(),
            asset_id,
            ISSUE_AMOUNT,
            issue_portfolio.unwrap_or_default()
        ));
    }

    asset_id
}
