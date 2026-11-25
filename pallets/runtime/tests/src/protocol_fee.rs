use super::{
    ext_builder::PROTOCOL_OP_BASE_FEE,
    storage::{register_keyring_account_with_balance, TestStorage},
    ExtBuilder,
};
use frame_support::{assert_noop, assert_ok};
use polymesh_primitives::protocol_fee::ProtocolOp;
use polymesh_primitives::traits::CurrentFeePayer;
use polymesh_primitives::PosRatio;
use sp_keyring::Sr25519Keyring;

type Error = pallet_protocol_fee::Error<TestStorage>;
type ProtocolFee = pallet_protocol_fee::Pallet<TestStorage>;

#[test]
fn can_compute_fee() {
    ExtBuilder::default().build().execute_with(|| {
        assert_eq!(
            ProtocolFee::compute_fee(&[ProtocolOp::AssetIssue]),
            PROTOCOL_OP_BASE_FEE
        );
    });
}

#[test]
fn can_charge_fee_batch() {
    ExtBuilder::default().build().execute_with(|| {
        let _ =
            register_keyring_account_with_balance(Sr25519Keyring::Alice, PROTOCOL_OP_BASE_FEE * 10)
                .unwrap();
        TestStorage::set_payer_context(Some(Sr25519Keyring::Alice.to_account_id()));
        assert_eq!(
            TestStorage::get_payer_from_context(),
            Some(Sr25519Keyring::Alice.to_account_id())
        );
        assert_ok!(ProtocolFee::batch_charge_fee(ProtocolOp::AssetIssue, 7));
        assert_noop!(
            ProtocolFee::batch_charge_fee(ProtocolOp::AssetIssue, 7),
            Error::InsufficientAccountBalance
        );
    });
}

#[test]
fn change_coefficient_rejects_values_outside_unit_range() {
    ExtBuilder::default().build().execute_with(|| {
        let root = frame_system::RawOrigin::Root.into();
        // Numerator greater than denominator would be silently clamped to `1`.
        assert_noop!(
            ProtocolFee::change_coefficient(root, PosRatio(2, 1)),
            Error::InvalidCoefficient
        );
        // A zero denominator would also be silently clamped to `1`.
        assert_noop!(
            ProtocolFee::change_coefficient(frame_system::RawOrigin::Root.into(), PosRatio(1, 0)),
            Error::InvalidCoefficient
        );
        assert_eq!(
            pallet_protocol_fee::Coefficient::<TestStorage>::get(),
            PosRatio(1, 1)
        );
    });
}

#[test]
fn change_coefficient_applies_discount() {
    ExtBuilder::default().build().execute_with(|| {
        assert_ok!(ProtocolFee::change_coefficient(
            frame_system::RawOrigin::Root.into(),
            PosRatio(1, 2)
        ));
        assert_eq!(
            ProtocolFee::compute_fee(&[ProtocolOp::AssetIssue]),
            PROTOCOL_OP_BASE_FEE / 2
        );
        assert_ok!(ProtocolFee::change_coefficient(
            frame_system::RawOrigin::Root.into(),
            PosRatio(0, 1)
        ));
        assert_eq!(ProtocolFee::compute_fee(&[ProtocolOp::AssetIssue]), 0);
    });
}
