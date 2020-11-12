use super::{
    pips_test::User,
    storage::{provide_scope_claim_to_multiple_parties, root, Checkpoint, TestStorage},
    ExtBuilder,
};
use frame_support::{
    assert_noop, assert_ok,
    dispatch::{DispatchError, DispatchResult},
    StorageDoubleMap, StorageMap,
};
use pallet_asset::checkpoint::ScheduleId;
use pallet_corporate_actions::{
    CACheckpoint, CADetails, CAId, CAIdSequence, CAKind, CorporateAction, LocalCAId,
    RecordDateSpec, TargetIdentities,
    TargetTreatment::{Exclude, Include},
    Tax,
};
use polymesh_primitives::{
    calendar::CheckpointId, AuthorizationData, Document, DocumentId, IdentityId, Moment,
    PortfolioId, Signatory, Ticker, AssetName
};
use sp_arithmetic::Permill;
use std::convert::TryInto;
use test_client::AccountKeyring;

type System = frame_system::Module<TestStorage>;
type Origin = <TestStorage as frame_system::Trait>::Origin;
type Asset = pallet_asset::Module<TestStorage>;
type AssetError = pallet_asset::Error<TestStorage>;
type Timestamp = pallet_timestamp::Module<TestStorage>;
type Identity = pallet_identity::Module<TestStorage>;
type Authorizations = pallet_identity::Authorizations<TestStorage>;
type ComplianceManager = pallet_compliance_manager::Module<TestStorage>;
type CA = pallet_corporate_actions::Module<TestStorage>;
type Error = pallet_corporate_actions::Error<TestStorage>;

const CDDP: AccountKeyring = AccountKeyring::Eve;

const P0: Permill = Permill::zero();
const P25: Permill = Permill::from_percent(25);
const P50: Permill = Permill::from_percent(50);
const P75: Permill = Permill::from_percent(75);

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

fn init_ca(
    owner: User,
    ticker: Ticker,
    kind: CAKind,
    date: Option<Moment>,
    details: String,
    targets: Option<TargetIdentities>,
    default_wht: Option<Tax>,
    wht: Option<Vec<(IdentityId, Tax)>>,
) -> Result<CorporateAction, DispatchError> {
    let id = CA::ca_id_sequence(ticker);
    let sig = owner.signer();
    let details = CADetails(details.as_bytes().to_vec());
    let date = date.map(RecordDateSpec::Scheduled);
    CA::initiate_corporate_action(sig, ticker, kind, date, details, targets, default_wht, wht)?;
    Ok(CA::corporate_actions(ticker, id).unwrap())
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
                // ...`set_did_withholding_tax`,
                $assert!(CA::set_did_withholding_tax(
                    $user.signer(),
                    ticker,
                    other.did,
                    None,
                ) $(, $tail)?);
                // ..., `initiate_corporate_action`,
                $assert!(CA::initiate_corporate_action(
                    $user.signer(),
                    ticker,
                    CAKind::Other,
                    None,
                    <_>::default(),
                    None,
                    None,
                    None,
                ) $(, $tail)?);
                // ..., and `link_ca_doc`.
                let id = CAId {
                    ticker,
                    local_id: LocalCAId(0),
                };
                $assert!(CA::link_ca_doc($user.signer(), id, vec![]) $(, $tail)?);
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
        assert_eq!(CA::default_withholding_tax(ticker), P0);
        assert_ok!(CA::set_default_withholding_tax(owner.signer(), ticker, P50));
        assert_eq!(CA::default_withholding_tax(ticker), P50);
    });
}

#[test]
fn set_did_withholding_tax_works() {
    test(|ticker, [owner, foo, bar]| {
        transfer(&ticker, owner, foo);
        transfer(&ticker, owner, bar);

        let check = |user: User, tax, expect| {
            assert_ok!(CA::set_did_withholding_tax(
                owner.signer(),
                ticker,
                user.did,
                tax
            ));
            assert_eq!(CA::did_withholding_tax(ticker), expect);
        };
        check(foo, Some(P25), vec![(foo.did, P25)]);
        check(bar, Some(P75), vec![(foo.did, P25), (bar.did, P75)]);
        check(foo, Some(P50), vec![(foo.did, P50), (bar.did, P75)]);
        check(foo, None, vec![(bar.did, P75)]);
    });
}

#[test]
fn set_max_details_length_only_root() {
    ExtBuilder::default().build().execute_with(|| {
        let alice = User::new(AccountKeyring::Alice).signer();
        assert_noop!(
            CA::set_max_details_length(alice, 5),
            DispatchError::BadOrigin,
        );
        assert_ok!(CA::set_max_details_length(root(), 10));
        assert_eq!(CA::max_details_length(), 10);
    });
}

#[test]
fn initiate_corporate_action_details() {
    test(|ticker, [owner, ..]| {
        assert_ok!(CA::set_max_details_length(root(), 2));
        let init_ca = |details: &str| -> DispatchResult {
            let ca = init_ca(
                owner,
                ticker,
                CAKind::Other,
                None,
                details.to_owned(),
                None,
                None,
                None,
            )?;
            assert_eq!(details.as_bytes(), ca.details.as_slice());
            Ok(())
        };
        assert_ok!(init_ca("f"));
        assert_ok!(init_ca("fo"));
        assert_noop!(init_ca("foo"), Error::DetailsTooLong);
        assert_noop!(init_ca("❤️"), Error::DetailsTooLong);
    });
}

#[test]
fn initiate_corporate_action_local_id_overflow() {
    test(|ticker, [owner, ..]| {
        CAIdSequence::insert(ticker, LocalCAId(u32::MAX - 2));
        let init_ca = || {
            init_ca(
                owner,
                ticker,
                CAKind::Other,
                None,
                <_>::default(),
                None,
                None,
                None,
            )
        };
        assert_ok!(init_ca()); // -2; OK
        assert_ok!(init_ca()); // -1; OK
        assert_noop!(init_ca(), Error::LocalCAIdOverflow); // 0; Next overflows, so error already.
    });
}

#[test]
fn initiate_corporate_action_record_date() {
    test(|ticker, [owner, foo, _]| {
        assert_ok!(Checkpoint::set_schedules_max_complexity(root(), 1));

        Timestamp::set_timestamp(0);

        let mut cp_id = CheckpointId(0);
        let mut schedule_id = ScheduleId(0);

        let mut check = |date| {
            let ca = init_ca(
                owner,
                ticker,
                CAKind::Other,
                date,
                <_>::default(),
                None,
                None,
                None,
            )
            .unwrap();
            assert_eq!(date, ca.record_date.map(|x| x.date));
            if let (Some(date), Some(rd)) = (date, ca.record_date) {
                cp_id.0 += 1;
                schedule_id.0 += 1;

                assert_eq!(date, rd.date);
                match rd.checkpoint {
                    CACheckpoint::Scheduled(id) => assert_eq!(schedule_id, id),
                    CACheckpoint::Existing(_) => panic!(),
                }

                Timestamp::set_timestamp(date);
                transfer(&ticker, owner, foo);

                assert_eq!(
                    Checkpoint::schedule_points((ticker, schedule_id)),
                    vec![cp_id]
                );
                assert_eq!(date, Checkpoint::timestamps(cp_id));
            }
        };

        check(None);
        check(Some(50_000));
        check(Some(100_000));

        assert_eq!(Checkpoint::checkpoint_id_sequence(ticker), CheckpointId(2));
    });
}

#[test]
fn initiate_corporate_action_kind() {
    test(|ticker, [owner, ..]| {
        for kind in &[
            CAKind::PredictableBenefit,
            CAKind::UnpredictableBenefit,
            CAKind::IssuerNotice,
            CAKind::Reorganization,
            CAKind::Other,
        ] {
            let ca = init_ca(owner, ticker, *kind, None, <_>::default(), None, None, None).unwrap();
            assert_eq!(*kind, ca.kind);
        }
    });
}

fn basic_ca(
    owner: User,
    ticker: Ticker,
    targets: Option<TargetIdentities>,
    default_wht: Option<Tax>,
    wht: Option<Vec<(IdentityId, Tax)>>,
) -> CorporateAction {
    init_ca(
        owner,
        ticker,
        CAKind::Other,
        None,
        <_>::default(),
        targets,
        default_wht,
        wht,
    )
    .unwrap()
}

#[test]
fn initiate_corporate_action_default_tax() {
    test(|ticker, [owner, ..]| {
        let ca = |dwt| basic_ca(owner, ticker, None, dwt, None).default_withholding_tax;
        assert_ok!(CA::set_default_withholding_tax(owner.signer(), ticker, P25));
        assert_eq!(ca(None), P25);
        assert_eq!(ca(Some(P50)), P50);
    });
}

#[test]
fn initiate_corporate_action_did_tax() {
    test(|ticker, [owner, foo, bar]| {
        let ca = |wt| basic_ca(owner, ticker, None, None, wt).withholding_tax;

        let wts = vec![(foo.did, P25), (bar.did, P75)];
        for (did, wt) in wts.iter().copied() {
            assert_ok!(CA::set_did_withholding_tax(
                owner.signer(),
                ticker,
                did,
                Some(wt)
            ));
        }
        assert_eq!(ca(None), wts);

        let wts = vec![(foo.did, P0), (bar.did, P50)];
        assert_eq!(ca(Some(wts.clone())), wts);
    });
}

#[test]
#[should_panic]
fn initiate_corporate_action_did_tax_dupe() {
    test(|ticker, [owner, foo, bar]| {
        let wt = Some(vec![(bar.did, P75), (foo.did, P0), (bar.did, P50)]);
        basic_ca(owner, ticker, None, None, wt);
    });
}

#[test]
fn initiate_corporate_action_targets() {
    test(|ticker, [owner, foo, bar]| {
        let ca = |targets| basic_ca(owner, ticker, targets, None, None).targets;
        let ids = |treatment, identities| TargetIdentities {
            treatment,
            identities,
        };

        let t1 = ids(Include, vec![foo.did]);
        assert_ok!(CA::set_default_targets(owner.signer(), ticker, t1.clone()));
        assert_eq!(ca(None), t1);

        assert_eq!(
            ca(Some(ids(Exclude, vec![bar.did, foo.did, bar.did]))),
            ids(Exclude, vec![foo.did, bar.did]),
        );
    });
}

#[test]
fn link_ca_docs_works() {
    test(|ticker, [owner, ..]| {
        let local_id = LocalCAId(0);
        let id = CAId { ticker, local_id };

        let link = |docs| CA::link_ca_doc(owner.signer(), id, docs);
        let link_ok = |docs: Vec<_>| {
            assert_ok!(link(docs.clone()));
            assert_eq!(CA::ca_doc_link(id), docs);
        };

        // Link to a CA that doesn't exist, and ensure failure.
        assert_noop!(link(vec![]), Error::NoSuchCA);

        // Make it exist, and check that linking to no docs works.
        basic_ca(owner, ticker, None, None, None);
        link_ok(vec![]);

        // Now link it to docs that don't exist, and ensure failure.
        let id0 = DocumentId(0);
        assert_noop!(link(vec![id0]), AssetError::NoSuchDoc);

        // Add the document.
        let doc = Document {
            name: b"foo".into(),
            uri: b"https://example.com".into(),
            content_hash: b"0xdeadbeef".into(),
            doc_type: None,
            filing_date: None,
        };
        assert_ok!(Asset::add_documents(owner.signer(), vec![doc], ticker));

        // The document exists, but we add a second one that does not, so still expecting failure.
        assert_noop!(link(vec![id0, DocumentId(1)]), AssetError::NoSuchDoc);

        // Finally, we only link the document, and it all works out.
        link_ok(vec![id0]);
    });
}
