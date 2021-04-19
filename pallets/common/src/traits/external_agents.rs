use frame_support::{decl_event, weights::Weight};
use polymesh_primitives::agent::{AGId, AgentGroup};
use polymesh_primitives::{EventDid, ExtrinsicPermissions, IdentityId, Ticker};

pub trait WeightInfo {
    fn create_group() -> Weight;
    fn set_group_permissions() -> Weight;
    fn remove_agent() -> Weight;
    fn change_group() -> Weight;
}

pub trait Trait: frame_system::Trait + crate::balances::Trait {
    /// The overarching event type.
    type Event: From<Event> + Into<<Self as frame_system::Trait>::Event>;

    type WeightInfo: WeightInfo;
}

decl_event! {
    pub enum Event {
        /// An Agent Group was created.
        ///
        /// (Caller DID, AG's ticker, AG's ID, AG's permissions)
        GroupCreated(EventDid, Ticker, AGId, ExtrinsicPermissions),

        /// An Agent Group's permissions was updated.
        ///
        /// (Caller DID, AG's ticker, AG's ID, AG's new permissions)
        GroupPermissionsUpdated(EventDid, Ticker, AGId, ExtrinsicPermissions),

        /// An agent was added.
        ///
        /// (Caller/Agent DID, Agent's ticker, Agent's group)
        AgentAdded(EventDid, Ticker, AgentGroup),

        /// An agent was removed.
        ///
        /// (Caller DID, Agent's ticker, Agent's DID)
        AgentRemoved(EventDid, Ticker, IdentityId),

        /// An agent's group was changed.
        ///
        /// (Caller DID, Agent's ticker, Agent's DID, The new group of the agent)
        GroupChanged(EventDid, Ticker, IdentityId, AgentGroup),
    }
}
