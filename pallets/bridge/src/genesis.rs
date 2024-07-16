use crate::{BridgeTxDetail, BridgeTxStatus, Config, GenesisConfig};

use frame_support::storage::StorageDoubleMap;
use polymesh_common_utilities::{balances::CheckCdd, constants::currency::POLY, Context};
use polymesh_primitives::{Permissions, Signatory};
use sp_runtime::traits::Zero;
use sp_std::convert::TryFrom;

type Identity<T> = pallet_identity::Module<T>;

pub(crate) fn admin<T: Config>(config: &GenesisConfig<T>) -> Option<T::AccountId> {
    config.admin.clone()
}

pub(crate) fn controller<T: Config>(config: &GenesisConfig<T>) -> Option<T::AccountId> {
    if config.signatures_required > u64::try_from(config.signers.len()).unwrap_or_default() {
        panic!("too many signatures required");
    }

    if config.signatures_required == 0 {
        // Default to the empty signer set.
        return Default::default();
    }

    let creator = config.creator.as_ref().expect("Bridge creator.");

    let multisig_id = pallet_multisig::Pallet::<T>::base_create_multisig(
        creator.clone(),
        config.signers.as_slice(),
        config.signatures_required,
    )
    .expect("cannot create the bridge multisig");
    log::info!("Created bridge multisig {}", multisig_id);

    for signer in &config.signers {
        log::info!("Accepting bridge signer auth for {:?}", signer);
        let last_auth = <pallet_identity::Authorizations<T>>::iter_prefix_values(
            Signatory::Account(signer.clone()),
        )
        .next()
        .expect("cannot find bridge signer auth")
        .auth_id;
        pallet_multisig::Pallet::<T>::base_accept_multisig_signer(signer.clone(), last_auth)
            .expect("cannot accept bridge signer auth");
    }

    let creator_did = Context::current_identity_or::<Identity<T>>(creator)
        .expect("bridge creator account has no identity");

    Identity::<T>::unsafe_join_identity(creator_did, Permissions::default(), multisig_id.clone());
    log::info!("Joined identity {} as signer {}", creator_did, multisig_id);

    Some(multisig_id)
}

pub(crate) fn bridge_tx_details<T: Config>(
    config: &GenesisConfig<T>,
) -> Vec<(T::AccountId, u32, BridgeTxDetail<T::BlockNumber>)> {
    config
        .complete_txs
        .iter()
        .map(|tx| {
            let recipient = tx.recipient.clone();
            let detail = BridgeTxDetail {
                amount: tx.amount,
                status: BridgeTxStatus::Handled,
                execution_block: Zero::zero(),
                tx_hash: tx.tx_hash,
            };
            let recipient_did = T::CddChecker::get_key_cdd_did(&recipient);

            log::info!(
                "Credited Genesis bridge transaction to {:?}(did={:?}) with nonce {} for {:?} POLYX",
                recipient,
                recipient_did,
                tx.nonce,
                tx.amount / POLY
            );

            crate::Module::<T>::issue(&recipient, &tx.amount, recipient_did)
                .expect("Minting failed");
            (recipient, tx.nonce, detail)
        })
        .collect::<Vec<_>>()
}
