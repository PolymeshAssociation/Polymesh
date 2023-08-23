use super::{
    ext_builder::PROTOCOL_OP_BASE_FEE,
    storage::{register_keyring_account_with_balance, TestStorage},
    ExtBuilder,
};
use frame_support::{assert_noop, assert_ok};
use polymesh_common_utilities::{
    protocol_fee::ProtocolOp, traits::transaction_payment::CddAndFeeDetails,
};
use sp_keyring::AccountKeyring;

type Error = pallet_protocol_fee::Error<TestStorage>;
type ProtocolFee = pallet_protocol_fee::Module<TestStorage>;

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
            register_keyring_account_with_balance(AccountKeyring::Alice, PROTOCOL_OP_BASE_FEE * 10)
                .unwrap();
        TestStorage::set_payer_context(Some(AccountKeyring::Alice.to_account_id()));
        assert_eq!(
            TestStorage::get_payer_from_context(),
            Some(AccountKeyring::Alice.to_account_id())
        );
        assert_ok!(ProtocolFee::batch_charge_fee(ProtocolOp::AssetIssue, 7));
        assert_noop!(
            ProtocolFee::batch_charge_fee(ProtocolOp::AssetIssue, 7),
            Error::InsufficientAccountBalance
        );
    });
}
