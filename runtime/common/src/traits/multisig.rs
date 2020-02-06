use polymesh_primitives::Signatory;

use frame_support::dispatch::DispatchResult;

/// This trait is used to add a signer to a multisig
pub trait AddSignerMultiSig {
    /// Accept and add a multisig signer
    ///
    /// # Arguments
    /// * `signer` did/key of the signer
    /// * `auth_id` Authorization id of the authorization created by the multisig
    fn accept_multisig_signer(signer: Signatory, auth_id: u64) -> DispatchResult;
}
