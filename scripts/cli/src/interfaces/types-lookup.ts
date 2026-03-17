// Auto-generated via `yarn polkadot-types-from-defs`, do not edit
/* eslint-disable */

// import type lookup before we augment - in some environments
// this is required to allow for ambient/previous definitions
import '@polkadot/types/lookup';

import type { BTreeMap, BTreeSet, Bytes, Compact, Enum, Null, Option, Result, Struct, Text, U8aFixed, Vec, bool, u128, u16, u32, u64, u8 } from '@polkadot/types-codec';
import type { ITuple } from '@polkadot/types-codec/types';
import type { AccountId32, Call, H256, H512, MultiAddress, PerU16, Perbill, Percent, Permill } from '@polkadot/types/interfaces/runtime';
import type { Event } from '@polkadot/types/interfaces/system';

declare module '@polkadot/types/lookup' {
  /** @name FrameSystemAccountInfo (3) */
  interface FrameSystemAccountInfo extends Struct {
    readonly nonce: u32;
    readonly consumers: u32;
    readonly providers: u32;
    readonly sufficients: u32;
    readonly data: PalletBalancesAccountData;
  }

  /** @name PalletBalancesAccountData (5) */
  interface PalletBalancesAccountData extends Struct {
    readonly free: u128;
    readonly reserved: u128;
    readonly frozen: u128;
    readonly flags: u128;
  }

  /** @name FrameSupportDispatchPerDispatchClassWeight (9) */
  interface FrameSupportDispatchPerDispatchClassWeight extends Struct {
    readonly normal: SpWeightsWeightV2Weight;
    readonly operational: SpWeightsWeightV2Weight;
    readonly mandatory: SpWeightsWeightV2Weight;
  }

  /** @name SpWeightsWeightV2Weight (10) */
  interface SpWeightsWeightV2Weight extends Struct {
    readonly refTime: Compact<u64>;
    readonly proofSize: Compact<u64>;
  }

  /** @name SpRuntimeDigest (15) */
  interface SpRuntimeDigest extends Struct {
    readonly logs: Vec<SpRuntimeDigestDigestItem>;
  }

  /** @name SpRuntimeDigestDigestItem (17) */
  interface SpRuntimeDigestDigestItem extends Enum {
    readonly isOther: boolean;
    readonly asOther: Bytes;
    readonly isConsensus: boolean;
    readonly asConsensus: ITuple<[U8aFixed, Bytes]>;
    readonly isSeal: boolean;
    readonly asSeal: ITuple<[U8aFixed, Bytes]>;
    readonly isPreRuntime: boolean;
    readonly asPreRuntime: ITuple<[U8aFixed, Bytes]>;
    readonly isRuntimeEnvironmentUpdated: boolean;
    readonly type: 'Other' | 'Consensus' | 'Seal' | 'PreRuntime' | 'RuntimeEnvironmentUpdated';
  }

  /** @name FrameSystemEventRecord (20) */
  interface FrameSystemEventRecord extends Struct {
    readonly phase: FrameSystemPhase;
    readonly event: Event;
    readonly topics: Vec<H256>;
  }

  /** @name FrameSystemEvent (22) */
  interface FrameSystemEvent extends Enum {
    readonly isExtrinsicSuccess: boolean;
    readonly asExtrinsicSuccess: {
      readonly dispatchInfo: FrameSystemDispatchEventInfo;
    } & Struct;
    readonly isExtrinsicFailed: boolean;
    readonly asExtrinsicFailed: {
      readonly dispatchError: SpRuntimeDispatchError;
      readonly dispatchInfo: FrameSystemDispatchEventInfo;
    } & Struct;
    readonly isCodeUpdated: boolean;
    readonly isNewAccount: boolean;
    readonly asNewAccount: {
      readonly account: AccountId32;
    } & Struct;
    readonly isKilledAccount: boolean;
    readonly asKilledAccount: {
      readonly account: AccountId32;
    } & Struct;
    readonly isRemarked: boolean;
    readonly asRemarked: {
      readonly sender: AccountId32;
      readonly hash_: H256;
    } & Struct;
    readonly isUpgradeAuthorized: boolean;
    readonly asUpgradeAuthorized: {
      readonly codeHash: H256;
      readonly checkVersion: bool;
    } & Struct;
    readonly isRejectedInvalidAuthorizedUpgrade: boolean;
    readonly asRejectedInvalidAuthorizedUpgrade: {
      readonly codeHash: H256;
      readonly error: SpRuntimeDispatchError;
    } & Struct;
    readonly type: 'ExtrinsicSuccess' | 'ExtrinsicFailed' | 'CodeUpdated' | 'NewAccount' | 'KilledAccount' | 'Remarked' | 'UpgradeAuthorized' | 'RejectedInvalidAuthorizedUpgrade';
  }

  /** @name FrameSystemDispatchEventInfo (23) */
  interface FrameSystemDispatchEventInfo extends Struct {
    readonly weight: SpWeightsWeightV2Weight;
    readonly class: FrameSupportDispatchDispatchClass;
    readonly paysFee: FrameSupportDispatchPays;
  }

  /** @name FrameSupportDispatchDispatchClass (24) */
  interface FrameSupportDispatchDispatchClass extends Enum {
    readonly isNormal: boolean;
    readonly isOperational: boolean;
    readonly isMandatory: boolean;
    readonly type: 'Normal' | 'Operational' | 'Mandatory';
  }

  /** @name FrameSupportDispatchPays (25) */
  interface FrameSupportDispatchPays extends Enum {
    readonly isYes: boolean;
    readonly isNo: boolean;
    readonly type: 'Yes' | 'No';
  }

  /** @name SpRuntimeDispatchError (26) */
  interface SpRuntimeDispatchError extends Enum {
    readonly isOther: boolean;
    readonly isCannotLookup: boolean;
    readonly isBadOrigin: boolean;
    readonly isModule: boolean;
    readonly asModule: SpRuntimeModuleError;
    readonly isConsumerRemaining: boolean;
    readonly isNoProviders: boolean;
    readonly isTooManyConsumers: boolean;
    readonly isToken: boolean;
    readonly asToken: SpRuntimeTokenError;
    readonly isArithmetic: boolean;
    readonly asArithmetic: SpArithmeticArithmeticError;
    readonly isTransactional: boolean;
    readonly asTransactional: SpRuntimeTransactionalError;
    readonly isExhausted: boolean;
    readonly isCorruption: boolean;
    readonly isUnavailable: boolean;
    readonly isRootNotAllowed: boolean;
    readonly isTrie: boolean;
    readonly asTrie: SpRuntimeProvingTrieTrieError;
    readonly type: 'Other' | 'CannotLookup' | 'BadOrigin' | 'Module' | 'ConsumerRemaining' | 'NoProviders' | 'TooManyConsumers' | 'Token' | 'Arithmetic' | 'Transactional' | 'Exhausted' | 'Corruption' | 'Unavailable' | 'RootNotAllowed' | 'Trie';
  }

  /** @name SpRuntimeModuleError (27) */
  interface SpRuntimeModuleError extends Struct {
    readonly index: u8;
    readonly error: U8aFixed;
  }

  /** @name SpRuntimeTokenError (28) */
  interface SpRuntimeTokenError extends Enum {
    readonly isFundsUnavailable: boolean;
    readonly isOnlyProvider: boolean;
    readonly isBelowMinimum: boolean;
    readonly isCannotCreate: boolean;
    readonly isUnknownAsset: boolean;
    readonly isFrozen: boolean;
    readonly isUnsupported: boolean;
    readonly isCannotCreateHold: boolean;
    readonly isNotExpendable: boolean;
    readonly isBlocked: boolean;
    readonly type: 'FundsUnavailable' | 'OnlyProvider' | 'BelowMinimum' | 'CannotCreate' | 'UnknownAsset' | 'Frozen' | 'Unsupported' | 'CannotCreateHold' | 'NotExpendable' | 'Blocked';
  }

  /** @name SpArithmeticArithmeticError (29) */
  interface SpArithmeticArithmeticError extends Enum {
    readonly isUnderflow: boolean;
    readonly isOverflow: boolean;
    readonly isDivisionByZero: boolean;
    readonly type: 'Underflow' | 'Overflow' | 'DivisionByZero';
  }

  /** @name SpRuntimeTransactionalError (30) */
  interface SpRuntimeTransactionalError extends Enum {
    readonly isLimitReached: boolean;
    readonly isNoLayer: boolean;
    readonly type: 'LimitReached' | 'NoLayer';
  }

  /** @name SpRuntimeProvingTrieTrieError (31) */
  interface SpRuntimeProvingTrieTrieError extends Enum {
    readonly isInvalidStateRoot: boolean;
    readonly isIncompleteDatabase: boolean;
    readonly isValueAtIncompleteKey: boolean;
    readonly isDecoderError: boolean;
    readonly isInvalidHash: boolean;
    readonly isDuplicateKey: boolean;
    readonly isExtraneousNode: boolean;
    readonly isExtraneousValue: boolean;
    readonly isExtraneousHashReference: boolean;
    readonly isInvalidChildReference: boolean;
    readonly isValueMismatch: boolean;
    readonly isIncompleteProof: boolean;
    readonly isRootMismatch: boolean;
    readonly isDecodeError: boolean;
    readonly type: 'InvalidStateRoot' | 'IncompleteDatabase' | 'ValueAtIncompleteKey' | 'DecoderError' | 'InvalidHash' | 'DuplicateKey' | 'ExtraneousNode' | 'ExtraneousValue' | 'ExtraneousHashReference' | 'InvalidChildReference' | 'ValueMismatch' | 'IncompleteProof' | 'RootMismatch' | 'DecodeError';
  }

  /** @name PalletIndicesEvent (32) */
  interface PalletIndicesEvent extends Enum {
    readonly isIndexAssigned: boolean;
    readonly asIndexAssigned: {
      readonly who: AccountId32;
      readonly index: u32;
    } & Struct;
    readonly isIndexFreed: boolean;
    readonly asIndexFreed: {
      readonly index: u32;
    } & Struct;
    readonly isIndexFrozen: boolean;
    readonly asIndexFrozen: {
      readonly index: u32;
      readonly who: AccountId32;
    } & Struct;
    readonly isDepositPoked: boolean;
    readonly asDepositPoked: {
      readonly who: AccountId32;
      readonly index: u32;
      readonly oldDeposit: u128;
      readonly newDeposit: u128;
    } & Struct;
    readonly type: 'IndexAssigned' | 'IndexFreed' | 'IndexFrozen' | 'DepositPoked';
  }

  /** @name PalletBalancesEvent (33) */
  interface PalletBalancesEvent extends Enum {
    readonly isEndowed: boolean;
    readonly asEndowed: {
      readonly account: AccountId32;
      readonly freeBalance: u128;
    } & Struct;
    readonly isDustLost: boolean;
    readonly asDustLost: {
      readonly account: AccountId32;
      readonly amount: u128;
    } & Struct;
    readonly isTransfer: boolean;
    readonly asTransfer: {
      readonly from: AccountId32;
      readonly to: AccountId32;
      readonly amount: u128;
    } & Struct;
    readonly isBalanceSet: boolean;
    readonly asBalanceSet: {
      readonly who: AccountId32;
      readonly free: u128;
    } & Struct;
    readonly isReserved: boolean;
    readonly asReserved: {
      readonly who: AccountId32;
      readonly amount: u128;
    } & Struct;
    readonly isUnreserved: boolean;
    readonly asUnreserved: {
      readonly who: AccountId32;
      readonly amount: u128;
    } & Struct;
    readonly isReserveRepatriated: boolean;
    readonly asReserveRepatriated: {
      readonly from: AccountId32;
      readonly to: AccountId32;
      readonly amount: u128;
      readonly destinationStatus: FrameSupportTokensMiscBalanceStatus;
    } & Struct;
    readonly isDeposit: boolean;
    readonly asDeposit: {
      readonly who: AccountId32;
      readonly amount: u128;
    } & Struct;
    readonly isWithdraw: boolean;
    readonly asWithdraw: {
      readonly who: AccountId32;
      readonly amount: u128;
    } & Struct;
    readonly isSlashed: boolean;
    readonly asSlashed: {
      readonly who: AccountId32;
      readonly amount: u128;
    } & Struct;
    readonly isMinted: boolean;
    readonly asMinted: {
      readonly who: AccountId32;
      readonly amount: u128;
    } & Struct;
    readonly isMintedCredit: boolean;
    readonly asMintedCredit: {
      readonly amount: u128;
    } & Struct;
    readonly isBurned: boolean;
    readonly asBurned: {
      readonly who: AccountId32;
      readonly amount: u128;
    } & Struct;
    readonly isBurnedDebt: boolean;
    readonly asBurnedDebt: {
      readonly amount: u128;
    } & Struct;
    readonly isSuspended: boolean;
    readonly asSuspended: {
      readonly who: AccountId32;
      readonly amount: u128;
    } & Struct;
    readonly isRestored: boolean;
    readonly asRestored: {
      readonly who: AccountId32;
      readonly amount: u128;
    } & Struct;
    readonly isUpgraded: boolean;
    readonly asUpgraded: {
      readonly who: AccountId32;
    } & Struct;
    readonly isIssued: boolean;
    readonly asIssued: {
      readonly amount: u128;
    } & Struct;
    readonly isRescinded: boolean;
    readonly asRescinded: {
      readonly amount: u128;
    } & Struct;
    readonly isLocked: boolean;
    readonly asLocked: {
      readonly who: AccountId32;
      readonly amount: u128;
    } & Struct;
    readonly isUnlocked: boolean;
    readonly asUnlocked: {
      readonly who: AccountId32;
      readonly amount: u128;
    } & Struct;
    readonly isFrozen: boolean;
    readonly asFrozen: {
      readonly who: AccountId32;
      readonly amount: u128;
    } & Struct;
    readonly isThawed: boolean;
    readonly asThawed: {
      readonly who: AccountId32;
      readonly amount: u128;
    } & Struct;
    readonly isTotalIssuanceForced: boolean;
    readonly asTotalIssuanceForced: {
      readonly old: u128;
      readonly new_: u128;
    } & Struct;
    readonly isHeld: boolean;
    readonly asHeld: {
      readonly reason: PolymeshRuntimeDevelopRuntimeRuntimeHoldReason;
      readonly who: AccountId32;
      readonly amount: u128;
    } & Struct;
    readonly isBurnedHeld: boolean;
    readonly asBurnedHeld: {
      readonly reason: PolymeshRuntimeDevelopRuntimeRuntimeHoldReason;
      readonly who: AccountId32;
      readonly amount: u128;
    } & Struct;
    readonly isTransferOnHold: boolean;
    readonly asTransferOnHold: {
      readonly reason: PolymeshRuntimeDevelopRuntimeRuntimeHoldReason;
      readonly source: AccountId32;
      readonly dest: AccountId32;
      readonly amount: u128;
    } & Struct;
    readonly isTransferAndHold: boolean;
    readonly asTransferAndHold: {
      readonly reason: PolymeshRuntimeDevelopRuntimeRuntimeHoldReason;
      readonly source: AccountId32;
      readonly dest: AccountId32;
      readonly transferred: u128;
    } & Struct;
    readonly isReleased: boolean;
    readonly asReleased: {
      readonly reason: PolymeshRuntimeDevelopRuntimeRuntimeHoldReason;
      readonly who: AccountId32;
      readonly amount: u128;
    } & Struct;
    readonly isUnexpected: boolean;
    readonly asUnexpected: PalletBalancesUnexpectedKind;
    readonly isTransferWithMemo: boolean;
    readonly asTransferWithMemo: {
      readonly from: AccountId32;
      readonly to: AccountId32;
      readonly amount: u128;
      readonly memo: Option<PolymeshPrimitivesMemo>;
    } & Struct;
    readonly type: 'Endowed' | 'DustLost' | 'Transfer' | 'BalanceSet' | 'Reserved' | 'Unreserved' | 'ReserveRepatriated' | 'Deposit' | 'Withdraw' | 'Slashed' | 'Minted' | 'MintedCredit' | 'Burned' | 'BurnedDebt' | 'Suspended' | 'Restored' | 'Upgraded' | 'Issued' | 'Rescinded' | 'Locked' | 'Unlocked' | 'Frozen' | 'Thawed' | 'TotalIssuanceForced' | 'Held' | 'BurnedHeld' | 'TransferOnHold' | 'TransferAndHold' | 'Released' | 'Unexpected' | 'TransferWithMemo';
  }

  /** @name FrameSupportTokensMiscBalanceStatus (34) */
  interface FrameSupportTokensMiscBalanceStatus extends Enum {
    readonly isFree: boolean;
    readonly isReserved: boolean;
    readonly type: 'Free' | 'Reserved';
  }

  /** @name PolymeshRuntimeDevelopRuntimeRuntimeHoldReason (35) */
  interface PolymeshRuntimeDevelopRuntimeRuntimeHoldReason extends Enum {
    readonly isStaking: boolean;
    readonly asStaking: PalletStakingPalletHoldReason;
    readonly isSession: boolean;
    readonly asSession: PalletSessionHoldReason;
    readonly isContracts: boolean;
    readonly asContracts: PalletContractsHoldReason;
    readonly isPreimage: boolean;
    readonly asPreimage: PalletPreimageHoldReason;
    readonly type: 'Staking' | 'Session' | 'Contracts' | 'Preimage';
  }

  /** @name PalletStakingPalletHoldReason (36) */
  interface PalletStakingPalletHoldReason extends Enum {
    readonly isStaking: boolean;
    readonly type: 'Staking';
  }

  /** @name PalletSessionHoldReason (37) */
  interface PalletSessionHoldReason extends Enum {
    readonly isKeys: boolean;
    readonly type: 'Keys';
  }

  /** @name PalletContractsHoldReason (38) */
  interface PalletContractsHoldReason extends Enum {
    readonly isCodeUploadDepositReserve: boolean;
    readonly isStorageDepositReserve: boolean;
    readonly type: 'CodeUploadDepositReserve' | 'StorageDepositReserve';
  }

  /** @name PalletPreimageHoldReason (39) */
  interface PalletPreimageHoldReason extends Enum {
    readonly isPreimage: boolean;
    readonly type: 'Preimage';
  }

  /** @name PalletBalancesUnexpectedKind (40) */
  interface PalletBalancesUnexpectedKind extends Enum {
    readonly isBalanceUpdated: boolean;
    readonly isFailedToMutateAccount: boolean;
    readonly type: 'BalanceUpdated' | 'FailedToMutateAccount';
  }

  /** @name PolymeshPrimitivesMemo (42) */
  interface PolymeshPrimitivesMemo extends U8aFixed {}

  /** @name PalletTransactionPaymentEvent (43) */
  interface PalletTransactionPaymentEvent extends Enum {
    readonly isTransactionFeePaid: boolean;
    readonly asTransactionFeePaid: {
      readonly who: AccountId32;
      readonly actualFee: u128;
      readonly tip: u128;
    } & Struct;
    readonly type: 'TransactionFeePaid';
  }

  /** @name PalletIdentityEvent (44) */
  interface PalletIdentityEvent extends Enum {
    readonly isDidCreated: boolean;
    readonly asDidCreated: ITuple<[PolymeshPrimitivesIdentityId, AccountId32, Vec<PolymeshPrimitivesSecondaryKey>]>;
    readonly isSecondaryKeysAdded: boolean;
    readonly asSecondaryKeysAdded: ITuple<[PolymeshPrimitivesIdentityId, Vec<PolymeshPrimitivesSecondaryKey>]>;
    readonly isSecondaryKeysRemoved: boolean;
    readonly asSecondaryKeysRemoved: ITuple<[PolymeshPrimitivesIdentityId, Vec<AccountId32>]>;
    readonly isSecondaryKeyLeftIdentity: boolean;
    readonly asSecondaryKeyLeftIdentity: ITuple<[PolymeshPrimitivesIdentityId, AccountId32]>;
    readonly isSecondaryKeyPermissionsUpdated: boolean;
    readonly asSecondaryKeyPermissionsUpdated: ITuple<[PolymeshPrimitivesIdentityId, AccountId32, PolymeshPrimitivesSecondaryKeyPermissions, PolymeshPrimitivesSecondaryKeyPermissions]>;
    readonly isPrimaryKeyUpdated: boolean;
    readonly asPrimaryKeyUpdated: ITuple<[PolymeshPrimitivesIdentityId, AccountId32, AccountId32]>;
    readonly isClaimAdded: boolean;
    readonly asClaimAdded: ITuple<[PolymeshPrimitivesIdentityId, PolymeshPrimitivesIdentityClaim]>;
    readonly isClaimRevoked: boolean;
    readonly asClaimRevoked: ITuple<[PolymeshPrimitivesIdentityId, PolymeshPrimitivesIdentityClaim]>;
    readonly isAuthorizationAdded: boolean;
    readonly asAuthorizationAdded: ITuple<[PolymeshPrimitivesIdentityId, Option<PolymeshPrimitivesIdentityId>, Option<AccountId32>, u64, PolymeshPrimitivesAuthorizationAuthorizationData, Option<u64>]>;
    readonly isAuthorizationRevoked: boolean;
    readonly asAuthorizationRevoked: ITuple<[Option<PolymeshPrimitivesIdentityId>, Option<AccountId32>, u64]>;
    readonly isAuthorizationRejected: boolean;
    readonly asAuthorizationRejected: ITuple<[Option<PolymeshPrimitivesIdentityId>, Option<AccountId32>, u64]>;
    readonly isAuthorizationConsumed: boolean;
    readonly asAuthorizationConsumed: ITuple<[Option<PolymeshPrimitivesIdentityId>, Option<AccountId32>, u64]>;
    readonly isAuthorizationRetryLimitReached: boolean;
    readonly asAuthorizationRetryLimitReached: ITuple<[Option<PolymeshPrimitivesIdentityId>, Option<AccountId32>, u64]>;
    readonly isSecondaryKeysFrozen: boolean;
    readonly asSecondaryKeysFrozen: PolymeshPrimitivesIdentityId;
    readonly isSecondaryKeysUnfrozen: boolean;
    readonly asSecondaryKeysUnfrozen: PolymeshPrimitivesIdentityId;
    readonly isCustomClaimTypeAdded: boolean;
    readonly asCustomClaimTypeAdded: ITuple<[PolymeshPrimitivesIdentityId, u32, Bytes]>;
    readonly isChildDidCreated: boolean;
    readonly asChildDidCreated: ITuple<[PolymeshPrimitivesIdentityId, PolymeshPrimitivesIdentityId, AccountId32]>;
    readonly isChildDidUnlinked: boolean;
    readonly asChildDidUnlinked: ITuple<[PolymeshPrimitivesIdentityId, PolymeshPrimitivesIdentityId, PolymeshPrimitivesIdentityId]>;
    readonly type: 'DidCreated' | 'SecondaryKeysAdded' | 'SecondaryKeysRemoved' | 'SecondaryKeyLeftIdentity' | 'SecondaryKeyPermissionsUpdated' | 'PrimaryKeyUpdated' | 'ClaimAdded' | 'ClaimRevoked' | 'AuthorizationAdded' | 'AuthorizationRevoked' | 'AuthorizationRejected' | 'AuthorizationConsumed' | 'AuthorizationRetryLimitReached' | 'SecondaryKeysFrozen' | 'SecondaryKeysUnfrozen' | 'CustomClaimTypeAdded' | 'ChildDidCreated' | 'ChildDidUnlinked';
  }

  /** @name PolymeshPrimitivesIdentityId (45) */
  interface PolymeshPrimitivesIdentityId extends U8aFixed {}

  /** @name PolymeshPrimitivesSecondaryKey (47) */
  interface PolymeshPrimitivesSecondaryKey extends Struct {
    readonly key: AccountId32;
    readonly permissions: PolymeshPrimitivesSecondaryKeyPermissions;
  }

  /** @name PolymeshPrimitivesSecondaryKeyPermissions (48) */
  interface PolymeshPrimitivesSecondaryKeyPermissions extends Struct {
    readonly asset: PolymeshPrimitivesSubsetSubsetRestrictionAssetId;
    readonly extrinsic: PolymeshPrimitivesSecondaryKeyExtrinsicPermissions;
    readonly portfolio: PolymeshPrimitivesSubsetSubsetRestrictionPortfolioId;
  }

  /** @name PolymeshPrimitivesSubsetSubsetRestrictionAssetId (49) */
  interface PolymeshPrimitivesSubsetSubsetRestrictionAssetId extends Enum {
    readonly isWhole: boolean;
    readonly isThese: boolean;
    readonly asThese: BTreeSet<PolymeshPrimitivesAssetAssetId>;
    readonly isExcept: boolean;
    readonly asExcept: BTreeSet<PolymeshPrimitivesAssetAssetId>;
    readonly type: 'Whole' | 'These' | 'Except';
  }

  /** @name PolymeshPrimitivesAssetAssetId (50) */
  interface PolymeshPrimitivesAssetAssetId extends U8aFixed {}

  /** @name PolymeshPrimitivesSecondaryKeyExtrinsicPermissions (54) */
  interface PolymeshPrimitivesSecondaryKeyExtrinsicPermissions extends Enum {
    readonly isWhole: boolean;
    readonly isThese: boolean;
    readonly asThese: BTreeMap<Text, PolymeshPrimitivesSecondaryKeyPalletPermissions>;
    readonly isExcept: boolean;
    readonly asExcept: BTreeMap<Text, PolymeshPrimitivesSecondaryKeyPalletPermissions>;
    readonly type: 'Whole' | 'These' | 'Except';
  }

  /** @name PolymeshPrimitivesSecondaryKeyPalletPermissions (58) */
  interface PolymeshPrimitivesSecondaryKeyPalletPermissions extends Struct {
    readonly extrinsics: PolymeshPrimitivesSubsetSubsetRestrictionExtrinsicName;
  }

  /** @name PolymeshPrimitivesSubsetSubsetRestrictionExtrinsicName (59) */
  interface PolymeshPrimitivesSubsetSubsetRestrictionExtrinsicName extends Enum {
    readonly isWhole: boolean;
    readonly isThese: boolean;
    readonly asThese: BTreeSet<Text>;
    readonly isExcept: boolean;
    readonly asExcept: BTreeSet<Text>;
    readonly type: 'Whole' | 'These' | 'Except';
  }

  /** @name PolymeshPrimitivesSubsetSubsetRestrictionPortfolioId (65) */
  interface PolymeshPrimitivesSubsetSubsetRestrictionPortfolioId extends Enum {
    readonly isWhole: boolean;
    readonly isThese: boolean;
    readonly asThese: BTreeSet<PolymeshPrimitivesIdentityIdPortfolioId>;
    readonly isExcept: boolean;
    readonly asExcept: BTreeSet<PolymeshPrimitivesIdentityIdPortfolioId>;
    readonly type: 'Whole' | 'These' | 'Except';
  }

  /** @name PolymeshPrimitivesIdentityIdPortfolioId (66) */
  interface PolymeshPrimitivesIdentityIdPortfolioId extends Struct {
    readonly did: PolymeshPrimitivesIdentityId;
    readonly kind: PolymeshPrimitivesIdentityIdPortfolioKind;
  }

  /** @name PolymeshPrimitivesIdentityIdPortfolioKind (67) */
  interface PolymeshPrimitivesIdentityIdPortfolioKind extends Enum {
    readonly isDefault: boolean;
    readonly isUser: boolean;
    readonly asUser: u64;
    readonly type: 'Default' | 'User';
  }

  /** @name PolymeshPrimitivesIdentityClaim (72) */
  interface PolymeshPrimitivesIdentityClaim extends Struct {
    readonly claimIssuer: PolymeshPrimitivesIdentityId;
    readonly issuanceDate: u64;
    readonly lastUpdateDate: u64;
    readonly expiry: Option<u64>;
    readonly claim: PolymeshPrimitivesIdentityClaimClaim;
  }

  /** @name PolymeshPrimitivesIdentityClaimClaim (74) */
  interface PolymeshPrimitivesIdentityClaimClaim extends Enum {
    readonly isAccredited: boolean;
    readonly asAccredited: PolymeshPrimitivesIdentityClaimScope;
    readonly isAffiliate: boolean;
    readonly asAffiliate: PolymeshPrimitivesIdentityClaimScope;
    readonly isBuyLockup: boolean;
    readonly asBuyLockup: PolymeshPrimitivesIdentityClaimScope;
    readonly isSellLockup: boolean;
    readonly asSellLockup: PolymeshPrimitivesIdentityClaimScope;
    readonly isCustomerDueDiligence: boolean;
    readonly asCustomerDueDiligence: PolymeshPrimitivesCddId;
    readonly isKnowYourCustomer: boolean;
    readonly asKnowYourCustomer: PolymeshPrimitivesIdentityClaimScope;
    readonly isJurisdiction: boolean;
    readonly asJurisdiction: ITuple<[PolymeshPrimitivesJurisdictionCountryCode, PolymeshPrimitivesIdentityClaimScope]>;
    readonly isExempted: boolean;
    readonly asExempted: PolymeshPrimitivesIdentityClaimScope;
    readonly isBlocked: boolean;
    readonly asBlocked: PolymeshPrimitivesIdentityClaimScope;
    readonly isCustom: boolean;
    readonly asCustom: ITuple<[u32, Option<PolymeshPrimitivesIdentityClaimScope>]>;
    readonly type: 'Accredited' | 'Affiliate' | 'BuyLockup' | 'SellLockup' | 'CustomerDueDiligence' | 'KnowYourCustomer' | 'Jurisdiction' | 'Exempted' | 'Blocked' | 'Custom';
  }

  /** @name PolymeshPrimitivesIdentityClaimScope (75) */
  interface PolymeshPrimitivesIdentityClaimScope extends Enum {
    readonly isIdentity: boolean;
    readonly asIdentity: PolymeshPrimitivesIdentityId;
    readonly isAsset: boolean;
    readonly asAsset: PolymeshPrimitivesAssetAssetId;
    readonly isCustom: boolean;
    readonly asCustom: Bytes;
    readonly type: 'Identity' | 'Asset' | 'Custom';
  }

  /** @name PolymeshPrimitivesCddId (76) */
  interface PolymeshPrimitivesCddId extends U8aFixed {}

  /** @name PolymeshPrimitivesJurisdictionCountryCode (77) */
  interface PolymeshPrimitivesJurisdictionCountryCode extends Enum {
    readonly isAf: boolean;
    readonly isAx: boolean;
    readonly isAl: boolean;
    readonly isDz: boolean;
    readonly isAs: boolean;
    readonly isAd: boolean;
    readonly isAo: boolean;
    readonly isAi: boolean;
    readonly isAq: boolean;
    readonly isAg: boolean;
    readonly isAr: boolean;
    readonly isAm: boolean;
    readonly isAw: boolean;
    readonly isAu: boolean;
    readonly isAt: boolean;
    readonly isAz: boolean;
    readonly isBs: boolean;
    readonly isBh: boolean;
    readonly isBd: boolean;
    readonly isBb: boolean;
    readonly isBy: boolean;
    readonly isBe: boolean;
    readonly isBz: boolean;
    readonly isBj: boolean;
    readonly isBm: boolean;
    readonly isBt: boolean;
    readonly isBo: boolean;
    readonly isBa: boolean;
    readonly isBw: boolean;
    readonly isBv: boolean;
    readonly isBr: boolean;
    readonly isVg: boolean;
    readonly isIo: boolean;
    readonly isBn: boolean;
    readonly isBg: boolean;
    readonly isBf: boolean;
    readonly isBi: boolean;
    readonly isKh: boolean;
    readonly isCm: boolean;
    readonly isCa: boolean;
    readonly isCv: boolean;
    readonly isKy: boolean;
    readonly isCf: boolean;
    readonly isTd: boolean;
    readonly isCl: boolean;
    readonly isCn: boolean;
    readonly isHk: boolean;
    readonly isMo: boolean;
    readonly isCx: boolean;
    readonly isCc: boolean;
    readonly isCo: boolean;
    readonly isKm: boolean;
    readonly isCg: boolean;
    readonly isCd: boolean;
    readonly isCk: boolean;
    readonly isCr: boolean;
    readonly isCi: boolean;
    readonly isHr: boolean;
    readonly isCu: boolean;
    readonly isCy: boolean;
    readonly isCz: boolean;
    readonly isDk: boolean;
    readonly isDj: boolean;
    readonly isDm: boolean;
    readonly isDo: boolean;
    readonly isEc: boolean;
    readonly isEg: boolean;
    readonly isSv: boolean;
    readonly isGq: boolean;
    readonly isEr: boolean;
    readonly isEe: boolean;
    readonly isEt: boolean;
    readonly isFk: boolean;
    readonly isFo: boolean;
    readonly isFj: boolean;
    readonly isFi: boolean;
    readonly isFr: boolean;
    readonly isGf: boolean;
    readonly isPf: boolean;
    readonly isTf: boolean;
    readonly isGa: boolean;
    readonly isGm: boolean;
    readonly isGe: boolean;
    readonly isDe: boolean;
    readonly isGh: boolean;
    readonly isGi: boolean;
    readonly isGr: boolean;
    readonly isGl: boolean;
    readonly isGd: boolean;
    readonly isGp: boolean;
    readonly isGu: boolean;
    readonly isGt: boolean;
    readonly isGg: boolean;
    readonly isGn: boolean;
    readonly isGw: boolean;
    readonly isGy: boolean;
    readonly isHt: boolean;
    readonly isHm: boolean;
    readonly isVa: boolean;
    readonly isHn: boolean;
    readonly isHu: boolean;
    readonly isIs: boolean;
    readonly isIn: boolean;
    readonly isId: boolean;
    readonly isIr: boolean;
    readonly isIq: boolean;
    readonly isIe: boolean;
    readonly isIm: boolean;
    readonly isIl: boolean;
    readonly isIt: boolean;
    readonly isJm: boolean;
    readonly isJp: boolean;
    readonly isJe: boolean;
    readonly isJo: boolean;
    readonly isKz: boolean;
    readonly isKe: boolean;
    readonly isKi: boolean;
    readonly isKp: boolean;
    readonly isKr: boolean;
    readonly isKw: boolean;
    readonly isKg: boolean;
    readonly isLa: boolean;
    readonly isLv: boolean;
    readonly isLb: boolean;
    readonly isLs: boolean;
    readonly isLr: boolean;
    readonly isLy: boolean;
    readonly isLi: boolean;
    readonly isLt: boolean;
    readonly isLu: boolean;
    readonly isMk: boolean;
    readonly isMg: boolean;
    readonly isMw: boolean;
    readonly isMy: boolean;
    readonly isMv: boolean;
    readonly isMl: boolean;
    readonly isMt: boolean;
    readonly isMh: boolean;
    readonly isMq: boolean;
    readonly isMr: boolean;
    readonly isMu: boolean;
    readonly isYt: boolean;
    readonly isMx: boolean;
    readonly isFm: boolean;
    readonly isMd: boolean;
    readonly isMc: boolean;
    readonly isMn: boolean;
    readonly isMe: boolean;
    readonly isMs: boolean;
    readonly isMa: boolean;
    readonly isMz: boolean;
    readonly isMm: boolean;
    readonly isNa: boolean;
    readonly isNr: boolean;
    readonly isNp: boolean;
    readonly isNl: boolean;
    readonly isAn: boolean;
    readonly isNc: boolean;
    readonly isNz: boolean;
    readonly isNi: boolean;
    readonly isNe: boolean;
    readonly isNg: boolean;
    readonly isNu: boolean;
    readonly isNf: boolean;
    readonly isMp: boolean;
    readonly isNo: boolean;
    readonly isOm: boolean;
    readonly isPk: boolean;
    readonly isPw: boolean;
    readonly isPs: boolean;
    readonly isPa: boolean;
    readonly isPg: boolean;
    readonly isPy: boolean;
    readonly isPe: boolean;
    readonly isPh: boolean;
    readonly isPn: boolean;
    readonly isPl: boolean;
    readonly isPt: boolean;
    readonly isPr: boolean;
    readonly isQa: boolean;
    readonly isRe: boolean;
    readonly isRo: boolean;
    readonly isRu: boolean;
    readonly isRw: boolean;
    readonly isBl: boolean;
    readonly isSh: boolean;
    readonly isKn: boolean;
    readonly isLc: boolean;
    readonly isMf: boolean;
    readonly isPm: boolean;
    readonly isVc: boolean;
    readonly isWs: boolean;
    readonly isSm: boolean;
    readonly isSt: boolean;
    readonly isSa: boolean;
    readonly isSn: boolean;
    readonly isRs: boolean;
    readonly isSc: boolean;
    readonly isSl: boolean;
    readonly isSg: boolean;
    readonly isSk: boolean;
    readonly isSi: boolean;
    readonly isSb: boolean;
    readonly isSo: boolean;
    readonly isZa: boolean;
    readonly isGs: boolean;
    readonly isSs: boolean;
    readonly isEs: boolean;
    readonly isLk: boolean;
    readonly isSd: boolean;
    readonly isSr: boolean;
    readonly isSj: boolean;
    readonly isSz: boolean;
    readonly isSe: boolean;
    readonly isCh: boolean;
    readonly isSy: boolean;
    readonly isTw: boolean;
    readonly isTj: boolean;
    readonly isTz: boolean;
    readonly isTh: boolean;
    readonly isTl: boolean;
    readonly isTg: boolean;
    readonly isTk: boolean;
    readonly isTo: boolean;
    readonly isTt: boolean;
    readonly isTn: boolean;
    readonly isTr: boolean;
    readonly isTm: boolean;
    readonly isTc: boolean;
    readonly isTv: boolean;
    readonly isUg: boolean;
    readonly isUa: boolean;
    readonly isAe: boolean;
    readonly isGb: boolean;
    readonly isUs: boolean;
    readonly isUm: boolean;
    readonly isUy: boolean;
    readonly isUz: boolean;
    readonly isVu: boolean;
    readonly isVe: boolean;
    readonly isVn: boolean;
    readonly isVi: boolean;
    readonly isWf: boolean;
    readonly isEh: boolean;
    readonly isYe: boolean;
    readonly isZm: boolean;
    readonly isZw: boolean;
    readonly isBq: boolean;
    readonly isCw: boolean;
    readonly isSx: boolean;
    readonly type: 'Af' | 'Ax' | 'Al' | 'Dz' | 'As' | 'Ad' | 'Ao' | 'Ai' | 'Aq' | 'Ag' | 'Ar' | 'Am' | 'Aw' | 'Au' | 'At' | 'Az' | 'Bs' | 'Bh' | 'Bd' | 'Bb' | 'By' | 'Be' | 'Bz' | 'Bj' | 'Bm' | 'Bt' | 'Bo' | 'Ba' | 'Bw' | 'Bv' | 'Br' | 'Vg' | 'Io' | 'Bn' | 'Bg' | 'Bf' | 'Bi' | 'Kh' | 'Cm' | 'Ca' | 'Cv' | 'Ky' | 'Cf' | 'Td' | 'Cl' | 'Cn' | 'Hk' | 'Mo' | 'Cx' | 'Cc' | 'Co' | 'Km' | 'Cg' | 'Cd' | 'Ck' | 'Cr' | 'Ci' | 'Hr' | 'Cu' | 'Cy' | 'Cz' | 'Dk' | 'Dj' | 'Dm' | 'Do' | 'Ec' | 'Eg' | 'Sv' | 'Gq' | 'Er' | 'Ee' | 'Et' | 'Fk' | 'Fo' | 'Fj' | 'Fi' | 'Fr' | 'Gf' | 'Pf' | 'Tf' | 'Ga' | 'Gm' | 'Ge' | 'De' | 'Gh' | 'Gi' | 'Gr' | 'Gl' | 'Gd' | 'Gp' | 'Gu' | 'Gt' | 'Gg' | 'Gn' | 'Gw' | 'Gy' | 'Ht' | 'Hm' | 'Va' | 'Hn' | 'Hu' | 'Is' | 'In' | 'Id' | 'Ir' | 'Iq' | 'Ie' | 'Im' | 'Il' | 'It' | 'Jm' | 'Jp' | 'Je' | 'Jo' | 'Kz' | 'Ke' | 'Ki' | 'Kp' | 'Kr' | 'Kw' | 'Kg' | 'La' | 'Lv' | 'Lb' | 'Ls' | 'Lr' | 'Ly' | 'Li' | 'Lt' | 'Lu' | 'Mk' | 'Mg' | 'Mw' | 'My' | 'Mv' | 'Ml' | 'Mt' | 'Mh' | 'Mq' | 'Mr' | 'Mu' | 'Yt' | 'Mx' | 'Fm' | 'Md' | 'Mc' | 'Mn' | 'Me' | 'Ms' | 'Ma' | 'Mz' | 'Mm' | 'Na' | 'Nr' | 'Np' | 'Nl' | 'An' | 'Nc' | 'Nz' | 'Ni' | 'Ne' | 'Ng' | 'Nu' | 'Nf' | 'Mp' | 'No' | 'Om' | 'Pk' | 'Pw' | 'Ps' | 'Pa' | 'Pg' | 'Py' | 'Pe' | 'Ph' | 'Pn' | 'Pl' | 'Pt' | 'Pr' | 'Qa' | 'Re' | 'Ro' | 'Ru' | 'Rw' | 'Bl' | 'Sh' | 'Kn' | 'Lc' | 'Mf' | 'Pm' | 'Vc' | 'Ws' | 'Sm' | 'St' | 'Sa' | 'Sn' | 'Rs' | 'Sc' | 'Sl' | 'Sg' | 'Sk' | 'Si' | 'Sb' | 'So' | 'Za' | 'Gs' | 'Ss' | 'Es' | 'Lk' | 'Sd' | 'Sr' | 'Sj' | 'Sz' | 'Se' | 'Ch' | 'Sy' | 'Tw' | 'Tj' | 'Tz' | 'Th' | 'Tl' | 'Tg' | 'Tk' | 'To' | 'Tt' | 'Tn' | 'Tr' | 'Tm' | 'Tc' | 'Tv' | 'Ug' | 'Ua' | 'Ae' | 'Gb' | 'Us' | 'Um' | 'Uy' | 'Uz' | 'Vu' | 'Ve' | 'Vn' | 'Vi' | 'Wf' | 'Eh' | 'Ye' | 'Zm' | 'Zw' | 'Bq' | 'Cw' | 'Sx';
  }

  /** @name PolymeshPrimitivesAuthorizationAuthorizationData (82) */
  interface PolymeshPrimitivesAuthorizationAuthorizationData extends Enum {
    readonly isAttestPrimaryKeyRotation: boolean;
    readonly asAttestPrimaryKeyRotation: PolymeshPrimitivesIdentityId;
    readonly isRotatePrimaryKey: boolean;
    readonly isTransferTicker: boolean;
    readonly asTransferTicker: PolymeshPrimitivesTicker;
    readonly isAddMultiSigSigner: boolean;
    readonly asAddMultiSigSigner: AccountId32;
    readonly isTransferAssetOwnership: boolean;
    readonly asTransferAssetOwnership: PolymeshPrimitivesAssetAssetId;
    readonly isJoinIdentity: boolean;
    readonly asJoinIdentity: PolymeshPrimitivesSecondaryKeyPermissions;
    readonly isPortfolioCustody: boolean;
    readonly asPortfolioCustody: PolymeshPrimitivesIdentityIdPortfolioId;
    readonly isBecomeAgent: boolean;
    readonly asBecomeAgent: ITuple<[PolymeshPrimitivesAssetAssetId, PolymeshPrimitivesAgentAgentGroup]>;
    readonly isOldAddRelayerPayingKey: boolean;
    readonly asOldAddRelayerPayingKey: ITuple<[AccountId32, AccountId32, u128]>;
    readonly isRotatePrimaryKeyToSecondary: boolean;
    readonly asRotatePrimaryKeyToSecondary: PolymeshPrimitivesSecondaryKeyPermissions;
    readonly type: 'AttestPrimaryKeyRotation' | 'RotatePrimaryKey' | 'TransferTicker' | 'AddMultiSigSigner' | 'TransferAssetOwnership' | 'JoinIdentity' | 'PortfolioCustody' | 'BecomeAgent' | 'OldAddRelayerPayingKey' | 'RotatePrimaryKeyToSecondary';
  }

  /** @name PolymeshPrimitivesTicker (83) */
  interface PolymeshPrimitivesTicker extends U8aFixed {}

  /** @name PolymeshPrimitivesAgentAgentGroup (85) */
  interface PolymeshPrimitivesAgentAgentGroup extends Enum {
    readonly isFull: boolean;
    readonly isCustom: boolean;
    readonly asCustom: u32;
    readonly isExceptMeta: boolean;
    readonly isPolymeshV1CAA: boolean;
    readonly isPolymeshV1PIA: boolean;
    readonly type: 'Full' | 'Custom' | 'ExceptMeta' | 'PolymeshV1CAA' | 'PolymeshV1PIA';
  }

  /** @name PalletGroupEvent (87) */
  interface PalletGroupEvent extends Enum {
    readonly isMemberAdded: boolean;
    readonly asMemberAdded: ITuple<[PolymeshPrimitivesIdentityId, PolymeshPrimitivesIdentityId]>;
    readonly isMemberRemoved: boolean;
    readonly asMemberRemoved: ITuple<[PolymeshPrimitivesIdentityId, PolymeshPrimitivesIdentityId]>;
    readonly isMemberRevoked: boolean;
    readonly asMemberRevoked: ITuple<[PolymeshPrimitivesIdentityId, PolymeshPrimitivesIdentityId]>;
    readonly isMembersSwapped: boolean;
    readonly asMembersSwapped: ITuple<[PolymeshPrimitivesIdentityId, PolymeshPrimitivesIdentityId, PolymeshPrimitivesIdentityId]>;
    readonly isMembersReset: boolean;
    readonly asMembersReset: ITuple<[PolymeshPrimitivesIdentityId, Vec<PolymeshPrimitivesIdentityId>]>;
    readonly isActiveLimitChanged: boolean;
    readonly asActiveLimitChanged: ITuple<[PolymeshPrimitivesIdentityId, u32, u32]>;
    readonly type: 'MemberAdded' | 'MemberRemoved' | 'MemberRevoked' | 'MembersSwapped' | 'MembersReset' | 'ActiveLimitChanged';
  }

  /** @name PalletCommitteeEvent (89) */
  interface PalletCommitteeEvent extends Enum {
    readonly isProposed: boolean;
    readonly asProposed: ITuple<[PolymeshPrimitivesIdentityId, u32, H256]>;
    readonly isVoted: boolean;
    readonly asVoted: ITuple<[PolymeshPrimitivesIdentityId, u32, H256, bool, u32, u32, u32]>;
    readonly isVoteRetracted: boolean;
    readonly asVoteRetracted: ITuple<[PolymeshPrimitivesIdentityId, u32, H256, bool]>;
    readonly isFinalVotes: boolean;
    readonly asFinalVotes: ITuple<[Option<PolymeshPrimitivesIdentityId>, u32, H256, Vec<PolymeshPrimitivesIdentityId>, Vec<PolymeshPrimitivesIdentityId>]>;
    readonly isApproved: boolean;
    readonly asApproved: ITuple<[Option<PolymeshPrimitivesIdentityId>, H256, u32, u32, u32]>;
    readonly isRejected: boolean;
    readonly asRejected: ITuple<[Option<PolymeshPrimitivesIdentityId>, H256, u32, u32, u32]>;
    readonly isExecuted: boolean;
    readonly asExecuted: ITuple<[Option<PolymeshPrimitivesIdentityId>, H256, Result<Null, SpRuntimeDispatchError>]>;
    readonly isReleaseCoordinatorUpdated: boolean;
    readonly asReleaseCoordinatorUpdated: Option<PolymeshPrimitivesIdentityId>;
    readonly isExpiresAfterUpdated: boolean;
    readonly asExpiresAfterUpdated: ITuple<[PolymeshPrimitivesIdentityId, PolymeshPrimitivesMaybeBlock]>;
    readonly isVoteThresholdUpdated: boolean;
    readonly asVoteThresholdUpdated: ITuple<[PolymeshPrimitivesIdentityId, u32, u32]>;
    readonly type: 'Proposed' | 'Voted' | 'VoteRetracted' | 'FinalVotes' | 'Approved' | 'Rejected' | 'Executed' | 'ReleaseCoordinatorUpdated' | 'ExpiresAfterUpdated' | 'VoteThresholdUpdated';
  }

  /** @name PolymeshPrimitivesMaybeBlock (92) */
  interface PolymeshPrimitivesMaybeBlock extends Enum {
    readonly isSome: boolean;
    readonly asSome: u32;
    readonly isNone: boolean;
    readonly type: 'Some' | 'None';
  }

  /** @name PalletMultisigEvent (98) */
  interface PalletMultisigEvent extends Enum {
    readonly isMultiSigCreated: boolean;
    readonly asMultiSigCreated: {
      readonly callerDid: PolymeshPrimitivesIdentityId;
      readonly multisig: AccountId32;
      readonly caller: AccountId32;
      readonly signers: Vec<AccountId32>;
      readonly sigsRequired: u64;
    } & Struct;
    readonly isProposalAdded: boolean;
    readonly asProposalAdded: {
      readonly callerDid: Option<PolymeshPrimitivesIdentityId>;
      readonly multisig: AccountId32;
      readonly proposalId: u64;
    } & Struct;
    readonly isProposalExecuted: boolean;
    readonly asProposalExecuted: {
      readonly callerDid: Option<PolymeshPrimitivesIdentityId>;
      readonly multisig: AccountId32;
      readonly proposalId: u64;
      readonly result: Result<Null, SpRuntimeDispatchError>;
    } & Struct;
    readonly isMultiSigSignerAdded: boolean;
    readonly asMultiSigSignerAdded: {
      readonly callerDid: PolymeshPrimitivesIdentityId;
      readonly multisig: AccountId32;
      readonly signer: AccountId32;
    } & Struct;
    readonly isMultiSigSignersAuthorized: boolean;
    readonly asMultiSigSignersAuthorized: {
      readonly callerDid: PolymeshPrimitivesIdentityId;
      readonly multisig: AccountId32;
      readonly signers: Vec<AccountId32>;
    } & Struct;
    readonly isMultiSigSignersRemoved: boolean;
    readonly asMultiSigSignersRemoved: {
      readonly callerDid: PolymeshPrimitivesIdentityId;
      readonly multisig: AccountId32;
      readonly signers: Vec<AccountId32>;
    } & Struct;
    readonly isMultiSigSignersRequiredChanged: boolean;
    readonly asMultiSigSignersRequiredChanged: {
      readonly callerDid: Option<PolymeshPrimitivesIdentityId>;
      readonly multisig: AccountId32;
      readonly sigsRequired: u64;
    } & Struct;
    readonly isProposalApprovalVote: boolean;
    readonly asProposalApprovalVote: {
      readonly callerDid: Option<PolymeshPrimitivesIdentityId>;
      readonly multisig: AccountId32;
      readonly signer: AccountId32;
      readonly proposalId: u64;
    } & Struct;
    readonly isProposalRejectionVote: boolean;
    readonly asProposalRejectionVote: {
      readonly callerDid: Option<PolymeshPrimitivesIdentityId>;
      readonly multisig: AccountId32;
      readonly signer: AccountId32;
      readonly proposalId: u64;
    } & Struct;
    readonly isProposalApproved: boolean;
    readonly asProposalApproved: {
      readonly callerDid: Option<PolymeshPrimitivesIdentityId>;
      readonly multisig: AccountId32;
      readonly proposalId: u64;
    } & Struct;
    readonly isProposalRejected: boolean;
    readonly asProposalRejected: {
      readonly callerDid: Option<PolymeshPrimitivesIdentityId>;
      readonly multisig: AccountId32;
      readonly proposalId: u64;
    } & Struct;
    readonly isMultiSigAddedAdmin: boolean;
    readonly asMultiSigAddedAdmin: {
      readonly callerDid: PolymeshPrimitivesIdentityId;
      readonly multisig: AccountId32;
      readonly adminDid: PolymeshPrimitivesIdentityId;
    } & Struct;
    readonly isMultiSigRemovedAdmin: boolean;
    readonly asMultiSigRemovedAdmin: {
      readonly callerDid: PolymeshPrimitivesIdentityId;
      readonly multisig: AccountId32;
      readonly adminDid: PolymeshPrimitivesIdentityId;
    } & Struct;
    readonly isMultiSigRemovedPayingDid: boolean;
    readonly asMultiSigRemovedPayingDid: {
      readonly callerDid: PolymeshPrimitivesIdentityId;
      readonly multisig: AccountId32;
      readonly payingDid: PolymeshPrimitivesIdentityId;
    } & Struct;
    readonly type: 'MultiSigCreated' | 'ProposalAdded' | 'ProposalExecuted' | 'MultiSigSignerAdded' | 'MultiSigSignersAuthorized' | 'MultiSigSignersRemoved' | 'MultiSigSignersRequiredChanged' | 'ProposalApprovalVote' | 'ProposalRejectionVote' | 'ProposalApproved' | 'ProposalRejected' | 'MultiSigAddedAdmin' | 'MultiSigRemovedAdmin' | 'MultiSigRemovedPayingDid';
  }

  /** @name PalletValidatorsEvent (100) */
  interface PalletValidatorsEvent extends Enum {
    readonly isNominated: boolean;
    readonly asNominated: {
      readonly nominatorIdentity: PolymeshPrimitivesIdentityId;
      readonly stash: AccountId32;
      readonly targets: Vec<AccountId32>;
    } & Struct;
    readonly isPermissionedIdentityAdded: boolean;
    readonly asPermissionedIdentityAdded: {
      readonly governanceCouncillDid: PolymeshPrimitivesIdentityId;
      readonly validatorsIdentity: PolymeshPrimitivesIdentityId;
    } & Struct;
    readonly isPermissionedIdentityRemoved: boolean;
    readonly asPermissionedIdentityRemoved: {
      readonly governanceCouncillDid: PolymeshPrimitivesIdentityId;
      readonly validatorsIdentity: PolymeshPrimitivesIdentityId;
    } & Struct;
    readonly isInvalidatedNominators: boolean;
    readonly asInvalidatedNominators: {
      readonly governanceCouncillDid: PolymeshPrimitivesIdentityId;
      readonly governanceCouncillAccount: PolymeshPrimitivesIdentityId;
      readonly expiredNominators: Vec<AccountId32>;
    } & Struct;
    readonly isSlashingAllowedForChanged: boolean;
    readonly asSlashingAllowedForChanged: {
      readonly slashingSwitch: PalletValidatorsSlashingSwitch;
    } & Struct;
    readonly isRewardPaymentSchedulingInterrupted: boolean;
    readonly asRewardPaymentSchedulingInterrupted: {
      readonly accountId: AccountId32;
      readonly era: u32;
      readonly error: SpRuntimeDispatchError;
    } & Struct;
    readonly isCommissionCapUpdated: boolean;
    readonly asCommissionCapUpdated: {
      readonly governanceCouncillDid: PolymeshPrimitivesIdentityId;
      readonly oldCommissionCap: Perbill;
      readonly newCommissionCap: Perbill;
    } & Struct;
    readonly type: 'Nominated' | 'PermissionedIdentityAdded' | 'PermissionedIdentityRemoved' | 'InvalidatedNominators' | 'SlashingAllowedForChanged' | 'RewardPaymentSchedulingInterrupted' | 'CommissionCapUpdated';
  }

  /** @name PalletValidatorsSlashingSwitch (101) */
  interface PalletValidatorsSlashingSwitch extends Enum {
    readonly isValidator: boolean;
    readonly isValidatorAndNominator: boolean;
    readonly isNone: boolean;
    readonly type: 'Validator' | 'ValidatorAndNominator' | 'None';
  }

  /** @name PalletStakingPalletEvent (103) */
  interface PalletStakingPalletEvent extends Enum {
    readonly isEraPaid: boolean;
    readonly asEraPaid: {
      readonly eraIndex: u32;
      readonly validatorPayout: u128;
      readonly remainder: u128;
    } & Struct;
    readonly isRewarded: boolean;
    readonly asRewarded: {
      readonly stash: AccountId32;
      readonly dest: PalletStakingRewardDestination;
      readonly amount: u128;
    } & Struct;
    readonly isSlashed: boolean;
    readonly asSlashed: {
      readonly staker: AccountId32;
      readonly amount: u128;
    } & Struct;
    readonly isSlashReported: boolean;
    readonly asSlashReported: {
      readonly validator: AccountId32;
      readonly fraction: Perbill;
      readonly slashEra: u32;
    } & Struct;
    readonly isOldSlashingReportDiscarded: boolean;
    readonly asOldSlashingReportDiscarded: {
      readonly sessionIndex: u32;
    } & Struct;
    readonly isStakersElected: boolean;
    readonly isBonded: boolean;
    readonly asBonded: {
      readonly stash: AccountId32;
      readonly amount: u128;
    } & Struct;
    readonly isUnbonded: boolean;
    readonly asUnbonded: {
      readonly stash: AccountId32;
      readonly amount: u128;
    } & Struct;
    readonly isWithdrawn: boolean;
    readonly asWithdrawn: {
      readonly stash: AccountId32;
      readonly amount: u128;
    } & Struct;
    readonly isKicked: boolean;
    readonly asKicked: {
      readonly nominator: AccountId32;
      readonly stash: AccountId32;
    } & Struct;
    readonly isStakingElectionFailed: boolean;
    readonly isChilled: boolean;
    readonly asChilled: {
      readonly stash: AccountId32;
    } & Struct;
    readonly isPayoutStarted: boolean;
    readonly asPayoutStarted: {
      readonly eraIndex: u32;
      readonly validatorStash: AccountId32;
      readonly page: u32;
      readonly next: Option<u32>;
    } & Struct;
    readonly isValidatorPrefsSet: boolean;
    readonly asValidatorPrefsSet: {
      readonly stash: AccountId32;
      readonly prefs: PalletStakingValidatorPrefs;
    } & Struct;
    readonly isSnapshotVotersSizeExceeded: boolean;
    readonly asSnapshotVotersSizeExceeded: {
      readonly size_: u32;
    } & Struct;
    readonly isSnapshotTargetsSizeExceeded: boolean;
    readonly asSnapshotTargetsSizeExceeded: {
      readonly size_: u32;
    } & Struct;
    readonly isForceEra: boolean;
    readonly asForceEra: {
      readonly mode: PalletStakingForcing;
    } & Struct;
    readonly isControllerBatchDeprecated: boolean;
    readonly asControllerBatchDeprecated: {
      readonly failures: u32;
    } & Struct;
    readonly isCurrencyMigrated: boolean;
    readonly asCurrencyMigrated: {
      readonly stash: AccountId32;
      readonly forceWithdraw: u128;
    } & Struct;
    readonly type: 'EraPaid' | 'Rewarded' | 'Slashed' | 'SlashReported' | 'OldSlashingReportDiscarded' | 'StakersElected' | 'Bonded' | 'Unbonded' | 'Withdrawn' | 'Kicked' | 'StakingElectionFailed' | 'Chilled' | 'PayoutStarted' | 'ValidatorPrefsSet' | 'SnapshotVotersSizeExceeded' | 'SnapshotTargetsSizeExceeded' | 'ForceEra' | 'ControllerBatchDeprecated' | 'CurrencyMigrated';
  }

  /** @name PalletStakingRewardDestination (104) */
  interface PalletStakingRewardDestination extends Enum {
    readonly isStaked: boolean;
    readonly isStash: boolean;
    readonly isController: boolean;
    readonly isAccount: boolean;
    readonly asAccount: AccountId32;
    readonly isNone: boolean;
    readonly type: 'Staked' | 'Stash' | 'Controller' | 'Account' | 'None';
  }

  /** @name PalletStakingValidatorPrefs (106) */
  interface PalletStakingValidatorPrefs extends Struct {
    readonly commission: Compact<Perbill>;
    readonly blocked: bool;
  }

  /** @name PalletStakingForcing (108) */
  interface PalletStakingForcing extends Enum {
    readonly isNotForcing: boolean;
    readonly isForceNew: boolean;
    readonly isForceNone: boolean;
    readonly isForceAlways: boolean;
    readonly type: 'NotForcing' | 'ForceNew' | 'ForceNone' | 'ForceAlways';
  }

  /** @name PalletOffencesEvent (109) */
  interface PalletOffencesEvent extends Enum {
    readonly isOffence: boolean;
    readonly asOffence: {
      readonly kind: U8aFixed;
      readonly timeslot: Bytes;
    } & Struct;
    readonly type: 'Offence';
  }

  /** @name PalletSessionEvent (110) */
  interface PalletSessionEvent extends Enum {
    readonly isNewSession: boolean;
    readonly asNewSession: {
      readonly sessionIndex: u32;
    } & Struct;
    readonly isNewQueued: boolean;
    readonly isValidatorDisabled: boolean;
    readonly asValidatorDisabled: {
      readonly validator: AccountId32;
    } & Struct;
    readonly isValidatorReenabled: boolean;
    readonly asValidatorReenabled: {
      readonly validator: AccountId32;
    } & Struct;
    readonly type: 'NewSession' | 'NewQueued' | 'ValidatorDisabled' | 'ValidatorReenabled';
  }

  /** @name PalletGrandpaEvent (111) */
  interface PalletGrandpaEvent extends Enum {
    readonly isNewAuthorities: boolean;
    readonly asNewAuthorities: {
      readonly authoritySet: Vec<ITuple<[SpConsensusGrandpaAppPublic, u64]>>;
    } & Struct;
    readonly isPaused: boolean;
    readonly isResumed: boolean;
    readonly type: 'NewAuthorities' | 'Paused' | 'Resumed';
  }

  /** @name SpConsensusGrandpaAppPublic (114) */
  interface SpConsensusGrandpaAppPublic extends U8aFixed {}

  /** @name PalletSessionHistoricalPalletEvent (115) */
  interface PalletSessionHistoricalPalletEvent extends Enum {
    readonly isRootStored: boolean;
    readonly asRootStored: {
      readonly index: u32;
    } & Struct;
    readonly isRootsPruned: boolean;
    readonly asRootsPruned: {
      readonly upTo: u32;
    } & Struct;
    readonly type: 'RootStored' | 'RootsPruned';
  }

  /** @name PalletImOnlineEvent (116) */
  interface PalletImOnlineEvent extends Enum {
    readonly isHeartbeatReceived: boolean;
    readonly asHeartbeatReceived: {
      readonly authorityId: PalletImOnlineSr25519AppSr25519Public;
    } & Struct;
    readonly isAllGood: boolean;
    readonly isSomeOffline: boolean;
    readonly asSomeOffline: {
      readonly offline: Vec<ITuple<[AccountId32, SpStakingExposure]>>;
    } & Struct;
    readonly type: 'HeartbeatReceived' | 'AllGood' | 'SomeOffline';
  }

  /** @name PalletImOnlineSr25519AppSr25519Public (117) */
  interface PalletImOnlineSr25519AppSr25519Public extends U8aFixed {}

  /** @name SpStakingExposure (120) */
  interface SpStakingExposure extends Struct {
    readonly total: Compact<u128>;
    readonly own: Compact<u128>;
    readonly others: Vec<SpStakingIndividualExposure>;
  }

  /** @name SpStakingIndividualExposure (123) */
  interface SpStakingIndividualExposure extends Struct {
    readonly who: AccountId32;
    readonly value: Compact<u128>;
  }

  /** @name PalletSudoEvent (124) */
  interface PalletSudoEvent extends Enum {
    readonly isSudid: boolean;
    readonly asSudid: {
      readonly sudoResult: Result<Null, SpRuntimeDispatchError>;
    } & Struct;
    readonly isKeyChanged: boolean;
    readonly asKeyChanged: {
      readonly old: Option<AccountId32>;
      readonly new_: AccountId32;
    } & Struct;
    readonly isKeyRemoved: boolean;
    readonly isSudoAsDone: boolean;
    readonly asSudoAsDone: {
      readonly sudoResult: Result<Null, SpRuntimeDispatchError>;
    } & Struct;
    readonly type: 'Sudid' | 'KeyChanged' | 'KeyRemoved' | 'SudoAsDone';
  }

  /** @name PalletAssetEvent (125) */
  interface PalletAssetEvent extends Enum {
    readonly isAssetCreated: boolean;
    readonly asAssetCreated: ITuple<[PolymeshPrimitivesIdentityId, PolymeshPrimitivesAssetAssetId, bool, PolymeshPrimitivesAssetAssetType, PolymeshPrimitivesIdentityId, Bytes, Vec<PolymeshPrimitivesAssetIdentifier>, Option<Bytes>]>;
    readonly isIdentifiersUpdated: boolean;
    readonly asIdentifiersUpdated: ITuple<[PolymeshPrimitivesIdentityId, PolymeshPrimitivesAssetAssetId, Vec<PolymeshPrimitivesAssetIdentifier>]>;
    readonly isDivisibilityChanged: boolean;
    readonly asDivisibilityChanged: ITuple<[PolymeshPrimitivesIdentityId, PolymeshPrimitivesAssetAssetId, bool]>;
    readonly isTickerRegistered: boolean;
    readonly asTickerRegistered: ITuple<[PolymeshPrimitivesIdentityId, PolymeshPrimitivesTicker, Option<u64>]>;
    readonly isTickerTransferred: boolean;
    readonly asTickerTransferred: ITuple<[PolymeshPrimitivesIdentityId, PolymeshPrimitivesTicker, PolymeshPrimitivesIdentityId]>;
    readonly isAssetOwnershipTransferred: boolean;
    readonly asAssetOwnershipTransferred: ITuple<[PolymeshPrimitivesIdentityId, PolymeshPrimitivesAssetAssetId, PolymeshPrimitivesIdentityId]>;
    readonly isAssetFrozen: boolean;
    readonly asAssetFrozen: ITuple<[PolymeshPrimitivesIdentityId, PolymeshPrimitivesAssetAssetId]>;
    readonly isAssetUnfrozen: boolean;
    readonly asAssetUnfrozen: ITuple<[PolymeshPrimitivesIdentityId, PolymeshPrimitivesAssetAssetId]>;
    readonly isAssetRenamed: boolean;
    readonly asAssetRenamed: ITuple<[PolymeshPrimitivesIdentityId, PolymeshPrimitivesAssetAssetId, Bytes]>;
    readonly isFundingRoundSet: boolean;
    readonly asFundingRoundSet: ITuple<[PolymeshPrimitivesIdentityId, PolymeshPrimitivesAssetAssetId, Bytes]>;
    readonly isDocumentAdded: boolean;
    readonly asDocumentAdded: ITuple<[PolymeshPrimitivesIdentityId, PolymeshPrimitivesAssetAssetId, u32, PolymeshPrimitivesDocument]>;
    readonly isDocumentRemoved: boolean;
    readonly asDocumentRemoved: ITuple<[PolymeshPrimitivesIdentityId, PolymeshPrimitivesAssetAssetId, u32]>;
    readonly isControllerTransfer: boolean;
    readonly asControllerTransfer: ITuple<[PolymeshPrimitivesIdentityId, PolymeshPrimitivesAssetAssetId, PolymeshPrimitivesAssetAssetHolder, u128]>;
    readonly isCustomAssetTypeExists: boolean;
    readonly asCustomAssetTypeExists: ITuple<[PolymeshPrimitivesIdentityId, u32, Bytes]>;
    readonly isCustomAssetTypeRegistered: boolean;
    readonly asCustomAssetTypeRegistered: ITuple<[PolymeshPrimitivesIdentityId, u32, Bytes]>;
    readonly isSetAssetMetadataValue: boolean;
    readonly asSetAssetMetadataValue: ITuple<[PolymeshPrimitivesIdentityId, PolymeshPrimitivesAssetAssetId, Bytes, Option<PolymeshPrimitivesAssetMetadataAssetMetadataValueDetail>]>;
    readonly isSetAssetMetadataValueDetails: boolean;
    readonly asSetAssetMetadataValueDetails: ITuple<[PolymeshPrimitivesIdentityId, PolymeshPrimitivesAssetAssetId, PolymeshPrimitivesAssetMetadataAssetMetadataValueDetail]>;
    readonly isRegisterAssetMetadataLocalType: boolean;
    readonly asRegisterAssetMetadataLocalType: ITuple<[PolymeshPrimitivesIdentityId, PolymeshPrimitivesAssetAssetId, Bytes, u64, PolymeshPrimitivesAssetMetadataAssetMetadataSpec]>;
    readonly isRegisterAssetMetadataGlobalType: boolean;
    readonly asRegisterAssetMetadataGlobalType: ITuple<[Bytes, u64, PolymeshPrimitivesAssetMetadataAssetMetadataSpec]>;
    readonly isAssetTypeChanged: boolean;
    readonly asAssetTypeChanged: ITuple<[PolymeshPrimitivesIdentityId, PolymeshPrimitivesAssetAssetId, PolymeshPrimitivesAssetAssetType]>;
    readonly isLocalMetadataKeyDeleted: boolean;
    readonly asLocalMetadataKeyDeleted: ITuple<[PolymeshPrimitivesIdentityId, PolymeshPrimitivesAssetAssetId, u64]>;
    readonly isMetadataValueDeleted: boolean;
    readonly asMetadataValueDeleted: ITuple<[PolymeshPrimitivesIdentityId, PolymeshPrimitivesAssetAssetId, PolymeshPrimitivesAssetMetadataAssetMetadataKey]>;
    readonly isAssetBalanceUpdated: boolean;
    readonly asAssetBalanceUpdated: ITuple<[PolymeshPrimitivesIdentityId, PolymeshPrimitivesAssetAssetId, u128, Option<PolymeshPrimitivesAssetAssetHolder>, Option<PolymeshPrimitivesAssetAssetHolder>, PolymeshPrimitivesAssetHoldingsUpdateReason]>;
    readonly isAssetAffirmationExemption: boolean;
    readonly asAssetAffirmationExemption: PolymeshPrimitivesAssetAssetId;
    readonly isRemoveAssetAffirmationExemption: boolean;
    readonly asRemoveAssetAffirmationExemption: PolymeshPrimitivesAssetAssetId;
    readonly isPreApprovedAsset: boolean;
    readonly asPreApprovedAsset: ITuple<[PolymeshPrimitivesIdentityId, PolymeshPrimitivesAssetAssetId]>;
    readonly isRemovePreApprovedAsset: boolean;
    readonly asRemovePreApprovedAsset: ITuple<[PolymeshPrimitivesIdentityId, PolymeshPrimitivesAssetAssetId]>;
    readonly isAssetMediatorsAdded: boolean;
    readonly asAssetMediatorsAdded: ITuple<[PolymeshPrimitivesIdentityId, PolymeshPrimitivesAssetAssetId, BTreeSet<PolymeshPrimitivesIdentityId>]>;
    readonly isAssetMediatorsRemoved: boolean;
    readonly asAssetMediatorsRemoved: ITuple<[PolymeshPrimitivesIdentityId, PolymeshPrimitivesAssetAssetId, BTreeSet<PolymeshPrimitivesIdentityId>]>;
    readonly isTickerLinkedToAsset: boolean;
    readonly asTickerLinkedToAsset: ITuple<[PolymeshPrimitivesIdentityId, PolymeshPrimitivesTicker, PolymeshPrimitivesAssetAssetId]>;
    readonly isTickerUnlinkedFromAsset: boolean;
    readonly asTickerUnlinkedFromAsset: ITuple<[PolymeshPrimitivesIdentityId, PolymeshPrimitivesTicker, PolymeshPrimitivesAssetAssetId]>;
    readonly isGlobalMetadataSpecUpdated: boolean;
    readonly asGlobalMetadataSpecUpdated: ITuple<[Bytes, PolymeshPrimitivesAssetMetadataAssetMetadataSpec]>;
    readonly isCreatedAssetTransfer: boolean;
    readonly asCreatedAssetTransfer: {
      readonly assetId: PolymeshPrimitivesAssetAssetId;
      readonly from: AccountId32;
      readonly to: AccountId32;
      readonly amount: u128;
      readonly memo: Option<PolymeshPrimitivesMemo>;
      readonly pendingTransferId: Option<u64>;
    } & Struct;
    readonly isMandatoryReceiverAffirmationSet: boolean;
    readonly asMandatoryReceiverAffirmationSet: ITuple<[PolymeshPrimitivesIdentityId, bool]>;
    readonly type: 'AssetCreated' | 'IdentifiersUpdated' | 'DivisibilityChanged' | 'TickerRegistered' | 'TickerTransferred' | 'AssetOwnershipTransferred' | 'AssetFrozen' | 'AssetUnfrozen' | 'AssetRenamed' | 'FundingRoundSet' | 'DocumentAdded' | 'DocumentRemoved' | 'ControllerTransfer' | 'CustomAssetTypeExists' | 'CustomAssetTypeRegistered' | 'SetAssetMetadataValue' | 'SetAssetMetadataValueDetails' | 'RegisterAssetMetadataLocalType' | 'RegisterAssetMetadataGlobalType' | 'AssetTypeChanged' | 'LocalMetadataKeyDeleted' | 'MetadataValueDeleted' | 'AssetBalanceUpdated' | 'AssetAffirmationExemption' | 'RemoveAssetAffirmationExemption' | 'PreApprovedAsset' | 'RemovePreApprovedAsset' | 'AssetMediatorsAdded' | 'AssetMediatorsRemoved' | 'TickerLinkedToAsset' | 'TickerUnlinkedFromAsset' | 'GlobalMetadataSpecUpdated' | 'CreatedAssetTransfer' | 'MandatoryReceiverAffirmationSet';
  }

  /** @name PolymeshPrimitivesAssetAssetType (126) */
  interface PolymeshPrimitivesAssetAssetType extends Enum {
    readonly isEquityCommon: boolean;
    readonly isEquityPreferred: boolean;
    readonly isCommodity: boolean;
    readonly isFixedIncome: boolean;
    readonly isReit: boolean;
    readonly isFund: boolean;
    readonly isRevenueShareAgreement: boolean;
    readonly isStructuredProduct: boolean;
    readonly isDerivative: boolean;
    readonly isCustom: boolean;
    readonly asCustom: u32;
    readonly isStableCoin: boolean;
    readonly isNonFungible: boolean;
    readonly asNonFungible: PolymeshPrimitivesAssetNonFungibleType;
    readonly type: 'EquityCommon' | 'EquityPreferred' | 'Commodity' | 'FixedIncome' | 'Reit' | 'Fund' | 'RevenueShareAgreement' | 'StructuredProduct' | 'Derivative' | 'Custom' | 'StableCoin' | 'NonFungible';
  }

  /** @name PolymeshPrimitivesAssetNonFungibleType (128) */
  interface PolymeshPrimitivesAssetNonFungibleType extends Enum {
    readonly isDerivative: boolean;
    readonly isFixedIncome: boolean;
    readonly isInvoice: boolean;
    readonly isCustom: boolean;
    readonly asCustom: u32;
    readonly type: 'Derivative' | 'FixedIncome' | 'Invoice' | 'Custom';
  }

  /** @name PolymeshPrimitivesAssetIdentifier (131) */
  interface PolymeshPrimitivesAssetIdentifier extends Enum {
    readonly isCusip: boolean;
    readonly asCusip: U8aFixed;
    readonly isCins: boolean;
    readonly asCins: U8aFixed;
    readonly isIsin: boolean;
    readonly asIsin: U8aFixed;
    readonly isLei: boolean;
    readonly asLei: U8aFixed;
    readonly isFigi: boolean;
    readonly asFigi: U8aFixed;
    readonly type: 'Cusip' | 'Cins' | 'Isin' | 'Lei' | 'Figi';
  }

  /** @name PolymeshPrimitivesDocument (137) */
  interface PolymeshPrimitivesDocument extends Struct {
    readonly uri: Bytes;
    readonly contentHash: PolymeshPrimitivesDocumentHash;
    readonly name: Bytes;
    readonly docType: Option<Bytes>;
    readonly filingDate: Option<u64>;
  }

  /** @name PolymeshPrimitivesDocumentHash (139) */
  interface PolymeshPrimitivesDocumentHash extends Enum {
    readonly isNone: boolean;
    readonly isH512: boolean;
    readonly asH512: U8aFixed;
    readonly isH384: boolean;
    readonly asH384: U8aFixed;
    readonly isH320: boolean;
    readonly asH320: U8aFixed;
    readonly isH256: boolean;
    readonly asH256: U8aFixed;
    readonly isH224: boolean;
    readonly asH224: U8aFixed;
    readonly isH192: boolean;
    readonly asH192: U8aFixed;
    readonly isH160: boolean;
    readonly asH160: U8aFixed;
    readonly isH128: boolean;
    readonly asH128: U8aFixed;
    readonly type: 'None' | 'H512' | 'H384' | 'H320' | 'H256' | 'H224' | 'H192' | 'H160' | 'H128';
  }

  /** @name PolymeshPrimitivesAssetAssetHolder (148) */
  interface PolymeshPrimitivesAssetAssetHolder extends Enum {
    readonly isPortfolio: boolean;
    readonly asPortfolio: PolymeshPrimitivesIdentityIdPortfolioId;
    readonly isAccount: boolean;
    readonly asAccount: AccountId32;
    readonly type: 'Portfolio' | 'Account';
  }

  /** @name PolymeshPrimitivesAssetMetadataAssetMetadataValueDetail (151) */
  interface PolymeshPrimitivesAssetMetadataAssetMetadataValueDetail extends Struct {
    readonly expire: Option<u64>;
    readonly lockStatus: PolymeshPrimitivesAssetMetadataAssetMetadataLockStatus;
  }

  /** @name PolymeshPrimitivesAssetMetadataAssetMetadataLockStatus (152) */
  interface PolymeshPrimitivesAssetMetadataAssetMetadataLockStatus extends Enum {
    readonly isUnlocked: boolean;
    readonly isLocked: boolean;
    readonly isLockedUntil: boolean;
    readonly asLockedUntil: u64;
    readonly type: 'Unlocked' | 'Locked' | 'LockedUntil';
  }

  /** @name PolymeshPrimitivesAssetMetadataAssetMetadataSpec (155) */
  interface PolymeshPrimitivesAssetMetadataAssetMetadataSpec extends Struct {
    readonly url: Option<Bytes>;
    readonly description: Option<Bytes>;
    readonly typeDef: Option<Bytes>;
  }

  /** @name PolymeshPrimitivesAssetMetadataAssetMetadataKey (162) */
  interface PolymeshPrimitivesAssetMetadataAssetMetadataKey extends Enum {
    readonly isGlobal: boolean;
    readonly asGlobal: u64;
    readonly isLocal: boolean;
    readonly asLocal: u64;
    readonly type: 'Global' | 'Local';
  }

  /** @name PolymeshPrimitivesAssetHoldingsUpdateReason (164) */
  interface PolymeshPrimitivesAssetHoldingsUpdateReason extends Enum {
    readonly isIssued: boolean;
    readonly asIssued: {
      readonly fundingRoundName: Option<Bytes>;
    } & Struct;
    readonly isRedeemed: boolean;
    readonly isTransferred: boolean;
    readonly asTransferred: {
      readonly instructionId: Option<u64>;
      readonly instructionMemo: Option<PolymeshPrimitivesMemo>;
    } & Struct;
    readonly isControllerTransfer: boolean;
    readonly type: 'Issued' | 'Redeemed' | 'Transferred' | 'ControllerTransfer';
  }

  /** @name PalletCorporateActionsDistributionPalletEvent (168) */
  interface PalletCorporateActionsDistributionPalletEvent extends Enum {
    readonly isCreated: boolean;
    readonly asCreated: ITuple<[PolymeshPrimitivesEventOnly, PalletCorporateActionsCaId, PalletCorporateActionsDistribution]>;
    readonly isBenefitClaimed: boolean;
    readonly asBenefitClaimed: ITuple<[PolymeshPrimitivesEventOnly, PolymeshPrimitivesEventOnly, PalletCorporateActionsCaId, PalletCorporateActionsDistribution, u128, Permill]>;
    readonly isReclaimed: boolean;
    readonly asReclaimed: ITuple<[PolymeshPrimitivesEventOnly, PalletCorporateActionsCaId, u128]>;
    readonly isRemoved: boolean;
    readonly asRemoved: ITuple<[PolymeshPrimitivesEventOnly, PalletCorporateActionsCaId]>;
    readonly type: 'Created' | 'BenefitClaimed' | 'Reclaimed' | 'Removed';
  }

  /** @name PolymeshPrimitivesEventOnly (169) */
  interface PolymeshPrimitivesEventOnly extends PolymeshPrimitivesIdentityId {}

  /** @name PalletCorporateActionsCaId (170) */
  interface PalletCorporateActionsCaId extends Struct {
    readonly assetId: PolymeshPrimitivesAssetAssetId;
    readonly localId: u32;
  }

  /** @name PalletCorporateActionsDistribution (172) */
  interface PalletCorporateActionsDistribution extends Struct {
    readonly from: PolymeshPrimitivesIdentityIdPortfolioId;
    readonly currency: PolymeshPrimitivesAssetAssetId;
    readonly perShare: u128;
    readonly amount: u128;
    readonly remaining: u128;
    readonly reclaimed: bool;
    readonly paymentAt: u64;
    readonly expiresAt: Option<u64>;
  }

  /** @name PalletAssetCheckpointPalletEvent (174) */
  interface PalletAssetCheckpointPalletEvent extends Enum {
    readonly isCheckpointCreated: boolean;
    readonly asCheckpointCreated: ITuple<[Option<PolymeshPrimitivesIdentityId>, PolymeshPrimitivesAssetAssetId, u64, u128, u64]>;
    readonly isMaximumSchedulesComplexityChanged: boolean;
    readonly asMaximumSchedulesComplexityChanged: ITuple<[PolymeshPrimitivesIdentityId, u64]>;
    readonly isScheduleCreated: boolean;
    readonly asScheduleCreated: ITuple<[PolymeshPrimitivesIdentityId, PolymeshPrimitivesAssetAssetId, u64, PolymeshPrimitivesCheckpointScheduleCheckpoints]>;
    readonly isScheduleRemoved: boolean;
    readonly asScheduleRemoved: ITuple<[PolymeshPrimitivesIdentityId, PolymeshPrimitivesAssetAssetId, u64, PolymeshPrimitivesCheckpointScheduleCheckpoints]>;
    readonly type: 'CheckpointCreated' | 'MaximumSchedulesComplexityChanged' | 'ScheduleCreated' | 'ScheduleRemoved';
  }

  /** @name PolymeshPrimitivesCheckpointScheduleCheckpoints (177) */
  interface PolymeshPrimitivesCheckpointScheduleCheckpoints extends Struct {
    readonly pending: BTreeSet<u64>;
  }

  /** @name PalletComplianceManagerEvent (180) */
  interface PalletComplianceManagerEvent extends Enum {
    readonly isComplianceRequirementCreated: boolean;
    readonly asComplianceRequirementCreated: ITuple<[PolymeshPrimitivesIdentityId, PolymeshPrimitivesAssetAssetId, PolymeshPrimitivesComplianceManagerComplianceRequirement]>;
    readonly isComplianceRequirementRemoved: boolean;
    readonly asComplianceRequirementRemoved: ITuple<[PolymeshPrimitivesIdentityId, PolymeshPrimitivesAssetAssetId, u32]>;
    readonly isAssetComplianceReplaced: boolean;
    readonly asAssetComplianceReplaced: ITuple<[PolymeshPrimitivesIdentityId, PolymeshPrimitivesAssetAssetId, Vec<PolymeshPrimitivesComplianceManagerComplianceRequirement>]>;
    readonly isAssetComplianceReset: boolean;
    readonly asAssetComplianceReset: ITuple<[PolymeshPrimitivesIdentityId, PolymeshPrimitivesAssetAssetId]>;
    readonly isAssetComplianceResumed: boolean;
    readonly asAssetComplianceResumed: ITuple<[PolymeshPrimitivesIdentityId, PolymeshPrimitivesAssetAssetId]>;
    readonly isAssetCompliancePaused: boolean;
    readonly asAssetCompliancePaused: ITuple<[PolymeshPrimitivesIdentityId, PolymeshPrimitivesAssetAssetId]>;
    readonly isComplianceRequirementChanged: boolean;
    readonly asComplianceRequirementChanged: ITuple<[PolymeshPrimitivesIdentityId, PolymeshPrimitivesAssetAssetId, PolymeshPrimitivesComplianceManagerComplianceRequirement]>;
    readonly isTrustedDefaultClaimIssuerAdded: boolean;
    readonly asTrustedDefaultClaimIssuerAdded: ITuple<[PolymeshPrimitivesIdentityId, PolymeshPrimitivesAssetAssetId, PolymeshPrimitivesConditionTrustedIssuer]>;
    readonly isTrustedDefaultClaimIssuerRemoved: boolean;
    readonly asTrustedDefaultClaimIssuerRemoved: ITuple<[PolymeshPrimitivesIdentityId, PolymeshPrimitivesAssetAssetId, PolymeshPrimitivesIdentityId]>;
    readonly type: 'ComplianceRequirementCreated' | 'ComplianceRequirementRemoved' | 'AssetComplianceReplaced' | 'AssetComplianceReset' | 'AssetComplianceResumed' | 'AssetCompliancePaused' | 'ComplianceRequirementChanged' | 'TrustedDefaultClaimIssuerAdded' | 'TrustedDefaultClaimIssuerRemoved';
  }

  /** @name PolymeshPrimitivesComplianceManagerComplianceRequirement (181) */
  interface PolymeshPrimitivesComplianceManagerComplianceRequirement extends Struct {
    readonly senderConditions: Vec<PolymeshPrimitivesCondition>;
    readonly receiverConditions: Vec<PolymeshPrimitivesCondition>;
    readonly id: u32;
  }

  /** @name PolymeshPrimitivesCondition (183) */
  interface PolymeshPrimitivesCondition extends Struct {
    readonly conditionType: PolymeshPrimitivesConditionConditionType;
    readonly issuers: Vec<PolymeshPrimitivesConditionTrustedIssuer>;
  }

  /** @name PolymeshPrimitivesConditionConditionType (184) */
  interface PolymeshPrimitivesConditionConditionType extends Enum {
    readonly isIsPresent: boolean;
    readonly asIsPresent: PolymeshPrimitivesIdentityClaimClaim;
    readonly isIsAbsent: boolean;
    readonly asIsAbsent: PolymeshPrimitivesIdentityClaimClaim;
    readonly isIsAnyOf: boolean;
    readonly asIsAnyOf: Vec<PolymeshPrimitivesIdentityClaimClaim>;
    readonly isIsNoneOf: boolean;
    readonly asIsNoneOf: Vec<PolymeshPrimitivesIdentityClaimClaim>;
    readonly isIsIdentity: boolean;
    readonly asIsIdentity: PolymeshPrimitivesConditionTargetIdentity;
    readonly type: 'IsPresent' | 'IsAbsent' | 'IsAnyOf' | 'IsNoneOf' | 'IsIdentity';
  }

  /** @name PolymeshPrimitivesConditionTargetIdentity (186) */
  interface PolymeshPrimitivesConditionTargetIdentity extends Enum {
    readonly isExternalAgent: boolean;
    readonly isSpecific: boolean;
    readonly asSpecific: PolymeshPrimitivesIdentityId;
    readonly type: 'ExternalAgent' | 'Specific';
  }

  /** @name PolymeshPrimitivesConditionTrustedIssuer (188) */
  interface PolymeshPrimitivesConditionTrustedIssuer extends Struct {
    readonly issuer: PolymeshPrimitivesIdentityId;
    readonly trustedFor: PolymeshPrimitivesConditionTrustedFor;
  }

  /** @name PolymeshPrimitivesConditionTrustedFor (189) */
  interface PolymeshPrimitivesConditionTrustedFor extends Enum {
    readonly isAny: boolean;
    readonly isSpecific: boolean;
    readonly asSpecific: Vec<PolymeshPrimitivesIdentityClaimClaimType>;
    readonly type: 'Any' | 'Specific';
  }

  /** @name PolymeshPrimitivesIdentityClaimClaimType (191) */
  interface PolymeshPrimitivesIdentityClaimClaimType extends Enum {
    readonly isAccredited: boolean;
    readonly isAffiliate: boolean;
    readonly isBuyLockup: boolean;
    readonly isSellLockup: boolean;
    readonly isCustomerDueDiligence: boolean;
    readonly isKnowYourCustomer: boolean;
    readonly isJurisdiction: boolean;
    readonly isExempted: boolean;
    readonly isBlocked: boolean;
    readonly isCustom: boolean;
    readonly asCustom: u32;
    readonly type: 'Accredited' | 'Affiliate' | 'BuyLockup' | 'SellLockup' | 'CustomerDueDiligence' | 'KnowYourCustomer' | 'Jurisdiction' | 'Exempted' | 'Blocked' | 'Custom';
  }

  /** @name PalletCorporateActionsEvent (193) */
  interface PalletCorporateActionsEvent extends Enum {
    readonly isMaxDetailsLengthChanged: boolean;
    readonly asMaxDetailsLengthChanged: ITuple<[PolymeshPrimitivesIdentityId, u32]>;
    readonly isDefaultTargetIdentitiesChanged: boolean;
    readonly asDefaultTargetIdentitiesChanged: ITuple<[PolymeshPrimitivesIdentityId, PolymeshPrimitivesAssetAssetId, PalletCorporateActionsTargetIdentities]>;
    readonly isDefaultWithholdingTaxChanged: boolean;
    readonly asDefaultWithholdingTaxChanged: ITuple<[PolymeshPrimitivesIdentityId, PolymeshPrimitivesAssetAssetId, Permill]>;
    readonly isDidWithholdingTaxChanged: boolean;
    readonly asDidWithholdingTaxChanged: ITuple<[PolymeshPrimitivesIdentityId, PolymeshPrimitivesAssetAssetId, PolymeshPrimitivesIdentityId, Option<Permill>]>;
    readonly isCaInitiated: boolean;
    readonly asCaInitiated: ITuple<[PolymeshPrimitivesEventOnly, PalletCorporateActionsCaId, PalletCorporateActionsCorporateAction, Bytes]>;
    readonly isCaLinkedToDoc: boolean;
    readonly asCaLinkedToDoc: ITuple<[PolymeshPrimitivesIdentityId, PalletCorporateActionsCaId, Vec<u32>]>;
    readonly isCaRemoved: boolean;
    readonly asCaRemoved: ITuple<[PolymeshPrimitivesEventOnly, PalletCorporateActionsCaId]>;
    readonly isRecordDateChanged: boolean;
    readonly asRecordDateChanged: ITuple<[PolymeshPrimitivesEventOnly, PalletCorporateActionsCaId, PalletCorporateActionsCorporateAction]>;
    readonly type: 'MaxDetailsLengthChanged' | 'DefaultTargetIdentitiesChanged' | 'DefaultWithholdingTaxChanged' | 'DidWithholdingTaxChanged' | 'CaInitiated' | 'CaLinkedToDoc' | 'CaRemoved' | 'RecordDateChanged';
  }

  /** @name PalletCorporateActionsTargetIdentities (194) */
  interface PalletCorporateActionsTargetIdentities extends Struct {
    readonly identities: Vec<PolymeshPrimitivesIdentityId>;
    readonly treatment: PalletCorporateActionsTargetTreatment;
  }

  /** @name PalletCorporateActionsTargetTreatment (195) */
  interface PalletCorporateActionsTargetTreatment extends Enum {
    readonly isInclude: boolean;
    readonly isExclude: boolean;
    readonly type: 'Include' | 'Exclude';
  }

  /** @name PalletCorporateActionsCorporateAction (197) */
  interface PalletCorporateActionsCorporateAction extends Struct {
    readonly kind: PalletCorporateActionsCaKind;
    readonly declDate: u64;
    readonly recordDate: Option<PalletCorporateActionsRecordDate>;
    readonly targets: PalletCorporateActionsTargetIdentities;
    readonly defaultWithholdingTax: Permill;
    readonly withholdingTax: Vec<ITuple<[PolymeshPrimitivesIdentityId, Permill]>>;
  }

  /** @name PalletCorporateActionsCaKind (198) */
  interface PalletCorporateActionsCaKind extends Enum {
    readonly isPredictableBenefit: boolean;
    readonly isUnpredictableBenefit: boolean;
    readonly isIssuerNotice: boolean;
    readonly isReorganization: boolean;
    readonly isOther: boolean;
    readonly type: 'PredictableBenefit' | 'UnpredictableBenefit' | 'IssuerNotice' | 'Reorganization' | 'Other';
  }

  /** @name PalletCorporateActionsRecordDate (200) */
  interface PalletCorporateActionsRecordDate extends Struct {
    readonly date: u64;
    readonly checkpoint: PalletCorporateActionsCaCheckpoint;
  }

  /** @name PalletCorporateActionsCaCheckpoint (201) */
  interface PalletCorporateActionsCaCheckpoint extends Enum {
    readonly isScheduled: boolean;
    readonly asScheduled: ITuple<[u64, u64]>;
    readonly isExisting: boolean;
    readonly asExisting: u64;
    readonly type: 'Scheduled' | 'Existing';
  }

  /** @name PalletCorporateActionsBallotPalletEvent (206) */
  interface PalletCorporateActionsBallotPalletEvent extends Enum {
    readonly isCreated: boolean;
    readonly asCreated: ITuple<[PolymeshPrimitivesIdentityId, PalletCorporateActionsCaId, PalletCorporateActionsBallotBallotTimeRange, PalletCorporateActionsBallotBallotMeta, bool]>;
    readonly isVoteCast: boolean;
    readonly asVoteCast: ITuple<[PolymeshPrimitivesIdentityId, PalletCorporateActionsCaId, Vec<PalletCorporateActionsBallotBallotVote>]>;
    readonly isRangeChanged: boolean;
    readonly asRangeChanged: ITuple<[PolymeshPrimitivesIdentityId, PalletCorporateActionsCaId, PalletCorporateActionsBallotBallotTimeRange]>;
    readonly isMetaChanged: boolean;
    readonly asMetaChanged: ITuple<[PolymeshPrimitivesIdentityId, PalletCorporateActionsCaId, PalletCorporateActionsBallotBallotMeta]>;
    readonly isRcvChanged: boolean;
    readonly asRcvChanged: ITuple<[PolymeshPrimitivesIdentityId, PalletCorporateActionsCaId, bool]>;
    readonly isRemoved: boolean;
    readonly asRemoved: ITuple<[PolymeshPrimitivesEventOnly, PalletCorporateActionsCaId]>;
    readonly type: 'Created' | 'VoteCast' | 'RangeChanged' | 'MetaChanged' | 'RcvChanged' | 'Removed';
  }

  /** @name PalletCorporateActionsBallotBallotTimeRange (207) */
  interface PalletCorporateActionsBallotBallotTimeRange extends Struct {
    readonly start: u64;
    readonly end: u64;
  }

  /** @name PalletCorporateActionsBallotBallotMeta (208) */
  interface PalletCorporateActionsBallotBallotMeta extends Struct {
    readonly title: Bytes;
    readonly motions: Vec<PalletCorporateActionsBallotMotion>;
  }

  /** @name PalletCorporateActionsBallotMotion (211) */
  interface PalletCorporateActionsBallotMotion extends Struct {
    readonly title: Bytes;
    readonly infoLink: Bytes;
    readonly choices: Vec<Bytes>;
  }

  /** @name PalletCorporateActionsBallotBallotVote (217) */
  interface PalletCorporateActionsBallotBallotVote extends Struct {
    readonly power: u128;
    readonly fallback: Option<u16>;
  }

  /** @name PalletPipsEvent (220) */
  interface PalletPipsEvent extends Enum {
    readonly isHistoricalPipsPruned: boolean;
    readonly asHistoricalPipsPruned: ITuple<[PolymeshPrimitivesIdentityId, bool, bool]>;
    readonly isProposalCreated: boolean;
    readonly asProposalCreated: ITuple<[PolymeshPrimitivesIdentityId, PalletPipsProposer, u32, u128, Option<Bytes>, Option<Bytes>, PolymeshPrimitivesMaybeBlock, PalletPipsProposalData]>;
    readonly isProposalStateUpdated: boolean;
    readonly asProposalStateUpdated: ITuple<[PolymeshPrimitivesIdentityId, u32, PalletPipsProposalState]>;
    readonly isVoted: boolean;
    readonly asVoted: ITuple<[PolymeshPrimitivesIdentityId, AccountId32, u32, bool, u128]>;
    readonly isPipClosed: boolean;
    readonly asPipClosed: ITuple<[PolymeshPrimitivesIdentityId, u32, bool]>;
    readonly isExecutionScheduled: boolean;
    readonly asExecutionScheduled: ITuple<[PolymeshPrimitivesIdentityId, u32, u32]>;
    readonly isDefaultEnactmentPeriodChanged: boolean;
    readonly asDefaultEnactmentPeriodChanged: ITuple<[PolymeshPrimitivesIdentityId, u32, u32]>;
    readonly isMinimumProposalDepositChanged: boolean;
    readonly asMinimumProposalDepositChanged: ITuple<[PolymeshPrimitivesIdentityId, u128, u128]>;
    readonly isPendingPipExpiryChanged: boolean;
    readonly asPendingPipExpiryChanged: ITuple<[PolymeshPrimitivesIdentityId, PolymeshPrimitivesMaybeBlock, PolymeshPrimitivesMaybeBlock]>;
    readonly isMaxPipSkipCountChanged: boolean;
    readonly asMaxPipSkipCountChanged: ITuple<[PolymeshPrimitivesIdentityId, u8, u8]>;
    readonly isActivePipLimitChanged: boolean;
    readonly asActivePipLimitChanged: ITuple<[PolymeshPrimitivesIdentityId, u32, u32]>;
    readonly isProposalRefund: boolean;
    readonly asProposalRefund: ITuple<[PolymeshPrimitivesIdentityId, u32, u128]>;
    readonly isSnapshotCleared: boolean;
    readonly asSnapshotCleared: ITuple<[PolymeshPrimitivesIdentityId, u32]>;
    readonly isSnapshotTaken: boolean;
    readonly asSnapshotTaken: ITuple<[PolymeshPrimitivesIdentityId, u32, Vec<PalletPipsSnapshottedPip>]>;
    readonly isPipSkipped: boolean;
    readonly asPipSkipped: ITuple<[PolymeshPrimitivesIdentityId, u32, u8]>;
    readonly isSnapshotResultsEnacted: boolean;
    readonly asSnapshotResultsEnacted: ITuple<[PolymeshPrimitivesIdentityId, Option<u32>, Vec<ITuple<[u32, u8]>>, Vec<u32>, Vec<u32>]>;
    readonly isExecutionSchedulingFailed: boolean;
    readonly asExecutionSchedulingFailed: ITuple<[PolymeshPrimitivesIdentityId, u32, u32]>;
    readonly isExpiryScheduled: boolean;
    readonly asExpiryScheduled: ITuple<[PolymeshPrimitivesIdentityId, u32, u32]>;
    readonly isExpirySchedulingFailed: boolean;
    readonly asExpirySchedulingFailed: ITuple<[PolymeshPrimitivesIdentityId, u32, u32]>;
    readonly isExecutionCancellingFailed: boolean;
    readonly asExecutionCancellingFailed: u32;
    readonly type: 'HistoricalPipsPruned' | 'ProposalCreated' | 'ProposalStateUpdated' | 'Voted' | 'PipClosed' | 'ExecutionScheduled' | 'DefaultEnactmentPeriodChanged' | 'MinimumProposalDepositChanged' | 'PendingPipExpiryChanged' | 'MaxPipSkipCountChanged' | 'ActivePipLimitChanged' | 'ProposalRefund' | 'SnapshotCleared' | 'SnapshotTaken' | 'PipSkipped' | 'SnapshotResultsEnacted' | 'ExecutionSchedulingFailed' | 'ExpiryScheduled' | 'ExpirySchedulingFailed' | 'ExecutionCancellingFailed';
  }

  /** @name PalletPipsProposer (221) */
  interface PalletPipsProposer extends Enum {
    readonly isCommunity: boolean;
    readonly asCommunity: AccountId32;
    readonly isCommittee: boolean;
    readonly asCommittee: PalletPipsCommittee;
    readonly type: 'Community' | 'Committee';
  }

  /** @name PalletPipsCommittee (222) */
  interface PalletPipsCommittee extends Enum {
    readonly isTechnical: boolean;
    readonly isUpgrade: boolean;
    readonly type: 'Technical' | 'Upgrade';
  }

  /** @name PalletPipsProposalData (226) */
  interface PalletPipsProposalData extends Enum {
    readonly isHash: boolean;
    readonly asHash: H256;
    readonly isProposal: boolean;
    readonly asProposal: Bytes;
    readonly type: 'Hash' | 'Proposal';
  }

  /** @name PalletPipsProposalState (227) */
  interface PalletPipsProposalState extends Enum {
    readonly isPending: boolean;
    readonly isRejected: boolean;
    readonly isScheduled: boolean;
    readonly isFailed: boolean;
    readonly isExecuted: boolean;
    readonly isExpired: boolean;
    readonly type: 'Pending' | 'Rejected' | 'Scheduled' | 'Failed' | 'Executed' | 'Expired';
  }

  /** @name PalletPipsSnapshottedPip (230) */
  interface PalletPipsSnapshottedPip extends Struct {
    readonly id: u32;
    readonly weight: ITuple<[bool, u128]>;
  }

  /** @name PalletPortfolioEvent (236) */
  interface PalletPortfolioEvent extends Enum {
    readonly isPortfolioCreated: boolean;
    readonly asPortfolioCreated: ITuple<[PolymeshPrimitivesIdentityId, u64, Bytes]>;
    readonly isPortfolioDeleted: boolean;
    readonly asPortfolioDeleted: ITuple<[PolymeshPrimitivesIdentityId, u64]>;
    readonly isPortfolioRenamed: boolean;
    readonly asPortfolioRenamed: ITuple<[PolymeshPrimitivesIdentityId, u64, Bytes]>;
    readonly isUserPortfolios: boolean;
    readonly asUserPortfolios: ITuple<[PolymeshPrimitivesIdentityId, Vec<ITuple<[u64, Bytes]>>]>;
    readonly isPortfolioCustodianChanged: boolean;
    readonly asPortfolioCustodianChanged: ITuple<[PolymeshPrimitivesIdentityId, PolymeshPrimitivesIdentityIdPortfolioId, PolymeshPrimitivesIdentityId]>;
    readonly isFundsMovedBetweenPortfolios: boolean;
    readonly asFundsMovedBetweenPortfolios: ITuple<[PolymeshPrimitivesIdentityId, PolymeshPrimitivesIdentityIdPortfolioId, PolymeshPrimitivesIdentityIdPortfolioId, PolymeshPrimitivesPortfolioFundDescription, Option<PolymeshPrimitivesMemo>]>;
    readonly isPreApprovedPortfolio: boolean;
    readonly asPreApprovedPortfolio: ITuple<[PolymeshPrimitivesIdentityId, PolymeshPrimitivesIdentityIdPortfolioId, PolymeshPrimitivesAssetAssetId]>;
    readonly isRevokePreApprovedPortfolio: boolean;
    readonly asRevokePreApprovedPortfolio: ITuple<[PolymeshPrimitivesIdentityId, PolymeshPrimitivesIdentityIdPortfolioId, PolymeshPrimitivesAssetAssetId]>;
    readonly isAllowIdentityToCreatePortfolios: boolean;
    readonly asAllowIdentityToCreatePortfolios: ITuple<[PolymeshPrimitivesIdentityId, PolymeshPrimitivesIdentityId]>;
    readonly isRevokeCreatePortfoliosPermission: boolean;
    readonly asRevokeCreatePortfoliosPermission: ITuple<[PolymeshPrimitivesIdentityId, PolymeshPrimitivesIdentityId]>;
    readonly type: 'PortfolioCreated' | 'PortfolioDeleted' | 'PortfolioRenamed' | 'UserPortfolios' | 'PortfolioCustodianChanged' | 'FundsMovedBetweenPortfolios' | 'PreApprovedPortfolio' | 'RevokePreApprovedPortfolio' | 'AllowIdentityToCreatePortfolios' | 'RevokeCreatePortfoliosPermission';
  }

  /** @name PolymeshPrimitivesPortfolioFundDescription (240) */
  interface PolymeshPrimitivesPortfolioFundDescription extends Enum {
    readonly isFungible: boolean;
    readonly asFungible: {
      readonly assetId: PolymeshPrimitivesAssetAssetId;
      readonly amount: u128;
    } & Struct;
    readonly isNonFungible: boolean;
    readonly asNonFungible: PolymeshPrimitivesNftNfTs;
    readonly type: 'Fungible' | 'NonFungible';
  }

  /** @name PolymeshPrimitivesNftNfTs (241) */
  interface PolymeshPrimitivesNftNfTs extends Struct {
    readonly assetId: PolymeshPrimitivesAssetAssetId;
    readonly ids: Vec<u64>;
  }

  /** @name PalletProtocolFeeEvent (244) */
  interface PalletProtocolFeeEvent extends Enum {
    readonly isFeeSet: boolean;
    readonly asFeeSet: ITuple<[PolymeshPrimitivesIdentityId, u128]>;
    readonly isCoefficientSet: boolean;
    readonly asCoefficientSet: ITuple<[PolymeshPrimitivesIdentityId, PolymeshPrimitivesPosRatio]>;
    readonly isFeeCharged: boolean;
    readonly asFeeCharged: ITuple<[AccountId32, u128]>;
    readonly type: 'FeeSet' | 'CoefficientSet' | 'FeeCharged';
  }

  /** @name PolymeshPrimitivesPosRatio (245) */
  interface PolymeshPrimitivesPosRatio extends ITuple<[u32, u32]> {}

  /** @name PalletSchedulerEvent (246) */
  interface PalletSchedulerEvent extends Enum {
    readonly isScheduled: boolean;
    readonly asScheduled: {
      readonly when: u32;
      readonly index: u32;
    } & Struct;
    readonly isCanceled: boolean;
    readonly asCanceled: {
      readonly when: u32;
      readonly index: u32;
    } & Struct;
    readonly isDispatched: boolean;
    readonly asDispatched: {
      readonly task: ITuple<[u32, u32]>;
      readonly id: Option<U8aFixed>;
      readonly result: Result<Null, SpRuntimeDispatchError>;
    } & Struct;
    readonly isRetrySet: boolean;
    readonly asRetrySet: {
      readonly task: ITuple<[u32, u32]>;
      readonly id: Option<U8aFixed>;
      readonly period: u32;
      readonly retries: u8;
    } & Struct;
    readonly isRetryCancelled: boolean;
    readonly asRetryCancelled: {
      readonly task: ITuple<[u32, u32]>;
      readonly id: Option<U8aFixed>;
    } & Struct;
    readonly isCallUnavailable: boolean;
    readonly asCallUnavailable: {
      readonly task: ITuple<[u32, u32]>;
      readonly id: Option<U8aFixed>;
    } & Struct;
    readonly isPeriodicFailed: boolean;
    readonly asPeriodicFailed: {
      readonly task: ITuple<[u32, u32]>;
      readonly id: Option<U8aFixed>;
    } & Struct;
    readonly isRetryFailed: boolean;
    readonly asRetryFailed: {
      readonly task: ITuple<[u32, u32]>;
      readonly id: Option<U8aFixed>;
    } & Struct;
    readonly isPermanentlyOverweight: boolean;
    readonly asPermanentlyOverweight: {
      readonly task: ITuple<[u32, u32]>;
      readonly id: Option<U8aFixed>;
    } & Struct;
    readonly isAgendaIncomplete: boolean;
    readonly asAgendaIncomplete: {
      readonly when: u32;
    } & Struct;
    readonly type: 'Scheduled' | 'Canceled' | 'Dispatched' | 'RetrySet' | 'RetryCancelled' | 'CallUnavailable' | 'PeriodicFailed' | 'RetryFailed' | 'PermanentlyOverweight' | 'AgendaIncomplete';
  }

  /** @name PalletSettlementEvent (249) */
  interface PalletSettlementEvent extends Enum {
    readonly isVenueCreated: boolean;
    readonly asVenueCreated: ITuple<[PolymeshPrimitivesIdentityId, u64, Bytes, PolymeshPrimitivesSettlementVenueType]>;
    readonly isVenueDetailsUpdated: boolean;
    readonly asVenueDetailsUpdated: ITuple<[PolymeshPrimitivesIdentityId, u64, Bytes]>;
    readonly isVenueTypeUpdated: boolean;
    readonly asVenueTypeUpdated: ITuple<[PolymeshPrimitivesIdentityId, u64, PolymeshPrimitivesSettlementVenueType]>;
    readonly isInstructionAffirmed: boolean;
    readonly asInstructionAffirmed: ITuple<[PolymeshPrimitivesIdentityId, PolymeshPrimitivesAssetAssetHolder, u64]>;
    readonly isAffirmationWithdrawn: boolean;
    readonly asAffirmationWithdrawn: ITuple<[PolymeshPrimitivesIdentityId, PolymeshPrimitivesAssetAssetHolder, u64]>;
    readonly isInstructionRejected: boolean;
    readonly asInstructionRejected: ITuple<[PolymeshPrimitivesIdentityId, u64]>;
    readonly isReceiptClaimed: boolean;
    readonly asReceiptClaimed: ITuple<[PolymeshPrimitivesIdentityId, u64, u64, u64, AccountId32, Option<PolymeshPrimitivesSettlementReceiptMetadata>]>;
    readonly isVenueFiltering: boolean;
    readonly asVenueFiltering: ITuple<[PolymeshPrimitivesIdentityId, PolymeshPrimitivesAssetAssetId, bool]>;
    readonly isVenuesAllowed: boolean;
    readonly asVenuesAllowed: ITuple<[PolymeshPrimitivesIdentityId, PolymeshPrimitivesAssetAssetId, Vec<u64>]>;
    readonly isVenuesBlocked: boolean;
    readonly asVenuesBlocked: ITuple<[PolymeshPrimitivesIdentityId, PolymeshPrimitivesAssetAssetId, Vec<u64>]>;
    readonly isLegFailedExecution: boolean;
    readonly asLegFailedExecution: ITuple<[PolymeshPrimitivesIdentityId, u64, u64]>;
    readonly isInstructionExecuted: boolean;
    readonly asInstructionExecuted: ITuple<[PolymeshPrimitivesIdentityId, u64]>;
    readonly isVenueUnauthorized: boolean;
    readonly asVenueUnauthorized: ITuple<[PolymeshPrimitivesIdentityId, PolymeshPrimitivesAssetAssetId, u64]>;
    readonly isSchedulingFailed: boolean;
    readonly asSchedulingFailed: ITuple<[u64, SpRuntimeDispatchError]>;
    readonly isInstructionRescheduled: boolean;
    readonly asInstructionRescheduled: ITuple<[PolymeshPrimitivesIdentityId, u64]>;
    readonly isVenueSignersUpdated: boolean;
    readonly asVenueSignersUpdated: ITuple<[PolymeshPrimitivesIdentityId, u64, Vec<AccountId32>, bool]>;
    readonly isSettlementManuallyExecuted: boolean;
    readonly asSettlementManuallyExecuted: ITuple<[PolymeshPrimitivesIdentityId, u64]>;
    readonly isInstructionCreated: boolean;
    readonly asInstructionCreated: ITuple<[PolymeshPrimitivesIdentityId, Option<u64>, u64, PolymeshPrimitivesSettlementSettlementType, Option<u64>, Option<u64>, Vec<PolymeshPrimitivesSettlementLeg>, Option<PolymeshPrimitivesMemo>]>;
    readonly isFailedToExecuteInstruction: boolean;
    readonly asFailedToExecuteInstruction: ITuple<[u64, SpRuntimeDispatchError]>;
    readonly isInstructionAutomaticallyAffirmed: boolean;
    readonly asInstructionAutomaticallyAffirmed: ITuple<[PolymeshPrimitivesIdentityId, PolymeshPrimitivesAssetAssetHolder, u64]>;
    readonly isMediatorAffirmationReceived: boolean;
    readonly asMediatorAffirmationReceived: ITuple<[PolymeshPrimitivesIdentityId, u64, Option<u64>]>;
    readonly isMediatorAffirmationWithdrawn: boolean;
    readonly asMediatorAffirmationWithdrawn: ITuple<[PolymeshPrimitivesIdentityId, u64]>;
    readonly isInstructionMediators: boolean;
    readonly asInstructionMediators: ITuple<[u64, BTreeSet<PolymeshPrimitivesIdentityId>]>;
    readonly isInstructionLocked: boolean;
    readonly asInstructionLocked: ITuple<[PolymeshPrimitivesIdentityId, u64]>;
    readonly type: 'VenueCreated' | 'VenueDetailsUpdated' | 'VenueTypeUpdated' | 'InstructionAffirmed' | 'AffirmationWithdrawn' | 'InstructionRejected' | 'ReceiptClaimed' | 'VenueFiltering' | 'VenuesAllowed' | 'VenuesBlocked' | 'LegFailedExecution' | 'InstructionExecuted' | 'VenueUnauthorized' | 'SchedulingFailed' | 'InstructionRescheduled' | 'VenueSignersUpdated' | 'SettlementManuallyExecuted' | 'InstructionCreated' | 'FailedToExecuteInstruction' | 'InstructionAutomaticallyAffirmed' | 'MediatorAffirmationReceived' | 'MediatorAffirmationWithdrawn' | 'InstructionMediators' | 'InstructionLocked';
  }

  /** @name PolymeshPrimitivesSettlementVenueType (252) */
  interface PolymeshPrimitivesSettlementVenueType extends Enum {
    readonly isOther: boolean;
    readonly isDistribution: boolean;
    readonly isSto: boolean;
    readonly isExchange: boolean;
    readonly type: 'Other' | 'Distribution' | 'Sto' | 'Exchange';
  }

  /** @name PolymeshPrimitivesSettlementReceiptMetadata (255) */
  interface PolymeshPrimitivesSettlementReceiptMetadata extends U8aFixed {}

  /** @name PolymeshPrimitivesSettlementSettlementType (258) */
  interface PolymeshPrimitivesSettlementSettlementType extends Enum {
    readonly isSettleOnAffirmation: boolean;
    readonly isSettleOnBlock: boolean;
    readonly asSettleOnBlock: u32;
    readonly isSettleManual: boolean;
    readonly asSettleManual: u32;
    readonly isSettleAfterLock: boolean;
    readonly type: 'SettleOnAffirmation' | 'SettleOnBlock' | 'SettleManual' | 'SettleAfterLock';
  }

  /** @name PolymeshPrimitivesSettlementLeg (260) */
  interface PolymeshPrimitivesSettlementLeg extends Enum {
    readonly isFungible: boolean;
    readonly asFungible: {
      readonly sender: PolymeshPrimitivesAssetAssetHolder;
      readonly receiver: PolymeshPrimitivesAssetAssetHolder;
      readonly assetId: PolymeshPrimitivesAssetAssetId;
      readonly amount: u128;
    } & Struct;
    readonly isNonFungible: boolean;
    readonly asNonFungible: {
      readonly sender: PolymeshPrimitivesAssetAssetHolder;
      readonly receiver: PolymeshPrimitivesAssetAssetHolder;
      readonly nfts: PolymeshPrimitivesNftNfTs;
    } & Struct;
    readonly isOffChain: boolean;
    readonly asOffChain: {
      readonly senderIdentity: PolymeshPrimitivesIdentityId;
      readonly receiverIdentity: PolymeshPrimitivesIdentityId;
      readonly ticker: PolymeshPrimitivesTicker;
      readonly amount: u128;
    } & Struct;
    readonly type: 'Fungible' | 'NonFungible' | 'OffChain';
  }

  /** @name PalletStatisticsEvent (261) */
  interface PalletStatisticsEvent extends Enum {
    readonly isStatTypesAdded: boolean;
    readonly asStatTypesAdded: ITuple<[PolymeshPrimitivesIdentityId, PolymeshPrimitivesAssetAssetId, Vec<PolymeshPrimitivesStatisticsStatType>]>;
    readonly isStatTypesRemoved: boolean;
    readonly asStatTypesRemoved: ITuple<[PolymeshPrimitivesIdentityId, PolymeshPrimitivesAssetAssetId, Vec<PolymeshPrimitivesStatisticsStatType>]>;
    readonly isAssetStatsUpdated: boolean;
    readonly asAssetStatsUpdated: ITuple<[PolymeshPrimitivesIdentityId, PolymeshPrimitivesAssetAssetId, PolymeshPrimitivesStatisticsStatType, Vec<PolymeshPrimitivesStatisticsStatUpdate>]>;
    readonly isSetAssetTransferCompliance: boolean;
    readonly asSetAssetTransferCompliance: ITuple<[PolymeshPrimitivesIdentityId, PolymeshPrimitivesAssetAssetId, Vec<PolymeshPrimitivesTransferComplianceTransferCondition>]>;
    readonly isTransferConditionExemptionsAdded: boolean;
    readonly asTransferConditionExemptionsAdded: ITuple<[PolymeshPrimitivesIdentityId, PolymeshPrimitivesTransferComplianceTransferConditionExemptKey, Vec<PolymeshPrimitivesIdentityId>]>;
    readonly isTransferConditionExemptionsRemoved: boolean;
    readonly asTransferConditionExemptionsRemoved: ITuple<[PolymeshPrimitivesIdentityId, PolymeshPrimitivesTransferComplianceTransferConditionExemptKey, Vec<PolymeshPrimitivesIdentityId>]>;
    readonly type: 'StatTypesAdded' | 'StatTypesRemoved' | 'AssetStatsUpdated' | 'SetAssetTransferCompliance' | 'TransferConditionExemptionsAdded' | 'TransferConditionExemptionsRemoved';
  }

  /** @name PolymeshPrimitivesStatisticsStatType (263) */
  interface PolymeshPrimitivesStatisticsStatType extends Struct {
    readonly operationType: PolymeshPrimitivesStatisticsStatOpType;
    readonly claimIssuer: Option<ITuple<[PolymeshPrimitivesIdentityClaimClaimType, PolymeshPrimitivesIdentityId]>>;
  }

  /** @name PolymeshPrimitivesStatisticsStatOpType (264) */
  interface PolymeshPrimitivesStatisticsStatOpType extends Enum {
    readonly isCount: boolean;
    readonly isBalance: boolean;
    readonly type: 'Count' | 'Balance';
  }

  /** @name PolymeshPrimitivesStatisticsStatUpdate (268) */
  interface PolymeshPrimitivesStatisticsStatUpdate extends Struct {
    readonly key2: PolymeshPrimitivesStatisticsStat2ndKey;
    readonly value: Option<u128>;
  }

  /** @name PolymeshPrimitivesStatisticsStat2ndKey (269) */
  interface PolymeshPrimitivesStatisticsStat2ndKey extends Enum {
    readonly isNoClaimStat: boolean;
    readonly isClaim: boolean;
    readonly asClaim: PolymeshPrimitivesStatisticsStatClaim;
    readonly type: 'NoClaimStat' | 'Claim';
  }

  /** @name PolymeshPrimitivesStatisticsStatClaim (270) */
  interface PolymeshPrimitivesStatisticsStatClaim extends Enum {
    readonly isAccredited: boolean;
    readonly asAccredited: bool;
    readonly isAffiliate: boolean;
    readonly asAffiliate: bool;
    readonly isJurisdiction: boolean;
    readonly asJurisdiction: Option<PolymeshPrimitivesJurisdictionCountryCode>;
    readonly type: 'Accredited' | 'Affiliate' | 'Jurisdiction';
  }

  /** @name PolymeshPrimitivesTransferComplianceTransferCondition (274) */
  interface PolymeshPrimitivesTransferComplianceTransferCondition extends Enum {
    readonly isMaxInvestorCount: boolean;
    readonly asMaxInvestorCount: u64;
    readonly isMaxInvestorOwnership: boolean;
    readonly asMaxInvestorOwnership: Permill;
    readonly isClaimCount: boolean;
    readonly asClaimCount: ITuple<[PolymeshPrimitivesStatisticsStatClaim, PolymeshPrimitivesIdentityId, u64, Option<u64>]>;
    readonly isClaimOwnership: boolean;
    readonly asClaimOwnership: ITuple<[PolymeshPrimitivesStatisticsStatClaim, PolymeshPrimitivesIdentityId, Permill, Permill]>;
    readonly type: 'MaxInvestorCount' | 'MaxInvestorOwnership' | 'ClaimCount' | 'ClaimOwnership';
  }

  /** @name PolymeshPrimitivesTransferComplianceTransferConditionExemptKey (275) */
  interface PolymeshPrimitivesTransferComplianceTransferConditionExemptKey extends Struct {
    readonly assetId: PolymeshPrimitivesAssetAssetId;
    readonly op: PolymeshPrimitivesStatisticsStatOpType;
    readonly claimType: Option<PolymeshPrimitivesIdentityClaimClaimType>;
  }

  /** @name PalletStoEvent (277) */
  interface PalletStoEvent extends Enum {
    readonly isFundraiserCreated: boolean;
    readonly asFundraiserCreated: {
      readonly agentDid: PolymeshPrimitivesIdentityId;
      readonly offeringAsset: PolymeshPrimitivesAssetAssetId;
      readonly raisingAsset: PolymeshPrimitivesAssetAssetId;
      readonly fundraiserId: u64;
      readonly fundraiserName: Bytes;
      readonly fundraiser: PalletStoFundraiser;
    } & Struct;
    readonly isInvested: boolean;
    readonly asInvested: {
      readonly investorDid: PolymeshPrimitivesIdentityId;
      readonly offeringAsset: PolymeshPrimitivesAssetAssetId;
      readonly fundraiserId: u64;
      readonly fundingAsset: PalletStoFundingAsset;
      readonly offeringAmount: u128;
      readonly raiseAmount: u128;
    } & Struct;
    readonly isFundraiserFrozen: boolean;
    readonly asFundraiserFrozen: {
      readonly agentDid: PolymeshPrimitivesIdentityId;
      readonly offeringAsset: PolymeshPrimitivesAssetAssetId;
      readonly fundraiserId: u64;
    } & Struct;
    readonly isFundraiserUnfrozen: boolean;
    readonly asFundraiserUnfrozen: {
      readonly agentDid: PolymeshPrimitivesIdentityId;
      readonly offeringAsset: PolymeshPrimitivesAssetAssetId;
      readonly fundraiserId: u64;
    } & Struct;
    readonly isFundraiserWindowModified: boolean;
    readonly asFundraiserWindowModified: {
      readonly agentDid: PolymeshPrimitivesEventOnly;
      readonly offeringAsset: PolymeshPrimitivesAssetAssetId;
      readonly fundraiserId: u64;
      readonly oldStart: u64;
      readonly oldEnd: Option<u64>;
      readonly newStart: u64;
      readonly newEnd: Option<u64>;
    } & Struct;
    readonly isFundraiserClosed: boolean;
    readonly asFundraiserClosed: {
      readonly agentDid: PolymeshPrimitivesIdentityId;
      readonly offeringAsset: PolymeshPrimitivesAssetAssetId;
      readonly fundraiserId: u64;
    } & Struct;
    readonly isFundraiserOffchainFundingEnabled: boolean;
    readonly asFundraiserOffchainFundingEnabled: {
      readonly agentDid: PolymeshPrimitivesIdentityId;
      readonly offeringAsset: PolymeshPrimitivesAssetAssetId;
      readonly fundraiserId: u64;
      readonly ticker: PolymeshPrimitivesTicker;
    } & Struct;
    readonly type: 'FundraiserCreated' | 'Invested' | 'FundraiserFrozen' | 'FundraiserUnfrozen' | 'FundraiserWindowModified' | 'FundraiserClosed' | 'FundraiserOffchainFundingEnabled';
  }

  /** @name PalletStoFundraiser (280) */
  interface PalletStoFundraiser extends Struct {
    readonly creator: PolymeshPrimitivesIdentityId;
    readonly offeringPortfolio: PolymeshPrimitivesIdentityIdPortfolioId;
    readonly offeringAsset: PolymeshPrimitivesAssetAssetId;
    readonly raisingPortfolio: PolymeshPrimitivesIdentityIdPortfolioId;
    readonly raisingAsset: PolymeshPrimitivesAssetAssetId;
    readonly tiers: Vec<PalletStoFundraiserTier>;
    readonly venueId: u64;
    readonly start: u64;
    readonly end: Option<u64>;
    readonly status: PalletStoFundraiserStatus;
    readonly minimumInvestment: u128;
  }

  /** @name PalletStoFundraiserTier (282) */
  interface PalletStoFundraiserTier extends Struct {
    readonly total: u128;
    readonly price: u128;
    readonly remaining: u128;
  }

  /** @name PalletStoFundraiserStatus (283) */
  interface PalletStoFundraiserStatus extends Enum {
    readonly isLive: boolean;
    readonly isFrozen: boolean;
    readonly isClosed: boolean;
    readonly isClosedEarly: boolean;
    readonly type: 'Live' | 'Frozen' | 'Closed' | 'ClosedEarly';
  }

  /** @name PalletStoFundingAsset (284) */
  interface PalletStoFundingAsset extends Enum {
    readonly isOnChain: boolean;
    readonly asOnChain: PolymeshPrimitivesAssetAssetId;
    readonly isOffChain: boolean;
    readonly asOffChain: PolymeshPrimitivesTicker;
    readonly type: 'OnChain' | 'OffChain';
  }

  /** @name PalletTreasuryEvent (285) */
  interface PalletTreasuryEvent extends Enum {
    readonly isTreasuryDisbursement: boolean;
    readonly asTreasuryDisbursement: ITuple<[PolymeshPrimitivesIdentityId, PolymeshPrimitivesIdentityId, AccountId32, u128]>;
    readonly isTreasuryDisbursementFailed: boolean;
    readonly asTreasuryDisbursementFailed: ITuple<[PolymeshPrimitivesIdentityId, PolymeshPrimitivesIdentityId, AccountId32, u128]>;
    readonly isTreasuryReimbursement: boolean;
    readonly asTreasuryReimbursement: ITuple<[PolymeshPrimitivesIdentityId, u128]>;
    readonly type: 'TreasuryDisbursement' | 'TreasuryDisbursementFailed' | 'TreasuryReimbursement';
  }

  /** @name PalletUtilityEvent (286) */
  interface PalletUtilityEvent extends Enum {
    readonly isBatchInterrupted: boolean;
    readonly asBatchInterrupted: {
      readonly index: u32;
      readonly error: SpRuntimeDispatchError;
    } & Struct;
    readonly isBatchCompleted: boolean;
    readonly isBatchCompletedWithErrors: boolean;
    readonly isItemCompleted: boolean;
    readonly isItemFailed: boolean;
    readonly asItemFailed: {
      readonly error: SpRuntimeDispatchError;
    } & Struct;
    readonly isDispatchedAs: boolean;
    readonly asDispatchedAs: {
      readonly result: Result<Null, SpRuntimeDispatchError>;
    } & Struct;
    readonly type: 'BatchInterrupted' | 'BatchCompleted' | 'BatchCompletedWithErrors' | 'ItemCompleted' | 'ItemFailed' | 'DispatchedAs';
  }

  /** @name PalletExternalAgentsEvent (287) */
  interface PalletExternalAgentsEvent extends Enum {
    readonly isGroupCreated: boolean;
    readonly asGroupCreated: ITuple<[PolymeshPrimitivesEventOnly, PolymeshPrimitivesAssetAssetId, u32, PolymeshPrimitivesSecondaryKeyExtrinsicPermissions]>;
    readonly isGroupPermissionsUpdated: boolean;
    readonly asGroupPermissionsUpdated: ITuple<[PolymeshPrimitivesEventOnly, PolymeshPrimitivesAssetAssetId, u32, PolymeshPrimitivesSecondaryKeyExtrinsicPermissions]>;
    readonly isAgentAdded: boolean;
    readonly asAgentAdded: ITuple<[PolymeshPrimitivesEventOnly, PolymeshPrimitivesAssetAssetId, PolymeshPrimitivesAgentAgentGroup]>;
    readonly isAgentRemoved: boolean;
    readonly asAgentRemoved: ITuple<[PolymeshPrimitivesEventOnly, PolymeshPrimitivesAssetAssetId, PolymeshPrimitivesIdentityId]>;
    readonly isGroupChanged: boolean;
    readonly asGroupChanged: ITuple<[PolymeshPrimitivesEventOnly, PolymeshPrimitivesAssetAssetId, PolymeshPrimitivesIdentityId, PolymeshPrimitivesAgentAgentGroup]>;
    readonly type: 'GroupCreated' | 'GroupPermissionsUpdated' | 'AgentAdded' | 'AgentRemoved' | 'GroupChanged';
  }

  /** @name PalletRelayerEvent (288) */
  interface PalletRelayerEvent extends Enum {
    readonly isApprovedSubsidy: boolean;
    readonly asApprovedSubsidy: {
      readonly userKey: AccountId32;
      readonly payingKey: AccountId32;
      readonly initialPolyxLimit: u128;
    } & Struct;
    readonly isAcceptedSubsidy: boolean;
    readonly asAcceptedSubsidy: {
      readonly userKey: AccountId32;
      readonly payingKey: AccountId32;
      readonly initialPolyxLimit: u128;
    } & Struct;
    readonly isRemovedSubsidy: boolean;
    readonly asRemovedSubsidy: {
      readonly userKey: AccountId32;
      readonly payingKey: AccountId32;
      readonly remaining: u128;
    } & Struct;
    readonly isRemovedPendingSubsidy: boolean;
    readonly asRemovedPendingSubsidy: {
      readonly userKey: AccountId32;
      readonly payingKey: AccountId32;
      readonly initialPolyxLimit: u128;
    } & Struct;
    readonly isUpdatedPolyxLimit: boolean;
    readonly asUpdatedPolyxLimit: {
      readonly userKey: AccountId32;
      readonly payingKey: AccountId32;
      readonly remaining: u128;
      readonly oldRemaining: u128;
    } & Struct;
    readonly isSubsidyDebited: boolean;
    readonly asSubsidyDebited: {
      readonly userKey: AccountId32;
      readonly payingKey: AccountId32;
      readonly amount: u128;
    } & Struct;
    readonly isRelayedTx: boolean;
    readonly asRelayedTx: {
      readonly caller: AccountId32;
      readonly target: AccountId32;
      readonly result: Result<Null, SpRuntimeDispatchError>;
    } & Struct;
    readonly type: 'ApprovedSubsidy' | 'AcceptedSubsidy' | 'RemovedSubsidy' | 'RemovedPendingSubsidy' | 'UpdatedPolyxLimit' | 'SubsidyDebited' | 'RelayedTx';
  }

  /** @name PalletContractsEvent (289) */
  interface PalletContractsEvent extends Enum {
    readonly isInstantiated: boolean;
    readonly asInstantiated: {
      readonly deployer: AccountId32;
      readonly contract: AccountId32;
    } & Struct;
    readonly isTerminated: boolean;
    readonly asTerminated: {
      readonly contract: AccountId32;
      readonly beneficiary: AccountId32;
    } & Struct;
    readonly isCodeStored: boolean;
    readonly asCodeStored: {
      readonly codeHash: H256;
      readonly depositHeld: u128;
      readonly uploader: AccountId32;
    } & Struct;
    readonly isContractEmitted: boolean;
    readonly asContractEmitted: {
      readonly contract: AccountId32;
      readonly data: Bytes;
    } & Struct;
    readonly isCodeRemoved: boolean;
    readonly asCodeRemoved: {
      readonly codeHash: H256;
      readonly depositReleased: u128;
      readonly remover: AccountId32;
    } & Struct;
    readonly isContractCodeUpdated: boolean;
    readonly asContractCodeUpdated: {
      readonly contract: AccountId32;
      readonly newCodeHash: H256;
      readonly oldCodeHash: H256;
    } & Struct;
    readonly isCalled: boolean;
    readonly asCalled: {
      readonly caller: PalletContractsOrigin;
      readonly contract: AccountId32;
    } & Struct;
    readonly isDelegateCalled: boolean;
    readonly asDelegateCalled: {
      readonly contract: AccountId32;
      readonly codeHash: H256;
    } & Struct;
    readonly isStorageDepositTransferredAndHeld: boolean;
    readonly asStorageDepositTransferredAndHeld: {
      readonly from: AccountId32;
      readonly to: AccountId32;
      readonly amount: u128;
    } & Struct;
    readonly isStorageDepositTransferredAndReleased: boolean;
    readonly asStorageDepositTransferredAndReleased: {
      readonly from: AccountId32;
      readonly to: AccountId32;
      readonly amount: u128;
    } & Struct;
    readonly type: 'Instantiated' | 'Terminated' | 'CodeStored' | 'ContractEmitted' | 'CodeRemoved' | 'ContractCodeUpdated' | 'Called' | 'DelegateCalled' | 'StorageDepositTransferredAndHeld' | 'StorageDepositTransferredAndReleased';
  }

  /** @name PalletContractsOrigin (290) */
  interface PalletContractsOrigin extends Enum {
    readonly isRoot: boolean;
    readonly isSigned: boolean;
    readonly asSigned: AccountId32;
    readonly type: 'Root' | 'Signed';
  }

  /** @name PolymeshRuntimeDevelopRuntime (291) */
  type PolymeshRuntimeDevelopRuntime = Null;

  /** @name PolymeshContractsEvent (292) */
  interface PolymeshContractsEvent extends Enum {
    readonly isApiHashUpdated: boolean;
    readonly asApiHashUpdated: ITuple<[PolymeshContractsApi, PolymeshContractsChainVersion, H256]>;
    readonly isScRuntimeCall: boolean;
    readonly asScRuntimeCall: ITuple<[AccountId32, PolymeshContractsChainExtensionExtrinsicId]>;
    readonly type: 'ApiHashUpdated' | 'ScRuntimeCall';
  }

  /** @name PolymeshContractsApi (293) */
  interface PolymeshContractsApi extends Struct {
    readonly desc: U8aFixed;
    readonly major: u32;
  }

  /** @name PolymeshContractsChainVersion (294) */
  interface PolymeshContractsChainVersion extends Struct {
    readonly specVersion: u32;
    readonly txVersion: u32;
  }

  /** @name PolymeshContractsChainExtensionExtrinsicId (295) */
  interface PolymeshContractsChainExtensionExtrinsicId extends ITuple<[u8, u8]> {}

  /** @name PalletPreimageEvent (296) */
  interface PalletPreimageEvent extends Enum {
    readonly isNoted: boolean;
    readonly asNoted: {
      readonly hash_: H256;
    } & Struct;
    readonly isRequested: boolean;
    readonly asRequested: {
      readonly hash_: H256;
    } & Struct;
    readonly isCleared: boolean;
    readonly asCleared: {
      readonly hash_: H256;
    } & Struct;
    readonly type: 'Noted' | 'Requested' | 'Cleared';
  }

  /** @name PalletNftEvent (297) */
  interface PalletNftEvent extends Enum {
    readonly isNftCollectionCreated: boolean;
    readonly asNftCollectionCreated: ITuple<[PolymeshPrimitivesIdentityId, PolymeshPrimitivesAssetAssetId, u64]>;
    readonly isNftHoldingsUpdated: boolean;
    readonly asNftHoldingsUpdated: ITuple<[PolymeshPrimitivesIdentityId, PolymeshPrimitivesNftNfTs, Option<PolymeshPrimitivesAssetAssetHolder>, Option<PolymeshPrimitivesAssetAssetHolder>, PolymeshPrimitivesAssetHoldingsUpdateReason]>;
    readonly type: 'NftCollectionCreated' | 'NftHoldingsUpdated';
  }

  /** @name PalletElectionProviderMultiPhaseEvent (299) */
  interface PalletElectionProviderMultiPhaseEvent extends Enum {
    readonly isSolutionStored: boolean;
    readonly asSolutionStored: {
      readonly compute: PalletElectionProviderMultiPhaseElectionCompute;
      readonly origin: Option<AccountId32>;
      readonly prevEjected: bool;
    } & Struct;
    readonly isElectionFinalized: boolean;
    readonly asElectionFinalized: {
      readonly compute: PalletElectionProviderMultiPhaseElectionCompute;
      readonly score: SpNposElectionsElectionScore;
    } & Struct;
    readonly isElectionFailed: boolean;
    readonly isRewarded: boolean;
    readonly asRewarded: {
      readonly account: AccountId32;
      readonly value: u128;
    } & Struct;
    readonly isSlashed: boolean;
    readonly asSlashed: {
      readonly account: AccountId32;
      readonly value: u128;
    } & Struct;
    readonly isPhaseTransitioned: boolean;
    readonly asPhaseTransitioned: {
      readonly from: PalletElectionProviderMultiPhasePhase;
      readonly to: PalletElectionProviderMultiPhasePhase;
      readonly round: u32;
    } & Struct;
    readonly type: 'SolutionStored' | 'ElectionFinalized' | 'ElectionFailed' | 'Rewarded' | 'Slashed' | 'PhaseTransitioned';
  }

  /** @name PalletElectionProviderMultiPhaseElectionCompute (300) */
  interface PalletElectionProviderMultiPhaseElectionCompute extends Enum {
    readonly isOnChain: boolean;
    readonly isSigned: boolean;
    readonly isUnsigned: boolean;
    readonly isFallback: boolean;
    readonly isEmergency: boolean;
    readonly type: 'OnChain' | 'Signed' | 'Unsigned' | 'Fallback' | 'Emergency';
  }

  /** @name SpNposElectionsElectionScore (301) */
  interface SpNposElectionsElectionScore extends Struct {
    readonly minimalStake: u128;
    readonly sumStake: u128;
    readonly sumStakeSquared: u128;
  }

  /** @name PalletElectionProviderMultiPhasePhase (302) */
  interface PalletElectionProviderMultiPhasePhase extends Enum {
    readonly isOff: boolean;
    readonly isSigned: boolean;
    readonly isUnsigned: boolean;
    readonly asUnsigned: ITuple<[bool, u32]>;
    readonly isEmergency: boolean;
    readonly type: 'Off' | 'Signed' | 'Unsigned' | 'Emergency';
  }

  /** @name FrameSystemPhase (304) */
  interface FrameSystemPhase extends Enum {
    readonly isApplyExtrinsic: boolean;
    readonly asApplyExtrinsic: u32;
    readonly isFinalization: boolean;
    readonly isInitialization: boolean;
    readonly type: 'ApplyExtrinsic' | 'Finalization' | 'Initialization';
  }

  /** @name FrameSystemLastRuntimeUpgradeInfo (307) */
  interface FrameSystemLastRuntimeUpgradeInfo extends Struct {
    readonly specVersion: Compact<u32>;
    readonly specName: Text;
  }

  /** @name FrameSystemCodeUpgradeAuthorization (310) */
  interface FrameSystemCodeUpgradeAuthorization extends Struct {
    readonly codeHash: H256;
    readonly checkVersion: bool;
  }

  /** @name FrameSystemCall (311) */
  interface FrameSystemCall extends Enum {
    readonly isRemark: boolean;
    readonly asRemark: {
      readonly remark: Bytes;
    } & Struct;
    readonly isSetHeapPages: boolean;
    readonly asSetHeapPages: {
      readonly pages: u64;
    } & Struct;
    readonly isSetCode: boolean;
    readonly asSetCode: {
      readonly code: Bytes;
    } & Struct;
    readonly isSetCodeWithoutChecks: boolean;
    readonly asSetCodeWithoutChecks: {
      readonly code: Bytes;
    } & Struct;
    readonly isSetStorage: boolean;
    readonly asSetStorage: {
      readonly items: Vec<ITuple<[Bytes, Bytes]>>;
    } & Struct;
    readonly isKillStorage: boolean;
    readonly asKillStorage: {
      readonly keys_: Vec<Bytes>;
    } & Struct;
    readonly isKillPrefix: boolean;
    readonly asKillPrefix: {
      readonly prefix: Bytes;
      readonly subkeys: u32;
    } & Struct;
    readonly isRemarkWithEvent: boolean;
    readonly asRemarkWithEvent: {
      readonly remark: Bytes;
    } & Struct;
    readonly isAuthorizeUpgrade: boolean;
    readonly asAuthorizeUpgrade: {
      readonly codeHash: H256;
    } & Struct;
    readonly isAuthorizeUpgradeWithoutChecks: boolean;
    readonly asAuthorizeUpgradeWithoutChecks: {
      readonly codeHash: H256;
    } & Struct;
    readonly isApplyAuthorizedUpgrade: boolean;
    readonly asApplyAuthorizedUpgrade: {
      readonly code: Bytes;
    } & Struct;
    readonly type: 'Remark' | 'SetHeapPages' | 'SetCode' | 'SetCodeWithoutChecks' | 'SetStorage' | 'KillStorage' | 'KillPrefix' | 'RemarkWithEvent' | 'AuthorizeUpgrade' | 'AuthorizeUpgradeWithoutChecks' | 'ApplyAuthorizedUpgrade';
  }

  /** @name FrameSystemLimitsBlockWeights (315) */
  interface FrameSystemLimitsBlockWeights extends Struct {
    readonly baseBlock: SpWeightsWeightV2Weight;
    readonly maxBlock: SpWeightsWeightV2Weight;
    readonly perClass: FrameSupportDispatchPerDispatchClassWeightsPerClass;
  }

  /** @name FrameSupportDispatchPerDispatchClassWeightsPerClass (316) */
  interface FrameSupportDispatchPerDispatchClassWeightsPerClass extends Struct {
    readonly normal: FrameSystemLimitsWeightsPerClass;
    readonly operational: FrameSystemLimitsWeightsPerClass;
    readonly mandatory: FrameSystemLimitsWeightsPerClass;
  }

  /** @name FrameSystemLimitsWeightsPerClass (317) */
  interface FrameSystemLimitsWeightsPerClass extends Struct {
    readonly baseExtrinsic: SpWeightsWeightV2Weight;
    readonly maxExtrinsic: Option<SpWeightsWeightV2Weight>;
    readonly maxTotal: Option<SpWeightsWeightV2Weight>;
    readonly reserved: Option<SpWeightsWeightV2Weight>;
  }

  /** @name FrameSystemLimitsBlockLength (319) */
  interface FrameSystemLimitsBlockLength extends Struct {
    readonly max: FrameSupportDispatchPerDispatchClassU32;
  }

  /** @name FrameSupportDispatchPerDispatchClassU32 (320) */
  interface FrameSupportDispatchPerDispatchClassU32 extends Struct {
    readonly normal: u32;
    readonly operational: u32;
    readonly mandatory: u32;
  }

  /** @name SpWeightsRuntimeDbWeight (321) */
  interface SpWeightsRuntimeDbWeight extends Struct {
    readonly read: u64;
    readonly write: u64;
  }

  /** @name SpVersionRuntimeVersion (322) */
  interface SpVersionRuntimeVersion extends Struct {
    readonly specName: Text;
    readonly implName: Text;
    readonly authoringVersion: u32;
    readonly specVersion: u32;
    readonly implVersion: u32;
    readonly apis: Vec<ITuple<[U8aFixed, u32]>>;
    readonly transactionVersion: u32;
    readonly systemVersion: u8;
  }

  /** @name FrameSystemError (327) */
  interface FrameSystemError extends Enum {
    readonly isInvalidSpecName: boolean;
    readonly isSpecVersionNeedsToIncrease: boolean;
    readonly isFailedToExtractRuntimeVersion: boolean;
    readonly isNonDefaultComposite: boolean;
    readonly isNonZeroRefCount: boolean;
    readonly isCallFiltered: boolean;
    readonly isMultiBlockMigrationsOngoing: boolean;
    readonly isNothingAuthorized: boolean;
    readonly isUnauthorized: boolean;
    readonly type: 'InvalidSpecName' | 'SpecVersionNeedsToIncrease' | 'FailedToExtractRuntimeVersion' | 'NonDefaultComposite' | 'NonZeroRefCount' | 'CallFiltered' | 'MultiBlockMigrationsOngoing' | 'NothingAuthorized' | 'Unauthorized';
  }

  /** @name SpConsensusBabeAppPublic (330) */
  interface SpConsensusBabeAppPublic extends U8aFixed {}

  /** @name SpConsensusBabeDigestsNextConfigDescriptor (333) */
  interface SpConsensusBabeDigestsNextConfigDescriptor extends Enum {
    readonly isV1: boolean;
    readonly asV1: {
      readonly c: ITuple<[u64, u64]>;
      readonly allowedSlots: SpConsensusBabeAllowedSlots;
    } & Struct;
    readonly type: 'V1';
  }

  /** @name SpConsensusBabeAllowedSlots (335) */
  interface SpConsensusBabeAllowedSlots extends Enum {
    readonly isPrimarySlots: boolean;
    readonly isPrimaryAndSecondaryPlainSlots: boolean;
    readonly isPrimaryAndSecondaryVRFSlots: boolean;
    readonly type: 'PrimarySlots' | 'PrimaryAndSecondaryPlainSlots' | 'PrimaryAndSecondaryVRFSlots';
  }

  /** @name SpConsensusBabeDigestsPreDigest (339) */
  interface SpConsensusBabeDigestsPreDigest extends Enum {
    readonly isPrimary: boolean;
    readonly asPrimary: SpConsensusBabeDigestsPrimaryPreDigest;
    readonly isSecondaryPlain: boolean;
    readonly asSecondaryPlain: SpConsensusBabeDigestsSecondaryPlainPreDigest;
    readonly isSecondaryVRF: boolean;
    readonly asSecondaryVRF: SpConsensusBabeDigestsSecondaryVRFPreDigest;
    readonly type: 'Primary' | 'SecondaryPlain' | 'SecondaryVRF';
  }

  /** @name SpConsensusBabeDigestsPrimaryPreDigest (340) */
  interface SpConsensusBabeDigestsPrimaryPreDigest extends Struct {
    readonly authorityIndex: u32;
    readonly slot: u64;
    readonly vrfSignature: SpCoreSr25519VrfVrfSignature;
  }

  /** @name SpCoreSr25519VrfVrfSignature (341) */
  interface SpCoreSr25519VrfVrfSignature extends Struct {
    readonly preOutput: U8aFixed;
    readonly proof: U8aFixed;
  }

  /** @name SpConsensusBabeDigestsSecondaryPlainPreDigest (342) */
  interface SpConsensusBabeDigestsSecondaryPlainPreDigest extends Struct {
    readonly authorityIndex: u32;
    readonly slot: u64;
  }

  /** @name SpConsensusBabeDigestsSecondaryVRFPreDigest (343) */
  interface SpConsensusBabeDigestsSecondaryVRFPreDigest extends Struct {
    readonly authorityIndex: u32;
    readonly slot: u64;
    readonly vrfSignature: SpCoreSr25519VrfVrfSignature;
  }

  /** @name SpConsensusBabeBabeEpochConfiguration (344) */
  interface SpConsensusBabeBabeEpochConfiguration extends Struct {
    readonly c: ITuple<[u64, u64]>;
    readonly allowedSlots: SpConsensusBabeAllowedSlots;
  }

  /** @name PalletBabeCall (348) */
  interface PalletBabeCall extends Enum {
    readonly isReportEquivocation: boolean;
    readonly asReportEquivocation: {
      readonly equivocationProof: SpConsensusSlotsEquivocationProof;
      readonly keyOwnerProof: SpSessionMembershipProof;
    } & Struct;
    readonly isReportEquivocationUnsigned: boolean;
    readonly asReportEquivocationUnsigned: {
      readonly equivocationProof: SpConsensusSlotsEquivocationProof;
      readonly keyOwnerProof: SpSessionMembershipProof;
    } & Struct;
    readonly isPlanConfigChange: boolean;
    readonly asPlanConfigChange: {
      readonly config: SpConsensusBabeDigestsNextConfigDescriptor;
    } & Struct;
    readonly type: 'ReportEquivocation' | 'ReportEquivocationUnsigned' | 'PlanConfigChange';
  }

  /** @name SpConsensusSlotsEquivocationProof (349) */
  interface SpConsensusSlotsEquivocationProof extends Struct {
    readonly offender: SpConsensusBabeAppPublic;
    readonly slot: u64;
    readonly firstHeader: SpRuntimeHeader;
    readonly secondHeader: SpRuntimeHeader;
  }

  /** @name SpRuntimeHeader (350) */
  interface SpRuntimeHeader extends Struct {
    readonly parentHash: H256;
    readonly number: Compact<u32>;
    readonly stateRoot: H256;
    readonly extrinsicsRoot: H256;
    readonly digest: SpRuntimeDigest;
  }

  /** @name SpSessionMembershipProof (351) */
  interface SpSessionMembershipProof extends Struct {
    readonly session: u32;
    readonly trieNodes: Vec<Bytes>;
    readonly validatorCount: u32;
  }

  /** @name PalletBabeError (352) */
  interface PalletBabeError extends Enum {
    readonly isInvalidEquivocationProof: boolean;
    readonly isInvalidKeyOwnershipProof: boolean;
    readonly isDuplicateOffenceReport: boolean;
    readonly isInvalidConfiguration: boolean;
    readonly type: 'InvalidEquivocationProof' | 'InvalidKeyOwnershipProof' | 'DuplicateOffenceReport' | 'InvalidConfiguration';
  }

  /** @name PalletTimestampCall (353) */
  interface PalletTimestampCall extends Enum {
    readonly isSet: boolean;
    readonly asSet: {
      readonly now: Compact<u64>;
    } & Struct;
    readonly type: 'Set';
  }

  /** @name PalletIndicesCall (355) */
  interface PalletIndicesCall extends Enum {
    readonly isClaim: boolean;
    readonly asClaim: {
      readonly index: u32;
    } & Struct;
    readonly isTransfer: boolean;
    readonly asTransfer: {
      readonly new_: MultiAddress;
      readonly index: u32;
    } & Struct;
    readonly isFree: boolean;
    readonly asFree: {
      readonly index: u32;
    } & Struct;
    readonly isForceTransfer: boolean;
    readonly asForceTransfer: {
      readonly new_: MultiAddress;
      readonly index: u32;
      readonly freeze: bool;
    } & Struct;
    readonly isFreeze: boolean;
    readonly asFreeze: {
      readonly index: u32;
    } & Struct;
    readonly isPokeDeposit: boolean;
    readonly asPokeDeposit: {
      readonly index: u32;
    } & Struct;
    readonly type: 'Claim' | 'Transfer' | 'Free' | 'ForceTransfer' | 'Freeze' | 'PokeDeposit';
  }

  /** @name PalletIndicesError (357) */
  interface PalletIndicesError extends Enum {
    readonly isNotAssigned: boolean;
    readonly isNotOwner: boolean;
    readonly isInUse: boolean;
    readonly isNotTransfer: boolean;
    readonly isPermanent: boolean;
    readonly type: 'NotAssigned' | 'NotOwner' | 'InUse' | 'NotTransfer' | 'Permanent';
  }

  /** @name PalletBalancesBalanceLock (359) */
  interface PalletBalancesBalanceLock extends Struct {
    readonly id: U8aFixed;
    readonly amount: u128;
    readonly reasons: PalletBalancesReasons;
  }

  /** @name PalletBalancesReasons (360) */
  interface PalletBalancesReasons extends Enum {
    readonly isFee: boolean;
    readonly isMisc: boolean;
    readonly isAll: boolean;
    readonly type: 'Fee' | 'Misc' | 'All';
  }

  /** @name PalletBalancesReserveData (363) */
  interface PalletBalancesReserveData extends Struct {
    readonly id: U8aFixed;
    readonly amount: u128;
  }

  /** @name FrameSupportTokensMiscIdAmount (369) */
  interface FrameSupportTokensMiscIdAmount extends Struct {
    readonly id: U8aFixed;
    readonly amount: u128;
  }

  /** @name PalletBalancesCall (371) */
  interface PalletBalancesCall extends Enum {
    readonly isTransferAllowDeath: boolean;
    readonly asTransferAllowDeath: {
      readonly dest: MultiAddress;
      readonly value: Compact<u128>;
    } & Struct;
    readonly isForceTransfer: boolean;
    readonly asForceTransfer: {
      readonly source: MultiAddress;
      readonly dest: MultiAddress;
      readonly value: Compact<u128>;
    } & Struct;
    readonly isTransferKeepAlive: boolean;
    readonly asTransferKeepAlive: {
      readonly dest: MultiAddress;
      readonly value: Compact<u128>;
    } & Struct;
    readonly isTransferAll: boolean;
    readonly asTransferAll: {
      readonly dest: MultiAddress;
      readonly keepAlive: bool;
    } & Struct;
    readonly isForceUnreserve: boolean;
    readonly asForceUnreserve: {
      readonly who: MultiAddress;
      readonly amount: u128;
    } & Struct;
    readonly isUpgradeAccounts: boolean;
    readonly asUpgradeAccounts: {
      readonly who: Vec<AccountId32>;
    } & Struct;
    readonly isForceSetBalance: boolean;
    readonly asForceSetBalance: {
      readonly who: MultiAddress;
      readonly newFree: Compact<u128>;
    } & Struct;
    readonly isForceAdjustTotalIssuance: boolean;
    readonly asForceAdjustTotalIssuance: {
      readonly direction: PalletBalancesAdjustmentDirection;
      readonly delta: Compact<u128>;
    } & Struct;
    readonly isBurn: boolean;
    readonly asBurn: {
      readonly value: Compact<u128>;
      readonly keepAlive: bool;
    } & Struct;
    readonly isTransferWithMemo: boolean;
    readonly asTransferWithMemo: {
      readonly dest: MultiAddress;
      readonly value: Compact<u128>;
      readonly memo: Option<PolymeshPrimitivesMemo>;
    } & Struct;
    readonly type: 'TransferAllowDeath' | 'ForceTransfer' | 'TransferKeepAlive' | 'TransferAll' | 'ForceUnreserve' | 'UpgradeAccounts' | 'ForceSetBalance' | 'ForceAdjustTotalIssuance' | 'Burn' | 'TransferWithMemo';
  }

  /** @name PalletBalancesAdjustmentDirection (372) */
  interface PalletBalancesAdjustmentDirection extends Enum {
    readonly isIncrease: boolean;
    readonly isDecrease: boolean;
    readonly type: 'Increase' | 'Decrease';
  }

  /** @name PalletBalancesError (373) */
  interface PalletBalancesError extends Enum {
    readonly isVestingBalance: boolean;
    readonly isLiquidityRestrictions: boolean;
    readonly isInsufficientBalance: boolean;
    readonly isExistentialDeposit: boolean;
    readonly isExpendability: boolean;
    readonly isExistingVestingSchedule: boolean;
    readonly isDeadAccount: boolean;
    readonly isTooManyReserves: boolean;
    readonly isTooManyHolds: boolean;
    readonly isTooManyFreezes: boolean;
    readonly isIssuanceDeactivated: boolean;
    readonly isDeltaZero: boolean;
    readonly isLockIdentifierNotFound: boolean;
    readonly isOverflow: boolean;
    readonly isMaxLocksExceeded: boolean;
    readonly type: 'VestingBalance' | 'LiquidityRestrictions' | 'InsufficientBalance' | 'ExistentialDeposit' | 'Expendability' | 'ExistingVestingSchedule' | 'DeadAccount' | 'TooManyReserves' | 'TooManyHolds' | 'TooManyFreezes' | 'IssuanceDeactivated' | 'DeltaZero' | 'LockIdentifierNotFound' | 'Overflow' | 'MaxLocksExceeded';
  }

  /** @name PalletTransactionPaymentReleases (375) */
  interface PalletTransactionPaymentReleases extends Enum {
    readonly isV1Ancient: boolean;
    readonly isV2: boolean;
    readonly type: 'V1Ancient' | 'V2';
  }

  /** @name FrameSupportStorageNoDrop (376) */
  interface FrameSupportStorageNoDrop extends FrameSupportTokensFungibleImbalance {}

  /** @name FrameSupportTokensFungibleImbalance (377) */
  interface FrameSupportTokensFungibleImbalance extends Struct {
    readonly amount: u128;
  }

  /** @name PolymeshPrimitivesIdentityDidRecord (378) */
  interface PolymeshPrimitivesIdentityDidRecord extends Struct {
    readonly primaryKey: Option<AccountId32>;
  }

  /** @name PalletIdentityClaim1stKey (380) */
  interface PalletIdentityClaim1stKey extends Struct {
    readonly target: PolymeshPrimitivesIdentityId;
    readonly claimType: PolymeshPrimitivesIdentityClaimClaimType;
  }

  /** @name PalletIdentityClaim2ndKey (381) */
  interface PalletIdentityClaim2ndKey extends Struct {
    readonly issuer: PolymeshPrimitivesIdentityId;
    readonly scope: Option<PolymeshPrimitivesIdentityClaimScope>;
  }

  /** @name PolymeshPrimitivesSecondaryKeyKeyRecord (382) */
  interface PolymeshPrimitivesSecondaryKeyKeyRecord extends Enum {
    readonly isPrimaryKey: boolean;
    readonly asPrimaryKey: PolymeshPrimitivesIdentityId;
    readonly isSecondaryKey: boolean;
    readonly asSecondaryKey: PolymeshPrimitivesIdentityId;
    readonly isMultiSigSignerKey: boolean;
    readonly asMultiSigSignerKey: AccountId32;
    readonly type: 'PrimaryKey' | 'SecondaryKey' | 'MultiSigSignerKey';
  }

  /** @name PolymeshPrimitivesSecondaryKeySignatory (385) */
  interface PolymeshPrimitivesSecondaryKeySignatory extends Enum {
    readonly isIdentity: boolean;
    readonly asIdentity: PolymeshPrimitivesIdentityId;
    readonly isAccount: boolean;
    readonly asAccount: AccountId32;
    readonly type: 'Identity' | 'Account';
  }

  /** @name PolymeshPrimitivesAuthorization (386) */
  interface PolymeshPrimitivesAuthorization extends Struct {
    readonly authorizationData: PolymeshPrimitivesAuthorizationAuthorizationData;
    readonly authorizedBy: PolymeshPrimitivesIdentityId;
    readonly expiry: Option<u64>;
    readonly authId: u64;
    readonly count: u32;
  }

  /** @name PalletIdentityCall (390) */
  interface PalletIdentityCall extends Enum {
    readonly isCddRegisterDid: boolean;
    readonly asCddRegisterDid: {
      readonly targetAccount: AccountId32;
      readonly secondaryKeys: Vec<PolymeshPrimitivesSecondaryKey>;
    } & Struct;
    readonly isAcceptPrimaryKey: boolean;
    readonly asAcceptPrimaryKey: {
      readonly rotationAuthId: u64;
    } & Struct;
    readonly isJoinIdentityAsKey: boolean;
    readonly asJoinIdentityAsKey: {
      readonly authId: u64;
    } & Struct;
    readonly isLeaveIdentityAsKey: boolean;
    readonly isAddClaim: boolean;
    readonly asAddClaim: {
      readonly target: PolymeshPrimitivesIdentityId;
      readonly claim: PolymeshPrimitivesIdentityClaimClaim;
      readonly expiry: Option<u64>;
    } & Struct;
    readonly isRevokeClaim: boolean;
    readonly asRevokeClaim: {
      readonly target: PolymeshPrimitivesIdentityId;
      readonly claim: PolymeshPrimitivesIdentityClaimClaim;
    } & Struct;
    readonly isFreezeSecondaryKeys: boolean;
    readonly isUnfreezeSecondaryKeys: boolean;
    readonly isAddAuthorization: boolean;
    readonly asAddAuthorization: {
      readonly target: PolymeshPrimitivesSecondaryKeySignatory;
      readonly data: PolymeshPrimitivesAuthorizationAuthorizationData;
      readonly expiry: Option<u64>;
    } & Struct;
    readonly isRemoveAuthorization: boolean;
    readonly asRemoveAuthorization: {
      readonly target: PolymeshPrimitivesSecondaryKeySignatory;
      readonly authId: u64;
      readonly authIssuerPays: bool;
    } & Struct;
    readonly isGcAddCddClaim: boolean;
    readonly asGcAddCddClaim: {
      readonly target: PolymeshPrimitivesIdentityId;
    } & Struct;
    readonly isGcRevokeCddClaim: boolean;
    readonly asGcRevokeCddClaim: {
      readonly target: PolymeshPrimitivesIdentityId;
    } & Struct;
    readonly isRevokeClaimByIndex: boolean;
    readonly asRevokeClaimByIndex: {
      readonly target: PolymeshPrimitivesIdentityId;
      readonly claimType: PolymeshPrimitivesIdentityClaimClaimType;
      readonly scope: Option<PolymeshPrimitivesIdentityClaimScope>;
    } & Struct;
    readonly isRotatePrimaryKeyToSecondary: boolean;
    readonly asRotatePrimaryKeyToSecondary: {
      readonly authId: u64;
    } & Struct;
    readonly isAddSecondaryKeysWithAuthorization: boolean;
    readonly asAddSecondaryKeysWithAuthorization: {
      readonly additionalKeys: Vec<PolymeshPrimitivesIdentitySecondaryKeyWithAuth>;
      readonly expiresAt: u64;
    } & Struct;
    readonly isSetSecondaryKeyPermissions: boolean;
    readonly asSetSecondaryKeyPermissions: {
      readonly key: AccountId32;
      readonly perms: PolymeshPrimitivesSecondaryKeyPermissions;
    } & Struct;
    readonly isRemoveSecondaryKeys: boolean;
    readonly asRemoveSecondaryKeys: {
      readonly keysToRemove: Vec<AccountId32>;
    } & Struct;
    readonly isRegisterCustomClaimType: boolean;
    readonly asRegisterCustomClaimType: {
      readonly ty: Bytes;
    } & Struct;
    readonly isCddRegisterDidWithCdd: boolean;
    readonly asCddRegisterDidWithCdd: {
      readonly targetAccount: AccountId32;
      readonly secondaryKeys: Vec<PolymeshPrimitivesSecondaryKey>;
      readonly expiry: Option<u64>;
    } & Struct;
    readonly isCreateChildIdentity: boolean;
    readonly asCreateChildIdentity: {
      readonly secondaryKey: AccountId32;
    } & Struct;
    readonly isCreateChildIdentities: boolean;
    readonly asCreateChildIdentities: {
      readonly childKeys: Vec<PolymeshPrimitivesIdentityCreateChildIdentityWithAuth>;
      readonly expiresAt: u64;
    } & Struct;
    readonly isUnlinkChildIdentity: boolean;
    readonly asUnlinkChildIdentity: {
      readonly childDid: PolymeshPrimitivesIdentityId;
    } & Struct;
    readonly isRegisterDid: boolean;
    readonly asRegisterDid: {
      readonly targetAccount: AccountId32;
    } & Struct;
    readonly type: 'CddRegisterDid' | 'AcceptPrimaryKey' | 'JoinIdentityAsKey' | 'LeaveIdentityAsKey' | 'AddClaim' | 'RevokeClaim' | 'FreezeSecondaryKeys' | 'UnfreezeSecondaryKeys' | 'AddAuthorization' | 'RemoveAuthorization' | 'GcAddCddClaim' | 'GcRevokeCddClaim' | 'RevokeClaimByIndex' | 'RotatePrimaryKeyToSecondary' | 'AddSecondaryKeysWithAuthorization' | 'SetSecondaryKeyPermissions' | 'RemoveSecondaryKeys' | 'RegisterCustomClaimType' | 'CddRegisterDidWithCdd' | 'CreateChildIdentity' | 'CreateChildIdentities' | 'UnlinkChildIdentity' | 'RegisterDid';
  }

  /** @name PolymeshPrimitivesIdentitySecondaryKeyWithAuth (392) */
  interface PolymeshPrimitivesIdentitySecondaryKeyWithAuth extends Struct {
    readonly secondaryKey: PolymeshPrimitivesSecondaryKey;
    readonly authSignature: H512;
  }

  /** @name PolymeshPrimitivesIdentityCreateChildIdentityWithAuth (395) */
  interface PolymeshPrimitivesIdentityCreateChildIdentityWithAuth extends Struct {
    readonly key: AccountId32;
    readonly authSignature: H512;
  }

  /** @name PalletIdentityError (396) */
  interface PalletIdentityError extends Enum {
    readonly isAlreadyLinked: boolean;
    readonly isMissingIdentity: boolean;
    readonly isUnauthorized: boolean;
    readonly isInvalidAccountKey: boolean;
    readonly isUnAuthorizedDidRegistrar: boolean;
    readonly isInvalidAuthorizationFromOwner: boolean;
    readonly isInvalidAuthorizationFromDidRegistrar: boolean;
    readonly isNotDidRegistrarAttestation: boolean;
    readonly isAuthorizationsNotForSameDids: boolean;
    readonly isDidMustAlreadyExist: boolean;
    readonly isAuthorizationExpired: boolean;
    readonly isTargetDidInactive: boolean;
    readonly isAuthorizationHasBeenRevoked: boolean;
    readonly isInvalidAuthorizationSignature: boolean;
    readonly isKeyNotAllowed: boolean;
    readonly isNotPrimaryKey: boolean;
    readonly isDidDoesNotExist: boolean;
    readonly isDidAlreadyExists: boolean;
    readonly isSecondaryKeysContainPrimaryKey: boolean;
    readonly isFailedToChargeFee: boolean;
    readonly isNotASigner: boolean;
    readonly isCannotDecodeSignerAccountId: boolean;
    readonly isAccountKeyIsBeingUsed: boolean;
    readonly isCustomScopeTooLong: boolean;
    readonly isCustomClaimTypeAlreadyExists: boolean;
    readonly isCustomClaimTypeDoesNotExist: boolean;
    readonly isClaimDoesNotExist: boolean;
    readonly isIsChildIdentity: boolean;
    readonly isNoParentIdentity: boolean;
    readonly isNotParentOrChildIdentity: boolean;
    readonly isDuplicateKey: boolean;
    readonly isExceptNotAllowedForExtrinsics: boolean;
    readonly isExceededNumberOfGivenAuths: boolean;
    readonly isBadAuthorizationType: boolean;
    readonly isInvalidAuthorization: boolean;
    readonly isUnauthorizedCallerFrozenDid: boolean;
    readonly isUnauthorizedCallerDidInactive: boolean;
    readonly isUnauthorizedCallerMissingPermissions: boolean;
    readonly isIdentityNotFoundForAccountPortfolio: boolean;
    readonly type: 'AlreadyLinked' | 'MissingIdentity' | 'Unauthorized' | 'InvalidAccountKey' | 'UnAuthorizedDidRegistrar' | 'InvalidAuthorizationFromOwner' | 'InvalidAuthorizationFromDidRegistrar' | 'NotDidRegistrarAttestation' | 'AuthorizationsNotForSameDids' | 'DidMustAlreadyExist' | 'AuthorizationExpired' | 'TargetDidInactive' | 'AuthorizationHasBeenRevoked' | 'InvalidAuthorizationSignature' | 'KeyNotAllowed' | 'NotPrimaryKey' | 'DidDoesNotExist' | 'DidAlreadyExists' | 'SecondaryKeysContainPrimaryKey' | 'FailedToChargeFee' | 'NotASigner' | 'CannotDecodeSignerAccountId' | 'AccountKeyIsBeingUsed' | 'CustomScopeTooLong' | 'CustomClaimTypeAlreadyExists' | 'CustomClaimTypeDoesNotExist' | 'ClaimDoesNotExist' | 'IsChildIdentity' | 'NoParentIdentity' | 'NotParentOrChildIdentity' | 'DuplicateKey' | 'ExceptNotAllowedForExtrinsics' | 'ExceededNumberOfGivenAuths' | 'BadAuthorizationType' | 'InvalidAuthorization' | 'UnauthorizedCallerFrozenDid' | 'UnauthorizedCallerDidInactive' | 'UnauthorizedCallerMissingPermissions' | 'IdentityNotFoundForAccountPortfolio';
  }

  /** @name PolymeshPrimitivesGroupInactiveMember (398) */
  interface PolymeshPrimitivesGroupInactiveMember extends Struct {
    readonly id: PolymeshPrimitivesIdentityId;
    readonly deactivatedAt: u64;
    readonly expiry: Option<u64>;
  }

  /** @name PalletGroupCall (399) */
  interface PalletGroupCall extends Enum {
    readonly isSetActiveMembersLimit: boolean;
    readonly asSetActiveMembersLimit: {
      readonly limit: u32;
    } & Struct;
    readonly isDisableMember: boolean;
    readonly asDisableMember: {
      readonly who: PolymeshPrimitivesIdentityId;
      readonly expiry: Option<u64>;
      readonly at: Option<u64>;
    } & Struct;
    readonly isAddMember: boolean;
    readonly asAddMember: {
      readonly who: PolymeshPrimitivesIdentityId;
    } & Struct;
    readonly isRemoveMember: boolean;
    readonly asRemoveMember: {
      readonly who: PolymeshPrimitivesIdentityId;
    } & Struct;
    readonly isSwapMember: boolean;
    readonly asSwapMember: {
      readonly remove: PolymeshPrimitivesIdentityId;
      readonly add: PolymeshPrimitivesIdentityId;
    } & Struct;
    readonly isResetMembers: boolean;
    readonly asResetMembers: {
      readonly members: Vec<PolymeshPrimitivesIdentityId>;
    } & Struct;
    readonly isAbdicateMembership: boolean;
    readonly type: 'SetActiveMembersLimit' | 'DisableMember' | 'AddMember' | 'RemoveMember' | 'SwapMember' | 'ResetMembers' | 'AbdicateMembership';
  }

  /** @name PalletGroupError (400) */
  interface PalletGroupError extends Enum {
    readonly isOnlyPrimaryKeyAllowed: boolean;
    readonly isDuplicateMember: boolean;
    readonly isNoSuchMember: boolean;
    readonly isLastMemberCannotQuit: boolean;
    readonly isActiveMembersLimitExceeded: boolean;
    readonly isActiveMembersLimitOverflow: boolean;
    readonly type: 'OnlyPrimaryKeyAllowed' | 'DuplicateMember' | 'NoSuchMember' | 'LastMemberCannotQuit' | 'ActiveMembersLimitExceeded' | 'ActiveMembersLimitOverflow';
  }

  /** @name PalletCommitteeCall (402) */
  interface PalletCommitteeCall extends Enum {
    readonly isSetVoteThreshold: boolean;
    readonly asSetVoteThreshold: {
      readonly n: u32;
      readonly d: u32;
    } & Struct;
    readonly isSetReleaseCoordinator: boolean;
    readonly asSetReleaseCoordinator: {
      readonly id: PolymeshPrimitivesIdentityId;
    } & Struct;
    readonly isSetExpiresAfter: boolean;
    readonly asSetExpiresAfter: {
      readonly expiry: PolymeshPrimitivesMaybeBlock;
    } & Struct;
    readonly isVoteOrPropose: boolean;
    readonly asVoteOrPropose: {
      readonly approve: bool;
      readonly call: Call;
    } & Struct;
    readonly isVote: boolean;
    readonly asVote: {
      readonly proposal: H256;
      readonly index: u32;
      readonly approve: bool;
    } & Struct;
    readonly type: 'SetVoteThreshold' | 'SetReleaseCoordinator' | 'SetExpiresAfter' | 'VoteOrPropose' | 'Vote';
  }

  /** @name PalletMultisigCall (408) */
  interface PalletMultisigCall extends Enum {
    readonly isCreateMultisig: boolean;
    readonly asCreateMultisig: {
      readonly signers: Vec<AccountId32>;
      readonly sigsRequired: u64;
      readonly permissions: Option<PolymeshPrimitivesSecondaryKeyPermissions>;
    } & Struct;
    readonly isCreateProposal: boolean;
    readonly asCreateProposal: {
      readonly multisig: AccountId32;
      readonly proposal: Call;
      readonly expiry: Option<u64>;
    } & Struct;
    readonly isApprove: boolean;
    readonly asApprove: {
      readonly multisig: AccountId32;
      readonly proposalId: u64;
      readonly maxWeight: Option<SpWeightsWeightV2Weight>;
    } & Struct;
    readonly isReject: boolean;
    readonly asReject: {
      readonly multisig: AccountId32;
      readonly proposalId: u64;
    } & Struct;
    readonly isAcceptMultisigSigner: boolean;
    readonly asAcceptMultisigSigner: {
      readonly authId: u64;
    } & Struct;
    readonly isAddMultisigSigners: boolean;
    readonly asAddMultisigSigners: {
      readonly signers: Vec<AccountId32>;
    } & Struct;
    readonly isRemoveMultisigSigners: boolean;
    readonly asRemoveMultisigSigners: {
      readonly signers: Vec<AccountId32>;
    } & Struct;
    readonly isAddMultisigSignersViaAdmin: boolean;
    readonly asAddMultisigSignersViaAdmin: {
      readonly multisig: AccountId32;
      readonly signers: Vec<AccountId32>;
    } & Struct;
    readonly isRemoveMultisigSignersViaAdmin: boolean;
    readonly asRemoveMultisigSignersViaAdmin: {
      readonly multisig: AccountId32;
      readonly signers: Vec<AccountId32>;
    } & Struct;
    readonly isChangeSigsRequired: boolean;
    readonly asChangeSigsRequired: {
      readonly sigsRequired: u64;
    } & Struct;
    readonly isChangeSigsRequiredViaAdmin: boolean;
    readonly asChangeSigsRequiredViaAdmin: {
      readonly multisig: AccountId32;
      readonly signaturesRequired: u64;
    } & Struct;
    readonly isAddAdmin: boolean;
    readonly asAddAdmin: {
      readonly adminDid: PolymeshPrimitivesIdentityId;
    } & Struct;
    readonly isRemoveAdminViaAdmin: boolean;
    readonly asRemoveAdminViaAdmin: {
      readonly multisig: AccountId32;
    } & Struct;
    readonly isRemovePayer: boolean;
    readonly isRemovePayerViaPayer: boolean;
    readonly asRemovePayerViaPayer: {
      readonly multisig: AccountId32;
    } & Struct;
    readonly isApproveJoinIdentity: boolean;
    readonly asApproveJoinIdentity: {
      readonly multisig: AccountId32;
      readonly authId: u64;
    } & Struct;
    readonly isJoinIdentity: boolean;
    readonly asJoinIdentity: {
      readonly authId: u64;
    } & Struct;
    readonly isRemoveAdmin: boolean;
    readonly type: 'CreateMultisig' | 'CreateProposal' | 'Approve' | 'Reject' | 'AcceptMultisigSigner' | 'AddMultisigSigners' | 'RemoveMultisigSigners' | 'AddMultisigSignersViaAdmin' | 'RemoveMultisigSignersViaAdmin' | 'ChangeSigsRequired' | 'ChangeSigsRequiredViaAdmin' | 'AddAdmin' | 'RemoveAdminViaAdmin' | 'RemovePayer' | 'RemovePayerViaPayer' | 'ApproveJoinIdentity' | 'JoinIdentity' | 'RemoveAdmin';
  }

  /** @name PalletValidatorsCall (410) */
  interface PalletValidatorsCall extends Enum {
    readonly isAddPermissionedValidator: boolean;
    readonly asAddPermissionedValidator: {
      readonly identity: PolymeshPrimitivesIdentityId;
      readonly intendedCount: Option<u32>;
    } & Struct;
    readonly isRemovePermissionedValidator: boolean;
    readonly asRemovePermissionedValidator: {
      readonly identity: PolymeshPrimitivesIdentityId;
    } & Struct;
    readonly isPayoutStakersBySystem: boolean;
    readonly asPayoutStakersBySystem: {
      readonly validatorStash: AccountId32;
      readonly era: u32;
    } & Struct;
    readonly isChangeSlashingAllowedFor: boolean;
    readonly asChangeSlashingAllowedFor: {
      readonly slashingSwitch: PalletValidatorsSlashingSwitch;
    } & Struct;
    readonly isUpdatePermissionedValidatorIntendedCount: boolean;
    readonly asUpdatePermissionedValidatorIntendedCount: {
      readonly identity: PolymeshPrimitivesIdentityId;
      readonly newIntendedCount: u32;
    } & Struct;
    readonly isChillFromGovernance: boolean;
    readonly asChillFromGovernance: {
      readonly identity: PolymeshPrimitivesIdentityId;
      readonly stashKeys: Vec<AccountId32>;
    } & Struct;
    readonly isSetCommissionCap: boolean;
    readonly asSetCommissionCap: {
      readonly newCap: Perbill;
    } & Struct;
    readonly type: 'AddPermissionedValidator' | 'RemovePermissionedValidator' | 'PayoutStakersBySystem' | 'ChangeSlashingAllowedFor' | 'UpdatePermissionedValidatorIntendedCount' | 'ChillFromGovernance' | 'SetCommissionCap';
  }

  /** @name PalletStakingPalletCall (411) */
  interface PalletStakingPalletCall extends Enum {
    readonly isBond: boolean;
    readonly asBond: {
      readonly value: Compact<u128>;
      readonly payee: PalletStakingRewardDestination;
    } & Struct;
    readonly isBondExtra: boolean;
    readonly asBondExtra: {
      readonly maxAdditional: Compact<u128>;
    } & Struct;
    readonly isUnbond: boolean;
    readonly asUnbond: {
      readonly value: Compact<u128>;
    } & Struct;
    readonly isWithdrawUnbonded: boolean;
    readonly asWithdrawUnbonded: {
      readonly numSlashingSpans: u32;
    } & Struct;
    readonly isValidate: boolean;
    readonly asValidate: {
      readonly prefs: PalletStakingValidatorPrefs;
    } & Struct;
    readonly isNominate: boolean;
    readonly asNominate: {
      readonly targets: Vec<MultiAddress>;
    } & Struct;
    readonly isChill: boolean;
    readonly isSetPayee: boolean;
    readonly asSetPayee: {
      readonly payee: PalletStakingRewardDestination;
    } & Struct;
    readonly isSetController: boolean;
    readonly isSetValidatorCount: boolean;
    readonly asSetValidatorCount: {
      readonly new_: Compact<u32>;
    } & Struct;
    readonly isIncreaseValidatorCount: boolean;
    readonly asIncreaseValidatorCount: {
      readonly additional: Compact<u32>;
    } & Struct;
    readonly isScaleValidatorCount: boolean;
    readonly asScaleValidatorCount: {
      readonly factor: Percent;
    } & Struct;
    readonly isForceNoEras: boolean;
    readonly isForceNewEra: boolean;
    readonly isSetInvulnerables: boolean;
    readonly asSetInvulnerables: {
      readonly invulnerables: Vec<AccountId32>;
    } & Struct;
    readonly isForceUnstake: boolean;
    readonly asForceUnstake: {
      readonly stash: AccountId32;
      readonly numSlashingSpans: u32;
    } & Struct;
    readonly isForceNewEraAlways: boolean;
    readonly isCancelDeferredSlash: boolean;
    readonly asCancelDeferredSlash: {
      readonly era: u32;
      readonly slashIndices: Vec<u32>;
    } & Struct;
    readonly isPayoutStakers: boolean;
    readonly asPayoutStakers: {
      readonly validatorStash: AccountId32;
      readonly era: u32;
    } & Struct;
    readonly isRebond: boolean;
    readonly asRebond: {
      readonly value: Compact<u128>;
    } & Struct;
    readonly isReapStash: boolean;
    readonly asReapStash: {
      readonly stash: AccountId32;
      readonly numSlashingSpans: u32;
    } & Struct;
    readonly isKick: boolean;
    readonly asKick: {
      readonly who: Vec<MultiAddress>;
    } & Struct;
    readonly isSetStakingConfigs: boolean;
    readonly asSetStakingConfigs: {
      readonly minNominatorBond: PalletStakingPalletConfigOpU128;
      readonly minValidatorBond: PalletStakingPalletConfigOpU128;
      readonly maxNominatorCount: PalletStakingPalletConfigOpU32;
      readonly maxValidatorCount: PalletStakingPalletConfigOpU32;
      readonly chillThreshold: PalletStakingPalletConfigOpPercent;
      readonly minCommission: PalletStakingPalletConfigOpPerbill;
      readonly maxStakedRewards: PalletStakingPalletConfigOpPercent;
    } & Struct;
    readonly isChillOther: boolean;
    readonly asChillOther: {
      readonly stash: AccountId32;
    } & Struct;
    readonly isForceApplyMinCommission: boolean;
    readonly asForceApplyMinCommission: {
      readonly validatorStash: AccountId32;
    } & Struct;
    readonly isSetMinCommission: boolean;
    readonly asSetMinCommission: {
      readonly new_: Perbill;
    } & Struct;
    readonly isPayoutStakersByPage: boolean;
    readonly asPayoutStakersByPage: {
      readonly validatorStash: AccountId32;
      readonly era: u32;
      readonly page: u32;
    } & Struct;
    readonly isUpdatePayee: boolean;
    readonly asUpdatePayee: {
      readonly controller: AccountId32;
    } & Struct;
    readonly isDeprecateControllerBatch: boolean;
    readonly asDeprecateControllerBatch: {
      readonly controllers: Vec<AccountId32>;
    } & Struct;
    readonly isRestoreLedger: boolean;
    readonly asRestoreLedger: {
      readonly stash: AccountId32;
      readonly maybeController: Option<AccountId32>;
      readonly maybeTotal: Option<u128>;
      readonly maybeUnlocking: Option<Vec<PalletStakingUnlockChunk>>;
    } & Struct;
    readonly isMigrateCurrency: boolean;
    readonly asMigrateCurrency: {
      readonly stash: AccountId32;
    } & Struct;
    readonly isManualSlash: boolean;
    readonly asManualSlash: {
      readonly validatorStash: AccountId32;
      readonly era: u32;
      readonly slashFraction: Perbill;
    } & Struct;
    readonly type: 'Bond' | 'BondExtra' | 'Unbond' | 'WithdrawUnbonded' | 'Validate' | 'Nominate' | 'Chill' | 'SetPayee' | 'SetController' | 'SetValidatorCount' | 'IncreaseValidatorCount' | 'ScaleValidatorCount' | 'ForceNoEras' | 'ForceNewEra' | 'SetInvulnerables' | 'ForceUnstake' | 'ForceNewEraAlways' | 'CancelDeferredSlash' | 'PayoutStakers' | 'Rebond' | 'ReapStash' | 'Kick' | 'SetStakingConfigs' | 'ChillOther' | 'ForceApplyMinCommission' | 'SetMinCommission' | 'PayoutStakersByPage' | 'UpdatePayee' | 'DeprecateControllerBatch' | 'RestoreLedger' | 'MigrateCurrency' | 'ManualSlash';
  }

  /** @name PalletStakingPalletConfigOpU128 (415) */
  interface PalletStakingPalletConfigOpU128 extends Enum {
    readonly isNoop: boolean;
    readonly isSet: boolean;
    readonly asSet: u128;
    readonly isRemove: boolean;
    readonly type: 'Noop' | 'Set' | 'Remove';
  }

  /** @name PalletStakingPalletConfigOpU32 (416) */
  interface PalletStakingPalletConfigOpU32 extends Enum {
    readonly isNoop: boolean;
    readonly isSet: boolean;
    readonly asSet: u32;
    readonly isRemove: boolean;
    readonly type: 'Noop' | 'Set' | 'Remove';
  }

  /** @name PalletStakingPalletConfigOpPercent (417) */
  interface PalletStakingPalletConfigOpPercent extends Enum {
    readonly isNoop: boolean;
    readonly isSet: boolean;
    readonly asSet: Percent;
    readonly isRemove: boolean;
    readonly type: 'Noop' | 'Set' | 'Remove';
  }

  /** @name PalletStakingPalletConfigOpPerbill (418) */
  interface PalletStakingPalletConfigOpPerbill extends Enum {
    readonly isNoop: boolean;
    readonly isSet: boolean;
    readonly asSet: Perbill;
    readonly isRemove: boolean;
    readonly type: 'Noop' | 'Set' | 'Remove';
  }

  /** @name PalletStakingUnlockChunk (422) */
  interface PalletStakingUnlockChunk extends Struct {
    readonly value: Compact<u128>;
    readonly era: Compact<u32>;
  }

  /** @name PalletSessionCall (424) */
  interface PalletSessionCall extends Enum {
    readonly isSetKeys: boolean;
    readonly asSetKeys: {
      readonly keys_: PolymeshRuntimeDevelopRuntimeSessionKeys;
      readonly proof: Bytes;
    } & Struct;
    readonly isPurgeKeys: boolean;
    readonly type: 'SetKeys' | 'PurgeKeys';
  }

  /** @name PolymeshRuntimeDevelopRuntimeSessionKeys (425) */
  interface PolymeshRuntimeDevelopRuntimeSessionKeys extends Struct {
    readonly grandpa: SpConsensusGrandpaAppPublic;
    readonly babe: SpConsensusBabeAppPublic;
    readonly imOnline: PalletImOnlineSr25519AppSr25519Public;
    readonly authorityDiscovery: SpAuthorityDiscoveryAppPublic;
    readonly beefy: SpConsensusBeefyEcdsaCryptoPublic;
  }

  /** @name SpAuthorityDiscoveryAppPublic (426) */
  interface SpAuthorityDiscoveryAppPublic extends U8aFixed {}

  /** @name SpConsensusBeefyEcdsaCryptoPublic (427) */
  interface SpConsensusBeefyEcdsaCryptoPublic extends U8aFixed {}

  /** @name PalletGrandpaCall (429) */
  interface PalletGrandpaCall extends Enum {
    readonly isReportEquivocation: boolean;
    readonly asReportEquivocation: {
      readonly equivocationProof: SpConsensusGrandpaEquivocationProof;
      readonly keyOwnerProof: SpSessionMembershipProof;
    } & Struct;
    readonly isReportEquivocationUnsigned: boolean;
    readonly asReportEquivocationUnsigned: {
      readonly equivocationProof: SpConsensusGrandpaEquivocationProof;
      readonly keyOwnerProof: SpSessionMembershipProof;
    } & Struct;
    readonly isNoteStalled: boolean;
    readonly asNoteStalled: {
      readonly delay: u32;
      readonly bestFinalizedBlockNumber: u32;
    } & Struct;
    readonly type: 'ReportEquivocation' | 'ReportEquivocationUnsigned' | 'NoteStalled';
  }

  /** @name SpConsensusGrandpaEquivocationProof (430) */
  interface SpConsensusGrandpaEquivocationProof extends Struct {
    readonly setId: u64;
    readonly equivocation: SpConsensusGrandpaEquivocation;
  }

  /** @name SpConsensusGrandpaEquivocation (431) */
  interface SpConsensusGrandpaEquivocation extends Enum {
    readonly isPrevote: boolean;
    readonly asPrevote: FinalityGrandpaEquivocationPrevote;
    readonly isPrecommit: boolean;
    readonly asPrecommit: FinalityGrandpaEquivocationPrecommit;
    readonly type: 'Prevote' | 'Precommit';
  }

  /** @name FinalityGrandpaEquivocationPrevote (432) */
  interface FinalityGrandpaEquivocationPrevote extends Struct {
    readonly roundNumber: u64;
    readonly identity: SpConsensusGrandpaAppPublic;
    readonly first: ITuple<[FinalityGrandpaPrevote, SpConsensusGrandpaAppSignature]>;
    readonly second: ITuple<[FinalityGrandpaPrevote, SpConsensusGrandpaAppSignature]>;
  }

  /** @name FinalityGrandpaPrevote (433) */
  interface FinalityGrandpaPrevote extends Struct {
    readonly targetHash: H256;
    readonly targetNumber: u32;
  }

  /** @name SpConsensusGrandpaAppSignature (434) */
  interface SpConsensusGrandpaAppSignature extends U8aFixed {}

  /** @name FinalityGrandpaEquivocationPrecommit (436) */
  interface FinalityGrandpaEquivocationPrecommit extends Struct {
    readonly roundNumber: u64;
    readonly identity: SpConsensusGrandpaAppPublic;
    readonly first: ITuple<[FinalityGrandpaPrecommit, SpConsensusGrandpaAppSignature]>;
    readonly second: ITuple<[FinalityGrandpaPrecommit, SpConsensusGrandpaAppSignature]>;
  }

  /** @name FinalityGrandpaPrecommit (437) */
  interface FinalityGrandpaPrecommit extends Struct {
    readonly targetHash: H256;
    readonly targetNumber: u32;
  }

  /** @name PalletImOnlineCall (439) */
  interface PalletImOnlineCall extends Enum {
    readonly isHeartbeat: boolean;
    readonly asHeartbeat: {
      readonly heartbeat: PalletImOnlineHeartbeat;
      readonly signature: PalletImOnlineSr25519AppSr25519Signature;
    } & Struct;
    readonly type: 'Heartbeat';
  }

  /** @name PalletImOnlineHeartbeat (440) */
  interface PalletImOnlineHeartbeat extends Struct {
    readonly blockNumber: u32;
    readonly sessionIndex: u32;
    readonly authorityIndex: u32;
    readonly validatorsLen: u32;
  }

  /** @name PalletImOnlineSr25519AppSr25519Signature (441) */
  interface PalletImOnlineSr25519AppSr25519Signature extends U8aFixed {}

  /** @name PalletSudoCall (442) */
  interface PalletSudoCall extends Enum {
    readonly isSudo: boolean;
    readonly asSudo: {
      readonly call: Call;
    } & Struct;
    readonly isSudoUncheckedWeight: boolean;
    readonly asSudoUncheckedWeight: {
      readonly call: Call;
      readonly weight: SpWeightsWeightV2Weight;
    } & Struct;
    readonly isSetKey: boolean;
    readonly asSetKey: {
      readonly new_: MultiAddress;
    } & Struct;
    readonly isSudoAs: boolean;
    readonly asSudoAs: {
      readonly who: MultiAddress;
      readonly call: Call;
    } & Struct;
    readonly isRemoveKey: boolean;
    readonly type: 'Sudo' | 'SudoUncheckedWeight' | 'SetKey' | 'SudoAs' | 'RemoveKey';
  }

  /** @name PalletAssetCall (443) */
  interface PalletAssetCall extends Enum {
    readonly isRegisterUniqueTicker: boolean;
    readonly asRegisterUniqueTicker: {
      readonly ticker: PolymeshPrimitivesTicker;
    } & Struct;
    readonly isAcceptTickerTransfer: boolean;
    readonly asAcceptTickerTransfer: {
      readonly authId: u64;
    } & Struct;
    readonly isAcceptAssetOwnershipTransfer: boolean;
    readonly asAcceptAssetOwnershipTransfer: {
      readonly authId: u64;
    } & Struct;
    readonly isCreateAsset: boolean;
    readonly asCreateAsset: {
      readonly assetName: Bytes;
      readonly divisible: bool;
      readonly assetType: PolymeshPrimitivesAssetAssetType;
      readonly assetIdentifiers: Vec<PolymeshPrimitivesAssetIdentifier>;
      readonly fundingRoundName: Option<Bytes>;
    } & Struct;
    readonly isFreeze: boolean;
    readonly asFreeze: {
      readonly assetId: PolymeshPrimitivesAssetAssetId;
    } & Struct;
    readonly isUnfreeze: boolean;
    readonly asUnfreeze: {
      readonly assetId: PolymeshPrimitivesAssetAssetId;
    } & Struct;
    readonly isRenameAsset: boolean;
    readonly asRenameAsset: {
      readonly assetId: PolymeshPrimitivesAssetAssetId;
      readonly assetName: Bytes;
    } & Struct;
    readonly isIssue: boolean;
    readonly asIssue: {
      readonly assetId: PolymeshPrimitivesAssetAssetId;
      readonly amount: u128;
      readonly assetHolderKind: PolymeshPrimitivesAssetAssetHolderKind;
    } & Struct;
    readonly isRedeem: boolean;
    readonly asRedeem: {
      readonly assetId: PolymeshPrimitivesAssetAssetId;
      readonly value: u128;
      readonly assetHolderKind: PolymeshPrimitivesAssetAssetHolderKind;
    } & Struct;
    readonly isMakeDivisible: boolean;
    readonly asMakeDivisible: {
      readonly assetId: PolymeshPrimitivesAssetAssetId;
    } & Struct;
    readonly isAddDocuments: boolean;
    readonly asAddDocuments: {
      readonly docs: Vec<PolymeshPrimitivesDocument>;
      readonly assetId: PolymeshPrimitivesAssetAssetId;
    } & Struct;
    readonly isRemoveDocuments: boolean;
    readonly asRemoveDocuments: {
      readonly docsId: Vec<u32>;
      readonly assetId: PolymeshPrimitivesAssetAssetId;
    } & Struct;
    readonly isSetFundingRound: boolean;
    readonly asSetFundingRound: {
      readonly assetId: PolymeshPrimitivesAssetAssetId;
      readonly fundingRoundName: Bytes;
    } & Struct;
    readonly isUpdateIdentifiers: boolean;
    readonly asUpdateIdentifiers: {
      readonly assetId: PolymeshPrimitivesAssetAssetId;
      readonly assetIdentifiers: Vec<PolymeshPrimitivesAssetIdentifier>;
    } & Struct;
    readonly isControllerTransfer: boolean;
    readonly asControllerTransfer: {
      readonly assetId: PolymeshPrimitivesAssetAssetId;
      readonly value: u128;
      readonly assetHolder: PolymeshPrimitivesAssetAssetHolder;
    } & Struct;
    readonly isRegisterCustomAssetType: boolean;
    readonly asRegisterCustomAssetType: {
      readonly ty: Bytes;
    } & Struct;
    readonly isCreateAssetWithCustomType: boolean;
    readonly asCreateAssetWithCustomType: {
      readonly assetName: Bytes;
      readonly divisible: bool;
      readonly customAssetType: Bytes;
      readonly assetIdentifiers: Vec<PolymeshPrimitivesAssetIdentifier>;
      readonly fundingRoundName: Option<Bytes>;
    } & Struct;
    readonly isSetAssetMetadata: boolean;
    readonly asSetAssetMetadata: {
      readonly assetId: PolymeshPrimitivesAssetAssetId;
      readonly key: PolymeshPrimitivesAssetMetadataAssetMetadataKey;
      readonly value: Bytes;
      readonly detail: Option<PolymeshPrimitivesAssetMetadataAssetMetadataValueDetail>;
    } & Struct;
    readonly isSetAssetMetadataDetails: boolean;
    readonly asSetAssetMetadataDetails: {
      readonly assetId: PolymeshPrimitivesAssetAssetId;
      readonly key: PolymeshPrimitivesAssetMetadataAssetMetadataKey;
      readonly detail: PolymeshPrimitivesAssetMetadataAssetMetadataValueDetail;
    } & Struct;
    readonly isRegisterAndSetLocalAssetMetadata: boolean;
    readonly asRegisterAndSetLocalAssetMetadata: {
      readonly assetId: PolymeshPrimitivesAssetAssetId;
      readonly name: Bytes;
      readonly spec: PolymeshPrimitivesAssetMetadataAssetMetadataSpec;
      readonly value: Bytes;
      readonly detail: Option<PolymeshPrimitivesAssetMetadataAssetMetadataValueDetail>;
    } & Struct;
    readonly isRegisterAssetMetadataLocalType: boolean;
    readonly asRegisterAssetMetadataLocalType: {
      readonly assetId: PolymeshPrimitivesAssetAssetId;
      readonly name: Bytes;
      readonly spec: PolymeshPrimitivesAssetMetadataAssetMetadataSpec;
    } & Struct;
    readonly isRegisterAssetMetadataGlobalType: boolean;
    readonly asRegisterAssetMetadataGlobalType: {
      readonly name: Bytes;
      readonly spec: PolymeshPrimitivesAssetMetadataAssetMetadataSpec;
    } & Struct;
    readonly isUpdateAssetType: boolean;
    readonly asUpdateAssetType: {
      readonly assetId: PolymeshPrimitivesAssetAssetId;
      readonly assetType: PolymeshPrimitivesAssetAssetType;
    } & Struct;
    readonly isRemoveLocalMetadataKey: boolean;
    readonly asRemoveLocalMetadataKey: {
      readonly assetId: PolymeshPrimitivesAssetAssetId;
      readonly localKey: u64;
    } & Struct;
    readonly isRemoveMetadataValue: boolean;
    readonly asRemoveMetadataValue: {
      readonly assetId: PolymeshPrimitivesAssetAssetId;
      readonly metadataKey: PolymeshPrimitivesAssetMetadataAssetMetadataKey;
    } & Struct;
    readonly isExemptAssetAffirmation: boolean;
    readonly asExemptAssetAffirmation: {
      readonly assetId: PolymeshPrimitivesAssetAssetId;
    } & Struct;
    readonly isRemoveAssetAffirmationExemption: boolean;
    readonly asRemoveAssetAffirmationExemption: {
      readonly assetId: PolymeshPrimitivesAssetAssetId;
    } & Struct;
    readonly isPreApproveAsset: boolean;
    readonly asPreApproveAsset: {
      readonly assetId: PolymeshPrimitivesAssetAssetId;
    } & Struct;
    readonly isRemoveAssetPreApproval: boolean;
    readonly asRemoveAssetPreApproval: {
      readonly assetId: PolymeshPrimitivesAssetAssetId;
    } & Struct;
    readonly isAddMandatoryMediators: boolean;
    readonly asAddMandatoryMediators: {
      readonly assetId: PolymeshPrimitivesAssetAssetId;
      readonly mediators: BTreeSet<PolymeshPrimitivesIdentityId>;
    } & Struct;
    readonly isRemoveMandatoryMediators: boolean;
    readonly asRemoveMandatoryMediators: {
      readonly assetId: PolymeshPrimitivesAssetAssetId;
      readonly mediators: BTreeSet<PolymeshPrimitivesIdentityId>;
    } & Struct;
    readonly isLinkTickerToAssetId: boolean;
    readonly asLinkTickerToAssetId: {
      readonly ticker: PolymeshPrimitivesTicker;
      readonly assetId: PolymeshPrimitivesAssetAssetId;
    } & Struct;
    readonly isUnlinkTickerFromAssetId: boolean;
    readonly asUnlinkTickerFromAssetId: {
      readonly ticker: PolymeshPrimitivesTicker;
      readonly assetId: PolymeshPrimitivesAssetAssetId;
    } & Struct;
    readonly isUpdateGlobalMetadataSpec: boolean;
    readonly asUpdateGlobalMetadataSpec: {
      readonly assetMetadataName: Bytes;
      readonly assetMetadataSpec: PolymeshPrimitivesAssetMetadataAssetMetadataSpec;
    } & Struct;
    readonly isTransferAsset: boolean;
    readonly asTransferAsset: {
      readonly assetId: PolymeshPrimitivesAssetAssetId;
      readonly to: AccountId32;
      readonly amount: u128;
      readonly memo: Option<PolymeshPrimitivesMemo>;
    } & Struct;
    readonly isReceiverAffirmAssetTransfer: boolean;
    readonly asReceiverAffirmAssetTransfer: {
      readonly transferId: u64;
    } & Struct;
    readonly isRejectAssetTransfer: boolean;
    readonly asRejectAssetTransfer: {
      readonly transferId: u64;
    } & Struct;
    readonly isSetMandatoryReceiverAffirmation: boolean;
    readonly asSetMandatoryReceiverAffirmation: {
      readonly require: bool;
    } & Struct;
    readonly type: 'RegisterUniqueTicker' | 'AcceptTickerTransfer' | 'AcceptAssetOwnershipTransfer' | 'CreateAsset' | 'Freeze' | 'Unfreeze' | 'RenameAsset' | 'Issue' | 'Redeem' | 'MakeDivisible' | 'AddDocuments' | 'RemoveDocuments' | 'SetFundingRound' | 'UpdateIdentifiers' | 'ControllerTransfer' | 'RegisterCustomAssetType' | 'CreateAssetWithCustomType' | 'SetAssetMetadata' | 'SetAssetMetadataDetails' | 'RegisterAndSetLocalAssetMetadata' | 'RegisterAssetMetadataLocalType' | 'RegisterAssetMetadataGlobalType' | 'UpdateAssetType' | 'RemoveLocalMetadataKey' | 'RemoveMetadataValue' | 'ExemptAssetAffirmation' | 'RemoveAssetAffirmationExemption' | 'PreApproveAsset' | 'RemoveAssetPreApproval' | 'AddMandatoryMediators' | 'RemoveMandatoryMediators' | 'LinkTickerToAssetId' | 'UnlinkTickerFromAssetId' | 'UpdateGlobalMetadataSpec' | 'TransferAsset' | 'ReceiverAffirmAssetTransfer' | 'RejectAssetTransfer' | 'SetMandatoryReceiverAffirmation';
  }

  /** @name PolymeshPrimitivesAssetAssetHolderKind (444) */
  interface PolymeshPrimitivesAssetAssetHolderKind extends Enum {
    readonly isAccount: boolean;
    readonly isDefaultPortfolio: boolean;
    readonly isUserPortfolio: boolean;
    readonly asUserPortfolio: u64;
    readonly type: 'Account' | 'DefaultPortfolio' | 'UserPortfolio';
  }

  /** @name PalletCorporateActionsDistributionPalletCall (447) */
  interface PalletCorporateActionsDistributionPalletCall extends Enum {
    readonly isDistribute: boolean;
    readonly asDistribute: {
      readonly caId: PalletCorporateActionsCaId;
      readonly portfolio: Option<u64>;
      readonly currency: PolymeshPrimitivesAssetAssetId;
      readonly perShare: u128;
      readonly amount: u128;
      readonly paymentAt: u64;
      readonly expiresAt: Option<u64>;
    } & Struct;
    readonly isClaim: boolean;
    readonly asClaim: {
      readonly caId: PalletCorporateActionsCaId;
    } & Struct;
    readonly isPushBenefit: boolean;
    readonly asPushBenefit: {
      readonly caId: PalletCorporateActionsCaId;
      readonly holder: PolymeshPrimitivesIdentityId;
    } & Struct;
    readonly isReclaim: boolean;
    readonly asReclaim: {
      readonly caId: PalletCorporateActionsCaId;
    } & Struct;
    readonly isRemoveDistribution: boolean;
    readonly asRemoveDistribution: {
      readonly caId: PalletCorporateActionsCaId;
    } & Struct;
    readonly type: 'Distribute' | 'Claim' | 'PushBenefit' | 'Reclaim' | 'RemoveDistribution';
  }

  /** @name PalletAssetCheckpointPalletCall (449) */
  interface PalletAssetCheckpointPalletCall extends Enum {
    readonly isCreateCheckpoint: boolean;
    readonly asCreateCheckpoint: {
      readonly assetId: PolymeshPrimitivesAssetAssetId;
    } & Struct;
    readonly isSetSchedulesMaxComplexity: boolean;
    readonly asSetSchedulesMaxComplexity: {
      readonly maxComplexity: u64;
    } & Struct;
    readonly isCreateSchedule: boolean;
    readonly asCreateSchedule: {
      readonly assetId: PolymeshPrimitivesAssetAssetId;
      readonly schedule: PolymeshPrimitivesCheckpointScheduleCheckpoints;
    } & Struct;
    readonly isRemoveSchedule: boolean;
    readonly asRemoveSchedule: {
      readonly assetId: PolymeshPrimitivesAssetAssetId;
      readonly id: u64;
    } & Struct;
    readonly type: 'CreateCheckpoint' | 'SetSchedulesMaxComplexity' | 'CreateSchedule' | 'RemoveSchedule';
  }

  /** @name PalletComplianceManagerCall (450) */
  interface PalletComplianceManagerCall extends Enum {
    readonly isAddComplianceRequirement: boolean;
    readonly asAddComplianceRequirement: {
      readonly assetId: PolymeshPrimitivesAssetAssetId;
      readonly senderConditions: Vec<PolymeshPrimitivesCondition>;
      readonly receiverConditions: Vec<PolymeshPrimitivesCondition>;
    } & Struct;
    readonly isRemoveComplianceRequirement: boolean;
    readonly asRemoveComplianceRequirement: {
      readonly assetId: PolymeshPrimitivesAssetAssetId;
      readonly id: u32;
    } & Struct;
    readonly isReplaceAssetCompliance: boolean;
    readonly asReplaceAssetCompliance: {
      readonly assetId: PolymeshPrimitivesAssetAssetId;
      readonly assetCompliance: Vec<PolymeshPrimitivesComplianceManagerComplianceRequirement>;
    } & Struct;
    readonly isResetAssetCompliance: boolean;
    readonly asResetAssetCompliance: {
      readonly assetId: PolymeshPrimitivesAssetAssetId;
    } & Struct;
    readonly isPauseAssetCompliance: boolean;
    readonly asPauseAssetCompliance: {
      readonly assetId: PolymeshPrimitivesAssetAssetId;
    } & Struct;
    readonly isResumeAssetCompliance: boolean;
    readonly asResumeAssetCompliance: {
      readonly assetId: PolymeshPrimitivesAssetAssetId;
    } & Struct;
    readonly isAddDefaultTrustedClaimIssuer: boolean;
    readonly asAddDefaultTrustedClaimIssuer: {
      readonly assetId: PolymeshPrimitivesAssetAssetId;
      readonly issuer: PolymeshPrimitivesConditionTrustedIssuer;
    } & Struct;
    readonly isRemoveDefaultTrustedClaimIssuer: boolean;
    readonly asRemoveDefaultTrustedClaimIssuer: {
      readonly assetId: PolymeshPrimitivesAssetAssetId;
      readonly issuer: PolymeshPrimitivesIdentityId;
    } & Struct;
    readonly isChangeComplianceRequirement: boolean;
    readonly asChangeComplianceRequirement: {
      readonly assetId: PolymeshPrimitivesAssetAssetId;
      readonly newReq: PolymeshPrimitivesComplianceManagerComplianceRequirement;
    } & Struct;
    readonly type: 'AddComplianceRequirement' | 'RemoveComplianceRequirement' | 'ReplaceAssetCompliance' | 'ResetAssetCompliance' | 'PauseAssetCompliance' | 'ResumeAssetCompliance' | 'AddDefaultTrustedClaimIssuer' | 'RemoveDefaultTrustedClaimIssuer' | 'ChangeComplianceRequirement';
  }

  /** @name PalletCorporateActionsCall (451) */
  interface PalletCorporateActionsCall extends Enum {
    readonly isSetMaxDetailsLength: boolean;
    readonly asSetMaxDetailsLength: {
      readonly length: u32;
    } & Struct;
    readonly isSetDefaultTargets: boolean;
    readonly asSetDefaultTargets: {
      readonly assetId: PolymeshPrimitivesAssetAssetId;
      readonly targets: PalletCorporateActionsTargetIdentities;
    } & Struct;
    readonly isSetDefaultWithholdingTax: boolean;
    readonly asSetDefaultWithholdingTax: {
      readonly assetId: PolymeshPrimitivesAssetAssetId;
      readonly tax: Permill;
    } & Struct;
    readonly isSetDidWithholdingTax: boolean;
    readonly asSetDidWithholdingTax: {
      readonly assetId: PolymeshPrimitivesAssetAssetId;
      readonly taxedDid: PolymeshPrimitivesIdentityId;
      readonly tax: Option<Permill>;
    } & Struct;
    readonly isInitiateCorporateAction: boolean;
    readonly asInitiateCorporateAction: {
      readonly assetId: PolymeshPrimitivesAssetAssetId;
      readonly kind: PalletCorporateActionsCaKind;
      readonly declDate: u64;
      readonly recordDate: Option<PalletCorporateActionsRecordDateSpec>;
      readonly details: Bytes;
      readonly targets: Option<PalletCorporateActionsTargetIdentities>;
      readonly defaultWithholdingTax: Option<Permill>;
      readonly withholdingTax: Option<Vec<ITuple<[PolymeshPrimitivesIdentityId, Permill]>>>;
    } & Struct;
    readonly isLinkCaDoc: boolean;
    readonly asLinkCaDoc: {
      readonly id: PalletCorporateActionsCaId;
      readonly docs: Vec<u32>;
    } & Struct;
    readonly isRemoveCa: boolean;
    readonly asRemoveCa: {
      readonly caId: PalletCorporateActionsCaId;
    } & Struct;
    readonly isChangeRecordDate: boolean;
    readonly asChangeRecordDate: {
      readonly caId: PalletCorporateActionsCaId;
      readonly recordDate: Option<PalletCorporateActionsRecordDateSpec>;
    } & Struct;
    readonly isInitiateCorporateActionAndDistribute: boolean;
    readonly asInitiateCorporateActionAndDistribute: {
      readonly caArgs: PalletCorporateActionsInitiateCorporateActionArgs;
      readonly portfolio: Option<u64>;
      readonly currency: PolymeshPrimitivesAssetAssetId;
      readonly perShare: u128;
      readonly amount: u128;
      readonly paymentAt: u64;
      readonly expiresAt: Option<u64>;
    } & Struct;
    readonly isInitiateCorporateActionAndBallot: boolean;
    readonly asInitiateCorporateActionAndBallot: {
      readonly caArgs: PalletCorporateActionsInitiateCorporateActionArgs;
      readonly ballotTimeRange: PalletCorporateActionsBallotBallotTimeRange;
      readonly ballotMeta: PalletCorporateActionsBallotBallotMeta;
      readonly rcv: bool;
    } & Struct;
    readonly type: 'SetMaxDetailsLength' | 'SetDefaultTargets' | 'SetDefaultWithholdingTax' | 'SetDidWithholdingTax' | 'InitiateCorporateAction' | 'LinkCaDoc' | 'RemoveCa' | 'ChangeRecordDate' | 'InitiateCorporateActionAndDistribute' | 'InitiateCorporateActionAndBallot';
  }

  /** @name PalletCorporateActionsRecordDateSpec (453) */
  interface PalletCorporateActionsRecordDateSpec extends Enum {
    readonly isScheduled: boolean;
    readonly asScheduled: u64;
    readonly isExistingSchedule: boolean;
    readonly asExistingSchedule: u64;
    readonly isExisting: boolean;
    readonly asExisting: u64;
    readonly type: 'Scheduled' | 'ExistingSchedule' | 'Existing';
  }

  /** @name PalletCorporateActionsInitiateCorporateActionArgs (456) */
  interface PalletCorporateActionsInitiateCorporateActionArgs extends Struct {
    readonly assetId: PolymeshPrimitivesAssetAssetId;
    readonly kind: PalletCorporateActionsCaKind;
    readonly declDate: u64;
    readonly recordDate: Option<PalletCorporateActionsRecordDateSpec>;
    readonly details: Bytes;
    readonly targets: Option<PalletCorporateActionsTargetIdentities>;
    readonly defaultWithholdingTax: Option<Permill>;
    readonly withholdingTax: Option<Vec<ITuple<[PolymeshPrimitivesIdentityId, Permill]>>>;
  }

  /** @name PalletCorporateActionsBallotPalletCall (457) */
  interface PalletCorporateActionsBallotPalletCall extends Enum {
    readonly isAttachBallot: boolean;
    readonly asAttachBallot: {
      readonly caId: PalletCorporateActionsCaId;
      readonly range: PalletCorporateActionsBallotBallotTimeRange;
      readonly meta: PalletCorporateActionsBallotBallotMeta;
      readonly rcv: bool;
    } & Struct;
    readonly isVote: boolean;
    readonly asVote: {
      readonly caId: PalletCorporateActionsCaId;
      readonly votes: Vec<PalletCorporateActionsBallotBallotVote>;
    } & Struct;
    readonly isChangeEnd: boolean;
    readonly asChangeEnd: {
      readonly caId: PalletCorporateActionsCaId;
      readonly end: u64;
    } & Struct;
    readonly isChangeMeta: boolean;
    readonly asChangeMeta: {
      readonly caId: PalletCorporateActionsCaId;
      readonly meta: PalletCorporateActionsBallotBallotMeta;
    } & Struct;
    readonly isChangeRcv: boolean;
    readonly asChangeRcv: {
      readonly caId: PalletCorporateActionsCaId;
      readonly rcv: bool;
    } & Struct;
    readonly isRemoveBallot: boolean;
    readonly asRemoveBallot: {
      readonly caId: PalletCorporateActionsCaId;
    } & Struct;
    readonly type: 'AttachBallot' | 'Vote' | 'ChangeEnd' | 'ChangeMeta' | 'ChangeRcv' | 'RemoveBallot';
  }

  /** @name PalletPipsCall (458) */
  interface PalletPipsCall extends Enum {
    readonly isSetPruneHistoricalPips: boolean;
    readonly asSetPruneHistoricalPips: {
      readonly prune: bool;
    } & Struct;
    readonly isSetMinProposalDeposit: boolean;
    readonly asSetMinProposalDeposit: {
      readonly deposit: u128;
    } & Struct;
    readonly isSetDefaultEnactmentPeriod: boolean;
    readonly asSetDefaultEnactmentPeriod: {
      readonly duration: u32;
    } & Struct;
    readonly isSetPendingPipExpiry: boolean;
    readonly asSetPendingPipExpiry: {
      readonly expiry: PolymeshPrimitivesMaybeBlock;
    } & Struct;
    readonly isSetMaxPipSkipCount: boolean;
    readonly asSetMaxPipSkipCount: {
      readonly max: u8;
    } & Struct;
    readonly isSetActivePipLimit: boolean;
    readonly asSetActivePipLimit: {
      readonly limit: u32;
    } & Struct;
    readonly isPropose: boolean;
    readonly asPropose: {
      readonly proposal: Call;
      readonly deposit: u128;
      readonly url: Option<Bytes>;
      readonly description: Option<Bytes>;
    } & Struct;
    readonly isVote: boolean;
    readonly asVote: {
      readonly id: u32;
      readonly ayeOrNay: bool;
      readonly deposit: u128;
    } & Struct;
    readonly isApproveCommitteeProposal: boolean;
    readonly asApproveCommitteeProposal: {
      readonly id: u32;
    } & Struct;
    readonly isRejectProposal: boolean;
    readonly asRejectProposal: {
      readonly id: u32;
    } & Struct;
    readonly isPruneProposal: boolean;
    readonly asPruneProposal: {
      readonly id: u32;
    } & Struct;
    readonly isRescheduleExecution: boolean;
    readonly asRescheduleExecution: {
      readonly id: u32;
      readonly until: Option<u32>;
    } & Struct;
    readonly isClearSnapshot: boolean;
    readonly isSnapshot: boolean;
    readonly isEnactSnapshotResults: boolean;
    readonly asEnactSnapshotResults: {
      readonly results: Vec<ITuple<[u32, PalletPipsSnapshotResult]>>;
    } & Struct;
    readonly isExecuteScheduledPip: boolean;
    readonly asExecuteScheduledPip: {
      readonly id: u32;
    } & Struct;
    readonly isExpireScheduledPip: boolean;
    readonly asExpireScheduledPip: {
      readonly did: PolymeshPrimitivesIdentityId;
      readonly id: u32;
    } & Struct;
    readonly type: 'SetPruneHistoricalPips' | 'SetMinProposalDeposit' | 'SetDefaultEnactmentPeriod' | 'SetPendingPipExpiry' | 'SetMaxPipSkipCount' | 'SetActivePipLimit' | 'Propose' | 'Vote' | 'ApproveCommitteeProposal' | 'RejectProposal' | 'PruneProposal' | 'RescheduleExecution' | 'ClearSnapshot' | 'Snapshot' | 'EnactSnapshotResults' | 'ExecuteScheduledPip' | 'ExpireScheduledPip';
  }

  /** @name PalletPipsSnapshotResult (461) */
  interface PalletPipsSnapshotResult extends Enum {
    readonly isApprove: boolean;
    readonly isReject: boolean;
    readonly isSkip: boolean;
    readonly type: 'Approve' | 'Reject' | 'Skip';
  }

  /** @name PalletPortfolioCall (462) */
  interface PalletPortfolioCall extends Enum {
    readonly isCreatePortfolio: boolean;
    readonly asCreatePortfolio: {
      readonly name: Bytes;
    } & Struct;
    readonly isDeletePortfolio: boolean;
    readonly asDeletePortfolio: {
      readonly portfolioNumber: u64;
    } & Struct;
    readonly isRenamePortfolio: boolean;
    readonly asRenamePortfolio: {
      readonly portfolioNumber: u64;
      readonly newPortfolioName: Bytes;
    } & Struct;
    readonly isQuitPortfolioCustody: boolean;
    readonly asQuitPortfolioCustody: {
      readonly pid: PolymeshPrimitivesIdentityIdPortfolioId;
    } & Struct;
    readonly isAcceptPortfolioCustody: boolean;
    readonly asAcceptPortfolioCustody: {
      readonly authId: u64;
    } & Struct;
    readonly isMovePortfolioFunds: boolean;
    readonly asMovePortfolioFunds: {
      readonly from: PolymeshPrimitivesIdentityIdPortfolioId;
      readonly to: PolymeshPrimitivesIdentityIdPortfolioId;
      readonly funds: Vec<PolymeshPrimitivesPortfolioFund>;
    } & Struct;
    readonly isPreApprovePortfolio: boolean;
    readonly asPreApprovePortfolio: {
      readonly assetId: PolymeshPrimitivesAssetAssetId;
      readonly portfolioId: PolymeshPrimitivesIdentityIdPortfolioId;
    } & Struct;
    readonly isRemovePortfolioPreApproval: boolean;
    readonly asRemovePortfolioPreApproval: {
      readonly assetId: PolymeshPrimitivesAssetAssetId;
      readonly portfolioId: PolymeshPrimitivesIdentityIdPortfolioId;
    } & Struct;
    readonly isAllowIdentityToCreatePortfolios: boolean;
    readonly asAllowIdentityToCreatePortfolios: {
      readonly trustedIdentity: PolymeshPrimitivesIdentityId;
    } & Struct;
    readonly isRevokeCreatePortfoliosPermission: boolean;
    readonly asRevokeCreatePortfoliosPermission: {
      readonly identity: PolymeshPrimitivesIdentityId;
    } & Struct;
    readonly isCreateCustodyPortfolio: boolean;
    readonly asCreateCustodyPortfolio: {
      readonly portfolioOwnerId: PolymeshPrimitivesIdentityId;
      readonly portfolioName: Bytes;
    } & Struct;
    readonly type: 'CreatePortfolio' | 'DeletePortfolio' | 'RenamePortfolio' | 'QuitPortfolioCustody' | 'AcceptPortfolioCustody' | 'MovePortfolioFunds' | 'PreApprovePortfolio' | 'RemovePortfolioPreApproval' | 'AllowIdentityToCreatePortfolios' | 'RevokeCreatePortfoliosPermission' | 'CreateCustodyPortfolio';
  }

  /** @name PolymeshPrimitivesPortfolioFund (464) */
  interface PolymeshPrimitivesPortfolioFund extends Struct {
    readonly description: PolymeshPrimitivesPortfolioFundDescription;
    readonly memo: Option<PolymeshPrimitivesMemo>;
  }

  /** @name PalletProtocolFeeCall (465) */
  interface PalletProtocolFeeCall extends Enum {
    readonly isChangeCoefficient: boolean;
    readonly asChangeCoefficient: {
      readonly coefficient: PolymeshPrimitivesPosRatio;
    } & Struct;
    readonly isChangeBaseFee: boolean;
    readonly asChangeBaseFee: {
      readonly op: PolymeshPrimitivesProtocolFeeProtocolOp;
      readonly baseFee: u128;
    } & Struct;
    readonly type: 'ChangeCoefficient' | 'ChangeBaseFee';
  }

  /** @name PolymeshPrimitivesProtocolFeeProtocolOp (466) */
  interface PolymeshPrimitivesProtocolFeeProtocolOp extends Enum {
    readonly isAssetRegisterTicker: boolean;
    readonly isAssetIssue: boolean;
    readonly isAssetAddDocuments: boolean;
    readonly isAssetCreateAsset: boolean;
    readonly isCheckpointCreateSchedule: boolean;
    readonly isComplianceManagerAddComplianceRequirement: boolean;
    readonly isIdentityCddRegisterDid: boolean;
    readonly isIdentityAddClaim: boolean;
    readonly isIdentityAddSecondaryKeysWithAuthorization: boolean;
    readonly isPipsPropose: boolean;
    readonly isContractsPutCode: boolean;
    readonly isCorporateBallotAttachBallot: boolean;
    readonly isCapitalDistributionDistribute: boolean;
    readonly isNftCreateCollection: boolean;
    readonly isNftMint: boolean;
    readonly isIdentityCreateChildIdentity: boolean;
    readonly type: 'AssetRegisterTicker' | 'AssetIssue' | 'AssetAddDocuments' | 'AssetCreateAsset' | 'CheckpointCreateSchedule' | 'ComplianceManagerAddComplianceRequirement' | 'IdentityCddRegisterDid' | 'IdentityAddClaim' | 'IdentityAddSecondaryKeysWithAuthorization' | 'PipsPropose' | 'ContractsPutCode' | 'CorporateBallotAttachBallot' | 'CapitalDistributionDistribute' | 'NftCreateCollection' | 'NftMint' | 'IdentityCreateChildIdentity';
  }

  /** @name PalletSchedulerCall (467) */
  interface PalletSchedulerCall extends Enum {
    readonly isSchedule: boolean;
    readonly asSchedule: {
      readonly when: u32;
      readonly maybePeriodic: Option<ITuple<[u32, u32]>>;
      readonly priority: u8;
      readonly call: Call;
    } & Struct;
    readonly isCancel: boolean;
    readonly asCancel: {
      readonly when: u32;
      readonly index: u32;
    } & Struct;
    readonly isScheduleNamed: boolean;
    readonly asScheduleNamed: {
      readonly id: U8aFixed;
      readonly when: u32;
      readonly maybePeriodic: Option<ITuple<[u32, u32]>>;
      readonly priority: u8;
      readonly call: Call;
    } & Struct;
    readonly isCancelNamed: boolean;
    readonly asCancelNamed: {
      readonly id: U8aFixed;
    } & Struct;
    readonly isScheduleAfter: boolean;
    readonly asScheduleAfter: {
      readonly after: u32;
      readonly maybePeriodic: Option<ITuple<[u32, u32]>>;
      readonly priority: u8;
      readonly call: Call;
    } & Struct;
    readonly isScheduleNamedAfter: boolean;
    readonly asScheduleNamedAfter: {
      readonly id: U8aFixed;
      readonly after: u32;
      readonly maybePeriodic: Option<ITuple<[u32, u32]>>;
      readonly priority: u8;
      readonly call: Call;
    } & Struct;
    readonly isSetRetry: boolean;
    readonly asSetRetry: {
      readonly task: ITuple<[u32, u32]>;
      readonly retries: u8;
      readonly period: u32;
    } & Struct;
    readonly isSetRetryNamed: boolean;
    readonly asSetRetryNamed: {
      readonly id: U8aFixed;
      readonly retries: u8;
      readonly period: u32;
    } & Struct;
    readonly isCancelRetry: boolean;
    readonly asCancelRetry: {
      readonly task: ITuple<[u32, u32]>;
    } & Struct;
    readonly isCancelRetryNamed: boolean;
    readonly asCancelRetryNamed: {
      readonly id: U8aFixed;
    } & Struct;
    readonly type: 'Schedule' | 'Cancel' | 'ScheduleNamed' | 'CancelNamed' | 'ScheduleAfter' | 'ScheduleNamedAfter' | 'SetRetry' | 'SetRetryNamed' | 'CancelRetry' | 'CancelRetryNamed';
  }

  /** @name PalletSettlementCall (469) */
  interface PalletSettlementCall extends Enum {
    readonly isCreateVenue: boolean;
    readonly asCreateVenue: {
      readonly details: Bytes;
      readonly signers: Vec<AccountId32>;
      readonly typ: PolymeshPrimitivesSettlementVenueType;
    } & Struct;
    readonly isUpdateVenueDetails: boolean;
    readonly asUpdateVenueDetails: {
      readonly id: u64;
      readonly details: Bytes;
    } & Struct;
    readonly isUpdateVenueType: boolean;
    readonly asUpdateVenueType: {
      readonly id: u64;
      readonly typ: PolymeshPrimitivesSettlementVenueType;
    } & Struct;
    readonly isAffirmWithReceipts: boolean;
    readonly asAffirmWithReceipts: {
      readonly id: u64;
      readonly receiptDetails: Vec<PolymeshPrimitivesSettlementReceiptDetails>;
      readonly holderSet: BTreeSet<PolymeshPrimitivesAssetAssetHolder>;
    } & Struct;
    readonly isSetVenueFiltering: boolean;
    readonly asSetVenueFiltering: {
      readonly assetId: PolymeshPrimitivesAssetAssetId;
      readonly enabled: bool;
    } & Struct;
    readonly isAllowVenues: boolean;
    readonly asAllowVenues: {
      readonly assetId: PolymeshPrimitivesAssetAssetId;
      readonly venues: Vec<u64>;
    } & Struct;
    readonly isDisallowVenues: boolean;
    readonly asDisallowVenues: {
      readonly assetId: PolymeshPrimitivesAssetAssetId;
      readonly venues: Vec<u64>;
    } & Struct;
    readonly isUpdateVenueSigners: boolean;
    readonly asUpdateVenueSigners: {
      readonly id: u64;
      readonly signers: Vec<AccountId32>;
      readonly addSigners: bool;
    } & Struct;
    readonly isExecuteManualInstruction: boolean;
    readonly asExecuteManualInstruction: {
      readonly id: u64;
      readonly assetHolder: Option<PolymeshPrimitivesAssetAssetHolder>;
      readonly fungibleTransfers: u32;
      readonly nftsTransfers: u32;
      readonly offchainTransfers: u32;
      readonly weightLimit: Option<SpWeightsWeightV2Weight>;
    } & Struct;
    readonly isAddInstruction: boolean;
    readonly asAddInstruction: {
      readonly venueId: Option<u64>;
      readonly settlementType: PolymeshPrimitivesSettlementSettlementType;
      readonly tradeDate: Option<u64>;
      readonly valueDate: Option<u64>;
      readonly legs: Vec<PolymeshPrimitivesSettlementLeg>;
      readonly instructionMemo: Option<PolymeshPrimitivesMemo>;
    } & Struct;
    readonly isAddAndAffirmInstruction: boolean;
    readonly asAddAndAffirmInstruction: {
      readonly venueId: Option<u64>;
      readonly settlementType: PolymeshPrimitivesSettlementSettlementType;
      readonly tradeDate: Option<u64>;
      readonly valueDate: Option<u64>;
      readonly legs: Vec<PolymeshPrimitivesSettlementLeg>;
      readonly holderSet: BTreeSet<PolymeshPrimitivesAssetAssetHolder>;
      readonly instructionMemo: Option<PolymeshPrimitivesMemo>;
    } & Struct;
    readonly isAffirmInstruction: boolean;
    readonly asAffirmInstruction: {
      readonly id: u64;
      readonly holderSet: BTreeSet<PolymeshPrimitivesAssetAssetHolder>;
    } & Struct;
    readonly isWithdrawAffirmation: boolean;
    readonly asWithdrawAffirmation: {
      readonly id: u64;
      readonly holderSet: BTreeSet<PolymeshPrimitivesAssetAssetHolder>;
    } & Struct;
    readonly isRejectInstruction: boolean;
    readonly asRejectInstruction: {
      readonly id: u64;
      readonly assetHolder: PolymeshPrimitivesAssetAssetHolder;
    } & Struct;
    readonly isExecuteScheduledInstruction: boolean;
    readonly asExecuteScheduledInstruction: {
      readonly id: u64;
      readonly weightLimit: SpWeightsWeightV2Weight;
    } & Struct;
    readonly isAffirmWithReceiptsWithCount: boolean;
    readonly asAffirmWithReceiptsWithCount: {
      readonly id: u64;
      readonly receiptDetails: Vec<PolymeshPrimitivesSettlementReceiptDetails>;
      readonly holderSet: BTreeSet<PolymeshPrimitivesAssetAssetHolder>;
      readonly numberOfAssets: Option<PolymeshPrimitivesSettlementAffirmationCount>;
    } & Struct;
    readonly isAffirmInstructionWithCount: boolean;
    readonly asAffirmInstructionWithCount: {
      readonly id: u64;
      readonly holderSet: BTreeSet<PolymeshPrimitivesAssetAssetHolder>;
      readonly numberOfAssets: Option<PolymeshPrimitivesSettlementAffirmationCount>;
    } & Struct;
    readonly isRejectInstructionWithCount: boolean;
    readonly asRejectInstructionWithCount: {
      readonly id: u64;
      readonly assetHolder: PolymeshPrimitivesAssetAssetHolder;
      readonly numberOfAssets: Option<PolymeshPrimitivesSettlementAssetCount>;
    } & Struct;
    readonly isWithdrawAffirmationWithCount: boolean;
    readonly asWithdrawAffirmationWithCount: {
      readonly id: u64;
      readonly holderSet: BTreeSet<PolymeshPrimitivesAssetAssetHolder>;
      readonly numberOfAssets: Option<PolymeshPrimitivesSettlementAffirmationCount>;
    } & Struct;
    readonly isAddInstructionWithMediators: boolean;
    readonly asAddInstructionWithMediators: {
      readonly venueId: Option<u64>;
      readonly settlementType: PolymeshPrimitivesSettlementSettlementType;
      readonly tradeDate: Option<u64>;
      readonly valueDate: Option<u64>;
      readonly legs: Vec<PolymeshPrimitivesSettlementLeg>;
      readonly instructionMemo: Option<PolymeshPrimitivesMemo>;
      readonly mediators: BTreeSet<PolymeshPrimitivesIdentityId>;
    } & Struct;
    readonly isAddAndAffirmWithMediators: boolean;
    readonly asAddAndAffirmWithMediators: {
      readonly venueId: Option<u64>;
      readonly settlementType: PolymeshPrimitivesSettlementSettlementType;
      readonly tradeDate: Option<u64>;
      readonly valueDate: Option<u64>;
      readonly legs: Vec<PolymeshPrimitivesSettlementLeg>;
      readonly holderSet: BTreeSet<PolymeshPrimitivesAssetAssetHolder>;
      readonly instructionMemo: Option<PolymeshPrimitivesMemo>;
      readonly mediators: BTreeSet<PolymeshPrimitivesIdentityId>;
    } & Struct;
    readonly isAffirmInstructionAsMediator: boolean;
    readonly asAffirmInstructionAsMediator: {
      readonly instructionId: u64;
      readonly expiry: Option<u64>;
    } & Struct;
    readonly isWithdrawAffirmationAsMediator: boolean;
    readonly asWithdrawAffirmationAsMediator: {
      readonly instructionId: u64;
    } & Struct;
    readonly isRejectInstructionAsMediator: boolean;
    readonly asRejectInstructionAsMediator: {
      readonly instructionId: u64;
      readonly numberOfAssets: Option<PolymeshPrimitivesSettlementAssetCount>;
    } & Struct;
    readonly isLockInstruction: boolean;
    readonly asLockInstruction: {
      readonly instId: u64;
      readonly weightLimit: SpWeightsWeightV2Weight;
    } & Struct;
    readonly type: 'CreateVenue' | 'UpdateVenueDetails' | 'UpdateVenueType' | 'AffirmWithReceipts' | 'SetVenueFiltering' | 'AllowVenues' | 'DisallowVenues' | 'UpdateVenueSigners' | 'ExecuteManualInstruction' | 'AddInstruction' | 'AddAndAffirmInstruction' | 'AffirmInstruction' | 'WithdrawAffirmation' | 'RejectInstruction' | 'ExecuteScheduledInstruction' | 'AffirmWithReceiptsWithCount' | 'AffirmInstructionWithCount' | 'RejectInstructionWithCount' | 'WithdrawAffirmationWithCount' | 'AddInstructionWithMediators' | 'AddAndAffirmWithMediators' | 'AffirmInstructionAsMediator' | 'WithdrawAffirmationAsMediator' | 'RejectInstructionAsMediator' | 'LockInstruction';
  }

  /** @name PolymeshPrimitivesSettlementReceiptDetails (471) */
  interface PolymeshPrimitivesSettlementReceiptDetails extends Struct {
    readonly uid: u64;
    readonly instructionId: u64;
    readonly legId: u64;
    readonly signer: AccountId32;
    readonly signature: SpRuntimeMultiSignature;
    readonly expiresAt: u64;
    readonly metadata: Option<PolymeshPrimitivesSettlementReceiptMetadata>;
  }

  /** @name SpRuntimeMultiSignature (472) */
  interface SpRuntimeMultiSignature extends Enum {
    readonly isEd25519: boolean;
    readonly asEd25519: U8aFixed;
    readonly isSr25519: boolean;
    readonly asSr25519: U8aFixed;
    readonly isEcdsa: boolean;
    readonly asEcdsa: U8aFixed;
    readonly isEth: boolean;
    readonly asEth: U8aFixed;
    readonly type: 'Ed25519' | 'Sr25519' | 'Ecdsa' | 'Eth';
  }

  /** @name PolymeshPrimitivesSettlementAffirmationCount (478) */
  interface PolymeshPrimitivesSettlementAffirmationCount extends Struct {
    readonly senderAssetCount: PolymeshPrimitivesSettlementAssetCount;
    readonly receiverAssetCount: PolymeshPrimitivesSettlementAssetCount;
    readonly offchainCount: u32;
  }

  /** @name PolymeshPrimitivesSettlementAssetCount (479) */
  interface PolymeshPrimitivesSettlementAssetCount extends Struct {
    readonly fungible: u32;
    readonly nonFungible: u32;
    readonly offChain: u32;
  }

  /** @name PalletStatisticsCall (482) */
  interface PalletStatisticsCall extends Enum {
    readonly isSetActiveAssetStats: boolean;
    readonly asSetActiveAssetStats: {
      readonly assetId: PolymeshPrimitivesAssetAssetId;
      readonly statTypes: BTreeSet<PolymeshPrimitivesStatisticsStatType>;
    } & Struct;
    readonly isBatchUpdateAssetStats: boolean;
    readonly asBatchUpdateAssetStats: {
      readonly assetId: PolymeshPrimitivesAssetAssetId;
      readonly statType: PolymeshPrimitivesStatisticsStatType;
      readonly values: BTreeSet<PolymeshPrimitivesStatisticsStatUpdate>;
    } & Struct;
    readonly isSetAssetTransferCompliance: boolean;
    readonly asSetAssetTransferCompliance: {
      readonly assetId: PolymeshPrimitivesAssetAssetId;
      readonly transferConditions: BTreeSet<PolymeshPrimitivesTransferComplianceTransferCondition>;
    } & Struct;
    readonly isSetEntitiesExempt: boolean;
    readonly asSetEntitiesExempt: {
      readonly isExempt: bool;
      readonly exemptKey: PolymeshPrimitivesTransferComplianceTransferConditionExemptKey;
      readonly entities: BTreeSet<PolymeshPrimitivesIdentityId>;
    } & Struct;
    readonly type: 'SetActiveAssetStats' | 'BatchUpdateAssetStats' | 'SetAssetTransferCompliance' | 'SetEntitiesExempt';
  }

  /** @name PalletStoCall (486) */
  interface PalletStoCall extends Enum {
    readonly isCreateFundraiser: boolean;
    readonly asCreateFundraiser: {
      readonly offeringPortfolio: PolymeshPrimitivesIdentityIdPortfolioId;
      readonly offeringAsset: PolymeshPrimitivesAssetAssetId;
      readonly raisingPortfolio: PolymeshPrimitivesIdentityIdPortfolioId;
      readonly raisingAsset: PolymeshPrimitivesAssetAssetId;
      readonly tiers: Vec<PalletStoPriceTier>;
      readonly venueId: u64;
      readonly start: Option<u64>;
      readonly end: Option<u64>;
      readonly minimumInvestment: u128;
      readonly fundraiserName: Bytes;
    } & Struct;
    readonly isInvest: boolean;
    readonly asInvest: {
      readonly offeringAsset: PolymeshPrimitivesAssetAssetId;
      readonly fundraiserId: u64;
      readonly investmentPortfolio: PolymeshPrimitivesIdentityIdPortfolioId;
      readonly funding: PalletStoFundingMethod;
      readonly purchaseAmount: u128;
      readonly maxPrice: Option<u128>;
    } & Struct;
    readonly isFreezeFundraiser: boolean;
    readonly asFreezeFundraiser: {
      readonly offeringAsset: PolymeshPrimitivesAssetAssetId;
      readonly fundraiserId: u64;
    } & Struct;
    readonly isUnfreezeFundraiser: boolean;
    readonly asUnfreezeFundraiser: {
      readonly offeringAsset: PolymeshPrimitivesAssetAssetId;
      readonly fundraiserId: u64;
    } & Struct;
    readonly isModifyFundraiserWindow: boolean;
    readonly asModifyFundraiserWindow: {
      readonly offeringAsset: PolymeshPrimitivesAssetAssetId;
      readonly fundraiserId: u64;
      readonly start: u64;
      readonly end: Option<u64>;
    } & Struct;
    readonly isStop: boolean;
    readonly asStop: {
      readonly offeringAsset: PolymeshPrimitivesAssetAssetId;
      readonly fundraiserId: u64;
    } & Struct;
    readonly isEnableOffchainFunding: boolean;
    readonly asEnableOffchainFunding: {
      readonly offeringAsset: PolymeshPrimitivesAssetAssetId;
      readonly fundraiserId: u64;
      readonly ticker: PolymeshPrimitivesTicker;
    } & Struct;
    readonly type: 'CreateFundraiser' | 'Invest' | 'FreezeFundraiser' | 'UnfreezeFundraiser' | 'ModifyFundraiserWindow' | 'Stop' | 'EnableOffchainFunding';
  }

  /** @name PalletStoPriceTier (488) */
  interface PalletStoPriceTier extends Struct {
    readonly total: u128;
    readonly price: u128;
  }

  /** @name PalletStoFundingMethod (489) */
  interface PalletStoFundingMethod extends Enum {
    readonly isOnChain: boolean;
    readonly asOnChain: PolymeshPrimitivesIdentityIdPortfolioId;
    readonly isOffChain: boolean;
    readonly asOffChain: PolymeshPrimitivesStoFundraiserReceiptDetails;
    readonly type: 'OnChain' | 'OffChain';
  }

  /** @name PolymeshPrimitivesStoFundraiserReceiptDetails (490) */
  interface PolymeshPrimitivesStoFundraiserReceiptDetails extends Struct {
    readonly uid: u64;
    readonly signer: AccountId32;
    readonly signature: SpRuntimeMultiSignature;
    readonly expiresAt: u64;
    readonly metadata: Option<PolymeshPrimitivesSettlementReceiptMetadata>;
  }

  /** @name PalletTreasuryCall (491) */
  interface PalletTreasuryCall extends Enum {
    readonly isDisbursement: boolean;
    readonly asDisbursement: {
      readonly beneficiaries: Vec<PolymeshPrimitivesBeneficiary>;
    } & Struct;
    readonly isReimbursement: boolean;
    readonly asReimbursement: {
      readonly amount: u128;
    } & Struct;
    readonly type: 'Disbursement' | 'Reimbursement';
  }

  /** @name PolymeshPrimitivesBeneficiary (493) */
  interface PolymeshPrimitivesBeneficiary extends Struct {
    readonly id: PolymeshPrimitivesIdentityId;
    readonly amount: u128;
  }

  /** @name PalletUtilityCall (494) */
  interface PalletUtilityCall extends Enum {
    readonly isBatch: boolean;
    readonly asBatch: {
      readonly calls: Vec<Call>;
    } & Struct;
    readonly isBatchAll: boolean;
    readonly asBatchAll: {
      readonly calls: Vec<Call>;
    } & Struct;
    readonly isDispatchAs: boolean;
    readonly asDispatchAs: {
      readonly asOrigin: PolymeshRuntimeDevelopRuntimeOriginCaller;
      readonly call: Call;
    } & Struct;
    readonly isForceBatch: boolean;
    readonly asForceBatch: {
      readonly calls: Vec<Call>;
    } & Struct;
    readonly isWithWeight: boolean;
    readonly asWithWeight: {
      readonly call: Call;
      readonly weight: SpWeightsWeightV2Weight;
    } & Struct;
    readonly isAsDerivative: boolean;
    readonly asAsDerivative: {
      readonly index: u16;
      readonly call: Call;
    } & Struct;
    readonly type: 'Batch' | 'BatchAll' | 'DispatchAs' | 'ForceBatch' | 'WithWeight' | 'AsDerivative';
  }

  /** @name PolymeshRuntimeDevelopRuntimeOriginCaller (496) */
  interface PolymeshRuntimeDevelopRuntimeOriginCaller extends Enum {
    readonly isSystem: boolean;
    readonly asSystem: FrameSupportDispatchRawOrigin;
    readonly isPolymeshCommittee: boolean;
    readonly asPolymeshCommittee: PalletCommitteeRawOrigin;
    readonly isTechnicalCommittee: boolean;
    readonly asTechnicalCommittee: PalletCommitteeRawOrigin;
    readonly isUpgradeCommittee: boolean;
    readonly asUpgradeCommittee: PalletCommitteeRawOrigin;
    readonly type: 'System' | 'PolymeshCommittee' | 'TechnicalCommittee' | 'UpgradeCommittee';
  }

  /** @name FrameSupportDispatchRawOrigin (497) */
  interface FrameSupportDispatchRawOrigin extends Enum {
    readonly isRoot: boolean;
    readonly isSigned: boolean;
    readonly asSigned: AccountId32;
    readonly isNone: boolean;
    readonly isAuthorized: boolean;
    readonly type: 'Root' | 'Signed' | 'None' | 'Authorized';
  }

  /** @name PalletCommitteeRawOrigin (498) */
  interface PalletCommitteeRawOrigin extends Enum {
    readonly isEndorsed: boolean;
    readonly type: 'Endorsed';
  }

  /** @name PalletBaseCall (501) */
  type PalletBaseCall = Null;

  /** @name PalletExternalAgentsCall (502) */
  interface PalletExternalAgentsCall extends Enum {
    readonly isCreateGroup: boolean;
    readonly asCreateGroup: {
      readonly assetId: PolymeshPrimitivesAssetAssetId;
      readonly perms: PolymeshPrimitivesSecondaryKeyExtrinsicPermissions;
    } & Struct;
    readonly isSetGroupPermissions: boolean;
    readonly asSetGroupPermissions: {
      readonly assetId: PolymeshPrimitivesAssetAssetId;
      readonly id: u32;
      readonly perms: PolymeshPrimitivesSecondaryKeyExtrinsicPermissions;
    } & Struct;
    readonly isRemoveAgent: boolean;
    readonly asRemoveAgent: {
      readonly assetId: PolymeshPrimitivesAssetAssetId;
      readonly agent: PolymeshPrimitivesIdentityId;
    } & Struct;
    readonly isAbdicate: boolean;
    readonly asAbdicate: {
      readonly assetId: PolymeshPrimitivesAssetAssetId;
    } & Struct;
    readonly isChangeGroup: boolean;
    readonly asChangeGroup: {
      readonly assetId: PolymeshPrimitivesAssetAssetId;
      readonly agent: PolymeshPrimitivesIdentityId;
      readonly group: PolymeshPrimitivesAgentAgentGroup;
    } & Struct;
    readonly isAcceptBecomeAgent: boolean;
    readonly asAcceptBecomeAgent: {
      readonly authId: u64;
    } & Struct;
    readonly isCreateGroupAndAddAuth: boolean;
    readonly asCreateGroupAndAddAuth: {
      readonly assetId: PolymeshPrimitivesAssetAssetId;
      readonly perms: PolymeshPrimitivesSecondaryKeyExtrinsicPermissions;
      readonly target: PolymeshPrimitivesIdentityId;
      readonly expiry: Option<u64>;
    } & Struct;
    readonly isCreateAndChangeCustomGroup: boolean;
    readonly asCreateAndChangeCustomGroup: {
      readonly assetId: PolymeshPrimitivesAssetAssetId;
      readonly perms: PolymeshPrimitivesSecondaryKeyExtrinsicPermissions;
      readonly agent: PolymeshPrimitivesIdentityId;
    } & Struct;
    readonly type: 'CreateGroup' | 'SetGroupPermissions' | 'RemoveAgent' | 'Abdicate' | 'ChangeGroup' | 'AcceptBecomeAgent' | 'CreateGroupAndAddAuth' | 'CreateAndChangeCustomGroup';
  }

  /** @name PalletRelayerCall (503) */
  interface PalletRelayerCall extends Enum {
    readonly isApproveSubsidy: boolean;
    readonly asApproveSubsidy: {
      readonly userKey: AccountId32;
      readonly polyxLimit: u128;
    } & Struct;
    readonly isRevokeSubsidy: boolean;
    readonly asRevokeSubsidy: {
      readonly userKey: AccountId32;
    } & Struct;
    readonly isAcceptSubsidy: boolean;
    readonly asAcceptSubsidy: {
      readonly payingKey: AccountId32;
    } & Struct;
    readonly isRemoveSubsidy: boolean;
    readonly asRemoveSubsidy: {
      readonly userKey: AccountId32;
      readonly payingKey: AccountId32;
    } & Struct;
    readonly isUpdatePolyxLimit: boolean;
    readonly asUpdatePolyxLimit: {
      readonly userKey: AccountId32;
      readonly polyxLimit: u128;
    } & Struct;
    readonly isIncreasePolyxLimit: boolean;
    readonly asIncreasePolyxLimit: {
      readonly userKey: AccountId32;
      readonly amount: u128;
    } & Struct;
    readonly isDecreasePolyxLimit: boolean;
    readonly asDecreasePolyxLimit: {
      readonly userKey: AccountId32;
      readonly amount: u128;
    } & Struct;
    readonly isRelayTx: boolean;
    readonly asRelayTx: {
      readonly target: AccountId32;
      readonly signature: SpRuntimeMultiSignature;
      readonly call: Call;
      readonly expiresAt: u64;
    } & Struct;
    readonly type: 'ApproveSubsidy' | 'RevokeSubsidy' | 'AcceptSubsidy' | 'RemoveSubsidy' | 'UpdatePolyxLimit' | 'IncreasePolyxLimit' | 'DecreasePolyxLimit' | 'RelayTx';
  }

  /** @name PalletContractsCall (504) */
  interface PalletContractsCall extends Enum {
    readonly isCallOldWeight: boolean;
    readonly asCallOldWeight: {
      readonly dest: MultiAddress;
      readonly value: Compact<u128>;
      readonly gasLimit: Compact<u64>;
      readonly storageDepositLimit: Option<Compact<u128>>;
      readonly data: Bytes;
    } & Struct;
    readonly isInstantiateWithCodeOldWeight: boolean;
    readonly asInstantiateWithCodeOldWeight: {
      readonly value: Compact<u128>;
      readonly gasLimit: Compact<u64>;
      readonly storageDepositLimit: Option<Compact<u128>>;
      readonly code: Bytes;
      readonly data: Bytes;
      readonly salt: Bytes;
    } & Struct;
    readonly isInstantiateOldWeight: boolean;
    readonly asInstantiateOldWeight: {
      readonly value: Compact<u128>;
      readonly gasLimit: Compact<u64>;
      readonly storageDepositLimit: Option<Compact<u128>>;
      readonly codeHash: H256;
      readonly data: Bytes;
      readonly salt: Bytes;
    } & Struct;
    readonly isUploadCode: boolean;
    readonly asUploadCode: {
      readonly code: Bytes;
      readonly storageDepositLimit: Option<Compact<u128>>;
      readonly determinism: PalletContractsWasmDeterminism;
    } & Struct;
    readonly isRemoveCode: boolean;
    readonly asRemoveCode: {
      readonly codeHash: H256;
    } & Struct;
    readonly isSetCode: boolean;
    readonly asSetCode: {
      readonly dest: MultiAddress;
      readonly codeHash: H256;
    } & Struct;
    readonly isCall: boolean;
    readonly asCall: {
      readonly dest: MultiAddress;
      readonly value: Compact<u128>;
      readonly gasLimit: SpWeightsWeightV2Weight;
      readonly storageDepositLimit: Option<Compact<u128>>;
      readonly data: Bytes;
    } & Struct;
    readonly isInstantiateWithCode: boolean;
    readonly asInstantiateWithCode: {
      readonly value: Compact<u128>;
      readonly gasLimit: SpWeightsWeightV2Weight;
      readonly storageDepositLimit: Option<Compact<u128>>;
      readonly code: Bytes;
      readonly data: Bytes;
      readonly salt: Bytes;
    } & Struct;
    readonly isInstantiate: boolean;
    readonly asInstantiate: {
      readonly value: Compact<u128>;
      readonly gasLimit: SpWeightsWeightV2Weight;
      readonly storageDepositLimit: Option<Compact<u128>>;
      readonly codeHash: H256;
      readonly data: Bytes;
      readonly salt: Bytes;
    } & Struct;
    readonly isMigrate: boolean;
    readonly asMigrate: {
      readonly weightLimit: SpWeightsWeightV2Weight;
    } & Struct;
    readonly type: 'CallOldWeight' | 'InstantiateWithCodeOldWeight' | 'InstantiateOldWeight' | 'UploadCode' | 'RemoveCode' | 'SetCode' | 'Call' | 'InstantiateWithCode' | 'Instantiate' | 'Migrate';
  }

  /** @name PalletContractsWasmDeterminism (506) */
  interface PalletContractsWasmDeterminism extends Enum {
    readonly isEnforced: boolean;
    readonly isRelaxed: boolean;
    readonly type: 'Enforced' | 'Relaxed';
  }

  /** @name PolymeshContractsCall (507) */
  interface PolymeshContractsCall extends Enum {
    readonly isInstantiateWithCodePerms: boolean;
    readonly asInstantiateWithCodePerms: {
      readonly endowment: u128;
      readonly gasLimit: SpWeightsWeightV2Weight;
      readonly storageDepositLimit: Option<u128>;
      readonly code: Bytes;
      readonly data: Bytes;
      readonly salt: Bytes;
      readonly perms: PolymeshPrimitivesSecondaryKeyPermissions;
    } & Struct;
    readonly isInstantiateWithHashPerms: boolean;
    readonly asInstantiateWithHashPerms: {
      readonly endowment: u128;
      readonly gasLimit: SpWeightsWeightV2Weight;
      readonly storageDepositLimit: Option<u128>;
      readonly codeHash: H256;
      readonly data: Bytes;
      readonly salt: Bytes;
      readonly perms: PolymeshPrimitivesSecondaryKeyPermissions;
    } & Struct;
    readonly isUpdateCallRuntimeWhitelist: boolean;
    readonly asUpdateCallRuntimeWhitelist: {
      readonly updates: Vec<ITuple<[PolymeshContractsChainExtensionExtrinsicId, bool]>>;
    } & Struct;
    readonly isInstantiateWithCodeAsPrimaryKey: boolean;
    readonly asInstantiateWithCodeAsPrimaryKey: {
      readonly endowment: u128;
      readonly gasLimit: SpWeightsWeightV2Weight;
      readonly storageDepositLimit: Option<u128>;
      readonly code: Bytes;
      readonly data: Bytes;
      readonly salt: Bytes;
    } & Struct;
    readonly isInstantiateWithHashAsPrimaryKey: boolean;
    readonly asInstantiateWithHashAsPrimaryKey: {
      readonly endowment: u128;
      readonly gasLimit: SpWeightsWeightV2Weight;
      readonly storageDepositLimit: Option<u128>;
      readonly codeHash: H256;
      readonly data: Bytes;
      readonly salt: Bytes;
    } & Struct;
    readonly isUpgradeApi: boolean;
    readonly asUpgradeApi: {
      readonly api: PolymeshContractsApi;
      readonly nextUpgrade: PolymeshContractsNextUpgrade;
    } & Struct;
    readonly type: 'InstantiateWithCodePerms' | 'InstantiateWithHashPerms' | 'UpdateCallRuntimeWhitelist' | 'InstantiateWithCodeAsPrimaryKey' | 'InstantiateWithHashAsPrimaryKey' | 'UpgradeApi';
  }

  /** @name PolymeshContractsNextUpgrade (510) */
  interface PolymeshContractsNextUpgrade extends Struct {
    readonly chainVersion: PolymeshContractsChainVersion;
    readonly apiHash: PolymeshContractsApiCodeHash;
  }

  /** @name PolymeshContractsApiCodeHash (511) */
  interface PolymeshContractsApiCodeHash extends Struct {
    readonly hash_: H256;
  }

  /** @name PalletPreimageCall (512) */
  interface PalletPreimageCall extends Enum {
    readonly isNotePreimage: boolean;
    readonly asNotePreimage: {
      readonly bytes: Bytes;
    } & Struct;
    readonly isUnnotePreimage: boolean;
    readonly asUnnotePreimage: {
      readonly hash_: H256;
    } & Struct;
    readonly isRequestPreimage: boolean;
    readonly asRequestPreimage: {
      readonly hash_: H256;
    } & Struct;
    readonly isUnrequestPreimage: boolean;
    readonly asUnrequestPreimage: {
      readonly hash_: H256;
    } & Struct;
    readonly isEnsureUpdated: boolean;
    readonly asEnsureUpdated: {
      readonly hashes: Vec<H256>;
    } & Struct;
    readonly type: 'NotePreimage' | 'UnnotePreimage' | 'RequestPreimage' | 'UnrequestPreimage' | 'EnsureUpdated';
  }

  /** @name PalletNftCall (513) */
  interface PalletNftCall extends Enum {
    readonly isCreateNftCollection: boolean;
    readonly asCreateNftCollection: {
      readonly assetId: Option<PolymeshPrimitivesAssetAssetId>;
      readonly nftType: Option<PolymeshPrimitivesAssetNonFungibleType>;
      readonly collectionKeys: PolymeshPrimitivesNftNftCollectionKeys;
    } & Struct;
    readonly isIssueNft: boolean;
    readonly asIssueNft: {
      readonly assetId: PolymeshPrimitivesAssetAssetId;
      readonly nftMetadataAttributes: Vec<PolymeshPrimitivesNftNftMetadataAttribute>;
      readonly holdingsKind: PolymeshPrimitivesAssetAssetHolderKind;
    } & Struct;
    readonly isRedeemNft: boolean;
    readonly asRedeemNft: {
      readonly assetId: PolymeshPrimitivesAssetAssetId;
      readonly nftId: u64;
      readonly holdingsKind: PolymeshPrimitivesAssetAssetHolderKind;
      readonly numberOfKeys: Option<u8>;
    } & Struct;
    readonly isControllerTransfer: boolean;
    readonly asControllerTransfer: {
      readonly nfts: PolymeshPrimitivesNftNfTs;
      readonly source: PolymeshPrimitivesAssetAssetHolder;
      readonly callersHoldingsKind: PolymeshPrimitivesAssetAssetHolderKind;
    } & Struct;
    readonly type: 'CreateNftCollection' | 'IssueNft' | 'RedeemNft' | 'ControllerTransfer';
  }

  /** @name PolymeshPrimitivesNftNftCollectionKeys (516) */
  interface PolymeshPrimitivesNftNftCollectionKeys extends Vec<PolymeshPrimitivesAssetMetadataAssetMetadataKey> {}

  /** @name PolymeshPrimitivesNftNftMetadataAttribute (519) */
  interface PolymeshPrimitivesNftNftMetadataAttribute extends Struct {
    readonly key: PolymeshPrimitivesAssetMetadataAssetMetadataKey;
    readonly value: Bytes;
  }

  /** @name PalletElectionProviderMultiPhaseCall (521) */
  interface PalletElectionProviderMultiPhaseCall extends Enum {
    readonly isSubmitUnsigned: boolean;
    readonly asSubmitUnsigned: {
      readonly rawSolution: PalletElectionProviderMultiPhaseRawSolution;
      readonly witness: PalletElectionProviderMultiPhaseSolutionOrSnapshotSize;
    } & Struct;
    readonly isSetMinimumUntrustedScore: boolean;
    readonly asSetMinimumUntrustedScore: {
      readonly maybeNextScore: Option<SpNposElectionsElectionScore>;
    } & Struct;
    readonly isSetEmergencyElectionResult: boolean;
    readonly asSetEmergencyElectionResult: {
      readonly supports: Vec<ITuple<[AccountId32, SpNposElectionsSupport]>>;
    } & Struct;
    readonly isSubmit: boolean;
    readonly asSubmit: {
      readonly rawSolution: PalletElectionProviderMultiPhaseRawSolution;
    } & Struct;
    readonly isGovernanceFallback: boolean;
    readonly type: 'SubmitUnsigned' | 'SetMinimumUntrustedScore' | 'SetEmergencyElectionResult' | 'Submit' | 'GovernanceFallback';
  }

  /** @name PalletElectionProviderMultiPhaseRawSolution (522) */
  interface PalletElectionProviderMultiPhaseRawSolution extends Struct {
    readonly solution: PolymeshRuntimeCommonNposSolution16;
    readonly score: SpNposElectionsElectionScore;
    readonly round: u32;
  }

  /** @name PolymeshRuntimeCommonNposSolution16 (523) */
  interface PolymeshRuntimeCommonNposSolution16 extends Struct {
    readonly votes1: Vec<ITuple<[Compact<u32>, Compact<u16>]>>;
    readonly votes2: Vec<ITuple<[Compact<u32>, ITuple<[Compact<u16>, Compact<PerU16>]>, Compact<u16>]>>;
    readonly votes3: Vec<ITuple<[Compact<u32>, Vec<ITuple<[Compact<u16>, Compact<PerU16>]>>, Compact<u16>]>>;
    readonly votes4: Vec<ITuple<[Compact<u32>, Vec<ITuple<[Compact<u16>, Compact<PerU16>]>>, Compact<u16>]>>;
    readonly votes5: Vec<ITuple<[Compact<u32>, Vec<ITuple<[Compact<u16>, Compact<PerU16>]>>, Compact<u16>]>>;
    readonly votes6: Vec<ITuple<[Compact<u32>, Vec<ITuple<[Compact<u16>, Compact<PerU16>]>>, Compact<u16>]>>;
    readonly votes7: Vec<ITuple<[Compact<u32>, Vec<ITuple<[Compact<u16>, Compact<PerU16>]>>, Compact<u16>]>>;
    readonly votes8: Vec<ITuple<[Compact<u32>, Vec<ITuple<[Compact<u16>, Compact<PerU16>]>>, Compact<u16>]>>;
    readonly votes9: Vec<ITuple<[Compact<u32>, Vec<ITuple<[Compact<u16>, Compact<PerU16>]>>, Compact<u16>]>>;
    readonly votes10: Vec<ITuple<[Compact<u32>, Vec<ITuple<[Compact<u16>, Compact<PerU16>]>>, Compact<u16>]>>;
    readonly votes11: Vec<ITuple<[Compact<u32>, Vec<ITuple<[Compact<u16>, Compact<PerU16>]>>, Compact<u16>]>>;
    readonly votes12: Vec<ITuple<[Compact<u32>, Vec<ITuple<[Compact<u16>, Compact<PerU16>]>>, Compact<u16>]>>;
    readonly votes13: Vec<ITuple<[Compact<u32>, Vec<ITuple<[Compact<u16>, Compact<PerU16>]>>, Compact<u16>]>>;
    readonly votes14: Vec<ITuple<[Compact<u32>, Vec<ITuple<[Compact<u16>, Compact<PerU16>]>>, Compact<u16>]>>;
    readonly votes15: Vec<ITuple<[Compact<u32>, Vec<ITuple<[Compact<u16>, Compact<PerU16>]>>, Compact<u16>]>>;
    readonly votes16: Vec<ITuple<[Compact<u32>, Vec<ITuple<[Compact<u16>, Compact<PerU16>]>>, Compact<u16>]>>;
  }

  /** @name PalletElectionProviderMultiPhaseSolutionOrSnapshotSize (574) */
  interface PalletElectionProviderMultiPhaseSolutionOrSnapshotSize extends Struct {
    readonly voters: Compact<u32>;
    readonly targets: Compact<u32>;
  }

  /** @name SpNposElectionsSupport (578) */
  interface SpNposElectionsSupport extends Struct {
    readonly total: u128;
    readonly voters: Vec<ITuple<[AccountId32, u128]>>;
  }

  /** @name PolymeshTransactionPaymentCall (581) */
  interface PolymeshTransactionPaymentCall extends Enum {
    readonly isSetDisableFees: boolean;
    readonly asSetDisableFees: {
      readonly value: bool;
    } & Struct;
    readonly type: 'SetDisableFees';
  }

  /** @name PalletBeefyCall (582) */
  interface PalletBeefyCall extends Enum {
    readonly isReportDoubleVoting: boolean;
    readonly asReportDoubleVoting: {
      readonly equivocationProof: SpConsensusBeefyDoubleVotingProof;
      readonly keyOwnerProof: SpSessionMembershipProof;
    } & Struct;
    readonly isReportDoubleVotingUnsigned: boolean;
    readonly asReportDoubleVotingUnsigned: {
      readonly equivocationProof: SpConsensusBeefyDoubleVotingProof;
      readonly keyOwnerProof: SpSessionMembershipProof;
    } & Struct;
    readonly isSetNewGenesis: boolean;
    readonly asSetNewGenesis: {
      readonly delayInBlocks: u32;
    } & Struct;
    readonly isReportForkVoting: boolean;
    readonly asReportForkVoting: {
      readonly equivocationProof: SpConsensusBeefyForkVotingProof;
      readonly keyOwnerProof: SpSessionMembershipProof;
    } & Struct;
    readonly isReportForkVotingUnsigned: boolean;
    readonly asReportForkVotingUnsigned: {
      readonly equivocationProof: SpConsensusBeefyForkVotingProof;
      readonly keyOwnerProof: SpSessionMembershipProof;
    } & Struct;
    readonly isReportFutureBlockVoting: boolean;
    readonly asReportFutureBlockVoting: {
      readonly equivocationProof: SpConsensusBeefyFutureBlockVotingProof;
      readonly keyOwnerProof: SpSessionMembershipProof;
    } & Struct;
    readonly isReportFutureBlockVotingUnsigned: boolean;
    readonly asReportFutureBlockVotingUnsigned: {
      readonly equivocationProof: SpConsensusBeefyFutureBlockVotingProof;
      readonly keyOwnerProof: SpSessionMembershipProof;
    } & Struct;
    readonly type: 'ReportDoubleVoting' | 'ReportDoubleVotingUnsigned' | 'SetNewGenesis' | 'ReportForkVoting' | 'ReportForkVotingUnsigned' | 'ReportFutureBlockVoting' | 'ReportFutureBlockVotingUnsigned';
  }

  /** @name SpConsensusBeefyDoubleVotingProof (583) */
  interface SpConsensusBeefyDoubleVotingProof extends Struct {
    readonly first: SpConsensusBeefyVoteMessage;
    readonly second: SpConsensusBeefyVoteMessage;
  }

  /** @name SpConsensusBeefyEcdsaCryptoSignature (584) */
  interface SpConsensusBeefyEcdsaCryptoSignature extends U8aFixed {}

  /** @name SpConsensusBeefyVoteMessage (585) */
  interface SpConsensusBeefyVoteMessage extends Struct {
    readonly commitment: SpConsensusBeefyCommitment;
    readonly id: SpConsensusBeefyEcdsaCryptoPublic;
    readonly signature: SpConsensusBeefyEcdsaCryptoSignature;
  }

  /** @name SpConsensusBeefyCommitment (586) */
  interface SpConsensusBeefyCommitment extends Struct {
    readonly payload: SpConsensusBeefyPayload;
    readonly blockNumber: u32;
    readonly validatorSetId: u64;
  }

  /** @name SpConsensusBeefyPayload (587) */
  interface SpConsensusBeefyPayload extends Vec<ITuple<[U8aFixed, Bytes]>> {}

  /** @name SpConsensusBeefyForkVotingProof (591) */
  interface SpConsensusBeefyForkVotingProof extends Struct {
    readonly vote: SpConsensusBeefyVoteMessage;
    readonly ancestryProof: SpMmrPrimitivesAncestryProof;
    readonly header: SpRuntimeHeader;
  }

  /** @name SpMmrPrimitivesAncestryProof (592) */
  interface SpMmrPrimitivesAncestryProof extends Struct {
    readonly prevPeaks: Vec<H256>;
    readonly prevLeafCount: u64;
    readonly leafCount: u64;
    readonly items: Vec<ITuple<[u64, H256]>>;
  }

  /** @name SpConsensusBeefyFutureBlockVotingProof (595) */
  interface SpConsensusBeefyFutureBlockVotingProof extends Struct {
    readonly vote: SpConsensusBeefyVoteMessage;
  }

  /** @name PalletCommitteePolymeshVotes (596) */
  interface PalletCommitteePolymeshVotes extends Struct {
    readonly index: u32;
    readonly ayes: Vec<PolymeshPrimitivesIdentityId>;
    readonly nays: Vec<PolymeshPrimitivesIdentityId>;
    readonly expiry: PolymeshPrimitivesMaybeBlock;
  }

  /** @name PalletCommitteeError (597) */
  interface PalletCommitteeError extends Enum {
    readonly isDuplicateVote: boolean;
    readonly isNotAMember: boolean;
    readonly isNoSuchProposal: boolean;
    readonly isProposalExpired: boolean;
    readonly isDuplicateProposal: boolean;
    readonly isMismatchedVotingIndex: boolean;
    readonly isInvalidProportion: boolean;
    readonly isFirstVoteReject: boolean;
    readonly isProposalsLimitReached: boolean;
    readonly type: 'DuplicateVote' | 'NotAMember' | 'NoSuchProposal' | 'ProposalExpired' | 'DuplicateProposal' | 'MismatchedVotingIndex' | 'InvalidProportion' | 'FirstVoteReject' | 'ProposalsLimitReached';
  }

  /** @name PolymeshPrimitivesMultisigProposalVoteCount (606) */
  interface PolymeshPrimitivesMultisigProposalVoteCount extends Struct {
    readonly approvals: u64;
    readonly rejections: u64;
  }

  /** @name PolymeshPrimitivesMultisigProposalState (607) */
  interface PolymeshPrimitivesMultisigProposalState extends Enum {
    readonly isActive: boolean;
    readonly asActive: {
      readonly until: Option<u64>;
    } & Struct;
    readonly isExecutionSuccessful: boolean;
    readonly isExecutionFailed: boolean;
    readonly isRejected: boolean;
    readonly type: 'Active' | 'ExecutionSuccessful' | 'ExecutionFailed' | 'Rejected';
  }

  /** @name PalletMultisigError (609) */
  interface PalletMultisigError extends Enum {
    readonly isProposalMissing: boolean;
    readonly isDecodingError: boolean;
    readonly isRequiredSignersIsZero: boolean;
    readonly isNotASigner: boolean;
    readonly isNoSuchMultisig: boolean;
    readonly isNotEnoughSigners: boolean;
    readonly isNonceOverflow: boolean;
    readonly isAlreadyVoted: boolean;
    readonly isAlreadyASigner: boolean;
    readonly isIdentityNotAdmin: boolean;
    readonly isIdentityNotPayer: boolean;
    readonly isSignerAlreadyLinkedToMultisig: boolean;
    readonly isSignerAlreadyLinkedToIdentity: boolean;
    readonly isNestingNotAllowed: boolean;
    readonly isProposalAlreadyRejected: boolean;
    readonly isProposalExpired: boolean;
    readonly isProposalAlreadyExecuted: boolean;
    readonly isMaxWeightTooLow: boolean;
    readonly isMultisigMissingIdentity: boolean;
    readonly isTooManySigners: boolean;
    readonly isNoPayingDid: boolean;
    readonly isInvalidExpiryDate: boolean;
    readonly isInvalidatedProposal: boolean;
    readonly isAdminNotFound: boolean;
    readonly isBadAuthorizationType: boolean;
    readonly type: 'ProposalMissing' | 'DecodingError' | 'RequiredSignersIsZero' | 'NotASigner' | 'NoSuchMultisig' | 'NotEnoughSigners' | 'NonceOverflow' | 'AlreadyVoted' | 'AlreadyASigner' | 'IdentityNotAdmin' | 'IdentityNotPayer' | 'SignerAlreadyLinkedToMultisig' | 'SignerAlreadyLinkedToIdentity' | 'NestingNotAllowed' | 'ProposalAlreadyRejected' | 'ProposalExpired' | 'ProposalAlreadyExecuted' | 'MaxWeightTooLow' | 'MultisigMissingIdentity' | 'TooManySigners' | 'NoPayingDid' | 'InvalidExpiryDate' | 'InvalidatedProposal' | 'AdminNotFound' | 'BadAuthorizationType';
  }

  /** @name PalletValidatorsPermissionedIdentityPrefs (610) */
  interface PalletValidatorsPermissionedIdentityPrefs extends Struct {
    readonly intendedCount: u32;
    readonly runningCount: u32;
  }

  /** @name PalletValidatorsError (611) */
  interface PalletValidatorsError extends Enum {
    readonly isStashIdentityDoesNotExist: boolean;
    readonly isStashIdentityNotPermissioned: boolean;
    readonly isIdentityIsAlreadyPermissioned: boolean;
    readonly isIdentityIsInactive: boolean;
    readonly isIntendedCountIsExceedingConsensusLimit: boolean;
    readonly isIdentityNotFound: boolean;
    readonly isValidatorNotFound: boolean;
    readonly isCommissionTooHigh: boolean;
    readonly isCommissionUnchanged: boolean;
    readonly type: 'StashIdentityDoesNotExist' | 'StashIdentityNotPermissioned' | 'IdentityIsAlreadyPermissioned' | 'IdentityIsInactive' | 'IntendedCountIsExceedingConsensusLimit' | 'IdentityNotFound' | 'ValidatorNotFound' | 'CommissionTooHigh' | 'CommissionUnchanged';
  }

  /** @name PalletStakingStakingLedger (612) */
  interface PalletStakingStakingLedger extends Struct {
    readonly stash: AccountId32;
    readonly total: Compact<u128>;
    readonly active: Compact<u128>;
    readonly unlocking: Vec<PalletStakingUnlockChunk>;
    readonly legacyClaimedRewards: Vec<u32>;
  }

  /** @name PalletStakingNominations (614) */
  interface PalletStakingNominations extends Struct {
    readonly targets: Vec<AccountId32>;
    readonly submittedIn: u32;
    readonly suppressed: bool;
  }

  /** @name PalletStakingActiveEraInfo (616) */
  interface PalletStakingActiveEraInfo extends Struct {
    readonly index: u32;
    readonly start: Option<u64>;
  }

  /** @name SpStakingPagedExposureMetadata (618) */
  interface SpStakingPagedExposureMetadata extends Struct {
    readonly total: Compact<u128>;
    readonly own: Compact<u128>;
    readonly nominatorCount: u32;
    readonly pageCount: u32;
  }

  /** @name SpStakingExposurePage (620) */
  interface SpStakingExposurePage extends Struct {
    readonly pageTotal: Compact<u128>;
    readonly others: Vec<SpStakingIndividualExposure>;
  }

  /** @name PalletStakingEraRewardPoints (621) */
  interface PalletStakingEraRewardPoints extends Struct {
    readonly total: u32;
    readonly individual: BTreeMap<AccountId32, u32>;
  }

  /** @name PalletStakingUnappliedSlash (626) */
  interface PalletStakingUnappliedSlash extends Struct {
    readonly validator: AccountId32;
    readonly own: u128;
    readonly others: Vec<ITuple<[AccountId32, u128]>>;
    readonly reporters: Vec<AccountId32>;
    readonly payout: u128;
  }

  /** @name PalletStakingSlashingSlashingSpans (628) */
  interface PalletStakingSlashingSlashingSpans extends Struct {
    readonly spanIndex: u32;
    readonly lastStart: u32;
    readonly lastNonzeroSlash: u32;
    readonly prior: Vec<u32>;
  }

  /** @name PalletStakingSlashingSpanRecord (629) */
  interface PalletStakingSlashingSpanRecord extends Struct {
    readonly slashed: u128;
    readonly paidOut: u128;
  }

  /** @name PalletStakingPalletError (630) */
  interface PalletStakingPalletError extends Enum {
    readonly isNotController: boolean;
    readonly isNotStash: boolean;
    readonly isAlreadyBonded: boolean;
    readonly isAlreadyPaired: boolean;
    readonly isEmptyTargets: boolean;
    readonly isDuplicateIndex: boolean;
    readonly isInvalidSlashIndex: boolean;
    readonly isInsufficientBond: boolean;
    readonly isNoMoreChunks: boolean;
    readonly isNoUnlockChunk: boolean;
    readonly isFundedTarget: boolean;
    readonly isInvalidEraToReward: boolean;
    readonly isInvalidNumberOfNominations: boolean;
    readonly isNotSortedAndUnique: boolean;
    readonly isAlreadyClaimed: boolean;
    readonly isInvalidPage: boolean;
    readonly isIncorrectHistoryDepth: boolean;
    readonly isIncorrectSlashingSpans: boolean;
    readonly isBadState: boolean;
    readonly isTooManyTargets: boolean;
    readonly isBadTarget: boolean;
    readonly isCannotChillOther: boolean;
    readonly isTooManyNominators: boolean;
    readonly isTooManyValidators: boolean;
    readonly isCommissionTooLow: boolean;
    readonly isBoundNotMet: boolean;
    readonly isControllerDeprecated: boolean;
    readonly isCannotRestoreLedger: boolean;
    readonly isRewardDestinationRestricted: boolean;
    readonly isNotEnoughFunds: boolean;
    readonly isVirtualStakerNotAllowed: boolean;
    readonly isCannotReapStash: boolean;
    readonly isAlreadyMigrated: boolean;
    readonly isRestricted: boolean;
    readonly type: 'NotController' | 'NotStash' | 'AlreadyBonded' | 'AlreadyPaired' | 'EmptyTargets' | 'DuplicateIndex' | 'InvalidSlashIndex' | 'InsufficientBond' | 'NoMoreChunks' | 'NoUnlockChunk' | 'FundedTarget' | 'InvalidEraToReward' | 'InvalidNumberOfNominations' | 'NotSortedAndUnique' | 'AlreadyClaimed' | 'InvalidPage' | 'IncorrectHistoryDepth' | 'IncorrectSlashingSpans' | 'BadState' | 'TooManyTargets' | 'BadTarget' | 'CannotChillOther' | 'TooManyNominators' | 'TooManyValidators' | 'CommissionTooLow' | 'BoundNotMet' | 'ControllerDeprecated' | 'CannotRestoreLedger' | 'RewardDestinationRestricted' | 'NotEnoughFunds' | 'VirtualStakerNotAllowed' | 'CannotReapStash' | 'AlreadyMigrated' | 'Restricted';
  }

  /** @name SpStakingOffenceOffenceDetails (631) */
  interface SpStakingOffenceOffenceDetails extends Struct {
    readonly offender: ITuple<[AccountId32, SpStakingExposure]>;
    readonly reporters: Vec<AccountId32>;
  }

  /** @name SpCoreCryptoKeyTypeId (639) */
  interface SpCoreCryptoKeyTypeId extends U8aFixed {}

  /** @name PalletSessionError (640) */
  interface PalletSessionError extends Enum {
    readonly isInvalidProof: boolean;
    readonly isNoAssociatedValidatorId: boolean;
    readonly isDuplicatedKey: boolean;
    readonly isNoKeys: boolean;
    readonly isNoAccount: boolean;
    readonly type: 'InvalidProof' | 'NoAssociatedValidatorId' | 'DuplicatedKey' | 'NoKeys' | 'NoAccount';
  }

  /** @name PalletGrandpaStoredState (643) */
  interface PalletGrandpaStoredState extends Enum {
    readonly isLive: boolean;
    readonly isPendingPause: boolean;
    readonly asPendingPause: {
      readonly scheduledAt: u32;
      readonly delay: u32;
    } & Struct;
    readonly isPaused: boolean;
    readonly isPendingResume: boolean;
    readonly asPendingResume: {
      readonly scheduledAt: u32;
      readonly delay: u32;
    } & Struct;
    readonly type: 'Live' | 'PendingPause' | 'Paused' | 'PendingResume';
  }

  /** @name PalletGrandpaStoredPendingChange (644) */
  interface PalletGrandpaStoredPendingChange extends Struct {
    readonly scheduledAt: u32;
    readonly delay: u32;
    readonly nextAuthorities: Vec<ITuple<[SpConsensusGrandpaAppPublic, u64]>>;
    readonly forced: Option<u32>;
  }

  /** @name PalletGrandpaError (646) */
  interface PalletGrandpaError extends Enum {
    readonly isPauseFailed: boolean;
    readonly isResumeFailed: boolean;
    readonly isChangePending: boolean;
    readonly isTooSoon: boolean;
    readonly isInvalidKeyOwnershipProof: boolean;
    readonly isInvalidEquivocationProof: boolean;
    readonly isDuplicateOffenceReport: boolean;
    readonly type: 'PauseFailed' | 'ResumeFailed' | 'ChangePending' | 'TooSoon' | 'InvalidKeyOwnershipProof' | 'InvalidEquivocationProof' | 'DuplicateOffenceReport';
  }

  /** @name PalletImOnlineError (650) */
  interface PalletImOnlineError extends Enum {
    readonly isInvalidKey: boolean;
    readonly isDuplicatedHeartbeat: boolean;
    readonly type: 'InvalidKey' | 'DuplicatedHeartbeat';
  }

  /** @name PalletSudoError (651) */
  interface PalletSudoError extends Enum {
    readonly isRequireSudo: boolean;
    readonly type: 'RequireSudo';
  }

  /** @name PalletAssetTickerRegistration (652) */
  interface PalletAssetTickerRegistration extends Struct {
    readonly owner: PolymeshPrimitivesIdentityId;
    readonly expiry: Option<u64>;
  }

  /** @name PalletAssetTickerRegistrationConfig (653) */
  interface PalletAssetTickerRegistrationConfig extends Struct {
    readonly maxTickerLength: u8;
    readonly registrationLength: Option<u64>;
  }

  /** @name PalletAssetAssetDetails (654) */
  interface PalletAssetAssetDetails extends Struct {
    readonly totalSupply: u128;
    readonly ownerDid: PolymeshPrimitivesIdentityId;
    readonly divisible: bool;
    readonly assetType: PolymeshPrimitivesAssetAssetType;
  }

  /** @name PalletAssetError (665) */
  interface PalletAssetError extends Enum {
    readonly isUnauthorized: boolean;
    readonly isAssetAlreadyCreated: boolean;
    readonly isTickerTooLong: boolean;
    readonly isTickerNotAlphanumeric: boolean;
    readonly isTickerAlreadyRegistered: boolean;
    readonly isTotalSupplyAboveLimit: boolean;
    readonly isNoSuchAsset: boolean;
    readonly isAlreadyFrozen: boolean;
    readonly isNotAnOwner: boolean;
    readonly isBalanceOverflow: boolean;
    readonly isTotalSupplyOverflow: boolean;
    readonly isInvalidGranularity: boolean;
    readonly isNotFrozen: boolean;
    readonly isInvalidTransfer: boolean;
    readonly isInsufficientBalance: boolean;
    readonly isAssetAlreadyDivisible: boolean;
    readonly isInvalidEthereumSignature: boolean;
    readonly isTickerRegistrationExpired: boolean;
    readonly isSenderSameAsReceiver: boolean;
    readonly isNoSuchDoc: boolean;
    readonly isMaxLengthOfAssetNameExceeded: boolean;
    readonly isFundingRoundNameMaxLengthExceeded: boolean;
    readonly isInvalidAssetIdentifier: boolean;
    readonly isInvestorUniquenessClaimNotAllowed: boolean;
    readonly isInvalidCustomAssetTypeId: boolean;
    readonly isAssetMetadataNameMaxLengthExceeded: boolean;
    readonly isAssetMetadataValueMaxLengthExceeded: boolean;
    readonly isAssetMetadataTypeDefMaxLengthExceeded: boolean;
    readonly isAssetMetadataKeyIsMissing: boolean;
    readonly isAssetMetadataValueIsLocked: boolean;
    readonly isAssetMetadataLocalKeyAlreadyExists: boolean;
    readonly isAssetMetadataGlobalKeyAlreadyExists: boolean;
    readonly isTickerFirstByteNotValid: boolean;
    readonly isUnexpectedNonFungibleToken: boolean;
    readonly isIncompatibleAssetTypeUpdate: boolean;
    readonly isAssetMetadataKeyBelongsToNFTCollection: boolean;
    readonly isAssetMetadataValueIsEmpty: boolean;
    readonly isNumberOfAssetMediatorsExceeded: boolean;
    readonly isInvalidTickerCharacter: boolean;
    readonly isInvalidTransferFrozenAsset: boolean;
    readonly isInvalidTransferComplianceFailure: boolean;
    readonly isInvalidTransferInvalidReceiverDID: boolean;
    readonly isTickerRegistrationNotFound: boolean;
    readonly isTickerIsAlreadyLinkedToAnAsset: boolean;
    readonly isAssetIdGenerationError: boolean;
    readonly isTickerNotRegisteredToCaller: boolean;
    readonly isAssetIsAlreadyLinkedToATicker: boolean;
    readonly isTickerIsNotLinkedToTheAsset: boolean;
    readonly isBadAuthorizationType: boolean;
    readonly isUnauthorizedHolderKey: boolean;
    readonly isKeyNotFoundForDid: boolean;
    readonly isInsufficientTokensLocked: boolean;
    readonly type: 'Unauthorized' | 'AssetAlreadyCreated' | 'TickerTooLong' | 'TickerNotAlphanumeric' | 'TickerAlreadyRegistered' | 'TotalSupplyAboveLimit' | 'NoSuchAsset' | 'AlreadyFrozen' | 'NotAnOwner' | 'BalanceOverflow' | 'TotalSupplyOverflow' | 'InvalidGranularity' | 'NotFrozen' | 'InvalidTransfer' | 'InsufficientBalance' | 'AssetAlreadyDivisible' | 'InvalidEthereumSignature' | 'TickerRegistrationExpired' | 'SenderSameAsReceiver' | 'NoSuchDoc' | 'MaxLengthOfAssetNameExceeded' | 'FundingRoundNameMaxLengthExceeded' | 'InvalidAssetIdentifier' | 'InvestorUniquenessClaimNotAllowed' | 'InvalidCustomAssetTypeId' | 'AssetMetadataNameMaxLengthExceeded' | 'AssetMetadataValueMaxLengthExceeded' | 'AssetMetadataTypeDefMaxLengthExceeded' | 'AssetMetadataKeyIsMissing' | 'AssetMetadataValueIsLocked' | 'AssetMetadataLocalKeyAlreadyExists' | 'AssetMetadataGlobalKeyAlreadyExists' | 'TickerFirstByteNotValid' | 'UnexpectedNonFungibleToken' | 'IncompatibleAssetTypeUpdate' | 'AssetMetadataKeyBelongsToNFTCollection' | 'AssetMetadataValueIsEmpty' | 'NumberOfAssetMediatorsExceeded' | 'InvalidTickerCharacter' | 'InvalidTransferFrozenAsset' | 'InvalidTransferComplianceFailure' | 'InvalidTransferInvalidReceiverDID' | 'TickerRegistrationNotFound' | 'TickerIsAlreadyLinkedToAnAsset' | 'AssetIdGenerationError' | 'TickerNotRegisteredToCaller' | 'AssetIsAlreadyLinkedToATicker' | 'TickerIsNotLinkedToTheAsset' | 'BadAuthorizationType' | 'UnauthorizedHolderKey' | 'KeyNotFoundForDid' | 'InsufficientTokensLocked';
  }

  /** @name PalletCorporateActionsDistributionPalletError (668) */
  interface PalletCorporateActionsDistributionPalletError extends Enum {
    readonly isCaNotBenefit: boolean;
    readonly isAlreadyExists: boolean;
    readonly isExpiryBeforePayment: boolean;
    readonly isHolderAlreadyPaid: boolean;
    readonly isNoSuchDistribution: boolean;
    readonly isCannotClaimBeforeStart: boolean;
    readonly isCannotClaimAfterExpiry: boolean;
    readonly isBalancePerShareProductOverflowed: boolean;
    readonly isNotDistributionCreator: boolean;
    readonly isAlreadyReclaimed: boolean;
    readonly isNotExpired: boolean;
    readonly isDistributionStarted: boolean;
    readonly isInsufficientRemainingAmount: boolean;
    readonly isDistributionAmountIsZero: boolean;
    readonly isDistributionPerShareIsZero: boolean;
    readonly type: 'CaNotBenefit' | 'AlreadyExists' | 'ExpiryBeforePayment' | 'HolderAlreadyPaid' | 'NoSuchDistribution' | 'CannotClaimBeforeStart' | 'CannotClaimAfterExpiry' | 'BalancePerShareProductOverflowed' | 'NotDistributionCreator' | 'AlreadyReclaimed' | 'NotExpired' | 'DistributionStarted' | 'InsufficientRemainingAmount' | 'DistributionAmountIsZero' | 'DistributionPerShareIsZero';
  }

  /** @name PolymeshPrimitivesCheckpointNextCheckpoints (672) */
  interface PolymeshPrimitivesCheckpointNextCheckpoints extends Struct {
    readonly nextAt: u64;
    readonly totalPending: u64;
    readonly schedules: BTreeMap<u64, u64>;
  }

  /** @name PalletAssetCheckpointPalletError (678) */
  interface PalletAssetCheckpointPalletError extends Enum {
    readonly isNoSuchSchedule: boolean;
    readonly isScheduleNotRemovable: boolean;
    readonly isSchedulesOverMaxComplexity: boolean;
    readonly isScheduleIsEmpty: boolean;
    readonly isScheduleFinished: boolean;
    readonly isScheduleHasExpiredCheckpoints: boolean;
    readonly type: 'NoSuchSchedule' | 'ScheduleNotRemovable' | 'SchedulesOverMaxComplexity' | 'ScheduleIsEmpty' | 'ScheduleFinished' | 'ScheduleHasExpiredCheckpoints';
  }

  /** @name PolymeshPrimitivesComplianceManagerAssetCompliance (679) */
  interface PolymeshPrimitivesComplianceManagerAssetCompliance extends Struct {
    readonly paused: bool;
    readonly requirements: Vec<PolymeshPrimitivesComplianceManagerComplianceRequirement>;
  }

  /** @name PalletComplianceManagerError (681) */
  interface PalletComplianceManagerError extends Enum {
    readonly isUnauthorized: boolean;
    readonly isDidNotExist: boolean;
    readonly isInvalidComplianceRequirementId: boolean;
    readonly isIncorrectOperationOnTrustedIssuer: boolean;
    readonly isDuplicateComplianceRequirements: boolean;
    readonly isComplianceRequirementTooComplex: boolean;
    readonly isWeightLimitExceeded: boolean;
    readonly type: 'Unauthorized' | 'DidNotExist' | 'InvalidComplianceRequirementId' | 'IncorrectOperationOnTrustedIssuer' | 'DuplicateComplianceRequirements' | 'ComplianceRequirementTooComplex' | 'WeightLimitExceeded';
  }

  /** @name PalletCorporateActionsError (684) */
  interface PalletCorporateActionsError extends Enum {
    readonly isDetailsTooLong: boolean;
    readonly isDuplicateDidTax: boolean;
    readonly isTooManyDidTaxes: boolean;
    readonly isTooManyTargetIds: boolean;
    readonly isNoSuchCheckpointId: boolean;
    readonly isNoSuchCA: boolean;
    readonly isNoRecordDate: boolean;
    readonly isRecordDateAfterStart: boolean;
    readonly isDeclDateAfterRecordDate: boolean;
    readonly isDeclDateInFuture: boolean;
    readonly isNotTargetedByCA: boolean;
    readonly type: 'DetailsTooLong' | 'DuplicateDidTax' | 'TooManyDidTaxes' | 'TooManyTargetIds' | 'NoSuchCheckpointId' | 'NoSuchCA' | 'NoRecordDate' | 'RecordDateAfterStart' | 'DeclDateAfterRecordDate' | 'DeclDateInFuture' | 'NotTargetedByCA';
  }

  /** @name PalletCorporateActionsBallotPalletError (688) */
  interface PalletCorporateActionsBallotPalletError extends Enum {
    readonly isCaNotNotice: boolean;
    readonly isAlreadyExists: boolean;
    readonly isNoSuchBallot: boolean;
    readonly isStartAfterEnd: boolean;
    readonly isNowAfterEnd: boolean;
    readonly isNumberOfChoicesOverflow: boolean;
    readonly isVotingAlreadyStarted: boolean;
    readonly isVotingNotStarted: boolean;
    readonly isVotingAlreadyEnded: boolean;
    readonly isWrongVoteCount: boolean;
    readonly isInsufficientVotes: boolean;
    readonly isNoSuchRCVFallback: boolean;
    readonly isRcvSelfCycle: boolean;
    readonly isRcvNotAllowed: boolean;
    readonly type: 'CaNotNotice' | 'AlreadyExists' | 'NoSuchBallot' | 'StartAfterEnd' | 'NowAfterEnd' | 'NumberOfChoicesOverflow' | 'VotingAlreadyStarted' | 'VotingNotStarted' | 'VotingAlreadyEnded' | 'WrongVoteCount' | 'InsufficientVotes' | 'NoSuchRCVFallback' | 'RcvSelfCycle' | 'RcvNotAllowed';
  }

  /** @name PalletPermissionsError (689) */
  interface PalletPermissionsError extends Enum {
    readonly isUnauthorizedCaller: boolean;
    readonly type: 'UnauthorizedCaller';
  }

  /** @name PalletPipsPipsMetadata (690) */
  interface PalletPipsPipsMetadata extends Struct {
    readonly id: u32;
    readonly url: Option<Bytes>;
    readonly description: Option<Bytes>;
    readonly createdAt: u32;
    readonly transactionVersion: u32;
    readonly expiry: PolymeshPrimitivesMaybeBlock;
  }

  /** @name PalletPipsDepositInfo (692) */
  interface PalletPipsDepositInfo extends Struct {
    readonly owner: AccountId32;
    readonly amount: u128;
  }

  /** @name PalletPipsPip (693) */
  interface PalletPipsPip extends Struct {
    readonly id: u32;
    readonly proposal: Call;
    readonly proposer: PalletPipsProposer;
  }

  /** @name PalletPipsVotingResult (694) */
  interface PalletPipsVotingResult extends Struct {
    readonly ayesCount: u32;
    readonly ayesStake: u128;
    readonly naysCount: u32;
    readonly naysStake: u128;
  }

  /** @name PalletPipsVote (695) */
  interface PalletPipsVote extends ITuple<[bool, u128]> {}

  /** @name PalletPipsSnapshotMetadata (696) */
  interface PalletPipsSnapshotMetadata extends Struct {
    readonly createdAt: u32;
    readonly madeBy: AccountId32;
    readonly id: u32;
  }

  /** @name PalletPipsError (698) */
  interface PalletPipsError extends Enum {
    readonly isRescheduleNotByReleaseCoordinator: boolean;
    readonly isNotFromCommunity: boolean;
    readonly isNotByCommittee: boolean;
    readonly isTooManyActivePips: boolean;
    readonly isIncorrectDeposit: boolean;
    readonly isInsufficientDeposit: boolean;
    readonly isNoSuchProposal: boolean;
    readonly isNotACommitteeMember: boolean;
    readonly isInvalidFutureBlockNumber: boolean;
    readonly isNumberOfVotesExceeded: boolean;
    readonly isStakeAmountOfVotesExceeded: boolean;
    readonly isMissingCurrentIdentity: boolean;
    readonly isIncorrectProposalState: boolean;
    readonly isCannotSkipPip: boolean;
    readonly isSnapshotResultTooLarge: boolean;
    readonly isSnapshotIdMismatch: boolean;
    readonly isScheduledProposalDoesntExist: boolean;
    readonly isProposalNotInScheduledState: boolean;
    readonly isInvalidPipId: boolean;
    readonly isInvalidTaskName: boolean;
    readonly type: 'RescheduleNotByReleaseCoordinator' | 'NotFromCommunity' | 'NotByCommittee' | 'TooManyActivePips' | 'IncorrectDeposit' | 'InsufficientDeposit' | 'NoSuchProposal' | 'NotACommitteeMember' | 'InvalidFutureBlockNumber' | 'NumberOfVotesExceeded' | 'StakeAmountOfVotesExceeded' | 'MissingCurrentIdentity' | 'IncorrectProposalState' | 'CannotSkipPip' | 'SnapshotResultTooLarge' | 'SnapshotIdMismatch' | 'ScheduledProposalDoesntExist' | 'ProposalNotInScheduledState' | 'InvalidPipId' | 'InvalidTaskName';
  }

  /** @name PalletPortfolioError (706) */
  interface PalletPortfolioError extends Enum {
    readonly isPortfolioDoesNotExist: boolean;
    readonly isInsufficientPortfolioBalance: boolean;
    readonly isDestinationIsSamePortfolio: boolean;
    readonly isPortfolioNameAlreadyInUse: boolean;
    readonly isSecondaryKeyNotAuthorizedForPortfolio: boolean;
    readonly isUnauthorizedCustodian: boolean;
    readonly isInsufficientTokensLocked: boolean;
    readonly isPortfolioNotEmpty: boolean;
    readonly isDifferentIdentityPortfolios: boolean;
    readonly isNoDuplicateAssetsAllowed: boolean;
    readonly isNftNotFoundInPortfolio: boolean;
    readonly isNftAlreadyLocked: boolean;
    readonly isNftNotLocked: boolean;
    readonly isInvalidTransferNFTNotOwned: boolean;
    readonly isInvalidTransferNFTIsLocked: boolean;
    readonly isEmptyTransfer: boolean;
    readonly isMissingOwnersPermission: boolean;
    readonly isSelfAdditionNotAllowed: boolean;
    readonly isBadAuthorizationType: boolean;
    readonly isDefaultPortfoliosCannotHaveCustodians: boolean;
    readonly isUnauthorizedPortfolioKey: boolean;
    readonly isKeyNotFoundForCaller: boolean;
    readonly isInsufficientBalance: boolean;
    readonly type: 'PortfolioDoesNotExist' | 'InsufficientPortfolioBalance' | 'DestinationIsSamePortfolio' | 'PortfolioNameAlreadyInUse' | 'SecondaryKeyNotAuthorizedForPortfolio' | 'UnauthorizedCustodian' | 'InsufficientTokensLocked' | 'PortfolioNotEmpty' | 'DifferentIdentityPortfolios' | 'NoDuplicateAssetsAllowed' | 'NftNotFoundInPortfolio' | 'NftAlreadyLocked' | 'NftNotLocked' | 'InvalidTransferNFTNotOwned' | 'InvalidTransferNFTIsLocked' | 'EmptyTransfer' | 'MissingOwnersPermission' | 'SelfAdditionNotAllowed' | 'BadAuthorizationType' | 'DefaultPortfoliosCannotHaveCustodians' | 'UnauthorizedPortfolioKey' | 'KeyNotFoundForCaller' | 'InsufficientBalance';
  }

  /** @name PalletProtocolFeeError (707) */
  interface PalletProtocolFeeError extends Enum {
    readonly isInsufficientAccountBalance: boolean;
    readonly isUnHandledImbalances: boolean;
    readonly isInsufficientSubsidyBalance: boolean;
    readonly type: 'InsufficientAccountBalance' | 'UnHandledImbalances' | 'InsufficientSubsidyBalance';
  }

  /** @name PalletSchedulerScheduled (710) */
  interface PalletSchedulerScheduled extends Struct {
    readonly maybeId: Option<U8aFixed>;
    readonly priority: u8;
    readonly call: FrameSupportPreimagesBounded;
    readonly maybePeriodic: Option<ITuple<[u32, u32]>>;
    readonly origin: PolymeshRuntimeDevelopRuntimeOriginCaller;
  }

  /** @name FrameSupportPreimagesBounded (711) */
  interface FrameSupportPreimagesBounded extends Enum {
    readonly isLegacy: boolean;
    readonly asLegacy: {
      readonly hash_: H256;
    } & Struct;
    readonly isInline: boolean;
    readonly asInline: Bytes;
    readonly isLookup: boolean;
    readonly asLookup: {
      readonly hash_: H256;
      readonly len: u32;
    } & Struct;
    readonly type: 'Legacy' | 'Inline' | 'Lookup';
  }

  /** @name SpRuntimeBlakeTwo256 (712) */
  type SpRuntimeBlakeTwo256 = Null;

  /** @name PalletSchedulerRetryConfig (715) */
  interface PalletSchedulerRetryConfig extends Struct {
    readonly totalRetries: u8;
    readonly remaining: u8;
    readonly period: u32;
  }

  /** @name PalletSchedulerError (716) */
  interface PalletSchedulerError extends Enum {
    readonly isFailedToSchedule: boolean;
    readonly isNotFound: boolean;
    readonly isTargetBlockNumberInPast: boolean;
    readonly isRescheduleNoChange: boolean;
    readonly isNamed: boolean;
    readonly type: 'FailedToSchedule' | 'NotFound' | 'TargetBlockNumberInPast' | 'RescheduleNoChange' | 'Named';
  }

  /** @name PolymeshPrimitivesSettlementVenue (717) */
  interface PolymeshPrimitivesSettlementVenue extends Struct {
    readonly creator: PolymeshPrimitivesIdentityId;
    readonly venueType: PolymeshPrimitivesSettlementVenueType;
  }

  /** @name PolymeshPrimitivesSettlementInstruction (721) */
  interface PolymeshPrimitivesSettlementInstruction extends Struct {
    readonly instructionId: u64;
    readonly venueId: Option<u64>;
    readonly settlementType: PolymeshPrimitivesSettlementSettlementType;
    readonly createdAt: Option<u64>;
    readonly tradeDate: Option<u64>;
    readonly valueDate: Option<u64>;
  }

  /** @name PolymeshPrimitivesSettlementLegStatus (723) */
  interface PolymeshPrimitivesSettlementLegStatus extends Enum {
    readonly isPendingTokenLock: boolean;
    readonly isExecutionPending: boolean;
    readonly isExecutionToBeSkipped: boolean;
    readonly asExecutionToBeSkipped: ITuple<[AccountId32, u64]>;
    readonly type: 'PendingTokenLock' | 'ExecutionPending' | 'ExecutionToBeSkipped';
  }

  /** @name PolymeshPrimitivesSettlementAffirmationStatus (725) */
  interface PolymeshPrimitivesSettlementAffirmationStatus extends Enum {
    readonly isUnknown: boolean;
    readonly isPending: boolean;
    readonly isAffirmed: boolean;
    readonly type: 'Unknown' | 'Pending' | 'Affirmed';
  }

  /** @name PolymeshPrimitivesSettlementInstructionStatus (728) */
  interface PolymeshPrimitivesSettlementInstructionStatus extends Enum {
    readonly isUnknown: boolean;
    readonly isPending: boolean;
    readonly isFailed: boolean;
    readonly isSuccess: boolean;
    readonly asSuccess: u32;
    readonly isRejected: boolean;
    readonly asRejected: u32;
    readonly isLockedForExecution: boolean;
    readonly type: 'Unknown' | 'Pending' | 'Failed' | 'Success' | 'Rejected' | 'LockedForExecution';
  }

  /** @name PolymeshPrimitivesSettlementMediatorAffirmationStatus (730) */
  interface PolymeshPrimitivesSettlementMediatorAffirmationStatus extends Enum {
    readonly isUnknown: boolean;
    readonly isPending: boolean;
    readonly isAffirmed: boolean;
    readonly asAffirmed: {
      readonly expiry: Option<u64>;
    } & Struct;
    readonly type: 'Unknown' | 'Pending' | 'Affirmed';
  }

  /** @name PalletSettlementError (731) */
  interface PalletSettlementError extends Enum {
    readonly isInvalidVenue: boolean;
    readonly isUnauthorized: boolean;
    readonly isInstructionNotAffirmed: boolean;
    readonly isUnauthorizedSigner: boolean;
    readonly isReceiptAlreadyClaimed: boolean;
    readonly isUnauthorizedVenue: boolean;
    readonly isInstructionDatesInvalid: boolean;
    readonly isInstructionSettleBlockPassed: boolean;
    readonly isInvalidSignature: boolean;
    readonly isSameSenderReceiver: boolean;
    readonly isSettleOnPastBlock: boolean;
    readonly isUnexpectedAffirmationStatus: boolean;
    readonly isFailedToSchedule: boolean;
    readonly isUnknownInstruction: boolean;
    readonly isSignerAlreadyExists: boolean;
    readonly isSignerDoesNotExist: boolean;
    readonly isZeroAmount: boolean;
    readonly isInstructionSettleBlockNotReached: boolean;
    readonly isCallerIsNotAParty: boolean;
    readonly isMaxNumberOfNFTsExceeded: boolean;
    readonly isNumberOfTransferredNFTsUnderestimated: boolean;
    readonly isReceiptForInvalidLegType: boolean;
    readonly isWeightLimitExceeded: boolean;
    readonly isMaxNumberOfFungibleAssetsExceeded: boolean;
    readonly isMaxNumberOfOffChainAssetsExceeded: boolean;
    readonly isNumberOfFungibleTransfersUnderestimated: boolean;
    readonly isUnexpectedOFFChainAsset: boolean;
    readonly isOffChainAssetCantBeLocked: boolean;
    readonly isNumberOfOffChainTransfersUnderestimated: boolean;
    readonly isLegNotFound: boolean;
    readonly isInputWeightIsLessThanMinimum: boolean;
    readonly isMaxNumberOfReceiptsExceeded: boolean;
    readonly isNotAllAffirmationsHaveBeenReceived: boolean;
    readonly isInvalidInstructionStatusForExecution: boolean;
    readonly isFailedToReleaseLockOrTransferAssets: boolean;
    readonly isDuplicateReceiptUid: boolean;
    readonly isReceiptInstructionIdMissmatch: boolean;
    readonly isMultipleReceiptsForOneLeg: boolean;
    readonly isUnexpectedLegStatus: boolean;
    readonly isNumberOfVenueSignersExceeded: boolean;
    readonly isCallerIsNotAMediator: boolean;
    readonly isInvalidExpiryDate: boolean;
    readonly isMediatorAffirmationExpired: boolean;
    readonly isOffChainAssetsMustHaveAVenue: boolean;
    readonly isUnexpectedSettlementType: boolean;
    readonly isInvalidInstructionStatusForRejection: boolean;
    readonly isLockTimestampNotFound: boolean;
    readonly isExceededMaximumLockingPeriod: boolean;
    readonly isFailedAssetTransferringConditions: boolean;
    readonly isInvalidInstructionStatusForWithdrawal: boolean;
    readonly isReceiverIdentityNotFound: boolean;
    readonly isInvalidAccountId: boolean;
    readonly isInvalidTaskName: boolean;
    readonly isReceiptExpired: boolean;
    readonly type: 'InvalidVenue' | 'Unauthorized' | 'InstructionNotAffirmed' | 'UnauthorizedSigner' | 'ReceiptAlreadyClaimed' | 'UnauthorizedVenue' | 'InstructionDatesInvalid' | 'InstructionSettleBlockPassed' | 'InvalidSignature' | 'SameSenderReceiver' | 'SettleOnPastBlock' | 'UnexpectedAffirmationStatus' | 'FailedToSchedule' | 'UnknownInstruction' | 'SignerAlreadyExists' | 'SignerDoesNotExist' | 'ZeroAmount' | 'InstructionSettleBlockNotReached' | 'CallerIsNotAParty' | 'MaxNumberOfNFTsExceeded' | 'NumberOfTransferredNFTsUnderestimated' | 'ReceiptForInvalidLegType' | 'WeightLimitExceeded' | 'MaxNumberOfFungibleAssetsExceeded' | 'MaxNumberOfOffChainAssetsExceeded' | 'NumberOfFungibleTransfersUnderestimated' | 'UnexpectedOFFChainAsset' | 'OffChainAssetCantBeLocked' | 'NumberOfOffChainTransfersUnderestimated' | 'LegNotFound' | 'InputWeightIsLessThanMinimum' | 'MaxNumberOfReceiptsExceeded' | 'NotAllAffirmationsHaveBeenReceived' | 'InvalidInstructionStatusForExecution' | 'FailedToReleaseLockOrTransferAssets' | 'DuplicateReceiptUid' | 'ReceiptInstructionIdMissmatch' | 'MultipleReceiptsForOneLeg' | 'UnexpectedLegStatus' | 'NumberOfVenueSignersExceeded' | 'CallerIsNotAMediator' | 'InvalidExpiryDate' | 'MediatorAffirmationExpired' | 'OffChainAssetsMustHaveAVenue' | 'UnexpectedSettlementType' | 'InvalidInstructionStatusForRejection' | 'LockTimestampNotFound' | 'ExceededMaximumLockingPeriod' | 'FailedAssetTransferringConditions' | 'InvalidInstructionStatusForWithdrawal' | 'ReceiverIdentityNotFound' | 'InvalidAccountId' | 'InvalidTaskName' | 'ReceiptExpired';
  }

  /** @name PolymeshPrimitivesStatisticsStat1stKey (734) */
  interface PolymeshPrimitivesStatisticsStat1stKey extends Struct {
    readonly assetId: PolymeshPrimitivesAssetAssetId;
    readonly statType: PolymeshPrimitivesStatisticsStatType;
  }

  /** @name PolymeshPrimitivesTransferComplianceAssetTransferCompliance (735) */
  interface PolymeshPrimitivesTransferComplianceAssetTransferCompliance extends Struct {
    readonly paused: bool;
    readonly requirements: BTreeSet<PolymeshPrimitivesTransferComplianceTransferCondition>;
  }

  /** @name PalletStatisticsError (739) */
  interface PalletStatisticsError extends Enum {
    readonly isInvalidTransferStatisticsFailure: boolean;
    readonly isStatTypeMissing: boolean;
    readonly isStatTypeNeededByTransferCondition: boolean;
    readonly isCannotRemoveStatTypeInUse: boolean;
    readonly isStatTypeLimitReached: boolean;
    readonly isTransferConditionLimitReached: boolean;
    readonly isWeightLimitExceeded: boolean;
    readonly type: 'InvalidTransferStatisticsFailure' | 'StatTypeMissing' | 'StatTypeNeededByTransferCondition' | 'CannotRemoveStatTypeInUse' | 'StatTypeLimitReached' | 'TransferConditionLimitReached' | 'WeightLimitExceeded';
  }

  /** @name PalletStoError (742) */
  interface PalletStoError extends Enum {
    readonly isUnauthorized: boolean;
    readonly isOverflow: boolean;
    readonly isInsufficientTokensRemaining: boolean;
    readonly isFundraiserNotFound: boolean;
    readonly isFundraiserNotLive: boolean;
    readonly isFundraiserClosed: boolean;
    readonly isFundraiserExpired: boolean;
    readonly isInvalidVenue: boolean;
    readonly isInvalidPriceTiers: boolean;
    readonly isInvalidOfferingWindow: boolean;
    readonly isMaxPriceExceeded: boolean;
    readonly isInvestmentAmountTooLow: boolean;
    readonly isInvalidSignature: boolean;
    readonly isOffchainFundingNotAllowed: boolean;
    readonly isReceiptExpired: boolean;
    readonly type: 'Unauthorized' | 'Overflow' | 'InsufficientTokensRemaining' | 'FundraiserNotFound' | 'FundraiserNotLive' | 'FundraiserClosed' | 'FundraiserExpired' | 'InvalidVenue' | 'InvalidPriceTiers' | 'InvalidOfferingWindow' | 'MaxPriceExceeded' | 'InvestmentAmountTooLow' | 'InvalidSignature' | 'OffchainFundingNotAllowed' | 'ReceiptExpired';
  }

  /** @name PalletTreasuryError (743) */
  interface PalletTreasuryError extends Enum {
    readonly isInsufficientBalance: boolean;
    readonly isInvalidIdentity: boolean;
    readonly type: 'InsufficientBalance' | 'InvalidIdentity';
  }

  /** @name PalletUtilityError (744) */
  interface PalletUtilityError extends Enum {
    readonly isTooManyCalls: boolean;
    readonly isUnableToDeriveAccountId: boolean;
    readonly type: 'TooManyCalls' | 'UnableToDeriveAccountId';
  }

  /** @name PalletBaseError (745) */
  interface PalletBaseError extends Enum {
    readonly isTooLong: boolean;
    readonly isCounterOverflow: boolean;
    readonly isInvalidAccountId: boolean;
    readonly type: 'TooLong' | 'CounterOverflow' | 'InvalidAccountId';
  }

  /** @name PalletExternalAgentsError (748) */
  interface PalletExternalAgentsError extends Enum {
    readonly isNoSuchAG: boolean;
    readonly isUnauthorizedAgent: boolean;
    readonly isAlreadyAnAgent: boolean;
    readonly isNotAnAgent: boolean;
    readonly isRemovingLastFullAgent: boolean;
    readonly isSecondaryKeyNotAuthorizedForAsset: boolean;
    readonly isBadAuthorizationType: boolean;
    readonly isExceptPermissionsNotAllowed: boolean;
    readonly type: 'NoSuchAG' | 'UnauthorizedAgent' | 'AlreadyAnAgent' | 'NotAnAgent' | 'RemovingLastFullAgent' | 'SecondaryKeyNotAuthorizedForAsset' | 'BadAuthorizationType' | 'ExceptPermissionsNotAllowed';
  }

  /** @name PalletRelayerSubsidy (749) */
  interface PalletRelayerSubsidy extends Struct {
    readonly payingKey: AccountId32;
    readonly remaining: u128;
  }

  /** @name PalletRelayerError (750) */
  interface PalletRelayerError extends Enum {
    readonly isNoPayingKey: boolean;
    readonly isNotPayingKey: boolean;
    readonly isNotAuthorized: boolean;
    readonly isOverflow: boolean;
    readonly isNoPendingSubsidy: boolean;
    readonly isInvalidSignature: boolean;
    readonly isExpiredRelayTx: boolean;
    readonly type: 'NoPayingKey' | 'NotPayingKey' | 'NotAuthorized' | 'Overflow' | 'NoPendingSubsidy' | 'InvalidSignature' | 'ExpiredRelayTx';
  }

  /** @name PalletContractsWasmCodeInfo (752) */
  interface PalletContractsWasmCodeInfo extends Struct {
    readonly owner: AccountId32;
    readonly deposit: Compact<u128>;
    readonly refcount: Compact<u64>;
    readonly determinism: PalletContractsWasmDeterminism;
    readonly codeLen: u32;
  }

  /** @name PalletContractsStorageContractInfo (753) */
  interface PalletContractsStorageContractInfo extends Struct {
    readonly trieId: Bytes;
    readonly codeHash: H256;
    readonly storageBytes: u32;
    readonly storageItems: u32;
    readonly storageByteDeposit: u128;
    readonly storageItemDeposit: u128;
    readonly storageBaseDeposit: u128;
    readonly delegateDependencies: BTreeMap<H256, u128>;
  }

  /** @name PalletContractsStorageDeletionQueueManager (758) */
  interface PalletContractsStorageDeletionQueueManager extends Struct {
    readonly insertCounter: u32;
    readonly deleteCounter: u32;
  }

  /** @name PalletContractsSchedule (760) */
  interface PalletContractsSchedule extends Struct {
    readonly limits: PalletContractsScheduleLimits;
    readonly instructionWeights: PalletContractsScheduleInstructionWeights;
  }

  /** @name PalletContractsScheduleLimits (761) */
  interface PalletContractsScheduleLimits extends Struct {
    readonly eventTopics: u32;
    readonly memoryPages: u32;
    readonly subjectLen: u32;
    readonly payloadLen: u32;
    readonly runtimeMemory: u32;
    readonly validatorRuntimeMemory: u32;
    readonly eventRefTime: u64;
  }

  /** @name PalletContractsScheduleInstructionWeights (762) */
  interface PalletContractsScheduleInstructionWeights extends Struct {
    readonly base: u32;
  }

  /** @name PalletContractsEnvironment (763) */
  interface PalletContractsEnvironment extends Struct {
    readonly accountId: PalletContractsEnvironmentTypeAccountId32;
    readonly balance: PalletContractsEnvironmentTypeU128;
    readonly hash_: PalletContractsEnvironmentTypeH256;
    readonly hasher: PalletContractsEnvironmentTypeBlakeTwo256;
    readonly timestamp: PalletContractsEnvironmentTypeU64;
    readonly blockNumber: PalletContractsEnvironmentTypeU32;
  }

  /** @name PalletContractsEnvironmentTypeAccountId32 (764) */
  type PalletContractsEnvironmentTypeAccountId32 = Null;

  /** @name PalletContractsEnvironmentTypeU128 (765) */
  type PalletContractsEnvironmentTypeU128 = Null;

  /** @name PalletContractsEnvironmentTypeH256 (766) */
  type PalletContractsEnvironmentTypeH256 = Null;

  /** @name PalletContractsEnvironmentTypeBlakeTwo256 (767) */
  type PalletContractsEnvironmentTypeBlakeTwo256 = Null;

  /** @name PalletContractsEnvironmentTypeU64 (768) */
  type PalletContractsEnvironmentTypeU64 = Null;

  /** @name PalletContractsEnvironmentTypeU32 (769) */
  type PalletContractsEnvironmentTypeU32 = Null;

  /** @name PalletContractsError (771) */
  interface PalletContractsError extends Enum {
    readonly isInvalidSchedule: boolean;
    readonly isInvalidCallFlags: boolean;
    readonly isOutOfGas: boolean;
    readonly isOutputBufferTooSmall: boolean;
    readonly isTransferFailed: boolean;
    readonly isMaxCallDepthReached: boolean;
    readonly isContractNotFound: boolean;
    readonly isCodeTooLarge: boolean;
    readonly isCodeNotFound: boolean;
    readonly isCodeInfoNotFound: boolean;
    readonly isOutOfBounds: boolean;
    readonly isDecodingFailed: boolean;
    readonly isContractTrapped: boolean;
    readonly isValueTooLarge: boolean;
    readonly isTerminatedWhileReentrant: boolean;
    readonly isInputForwarded: boolean;
    readonly isRandomSubjectTooLong: boolean;
    readonly isTooManyTopics: boolean;
    readonly isNoChainExtension: boolean;
    readonly isXcmDecodeFailed: boolean;
    readonly isDuplicateContract: boolean;
    readonly isTerminatedInConstructor: boolean;
    readonly isReentranceDenied: boolean;
    readonly isStateChangeDenied: boolean;
    readonly isStorageDepositNotEnoughFunds: boolean;
    readonly isStorageDepositLimitExhausted: boolean;
    readonly isCodeInUse: boolean;
    readonly isContractReverted: boolean;
    readonly isCodeRejected: boolean;
    readonly isIndeterministic: boolean;
    readonly isMigrationInProgress: boolean;
    readonly isNoMigrationPerformed: boolean;
    readonly isMaxDelegateDependenciesReached: boolean;
    readonly isDelegateDependencyNotFound: boolean;
    readonly isDelegateDependencyAlreadyExists: boolean;
    readonly isCannotAddSelfAsDelegateDependency: boolean;
    readonly isOutOfTransientStorage: boolean;
    readonly type: 'InvalidSchedule' | 'InvalidCallFlags' | 'OutOfGas' | 'OutputBufferTooSmall' | 'TransferFailed' | 'MaxCallDepthReached' | 'ContractNotFound' | 'CodeTooLarge' | 'CodeNotFound' | 'CodeInfoNotFound' | 'OutOfBounds' | 'DecodingFailed' | 'ContractTrapped' | 'ValueTooLarge' | 'TerminatedWhileReentrant' | 'InputForwarded' | 'RandomSubjectTooLong' | 'TooManyTopics' | 'NoChainExtension' | 'XcmDecodeFailed' | 'DuplicateContract' | 'TerminatedInConstructor' | 'ReentranceDenied' | 'StateChangeDenied' | 'StorageDepositNotEnoughFunds' | 'StorageDepositLimitExhausted' | 'CodeInUse' | 'ContractReverted' | 'CodeRejected' | 'Indeterministic' | 'MigrationInProgress' | 'NoMigrationPerformed' | 'MaxDelegateDependenciesReached' | 'DelegateDependencyNotFound' | 'DelegateDependencyAlreadyExists' | 'CannotAddSelfAsDelegateDependency' | 'OutOfTransientStorage';
  }

  /** @name PolymeshContractsError (773) */
  interface PolymeshContractsError extends Enum {
    readonly isInvalidFuncId: boolean;
    readonly isInvalidRuntimeCall: boolean;
    readonly isReadStorageFailed: boolean;
    readonly isDataLeftAfterDecoding: boolean;
    readonly isInLenTooLarge: boolean;
    readonly isOutLenTooLarge: boolean;
    readonly isInstantiatorWithNoIdentity: boolean;
    readonly isRuntimeCallDenied: boolean;
    readonly isCallerNotAPrimaryKey: boolean;
    readonly isMissingKeyPermissions: boolean;
    readonly isInvalidChainVersion: boolean;
    readonly isNoUpgradesSupported: boolean;
    readonly type: 'InvalidFuncId' | 'InvalidRuntimeCall' | 'ReadStorageFailed' | 'DataLeftAfterDecoding' | 'InLenTooLarge' | 'OutLenTooLarge' | 'InstantiatorWithNoIdentity' | 'RuntimeCallDenied' | 'CallerNotAPrimaryKey' | 'MissingKeyPermissions' | 'InvalidChainVersion' | 'NoUpgradesSupported';
  }

  /** @name PalletPreimageOldRequestStatus (774) */
  interface PalletPreimageOldRequestStatus extends Enum {
    readonly isUnrequested: boolean;
    readonly asUnrequested: {
      readonly deposit: ITuple<[AccountId32, u128]>;
      readonly len: u32;
    } & Struct;
    readonly isRequested: boolean;
    readonly asRequested: {
      readonly deposit: Option<ITuple<[AccountId32, u128]>>;
      readonly count: u32;
      readonly len: Option<u32>;
    } & Struct;
    readonly type: 'Unrequested' | 'Requested';
  }

  /** @name PalletPreimageRequestStatus (776) */
  interface PalletPreimageRequestStatus extends Enum {
    readonly isUnrequested: boolean;
    readonly asUnrequested: {
      readonly ticket: ITuple<[AccountId32, u128]>;
      readonly len: u32;
    } & Struct;
    readonly isRequested: boolean;
    readonly asRequested: {
      readonly maybeTicket: Option<ITuple<[AccountId32, u128]>>;
      readonly count: u32;
      readonly maybeLen: Option<u32>;
    } & Struct;
    readonly type: 'Unrequested' | 'Requested';
  }

  /** @name PalletPreimageError (781) */
  interface PalletPreimageError extends Enum {
    readonly isTooBig: boolean;
    readonly isAlreadyNoted: boolean;
    readonly isNotAuthorized: boolean;
    readonly isNotNoted: boolean;
    readonly isRequested: boolean;
    readonly isNotRequested: boolean;
    readonly isTooMany: boolean;
    readonly isTooFew: boolean;
    readonly type: 'TooBig' | 'AlreadyNoted' | 'NotAuthorized' | 'NotNoted' | 'Requested' | 'NotRequested' | 'TooMany' | 'TooFew';
  }

  /** @name PolymeshPrimitivesNftNftCollection (782) */
  interface PolymeshPrimitivesNftNftCollection extends Struct {
    readonly id: u64;
    readonly assetId: PolymeshPrimitivesAssetAssetId;
  }

  /** @name PolymeshPrimitivesNftNftOwnerStatus (787) */
  interface PolymeshPrimitivesNftNftOwnerStatus extends Enum {
    readonly isNotOwned: boolean;
    readonly isOwner: boolean;
    readonly isOwnerLocked: boolean;
    readonly type: 'NotOwned' | 'Owner' | 'OwnerLocked';
  }

  /** @name PalletNftError (788) */
  interface PalletNftError extends Enum {
    readonly isBalanceOverflow: boolean;
    readonly isBalanceUnderflow: boolean;
    readonly isCollectionAlredyRegistered: boolean;
    readonly isCollectionNotFound: boolean;
    readonly isDuplicateMetadataKey: boolean;
    readonly isDuplicatedNFTId: boolean;
    readonly isInvalidAssetType: boolean;
    readonly isInvalidMetadataAttribute: boolean;
    readonly isInvalidNFTTransferCollectionNotFound: boolean;
    readonly isInvalidNFTTransferSamePortfolio: boolean;
    readonly isInvalidNFTTransferNFTNotOwned: boolean;
    readonly isInvalidNFTTransferCountOverflow: boolean;
    readonly isInvalidNFTTransferComplianceFailure: boolean;
    readonly isInvalidNFTTransferFrozenAsset: boolean;
    readonly isInvalidNFTTransferInsufficientCount: boolean;
    readonly isMaxNumberOfKeysExceeded: boolean;
    readonly isMaxNumberOfNFTsPerLegExceeded: boolean;
    readonly isNftNotFound: boolean;
    readonly isUnregisteredMetadataKey: boolean;
    readonly isZeroCount: boolean;
    readonly isSupplyOverflow: boolean;
    readonly isSupplyUnderflow: boolean;
    readonly isInvalidNFTTransferNFTIsLocked: boolean;
    readonly isInvalidNFTTransferSenderDidMatchesReceiverDid: boolean;
    readonly isInvalidNFTTransferInvalidReceiverDID: boolean;
    readonly isInvalidAssetId: boolean;
    readonly isNftIsLocked: boolean;
    readonly isNumberOfKeysIsLessThanExpected: boolean;
    readonly isNftIsNotLocked: boolean;
    readonly type: 'BalanceOverflow' | 'BalanceUnderflow' | 'CollectionAlredyRegistered' | 'CollectionNotFound' | 'DuplicateMetadataKey' | 'DuplicatedNFTId' | 'InvalidAssetType' | 'InvalidMetadataAttribute' | 'InvalidNFTTransferCollectionNotFound' | 'InvalidNFTTransferSamePortfolio' | 'InvalidNFTTransferNFTNotOwned' | 'InvalidNFTTransferCountOverflow' | 'InvalidNFTTransferComplianceFailure' | 'InvalidNFTTransferFrozenAsset' | 'InvalidNFTTransferInsufficientCount' | 'MaxNumberOfKeysExceeded' | 'MaxNumberOfNFTsPerLegExceeded' | 'NftNotFound' | 'UnregisteredMetadataKey' | 'ZeroCount' | 'SupplyOverflow' | 'SupplyUnderflow' | 'InvalidNFTTransferNFTIsLocked' | 'InvalidNFTTransferSenderDidMatchesReceiverDid' | 'InvalidNFTTransferInvalidReceiverDID' | 'InvalidAssetId' | 'NftIsLocked' | 'NumberOfKeysIsLessThanExpected' | 'NftIsNotLocked';
  }

  /** @name PalletElectionProviderMultiPhaseReadySolution (789) */
  interface PalletElectionProviderMultiPhaseReadySolution extends Struct {
    readonly supports: FrameElectionProviderSupportBoundedSupports;
    readonly score: SpNposElectionsElectionScore;
    readonly compute: PalletElectionProviderMultiPhaseElectionCompute;
  }

  /** @name FrameElectionProviderSupportBoundedSupports (790) */
  interface FrameElectionProviderSupportBoundedSupports extends Vec<ITuple<[AccountId32, FrameElectionProviderSupportBoundedSupport]>> {}

  /** @name FrameElectionProviderSupportBoundedSupport (793) */
  interface FrameElectionProviderSupportBoundedSupport extends Struct {
    readonly total: u128;
    readonly voters: Vec<ITuple<[AccountId32, u128]>>;
  }

  /** @name PalletElectionProviderMultiPhaseRoundSnapshot (796) */
  interface PalletElectionProviderMultiPhaseRoundSnapshot extends Struct {
    readonly voters: Vec<ITuple<[AccountId32, u64, Vec<AccountId32>]>>;
    readonly targets: Vec<AccountId32>;
  }

  /** @name PalletElectionProviderMultiPhaseSignedSignedSubmission (802) */
  interface PalletElectionProviderMultiPhaseSignedSignedSubmission extends Struct {
    readonly who: AccountId32;
    readonly deposit: u128;
    readonly rawSolution: PalletElectionProviderMultiPhaseRawSolution;
    readonly callFee: u128;
  }

  /** @name PalletElectionProviderMultiPhaseError (803) */
  interface PalletElectionProviderMultiPhaseError extends Enum {
    readonly isPreDispatchEarlySubmission: boolean;
    readonly isPreDispatchWrongWinnerCount: boolean;
    readonly isPreDispatchWeakSubmission: boolean;
    readonly isSignedQueueFull: boolean;
    readonly isSignedCannotPayDeposit: boolean;
    readonly isSignedInvalidWitness: boolean;
    readonly isSignedTooMuchWeight: boolean;
    readonly isOcwCallWrongEra: boolean;
    readonly isMissingSnapshotMetadata: boolean;
    readonly isInvalidSubmissionIndex: boolean;
    readonly isCallNotAllowed: boolean;
    readonly isFallbackFailed: boolean;
    readonly isBoundNotMet: boolean;
    readonly isTooManyWinners: boolean;
    readonly isPreDispatchDifferentRound: boolean;
    readonly type: 'PreDispatchEarlySubmission' | 'PreDispatchWrongWinnerCount' | 'PreDispatchWeakSubmission' | 'SignedQueueFull' | 'SignedCannotPayDeposit' | 'SignedInvalidWitness' | 'SignedTooMuchWeight' | 'OcwCallWrongEra' | 'MissingSnapshotMetadata' | 'InvalidSubmissionIndex' | 'CallNotAllowed' | 'FallbackFailed' | 'BoundNotMet' | 'TooManyWinners' | 'PreDispatchDifferentRound';
  }

  /** @name PalletBeefyError (806) */
  interface PalletBeefyError extends Enum {
    readonly isInvalidKeyOwnershipProof: boolean;
    readonly isInvalidDoubleVotingProof: boolean;
    readonly isInvalidForkVotingProof: boolean;
    readonly isInvalidFutureBlockVotingProof: boolean;
    readonly isInvalidEquivocationProofSession: boolean;
    readonly isInvalidEquivocationProofSessionMember: boolean;
    readonly isDuplicateOffenceReport: boolean;
    readonly isInvalidConfiguration: boolean;
    readonly type: 'InvalidKeyOwnershipProof' | 'InvalidDoubleVotingProof' | 'InvalidForkVotingProof' | 'InvalidFutureBlockVotingProof' | 'InvalidEquivocationProofSession' | 'InvalidEquivocationProofSessionMember' | 'DuplicateOffenceReport' | 'InvalidConfiguration';
  }

  /** @name SpConsensusBeefyMmrBeefyAuthoritySet (807) */
  interface SpConsensusBeefyMmrBeefyAuthoritySet extends Struct {
    readonly id: u64;
    readonly len: u32;
    readonly keysetCommitment: H256;
  }

  /** @name FrameSystemExtensionsAuthorizeCall (810) */
  type FrameSystemExtensionsAuthorizeCall = Null;

  /** @name FrameSystemExtensionsCheckNonZeroSender (811) */
  type FrameSystemExtensionsCheckNonZeroSender = Null;

  /** @name FrameSystemExtensionsCheckSpecVersion (812) */
  type FrameSystemExtensionsCheckSpecVersion = Null;

  /** @name FrameSystemExtensionsCheckTxVersion (813) */
  type FrameSystemExtensionsCheckTxVersion = Null;

  /** @name FrameSystemExtensionsCheckGenesis (814) */
  type FrameSystemExtensionsCheckGenesis = Null;

  /** @name FrameSystemExtensionsCheckNonce (817) */
  interface FrameSystemExtensionsCheckNonce extends Compact<u32> {}

  /** @name FrameSystemExtensionsCheckWeight (818) */
  type FrameSystemExtensionsCheckWeight = Null;

  /** @name PolymeshTransactionPaymentChargeTransactionPayment (819) */
  interface PolymeshTransactionPaymentChargeTransactionPayment extends Compact<u128> {}

  /** @name PalletPermissionsStoreCallMetadata (820) */
  type PalletPermissionsStoreCallMetadata = Null;

  /** @name FrameMetadataHashExtensionCheckMetadataHash (821) */
  interface FrameMetadataHashExtensionCheckMetadataHash extends Struct {
    readonly mode: FrameMetadataHashExtensionMode;
  }

  /** @name FrameMetadataHashExtensionMode (822) */
  interface FrameMetadataHashExtensionMode extends Enum {
    readonly isDisabled: boolean;
    readonly isEnabled: boolean;
    readonly type: 'Disabled' | 'Enabled';
  }

  /** @name FrameSystemExtensionsWeightReclaim (823) */
  type FrameSystemExtensionsWeightReclaim = Null;

} // declare module
