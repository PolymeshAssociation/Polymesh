use crate::{
    storage::{Call, TestStorage},
    ExtBuilder,
};

use pallet_balances::Call as BalancesCall;
use polymesh_extensions::SignedExtra;

use frame_support::{weights::DispatchClass, weights::DispatchInfo};
use sp_core::sr25519::Public;
use sp_runtime::traits::SignedExtension;
use test_client::AccountKeyring;

fn make_call(user: Public) -> (<TestStorage as frame_system::Trait>::Call, usize) {
    (Call::Balances(BalancesCall::transfer(user, 69)), 10)
}

#[test]
fn normal_tx_ext() {
    ExtBuilder::default()
        .monied(true)
        .build()
        .execute_with(normal_tx);
}

fn normal_tx() {
    let user = AccountKeyring::Alice.public();
    let (call, len) = make_call(user.clone());
    let info = DispatchInfo {
        weight: 100,
        ..Default::default()
    };

    // Normat Tx with tip. Expected an error.
    let sign_extra = SignedExtra::<TestStorage>::new(0, 10, 0u64.into(), 42u128.into());
    let tx_validity = sign_extra.validate(&user, &call, &info, len);
    assert!(tx_validity.is_err());

    // Normal TX without any tip.
    let sign_extra = SignedExtra::<TestStorage>::new(0, 10, 0u64.into(), 0u128.into());
    let tx_validity = sign_extra
        .validate(&user, &call, &info, len)
        .expect("Tx should be valid");
    assert_eq!(tx_validity.priority, 0);
}

#[test]
fn operational_tx_ext() {
    ExtBuilder::default()
        .monied(true)
        .build()
        .execute_with(operational_tx);
}

fn operational_tx() {
    let user = AccountKeyring::Alice.public();
    let (call, len) = make_call(user.clone());
    let info = DispatchInfo {
        weight: 100,
        class: DispatchClass::Operational,
        ..Default::default()
    };

    // Operational TX with tip.
    let tip = 42u128;
    let sign_extra = SignedExtra::<TestStorage>::new(0, 10, 0u64.into(), tip.into());
    let tx_validity = sign_extra
        .validate(&user, &call, &info, len)
        .expect("Tx should be valid");
    assert_eq!(tx_validity.priority as u128, tip);

    // Operational TX without any tip.
    let sign_extra = SignedExtra::<TestStorage>::new(0, 10, 0u64.into(), 0u128.into());
    let tx_validity = sign_extra
        .validate(&user, &call, &info, len)
        .expect("Tx should be valid");
    assert_eq!(tx_validity.priority, 0);
}
