use crate::{
    traits::identity,
    CommonTrait,
};
use frame_support::{decl_event, weights::Weight};
use polymesh_primitives::EventDid;

pub trait WeightInfo {
    fn set_paying_key() -> Weight;
}

pub trait Trait: CommonTrait + identity::Trait {
    /// The overarching event type.
    type Event: From<Event<Self>> + Into<<Self as frame_system::Trait>::Event>;

    type WeightInfo: WeightInfo;
}

decl_event! {
    pub enum Event<T>
    where
        AccountId = <T as frame_system::Trait>::AccountId,
    {
        /// Set paying key
        ///
        /// (Caller DID, User Key, auth id)
        SetPayingKey(EventDid, AccountId, u64),

    }
}
