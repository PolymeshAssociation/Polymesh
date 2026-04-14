use codec::Encode;
use frame_support::assert_ok;
use frame_support::dispatch::{DispatchClass, DispatchInfo};
use frame_support::dispatch::{GetDispatchInfo, Pays, PostDispatchInfo};
use frame_support::traits::Currency;
use frame_support::weights::{Weight, WeightToFee};
use sp_arithmetic::traits::One;
use sp_keyring::Sr25519Keyring;
use sp_runtime::generic::UncheckedExtrinsic;
use sp_runtime::traits::{Dispatchable, TransactionExtension, TxBaseImplication};
use sp_runtime::transaction_validity::TransactionSource;
use sp_runtime::transaction_validity::{InvalidTransaction, TransactionValidityError};
use sp_runtime::{FixedPointNumber, MultiAddress};

use pallet_balances::Call as BalancesCall;
use pallet_identity::Authorizations;
use pallet_multisig::ProposalStates;
use pallet_transaction_payment::{Multiplier, NextFeeMultiplier, RuntimeDispatchInfo};
use polymesh_primitives::multisig::ProposalState;
use polymesh_primitives::traits::CurrentFeePayer;
use polymesh_primitives::transaction_payment::CallPaymentInfo;
use polymesh_primitives::TransactionError;
use polymesh_primitives::{AccountId, AuthorizationData, Permissions, Signatory};
use polymesh_transaction_payment::{ChargeTransactionPayment, Val};

use super::ext_builder::ExtBuilder;
use super::storage::{Address, RuntimeCall, TestStorage};
use crate::multisig::{create_multisig_default_perms, create_signers};
use crate::storage::{get_last_auth_id, User};

type Identity = pallet_identity::Pallet<TestStorage>;
type MultiSig = pallet_multisig::Pallet<TestStorage>;
type MultisigError = pallet_multisig::Error<TestStorage>;
type Origin = <TestStorage as frame_system::Config>::RuntimeOrigin;
type RuntimeOrigin = <TestStorage as frame_system::Config>::RuntimeOrigin;

fn call() -> <TestStorage as frame_system::Config>::RuntimeCall {
    RuntimeCall::Balances(BalancesCall::transfer_allow_death {
        dest: MultiAddress::Id(Sr25519Keyring::Alice.to_account_id()),
        value: 69,
    })
}

type Balances = pallet_balances::Pallet<TestStorage>;
type System = frame_system::Pallet<TestStorage>;
type TransactionPayment = pallet_transaction_payment::Pallet<TestStorage>;

/// create a transaction info struct from weight. Handy to avoid building the whole struct.
pub fn info_from_weight(w: u64) -> DispatchInfo {
    // pays_fee: Pays::Yes -- class: DispatchClass::Normal
    DispatchInfo {
        call_weight: Weight::from_parts(w, 0),
        ..Default::default()
    }
}

fn weight_to_fee(weight: Weight) -> u128 {
    <TestStorage as pallet_transaction_payment::Config>::WeightToFee::weight_to_fee(&weight)
}

fn operational_info_from_weight(w: u64) -> DispatchInfo {
    DispatchInfo {
        call_weight: Weight::from_parts(w, 0),
        class: DispatchClass::Operational,
        ..Default::default()
    }
}

fn post_info_from_weight(w: u64) -> PostDispatchInfo {
    PostDispatchInfo {
        actual_weight: Some(Weight::from_parts(w, 0)),
        pays_fee: Pays::Yes,
    }
}

fn default_post_info() -> PostDispatchInfo {
    PostDispatchInfo {
        actual_weight: None,
        pays_fee: Pays::Yes,
    }
}

#[test]
fn signed_extension_transaction_payment_work() {
    ExtBuilder::default()
        .monied(true)
        .transaction_fees(5, 1, 1)
        .build()
        .execute_with(|| {
            let bob = Sr25519Keyring::Bob.to_account_id();
            let alice = Sr25519Keyring::Alice.to_account_id();

            let len = 10;
            let bob_origin = RuntimeOrigin::signed(bob.clone());

            let val = ChargeTransactionPayment::<TestStorage>::from(0)
                .validate(
                    bob_origin.clone(),
                    &call(),
                    &info_from_weight(5),
                    len,
                    Default::default(),
                    &TxBaseImplication(()),
                    TransactionSource::InBlock,
                )
                .unwrap();

            let pre = ChargeTransactionPayment::<TestStorage>::from(0)
                .prepare(val.1, &bob_origin, &call(), &info_from_weight(5), len)
                .unwrap();
            assert_eq!(Balances::free_balance(&bob), 1999969001);

            assert!(ChargeTransactionPayment::<TestStorage>::post_dispatch(
                pre,
                &info_from_weight(5),
                &mut default_post_info(),
                len,
                &Ok(())
            )
            .is_ok());
            assert_eq!(Balances::free_balance(&bob), 1999969001);

            let alice_origin = RuntimeOrigin::signed(alice.clone());

            let val = ChargeTransactionPayment::<TestStorage>::from(0)
                .validate(
                    alice_origin.clone(),
                    &call(),
                    &info_from_weight(100),
                    len,
                    Default::default(),
                    &TxBaseImplication(()),
                    TransactionSource::InBlock,
                )
                .unwrap();

            let pre = ChargeTransactionPayment::<TestStorage>::from(0 /* tipped */)
                .prepare(val.1, &alice_origin, &call(), &info_from_weight(100), len)
                .unwrap();
            assert_eq!(Balances::free_balance(&alice), 999969001);

            assert!(ChargeTransactionPayment::<TestStorage>::post_dispatch(
                pre,
                &info_from_weight(100),
                &mut post_info_from_weight(50),
                len,
                &Ok(())
            )
            .is_ok());
            assert_eq!(Balances::free_balance(&alice), 999969001);
        });
}

#[test]
fn signed_extension_transaction_payment_multiplied_refund_works() {
    ExtBuilder::default()
        .monied(true)
        .transaction_fees(5, 1, 1)
        .build()
        .execute_with(|| {
            let user = Sr25519Keyring::Alice.to_account_id();
            let len = 10;
            NextFeeMultiplier::<TestStorage>::put(Multiplier::saturating_from_rational(3, 2));

            let alice_origin = RuntimeOrigin::signed(user.clone());

            let val = ChargeTransactionPayment::<TestStorage>::from(0)
                .validate(
                    alice_origin.clone(),
                    &call(),
                    &info_from_weight(100),
                    len,
                    Default::default(),
                    &TxBaseImplication(()),
                    TransactionSource::InBlock,
                )
                .unwrap();

            let pre = ChargeTransactionPayment::<TestStorage>::from(0 /* tipped */)
                .prepare(val.1, &alice_origin, &call(), &info_from_weight(100), len)
                .unwrap();
            // 5 base fee, 10 byte fee, 3/2 * 100 weight fee, 5 tip
            assert_eq!(Balances::free_balance(&user), 999969001);

            assert!(ChargeTransactionPayment::<TestStorage>::post_dispatch(
                pre,
                &info_from_weight(100),
                &mut post_info_from_weight(50),
                len,
                &Ok(())
            )
            .is_ok());
            // 75 (3/2 of the returned 50 units of weight) is refunded
            assert_eq!(Balances::free_balance(&user), 999969001);
        });
}

#[test]
fn signed_extension_transaction_payment_is_bounded() {
    ExtBuilder::default()
        .monied(true)
        .balance_factor(100)
        .transaction_fees(0, 0, 1)
        .build()
        .execute_with(|| {
            let user = Sr25519Keyring::Bob.to_account_id();
            let free_user = Balances::free_balance(&user);

            // Get the current weight settings.
            let weights = <TestStorage as frame_system::Config>::BlockWeights::get();

            // Calculate maximum transaction fee.
            let weight = weights.get(DispatchClass::Normal).base_extrinsic;
            let base_fee = weight_to_fee(weight);
            let len_fee = pallet_transaction_payment::Pallet::<TestStorage>::length_to_fee(10);
            let max_block_fee = weight_to_fee(weights.max_block);
            let max_fee = base_fee + len_fee + max_block_fee;

            // maximum weight possible
            let bob_origin = RuntimeOrigin::signed(user.clone());

            let val = ChargeTransactionPayment::<TestStorage>::from(0)
                .validate(
                    bob_origin.clone(),
                    &call(),
                    &info_from_weight(u64::MAX),
                    10,
                    Default::default(),
                    &TxBaseImplication(()),
                    TransactionSource::InBlock,
                )
                .unwrap();

            ChargeTransactionPayment::<TestStorage>::from(0)
                .prepare(val.1, &bob_origin, &call(), &info_from_weight(u64::MAX), 10)
                .unwrap();
            // fee will be proportional to what is the actual maximum weight in the runtime.
            assert_eq!(Balances::free_balance(&user), (free_user - max_fee));
        });
}

#[test]
fn signed_extension_allows_free_transactions() {
    ExtBuilder::default()
        .transaction_fees(100, 1, 1)
        .balance_factor(0)
        .build()
        .execute_with(|| {
            let user = Sr25519Keyring::Bob.to_account_id();
            let bob_origin = RuntimeOrigin::signed(user.clone());
            // I ain't have a penny.
            assert_eq!(Balances::free_balance(&user), 0);

            let len = 100;

            // This is a completely free (and thus wholly insecure/DoS-ridden) transaction.
            let operational_transaction = DispatchInfo {
                call_weight: Weight::from_parts(0, 0),
                extension_weight: Default::default(),
                class: DispatchClass::Operational,
                pays_fee: Pays::No,
            };
            assert!(ChargeTransactionPayment::<TestStorage>::from(0)
                .validate(
                    bob_origin.clone(),
                    &call(),
                    &operational_transaction,
                    len,
                    Default::default(),
                    &TxBaseImplication(()),
                    TransactionSource::InBlock,
                )
                .is_ok());

            // like a InsecureFreeNormal
            let free_transaction = DispatchInfo {
                call_weight: Weight::from_parts(0, 0),
                extension_weight: Default::default(),
                class: DispatchClass::Normal,
                pays_fee: Pays::Yes,
            };
            assert!(ChargeTransactionPayment::<TestStorage>::from(0)
                .validate(
                    bob_origin,
                    &call(),
                    &free_transaction,
                    len,
                    Default::default(),
                    &TxBaseImplication(()),
                    TransactionSource::InBlock
                )
                .is_err());
        });
}

#[test]
fn signed_ext_length_fee_is_also_updated_per_congestion() {
    ExtBuilder::default()
        .transaction_fees(5, 1, 1)
        .monied(true)
        .balance_factor(10)
        .build()
        .execute_with(|| {
            // all fees should be x1.5
            NextFeeMultiplier::<TestStorage>::put(Multiplier::saturating_from_rational(3, 2));
            let len = 10;
            let user = Sr25519Keyring::Bob.to_account_id();
            let bob_origin = RuntimeOrigin::signed(user.clone());

            let val = ChargeTransactionPayment::<TestStorage>::from(0)
                .validate(
                    bob_origin.clone(),
                    &call(),
                    &info_from_weight(3),
                    len,
                    Default::default(),
                    &TxBaseImplication(()),
                    TransactionSource::InBlock,
                )
                .unwrap();

            assert!(ChargeTransactionPayment::<TestStorage>::from(0) // tipped
                .prepare(
                    val.1,
                    &bob_origin,
                    &call(),
                    &info_from_weight(3),
                    len
                )
                .is_ok());
            assert_eq!(Balances::free_balance(&user), 19999969001);
        })
}

#[test]
fn query_info_works() {
    ExtBuilder::default()
        .monied(true)
        .transaction_fees(5, 1, 2)
        .build()
        .execute_with(|| {
            let acc = Sr25519Keyring::Bob.to_account_id();
            let key = Sr25519Keyring::from_account_id(&acc).unwrap();
            let signature = ().using_encoded(|b| key.sign(b));
            let unchecked_extrinsic =
                UncheckedExtrinsic::new_signed(call(), Address::Id(acc), signature, ());
            let info = unchecked_extrinsic.get_dispatch_info();

            // all fees should be x1.5
            NextFeeMultiplier::<TestStorage>::put(Multiplier::saturating_from_rational(3, 2));

            assert_eq!(
                TransactionPayment::query_info(
                    unchecked_extrinsic.clone(),
                    unchecked_extrinsic.encode().len() as u32
                ),
                RuntimeDispatchInfo {
                    weight: info.call_weight,
                    class: info.class,
                    partial_fee: 65098
                },
            );
        });
}

#[test]
fn compute_fee_works_without_multiplier() {
    ExtBuilder::default()
        .transaction_fees(100, 10, 1)
        .monied(false)
        .build()
        .execute_with(|| {
            // Next fee multiplier is zero
            assert_eq!(NextFeeMultiplier::<TestStorage>::get(), Multiplier::one());

            // Tip only, no fees works
            let dispatch_info = DispatchInfo {
                call_weight: Weight::from_parts(0, 0),
                extension_weight: Default::default(),
                class: DispatchClass::Operational,
                pays_fee: Pays::No,
            };
            assert_eq!(TransactionPayment::compute_fee(0, &dispatch_info, 10), 10);
            // No tip, only base fee works
            let dispatch_info = DispatchInfo {
                call_weight: Weight::from_parts(0, 0),
                extension_weight: Default::default(),
                class: DispatchClass::Operational,
                pays_fee: Pays::Yes,
            };
            assert_eq!(
                TransactionPayment::compute_fee(0, &dispatch_info, 0),
                29_999
            );
            // Tip + base fee works
            assert_eq!(
                TransactionPayment::compute_fee(0, &dispatch_info, 69),
                30_068
            );
            // Len (byte fee) + base fee works
            assert_eq!(
                TransactionPayment::compute_fee(42, &dispatch_info, 0),
                34_199
            );
            // Weight fee + base fee works
            let dispatch_info = DispatchInfo {
                call_weight: Weight::from_parts(1000, 0),
                extension_weight: Default::default(),
                class: DispatchClass::Operational,
                pays_fee: Pays::Yes,
            };
            assert_eq!(
                TransactionPayment::compute_fee(0, &dispatch_info, 0),
                29_999
            );
        });
}

#[test]
fn compute_fee_works_with_multiplier() {
    ExtBuilder::default()
        .transaction_fees(100, 10, 1)
        .monied(false)
        .build()
        .execute_with(|| {
            // Add a next fee multiplier. Fees will be x3/2.
            NextFeeMultiplier::<TestStorage>::put(Multiplier::saturating_from_rational(3, 2));
            // Base fee is unaffected by multiplier
            let dispatch_info = DispatchInfo {
                call_weight: Weight::from_parts(0, 0),
                extension_weight: Default::default(),
                class: DispatchClass::Operational,
                pays_fee: Pays::Yes,
            };
            assert_eq!(
                TransactionPayment::compute_fee(0, &dispatch_info, 0),
                29_999
            );

            // Everything works together :)
            let dispatch_info = DispatchInfo {
                call_weight: Weight::from_parts(123, 0),
                extension_weight: Default::default(),
                class: DispatchClass::Operational,
                pays_fee: Pays::Yes,
            };
            // 123 weight, 456 length, 100 base
            assert_eq!(
                TransactionPayment::compute_fee(456, &dispatch_info, 789),
                76_388,
            );
        });
}

#[test]
fn compute_fee_works_with_negative_multiplier() {
    ExtBuilder::default()
        .transaction_fees(100, 10, 1)
        .monied(false)
        .build()
        .execute_with(|| {
            // Add a next fee multiplier. All fees will be x1/2.
            NextFeeMultiplier::<TestStorage>::put(Multiplier::saturating_from_rational(1, 2));

            // Base fee is unaffected by multiplier.
            let dispatch_info = DispatchInfo {
                call_weight: Weight::from_parts(0, 0),
                extension_weight: Default::default(),
                class: DispatchClass::Operational,
                pays_fee: Pays::Yes,
            };
            assert_eq!(
                TransactionPayment::compute_fee(0, &dispatch_info, 0),
                29_999
            );

            // Everything works together.
            let dispatch_info = DispatchInfo {
                call_weight: Weight::from_parts(123, 0),
                extension_weight: Default::default(),
                class: DispatchClass::Operational,
                pays_fee: Pays::Yes,
            };
            // 123 weight, 456 length, 100 base
            assert_eq!(
                TransactionPayment::compute_fee(456, &dispatch_info, 789),
                76_388,
            );
        });
}

#[test]
fn compute_fee_does_not_overflow() {
    ExtBuilder::default()
        .transaction_fees(100, 10, 1)
        .monied(false)
        .build()
        .execute_with(|| {
            // Overflow is handled
            let dispatch_info = DispatchInfo {
                call_weight: Weight::MAX,
                extension_weight: Default::default(),
                class: DispatchClass::Operational,
                pays_fee: Pays::Yes,
            };
            assert_eq!(
                TransactionPayment::compute_fee(
                    <u32>::max_value(),
                    &dispatch_info,
                    <u128>::max_value()
                ),
                <u128>::max_value()
            );
        });
}

#[test]
fn actual_weight_higher_than_max_refunds_nothing() {
    ExtBuilder::default()
        .monied(true)
        .transaction_fees(5, 1, 1)
        .build()
        .execute_with(|| {
            let len = 10;
            let user = Sr25519Keyring::Alice.to_account_id();
            let alice_origin = RuntimeOrigin::signed(user.clone());

            let val = ChargeTransactionPayment::<TestStorage>::from(0)
                .validate(
                    alice_origin.clone(),
                    &call(),
                    &info_from_weight(5),
                    len,
                    Default::default(),
                    &TxBaseImplication(()),
                    TransactionSource::InBlock,
                )
                .unwrap();

            let pre = ChargeTransactionPayment::<TestStorage>::from(0 /* tipped */)
                .prepare(val.1, &alice_origin, &call(), &info_from_weight(100), len)
                .unwrap();
            assert_eq!(Balances::free_balance(&user), 999969001);

            ChargeTransactionPayment::<TestStorage>::post_dispatch(
                pre,
                &info_from_weight(100),
                &mut post_info_from_weight(101),
                len,
                &Ok(()),
            )
            .unwrap();
            assert_eq!(Balances::free_balance(&user), 999969001);
        });
}

#[test]
fn zero_transfer_on_free_transaction() {
    ExtBuilder::default()
        .monied(true)
        .transaction_fees(5, 1, 1)
        .build()
        .execute_with(|| {
            // So events are emitted
            System::set_block_number(10);
            let len = 10;
            let dispatch_info = DispatchInfo {
                call_weight: Weight::from_parts(100, 0),
                extension_weight: Default::default(),
                pays_fee: Pays::No,
                class: DispatchClass::Normal,
            };
            let user = Sr25519Keyring::Alice.to_account_id();
            let bal_init = Balances::total_balance(&user);
            let alice_origin = RuntimeOrigin::signed(user.clone());

            let val = ChargeTransactionPayment::<TestStorage>::from(0)
                .validate(
                    alice_origin.clone(),
                    &call(),
                    &dispatch_info,
                    len,
                    Default::default(),
                    &TxBaseImplication(()),
                    TransactionSource::InBlock,
                )
                .unwrap();

            let pre = ChargeTransactionPayment::<TestStorage>::from(0)
                .prepare(val.1, &alice_origin, &call(), &dispatch_info, len)
                .unwrap();

            assert_eq!(Balances::total_balance(&user), bal_init);
            assert!(ChargeTransactionPayment::<TestStorage>::post_dispatch(
                pre,
                &dispatch_info,
                &mut default_post_info(),
                len,
                &Ok(())
            )
            .is_ok());
            assert_eq!(Balances::total_balance(&user), bal_init);
            // One event for tx fee payment `TransactionFeePaid`.
            assert_eq!(System::events().len(), 1);
        });
}

#[test]
fn refund_consistent_with_actual_weight() {
    ExtBuilder::default()
        .monied(true)
        .transaction_fees(7, 1, 1)
        .build()
        .execute_with(|| {
            let info = info_from_weight(100);
            let mut post_info = post_info_from_weight(33);
            let alice = Sr25519Keyring::Alice.to_account_id();
            let prev_balance = Balances::free_balance(&alice);
            let len = 10;
            let tip = 0;

            NextFeeMultiplier::<TestStorage>::put(Multiplier::saturating_from_rational(5, 4));

            let alice_origin = RuntimeOrigin::signed(alice.clone());

            let val = ChargeTransactionPayment::<TestStorage>::from(0)
                .validate(
                    alice_origin.clone(),
                    &call(),
                    &info_from_weight(100),
                    len,
                    Default::default(),
                    &TxBaseImplication(()),
                    TransactionSource::InBlock,
                )
                .unwrap();

            let pre = ChargeTransactionPayment::<TestStorage>::from(tip)
                .prepare(val.1, &alice_origin, &call(), &info, len)
                .unwrap();

            ChargeTransactionPayment::<TestStorage>::post_dispatch(
                pre,
                &info,
                &mut post_info,
                len,
                &Ok(()),
            )
            .unwrap();

            let refund_based_fee = prev_balance - Balances::free_balance(&alice);
            let actual_fee =
                TransactionPayment::compute_actual_fee(len as u32, &info, &post_info, tip);

            // 33 weight, 10 length, 7 base, 5 tip
            assert_eq!(actual_fee, 30_999);
            assert_eq!(refund_based_fee, actual_fee);
        });
}

#[test]
fn normal_tx_with_tip() {
    ExtBuilder::default()
        .monied(true)
        .build()
        .execute_with(normal_tx_with_tip_ext);
}

fn normal_tx_with_tip_ext() {
    let len = 10;
    let tip = 42;
    let user = Sr25519Keyring::Alice.to_account_id();
    let alice_origin = RuntimeOrigin::signed(user.clone());
    let call = call();
    let normal_info = info_from_weight(100);

    // Invalid normal tx with tip.
    let expected_err = TransactionValidityError::Invalid(InvalidTransaction::Custom(
        TransactionError::ZeroTip as u8,
    ));
    let pre_err = ChargeTransactionPayment::<TestStorage>::from(tip)
        .validate(
            alice_origin.clone(),
            &call,
            &normal_info,
            len,
            Default::default(),
            &TxBaseImplication(()),
            TransactionSource::InBlock,
        )
        .unwrap_err();
    assert!(pre_err == expected_err);

    // Valid normal tx.
    assert!(ChargeTransactionPayment::<TestStorage>::from(0)
        .prepare(Val::NoCharge, &alice_origin, &call, &normal_info, len)
        .is_ok());
}

#[test]
fn operational_tx_with_tip() {
    let did_registrar = Sr25519Keyring::Bob.to_account_id();
    let gc_member = Sr25519Keyring::Charlie.to_account_id();

    ExtBuilder::default()
        .monied(true)
        .did_registrars(vec![did_registrar.clone()])
        .governance_committee(vec![gc_member.clone()])
        .build()
        .execute_with(|| operational_tx_with_tip_ext(did_registrar, gc_member));
}

fn operational_tx_with_tip_ext(registrar: AccountId, gc: AccountId) {
    let len = 10;
    let tip = 42;
    let user = Sr25519Keyring::Alice.to_account_id();
    let alice_origin = RuntimeOrigin::signed(user.clone());
    let call = call();
    let operational_info = operational_info_from_weight(100);

    // Valid operational tx with `tip == 0`.
    assert!(ChargeTransactionPayment::<TestStorage>::from(0)
        .prepare(Val::NoCharge, &alice_origin, &call, &operational_info, len)
        .is_ok());

    // Valid operational tx with tip. Only DID registrars and Governance members can tip.
    assert!(ChargeTransactionPayment::<TestStorage>::from(tip)
        .validate(
            alice_origin.clone(),
            &call,
            &operational_info,
            len,
            Default::default(),
            &TxBaseImplication(()),
            TransactionSource::InBlock,
        )
        .is_err());

    // Governance can tip.
    let gc_origin = RuntimeOrigin::signed(gc.clone());

    let val = ChargeTransactionPayment::<TestStorage>::from(tip)
        .validate(
            gc_origin.clone(),
            &call,
            &operational_info,
            len,
            Default::default(),
            &TxBaseImplication(()),
            TransactionSource::InBlock,
        )
        .unwrap();

    assert!(ChargeTransactionPayment::<TestStorage>::from(tip)
        .prepare(val.1, &gc_origin, &call, &operational_info, len)
        .is_ok());

    // DID registrar can also tip.
    let registrar_origin = RuntimeOrigin::signed(registrar.clone());

    let val = ChargeTransactionPayment::<TestStorage>::from(tip)
        .validate(
            registrar_origin.clone(),
            &call,
            &operational_info,
            len,
            Default::default(),
            &TxBaseImplication(()),
            TransactionSource::InBlock,
        )
        .unwrap();

    assert!(ChargeTransactionPayment::<TestStorage>::from(tip)
        .prepare(val.1, &registrar_origin, &call, &operational_info, len)
        .is_ok());
}

#[test]
fn validation_should_reject_duplicate_votes() {
    ExtBuilder::default()
        .monied(true)
        .transaction_fees(5, 1, 1)
        .build()
        .execute_with(|| {
            let alice = User::new(Sr25519Keyring::Alice);
            let bob = Sr25519Keyring::Bob.to_account_id();
            let charlie = Sr25519Keyring::Charlie.to_account_id();

            // Create a multisig with Bob and Charlie as signers, and Alice as the creator.
            let ms_address = create_multisig_default_perms(
                alice.acc(),
                create_signers(vec![bob.clone(), charlie.clone()]),
                2,
            );
            let bob_auth_id = get_last_auth_id(&Signatory::Account(bob.clone()));
            let charlie_auth_id = get_last_auth_id(&Signatory::Account(charlie.clone()));
            assert_ok!(MultiSig::accept_multisig_signer(
                Origin::signed(bob.clone()),
                bob_auth_id
            ));
            assert_ok!(MultiSig::accept_multisig_signer(
                Origin::signed(charlie.clone()),
                charlie_auth_id,
            ));

            assert_ok!(Identity::leave_identity_as_key(Origin::signed(
                ms_address.clone()
            )));
            let ms_signatory = Signatory::Account(ms_address.clone());
            let auth_id = Identity::add_auth(
                alice.did,
                ms_signatory.clone(),
                AuthorizationData::JoinIdentity(Permissions::empty()),
                None,
            )
            .unwrap();
            let call = crate::storage::RuntimeCall::MultiSig(
                pallet_multisig::Call::approve_join_identity {
                    multisig: ms_address.clone(),
                    auth_id,
                },
            );
            let info = call.get_dispatch_info();
            let len = 10;

            assert_eq!(
                crate::storage::TxFeeHandler::call_payment_info(&call, bob.clone()),
                Ok(CallPaymentInfo::new(
                    alice.acc(),
                    Some(auth_id),
                    Some(Signatory::Account(ms_address.clone()))
                ))
            );
            assert_ok!(MultiSig::approve_join_identity(
                Origin::signed(bob.clone()),
                ms_address.clone(),
                auth_id,
            ));

            assert_eq!(
                ChargeTransactionPayment::<TestStorage>::from(0)
                    .validate(
                        Origin::signed(bob.clone()),
                        &call,
                        &info,
                        len,
                        Default::default(),
                        &TxBaseImplication(()),
                        TransactionSource::InBlock,
                    )
                    .unwrap_err(),
                TransactionValidityError::Invalid(InvalidTransaction::Custom(5))
            );
        });
}

#[test]
fn post_dispatch_decreases_auth_count() {
    ExtBuilder::default()
        .monied(true)
        .transaction_fees(5, 1, 1)
        .build()
        .execute_with(|| {
            let alice = User::new(Sr25519Keyring::Alice);
            let bob = Sr25519Keyring::Bob.to_account_id();
            let charlie = Sr25519Keyring::Charlie.to_account_id();

            // Create a multisig with Bob and Charlie as signers, and Alice as the creator.
            let ms_address = create_multisig_default_perms(
                alice.acc(),
                create_signers(vec![bob.clone(), charlie.clone()]),
                2,
            );
            let bob_auth_id = get_last_auth_id(&Signatory::Account(bob.clone()));
            let charlie_auth_id = get_last_auth_id(&Signatory::Account(charlie.clone()));
            assert_ok!(MultiSig::accept_multisig_signer(
                Origin::signed(bob.clone()),
                bob_auth_id
            ));
            assert_ok!(MultiSig::accept_multisig_signer(
                Origin::signed(charlie.clone()),
                charlie_auth_id,
            ));

            assert_ok!(Identity::leave_identity_as_key(Origin::signed(
                ms_address.clone()
            )));
            let ms_signatory = Signatory::Account(ms_address.clone());
            let auth_id = Identity::add_auth(
                alice.did,
                ms_signatory.clone(),
                AuthorizationData::JoinIdentity(Permissions::empty()),
                None,
            )
            .unwrap();
            let call = crate::storage::RuntimeCall::MultiSig(
                pallet_multisig::Call::approve_join_identity {
                    multisig: ms_address.clone(),
                    auth_id,
                },
            );
            let info = call.get_dispatch_info();
            let len = 10;

            assert_eq!(
                crate::storage::TxFeeHandler::call_payment_info(&call, bob.clone()),
                Ok(CallPaymentInfo::new(
                    alice.acc(),
                    Some(auth_id),
                    Some(Signatory::Account(ms_address.clone()))
                ))
            );
            assert_ok!(MultiSig::approve_join_identity(
                Origin::signed(bob.clone()),
                ms_address.clone(),
                auth_id,
            ));

            let auth_before = Authorizations::<TestStorage>::get(&ms_signatory, auth_id).unwrap();
            let alice_before = Balances::free_balance(&alice.acc());
            // After validation and preparation, Alice should be charged
            let val = ChargeTransactionPayment::<TestStorage>::from(0)
                .validate(
                    Origin::signed(charlie.clone()),
                    &call,
                    &info,
                    len,
                    Default::default(),
                    &TxBaseImplication(()),
                    TransactionSource::InBlock,
                )
                .unwrap();
            let pre = ChargeTransactionPayment::<TestStorage>::from(0)
                .prepare(val.1, &Origin::signed(bob.clone()), &call, &info, len)
                .unwrap();
            assert!(alice_before > Balances::free_balance(&alice.acc()));
            // Forces an error
            let proposal_id =
                pallet_multisig::AuthToProposalId::<TestStorage>::get(&ms_address, &auth_id)
                    .unwrap();
            ProposalStates::<TestStorage>::insert(
                &ms_address,
                &proposal_id,
                ProposalState::Rejected,
            );
            // If dispatch fails, Alice should be refunded and the authorization count should decrease by 1
            let (mut post_info, dispatch_result) =
                match call.clone().dispatch(Origin::signed(charlie.clone())) {
                    Ok(info) => (info, Ok(())),
                    Err(err) => (err.post_info, Err(err.error)),
                };
            assert_eq!(
                dispatch_result,
                Err(MultisigError::ProposalAlreadyRejected.into())
            );
            assert!(ChargeTransactionPayment::<TestStorage>::post_dispatch(
                pre,
                &info,
                &mut post_info,
                len,
                &dispatch_result,
            )
            .is_ok());
            let auth_after_retry =
                Authorizations::<TestStorage>::get(&ms_signatory, auth_id).unwrap();
            assert_eq!(auth_before.count, auth_after_retry.count + 1);
        });
}
