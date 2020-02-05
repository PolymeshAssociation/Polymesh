use crate::test::storage::{build_ext, register_keyring_account, TestStorage};
use polymesh_runtime_common::traits::{
    CommonTrait,
    group, identity,
    asset::AcceptTransfer,
    multisig::AddSignerMultiSig
};

use frame_support::{
    assert_err, assert_ok,
    dispatch::DispatchResult,
    impl_outer_origin, parameter_types,
    traits::Get,
    weights::{DispatchInfo, Weight},
};
use frame_system::EnsureSignedBy;
use pallet_transaction_payment::ChargeTransactionPayment;
use sp_core::H256;
use sp_io::{self};
use sp_runtime::{
    testing::Header,
    traits::{Convert, IdentityLookup, SignedExtension, Verify},
    AnySignature, Perbill,
};
use std::{cell::RefCell, result::Result};
use test_client::AccountKeyring;

impl_outer_origin! {
    pub enum Origin for Runtime {}
}

pub struct ExistentialDeposit;
impl Get<u128> for ExistentialDeposit {
    fn get() -> u128 {
        EXISTENTIAL_DEPOSIT.with(|v| *v.borrow())
    }
}

pub struct TransferFee;
impl Get<u128> for TransferFee {
    fn get() -> u128 {
        TRANSFER_FEE.with(|v| *v.borrow())
    }
}

pub struct CreationFee;
impl Get<u128> for CreationFee {
    fn get() -> u128 {
        CREATION_FEE.with(|v| *v.borrow())
    }
}



pub struct TransactionBaseFee;
impl Get<u128> for TransactionBaseFee {
    fn get() -> u128 {
        TRANSACTION_BASE_FEE.with(|v| *v.borrow())
    }
}

pub struct TransactionByteFee;
impl Get<u128> for TransactionByteFee {
    fn get() -> u128 {
        TRANSACTION_BYTE_FEE.with(|v| *v.borrow())
    }
}

pub struct WeightToFee(u128);
impl Convert<Weight, u128> for WeightToFee {
    fn convert(t: Weight) -> u128 {
        WEIGHT_TO_FEE.with(|v| *v.borrow() * (t as u128))
    }
}

pub struct ExtBuilder {
    transaction_base_fee: u128,
    transaction_byte_fee: u128,
    weight_to_fee: u128,
    existential_deposit: u128,
    transfer_fee: u128,
    creation_fee: u128,
    monied: bool,
    vesting: bool,
}
impl Default for ExtBuilder {
    fn default() -> Self {
        Self {
            transaction_base_fee: 0,
            transaction_byte_fee: 0,
            weight_to_fee: 0,
            existential_deposit: 0,
            transfer_fee: 0,
            creation_fee: 0,
            monied: false,
            vesting: false,
        }
    }
}
impl ExtBuilder {
    pub fn transaction_fees(
        mut self,
        base_fee: u128,
        byte_fee: u128,
        weight_fee: u128,
    ) -> Self {
        self.transaction_base_fee = base_fee;
        self.transaction_byte_fee = byte_fee;
        self.weight_to_fee = weight_fee;
        self
    }
    pub fn existential_deposit(mut self, existential_deposit: u128) -> Self {
        self.existential_deposit = existential_deposit;
        self
    }
    #[allow(dead_code)]
    pub fn transfer_fee(mut self, transfer_fee: u128) -> Self {
        self.transfer_fee = transfer_fee;
        self
    }
    pub fn monied(mut self, monied: bool) -> Self {
        self.monied = monied;
        if self.existential_deposit == 0 {
            self.existential_deposit = 1;
        }
        self
    }
    pub fn set_associated_consts(&self) {
        EXISTENTIAL_DEPOSIT.with(|v| *v.borrow_mut() = self.existential_deposit);
        TRANSFER_FEE.with(|v| *v.borrow_mut() = self.transfer_fee);
        CREATION_FEE.with(|v| *v.borrow_mut() = self.creation_fee);
        TRANSACTION_BASE_FEE.with(|v| *v.borrow_mut() = self.transaction_base_fee);
        TRANSACTION_BYTE_FEE.with(|v| *v.borrow_mut() = self.transaction_byte_fee);
        WEIGHT_TO_FEE.with(|v| *v.borrow_mut() = self.weight_to_fee);
    }
    pub fn build(self) -> sp_io::TestExternalities {
        self.set_associated_consts();
        let mut t = frame_system::GenesisConfig::default()
            .build_storage::<TestStorage>()
            .unwrap();
        GenesisConfig::<TestStorage> {
            balances: if self.monied {
                vec![
                    (
                        AccountKeyring::Alice.public(),
                        10 * self.existential_deposit,
                    ),
                    (AccountKeyring::Bob.public(), 20 * self.existential_deposit),
                    (
                        AccountKeyring::Charlie.public(),
                        30 * self.existential_deposit,
                    ),
                    (AccountKeyring::Dave.public(), 40 * self.existential_deposit),
                    // (12, 10 * self.existential_deposit),
                ]
            } else {
                vec![]
            },
            vesting: if self.vesting && self.monied {
                vec![
                    (
                        AccountKeyring::Alice.public(),
                        0,
                        10,
                        5 * self.existential_deposit,
                    ),
                    (AccountKeyring::Bob.public(), 10, 20, 0),
                    // (12, 10, 20, 5 * self.existential_deposit),
                ]
            } else {
                vec![]
            },
        }
        .assimilate_storage(&mut t)
            .unwrap();
                    t.into()
    }
}

pub type Balances = Module<TestStorage>;
pub type Identity = Module<TestStorage>;
pub type TransactionPayment = pallet_transaction_payment::Module<TestStorage>;

pub const CALL: &<Runtime as frame_system::Trait>::Call = &();

/// create a transaction info struct from weight. Handy to avoid building the whole struct.
pub fn info_from_weight(w: Weight) -> DispatchInfo {
    DispatchInfo {
        weight: w,
        ..Default::default()
    }
}

fn make_account(
    account_id: &AccountId,
) -> Result<(<Runtime as frame_system::Trait>::Origin, IdentityId), &'static str> {
    let signed_id = Origin::signed(account_id.clone());
    Identity::register_did(signed_id.clone(), vec![]);
    let did = Identity::get_identity(&Key::try_from(account_id.encode())?).unwrap();
    Ok((signed_id, did))
}

#[test]
#[ignore]
fn signed_extension_charge_transaction_payment_work() {
    ExtBuilder::default()
        .existential_deposit(10)
        .transaction_fees(10, 1, 5)
        .monied(true)
        .build()
        .execute_with(|| {
            let len = 10;
            let alice_pub = AccountKeyring::Alice.public();
            assert!(
                <ChargeTransactionPayment<Runtime> as SignedExtension>::pre_dispatch(
                    ChargeTransactionPayment::from(0),
                    &alice_pub,
                    CALL,
                    info_from_weight(5),
                    len
                )
                .is_ok()
            );
            assert_eq!(Balances::free_balance(&alice_pub), 100 - 20 - 25);
            assert!(
                <ChargeTransactionPayment<Runtime> as SignedExtension>::pre_dispatch(
                    ChargeTransactionPayment::from(0 /* 0 tip */),
                    &alice_pub,
                    CALL,
                    info_from_weight(3),
                    len
                )
                .is_ok()
            );
            assert_eq!(Balances::free_balance(&alice_pub), 100 - 20 - 25 - 20 - 15);
        });
}

#[test]
fn tipping_fails() {
    ExtBuilder::default()
        .existential_deposit(10)
        .transaction_fees(10, 1, 5)
        .monied(true)
        .build()
        .execute_with(|| {
            let len = 10;
            assert!(
                <ChargeTransactionPayment<Runtime> as SignedExtension>::pre_dispatch(
                    ChargeTransactionPayment::from(5 /* 5 tip */),
                    &AccountKeyring::Alice.public(),
                    CALL,
                    info_from_weight(3),
                    len
                )
                .is_err()
            );
        });
}

#[test]
#[ignore]
fn should_charge_identity() {
    ExtBuilder::default()
        .existential_deposit(10)
        .transaction_fees(10, 1, 5)
        .monied(true)
        .build()
        .execute_with(|| {
            let dave_pub = AccountKeyring::Dave.public();
            let (signed_acc_id, acc_did) = make_account(&dave_pub).unwrap();
            let len = 10;
            assert!(
                <ChargeTransactionPayment<Runtime> as SignedExtension>::pre_dispatch(
                    ChargeTransactionPayment::from(0 /* 0 tip */),
                    &dave_pub,
                    CALL,
                    info_from_weight(3),
                    len
                )
                .is_ok()
            );

            assert_ok!(Balances::change_charge_did_flag(
                    signed_acc_id.clone(),
                    true
            ));
            assert!(
                <ChargeTransactionPayment<Runtime> as SignedExtension>::pre_dispatch(
                    ChargeTransactionPayment::from(0 /* 0 tip */),
                    &dave_pub,
                    CALL,
                    info_from_weight(3),
                    len
                )
                .is_err()
            ); // no balance in identity
            assert_eq!(Balances::free_balance(&dave_pub), 365);
            assert_ok!(Balances::top_up_identity_balance(
                    signed_acc_id.clone(),
                    acc_did,
                    300
            ));
            assert_eq!(Balances::free_balance(&dave_pub), 65);
            assert_eq!(Balances::identity_balance(acc_did), 300);
            assert!(
                <ChargeTransactionPayment<Runtime> as SignedExtension>::pre_dispatch(
                    ChargeTransactionPayment::from(0 /* 0 tip */),
                    &dave_pub,
                    CALL,
                    info_from_weight(3),
                    len
                )
                .is_ok()
            );
            assert_ok!(Balances::reclaim_identity_balance(
                    signed_acc_id.clone(),
                    acc_did,
                    230
            ));
            assert_err!(
                Balances::reclaim_identity_balance(signed_acc_id, acc_did, 230),
                "too few free funds in account"
            );
            assert_eq!(Balances::free_balance(&dave_pub), 295);
            assert_eq!(Balances::identity_balance(acc_did), 35);
        });
}

