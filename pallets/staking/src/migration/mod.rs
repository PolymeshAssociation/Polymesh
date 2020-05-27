use super::*;
mod deprecated;

pub fn on_runtime_upgrade<T: Trait>() {
    match StorageVersion::get() {
        Releases::V1_1_0 => upgrade_storage::<T>(),
        Releases::V1_0_0 => return,
    }
}

fn upgrade_storage<T: Trait>() {
    let validators = deprecated::PermissionedValidators::<T>::enumerate()
        .into_iter()
        .map(|(a, _)| a)
        .collect::<Vec<T::AccountId>>();
    validators.iter().for_each(|v| {
        <PermissionedValidators<T>>::insert(v, true);
        frame_support::print("validators are moving to new storage");
    });
    //deprecated::PermissionedValidators::<T>::remove_all();

    StorageVersion::put(Releases::V1_1_0);
}
