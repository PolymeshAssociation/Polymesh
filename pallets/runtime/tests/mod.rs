mod all;
pub use all::storage::{
    account_from, add_signing_item, get_identity_id, make_account, make_account_with_balance,
    make_account_without_cdd, register_keyring_account_with_balance,
    register_keyring_account_without_cdd, TestStorage,
};
