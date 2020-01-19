use crate::{
    statistics,
    test::storage::{build_ext, register_keyring_account, TestStorage},
}

#[test]
fn investor_count_per_asset_test() {
    with_externalities(&mut build_ext(), investor_count_per_asset_test_with_ext)
}

fn investor_count_per_asset_test_with_ext() {

}

