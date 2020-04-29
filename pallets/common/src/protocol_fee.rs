use codec::{Decode, Encode};
use frame_support::dispatch::DispatchResult;
use polymesh_primitives::Signatory;
#[cfg(feature = "std")]
use sp_runtime::{Deserialize, Serialize};

/// Protocol fee operations.
#[derive(Decode, Encode, Clone, Copy, Debug, Hash, PartialEq, Eq, PartialOrd, Ord)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub enum ProtocolOp {
    AssetRegisterTicker,
    AssetIssue,
    AssetAddDocument,
    AssetCreateToken,
    DividendNew,
    GeneralTmAddActiveRule,
    IdentityRegisterDid,
    IdentityCddRegisterDid,
    IdentityAddClaim,
    IdentitySetMasterKey,
    IdentityAddSigningItem,
    PipsPropose,
    VotingAddBallot,
}

/// Common interface to protocol fees for runtime modules.
pub trait ChargeProtocolFee<AccountId> {
    /// Computes the fee of the operation and charges it to the given signatory.
    fn charge_fee(signatory: &Signatory, op: ProtocolOp) -> DispatchResult;

    /// Computes the fee for `count` similar operations, and charges that fee to the given
    /// signatory.
    fn charge_fee_batch(signatory: &Signatory, op: ProtocolOp, count: usize) -> DispatchResult;
}
