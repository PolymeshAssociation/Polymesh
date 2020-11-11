use super::{
    pips_test::User,
    storage::{provide_scope_claim_to_multiple_parties, root, Balance, Checkpoint, TestStorage},
    ExtBuilder,
};
use core::iter;
use frame_support::{
    assert_noop, assert_ok,
    dispatch::{DispatchError, DispatchResult},
    IterableStorageDoubleMap, StorageDoubleMap, StorageMap,
};
use pallet_asset::checkpoint::{ScheduleId, StoredSchedule};
use pallet_corporate_actions::{
    ballot::{self, BallotMeta, BallotTimeRange, BallotVote, Motion},
    distribution::{self, Distribution},
    CACheckpoint, CADetails, CAId, CAIdSequence, CAKind, CorporateAction, LocalCAId, RecordDate,
    RecordDateSpec, TargetIdentities,
    TargetTreatment::{Exclude, Include},
    Tax,
};
use polymesh_common_utilities::asset::AssetName;
use polymesh_primitives::{
    calendar::{CheckpointId, CheckpointSchedule},
    AuthorizationData, Document, DocumentId, IdentityId, Moment, PortfolioId, Signatory, Ticker,
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
type Ballot = ballot::Module<TestStorage>;
type Dist = distribution::Module<TestStorage>;
type Error = pallet_corporate_actions::Error<TestStorage>;
type BallotError = ballot::Error<TestStorage>;
type DistError = distribution::Error<TestStorage>;
type CPError = pallet_asset::checkpoint::Error<TestStorage>;
type Votes = ballot::Votes<TestStorage>;

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

type CAResult = Result<CorporateAction, DispatchError>;

fn init_ca(
    owner: User,
    ticker: Ticker,
    kind: CAKind,
    date: Option<RecordDateSpec>,
    details: String,
    targets: Option<TargetIdentities>,
    default_wht: Option<Tax>,
    wht: Option<Vec<(IdentityId, Tax)>>,
) -> CAResult {
    let id = CA::ca_id_sequence(ticker);
    let sig = owner.signer();
    let details = CADetails(details.as_bytes().to_vec());
    CA::initiate_corporate_action(sig, ticker, kind, date, details, targets, default_wht, wht)?;
    Ok(CA::corporate_actions(ticker, id).unwrap())
}

fn basic_ca(
    owner: User,
    ticker: Ticker,
    targets: Option<TargetIdentities>,
    default_wht: Option<Tax>,
    wht: Option<Vec<(IdentityId, Tax)>>,
) -> CAResult {
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
}

fn dated_ca(owner: User, ticker: Ticker, kind: CAKind, rd: Option<RecordDateSpec>) -> CAResult {
    init_ca(owner, ticker, kind, rd, <_>::default(), None, None, None)
}

fn moment_ca(owner: User, ticker: Ticker, kind: CAKind, rd: Option<Moment>) -> CAResult {
    dated_ca(owner, ticker, kind, rd.map(RecordDateSpec::Scheduled))
}

fn set_schedule_complexity() {
    Timestamp::set_timestamp(0);
    assert_ok!(Checkpoint::set_schedules_max_complexity(root(), 1000));
}

fn next_ca_id(ticker: Ticker) -> CAId {
    let local_id = CA::ca_id_sequence(ticker);
    CAId { ticker, local_id }
}

const TRANGE: BallotTimeRange = BallotTimeRange {
    start: 3000,
    end: 4000,
};

#[derive(Clone, Eq, PartialEq, Default, Debug)]
struct BallotData {
    meta: Option<BallotMeta>,
    range: Option<BallotTimeRange>,
    choices: Vec<u16>,
    rcv: bool,
    results: Vec<Balance>,
    votes: Vec<(IdentityId, Vec<BallotVote<Balance>>)>,
}

fn ballot_data(id: CAId) -> BallotData {
    BallotData {
        meta: Ballot::metas(id),
        range: Ballot::time_ranges(id),
        choices: Ballot::motion_choices(id),
        rcv: Ballot::rcv(id),
        results: Ballot::results(id),
        votes: Votes::iter_prefix(id).collect(),
    }
}

fn assert_ballot(id: CAId, data: &BallotData) {
    assert_eq!(&ballot_data(id), data);
}

#[test]
fn only_caa_authorized() {
    test(|ticker, [owner, caa, other]| {
        set_schedule_complexity();

        // Transfer some to Charlie & Bob.
        transfer(&ticker, owner, caa);
        transfer(&ticker, owner, other);

        let currency = create_asset(b"BETA", owner);

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
                let record_date = Some(RecordDateSpec::Scheduled(2000));
                let mk_ca = |kind| dated_ca($user, ticker, kind, record_date);
                let id = next_ca_id(ticker);
                $assert!(mk_ca(CAKind::IssuerNotice) $(, $tail)?);
                // ..., `link_ca_doc`,
                $assert!(CA::link_ca_doc($user.signer(), id, vec![]) $(, $tail)?);
                // ..., `change_record_date`,
                $assert!(CA::change_record_date($user.signer(), id, record_date) $(, $tail)?);
                // ..., `attach_ballot`,
                let meta = BallotMeta::default();
                $assert!(Ballot::attach_ballot($user.signer(), id, TRANGE, meta.clone(), false) $(, $tail)?);
                // ..., `change_end`,
                $assert!(Ballot::change_end($user.signer(), id, 5000) $(, $tail)?);
                // ..., `change_meta`,
                $assert!(Ballot::change_meta($user.signer(), id, meta) $(, $tail)?);
                // ..., `change_rcv`,
                $assert!(Ballot::change_rcv($user.signer(), id, true) $(, $tail)?);
                // ..., `remove_ballot`,
                $assert!(Ballot::remove_ballot($user.signer(), id) $(, $tail)?);
                // ..., `remove_ca`,
                $assert!(CA::remove_ca($user.signer(), id) $(, $tail)?);
                // ..., `distribute`,
                let id = next_ca_id(ticker);
                $assert!(mk_ca(CAKind::UnpredictableBenefit) $(, $tail)?);
                $assert!(Dist::distribute($user.signer(), id, None, currency, 0, 3000, None) $(, $tail)?);
                // ..., and `remove_distribution`.
                $assert!(Dist::remove_distribution($user.signer(), id) $(, $tail)?);
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
        let init_ca = || dated_ca(owner, ticker, CAKind::Other, None);
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
            let ca = moment_ca(owner, ticker, CAKind::Other, date).unwrap();
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

const ALL_CA_KINDS: &[CAKind] = &[
    CAKind::PredictableBenefit,
    CAKind::UnpredictableBenefit,
    CAKind::IssuerNotice,
    CAKind::Reorganization,
    CAKind::Other,
];

#[test]
fn initiate_corporate_action_kind() {
    test(|ticker, [owner, ..]| {
        for kind in ALL_CA_KINDS {
            assert_eq!(*kind, dated_ca(owner, ticker, *kind, None).unwrap().kind);
        }
    });
}

#[test]
fn initiate_corporate_action_default_tax() {
    test(|ticker, [owner, ..]| {
        let ca = |dwt| {
            basic_ca(owner, ticker, None, dwt, None)
                .unwrap()
                .default_withholding_tax
        };
        assert_ok!(CA::set_default_withholding_tax(owner.signer(), ticker, P25));
        assert_eq!(ca(None), P25);
        assert_eq!(ca(Some(P50)), P50);
    });
}

#[test]
fn initiate_corporate_action_did_tax() {
    test(|ticker, [owner, foo, bar]| {
        let ca = |wt| {
            basic_ca(owner, ticker, None, None, wt)
                .unwrap()
                .withholding_tax
        };

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
        basic_ca(owner, ticker, None, None, wt).unwrap();
    });
}

#[test]
fn initiate_corporate_action_targets() {
    test(|ticker, [owner, foo, bar]| {
        let ca = |targets| {
            basic_ca(owner, ticker, targets, None, None)
                .unwrap()
                .targets
        };
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

fn add_doc(owner: User, ticker: Ticker) {
    let doc = Document {
        name: b"foo".into(),
        uri: b"https://example.com".into(),
        content_hash: b"0xdeadbeef".into(),
        doc_type: None,
        filing_date: None,
    };
    assert_ok!(Asset::add_documents(owner.signer(), vec![doc], ticker));
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
        basic_ca(owner, ticker, None, None, None).unwrap();
        link_ok(vec![]);

        // Now link it to docs that don't exist, and ensure failure.
        let id0 = DocumentId(0);
        assert_noop!(link(vec![id0]), AssetError::NoSuchDoc);

        // Add the document.
        add_doc(owner, ticker);

        // The document exists, but we add a second one that does not, so still expecting failure.
        assert_noop!(link(vec![id0, DocumentId(1)]), AssetError::NoSuchDoc);

        // Finally, we only link the document, and it all works out.
        link_ok(vec![id0]);
    });
}

#[test]
fn remove_ca_works() {
    test(|ticker, [owner, ..]| {
        set_schedule_complexity();

        let ca = |kind, rd| moment_ca(owner, ticker, kind, rd).unwrap();
        let remove = |id| CA::remove_ca(owner.signer(), id);

        let assert_no_ca = |id: CAId| {
            assert_eq!(None, CA::corporate_actions(ticker, id.local_id));
            assert_eq!(CA::ca_doc_link(id), vec![]);
        };

        // Remove a CA that doesn't exist, and ensure failure.
        let id = next_ca_id(ticker);
        assert_noop!(remove(id), Error::NoSuchCA);

        // Create a CA, remove it, and ensure its no longer there.
        ca(CAKind::Other, None);
        add_doc(owner, ticker);
        let docs = vec![DocumentId(0)];
        assert_ok!(CA::link_ca_doc(owner.signer(), id, docs.clone()));
        assert_eq!(docs, CA::ca_doc_link(id));
        assert_ok!(remove(id));
        assert_no_ca(id);

        // Create a ballot CA, which hasn't started.
        let time = BallotTimeRange {
            start: 3000,
            end: 4000,
        };
        let motion = Motion {
            title: "".into(),
            info_link: "".into(),
            choices: vec!["".into()],
        };
        let meta = BallotMeta {
            title: vec![].into(),
            motions: vec![motion],
        };
        let mk_ballot = || {
            Timestamp::set_timestamp(0);
            let id = next_ca_id(ticker);
            ca(CAKind::IssuerNotice, Some(1000));
            assert_ballot(id, &<_>::default());
            assert_ok!(Ballot::attach_ballot(
                owner.signer(),
                id,
                time,
                meta.clone(),
                true,
            ));
            id
        };
        let id = mk_ballot();
        // Ensure the details are right.
        assert_ballot(
            id,
            &BallotData {
                meta: Some(meta.clone()),
                range: Some(time),
                choices: vec![1u16],
                rcv: true,
                ..<_>::default()
            },
        );
        // Sucessfully remove it. Edge condition `now == start - 1`.
        Timestamp::set_timestamp(3000 - 1);
        assert_ok!(remove(id));
        // And ensure all details were removed.
        assert_no_ca(id);
        assert_ballot(id, &<_>::default());

        // Create another ballot, move now => start date; try to remove, but fail.
        let id = mk_ballot();
        Timestamp::set_timestamp(3000); // now == start
        assert_noop!(remove(id), BallotError::VotingAlreadyStarted);
        Timestamp::set_timestamp(3001); // now == start + 1
        assert_noop!(remove(id), BallotError::VotingAlreadyStarted);

        // Create a distribution CA, which hasn't started.
        let currency = create_asset(b"BETA", owner);
        let mk_dist = || {
            Timestamp::set_timestamp(0);
            let id = next_ca_id(ticker);
            ca(CAKind::UnpredictableBenefit, Some(1000));
            assert_ok!(Dist::distribute(
                owner.signer(),
                id,
                None,
                currency,
                0,
                3000,
                None,
            ));
            id
        };
        let id = mk_dist();
        // Ensure the details are right.
        assert_eq!(
            Dist::distributions(id),
            Some(Distribution {
                from: PortfolioId::default_portfolio(owner.did),
                currency,
                amount: 0,
                remaining: 0,
                reclaimed: false,
                payment_at: 3000,
                expires_at: None,
            }),
        );
        // Sucessfully remove it. Edge condition `now == start - 1`.
        Timestamp::set_timestamp(3000 - 1);
        assert_ok!(remove(id));
        // And ensure all details were removed.
        assert_no_ca(id);
        assert_eq!(Dist::distributions(id), None);
    });
}

fn next_schedule_id(ticker: Ticker) -> ScheduleId {
    let ScheduleId(id) = Checkpoint::schedule_id_sequence(ticker);
    ScheduleId(id + 1)
}

#[test]
fn change_record_date_works() {
    test(|ticker, [owner, ..]| {
        set_schedule_complexity();

        let ca = |kind, rd| moment_ca(owner, ticker, kind, rd).unwrap();
        let change = |id, date| CA::change_record_date(owner.signer(), id, date);
        let change_ok = |id, date, expect| {
            assert_ok!(change(id, date));
            assert_eq!(
                expect,
                CA::corporate_actions(id.ticker, id.local_id)
                    .unwrap()
                    .record_date
            );
        };

        // Change for a CA that doesn't exist, and ensure failure.
        let id = next_ca_id(ticker);
        assert_noop!(change(id, None), Error::NoSuchCA);

        let spec_ts = |ts| Some(RecordDateSpec::Scheduled(ts));
        let spec_cp = |id| Some(RecordDateSpec::Existing(CheckpointId(id)));
        let spec_sh = |id| Some(RecordDateSpec::ExistingSchedule(id));
        let rd_cp = |date, id| {
            let checkpoint = CACheckpoint::Existing(CheckpointId(id));
            Some(RecordDate { date, checkpoint })
        };
        let rd_ts = |date, id| {
            let checkpoint = CACheckpoint::Scheduled(id);
            Some(RecordDate { date, checkpoint })
        };

        // Trigger `NoSuchCheckpointId`.
        ca(CAKind::Other, None);
        assert_noop!(change(id, spec_cp(42)), Error::NoSuchCheckpointId);

        // Successfully use a checkpoint which exists.
        assert_ok!(Checkpoint::create_checkpoint(owner.signer(), ticker));
        change_ok(id, spec_cp(1), rd_cp(0, 1));

        // Trigger `NoSuchSchedule`.
        assert_noop!(change(id, spec_sh(ScheduleId(42))), CPError::NoSuchSchedule);

        // Successfully use a schedule which exists.
        let sh_id = next_schedule_id(ticker);
        change_ok(id, spec_ts(1000), rd_ts(1000, sh_id));
        assert_eq!(Checkpoint::schedule_id_sequence(ticker), sh_id);
        assert!(!Checkpoint::schedule_removable((ticker, sh_id)));
        let mk_schedule = |at, id| {
            let period = <_>::default();
            let schedule = CheckpointSchedule { start: at, period };
            StoredSchedule { at, id, schedule }
        };
        assert_eq!(
            Checkpoint::schedules(ticker),
            vec![mk_schedule(1000, sh_id)]
        );
        change_ok(id, spec_sh(sh_id), rd_ts(1000, sh_id));

        // Use a removable schedule. Should fail.
        let sh_id2 = next_schedule_id(ticker);
        assert_ok!(Checkpoint::create_schedule(
            owner.signer(),
            ticker,
            2000.into()
        ));
        assert_eq!(Checkpoint::schedule_id_sequence(ticker), sh_id2);
        assert!(Checkpoint::schedule_removable((ticker, sh_id2)));
        assert_eq!(
            Checkpoint::schedules(ticker),
            vec![mk_schedule(1000, sh_id), mk_schedule(2000, sh_id2)]
        );
        assert_noop!(
            change(id, spec_sh(sh_id2)),
            Error::ExistingScheduleRemovable
        );

        // No need to test `RecordDateSpec::Scheduled` branch beyond what we have here.
        // To do so would replicate tests in the checkpoint module.

        // Test ballot branch.
        let id = next_ca_id(ticker);
        ca(CAKind::IssuerNotice, Some(1000));
        let time = BallotTimeRange {
            start: 5000,
            end: 7000,
        };
        let meta = BallotMeta::default();
        assert_ok!(Ballot::attach_ballot(owner.signer(), id, time, meta, true));
        let test_branch = |id, error: DispatchError| {
            let change_ok = |spec, expect| {
                change_ok(
                    id,
                    dbg!(spec_ts(spec)),
                    dbg!(rd_ts(expect, dbg!(next_schedule_id(ticker)))),
                )
            };
            Timestamp::set_timestamp(3000);
            change_ok(4999, 4000); // floor(4999 / 1000) * 1000 == 4000
            Timestamp::set_timestamp(4999);
            change_ok(4999, 4999); // Flooring not applied cause now == 2999.
            change_ok(5000, 5000); // floor(5000 / 1000) * 1000 == 5000
            change_ok(5001, 5000); // floor(5001 / 1000) * 1000 == 5000
            Timestamp::set_timestamp(5001);
            assert_noop!(change(id, spec_ts(5001)), Error::RecordDateAfterStart); // 5001 < 5000
            assert_noop!(change(id, spec_ts(6000)), Error::RecordDateAfterStart); // 6000 < 5000
            Timestamp::set_timestamp(6000);
            assert_noop!(change(id, spec_cp(1)), error); // 6000 < 4000
            Timestamp::set_timestamp(6001);
            assert_noop!(change(id, spec_cp(1)), error); // 6001 < 4000
        };
        test_branch(id, BallotError::VotingAlreadyStarted.into());

        // Test distribution branch.
        Timestamp::set_timestamp(0);
        let id = next_ca_id(ticker);
        ca(CAKind::PredictableBenefit, Some(1000));
        assert_ok!(Dist::distribute(
            owner.signer(),
            id,
            None,
            create_asset(b"BETA", owner),
            0,
            5000,
            None,
        ));
        test_branch(id, DistError::DistributionStarted.into());
    });
}

#[test]
fn attach_ballot_no_such_ca() {
    test(|ticker, [owner, ..]| {
        let id = next_ca_id(ticker);
        assert_noop!(
            Ballot::attach_ballot(owner.signer(), id, TRANGE, <_>::default(), true),
            Error::NoSuchCA
        );
    });
}

#[test]
fn attach_ballot_only_notice() {
    test(|ticker, [owner, ..]| {
        set_schedule_complexity();
        let attach = |id| Ballot::attach_ballot(owner.signer(), id, TRANGE, <_>::default(), true);
        for &kind in ALL_CA_KINDS {
            let id = next_ca_id(ticker);
            assert_ok!(moment_ca(owner, ticker, kind, Some(1000)));
            if let CAKind::IssuerNotice = kind {
                assert_ok!(attach(id));
            } else {
                assert_noop!(attach(id), BallotError::CANotNotice);
            }
        }
    });
}

fn notice_ca(owner: User, ticker: Ticker, rd: Option<Moment>) -> Result<CAId, DispatchError> {
    let id = next_ca_id(ticker);
    moment_ca(owner, ticker, CAKind::IssuerNotice, rd)?;
    Ok(id)
}

#[test]
fn attach_ballot_range_invariant() {
    test(|ticker, [owner, ..]| {
        set_schedule_complexity();

        let id = notice_ca(owner, ticker, Some(1000)).unwrap();

        let mut data = BallotData {
            rcv: true,
            meta: Some(<_>::default()),
            ..BallotData::default()
        };

        let mut attach = |id, time| -> DispatchResult {
            data.range = Some(time);
            let meta = data.meta.clone().unwrap();
            Ballot::attach_ballot(owner.signer(), id, time, meta, data.rcv)?;
            assert_ballot(id, &data);
            Ok(())
        };
        let range = |start| BallotTimeRange { start, end: 6000 };

        assert_noop!(attach(id, range(6001)), BallotError::StartAfterEnd);

        Timestamp::set_timestamp(6001);
        assert_noop!(attach(id, range(6000)), BallotError::NowAfterEnd);

        Timestamp::set_timestamp(4000);
        assert_ok!(attach(id, range(6000)));

        let id = notice_ca(owner, ticker, Some(5000)).unwrap();
        assert_noop!(attach(id, range(4999)), Error::RecordDateAfterStart);
        assert_ok!(attach(id, range(5000)));

        let id = notice_ca(owner, ticker, None).unwrap();
        assert_noop!(attach(id, range(6000)), Error::NoRecordDate);
    });
}

#[test]
fn attach_ballot_already_exists() {
    test(|ticker, [owner, ..]| {
        set_schedule_complexity();

        let id = notice_ca(owner, ticker, Some(1000)).unwrap();

        let attach = |id| Ballot::attach_ballot(owner.signer(), id, TRANGE, <_>::default(), true);

        assert_ok!(attach(id));
        assert_noop!(attach(id), BallotError::AlreadyExists);
        assert_ok!(Ballot::remove_ballot(owner.signer(), id));
        assert_ok!(attach(id));
    });
}

fn overflowing_meta() -> BallotMeta {
    BallotMeta {
        title: "".into(),
        motions: vec![Motion {
            title: "".into(),
            info_link: "".into(),
            choices: iter::repeat("".into())
                // `u16::MAX` doesn't overflow, but +1 does.
                .take(1 + u16::MAX as usize)
                .collect(),
        }],
    }
}

#[test]
fn attach_ballot_num_choices_overflow_u16() {
    test(|ticker, [owner, ..]| {
        set_schedule_complexity();

        // N.B. we do not test the total-choices-overflows-usize case since
        // that actually requires allocating an `usize` + 1 number of choices,
        // which is not reasonable as a test.

        let id = notice_ca(owner, ticker, Some(1000)).unwrap();
        assert_noop!(
            Ballot::attach_ballot(owner.signer(), id, TRANGE, overflowing_meta(), false),
            BallotError::NumberOfChoicesOverflow,
        );
    });
}

fn mk_meta() -> BallotMeta {
    let motion_a = Motion {
        title: "foo".into(),
        info_link: "www.acme.com".into(),
        choices: vec!["foo".into(), "bar".into(), "baz".into()],
    };
    let motion_b = Motion {
        title: "bar".into(),
        info_link: "www.emca.com".into(),
        choices: vec!["foo".into()],
    };
    BallotMeta {
        title: vec![].into(),
        motions: vec![motion_a, motion_b],
    }
}

#[test]
fn attach_ballot_works() {
    test(|ticker, [owner, ..]| {
        set_schedule_complexity();

        let data = BallotData {
            meta: Some(mk_meta()),
            range: Some(TRANGE),
            choices: vec![3, 1],
            ..<_>::default()
        };

        let id = notice_ca(owner, ticker, Some(1000)).unwrap();
        assert_ok!(Ballot::attach_ballot(
            owner.signer(),
            id,
            TRANGE,
            mk_meta(),
            false
        ));
        assert_ballot(id, &data);
    });
}

#[test]
fn change_end_works() {
    test(|ticker, [owner, ..]| {
        set_schedule_complexity();

        assert_noop!(
            Ballot::change_end(owner.signer(), next_ca_id(ticker), 0),
            BallotError::NoSuchBallot,
        );

        let range = BallotTimeRange {
            start: 2000,
            end: 4000,
        };

        let mut data = BallotData {
            range: Some(range),
            meta: Some(<_>::default()),
            ..<_>::default()
        };

        let id = notice_ca(owner, ticker, Some(1000)).unwrap();
        assert_ok!(Ballot::attach_ballot(
            owner.signer(),
            id,
            range,
            <_>::default(),
            false
        ));
        assert_ballot(id, &data);

        let mut change = |end| -> DispatchResult {
            Ballot::change_end(owner.signer(), id, end)?;
            data.range = Some(BallotTimeRange { end, ..range });
            assert_ballot(id, &data);
            Ok(())
        };

        Timestamp::set_timestamp(1999);
        assert_ok!(change(5000)); // Not started yet, OK.
        assert_ok!(change(2000)); // start == end, OK.
        assert_noop!(change(1999), BallotError::StartAfterEnd); // end is before start; bad!
        Timestamp::set_timestamp(2000);
        assert_noop!(change(5000), BallotError::VotingAlreadyStarted);
    });
}

#[test]
fn change_rcv_works() {
    test(|ticker, [owner, ..]| {
        for &rcv in &[true, false] {
            set_schedule_complexity();

            let id = notice_ca(owner, ticker, Some(1000)).unwrap();
            let change = |rcv| Ballot::change_rcv(owner.signer(), id, rcv);
            assert_noop!(change(rcv), BallotError::NoSuchBallot);
            assert_ballot(id, &<_>::default());

            let range = BallotTimeRange {
                start: 3000,
                end: 5000,
            };
            let mut data = BallotData {
                range: Some(range),
                meta: Some(<_>::default()),
                rcv,
                ..<_>::default()
            };

            assert_ok!(Ballot::attach_ballot(
                owner.signer(),
                id,
                range,
                <_>::default(),
                data.rcv
            ));
            assert_ballot(id, &data);

            Timestamp::set_timestamp(2999);
            data.rcv ^= true;
            assert_ok!(change(data.rcv));
            assert_ballot(id, &data);

            Timestamp::set_timestamp(3000);
            assert_noop!(change(!data.rcv), BallotError::VotingAlreadyStarted);
            assert_ballot(id, &data);
        }
    });
}

#[test]
fn change_meta_works() {
    test(|ticker, [owner, ..]| {
        set_schedule_complexity();

        let id = notice_ca(owner, ticker, Some(1000)).unwrap();
        let change = |meta| Ballot::change_meta(owner.signer(), id, meta);

        assert_noop!(change(<_>::default()), BallotError::NoSuchBallot);

        let range = BallotTimeRange {
            start: 4000,
            end: 6000,
        };
        let mut data = BallotData {
            range: Some(range),
            meta: Some(<_>::default()),
            ..<_>::default()
        };

        assert_ok!(Ballot::attach_ballot(
            owner.signer(),
            id,
            range,
            <_>::default(),
            data.rcv,
        ));
        assert_ballot(id, &data);

        Timestamp::set_timestamp(3999);
        assert_ok!(change(mk_meta()));
        data.meta = Some(mk_meta());
        data.choices = vec![3, 1];
        assert_ballot(id, &data);

        assert_noop!(
            change(overflowing_meta()),
            BallotError::NumberOfChoicesOverflow,
        );

        Timestamp::set_timestamp(4000);
        assert_noop!(change(mk_meta()), BallotError::VotingAlreadyStarted);
        assert_ballot(id, &data);
    });
}
