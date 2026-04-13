pub mod v7 {
    use frame_support::pallet_prelude::*;
    use polymesh_primitives::IdentityId;

    use crate::{Config, Pallet};

    #[frame_support::storage_alias]
    pub type ParentDid<T: Config> =
        StorageMap<Pallet<T>, Identity, IdentityId, IdentityId, OptionQuery>;

    #[frame_support::storage_alias]
    pub type ChildDid<T: Config> =
        StorageDoubleMap<Pallet<T>, Identity, IdentityId, Identity, IdentityId, bool, ValueQuery>;

    pub fn migrate_to_v8<T: Config>() -> Weight {
        let mut writes: u64 = 0;
        let mut reads: u64 = 0;

        for (child_did, parent_did) in ParentDid::<T>::drain() {
            ChildDid::<T>::remove(&parent_did, &child_did);
            reads += 1;
            writes += 2;
        }

        log::info!("Child identities removed: {} items", reads);

        T::DbWeight::get().reads_writes(reads, writes.into())
    }
}
