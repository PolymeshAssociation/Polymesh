use crate::asset_test::{a_token, an_asset, basic_asset};
use crate::ext_builder::ExtBuilder;
use crate::identity_test::test_with_bad_ext_perms;
use crate::storage::{TestStorage, User};
use frame_support::{assert_noop, assert_ok, StorageDoubleMap, StorageMap};
use pallet_external_agents::{AGIdSequence, GroupOfAgent, NumFullAgents};
use pallet_permissions::StoreCallMetadata;
use polymesh_primitives::{
    agent::{AGId, AgentGroup},
    AuthorizationData, ExtrinsicPermissions, PalletPermissions, Signatory, SubsetRestriction,
};
use test_client::AccountKeyring;

type ExternalAgents = pallet_external_agents::Module<TestStorage>;
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
        assert_noop!(create(<_>::default()), Error::LocalAGIdOverflow);
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

        // We only test the specifics of `BecomeAgent` here,
        // under the assumption that the generic auth infra is tested elsewhere.
        let add = |from: User, to: User, group| {
            let data = AuthorizationData::BecomeAgent(ticker, group);
            let sig = Signatory::Identity(to.did);
            Id::add_auth(from.did, sig, data, None)
        };
        let accept = |to: User, id| Id::accept_authorization(to.origin(), id);
        let check_num = |n| assert_eq!(ExternalAgents::num_full_agents(ticker), n);

        check_num(1);

        // Other is not an agent, so auths from them are not valid.
        let id = add(bob, owner, AgentGroup::Full);
        assert_noop!(accept(owner, id), Error::UnauthorizedAgent);
        check_num(1);

        // CAG is not valid.
        let add_bob = || add(owner, bob, AgentGroup::Custom(AGId(1)));
        let accept_bob = |id| {
            let r = accept(bob, id);
            check_num(1);
            r
        };
        let id = add_bob();
        assert_noop!(accept_bob(id), Error::NoSuchAG);

        // Make a CAG & Other an agent of it.
        let perms = make_perms("pallet_external_agent");
        assert_ok!(ExternalAgents::create_group(owner.origin(), ticker, perms));
        assert_ok!(accept_bob(add_bob()));

        // Just made them an agent, cannot do it again.
        let id = add_bob();
        assert_noop!(accept_bob(id), Error::AlreadyAnAgent);

        // Add another full agent and make sure count is incremented.
        let id = add(owner, charlie, AgentGroup::Full);
        assert_ok!(accept(charlie, id));
        check_num(2);

        // Force the count to overflow and test for graceful error.
        NumFullAgents::insert(ticker, u32::MAX);
        let id = add(owner, dave, AgentGroup::Full);
        assert_noop!(accept(dave, id), Error::NumFullAgentsOverflow);
    });
}
