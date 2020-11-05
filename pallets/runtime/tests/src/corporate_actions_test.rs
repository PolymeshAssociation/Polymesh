use super::{
    pips_test::User,
    storage::{provide_scope_claim_to_multiple_parties, TestStorage},
    ExtBuilder,
};
use frame_support::{assert_noop, assert_ok, dispatch::DispatchResult, StorageDoubleMap};
use pallet_corporate_actions::{
    TargetIdentities,
    TargetTreatment::{Exclude, Include},
};
use polymesh_common_utilities::traits::asset::AssetName;
use polymesh_primitives::{AuthorizationData, PortfolioId, Signatory, Ticker};
use sp_arithmetic::Permill;
use std::convert::TryInto;
use test_client::AccountKeyring;

type System = frame_system::Module<TestStorage>;
type Origin = <TestStorage as frame_system::Trait>::Origin;
type Asset = pallet_asset::Module<TestStorage>;
type AssetError = pallet_asset::Error<TestStorage>;
type Identity = pallet_identity::Module<TestStorage>;
type Authorizations = pallet_identity::Authorizations<TestStorage>;
type ComplianceManager = pallet_compliance_manager::Module<TestStorage>;
type CA = pallet_corporate_actions::Module<TestStorage>;
type Error = pallet_corporate_actions::Error<TestStorage>;

const CDDP: AccountKeyring = AccountKeyring::Eve;

#[track_caller]
fn test(logic: impl FnOnce(Ticker, [User; 3])) {
    ExtBuilder::default()
        .cdd_providers(vec![CDDP.public()])
        .build()
        .execute_with(|| {
            System::set_block_number(1);

            // Create some users.
            let alice = User::new(AccountKeyring::Alice);
            let bob = User::new(AccountKeyring::Bob);
            let charlie = User::new(AccountKeyring::Charlie);

            // Create the asset.
            let ticker = create_asset(b"ACME", alice);

            // Execute the test.
            logic(ticker, [alice, bob, charlie])
        });
}

fn transfer(ticker: &Ticker, from: User, to: User) {
    // Provide scope claim to sender and receiver of the transaction.
    provide_scope_claim_to_multiple_parties(&[from.did, to.did], *ticker, CDDP.public());
    assert_ok!(Asset::base_transfer(
        PortfolioId::default_portfolio(from.did),
        PortfolioId::default_portfolio(to.did),
        ticker,
        500
    ));
}

fn create_asset(ticker: &[u8], owner: User) -> Ticker {
    let asset_name: AssetName = ticker.into();
    let ticker = ticker.try_into().unwrap();

    // Create the asset.
    assert_ok!(Asset::create_asset(
        owner.signer(),
        asset_name,
        ticker,
        1_000_000,
        true,
        <_>::default(),
        vec![],
        None
    ));

    assert_eq!(Asset::token_details(ticker).owner_did, owner.did);

    // Allow all transfers
    assert_ok!(ComplianceManager::add_compliance_requirement(
        owner.signer(),
        ticker,
        vec![],
        vec![]
    ));

    ticker
}

fn add_caa_auth(ticker: Ticker, from: User, to: User) -> u64 {
    let sig: Signatory<_> = to.did.into();
    let data = AuthorizationData::TransferCorporateActionAgent(ticker);
    assert_ok!(Identity::add_authorization(from.signer(), sig, data, None));
    Authorizations::iter_prefix_values(sig)
        .next()
        .unwrap()
        .auth_id
}

fn transfer_caa(ticker: Ticker, from: User, to: User) -> DispatchResult {
    let auth_id = add_caa_auth(ticker, from, to);
    Identity::accept_authorization(to.signer(), auth_id)
}

#[test]
fn only_caa_authorized() {
    test(|ticker, [owner, caa, other]| {
        // Transfer some to Charlie & Bob.
        transfer(&ticker, owner, caa);
        transfer(&ticker, owner, other);

        macro_rules! checks {
            ($user:expr, $assert:ident $(, $tail:expr)?) => {
                // Check for `set_default_targets`, ...
                let owner_set_targets = |treatment| {
                    let ids = TargetIdentities { treatment, identities: vec![] };
                    CA::set_default_targets($user.signer(), ticker, ids)
                };
                $assert!(owner_set_targets(Include) $(, $tail)?);
                $assert!(owner_set_targets(Exclude) $(, $tail)?);
                // ...`set_default_withholding_tax`,
                $assert!(CA::set_default_withholding_tax(
                    $user.signer(),
                    ticker,
                    Permill::zero(),
                ) $(, $tail)?);
                // ..., and `set_did_withholding_tax`.
                $assert!(CA::set_did_withholding_tax(
                    $user.signer(),
                    ticker,
                    other.did,
                    None,
                ) $(, $tail)?);
            };
        }
        // Ensures passing for owner, but not to-be-CAA (Bob) and other.
        let owner_can_do_it = || {
            checks!(owner, assert_ok);
            checks!(caa, assert_noop, Error::UnauthorizedAsAgent);
            checks!(other, assert_noop, Error::UnauthorizedAsAgent);
        };
        // Ensures passing for Bob (the CAA), not owner, and not other.
        let caa_can_do_it = || {
            checks!(caa, assert_ok);
            checks!(owner, assert_noop, Error::UnauthorizedAsAgent);
            checks!(other, assert_noop, Error::UnauthorizedAsAgent);
        };
        let transfer_caa = |caa| {
            assert_ok!(transfer_caa(ticker, owner, caa));
        };

        // We start with owner being CAA.
        owner_can_do_it();
        // Transfer CAA to Bob.
        transfer_caa(caa);
        caa_can_do_it();
        // Demonstrate that CAA can be transferred back.
        transfer_caa(owner);
        owner_can_do_it();
        // Transfer to Bob again.
        transfer_caa(caa);
        caa_can_do_it();
        // Finally reset; ensuring that CAA is owner.
        assert_ok!(CA::reset_caa(owner.signer(), ticker));
        owner_can_do_it();
    });
}

#[test]
fn only_owner_reset() {
    test(|ticker, [owner, caa, other]| {
        assert_ok!(transfer_caa(ticker, owner, caa));
        let reset = |caller: User| CA::reset_caa(caller.signer(), ticker);
        assert_ok!(reset(owner));
        assert_noop!(reset(caa), AssetError::Unauthorized);
        assert_noop!(reset(other), AssetError::Unauthorized);
    });
}

#[test]
fn only_owner_caa_invite() {
    test(|ticker, [_, caa, other]| {
        let auth_id = add_caa_auth(ticker, other, caa);
        assert_noop!(
            Identity::accept_authorization(caa.signer(), auth_id),
            "Illegal use of Authorization"
        );
    });
}

#[test]
fn not_holder_works() {
    test(|ticker, [owner, _, other]| {
        assert_ok!(CA::set_did_withholding_tax(
            owner.signer(),
            ticker,
            other.did,
            None
        ));

        assert_ok!(CA::set_default_targets(
            owner.signer(),
            ticker,
            TargetIdentities {
                treatment: Exclude,
                identities: vec![other.did],
            }
        ));
    });
}

#[test]
fn set_default_targets_works() {
    test(|ticker, [owner, foo, bar]| {
        transfer(&ticker, owner, foo);
        transfer(&ticker, owner, bar);

        let set = |treatment, identities, expect_ids| {
            let ids = TargetIdentities {
                treatment,
                identities,
            };
            assert_ok!(CA::set_default_targets(owner.signer(), ticker, ids));
            let ids = TargetIdentities {
                treatment,
                identities: expect_ids,
            };
            assert_eq!(CA::default_target_identities(ticker), ids);
        };
        let expect = vec![foo.did, bar.did];
        set(Exclude, expect.clone(), expect.clone());
        set(Exclude, vec![bar.did, foo.did], expect.clone());
        set(Include, vec![foo.did, bar.did, foo.did], expect);
    });
}

#[test]
fn set_default_withholding_tax_works() {
    test(|ticker, [owner, ..]| {
        let tax = Permill::from_percent(50);
        assert_ok!(CA::set_default_withholding_tax(owner.signer(), ticker, tax));
        assert_eq!(CA::default_withholding_tax(ticker), tax);
    });
}

#[test]
fn set_did_withholding_tax_works() {
    test(|ticker, [owner, foo, bar]| {
        transfer(&ticker, owner, foo);
        transfer(&ticker, owner, bar);

        let check = |user: User, tax| {
            assert_ok!(CA::set_did_withholding_tax(
                owner.signer(),
                ticker,
                user.did,
                tax
            ));
            assert_eq!(CA::did_withholding_tax(ticker, user.did), tax);
        };
        check(foo, Some(Permill::from_percent(25)));
        check(bar, Some(Permill::from_percent(75)));
        check(foo, None);
    });
}
