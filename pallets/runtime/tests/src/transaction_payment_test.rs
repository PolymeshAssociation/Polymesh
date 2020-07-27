use super::ext_builder::{EXTRINSIC_BASE_WEIGHT, TRANSACTION_BYTE_FEE, WEIGHT_TO_FEE};
use super::storage::{Call, TestStorage};
use codec::Encode;
use frame_support::{
    parameter_types,
    traits::Currency,
    weights::{DispatchClass, DispatchInfo, GetDispatchInfo, Pays, PostDispatchInfo, Weight},
};
use pallet_balances::Call as BalancesCall;
use pallet_transaction_payment::{ChargeTransactionPayment, Multiplier, RuntimeDispatchInfo};
use sp_runtime::{testing::TestXt, traits::SignedExtension, FixedPointNumber};
use test_client::AccountKeyring;

fn call() -> <TestStorage as frame_system::Trait>::Call {
    Call::Balances(BalancesCall::transfer(AccountKeyring::Alice.public(), 69))
}

type AccountId = u64;
type Balance = u128;
type Origin = <TestStorage as frame_system::Trait>::Origin;
type Balances = pallet_balances::Module<TestStorage>;
type System = frame_system::Module<TestStorage>;
type TransactionPayment = pallet_transaction_payment::Module<TestStorage>;

parameter_types! {
    pub const MaximumBlockWeight: u64 = 4096;
}

pub struct ExtBuilder {
    balance_factor: u128,
    base_weight: u64,
    byte_fee: u128,
    weight_to_fee: u128,
}

impl Default for ExtBuilder {
    fn default() -> Self {
        Self {
            balance_factor: 1,
            base_weight: 0,
            byte_fee: 1,
            weight_to_fee: 1,
        }
    }
}

impl ExtBuilder {
    pub fn base_weight(mut self, base_weight: u64) -> Self {
        self.base_weight = base_weight;
        self
    }
    pub fn byte_fee(mut self, byte_fee: u128) -> Self {
        self.byte_fee = byte_fee;
        self
    }
    pub fn weight_fee(mut self, weight_to_fee: u128) -> Self {
        self.weight_to_fee = weight_to_fee;
        self
    }
    pub fn balance_factor(mut self, factor: u128) -> Self {
        self.balance_factor = factor;
        self
    }
    fn set_constants(&self) {
        EXTRINSIC_BASE_WEIGHT.with(|v| *v.borrow_mut() = self.base_weight);
        TRANSACTION_BYTE_FEE.with(|v| *v.borrow_mut() = self.byte_fee);
        WEIGHT_TO_FEE.with(|v| *v.borrow_mut() = self.weight_to_fee);
    }
    pub fn build(self) -> sp_io::TestExternalities {
        self.set_constants();
        let mut t = frame_system::GenesisConfig::default()
            .build_storage::<TestStorage>()
            .unwrap();
        pallet_balances::GenesisConfig::<TestStorage> {
            balances: if self.balance_factor > 0 {
                vec![
                    (AccountKeyring::Bob.public(), 10 * self.balance_factor),
                    (AccountKeyring::Alice.public(), 20 * self.balance_factor),
                    (AccountKeyring::Charlie.public(), 30 * self.balance_factor),
                    (AccountKeyring::Dave.public(), 40 * self.balance_factor),
                    (AccountKeyring::Eve.public(), 50 * self.balance_factor),
                    (AccountKeyring::Ferdie.public(), 60 * self.balance_factor),
                ]
            } else {
                vec![]
            },
            identity_balances: vec![],
        }
        .assimilate_storage(&mut t)
        .unwrap();
        t.into()
    }
}

/// create a transaction info struct from weight. Handy to avoid building the whole struct.
pub fn info_from_weight(w: Weight) -> DispatchInfo {
    // pays_fee: Pays::Yes -- class: DispatchClass::Normal
    DispatchInfo {
        weight: w,
        ..Default::default()
    }
}

fn post_info_from_weight(w: Weight) -> PostDispatchInfo {
    PostDispatchInfo {
        actual_weight: Some(w),
    }
}

fn default_post_info() -> PostDispatchInfo {
    PostDispatchInfo {
        actual_weight: None,
    }
}

#[test]
fn signed_extension_transaction_payment_work() {
    ExtBuilder::default()
        .balance_factor(10)
        .base_weight(5)
        .build()
        .execute_with(|| {
            let bob = AccountKeyring::Bob.public();
            let alice = AccountKeyring::Alice.public();

            let len = 10;
            let pre = ChargeTransactionPayment::<TestStorage>::from(0)
                .pre_dispatch(&bob, &call(), &info_from_weight(5), len)
                .unwrap();
            assert_eq!(Balances::free_balance(&bob), 100 - 5 - 5 - 10);

            assert!(ChargeTransactionPayment::<TestStorage>::post_dispatch(
                pre,
                &info_from_weight(5),
                &default_post_info(),
                len,
                &Ok(())
            )
            .is_ok());
            assert_eq!(Balances::free_balance(&bob), 100 - 5 - 5 - 10);

            let pre = ChargeTransactionPayment::<TestStorage>::from(0 /* tipped */)
                .pre_dispatch(&alice, &call(), &info_from_weight(100), len)
                .unwrap();
            assert_eq!(Balances::free_balance(&alice), 200 - 5 - 10 - 100 - 0);

            assert!(ChargeTransactionPayment::<TestStorage>::post_dispatch(
                pre,
                &info_from_weight(100),
                &post_info_from_weight(50),
                len,
                &Ok(())
            )
            .is_ok());
            assert_eq!(Balances::free_balance(&alice), 200 - 5 - 10 - 50 - 0);
        });
}

#[test]
fn signed_extension_transaction_payment_multiplied_refund_works() {
    ExtBuilder::default()
        .balance_factor(10)
        .base_weight(5)
        .build()
        .execute_with(|| {
            let user = AccountKeyring::Alice.public();
            let len = 10;
            TransactionPayment::put_next_fee_multiplier(Multiplier::saturating_from_rational(3, 2));

            let pre = ChargeTransactionPayment::<TestStorage>::from(0 /* tipped */)
                .pre_dispatch(&user, &call(), &info_from_weight(100), len)
                .unwrap();
            // 5 base fee, 10 byte fee, 3/2 * 100 weight fee, 5 tip
            assert_eq!(Balances::free_balance(&user), 200 - 5 - 10 - 150 - 0);

            assert!(ChargeTransactionPayment::<TestStorage>::post_dispatch(
                pre,
                &info_from_weight(100),
                &post_info_from_weight(50),
                len,
                &Ok(())
            )
            .is_ok());
            // 75 (3/2 of the returned 50 units of weight) is refunded
            assert_eq!(Balances::free_balance(&user), 200 - 5 - 10 - 75 - 0);
        });
}

#[test]
fn signed_extension_transaction_payment_is_bounded() {
    ExtBuilder::default()
        .balance_factor(1000)
        .byte_fee(0)
        .build()
        .execute_with(|| {
            let user = AccountKeyring::Bob.public();
            // maximum weight possible
            ChargeTransactionPayment::<TestStorage>::from(0)
                .pre_dispatch(&user, &call(), &info_from_weight(Weight::max_value()), 10)
                .unwrap();
            // fee will be proportional to what is the actual maximum weight in the runtime.
            assert_eq!(
                Balances::free_balance(&user),
                (10000 - <TestStorage as frame_system::Trait>::MaximumBlockWeight::get()) as u128
            );
        });
}

#[test]
fn signed_extension_allows_free_transactions() {
    ExtBuilder::default()
        .base_weight(100)
        .balance_factor(0)
        .build()
        .execute_with(|| {
            let user = AccountKeyring::Bob.public();
            // 1 ain't have a penny.
            assert_eq!(Balances::free_balance(&user), 0);

            let len = 100;

            // This is a completely free (and thus wholly insecure/DoS-ridden) transaction.
            let operational_transaction = DispatchInfo {
                weight: 0,
                class: DispatchClass::Operational,
                pays_fee: Pays::No,
            };
            assert!(ChargeTransactionPayment::<TestStorage>::from(0)
                .validate(&user, &call(), &operational_transaction, len)
                .is_ok());

            // like a InsecureFreeNormal
            let free_transaction = DispatchInfo {
                weight: 0,
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
        .base_weight(5)
        .balance_factor(10)
        .build()
        .execute_with(|| {
            // all fees should be x1.5
            TransactionPayment::put_next_fee_multiplier(Multiplier::saturating_from_rational(3, 2));
            let len = 10;
            let user = AccountKeyring::Bob.public();
            assert!(ChargeTransactionPayment::<TestStorage>::from(0) // tipped
                .pre_dispatch(&user, &call(), &info_from_weight(3), len)
                .is_ok());
            assert_eq!(
                Balances::free_balance(&user),
                100 // original
                    - 0 // tip
                    - 5 // base
                    - 10 // len
                    - (3 * 3 / 2) // adjusted weight
            );
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
        .base_weight(5)
        .weight_fee(2)
        .build()
        .execute_with(|| {
            // all fees should be x1.5
            TransactionPayment::put_next_fee_multiplier(Multiplier::saturating_from_rational(3, 2));

            assert_eq!(
                TransactionPayment::query_info(xt, len),
                RuntimeDispatchInfo {
                    weight: info.weight,
                    class: info.class,
                    partial_fee: 5 * 2 /* base * weight_fee */
                           + len as u128  /* len * 1 */
                           + info.weight.min(MaximumBlockWeight::get()) as u128 * 2 * 3 / 2 /* weight */
                },
            );
        });
}

#[test]
fn compute_fee_works_without_multiplier() {
    ExtBuilder::default()
        .base_weight(100)
        .byte_fee(10)
        .balance_factor(0)
        .build()
        .execute_with(|| {
            // Next fee multiplier is zero
            assert_eq!(TransactionPayment::next_fee_multiplier(), Multiplier::one());

            // Tip only, no fees works
            let dispatch_info = DispatchInfo {
                weight: 0,
                class: DispatchClass::Operational,
                pays_fee: Pays::No,
            };
            assert_eq!(TransactionPayment::compute_fee(0, &dispatch_info, 10), 10);
            // No tip, only base fee works
            let dispatch_info = DispatchInfo {
                weight: 0,
                class: DispatchClass::Operational,
                pays_fee: Pays::Yes,
            };
            assert_eq!(TransactionPayment::compute_fee(0, &dispatch_info, 0), 100);
            // Tip + base fee works
            assert_eq!(TransactionPayment::compute_fee(0, &dispatch_info, 69), 169);
            // Len (byte fee) + base fee works
            assert_eq!(TransactionPayment::compute_fee(42, &dispatch_info, 0), 520);
            // Weight fee + base fee works
            let dispatch_info = DispatchInfo {
                weight: 1000,
                class: DispatchClass::Operational,
                pays_fee: Pays::Yes,
            };
            assert_eq!(TransactionPayment::compute_fee(0, &dispatch_info, 0), 1100);
        });
}

#[test]
fn compute_fee_works_with_multiplier() {
    ExtBuilder::default()
        .base_weight(100)
        .byte_fee(10)
        .balance_factor(0)
        .build()
        .execute_with(|| {
            // Add a next fee multiplier. Fees will be x3/2.
            TransactionPayment::put_next_fee_multiplier(Multiplier::saturating_from_rational(3, 2));
            // Base fee is unaffected by multiplier
            let dispatch_info = DispatchInfo {
                weight: 0,
                class: DispatchClass::Operational,
                pays_fee: Pays::Yes,
            };
            assert_eq!(TransactionPayment::compute_fee(0, &dispatch_info, 0), 100);

            // Everything works together :)
            let dispatch_info = DispatchInfo {
                weight: 123,
                class: DispatchClass::Operational,
                pays_fee: Pays::Yes,
            };
            // 123 weight, 456 length, 100 base
            assert_eq!(
                TransactionPayment::compute_fee(456, &dispatch_info, 789),
                100 + (3 * 123 / 2) + 4560 + 789,
            );
        });
}

#[test]
fn compute_fee_works_with_negative_multiplier() {
    ExtBuilder::default()
        .base_weight(100)
        .byte_fee(10)
        .balance_factor(0)
        .build()
        .execute_with(|| {
            // Add a next fee multiplier. All fees will be x1/2.
            TransactionPayment::put_next_fee_multiplier(Multiplier::saturating_from_rational(1, 2));

            // Base fee is unaffected by multiplier.
            let dispatch_info = DispatchInfo {
                weight: 0,
                class: DispatchClass::Operational,
                pays_fee: Pays::Yes,
            };
            assert_eq!(TransactionPayment::compute_fee(0, &dispatch_info, 0), 100);

            // Everything works together.
            let dispatch_info = DispatchInfo {
                weight: 123,
                class: DispatchClass::Operational,
                pays_fee: Pays::Yes,
            };
            // 123 weight, 456 length, 100 base
            assert_eq!(
                TransactionPayment::compute_fee(456, &dispatch_info, 789),
                100 + (123 / 2) + 4560 + 789,
            );
        });
}

#[test]
fn compute_fee_does_not_overflow() {
    ExtBuilder::default()
        .base_weight(100)
        .byte_fee(10)
        .balance_factor(0)
        .build()
        .execute_with(|| {
            // Overflow is handled
            let dispatch_info = DispatchInfo {
                weight: Weight::max_value(),
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
        .balance_factor(10)
        .base_weight(5)
        .build()
        .execute_with(|| {
            let len = 10;
            let user = AccountKeyring::Alice.public();
            let pre = ChargeTransactionPayment::<TestStorage>::from(0 /* tipped */)
                .pre_dispatch(&user, &call(), &info_from_weight(100), len)
                .unwrap();
            assert_eq!(Balances::free_balance(&user), 200 - 0 - 10 - 100 - 5);

            assert!(ChargeTransactionPayment::<TestStorage>::post_dispatch(
                pre,
                &info_from_weight(100),
                &post_info_from_weight(101),
                len,
                &Ok(())
            )
            .is_ok());
            assert_eq!(Balances::free_balance(&user), 200 - 0 - 10 - 100 - 5);
        });
}

#[test]
fn zero_transfer_on_free_transaction() {
    ExtBuilder::default()
        .balance_factor(10)
        .base_weight(5)
        .build()
        .execute_with(|| {
            // So events are emitted
            System::set_block_number(10);
            let len = 10;
            let dispatch_info = DispatchInfo {
                weight: 100,
                pays_fee: Pays::No,
                class: DispatchClass::Normal,
            };
            let user = AccountKeyring::Alice.public();
            let bal_init = Balances::total_balance(&user);
            let pre = ChargeTransactionPayment::<TestStorage>::from(0)
                .pre_dispatch(&user, &call(), &dispatch_info, len)
                .unwrap();
            assert_eq!(Balances::total_balance(&user), bal_init);
            assert!(ChargeTransactionPayment::<TestStorage>::post_dispatch(
                pre,
                &dispatch_info,
                &default_post_info(),
                len,
                &Ok(())
            )
            .is_ok());
            assert_eq!(Balances::total_balance(&user), bal_init);
            // No events for such a scenario
            assert_eq!(System::events().len(), 0);
        });
}

#[test]
fn refund_consistent_with_actual_weight() {
    ExtBuilder::default()
        .balance_factor(10)
        .base_weight(7)
        .build()
        .execute_with(|| {
            let info = info_from_weight(100);
            let post_info = post_info_from_weight(33);
            let alice = AccountKeyring::Alice.public();
            let prev_balance = Balances::free_balance(&alice);
            let len = 10;
            let tip = 0;

            TransactionPayment::put_next_fee_multiplier(Multiplier::saturating_from_rational(5, 4));

            let pre = ChargeTransactionPayment::<TestStorage>::from(tip)
                .pre_dispatch(&alice, &call(), &info, len)
                .unwrap();

            ChargeTransactionPayment::<TestStorage>::post_dispatch(
                pre,
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
            assert_eq!(actual_fee, 7 + 10 + (33 * 5 / 4) + tip);
            assert_eq!(refund_based_fee, actual_fee);
        });
}
