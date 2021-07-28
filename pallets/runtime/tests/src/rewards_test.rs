use super::{
    storage::{TestStorage, User},
    ExtBuilder,
};
use codec::Encode;
use frame_support::{assert_noop, assert_ok};
use frame_system::RawOrigin;
use polymesh_common_utilities::constants::currency::POLY;
use polymesh_runtime_common::Currency;
use sp_runtime::DispatchError;
use test_client::AccountKeyring;

type Error = pallet_rewards::Error<TestStorage>;
type Rewards = pallet_rewards::Module<TestStorage>;
type ItnRewards = pallet_rewards::ItnRewards<TestStorage>;
type Balances = pallet_balances::Module<TestStorage>;
type Origin = <TestStorage as frame_system::Config>::Origin;
type OffChainSignature =
    <TestStorage as polymesh_common_utilities::traits::identity::Config>::OffChainSignature;

const INSUFFICIENT_BALANCE_ERROR: DispatchError = DispatchError::Module {
    index: 4,
    error: 2,
    message: Some("InsufficientBalance"),
};

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

    let claim_itn_reward_custom_sig =
        |origin: Option<Origin>, reward_user: &User, itn_user: &User, sig: OffChainSignature| {
            Rewards::claim_itn_reward(
                origin.unwrap_or(RawOrigin::None.into()),
                reward_user.acc(),
                itn_user.acc(),
                sig,
            )
        };
    let claim_itn_reward_basic =
        |origin, reward_user: &User, itn_user: &User, sig_user: Option<&User>| {
            let sig_user = sig_user.unwrap_or(itn_user);
            let mut msg = [0u8; 48];
            msg[..32].copy_from_slice(&reward_user.acc().encode());
            msg[32..].copy_from_slice(b"claim_itn_reward");
            claim_itn_reward_custom_sig(
                origin,
                reward_user,
                itn_user,
                sig_user.ring.sign(&msg).into(),
            )
        };

    // Check that providing an origin fails.
    assert_noop!(
        claim_itn_reward_basic(
            Some(RawOrigin::Signed(alice.acc()).into()),
            &alice,
            &alice,
            None
        ),
        DispatchError::BadOrigin
    );

    // Check that an address not in the table errors.
    assert_noop!(
        claim_itn_reward_basic(None, &alice, &alice, None),
        Error::UnknownItnAddress
    );

    // Check that claiming a reward fails when itn wallet is empty.
    assert_noop!(
        claim_itn_reward_basic(None, &alice, &bob, None),
        pallet_balances::Error::<TestStorage>::InsufficientBalance,
    );

    // Give itn rewards wallet some balance.
    let _ = Balances::deposit_into_existing(&Rewards::account_id(), 2 * POLY);
    assert_eq!(Balances::free_balance(Rewards::account_id()), 2 * POLY);

    // Check that, provided a signature from the wrong user, we get an error.
    assert_noop!(
        claim_itn_reward_basic(None, &alice, &bob, Some(&alice)),
        Error::InvalidSignature
    );

    // Check that a valid acc without the suffix fails.
    let sig = bob.ring.sign(&alice.acc().encode());
    assert_noop!(
        claim_itn_reward_custom_sig(None, &alice, &bob, sig.into()),
        Error::InvalidSignature
    );

    // Check that a valid acc with the wrong suffix fails.
    let mut msg = [0u8; 48];
    msg[..32].copy_from_slice(&alice.acc().encode());
    msg[32..].copy_from_slice(b"claim_NOT_reward");
    let sig = bob.ring.sign(&msg);
    assert_noop!(
        claim_itn_reward_custom_sig(None, &alice, &bob, sig.into()),
        Error::InvalidSignature
    );

    // Check balances have not changed.
    assert_eq!(Balances::free_balance(alice.acc()), alice_init_balance);
    assert_eq!(Balances::free_balance(bob.acc()), bob_init_balance);

    // Claim reward successfully.
    assert_ok!(claim_itn_reward_basic(None, &alice, &bob, None));

    // Check balances were updated.
    assert_eq!(
        Balances::free_balance(alice.acc()),
        alice_init_balance + (2 * POLY)
    );
    assert_eq!(Balances::free_balance(bob.acc()), bob_init_balance);
    assert_eq!(Balances::free_balance(Rewards::account_id()), 0);

    // Check double claim fails.
    assert_noop!(
        claim_itn_reward_basic(None, &alice, &bob, None),
        Error::ItnRewardAlreadyClaimed
    );
}
