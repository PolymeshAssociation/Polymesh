use crate::{GenesisConfig, Trait};

use frame_support::{debug, storage::StorageDoubleMap};
use polymesh_common_utilities::Context;
use polymesh_primitives::{Permissions, Signatory};
use sp_std::convert::TryFrom;

type Identity<T> = pallet_identity::Module<T>;

pub(crate) fn do_controller_genesis<T: Trait>(config: &GenesisConfig<T>) -> T::AccountId {
    if config.signatures_required > u64::try_from(config.signers.len()).unwrap_or_default() {
        panic!("too many signatures required");
    }

    if config.signatures_required == 0 {
        // Default to the empty signer set.
        return Default::default();
    }

    let multisig_id = pallet_multisig::Module::<T>::create_multisig_account(
        config.creator.clone(),
        config.signers.as_slice(),
        config.signatures_required,
    )
    .expect("cannot create the bridge multisig");
    debug::info!("Created bridge multisig {}", multisig_id);

    for signer in &config.signers {
        debug::info!("Accepting bridge signer auth for {:?}", signer);
        let last_auth = <pallet_identity::Authorizations<T>>::iter_prefix_values(signer)
            .next()
            .expect("cannot find bridge signer auth")
            .auth_id;
        <pallet_multisig::Module<T>>::unsafe_accept_multisig_signer(signer.clone(), last_auth)
            .expect("cannot accept bridge signer auth");
    }

    let creator_did = Context::current_identity_or::<Identity<T>>(&config.creator)
        .expect("bridge creator account has no identity");

    Identity::<T>::unsafe_join_identity(
        creator_did,
        Permissions::default(),
        Signatory::Account(multisig_id.clone()),
    )
    .expect("cannot link the bridge multisig");
    debug::info!("Joined identity {} as signer {}", creator_did, multisig_id);

    multisig_id
}
