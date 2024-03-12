#[cfg(feature = "std")]
use sp_runtime::{Deserialize, Serialize};

use codec::{Decode, Encode, HasCompact};
use sp_runtime::RuntimeDebug;
use scale_info::TypeInfo;

use crate::{Exposure, ValidatorIndex, NominatorIndex};

/// Preference of an identity regarding validation.
#[derive(Clone, Copy, Decode, Encode, Eq, PartialEq, RuntimeDebug, TypeInfo)]
pub struct PermissionedIdentityPrefs {
    /// Intended number of validators an identity wants to run.
    ///
    /// Act as a hard limit on the number of validators an identity can run.
    /// However, it can be amended using governance.
    ///
    /// The count satisfies `count < MaxValidatorPerIdentity * Self::validator_count()`.
    pub intended_count: u32,
    /// Keeps track of the running number of validators of a DID.
    pub running_count: u32,
}

impl Default for PermissionedIdentityPrefs {
    fn default() -> Self {
        Self { 
            intended_count: 1, 
            running_count: 0 
        }
    }
}

impl PermissionedIdentityPrefs {
    pub fn new(intended_count: u32) -> Self {
        Self { 
            intended_count, 
            running_count: 0 
        }
    }
}

/// Switch used to change the "victim" for slashing. Victims can be
/// validators, both validators and nominators, or no-one.
#[derive(Clone, Copy, Decode, Default, Encode, Eq, PartialEq, RuntimeDebug, TypeInfo)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub enum SlashingSwitch {
    /// Allow validators but not nominators to get slashed.
    Validator,
    /// Allow both validators and nominators to get slashed.
    ValidatorAndNominator,
    /// Forbid slashing.
    #[default]
    None,
}

/// Indicate how an election round was computed.
#[derive(Clone, Copy, Decode, Encode, Eq, PartialEq, RuntimeDebug, TypeInfo)]
pub enum ElectionCompute {
    /// Result was forcefully computed on chain at the end of the session.
    OnChain,
    /// Result was submitted and accepted to the chain via a signed transaction.
    Signed,
    /// Result was submitted and accepted to the chain via an unsigned transaction (by an
    /// authority).
    Unsigned,
}

/// The result of an election round.
#[derive(PartialEq, Eq, Clone, Encode, Decode, TypeInfo, RuntimeDebug)]
pub struct ElectionResult<AccountId, Balance: HasCompact> {
    /// Flat list of validators who have been elected.
    elected_stashes: Vec<AccountId>,
    /// Flat list of new exposures, to be updated in the [`Exposure`] storage.
    exposures: Vec<(AccountId, Exposure<AccountId, Balance>)>,
    /// Type of the result. This is kept on chain only to track and report the best score's
    /// submission type. An optimisation could remove this.
    pub compute: ElectionCompute,
}

/// The status of the upcoming (offchain) election.
#[derive(PartialEq, Eq, Clone, Encode, Decode, TypeInfo, RuntimeDebug)]
pub enum ElectionStatus<BlockNumber> {
    /// Nothing has and will happen for now. submission window is not open.
    Closed,
    /// The submission window has been open since the contained block number.
    Open(BlockNumber),
}

/// Some indications about the size of the election. This must be submitted with the solution.
///
/// Note that these values must reflect the __total__ number, not only those that are present in the
/// solution. In short, these should be the same size as the size of the values dumped in
/// `SnapshotValidators` and `SnapshotNominators`.
#[derive(PartialEq, Eq, Clone, Copy, Encode, Decode, TypeInfo, RuntimeDebug, Default)]
pub struct ElectionSize {
    /// Number of validators in the snapshot of the current election round.
    #[codec(compact)]
    pub validators: ValidatorIndex,
    /// Number of nominators in the snapshot of the current election round.
    #[codec(compact)]
    pub nominators: NominatorIndex,
}


impl<BlockNumber: PartialEq> ElectionStatus<BlockNumber> {
    pub fn is_open_at(&self, n: BlockNumber) -> bool {
        *self == Self::Open(n)
    }

    pub fn is_closed(&self) -> bool {
        match self {
            Self::Closed => true,
            _ => false
        }
    }

    pub fn is_open(&self) -> bool {
        !self.is_closed()
    }
}

impl<BlockNumber> Default for ElectionStatus<BlockNumber> {
    fn default() -> Self {
        Self::Closed
    }
}

// A value placed in storage that represents the current version of the Staking storage. This value
// is used by the `on_runtime_upgrade` logic to determine whether we run storage migration logic.
// This should match directly with the semantic versions of the Rust crate.
#[derive(Clone, Copy, Decode, Default, Encode, Eq, PartialEq, RuntimeDebug, TypeInfo)]
enum Releases {
    V1_0_0Ancient,
    V2_0_0,
    V3_0_0,
    V4_0_0,
    V5_0_0,
    V6_0_0,
    #[default]
    V6_0_1,
    V7_0_0,
}
