use super::{
    storage::{TestStorage, User},
    ExtBuilder,
};
use codec::Encode;
use frame_support::{assert_noop, assert_ok};
use frame_system::RawOrigin;
use polymesh_common_utilities::constants::currency::POLY;
use polymesh_runtime_common::Currency;
use test_client::AccountKeyring;

type Error = pallet_rewards::Error<TestStorage>;
type Rewards = pallet_rewards::Module<TestStorage>;
type ItnRewards = pallet_rewards::ItnRewards<TestStorage>;
type Balances = pallet_balances::Module<TestStorage>;

#[test]
fn basic_itn_claim_ext() {
    ExtBuilder::default()
        .monied(true)
        .set_itn_rewards(vec![(AccountKeyring::Bob.to_account_id(), 1 * POLY)])
        .build()
        .execute_with(basic_itn_claim);
}

fn basic_itn_claim() {
    let alice = User::new(AccountKeyring::Alice);
    let bob = User::new(AccountKeyring::Bob);
    let alice_init_balance = Balances::free_balance(alice.acc());
    let bob_init_balance = Balances::free_balance(bob.acc());

    let claim_itn_reward = |reward_user: &User, itn_user: &User, sig_user: Option<&User>| {
        let sig_user = sig_user.unwrap_or(itn_user);
        let mut msg = [0u8; 48];
        msg[..32].copy_from_slice(&reward_user.acc().encode());
        msg[32..].copy_from_slice(b"claim_itn_reward");
        Rewards::claim_itn_reward(
            RawOrigin::None.into(),
            reward_user.acc(),
            itn_user.acc(),
            sig_user.ring.sign(&msg).into(),
        )
    };

    // Check that an address not in the table errors.
    assert_noop!(
        claim_itn_reward(&alice, &alice, None),
        Error::UnknownItnAddress
    );

    // Check that claiming a reward fails when itn wallet is empty.
    assert_noop!(
        claim_itn_reward(&alice, &bob, None),
        Error::InsufficientBalance
    );

    // Give itn wallet some balance.
    let _ = Balances::deposit_into_existing(&Rewards::account_id(), 2 * POLY);
    assert_eq!(Rewards::balance(), 2 * POLY);

    // Check sig.
    assert_noop!(
        claim_itn_reward(&alice, &bob, Some(&alice)),
        Error::InvalidSignature
    );

    // Check balances have not changed.
    assert_eq!(Balances::free_balance(alice.acc()), alice_init_balance);
    assert_eq!(Balances::free_balance(bob.acc()), bob_init_balance);

    // Claim reward successfully.
    assert_ok!(claim_itn_reward(&alice, &bob, None));

    // Check balances were updated.
    assert_eq!(
        Balances::free_balance(alice.acc()),
        alice_init_balance + (2 * POLY)
    );
    assert_eq!(Balances::free_balance(bob.acc()), bob_init_balance);

    // Check double claim fails.
    assert_noop!(
        claim_itn_reward(&alice, &bob, None),
        Error::ItnRewardAlreadyClaimed
    );
}
