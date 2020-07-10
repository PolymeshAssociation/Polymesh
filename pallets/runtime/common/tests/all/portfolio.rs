use super::{
    storage::{register_keyring_account, TestStorage},
    ExtBuilder,
};
use frame_support::{assert_err, assert_ok};
use polymesh_primitives::{PortfolioId, PortfolioName};
use test_client::AccountKeyring;

type Error = pallet_portfolio::Error<TestStorage>;
type Origin = <TestStorage as frame_system::Trait>::Origin;
type Portfolio = pallet_portfolio::Module<TestStorage>;

#[test]
fn can_create_and_delete_portfolio() {
    ExtBuilder::default().build().execute_with(|| {
        let alice = AccountKeyring::Alice.public();
        let alice_id = register_keyring_account(AccountKeyring::Alice).unwrap();
        let name = PortfolioName::from([42u8].to_vec());
        let num = Portfolio::next_portfolio_number();
        assert_ok!(Portfolio::create_portfolio(
            Origin::signed(alice),
            name.clone()
        ));
        assert_eq!(Portfolio::portfolios(&alice_id, num), Some((num, name)));
        assert_ok!(Portfolio::delete_portfolio(Origin::signed(alice), num));
        assert!(Portfolio::portfolios(&alice_id, num).is_none());
    });
}
