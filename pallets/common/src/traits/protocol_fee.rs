use codec::{Decode, Encode};
use frame_support::dispatch::DispatchResult;
use polymesh_primitives::Signatory;
#[cfg(feature = "std")]
use sp_runtime::{Deserialize, Serialize};
use sp_std::{fmt::Debug, prelude::*};

/// A wrapper for the name of a chargeable operation.
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
#[derive(Decode, Encode, Clone, Debug, Default, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct OperationName(pub Vec<u8>);

impl<T: AsRef<[u8]>> From<T> for OperationName {
    fn from(s: T) -> Self {
        let s = s.as_ref();
        let mut v = Vec::with_capacity(s.len());
        v.extend_from_slice(s);
        OperationName(v)
    }
}

/// Common interface to protocol fees for runtime modules.
pub trait ChargeProtocolFee {
    /// The type of account.
    type AccountId;

    /// Computes the fee of the operation and charges it to the given signatory.
    fn charge_fee(signatory: &Signatory, name: &OperationName) -> DispatchResult;

    /// Computes the fee for `count` similar operations, and charges that fee to the given
    /// signatory.
    fn charge_fee_batch(
        signatory: &Signatory,
        name: &OperationName,
        count: usize,
    ) -> DispatchResult;

    /// Computes the fee of the operation, charges that fee to `signatory`, and pays it out
    /// collectively to `recipients` in equal parts.
    fn charge_fee_equal_parts(
        signatory: &Signatory,
        name: &OperationName,
        recipients: &[Self::AccountId],
    ) -> DispatchResult;
}
