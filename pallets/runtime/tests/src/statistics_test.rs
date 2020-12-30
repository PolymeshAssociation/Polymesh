use super::{
    storage::{make_account, register_keyring_account, TestStorage},
    ExtBuilder,
};
use frame_support::{assert_noop, assert_ok};
use pallet_asset::{self as asset, SecurityToken};
use pallet_compliance_manager as compliance_manager;
use pallet_statistics::{self as statistics, TransferManager};
use polymesh_primitives::{PortfolioId, Ticker};
use sp_std::convert::TryFrom;
use test_client::AccountKeyring;
use sp_core::sr25519::Public;
use polymesh_common_utilities::asset::AssetType;

type Origin = <TestStorage as frame_system::Trait>::Origin;
type Asset = asset::Module<TestStorage>;
type Statistic = statistics::Module<TestStorage>;
type ComplianceManager = compliance_manager::Module<TestStorage>;
type Error = statistics::Error<TestStorage>;

fn create_token(token_name: &[u8], ticker: Ticker, keyring: Public) {
    assert_ok!(Asset::create_asset(
        Origin::signed(keyring),
        token_name.into(),
        ticker,
        100_000,
        true,
        AssetType::default(),
        vec![],
        None,
    ));
    assert_ok!(ComplianceManager::add_compliance_requirement(
        Origin::signed(keyring),
        ticker,
        vec![],
        vec![]
    ));
}

#[test]
fn investor_count() {
    ExtBuilder::default()
        .build()
        .execute_with(|| investor_count_with_ext);
}

fn investor_count_with_ext() {
    let alice_did = register_keyring_account(AccountKeyring::Alice).unwrap();
    let alice_signed = Origin::signed(AccountKeyring::Alice.public());
    let bob_did = register_keyring_account(AccountKeyring::Bob).unwrap();
    let charlie_did = register_keyring_account(AccountKeyring::Charlie).unwrap();

    // 1. Alice create an asset.
    let token = SecurityToken {
        name: vec![0x01].into(),
        owner_did: alice_did,
        total_supply: 1_000_000,
        divisible: true,
        ..Default::default()
    };

    let identifiers = Vec::new();
    let ticker = Ticker::try_from(token.name.as_slice()).unwrap();
    assert_ok!(Asset::create_asset(
        alice_signed.clone(),
        token.name.clone(),
        ticker,
        1_000_000, // Total supply over the limit
        true,
        token.asset_type.clone(),
        identifiers.clone(),
        None,
    ));

    let ticker = Ticker::try_from(token.name.as_slice()).unwrap();
    assert_ok!(ComplianceManager::add_compliance_requirement(
        alice_signed.clone(),
        ticker,
        vec![],
        vec![]
    ));

    let unsafe_transfer = |from, to, value| {
        assert_ok!(Asset::unsafe_transfer(
            PortfolioId::default_portfolio(from),
            PortfolioId::default_portfolio(to),
            &ticker,
            value,
        ));
    };

    // Alice sends some tokens to Bob. Token has only one investor.
    unsafe_transfer(alice_did, bob_did, 500);
    assert_eq!(Statistic::investor_count(&ticker), 1);

    // Alice sends some tokens to Charlie. Token has now two investors.
    unsafe_transfer(alice_did, charlie_did, 5000);
    assert_eq!(Statistic::investor_count(&ticker), 2);

    // Bob sends all his tokens to Charlie, so now we have one investor again.
    unsafe_transfer(bob_did, charlie_did, 500);
    assert_eq!(Statistic::investor_count(&ticker), 1);
}

#[test]
fn should_add_tm() {
    ExtBuilder::default().build().execute_with(|| {
        let (token_owner_signed, _token_owner_did) = make_account(AccountKeyring::Alice.public()).unwrap();

        let token_name = b"ACME";
        let ticker = Ticker::try_from(&token_name[..]).unwrap();
        create_token(token_name, ticker, AccountKeyring::Alice.public());

        let mut tms = Vec::new();

        for i in 0..3u64 {
            tms.push(TransferManager::CountTransferManager(i));
        }

        assert_ok!(Statistic::add_transfer_manager(
            token_owner_signed.clone(),
            ticker,
            tms[0].clone()
        ));
        assert_eq!(Statistic::transfer_managers(ticker), [tms[0].clone()]);

        assert_noop!(
            Statistic::add_transfer_manager(token_owner_signed.clone(), ticker, tms[0].clone()),
            Error::DuplicateTransferManager
        );

        for i in 1..3u64 {
            assert_ok!(Statistic::add_transfer_manager(
                token_owner_signed.clone(),
                ticker,
                tms[i as usize].clone()
            ));
        }

        assert_eq!(Statistic::transfer_managers(ticker), tms);

        assert_noop!(
            Statistic::add_transfer_manager(
                token_owner_signed.clone(),
                ticker,
                TransferManager::CountTransferManager(1000000)
            ),
            Error::TransferManagersLimitReached
        );
    });
}

#[test]
fn should_remove_tm() {
    ExtBuilder::default().build().execute_with(|| {
        let (token_owner_signed, _token_owner_did) = make_account(AccountKeyring::Alice.public()).unwrap();

        let token_name = b"ACME";
        let ticker = Ticker::try_from(&token_name[..]).unwrap();
        create_token(token_name, ticker, AccountKeyring::Alice.public());

        let mut tms = Vec::new();

        for i in 0..3u64 {
            tms.push(TransferManager::CountTransferManager(i));
            assert_ok!(Statistic::add_transfer_manager(
                token_owner_signed.clone(),
                ticker,
                tms[i as usize].clone()
            ));
            assert_eq!(Statistic::transfer_managers(ticker), tms);
        }

        for _ in 0..3u64 {
            let tm = tms.pop().unwrap();
            assert_ok!(Statistic::remove_transfer_manager(
                token_owner_signed.clone(),
                ticker,
                tm
            ));
            assert_eq!(Statistic::transfer_managers(ticker), tms);
        }
    });
}

#[test]
fn should_add_remove_exempted_entities() {
    ExtBuilder::default().build().execute_with(|| {
        let (token_owner_signed, token_owner_did) = make_account(AccountKeyring::Alice.public()).unwrap();

        let token_name = b"ACME";
        let ticker = Ticker::try_from(&token_name[..]).unwrap();
        create_token(token_name, ticker, AccountKeyring::Alice.public());

        let tm = TransferManager::CountTransferManager(1000000);
        let assert_exemption = |boolean| assert_eq!(Statistic::entity_exempt((ticker, tm.clone()), token_owner_did), boolean);
        assert_exemption(false);
        assert_ok!(Statistic::add_exempted_entities(
            token_owner_signed.clone(),
            ticker,
            tm.clone(),
            vec![token_owner_did]
        ));
        assert_exemption(true);
        assert_ok!(Statistic::remove_exempted_entities(
            token_owner_signed.clone(),
            ticker,
            tm.clone(),
            vec![token_owner_did]
        ));
        assert_exemption(false);
    });
}
