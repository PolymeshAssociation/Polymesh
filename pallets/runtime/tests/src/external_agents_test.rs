use crate::asset_pallet::setup::create_and_issue_sample_asset;
use crate::ext_builder::ExtBuilder;
use crate::identity_test::test_with_bad_ext_perms;
use crate::storage::{TestStorage, User};
use frame_support::dispatch::DispatchResult;
use frame_support::{
    assert_noop, assert_ok, IterableStorageDoubleMap, StorageDoubleMap, StorageMap,
};
use pallet_external_agents::{AGIdSequence, AgentOf, GroupOfAgent, NumFullAgents};
use pallet_permissions::StoreCallMetadata;
use polymesh_primitives::asset::AssetId;
use polymesh_primitives::{
    agent::{AGId, AgentGroup},
    AuthorizationData, ExtrinsicPermissions, PalletPermissions, Signatory,
};
use sp_keyring::AccountKeyring;

type ExternalAgents = pallet_external_agents::Module<TestStorage>;
type BaseError = pallet_base::Error<TestStorage>;
type Error = pallet_external_agents::Error<TestStorage>;
type Id = pallet_identity::Module<TestStorage>;

fn set_extrinsic(name: &str) {
    StoreCallMetadata::<TestStorage>::set_call_metadata(
        "pallet_external_agent".into(),
        name.into(),
    );
}

fn make_perms(pallet: &str) -> ExtrinsicPermissions {
    ExtrinsicPermissions::these([PalletPermissions::entire_pallet(pallet.into())])
}

fn add_become_agent(
    asset_id: AssetId,
    from: User,
    to: User,
    group: AgentGroup,
    expected: DispatchResult,
) {
    let data = AuthorizationData::BecomeAgent(asset_id, group);
    let sig = Signatory::Identity(to.did);
    let auth = Id::add_auth(from.did, sig, data, None).unwrap();
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

        assert_noop!(
            ExternalAgents::create_group(owner.origin(), [0; 16].into(), <_>::default()),
            Error::UnauthorizedAgent
        );
        assert_noop!(
            ExternalAgents::set_group_permissions(
                owner.origin(),
                [0; 16].into(),
                AGId(0),
                <_>::default()
            ),
            Error::UnauthorizedAgent
        );

        let asset_id = create_and_issue_sample_asset(&owner);

        let create = |perms| ExternalAgents::create_group(owner.origin(), asset_id, perms);
        let set =
            |id, perms| ExternalAgents::set_group_permissions(owner.origin(), asset_id, id, perms);

        // Make the asset. Let's test permissions length limits.
        test_with_bad_ext_perms(|perms| {
            assert_too_long!(create(perms.clone()));
            assert_too_long!(set(AGId(0), perms));
        });

        // Still, `other` doesn't have agent permissions.
        let other = User::new(AccountKeyring::Bob);
        let other_create = |perms| ExternalAgents::create_group(other.origin(), asset_id, perms);
        let other_set =
            |id, perms| ExternalAgents::set_group_permissions(other.origin(), asset_id, id, perms);
        assert_noop!(other_create(<_>::default()), Error::UnauthorizedAgent);
        assert_noop!(other_set(AGId(1), <_>::default()), Error::UnauthorizedAgent);

        // Try setting perms for groups that don't exist.
        for g in 0..3 {
            assert_noop!(set(AGId(g), <_>::default()), Error::NoSuchAG);
        }

        // Manipulate storage so that ID will overflow.
        AGIdSequence::insert(asset_id, AGId(u32::MAX));
        assert_noop!(create(<_>::default()), BaseError::CounterOverflow);
        AGIdSequence::insert(asset_id, AGId::default());

        // Add a group successfully.
        let perms = make_perms("foo");
        assert_ok!(create(perms.clone()));
        assert_eq!(Some(perms), ExternalAgents::permissions(asset_id, AGId(1)));
        assert_eq!(AGId(1), ExternalAgents::agent_group_id_sequence(asset_id));

        // Now that the group does exist, modify its perms.
        let perms = make_perms("pallet_external_agent");
        assert_ok!(set(AGId(1), perms.clone()));
        assert_eq!(Some(perms), ExternalAgents::permissions(asset_id, AGId(1)));

        // Below we also test agent permissions checking logic.

        // Cheat a bit. Insert `other` as an agent but for a group that doesn't exist.
        GroupOfAgent::insert(asset_id, other.did, AgentGroup::Custom(AGId(2)));
        assert_noop!(other_create(<_>::default()), Error::UnauthorizedAgent);
        assert_noop!(other_set(AGId(1), <_>::default()), Error::UnauthorizedAgent);

        // This group we did just create.
        GroupOfAgent::insert(asset_id, other.did, AgentGroup::Custom(AGId(1)));
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

        // No asset made, so cannot remove non-agent.
        assert_noop!(
            ExternalAgents::remove_agent(owner.origin(), [0; 16].into(), other.did),
            Error::UnauthorizedAgent
        );
        assert_noop!(
            ExternalAgents::abdicate(owner.origin(), [0; 16].into()),
            Error::NotAnAgent
        );
        assert_noop!(
            ExternalAgents::change_group(
                owner.origin(),
                [0; 16].into(),
                owner.did,
                AgentGroup::Full
            ),
            Error::UnauthorizedAgent
        );

        let asset_id = create_and_issue_sample_asset(&owner);

        // Extrinsics under test:
        let remove = |u: User, who| ExternalAgents::remove_agent(u.origin(), asset_id, who);
        let abdicate = |u: User| ExternalAgents::abdicate(u.origin(), asset_id);
        let change = |u: User, a, g| ExternalAgents::change_group(u.origin(), asset_id, a, g);

        // Granting helpers:
        let grant =
            |u: User, group| ExternalAgents::unchecked_add_agent(asset_id, u.did, group).unwrap();
        let grant_full = |u| grant(u, AgentGroup::Full);

        // Asserts that `u` isn't an agent.
        let assert_group = |u: User, g| assert_eq!(g, GroupOfAgent::get(asset_id, u.did));
        let assert_not_agent = |u| assert_group(u, None);

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
            asset_id,
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
        let asset_id = create_and_issue_sample_asset(&owner);

        let check_num = |n| assert_eq!(ExternalAgents::num_full_agents(asset_id), n);

        check_num(1);

        // Other is not an agent, so auths from them are not valid.
        add_become_agent(
            asset_id,
            bob,
            owner,
            AgentGroup::Full,
            Err(Error::UnauthorizedAgent.into()),
        );
        check_num(1);

        // CAG is not valid
        add_become_agent(
            asset_id,
            owner,
            bob,
            AgentGroup::Custom(AGId(1)),
            Err(Error::NoSuchAG.into()),
        );

        // Make a CAG & Other an agent of it.
        let perms = make_perms("pallet_external_agent");
        assert_ok!(ExternalAgents::create_group(
            owner.origin(),
            asset_id,
            perms
        ));
        add_become_agent(asset_id, owner, bob, AgentGroup::Custom(AGId(1)), Ok(()));

        // Just made them an agent, cannot do it again.
        add_become_agent(
            asset_id,
            owner,
            bob,
            AgentGroup::Custom(AGId(1)),
            Err(Error::AlreadyAnAgent.into()),
        );

        // Add another full agent and make sure count is incremented.
        add_become_agent(asset_id, owner, charlie, AgentGroup::Full, Ok(()));
        check_num(2);

        // Force the count to overflow and test for graceful error.
        NumFullAgents::insert(asset_id, u32::MAX);
        add_become_agent(
            asset_id,
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
        let mut assets = (b'A'..b'Z')
            .map(|_| create_and_issue_sample_asset(&owner))
            .collect::<Vec<_>>();
        assets.sort();

        let check = |user: User| {
            let mut agent_of_tickers = AgentOf::iter_prefix(user.did)
                .map(|(asset_id, _)| asset_id)
                .collect::<Vec<_>>();
            let mut group_of_tickers = GroupOfAgent::iter()
                .filter(|(_, d, _)| *d == user.did)
                .map(|(t, ..)| t)
                .collect::<Vec<_>>();
            agent_of_tickers.sort();
            group_of_tickers.sort();
            assert_eq!(agent_of_tickers, assets);
            assert_eq!(agent_of_tickers, group_of_tickers);
        };
        let empty = |user: User| {
            assert!(AgentOf::iter_prefix(user.did).next().is_none());
            assert!(GroupOfAgent::iter()
                .filter(|(_, d, _)| *d == user.did)
                .next()
                .is_none());
        };
        let remove = |asset_id, user: User| {
            assert_ok!(ExternalAgents::abdicate(user.origin(), asset_id));
        };

        // Add EAs
        for asset_id in &assets {
            add_become_agent(*asset_id, owner, bob, AgentGroup::Full, Ok(()));
            add_become_agent(*asset_id, owner, charlie, AgentGroup::ExceptMeta, Ok(()));
            add_become_agent(*asset_id, owner, dave, AgentGroup::PolymeshV1CAA, Ok(()));
            assert_eq!(ExternalAgents::num_full_agents(asset_id), 2);
        }

        // Check the reverse mappings
        check(owner);
        check(bob);
        check(charlie);
        check(dave);

        // Remove EAs
        for asset_id in &assets {
            remove(*asset_id, bob);
            remove(*asset_id, charlie);
            remove(*asset_id, dave);
            assert_eq!(ExternalAgents::num_full_agents(asset_id), 1);
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
        let asset_id = create_and_issue_sample_asset(&owner);

        // Helpers for creating and setting permissions.
        let perms = make_perms("pallet_external_agent");
        let create = || {
            assert_ok!(ExternalAgents::create_group(
                owner.origin(),
                asset_id,
                perms.clone()
            ));
            AGIdSequence::get(asset_id)
        };
        let set =
            |g| ExternalAgents::set_group_permissions(other.origin(), asset_id, g, perms.clone());

        // Create two groups for `asset_id`.
        let a = create();
        let b = create();

        // Add `other` to group `a`.
        assert_ok!(ExternalAgents::unchecked_add_agent(
            asset_id,
            other.did,
            AgentGroup::Custom(a)
        ));

        // Confirm that `other` has access to `set_group_permissions`.
        set_extrinsic("set_group_permissions");
        assert_ok!(ExternalAgents::ensure_agent_permissioned(
            &asset_id, other.did
        ));
        assert_ok!(set(a));

        // Although `other` isn't part of the second group,
        // they are an agent with permissions for `set_group_permissions`,
        // and may therefore call it for the second group.
        assert_ok!(set(b));
    });
}
