use frame_support::dispatch::{DispatchClass, DispatchInfo};
use frame_support::weights::Weight;
use frame_system::{CheckEra, CheckGenesis, CheckNonce};
use frame_system::{CheckSpecVersion, CheckTxVersion, CheckWeight};
use sp_io::TestExternalities;
use sp_keyring::Sr25519Keyring;
use sp_runtime::traits::{TransactionExtension, TxBaseImplication};
use sp_runtime::transaction_validity::TransactionSource;
use sp_runtime::{generic, BuildStorage};
use sp_std::convert::From;

use pallet_group as group;
use pallet_identity as identity;
use polymesh_primitives::{identity_id::GenesisIdentityRecord, AccountId, IdentityId, Nonce};
use polymesh_runtime_develop::runtime::{RuntimeCall, TxExtension};
use polymesh_runtime_develop::Runtime;

type RuntimeOrigin = <Runtime as frame_system::Config>::RuntimeOrigin;

pub fn make_call() -> (<Runtime as frame_system::Config>::RuntimeCall, usize) {
    (
        RuntimeCall::System(frame_system::Call::remark { remark: vec![] }),
        10,
    )
}

/// Generate a `TxExtension` value as it is defined in `Runtime`.
/// It ensures that `Runtime` is using:
///     - Transaction `priority` == `tip`.
///     - Only `Operational` transactions could have `tip` != 0.
///     - `Normal` transactions have `priority` == 0, as `tip` == 0.
fn make_signed_extra(current_block: u64, period: u64, nonce: Nonce, tip: u128) -> TxExtension {
    (
        (
            frame_system::AuthorizeCall::new(),
            frame_system::CheckNonZeroSender::new(),
            CheckSpecVersion::<Runtime>::new(),
            CheckTxVersion::<Runtime>::new(),
            CheckGenesis::<Runtime>::new(),
        ),
        CheckEra::<Runtime>::from(generic::Era::mortal(period, current_block)),
        CheckNonce::<Runtime>::from(nonce),
        CheckWeight::<Runtime>::new(),
        polymesh_transaction_payment::ChargeTransactionPayment::<Runtime>::from(tip),
        pallet_permissions::StoreCallMetadata::<Runtime>::new(),
        frame_metadata_hash_extension::CheckMetadataHash::new(false),
        pallet_revive::evm::tx_extension::SetOrigin::default(),
        frame_system::WeightReclaim::<Runtime>::new(),
    )
}

/// Create the minimun storage for `Runtime` to validate transactions.
/// It is needed because almost any call requires an account with DID and a valid CDD claim.
/// In this genesis configuration, we define the following:
///     - `Alice`, `Bob`, and `Charlie` accounts are created with `1_000_000` balance.
///     - Those accounts have DIDs (starting from 0), and auto-generated CDD claims.
///     - Those accounts are added as CDD providers, so auto-generated CDD claims are valid.
fn make_min_storage() -> Result<TestExternalities, String> {
    let accounts = [
        Sr25519Keyring::Alice.to_account_id(),
        Sr25519Keyring::Bob.to_account_id(),
        Sr25519Keyring::Charlie.to_account_id(),
    ];
    let identities = accounts
        .iter()
        .enumerate()
        .map(|(idx, primary_key)| {
            let did = IdentityId::from(idx as u128);
            let issuers = vec![did];

            GenesisIdentityRecord {
                primary_key: Some(primary_key.clone()),
                issuers,
                did,
                ..Default::default()
            }
        })
        .collect::<Vec<_>>();

    let did = identities
        .iter()
        .map(|gen_id| gen_id.did)
        .collect::<Vec<_>>();

    let mut storage = frame_system::GenesisConfig::<Runtime>::default().build_storage()?;

    // Balances
    pallet_balances::GenesisConfig::<Runtime> {
        balances: accounts
            .iter()
            .map(|acc| (acc.clone(), 1_000_000))
            .collect::<Vec<_>>(),
        dev_accounts: None,
    }
    .assimilate_storage(&mut storage)?;

    // Sudo
    pallet_sudo::GenesisConfig::<Runtime> {
        key: Some(accounts[0].clone()),
    }
    .assimilate_storage(&mut storage)?;

    // Identity
    identity::GenesisConfig::<Runtime> {
        identities,
        ..Default::default()
    }
    .assimilate_storage(&mut storage)?;

    // CDD service
    group::GenesisConfig::<Runtime, group::Instance2> {
        active_members_limit: u32::MAX,
        active_members: did,
        ..Default::default()
    }
    .assimilate_storage(&mut storage)?;

    // Governance committee membership: only GC members may tip.
    group::GenesisConfig::<Runtime, group::Instance1> {
        active_members_limit: u32::MAX,
        active_members: vec![IdentityId::from(0u128)],
        ..Default::default()
    }
    .assimilate_storage(&mut storage)?;

    Ok(TestExternalities::new(storage))
}

#[test]
fn normal_tx_ext() -> Result<(), String> {
    make_min_storage()?.execute_with(normal_tx)
}

/// This test ensures the following rules are true for current `runtime::TxExtension`:
///   - Normal transactions can not have a tip.
///   - Priority of any transaction is its own tip.
fn normal_tx() -> Result<(), String> {
    let user = Sr25519Keyring::Alice.to_account_id();
    let alice_origin = RuntimeOrigin::signed(user.clone());
    let (call, len) = make_call();
    let info = DispatchInfo {
        call_weight: Weight::from_parts(100, 0),
        ..Default::default()
    };

    // Normat Tx with tip. Expected an error.
    let sign_extra = make_signed_extra(0, 10, 0, 42u128.into());
    let tx_validity = sign_extra.validate(
        alice_origin.clone(),
        &call,
        &info,
        len,
        Default::default(),
        &TxBaseImplication(()),
        TransactionSource::InBlock,
    );
    assert!(tx_validity.is_err());

    // Normal TX without any tip.
    let sign_extra = make_signed_extra(0, 10, 0, 0u128.into());
    let tx_validity = sign_extra
        .validate(
            alice_origin,
            &call,
            &info,
            len,
            Default::default(),
            &TxBaseImplication(()),
            TransactionSource::InBlock,
        )
        .expect("Tx should be valid");
    assert_eq!(tx_validity.0.priority, 786432);
    Ok(())
}

#[test]
fn operational_tx_ext() -> Result<(), String> {
    make_min_storage()?.execute_with(operational_tx)
}

/// This test double-checks the following statements:
///     - Operational transactions can have tip != 0.
///     - Priority of any transaction is its own tip.
fn operational_tx() -> Result<(), String> {
    let user: AccountId = Sr25519Keyring::Alice.public().into();
    let alice_origin = RuntimeOrigin::signed(user.clone());
    let (call, len) = make_call();
    let info = DispatchInfo {
        call_weight: Weight::from_parts(100, 0),
        class: DispatchClass::Operational,
        ..Default::default()
    };

    // Operational TX with tip.
    let tip = 42u128;
    let sign_extra = make_signed_extra(0, 10, 0, tip.into());
    let tx_validity = sign_extra
        .validate(
            alice_origin.clone(),
            &call,
            &info,
            len,
            Default::default(),
            &TxBaseImplication(()),
            TransactionSource::InBlock,
        )
        .expect("Tx should be valid");
    assert_eq!(tx_validity.0.priority as u128, 162789326848);

    // Operational TX without any tip.
    let sign_extra = make_signed_extra(0, 10, 0, 0u128.into());
    let tx_validity = sign_extra
        .validate(
            alice_origin,
            &call,
            &info,
            len,
            Default::default(),
            &TxBaseImplication(()),
            TransactionSource::InBlock,
        )
        .expect("Tx should be valid");
    assert_eq!(tx_validity.0.priority, 162525085696);
    Ok(())
}
