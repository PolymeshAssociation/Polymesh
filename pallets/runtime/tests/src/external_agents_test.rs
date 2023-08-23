use crate::asset_test::{a_token, an_asset, basic_asset};
use crate::ext_builder::ExtBuilder;
use crate::identity_test::test_with_bad_ext_perms;
use crate::storage::{TestStorage, User};
use frame_support::dispatch::DispatchResult;
use frame_support::{
    assert_noop, assert_ok, IterableStorageDoubleMap, StorageDoubleMap, StorageMap,
};
use pallet_external_agents::{AGIdSequence, AgentOf, GroupOfAgent, NumFullAgents};
use pallet_permissions::StoreCallMetadata;
use polymesh_common_utilities::constants::currency::POLY;
use polymesh_primitives::{
    agent::{AGId, AgentGroup},
    AuthorizationData, ExtrinsicPermissions, PalletPermissions, Signatory, SubsetRestriction,
    Ticker,
};
use sp_keyring::AccountKeyring;

type ExternalAgents = pallet_external_agents::Module<TestStorage>;
type BaseError = pallet_base::Error<TestStorage>;
type Error = pallet_external_agents::Error<TestStorage>;
type Id = pallet_identity::Module<TestStorage>;

fn set_extrinsic(name: &str) {
    StoreCallMetadata::<TestStorage>::set_call_metadata(
        b"pallet_external_agent".into(),
        name.into(),
    );
}

fn make_perms(pallet: &str) -> ExtrinsicPermissions {
    SubsetRestriction::elem(PalletPermissions::entire_pallet(pallet.into()))
}

fn add_become_agent(
    ticker: Ticker,
    from: User,
    to: User,
    group: AgentGroup,
    expected: DispatchResult,
) {
    let data = AuthorizationData::BecomeAgent(ticker, group);
    let sig = Signatory::Identity(to.did);
    let auth = Id::add_auth(from.did, sig, data, None);
    match expected {
        Ok(_) => {
            assert_ok!(ExternalAgents::accept_become_agent(to.origin(), auth));
        }
        Err(e) => {
            assert_eq!(
                ExternalAgents::accept_become_agent(to.origin(), auth),
                Err(e.into())
            );
        }
    };
}

#[test]
fn create_group_set_perms_works() {
    ExtBuilder::default().build().execute_with(|| {
        let owner = User::new(AccountKeyring::Alice);
        let (ticker, token) = a_token(owner.did);

        let create = |perms| ExternalAgents::create_group(owner.origin(), ticker, perms);
        let set =
            |id, perms| ExternalAgents::set_group_permissions(owner.origin(), ticker, id, perms);

        // No asset made, so no agents, so the "owner" is unauthorized now.
        assert_noop!(create(<_>::default()), Error::UnauthorizedAgent);
        assert_noop!(set(AGId(0), <_>::default()), Error::UnauthorizedAgent);

        // Make the asset. Let's test permissions length limits.
        assert_ok!(basic_asset(owner, ticker, &token));
        test_with_bad_ext_perms(|perms| {
            assert_too_long!(create(perms.clone()));
            assert_too_long!(set(AGId(0), perms));
        });

        // Still, `other` doesn't have agent permissions.
        let other = User::new(AccountKeyring::Bob);
        let other_create = |perms| ExternalAgents::create_group(other.origin(), ticker, perms);
        let other_set =
            |id, perms| ExternalAgents::set_group_permissions(other.origin(), ticker, id, perms);
        assert_noop!(other_create(<_>::default()), Error::UnauthorizedAgent);
        assert_noop!(other_set(AGId(1), <_>::default()), Error::UnauthorizedAgent);

        // Try setting perms for groups that don't exist.
        for g in 0..3 {
            assert_noop!(set(AGId(g), <_>::default()), Error::NoSuchAG);
        }

        // Manipulate storage so that ID will overflow.
        AGIdSequence::insert(ticker, AGId(u32::MAX));
        assert_noop!(create(<_>::default()), BaseError::CounterOverflow);
        AGIdSequence::insert(ticker, AGId::default());

        // Add a group successfully.
        let perms = make_perms("foo");
        assert_ok!(create(perms.clone()));
        assert_eq!(Some(perms), ExternalAgents::permissions(ticker, AGId(1)));
        assert_eq!(AGId(1), ExternalAgents::agent_group_id_sequence(ticker));

        // Now that the group does exist, modify its perms.
        let perms = make_perms("pallet_external_agent");
        assert_ok!(set(AGId(1), perms.clone()));
        assert_eq!(Some(perms), ExternalAgents::permissions(ticker, AGId(1)));

        // Below we also test agent permissions checking logic.

        // Cheat a bit. Insert `other` as an agent but for a group that doesn't exist.
        GroupOfAgent::insert(ticker, other.did, AgentGroup::Custom(AGId(2)));
        assert_noop!(other_create(<_>::default()), Error::UnauthorizedAgent);
        assert_noop!(other_set(AGId(1), <_>::default()), Error::UnauthorizedAgent);

        // This group we did just create.
        GroupOfAgent::insert(ticker, other.did, AgentGroup::Custom(AGId(1)));
        assert_noop!(other_create(make_perms("foo")), Error::UnauthorizedAgent);
        set_extrinsic("create_group");
        assert_ok!(other_create(make_perms("foo")));
        assert_ok!(other_set(AGId(2), make_perms("bar")));
    });
}

#[test]
fn remove_abdicate_change_works() {
    ExtBuilder::default().build().execute_with(|| {
        let owner = User::new(AccountKeyring::Alice);
        let other = User::new(AccountKeyring::Bob);
        let (ticker, token) = a_token(owner.did);

        // Extrinsics under test:
        let remove = |u: User, who| ExternalAgents::remove_agent(u.origin(), ticker, who);
        let abdicate = |u: User| ExternalAgents::abdicate(u.origin(), ticker);
        let change = |u: User, a, g| ExternalAgents::change_group(u.origin(), ticker, a, g);

        // Granting helpers:
        let grant =
            |u: User, group| ExternalAgents::unchecked_add_agent(ticker, u.did, group).unwrap();
        let grant_full = |u| grant(u, AgentGroup::Full);

        // Asserts that `u` isn't an agent.
        let assert_group = |u: User, g| assert_eq!(g, GroupOfAgent::get(ticker, u.did));
        let assert_not_agent = |u| assert_group(u, None);

        // No asset made, so cannot remove non-agent.
        assert_noop!(remove(owner, owner.did), Error::UnauthorizedAgent);
        assert_noop!(abdicate(owner), Error::NotAnAgent);
        assert_noop!(
            change(owner, owner.did, AgentGroup::Full),
            Error::UnauthorizedAgent
        );

        // Make the asset.
        assert_ok!(basic_asset(owner, ticker, &token));

        // Asset exists, and owner is an agent, but other isn't, yet.
        assert_noop!(remove(owner, other.did), Error::NotAnAgent);
        assert_noop!(abdicate(other), Error::NotAnAgent);
        assert_noop!(
            change(owner, other.did, AgentGroup::Full),
            Error::NotAnAgent
        );

        // Cannot remove the last agent.
        assert_noop!(remove(owner, owner.did), Error::RemovingLastFullAgent);
        assert_noop!(abdicate(owner), Error::RemovingLastFullAgent);

        // Add another agent.
        grant_full(other);

        // Owner abdicates successfully.
        assert_ok!(abdicate(owner));
        assert_not_agent(owner);

        // Now removing other doesn't work.
        assert_noop!(remove(other, other.did), Error::RemovingLastFullAgent);
        assert_noop!(abdicate(other), Error::RemovingLastFullAgent);

        // Reinstate owner.
        grant_full(owner);

        // Other removes themselves, sucessfully.
        assert_ok!(remove(other, other.did));
        assert_not_agent(other);

        // Reinstate other.
        grant_full(other);

        // Owner removes other, sucessfully.
        assert_ok!(remove(owner, other.did));
        assert_not_agent(other);

        // Grant other effectively empty perms. Yet, they can still abdicate.
        grant(other, AgentGroup::Custom(AGId(0)));
        assert_ok!(abdicate(other));

        // Owner changes to `Full` group, sucessfully.
        assert_ok!(change(owner, owner.did, AgentGroup::Full));
        assert_group(owner, Some(AgentGroup::Full));

        // Owner changes to a group that doesn't exist.
        let ag = AgentGroup::Custom(AGId(1));
        let change_1 = || change(owner, owner.did, ag);
        assert_noop!(change_1(), Error::NoSuchAG);

        // Make that AG.
        assert_ok!(ExternalAgents::create_group(
            owner.origin(),
            ticker,
            <_>::default()
        ));

        // Cannot change to it yet, as there would be no full agents left.
        assert_noop!(change_1(), Error::RemovingLastFullAgent);

        // Make other a full agent again, so we can demote owner.
        grant_full(other);
        assert_ok!(change_1());
        assert_group(owner, Some(ag));
    });
}

#[test]
fn add_works() {
    ExtBuilder::default().build().execute_with(|| {
        let owner = User::new(AccountKeyring::Alice);
        let bob = User::new(AccountKeyring::Bob);
        let charlie = User::new(AccountKeyring::Charlie);
        let dave = User::new(AccountKeyring::Dave);
        let ticker = an_asset(owner, false);

        let check_num = |n| assert_eq!(ExternalAgents::num_full_agents(ticker), n);

        check_num(1);

        // Other is not an agent, so auths from them are not valid.
        add_become_agent(
            ticker,
            bob,
            owner,
            AgentGroup::Full,
            Err(Error::UnauthorizedAgent.into()),
        );
        check_num(1);

        // CAG is not valid
        add_become_agent(
            ticker,
            owner,
            bob,
            AgentGroup::Custom(AGId(1)),
            Err(Error::NoSuchAG.into()),
        );

        // Make a CAG & Other an agent of it.
        let perms = make_perms("pallet_external_agent");
        assert_ok!(ExternalAgents::create_group(owner.origin(), ticker, perms));
        add_become_agent(ticker, owner, bob, AgentGroup::Custom(AGId(1)), Ok(()));

        // Just made them an agent, cannot do it again.
        add_become_agent(
            ticker,
            owner,
            bob,
            AgentGroup::Custom(AGId(1)),
            Err(Error::AlreadyAnAgent.into()),
        );

        // Add another full agent and make sure count is incremented.
        add_become_agent(ticker, owner, charlie, AgentGroup::Full, Ok(()));
        check_num(2);

        // Force the count to overflow and test for graceful error.
        NumFullAgents::insert(ticker, u32::MAX);
        add_become_agent(
            ticker,
            owner,
            dave,
            AgentGroup::Full,
            Err(BaseError::CounterOverflow.into()),
        )
    });
}

#[test]
fn agent_of_mapping_works() {
    ExtBuilder::default().build().execute_with(|| {
        let owner = User::new(AccountKeyring::Alice);
        let bob = User::new(AccountKeyring::Bob);
        let charlie = User::new(AccountKeyring::Charlie);
        let dave = User::new(AccountKeyring::Dave);
        let mut tickers = (b'A'..b'Z')
            .map(|ticker| {
                let ticker = Ticker::from_slice_truncated(&[ticker] as &[u8]);
                crate::sto_test::create_asset(owner.origin(), ticker, POLY);
                ticker
            })
            .collect::<Vec<_>>();
        tickers.sort();

        let check = |user: User| {
            let mut agent_of_tickers = AgentOf::iter_prefix(user.did)
                .map(|(ticker, _)| ticker)
                .collect::<Vec<_>>();
            let mut group_of_tickers = GroupOfAgent::iter()
                .filter(|(_, d, _)| *d == user.did)
                .map(|(t, ..)| t)
                .collect::<Vec<_>>();
            agent_of_tickers.sort();
            group_of_tickers.sort();
            assert_eq!(agent_of_tickers, tickers);
            assert_eq!(agent_of_tickers, group_of_tickers);
        };
        let empty = |user: User| {
            assert!(AgentOf::iter_prefix(user.did).next().is_none());
            assert!(GroupOfAgent::iter()
                .filter(|(_, d, _)| *d == user.did)
                .next()
                .is_none());
        };
        let remove = |ticker, user: User| {
            assert_ok!(ExternalAgents::abdicate(user.origin(), ticker));
        };

        // Add EAs
        for ticker in &tickers {
            add_become_agent(*ticker, owner, bob, AgentGroup::Full, Ok(()));
            add_become_agent(*ticker, owner, charlie, AgentGroup::ExceptMeta, Ok(()));
            add_become_agent(*ticker, owner, dave, AgentGroup::PolymeshV1CAA, Ok(()));
            assert_eq!(ExternalAgents::num_full_agents(ticker), 2);
        }

        // Check the reverse mappings
        check(owner);
        check(bob);
        check(charlie);
        check(dave);

        // Remove EAs
        for ticker in &tickers {
            remove(*ticker, bob);
            remove(*ticker, charlie);
            remove(*ticker, dave);
            assert_eq!(ExternalAgents::num_full_agents(ticker), 1);
        }

        // Check the reverse mappings are correct or empty
        check(owner);
        empty(bob);
        empty(charlie);
        empty(dave);
    });
}

#[test]
fn atredis_multi_group_perms() {
    ExtBuilder::default().build().execute_with(|| {
        let owner = User::new(AccountKeyring::Alice);
        let other = User::new(AccountKeyring::Bob);
        let ticker = an_asset(owner, false);

        // Helpers for creating and setting permissions.
        let perms = make_perms("pallet_external_agent");
        let create = || {
            assert_ok!(ExternalAgents::create_group(
                owner.origin(),
                ticker,
                perms.clone()
            ));
            AGIdSequence::get(ticker)
        };
        let set =
            |g| ExternalAgents::set_group_permissions(other.origin(), ticker, g, perms.clone());

        // Create two groups for `ticker`.
        let a = create();
        let b = create();

        // Add `other` to group `a`.
        assert_ok!(ExternalAgents::unchecked_add_agent(
            ticker,
            other.did,
            AgentGroup::Custom(a)
        ));

        // Confirm that `other` has access to `set_group_permissions`.
        set_extrinsic("set_group_permissions");
        assert_ok!(ExternalAgents::ensure_agent_permissioned(ticker, other.did));
        assert_ok!(set(a));

        // Although `other` isn't part of the second group,
        // they are an agent with permissions for `set_group_permissions`,
        // and may therefore call it for the second group.
        assert_ok!(set(b));
    });
}
