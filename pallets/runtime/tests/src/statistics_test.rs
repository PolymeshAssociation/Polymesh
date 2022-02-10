use super::{
    storage::{add_cdd_claim, add_investor_uniqueness_claim, TestStorage, User},
    ExtBuilder,
};
use frame_support::{assert_noop, assert_ok};
use pallet_asset as asset;
use pallet_compliance_manager as compliance_manager;
use pallet_statistics::{self as statistics};
use polymesh_primitives::{
    asset::AssetType,
    statistics::{HashablePermill, TransferManager},
    PortfolioId, Ticker,
};
use sp_arithmetic::Permill;
use sp_std::convert::TryFrom;
use test_client::AccountKeyring;

type Origin = <TestStorage as frame_system::Config>::Origin;
type Asset = asset::Module<TestStorage>;
type Statistic = statistics::Module<TestStorage>;
type ComplianceManager = compliance_manager::Module<TestStorage>;
type Error = statistics::Error<TestStorage>;
type AssetError = asset::Error<TestStorage>;

fn create_token(owner: User, disable_iu: bool) -> Ticker {
    let token_name = b"ACME";
    let ticker = Ticker::try_from(&token_name[..]).unwrap();
    assert_ok!(Asset::create_asset(
        owner.origin(),
        token_name.into(),
        ticker,
        true,
        AssetType::default(),
        vec![],
        None,
        disable_iu,
    ));
    assert_ok!(Asset::issue(owner.origin(), ticker, 1_000_000));
    assert_ok!(ComplianceManager::add_compliance_requirement(
        owner.origin(),
        ticker,
        vec![],
        vec![]
    ));
    ticker
}

#[track_caller]
fn do_valid_transfer(ticker: Ticker, from: User, to: User, amount: u128) {
    assert_ok!(Asset::base_transfer(
        PortfolioId::default_portfolio(from.did),
        PortfolioId::default_portfolio(to.did),
        &ticker,
        amount
    ));
}

#[track_caller]
fn ensure_invalid_transfer(ticker: Ticker, from: User, to: User, amount: u128) {
    assert_noop!(
        Asset::base_transfer(
            PortfolioId::default_portfolio(from.did),
            PortfolioId::default_portfolio(to.did),
            &ticker,
            amount
        ),
        AssetError::InvalidTransfer
    );
}

#[test]
fn investor_count() {
    ExtBuilder::default()
        .cdd_providers(vec![AccountKeyring::Eve.to_account_id()])
        .build()
        .execute_with(investor_count_with_ext);
}

fn investor_count_with_ext() {
    let cdd_provider = AccountKeyring::Eve.to_account_id();
    let alice = User::new(AccountKeyring::Alice);
    let bob = User::new(AccountKeyring::Bob);
    let charlie = User::new(AccountKeyring::Charlie);

    // 1. Create an asset with investor uniqueness.
    let ticker = create_token(alice, false);

    // Each user needs an investor uniqueness claim.
    alice.make_scope_claim(ticker, &cdd_provider);
    bob.make_scope_claim(ticker, &cdd_provider);
    charlie.make_scope_claim(ticker, &cdd_provider);

    // Alice sends some tokens to Bob. Token has only one investor.
    do_valid_transfer(ticker, alice, bob, 500);
    assert_eq!(Statistic::investor_count(&ticker), 2);

    // Alice sends some tokens to Charlie. Token has now two investors.
    do_valid_transfer(ticker, alice, charlie, 5000);
    assert_eq!(Statistic::investor_count(&ticker), 3);

    // Bob sends all his tokens to Charlie, so now we have one investor again.
    do_valid_transfer(ticker, bob, charlie, 500);
    assert_eq!(Statistic::investor_count(&ticker), 2);
}

#[test]
fn investor_count_disable_iu() {
    ExtBuilder::default()
        .cdd_providers(vec![AccountKeyring::Eve.to_account_id()])
        .build()
        .execute_with(investor_count_disable_iu_with_ext);
}

fn investor_count_disable_iu_with_ext() {
    let cdd_provider = AccountKeyring::Eve.to_account_id();
    let alice = User::new(AccountKeyring::Alice);
    let bob = User::new(AccountKeyring::Bob);
    let charlie = User::new(AccountKeyring::Charlie);

    // 1. Create an asset with disabled investor uniqueness.
    let ticker = create_token(alice, true);

    // Add CDD claim and create scope_id.
    let (scope_id, cdd_id, proof) =
        add_cdd_claim(alice.did, ticker, alice.uid(), cdd_provider, None);
    // Try adding an investor uniqueness claim.  Should fail.
    assert_noop!(
        add_investor_uniqueness_claim(alice.did, ticker, scope_id, cdd_id, proof),
        AssetError::InvestorUniquenessClaimNotAllowed
    );

    // Alice sends some tokens to Bob. Token has only one investor.
    do_valid_transfer(ticker, alice, bob, 500);
    assert_eq!(Statistic::investor_count(&ticker), 2);

    // Alice sends some tokens to Charlie. Token has now two investors.
    do_valid_transfer(ticker, alice, charlie, 5000);
    assert_eq!(Statistic::investor_count(&ticker), 3);

    // Bob sends all his tokens to Charlie, so now we have one investor again.
    do_valid_transfer(ticker, bob, charlie, 500);
    assert_eq!(Statistic::investor_count(&ticker), 2);
}

#[test]
fn should_add_tm() {
    ExtBuilder::default().build().execute_with(|| {
        let alice = User::new(AccountKeyring::Alice);

        let ticker = create_token(alice, false);

        let tms = (0..3u64)
            .map(TransferManager::CountTransferManager)
            .collect::<Vec<_>>();

        let add_tm = |tm| Statistic::add_transfer_manager(alice.origin(), ticker, tm);
        assert_ok!(add_tm(tms[0].clone()));
        assert_eq!(Statistic::transfer_managers(ticker), [tms[0].clone()]);

        assert_noop!(add_tm(tms[0].clone()), Error::DuplicateTransferManager);

        for tm in &tms[1..3] {
            assert_ok!(add_tm(tm.clone()));
        }

        assert_eq!(Statistic::transfer_managers(ticker), tms);

        assert_noop!(
            add_tm(TransferManager::CountTransferManager(1000000)),
            Error::TransferManagersLimitReached
        );
    });
}

#[test]
fn should_remove_tm() {
    ExtBuilder::default().build().execute_with(|| {
        let alice = User::new(AccountKeyring::Alice);

        let ticker = create_token(alice, false);

        let mut tms = Vec::new();

        for i in 0..3u64 {
            tms.push(TransferManager::CountTransferManager(i));
            assert_ok!(Statistic::add_transfer_manager(
                alice.origin(),
                ticker,
                tms[i as usize].clone()
            ));
            assert_eq!(Statistic::transfer_managers(ticker), tms);
        }

        for _ in 0..3u64 {
            let tm = tms.pop().unwrap();
            assert_ok!(Statistic::remove_transfer_manager(
                alice.origin(),
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
        let alice = User::new(AccountKeyring::Alice);

        let ticker = create_token(alice, false);

        let tm = TransferManager::CountTransferManager(1000000);
        let assert_exemption = |boolean| {
            assert_eq!(
                Statistic::entity_exempt((ticker, tm.clone()), alice.did),
                boolean
            )
        };
        assert_exemption(false);
        assert_ok!(Statistic::add_exempted_entities(
            alice.origin(),
            ticker,
            tm.clone(),
            vec![alice.did]
        ));
        assert_exemption(true);
        assert_ok!(Statistic::remove_exempted_entities(
            alice.origin(),
            ticker,
            tm.clone(),
            vec![alice.did]
        ));
        assert_exemption(false);
    });
}

#[test]
fn should_verify_tms() {
    ExtBuilder::default()
        .cdd_providers(vec![AccountKeyring::Eve.to_account_id()])
        .build()
        .execute_with(|| {
            let cdd_provider = AccountKeyring::Eve.to_account_id();
            let ticker = Ticker::try_from(&b"ACME"[..]).unwrap();
            let setup_account = |keyring: AccountKeyring| {
                let user = User::new(keyring);
                let (scope, _) = user.make_scope_claim(ticker, &cdd_provider);
                (user, scope)
            };
            let (alice, alice_scope) = setup_account(AccountKeyring::Alice);
            let (bob, _) = setup_account(AccountKeyring::Bob);
            let (charlie, charlie_scope) = setup_account(AccountKeyring::Charlie);
            let (dave, dave_scope) = setup_account(AccountKeyring::Dave);

            let ticker = create_token(alice, false);

            assert_eq!(Statistic::investor_count(&ticker), 1);

            // No TM attached, transfer should be valid
            do_valid_transfer(ticker, alice, bob, 10);
            assert_eq!(Statistic::investor_count(&ticker), 2);
            let add_tm = |tm| Statistic::add_transfer_manager(alice.origin(), ticker, tm);
            let add_ctm = |limit| {
                add_tm(TransferManager::CountTransferManager(limit)).expect("Failed to add CTM")
            };
            // Count TM attached with max investors limit reached already
            add_ctm(2);
            // Transfer that increases the investor count beyond the limit should fail
            ensure_invalid_transfer(ticker, bob, charlie, 5);
            // Transfer that keeps the investor count the same should succeed
            do_valid_transfer(ticker, bob, alice, 1);

            let add_exemption = |tm, scopes| {
                assert_ok!(Statistic::add_exempted_entities(
                    alice.origin(),
                    ticker,
                    tm,
                    scopes
                ));
            };

            // Add some folks to the exemption list
            add_exemption(
                TransferManager::CountTransferManager(2),
                vec![alice_scope, dave_scope],
            );
            // Transfer should fail when receiver is exempted but not sender for CTM
            ensure_invalid_transfer(ticker, bob, dave, 5);
            // Transfer should succeed since sender is exempted
            do_valid_transfer(ticker, alice, charlie, 5);

            // Bump CTM to 10
            assert_ok!(Statistic::remove_transfer_manager(
                alice.origin(),
                ticker,
                TransferManager::CountTransferManager(2)
            ));
            add_ctm(10);

            let ptm25 = TransferManager::PercentageTransferManager(HashablePermill(
                Permill::from_rational(1u32, 4u32),
            ));
            // Add ptm with max ownership limit of 50%
            assert_ok!(add_tm(ptm25.clone()));
            // Transfer should fail when receiver is breaching the limit
            ensure_invalid_transfer(ticker, alice, dave, 250_001);
            // Transfer should succeed when under limit
            do_valid_transfer(ticker, alice, dave, 250_000);
            // Transfer should fail when receiver is breaching the limit
            ensure_invalid_transfer(ticker, alice, dave, 1);

            // Add charlie to exemption list
            add_exemption(ptm25, vec![charlie_scope]);
            // Transfer should succeed since receiver is exempted
            do_valid_transfer(ticker, alice, charlie, 250_001);

            // Advanced scenario where charlie is limited at 30% but others at 25%
            assert_ok!(add_tm(TransferManager::PercentageTransferManager(
                HashablePermill(Permill::from_rational(3u32, 10u32))
            )));
            // Transfer should fail when dave is breaching the default limit
            ensure_invalid_transfer(ticker, alice, dave, 1);
            // Transfer should fail when charlie is breaching the advanced limit
            ensure_invalid_transfer(ticker, alice, charlie, 50_000);
            // Transfer should succeed when charlie under advanced limit
            do_valid_transfer(ticker, alice, charlie, 25_000);
        });
}
