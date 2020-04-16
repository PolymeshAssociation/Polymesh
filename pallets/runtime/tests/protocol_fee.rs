mod common;
use codec::Encode;
use common::{
    ext_builder::PROTOCOL_OP_BASE_FEE,
    storage::{register_keyring_account_with_balance, TestStorage},
    ExtBuilder,
};
use frame_support::{assert_err, assert_ok};
use polymesh_primitives::{AccountKey, Signatory};
use polymesh_runtime_common::protocol_fee::ProtocolOp;
use std::convert::TryFrom;
use test_client::AccountKeyring;

type Error = polymesh_protocol_fee::Error<TestStorage>;
type ProtocolFee = polymesh_protocol_fee::Module<TestStorage>;

#[test]
fn can_compute_fee() {
    ExtBuilder::default().build().execute_with(|| {
        assert_eq!(
            ProtocolFee::compute_fee(ProtocolOp::AssetIssue),
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
        let alice_signer =
            Signatory::from(AccountKey::try_from(AccountKeyring::Alice.public().encode()).unwrap());
        assert_ok!(ProtocolFee::charge_fee_batch(
            &alice_signer,
            ProtocolOp::AssetIssue,
            7,
        ));
        assert_err!(
            ProtocolFee::charge_fee_batch(&alice_signer, ProtocolOp::AssetIssue, 7,),
            Error::InsufficientAccountBalance
        );
    });
}
