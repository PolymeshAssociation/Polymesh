use frame_support::{assert_noop, assert_ok};
use frame_support::{StorageDoubleMap, StorageMap, StorageValue};
use sp_keyring::AccountKeyring;

use pallet_asset::Tickers;
use pallet_portfolio::{PortfolioAssetBalances, PortfolioLockedAssets};
use polymesh_primitives::asset::AssetType;
use polymesh_primitives::settlement::{
    InstructionId, Leg, SettlementType, VenueDetails, VenueId, VenueType,
};
use polymesh_primitives::{PortfolioId, PortfolioKind, PortfolioName, PortfolioNumber, Ticker};

use crate::storage::User;
use crate::{ExtBuilder, TestStorage};

type Asset = pallet_asset::Module<TestStorage>;
type AssetError = pallet_asset::Error<TestStorage>;
type Identity = pallet_identity::Module<TestStorage>;
type Portfolio = pallet_portfolio::Module<TestStorage>;
type Settlement = pallet_settlement::Module<TestStorage>;

#[test]
fn controller_transfer_locked_asset() {
    ExtBuilder::default().build().execute_with(|| {
        let bob = User::new(AccountKeyring::Bob);
        let alice = User::new(AccountKeyring::Alice);
        let ticker: Ticker = Ticker::from_slice_truncated(b"TICKER");
        let bob_default_portfolio = PortfolioId {
            did: bob.did,
            kind: PortfolioKind::Default,
        };
        let alice_user_portfolio = PortfolioId {
            did: alice.did,
            kind: PortfolioKind::User(PortfolioNumber(1)),
        };

        // Creates an asset an issue 1_000 tokens
        assert_ok!(Portfolio::create_portfolio(
            alice.origin(),
            PortfolioName(b"AliceUserPortfolio".to_vec())
        ));
        assert_ok!(Asset::create_asset(
            alice.origin(),
            ticker.as_ref().into(),
            ticker,
            true,
            AssetType::default(),
            Vec::new(),
            None,
        ));
        assert_ok!(Asset::issue(
            alice.origin(),
            ticker,
            1_000,
            PortfolioKind::User(PortfolioNumber(1))
        ));
        // Locks the asset by creating a settlement and affirming it
        assert_ok!(Settlement::create_venue(
            alice.origin(),
            VenueDetails::default(),
            vec![alice.acc()],
            VenueType::Other
        ));
        assert_ok!(Settlement::add_instruction(
            alice.origin(),
            VenueId(0),
            SettlementType::SettleOnAffirmation,
            None,
            None,
            vec![Leg::Fungible {
                sender: alice_user_portfolio,
                receiver: bob_default_portfolio,
                ticker: ticker,
                amount: 1_000,
            }],
            None,
        ));
        assert_ok!(Settlement::affirm_instruction(
            alice.origin(),
            InstructionId(0),
            vec![alice_user_portfolio]
        ),);
        // Assert balance and locked value
        assert_eq!(
            PortfolioAssetBalances::get(&alice_user_portfolio, &ticker),
            1_000
        );
        assert_eq!(
            PortfolioLockedAssets::get(&alice_user_portfolio, &ticker),
            1_000
        );
    });
}
