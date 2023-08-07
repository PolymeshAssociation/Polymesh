use super::ext_builder::ExtBuilder;
use super::storage::{RuntimeCall, TestStorage};
use codec::Encode;
use frame_support::{
    dispatch::{DispatchClass, DispatchInfo, GetDispatchInfo, Pays, PostDispatchInfo, Weight},
    traits::Currency,
    weights::WeightToFee,
};
use pallet_balances::Call as BalancesCall;
use pallet_transaction_payment::{ChargeTransactionPayment, Multiplier, RuntimeDispatchInfo};
use polymesh_primitives::AccountId;
use polymesh_primitives::TransactionError;
use sp_arithmetic::traits::One;
use sp_keyring::AccountKeyring;
use sp_runtime::{
    testing::TestXt,
    traits::SignedExtension,
    transaction_validity::{InvalidTransaction, TransactionValidityError},
    FixedPointNumber, MultiAddress,
};

fn call() -> <TestStorage as frame_system::Config>::RuntimeCall {
    RuntimeCall::Balances(BalancesCall::transfer {
        dest: MultiAddress::Id(AccountKeyring::Alice.to_account_id()),
        value: 69,
    })
}

type Balances = pallet_balances::Module<TestStorage>;
type System = frame_system::Pallet<TestStorage>;
type TransactionPayment = pallet_transaction_payment::Module<TestStorage>;

/// create a transaction info struct from weight. Handy to avoid building the whole struct.
pub fn info_from_weight(w: u64) -> DispatchInfo {
    // pays_fee: Pays::Yes -- class: DispatchClass::Normal
    DispatchInfo {
        weight: Weight::from_ref_time(w),
        ..Default::default()
    }
}

fn weight_to_fee(weight: Weight) -> u128 {
    <TestStorage as pallet_transaction_payment::Config>::WeightToFee::weight_to_fee(&weight)
}

fn operational_info_from_weight(w: u64) -> DispatchInfo {
    DispatchInfo {
        weight: Weight::from_ref_time(w),
        class: DispatchClass::Operational,
        ..Default::default()
    }
}

fn post_info_from_weight(w: u64) -> PostDispatchInfo {
    PostDispatchInfo {
        actual_weight: Some(Weight::from_ref_time(w)),
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
            let bob = AccountKeyring::Bob.to_account_id();
            let alice = AccountKeyring::Alice.to_account_id();

            let len = 10;
            let pre = ChargeTransactionPayment::<TestStorage>::from(0)
                .pre_dispatch(&bob, &call(), &info_from_weight(5), len)
                .unwrap();
            assert_eq!(Balances::free_balance(&bob), 1999969001);

            assert!(ChargeTransactionPayment::<TestStorage>::post_dispatch(
                Some(pre),
                &info_from_weight(5),
                &default_post_info(),
                len,
                &Ok(())
            )
            .is_ok());
            assert_eq!(Balances::free_balance(&bob), 1999969001);

            let pre = ChargeTransactionPayment::<TestStorage>::from(0 /* tipped */)
                .pre_dispatch(&alice, &call(), &info_from_weight(100), len)
                .unwrap();
            assert_eq!(Balances::free_balance(&alice), 999969001);

            assert!(ChargeTransactionPayment::<TestStorage>::post_dispatch(
                Some(pre),
                &info_from_weight(100),
                &post_info_from_weight(50),
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
            let user = AccountKeyring::Alice.to_account_id();
            let len = 10;
            TransactionPayment::put_next_fee_multiplier(Multiplier::saturating_from_rational(3, 2));

            let pre = ChargeTransactionPayment::<TestStorage>::from(0 /* tipped */)
                .pre_dispatch(&user, &call(), &info_from_weight(100), len)
                .unwrap();
            // 5 base fee, 10 byte fee, 3/2 * 100 weight fee, 5 tip
            assert_eq!(Balances::free_balance(&user), 999969001);

            assert!(ChargeTransactionPayment::<TestStorage>::post_dispatch(
                Some(pre),
                &info_from_weight(100),
                &post_info_from_weight(50),
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
            let user = AccountKeyring::Bob.to_account_id();
            let free_user = Balances::free_balance(&user);

            // Get the current weight settings.
            let weights = <TestStorage as frame_system::Config>::BlockWeights::get();
            let per_byte =
                <TestStorage as pallet_transaction_payment::Config>::TransactionByteFee::get();

            // Calculate maximum transaction fee.
            let base_fee = weight_to_fee(weights.get(DispatchClass::Normal).base_extrinsic);
            let len_fee = per_byte.saturating_mul(10);
            let max_block_fee = weight_to_fee(weights.max_block);
            let max_fee = base_fee + len_fee + max_block_fee;

            // maximum weight possible
            ChargeTransactionPayment::<TestStorage>::from(0)
                .pre_dispatch(&user, &call(), &info_from_weight(u64::MAX), 10)
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
            let user = AccountKeyring::Bob.to_account_id();
            // I ain't have a penny.
            assert_eq!(Balances::free_balance(&user), 0);

            let len = 100;

            // This is a completely free (and thus wholly insecure/DoS-ridden) transaction.
            let operational_transaction = DispatchInfo {
                weight: Weight::from_ref_time(0),
                class: DispatchClass::Operational,
                pays_fee: Pays::No,
            };
            assert!(ChargeTransactionPayment::<TestStorage>::from(0)
                .validate(&user, &call(), &operational_transaction, len)
                .is_ok());

            // like a InsecureFreeNormal
            let free_transaction = DispatchInfo {
                weight: Weight::from_ref_time(0),
                class: DispatchClass::Normal,
                pays_fee: Pays::Yes,
            };
            assert!(ChargeTransactionPayment::<TestStorage>::from(0)
                .validate(&user, &call(), &free_transaction, len)
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
            TransactionPayment::put_next_fee_multiplier(Multiplier::saturating_from_rational(3, 2));
            let len = 10;
            let user = AccountKeyring::Bob.to_account_id();
            assert!(ChargeTransactionPayment::<TestStorage>::from(0) // tipped
                .pre_dispatch(&user, &call(), &info_from_weight(3), len)
                .is_ok());
            assert_eq!(Balances::free_balance(&user), 19999969001);
        })
}

#[test]
fn query_info_works() {
    let origin = 111111;
    let extra = ();
    let xt = TestXt::new(call(), Some((origin, extra)));
    let info = xt.get_dispatch_info();
    let ext = xt.encode();
    let len = ext.len() as u32;
    ExtBuilder::default()
        .monied(true)
        .transaction_fees(5, 1, 2)
        .build()
        .execute_with(|| {
            // all fees should be x1.5
            TransactionPayment::put_next_fee_multiplier(Multiplier::saturating_from_rational(3, 2));

            assert_eq!(
                TransactionPayment::query_info(xt, len),
                RuntimeDispatchInfo {
                    weight: info.weight,
                    class: info.class,
                    partial_fee: 34599
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
            assert_eq!(TransactionPayment::next_fee_multiplier(), Multiplier::one());

            // Tip only, no fees works
            let dispatch_info = DispatchInfo {
                weight: Weight::from_ref_time(0),
                class: DispatchClass::Operational,
                pays_fee: Pays::No,
            };
            assert_eq!(TransactionPayment::compute_fee(0, &dispatch_info, 10), 10);
            // No tip, only base fee works
            let dispatch_info = DispatchInfo {
                weight: Weight::from_ref_time(0),
                class: DispatchClass::Operational,
                pays_fee: Pays::Yes,
            };
            assert_eq!(TransactionPayment::compute_fee(0, &dispatch_info, 0), 29999);
            // Tip + base fee works
            assert_eq!(
                TransactionPayment::compute_fee(0, &dispatch_info, 69),
                30068
            );
            // Len (byte fee) + base fee works
            assert_eq!(
                TransactionPayment::compute_fee(42, &dispatch_info, 0),
                34199
            );
            // Weight fee + base fee works
            let dispatch_info = DispatchInfo {
                weight: Weight::from_ref_time(1000),
                class: DispatchClass::Operational,
                pays_fee: Pays::Yes,
            };
            assert_eq!(TransactionPayment::compute_fee(0, &dispatch_info, 0), 29999);
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
            TransactionPayment::put_next_fee_multiplier(Multiplier::saturating_from_rational(3, 2));
            // Base fee is unaffected by multiplier
            let dispatch_info = DispatchInfo {
                weight: Weight::from_ref_time(0),
                class: DispatchClass::Operational,
                pays_fee: Pays::Yes,
            };
            assert_eq!(TransactionPayment::compute_fee(0, &dispatch_info, 0), 29999);

            // Everything works together :)
            let dispatch_info = DispatchInfo {
                weight: Weight::from_ref_time(123),
                class: DispatchClass::Operational,
                pays_fee: Pays::Yes,
            };
            // 123 weight, 456 length, 100 base
            assert_eq!(
                TransactionPayment::compute_fee(456, &dispatch_info, 789),
                76388,
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
            TransactionPayment::put_next_fee_multiplier(Multiplier::saturating_from_rational(1, 2));

            // Base fee is unaffected by multiplier.
            let dispatch_info = DispatchInfo {
                weight: Weight::from_ref_time(0),
                class: DispatchClass::Operational,
                pays_fee: Pays::Yes,
            };
            assert_eq!(TransactionPayment::compute_fee(0, &dispatch_info, 0), 29999);

            // Everything works together.
            let dispatch_info = DispatchInfo {
                weight: Weight::from_ref_time(123),
                class: DispatchClass::Operational,
                pays_fee: Pays::Yes,
            };
            // 123 weight, 456 length, 100 base
            assert_eq!(
                TransactionPayment::compute_fee(456, &dispatch_info, 789),
                76388,
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
                weight: Weight::MAX,
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
            let user = AccountKeyring::Alice.to_account_id();
            let pre = ChargeTransactionPayment::<TestStorage>::from(0 /* tipped */)
                .pre_dispatch(&user, &call(), &info_from_weight(100), len)
                .unwrap();
            assert_eq!(Balances::free_balance(&user), 999969001);

            ChargeTransactionPayment::<TestStorage>::post_dispatch(
                Some(pre),
                &info_from_weight(100),
                &post_info_from_weight(101),
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
                weight: Weight::from_ref_time(100),
                pays_fee: Pays::No,
                class: DispatchClass::Normal,
            };
            let user = AccountKeyring::Alice.to_account_id();
            let bal_init = Balances::total_balance(&user);
            let pre = ChargeTransactionPayment::<TestStorage>::from(0)
                .pre_dispatch(&user, &call(), &dispatch_info, len)
                .unwrap();
            assert_eq!(Balances::total_balance(&user), bal_init);
            assert!(ChargeTransactionPayment::<TestStorage>::post_dispatch(
                Some(pre),
                &dispatch_info,
                &default_post_info(),
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
            let post_info = post_info_from_weight(33);
            let alice = AccountKeyring::Alice.to_account_id();
            let prev_balance = Balances::free_balance(&alice);
            let len = 10;
            let tip = 0;

            TransactionPayment::put_next_fee_multiplier(Multiplier::saturating_from_rational(5, 4));

            let pre = ChargeTransactionPayment::<TestStorage>::from(tip)
                .pre_dispatch(&alice, &call(), &info, len)
                .unwrap();

            ChargeTransactionPayment::<TestStorage>::post_dispatch(
                Some(pre),
                &info,
                &post_info,
                len,
                &Ok(()),
            )
            .unwrap();

            let refund_based_fee = prev_balance - Balances::free_balance(&alice);
            let actual_fee =
                TransactionPayment::compute_actual_fee(len as u32, &info, &post_info, tip);

            // 33 weight, 10 length, 7 base, 5 tip
            assert_eq!(actual_fee, 30999);
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
    let user = AccountKeyring::Alice.to_account_id();
    let call = call();
    let normal_info = info_from_weight(100);

    // Invalid normal tx with tip.
    let expected_err = TransactionValidityError::Invalid(InvalidTransaction::Custom(
        TransactionError::ZeroTip as u8,
    ));
    let pre_err = ChargeTransactionPayment::<TestStorage>::from(tip)
        .pre_dispatch(&user, &call, &normal_info, len)
        .map(|_| ())
        .unwrap_err();
    assert!(pre_err == expected_err);

    // Valid normal tx.
    assert!(ChargeTransactionPayment::<TestStorage>::from(0)
        .pre_dispatch(&user, &call, &normal_info, len)
        .is_ok());
}

#[test]
fn operational_tx_with_tip() {
    let cdd_provider = AccountKeyring::Bob.to_account_id();
    let gc_member = AccountKeyring::Charlie.to_account_id();

    ExtBuilder::default()
        .monied(true)
        .cdd_providers(vec![cdd_provider.clone()])
        .governance_committee(vec![gc_member.clone()])
        .build()
        .execute_with(|| operational_tx_with_tip_ext(cdd_provider, gc_member));
}

fn operational_tx_with_tip_ext(cdd: AccountId, gc: AccountId) {
    let len = 10;
    let tip = 42;
    let user = AccountKeyring::Alice.to_account_id();
    let call = call();
    let operational_info = operational_info_from_weight(100);

    // Valid operational tx with `tip == 0`.
    assert!(ChargeTransactionPayment::<TestStorage>::from(0)
        .pre_dispatch(&user, &call, &operational_info, len)
        .is_ok());

    // Valid operational tx with tip. Only CDD and Governance members can tip.
    assert!(ChargeTransactionPayment::<TestStorage>::from(tip)
        .pre_dispatch(&user, &call, &operational_info, len)
        .is_err());

    // Governance can tip.
    assert!(ChargeTransactionPayment::<TestStorage>::from(tip)
        .pre_dispatch(&gc, &call, &operational_info, len)
        .is_ok());

    // CDD can also tip.
    assert!(ChargeTransactionPayment::<TestStorage>::from(tip)
        .pre_dispatch(&cdd, &call, &operational_info, len)
        .is_ok());
}
