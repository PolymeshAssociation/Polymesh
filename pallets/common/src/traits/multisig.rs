//! # Trait Interface to the Multisig Module
//!
//! The interface allows to process addition of a multisig signer from modules other than the
//! multisig module itself.

use polymesh_primitives::Signatory;

use frame_support::dispatch::DispatchResult;

/// This trait is used to add a signer to a multisig.
pub trait AddSignerMultiSig {
    /// Accepts and adds a multisig signer.
    ///
    /// # Arguments
    /// * `signer` - DID/key of the signer
    /// * `auth_id` - Authorization ID of the authorization created by the multisig.
    fn accept_multisig_signer(signer: Signatory, auth_id: u64) -> DispatchResult;
}
