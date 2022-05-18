// Auto-generated via `yarn polkadot-types-from-defs`, do not edit
/* eslint-disable */

declare module '@polkadot/types/lookup' {
  import type { BTreeMap, Bytes, Compact, Enum, Null, Option, Result, Struct, Text, U8aFixed, Vec, bool, u128, u16, u32, u64, u8 } from '@polkadot/types-codec';
  import type { ITuple } from '@polkadot/types-codec/types';
  import type { AccountId32, Call, H256, H512, MultiAddress, PerU16, Perbill, Percent, Permill } from '@polkadot/types/interfaces/runtime';
  import type { Event } from '@polkadot/types/interfaces/system';

  /** @name FrameSystemAccountInfo (3) */
  export interface FrameSystemAccountInfo extends Struct {
    readonly nonce: u32;
    readonly consumers: u32;
    readonly providers: u32;
    readonly sufficients: u32;
    readonly data: PolymeshCommonUtilitiesBalancesAccountData;
  }

  /** @name PolymeshCommonUtilitiesBalancesAccountData (5) */
  export interface PolymeshCommonUtilitiesBalancesAccountData extends Struct {
    readonly free: u128;
    readonly reserved: u128;
    readonly miscFrozen: u128;
    readonly feeFrozen: u128;
  }

  /** @name FrameSupportWeightsPerDispatchClassU64 (7) */
  export interface FrameSupportWeightsPerDispatchClassU64 extends Struct {
    readonly normal: u64;
    readonly operational: u64;
    readonly mandatory: u64;
  }

  /** @name SpRuntimeDigest (11) */
  export interface SpRuntimeDigest extends Struct {
    readonly logs: Vec<SpRuntimeDigestDigestItem>;
  }

  /** @name SpRuntimeDigestDigestItem (13) */
  export interface SpRuntimeDigestDigestItem extends Enum {
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

  /** @name FrameSystemEventRecord (16) */
  export interface FrameSystemEventRecord extends Struct {
    readonly phase: FrameSystemPhase;
    readonly event: Event;
    readonly topics: Vec<H256>;
  }

  /** @name FrameSystemEvent (18) */
  export interface FrameSystemEvent extends Enum {
    readonly isExtrinsicSuccess: boolean;
    readonly asExtrinsicSuccess: {
      readonly dispatchInfo: FrameSupportWeightsDispatchInfo;
    } & Struct;
    readonly isExtrinsicFailed: boolean;
    readonly asExtrinsicFailed: {
      readonly dispatchError: SpRuntimeDispatchError;
      readonly dispatchInfo: FrameSupportWeightsDispatchInfo;
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
    readonly type: 'ExtrinsicSuccess' | 'ExtrinsicFailed' | 'CodeUpdated' | 'NewAccount' | 'KilledAccount' | 'Remarked';
  }

  /** @name FrameSupportWeightsDispatchInfo (19) */
  export interface FrameSupportWeightsDispatchInfo extends Struct {
    readonly weight: u64;
    readonly class: FrameSupportWeightsDispatchClass;
    readonly paysFee: FrameSupportWeightsPays;
  }

  /** @name FrameSupportWeightsDispatchClass (20) */
  export interface FrameSupportWeightsDispatchClass extends Enum {
    readonly isNormal: boolean;
    readonly isOperational: boolean;
    readonly isMandatory: boolean;
    readonly type: 'Normal' | 'Operational' | 'Mandatory';
  }

  /** @name FrameSupportWeightsPays (21) */
  export interface FrameSupportWeightsPays extends Enum {
    readonly isYes: boolean;
    readonly isNo: boolean;
    readonly type: 'Yes' | 'No';
  }

  /** @name SpRuntimeDispatchError (22) */
  export interface SpRuntimeDispatchError extends Enum {
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
    readonly asArithmetic: SpRuntimeArithmeticError;
    readonly isTransactional: boolean;
    readonly asTransactional: SpRuntimeTransactionalError;
    readonly type: 'Other' | 'CannotLookup' | 'BadOrigin' | 'Module' | 'ConsumerRemaining' | 'NoProviders' | 'TooManyConsumers' | 'Token' | 'Arithmetic' | 'Transactional';
  }

  /** @name SpRuntimeModuleError (23) */
  export interface SpRuntimeModuleError extends Struct {
    readonly index: u8;
    readonly error: U8aFixed;
  }

  /** @name SpRuntimeTokenError (24) */
  export interface SpRuntimeTokenError extends Enum {
    readonly isNoFunds: boolean;
    readonly isWouldDie: boolean;
    readonly isBelowMinimum: boolean;
    readonly isCannotCreate: boolean;
    readonly isUnknownAsset: boolean;
    readonly isFrozen: boolean;
    readonly isUnsupported: boolean;
    readonly type: 'NoFunds' | 'WouldDie' | 'BelowMinimum' | 'CannotCreate' | 'UnknownAsset' | 'Frozen' | 'Unsupported';
  }

  /** @name SpRuntimeArithmeticError (25) */
  export interface SpRuntimeArithmeticError extends Enum {
    readonly isUnderflow: boolean;
    readonly isOverflow: boolean;
    readonly isDivisionByZero: boolean;
    readonly type: 'Underflow' | 'Overflow' | 'DivisionByZero';
  }

  /** @name SpRuntimeTransactionalError (26) */
  export interface SpRuntimeTransactionalError extends Enum {
    readonly isLimitReached: boolean;
    readonly isNoLayer: boolean;
    readonly type: 'LimitReached' | 'NoLayer';
  }

  /** @name PalletIndicesEvent (27) */
  export interface PalletIndicesEvent extends Enum {
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
    readonly type: 'IndexAssigned' | 'IndexFreed' | 'IndexFrozen';
  }

  /** @name PolymeshCommonUtilitiesBalancesRawEvent (28) */
  export interface PolymeshCommonUtilitiesBalancesRawEvent extends Enum {
    readonly isEndowed: boolean;
    readonly asEndowed: ITuple<[Option<PolymeshPrimitivesIdentityId>, AccountId32, u128]>;
    readonly isTransfer: boolean;
    readonly asTransfer: ITuple<[Option<PolymeshPrimitivesIdentityId>, AccountId32, Option<PolymeshPrimitivesIdentityId>, AccountId32, u128, Option<PolymeshCommonUtilitiesBalancesMemo>]>;
    readonly isBalanceSet: boolean;
    readonly asBalanceSet: ITuple<[PolymeshPrimitivesIdentityId, AccountId32, u128, u128]>;
    readonly isAccountBalanceBurned: boolean;
    readonly asAccountBalanceBurned: ITuple<[PolymeshPrimitivesIdentityId, AccountId32, u128]>;
    readonly isReserved: boolean;
    readonly asReserved: ITuple<[AccountId32, u128]>;
    readonly isUnreserved: boolean;
    readonly asUnreserved: ITuple<[AccountId32, u128]>;
    readonly isReserveRepatriated: boolean;
    readonly asReserveRepatriated: ITuple<[AccountId32, AccountId32, u128, FrameSupportTokensMiscBalanceStatus]>;
    readonly type: 'Endowed' | 'Transfer' | 'BalanceSet' | 'AccountBalanceBurned' | 'Reserved' | 'Unreserved' | 'ReserveRepatriated';
  }

  /** @name PolymeshPrimitivesIdentityId (30) */
  export interface PolymeshPrimitivesIdentityId extends U8aFixed {}

  /** @name PolymeshCommonUtilitiesBalancesMemo (32) */
  export interface PolymeshCommonUtilitiesBalancesMemo extends U8aFixed {}

  /** @name FrameSupportTokensMiscBalanceStatus (33) */
  export interface FrameSupportTokensMiscBalanceStatus extends Enum {
    readonly isFree: boolean;
    readonly isReserved: boolean;
    readonly type: 'Free' | 'Reserved';
  }

  /** @name PolymeshCommonUtilitiesIdentityRawEvent (34) */
  export interface PolymeshCommonUtilitiesIdentityRawEvent extends Enum {
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
    readonly isAssetDidRegistered: boolean;
    readonly asAssetDidRegistered: ITuple<[PolymeshPrimitivesIdentityId, PolymeshPrimitivesTicker]>;
    readonly isAuthorizationAdded: boolean;
    readonly asAuthorizationAdded: ITuple<[PolymeshPrimitivesIdentityId, Option<PolymeshPrimitivesIdentityId>, Option<AccountId32>, u64, PolymeshPrimitivesAuthorizationAuthorizationData, Option<u64>]>;
    readonly isAuthorizationRevoked: boolean;
    readonly asAuthorizationRevoked: ITuple<[Option<PolymeshPrimitivesIdentityId>, Option<AccountId32>, u64]>;
    readonly isAuthorizationRejected: boolean;
    readonly asAuthorizationRejected: ITuple<[Option<PolymeshPrimitivesIdentityId>, Option<AccountId32>, u64]>;
    readonly isAuthorizationConsumed: boolean;
    readonly asAuthorizationConsumed: ITuple<[Option<PolymeshPrimitivesIdentityId>, Option<AccountId32>, u64]>;
    readonly isCddRequirementForPrimaryKeyUpdated: boolean;
    readonly asCddRequirementForPrimaryKeyUpdated: bool;
    readonly isCddClaimsInvalidated: boolean;
    readonly asCddClaimsInvalidated: ITuple<[PolymeshPrimitivesIdentityId, u64]>;
    readonly isSecondaryKeysFrozen: boolean;
    readonly asSecondaryKeysFrozen: PolymeshPrimitivesIdentityId;
    readonly isSecondaryKeysUnfrozen: boolean;
    readonly asSecondaryKeysUnfrozen: PolymeshPrimitivesIdentityId;
    readonly type: 'DidCreated' | 'SecondaryKeysAdded' | 'SecondaryKeysRemoved' | 'SecondaryKeyLeftIdentity' | 'SecondaryKeyPermissionsUpdated' | 'PrimaryKeyUpdated' | 'ClaimAdded' | 'ClaimRevoked' | 'AssetDidRegistered' | 'AuthorizationAdded' | 'AuthorizationRevoked' | 'AuthorizationRejected' | 'AuthorizationConsumed' | 'CddRequirementForPrimaryKeyUpdated' | 'CddClaimsInvalidated' | 'SecondaryKeysFrozen' | 'SecondaryKeysUnfrozen';
  }

  /** @name PolymeshPrimitivesSecondaryKey (36) */
  export interface PolymeshPrimitivesSecondaryKey extends Struct {
    readonly key: AccountId32;
    readonly permissions: PolymeshPrimitivesSecondaryKeyPermissions;
  }

  /** @name PolymeshPrimitivesSecondaryKeyPermissions (37) */
  export interface PolymeshPrimitivesSecondaryKeyPermissions extends Struct {
    readonly asset: PolymeshPrimitivesSubsetSubsetRestrictionTicker;
    readonly extrinsic: PolymeshPrimitivesSubsetSubsetRestrictionPalletPermissions;
    readonly portfolio: PolymeshPrimitivesSubsetSubsetRestrictionPortfolioId;
  }

  /** @name PolymeshPrimitivesSubsetSubsetRestrictionTicker (38) */
  export interface PolymeshPrimitivesSubsetSubsetRestrictionTicker extends Enum {
    readonly isWhole: boolean;
    readonly isThese: boolean;
    readonly asThese: BTreeSetTicker;
    readonly isExcept: boolean;
    readonly asExcept: BTreeSetTicker;
    readonly type: 'Whole' | 'These' | 'Except';
  }

  /** @name PolymeshPrimitivesTicker (39) */
  export interface PolymeshPrimitivesTicker extends U8aFixed {}

  /** @name BTreeSetTicker (41) */
  export interface BTreeSetTicker extends Vec<PolymeshPrimitivesTicker> {}

  /** @name PolymeshPrimitivesSubsetSubsetRestrictionPalletPermissions (43) */
  export interface PolymeshPrimitivesSubsetSubsetRestrictionPalletPermissions extends Enum {
    readonly isWhole: boolean;
    readonly isThese: boolean;
    readonly asThese: BTreeSetPalletPermissions;
    readonly isExcept: boolean;
    readonly asExcept: BTreeSetPalletPermissions;
    readonly type: 'Whole' | 'These' | 'Except';
  }

  /** @name PolymeshPrimitivesSecondaryKeyPalletPermissions (44) */
  export interface PolymeshPrimitivesSecondaryKeyPalletPermissions extends Struct {
    readonly palletName: Bytes;
    readonly dispatchableNames: PolymeshPrimitivesSubsetSubsetRestrictionDispatchableName;
  }

  /** @name PolymeshPrimitivesSubsetSubsetRestrictionDispatchableName (46) */
  export interface PolymeshPrimitivesSubsetSubsetRestrictionDispatchableName extends Enum {
    readonly isWhole: boolean;
    readonly isThese: boolean;
    readonly asThese: BTreeSetDispatchableName;
    readonly isExcept: boolean;
    readonly asExcept: BTreeSetDispatchableName;
    readonly type: 'Whole' | 'These' | 'Except';
  }

  /** @name BTreeSetDispatchableName (48) */
  export interface BTreeSetDispatchableName extends Vec<Bytes> {}

  /** @name BTreeSetPalletPermissions (50) */
  export interface BTreeSetPalletPermissions extends Vec<PolymeshPrimitivesSecondaryKeyPalletPermissions> {}

  /** @name PolymeshPrimitivesSubsetSubsetRestrictionPortfolioId (52) */
  export interface PolymeshPrimitivesSubsetSubsetRestrictionPortfolioId extends Enum {
    readonly isWhole: boolean;
    readonly isThese: boolean;
    readonly asThese: BTreeSetPortfolioId;
    readonly isExcept: boolean;
    readonly asExcept: BTreeSetPortfolioId;
    readonly type: 'Whole' | 'These' | 'Except';
  }

  /** @name PolymeshPrimitivesIdentityIdPortfolioId (53) */
  export interface PolymeshPrimitivesIdentityIdPortfolioId extends Struct {
    readonly did: PolymeshPrimitivesIdentityId;
    readonly kind: PolymeshPrimitivesIdentityIdPortfolioKind;
  }

  /** @name PolymeshPrimitivesIdentityIdPortfolioKind (54) */
  export interface PolymeshPrimitivesIdentityIdPortfolioKind extends Enum {
    readonly isDefault: boolean;
    readonly isUser: boolean;
    readonly asUser: u64;
    readonly type: 'Default' | 'User';
  }

  /** @name BTreeSetPortfolioId (56) */
  export interface BTreeSetPortfolioId extends Vec<PolymeshPrimitivesIdentityIdPortfolioId> {}

  /** @name PolymeshPrimitivesIdentityClaim (59) */
  export interface PolymeshPrimitivesIdentityClaim extends Struct {
    readonly claimIssuer: PolymeshPrimitivesIdentityId;
    readonly issuanceDate: u64;
    readonly lastUpdateDate: u64;
    readonly expiry: Option<u64>;
    readonly claim: PolymeshPrimitivesIdentityClaimClaim;
  }

  /** @name PolymeshPrimitivesIdentityClaimClaim (61) */
  export interface PolymeshPrimitivesIdentityClaimClaim extends Enum {
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
    readonly isInvestorUniqueness: boolean;
    readonly asInvestorUniqueness: ITuple<[PolymeshPrimitivesIdentityClaimScope, PolymeshPrimitivesIdentityId, PolymeshPrimitivesCddId]>;
    readonly isNoData: boolean;
    readonly isInvestorUniquenessV2: boolean;
    readonly asInvestorUniquenessV2: PolymeshPrimitivesCddId;
    readonly type: 'Accredited' | 'Affiliate' | 'BuyLockup' | 'SellLockup' | 'CustomerDueDiligence' | 'KnowYourCustomer' | 'Jurisdiction' | 'Exempted' | 'Blocked' | 'InvestorUniqueness' | 'NoData' | 'InvestorUniquenessV2';
  }

  /** @name PolymeshPrimitivesIdentityClaimScope (62) */
  export interface PolymeshPrimitivesIdentityClaimScope extends Enum {
    readonly isIdentity: boolean;
    readonly asIdentity: PolymeshPrimitivesIdentityId;
    readonly isTicker: boolean;
    readonly asTicker: PolymeshPrimitivesTicker;
    readonly isCustom: boolean;
    readonly asCustom: Bytes;
    readonly type: 'Identity' | 'Ticker' | 'Custom';
  }

  /** @name PolymeshPrimitivesCddId (63) */
  export interface PolymeshPrimitivesCddId extends U8aFixed {}

  /** @name PolymeshPrimitivesJurisdictionCountryCode (64) */
  export interface PolymeshPrimitivesJurisdictionCountryCode extends Enum {
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

  /** @name PolymeshPrimitivesAuthorizationAuthorizationData (66) */
  export interface PolymeshPrimitivesAuthorizationAuthorizationData extends Enum {
    readonly isAttestPrimaryKeyRotation: boolean;
    readonly asAttestPrimaryKeyRotation: PolymeshPrimitivesIdentityId;
    readonly isRotatePrimaryKey: boolean;
    readonly isTransferTicker: boolean;
    readonly asTransferTicker: PolymeshPrimitivesTicker;
    readonly isAddMultiSigSigner: boolean;
    readonly asAddMultiSigSigner: AccountId32;
    readonly isTransferAssetOwnership: boolean;
    readonly asTransferAssetOwnership: PolymeshPrimitivesTicker;
    readonly isJoinIdentity: boolean;
    readonly asJoinIdentity: PolymeshPrimitivesSecondaryKeyPermissions;
    readonly isPortfolioCustody: boolean;
    readonly asPortfolioCustody: PolymeshPrimitivesIdentityIdPortfolioId;
    readonly isBecomeAgent: boolean;
    readonly asBecomeAgent: ITuple<[PolymeshPrimitivesTicker, PolymeshPrimitivesAgentAgentGroup]>;
    readonly isAddRelayerPayingKey: boolean;
    readonly asAddRelayerPayingKey: ITuple<[AccountId32, AccountId32, u128]>;
    readonly isRotatePrimaryKeyToSecondary: boolean;
    readonly asRotatePrimaryKeyToSecondary: PolymeshPrimitivesSecondaryKeyPermissions;
    readonly type: 'AttestPrimaryKeyRotation' | 'RotatePrimaryKey' | 'TransferTicker' | 'AddMultiSigSigner' | 'TransferAssetOwnership' | 'JoinIdentity' | 'PortfolioCustody' | 'BecomeAgent' | 'AddRelayerPayingKey' | 'RotatePrimaryKeyToSecondary';
  }

  /** @name PolymeshPrimitivesAgentAgentGroup (67) */
  export interface PolymeshPrimitivesAgentAgentGroup extends Enum {
    readonly isFull: boolean;
    readonly isCustom: boolean;
    readonly asCustom: u32;
    readonly isExceptMeta: boolean;
    readonly isPolymeshV1CAA: boolean;
    readonly isPolymeshV1PIA: boolean;
    readonly type: 'Full' | 'Custom' | 'ExceptMeta' | 'PolymeshV1CAA' | 'PolymeshV1PIA';
  }

  /** @name PolymeshCommonUtilitiesGroupRawEventInstance2 (70) */
  export interface PolymeshCommonUtilitiesGroupRawEventInstance2 extends Enum {
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
    readonly isDummy: boolean;
    readonly type: 'MemberAdded' | 'MemberRemoved' | 'MemberRevoked' | 'MembersSwapped' | 'MembersReset' | 'ActiveLimitChanged' | 'Dummy';
  }

  /** @name PalletGroupInstance2 (71) */
  export type PalletGroupInstance2 = Null;

  /** @name PalletCommitteeRawEventInstance1 (73) */
  export interface PalletCommitteeRawEventInstance1 extends Enum {
    readonly isProposed: boolean;
    readonly asProposed: ITuple<[PolymeshPrimitivesIdentityId, u32, H256]>;
    readonly isVoted: boolean;
    readonly asVoted: ITuple<[PolymeshPrimitivesIdentityId, u32, H256, bool, u32, u32, u32]>;
    readonly isVoteRetracted: boolean;
    readonly asVoteRetracted: ITuple<[PolymeshPrimitivesIdentityId, u32, H256, bool]>;
    readonly isFinalVotes: boolean;
    readonly asFinalVotes: ITuple<[PolymeshPrimitivesIdentityId, u32, H256, Vec<PolymeshPrimitivesIdentityId>, Vec<PolymeshPrimitivesIdentityId>]>;
    readonly isApproved: boolean;
    readonly asApproved: ITuple<[PolymeshPrimitivesIdentityId, H256, u32, u32, u32]>;
    readonly isRejected: boolean;
    readonly asRejected: ITuple<[PolymeshPrimitivesIdentityId, H256, u32, u32, u32]>;
    readonly isExecuted: boolean;
    readonly asExecuted: ITuple<[PolymeshPrimitivesIdentityId, H256, Result<Null, SpRuntimeDispatchError>]>;
    readonly isReleaseCoordinatorUpdated: boolean;
    readonly asReleaseCoordinatorUpdated: ITuple<[PolymeshPrimitivesIdentityId, Option<PolymeshPrimitivesIdentityId>]>;
    readonly isExpiresAfterUpdated: boolean;
    readonly asExpiresAfterUpdated: ITuple<[PolymeshPrimitivesIdentityId, PolymeshCommonUtilitiesMaybeBlock]>;
    readonly isVoteThresholdUpdated: boolean;
    readonly asVoteThresholdUpdated: ITuple<[PolymeshPrimitivesIdentityId, u32, u32]>;
    readonly type: 'Proposed' | 'Voted' | 'VoteRetracted' | 'FinalVotes' | 'Approved' | 'Rejected' | 'Executed' | 'ReleaseCoordinatorUpdated' | 'ExpiresAfterUpdated' | 'VoteThresholdUpdated';
  }

  /** @name PalletCommitteeInstance1 (74) */
  export type PalletCommitteeInstance1 = Null;

  /** @name PolymeshCommonUtilitiesMaybeBlock (77) */
  export interface PolymeshCommonUtilitiesMaybeBlock extends Enum {
    readonly isSome: boolean;
    readonly asSome: u32;
    readonly isNone: boolean;
    readonly type: 'Some' | 'None';
  }

  /** @name PolymeshCommonUtilitiesGroupRawEventInstance1 (78) */
  export interface PolymeshCommonUtilitiesGroupRawEventInstance1 extends Enum {
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
    readonly isDummy: boolean;
    readonly type: 'MemberAdded' | 'MemberRemoved' | 'MemberRevoked' | 'MembersSwapped' | 'MembersReset' | 'ActiveLimitChanged' | 'Dummy';
  }

  /** @name PalletGroupInstance1 (79) */
  export type PalletGroupInstance1 = Null;

  /** @name PalletCommitteeRawEventInstance3 (80) */
  export interface PalletCommitteeRawEventInstance3 extends Enum {
    readonly isProposed: boolean;
    readonly asProposed: ITuple<[PolymeshPrimitivesIdentityId, u32, H256]>;
    readonly isVoted: boolean;
    readonly asVoted: ITuple<[PolymeshPrimitivesIdentityId, u32, H256, bool, u32, u32, u32]>;
    readonly isVoteRetracted: boolean;
    readonly asVoteRetracted: ITuple<[PolymeshPrimitivesIdentityId, u32, H256, bool]>;
    readonly isFinalVotes: boolean;
    readonly asFinalVotes: ITuple<[PolymeshPrimitivesIdentityId, u32, H256, Vec<PolymeshPrimitivesIdentityId>, Vec<PolymeshPrimitivesIdentityId>]>;
    readonly isApproved: boolean;
    readonly asApproved: ITuple<[PolymeshPrimitivesIdentityId, H256, u32, u32, u32]>;
    readonly isRejected: boolean;
    readonly asRejected: ITuple<[PolymeshPrimitivesIdentityId, H256, u32, u32, u32]>;
    readonly isExecuted: boolean;
    readonly asExecuted: ITuple<[PolymeshPrimitivesIdentityId, H256, Result<Null, SpRuntimeDispatchError>]>;
    readonly isReleaseCoordinatorUpdated: boolean;
    readonly asReleaseCoordinatorUpdated: ITuple<[PolymeshPrimitivesIdentityId, Option<PolymeshPrimitivesIdentityId>]>;
    readonly isExpiresAfterUpdated: boolean;
    readonly asExpiresAfterUpdated: ITuple<[PolymeshPrimitivesIdentityId, PolymeshCommonUtilitiesMaybeBlock]>;
    readonly isVoteThresholdUpdated: boolean;
    readonly asVoteThresholdUpdated: ITuple<[PolymeshPrimitivesIdentityId, u32, u32]>;
    readonly type: 'Proposed' | 'Voted' | 'VoteRetracted' | 'FinalVotes' | 'Approved' | 'Rejected' | 'Executed' | 'ReleaseCoordinatorUpdated' | 'ExpiresAfterUpdated' | 'VoteThresholdUpdated';
  }

  /** @name PalletCommitteeInstance3 (81) */
  export type PalletCommitteeInstance3 = Null;

  /** @name PolymeshCommonUtilitiesGroupRawEventInstance3 (82) */
  export interface PolymeshCommonUtilitiesGroupRawEventInstance3 extends Enum {
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
    readonly isDummy: boolean;
    readonly type: 'MemberAdded' | 'MemberRemoved' | 'MemberRevoked' | 'MembersSwapped' | 'MembersReset' | 'ActiveLimitChanged' | 'Dummy';
  }

  /** @name PalletGroupInstance3 (83) */
  export type PalletGroupInstance3 = Null;

  /** @name PalletCommitteeRawEventInstance4 (84) */
  export interface PalletCommitteeRawEventInstance4 extends Enum {
    readonly isProposed: boolean;
    readonly asProposed: ITuple<[PolymeshPrimitivesIdentityId, u32, H256]>;
    readonly isVoted: boolean;
    readonly asVoted: ITuple<[PolymeshPrimitivesIdentityId, u32, H256, bool, u32, u32, u32]>;
    readonly isVoteRetracted: boolean;
    readonly asVoteRetracted: ITuple<[PolymeshPrimitivesIdentityId, u32, H256, bool]>;
    readonly isFinalVotes: boolean;
    readonly asFinalVotes: ITuple<[PolymeshPrimitivesIdentityId, u32, H256, Vec<PolymeshPrimitivesIdentityId>, Vec<PolymeshPrimitivesIdentityId>]>;
    readonly isApproved: boolean;
    readonly asApproved: ITuple<[PolymeshPrimitivesIdentityId, H256, u32, u32, u32]>;
    readonly isRejected: boolean;
    readonly asRejected: ITuple<[PolymeshPrimitivesIdentityId, H256, u32, u32, u32]>;
    readonly isExecuted: boolean;
    readonly asExecuted: ITuple<[PolymeshPrimitivesIdentityId, H256, Result<Null, SpRuntimeDispatchError>]>;
    readonly isReleaseCoordinatorUpdated: boolean;
    readonly asReleaseCoordinatorUpdated: ITuple<[PolymeshPrimitivesIdentityId, Option<PolymeshPrimitivesIdentityId>]>;
    readonly isExpiresAfterUpdated: boolean;
    readonly asExpiresAfterUpdated: ITuple<[PolymeshPrimitivesIdentityId, PolymeshCommonUtilitiesMaybeBlock]>;
    readonly isVoteThresholdUpdated: boolean;
    readonly asVoteThresholdUpdated: ITuple<[PolymeshPrimitivesIdentityId, u32, u32]>;
    readonly type: 'Proposed' | 'Voted' | 'VoteRetracted' | 'FinalVotes' | 'Approved' | 'Rejected' | 'Executed' | 'ReleaseCoordinatorUpdated' | 'ExpiresAfterUpdated' | 'VoteThresholdUpdated';
  }

  /** @name PalletCommitteeInstance4 (85) */
  export type PalletCommitteeInstance4 = Null;

  /** @name PolymeshCommonUtilitiesGroupRawEventInstance4 (86) */
  export interface PolymeshCommonUtilitiesGroupRawEventInstance4 extends Enum {
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
    readonly isDummy: boolean;
    readonly type: 'MemberAdded' | 'MemberRemoved' | 'MemberRevoked' | 'MembersSwapped' | 'MembersReset' | 'ActiveLimitChanged' | 'Dummy';
  }

  /** @name PalletGroupInstance4 (87) */
  export type PalletGroupInstance4 = Null;

  /** @name PalletMultisigRawEvent (88) */
  export interface PalletMultisigRawEvent extends Enum {
    readonly isMultiSigCreated: boolean;
    readonly asMultiSigCreated: ITuple<[PolymeshPrimitivesIdentityId, AccountId32, AccountId32, Vec<PolymeshPrimitivesSecondaryKeySignatory>, u64]>;
    readonly isProposalAdded: boolean;
    readonly asProposalAdded: ITuple<[PolymeshPrimitivesIdentityId, AccountId32, u64]>;
    readonly isProposalExecuted: boolean;
    readonly asProposalExecuted: ITuple<[PolymeshPrimitivesIdentityId, AccountId32, u64, bool]>;
    readonly isMultiSigSignerAdded: boolean;
    readonly asMultiSigSignerAdded: ITuple<[PolymeshPrimitivesIdentityId, AccountId32, PolymeshPrimitivesSecondaryKeySignatory]>;
    readonly isMultiSigSignerAuthorized: boolean;
    readonly asMultiSigSignerAuthorized: ITuple<[PolymeshPrimitivesIdentityId, AccountId32, PolymeshPrimitivesSecondaryKeySignatory]>;
    readonly isMultiSigSignerRemoved: boolean;
    readonly asMultiSigSignerRemoved: ITuple<[PolymeshPrimitivesIdentityId, AccountId32, PolymeshPrimitivesSecondaryKeySignatory]>;
    readonly isMultiSigSignaturesRequiredChanged: boolean;
    readonly asMultiSigSignaturesRequiredChanged: ITuple<[PolymeshPrimitivesIdentityId, AccountId32, u64]>;
    readonly isProposalApproved: boolean;
    readonly asProposalApproved: ITuple<[PolymeshPrimitivesIdentityId, AccountId32, PolymeshPrimitivesSecondaryKeySignatory, u64]>;
    readonly isProposalRejectionVote: boolean;
    readonly asProposalRejectionVote: ITuple<[PolymeshPrimitivesIdentityId, AccountId32, PolymeshPrimitivesSecondaryKeySignatory, u64]>;
    readonly isProposalRejected: boolean;
    readonly asProposalRejected: ITuple<[PolymeshPrimitivesIdentityId, AccountId32, u64]>;
    readonly isProposalExecutionFailed: boolean;
    readonly asProposalExecutionFailed: SpRuntimeDispatchError;
    readonly isSchedulingFailed: boolean;
    readonly asSchedulingFailed: SpRuntimeDispatchError;
    readonly type: 'MultiSigCreated' | 'ProposalAdded' | 'ProposalExecuted' | 'MultiSigSignerAdded' | 'MultiSigSignerAuthorized' | 'MultiSigSignerRemoved' | 'MultiSigSignaturesRequiredChanged' | 'ProposalApproved' | 'ProposalRejectionVote' | 'ProposalRejected' | 'ProposalExecutionFailed' | 'SchedulingFailed';
  }

  /** @name PolymeshPrimitivesSecondaryKeySignatory (90) */
  export interface PolymeshPrimitivesSecondaryKeySignatory extends Enum {
    readonly isIdentity: boolean;
    readonly asIdentity: PolymeshPrimitivesIdentityId;
    readonly isAccount: boolean;
    readonly asAccount: AccountId32;
    readonly type: 'Identity' | 'Account';
  }

  /** @name PalletBridgeRawEvent (91) */
  export interface PalletBridgeRawEvent extends Enum {
    readonly isControllerChanged: boolean;
    readonly asControllerChanged: ITuple<[PolymeshPrimitivesIdentityId, AccountId32]>;
    readonly isAdminChanged: boolean;
    readonly asAdminChanged: ITuple<[PolymeshPrimitivesIdentityId, AccountId32]>;
    readonly isTimelockChanged: boolean;
    readonly asTimelockChanged: ITuple<[PolymeshPrimitivesIdentityId, u32]>;
    readonly isBridged: boolean;
    readonly asBridged: ITuple<[PolymeshPrimitivesIdentityId, PalletBridgeBridgeTx]>;
    readonly isFrozen: boolean;
    readonly asFrozen: PolymeshPrimitivesIdentityId;
    readonly isUnfrozen: boolean;
    readonly asUnfrozen: PolymeshPrimitivesIdentityId;
    readonly isFrozenTx: boolean;
    readonly asFrozenTx: ITuple<[PolymeshPrimitivesIdentityId, PalletBridgeBridgeTx]>;
    readonly isUnfrozenTx: boolean;
    readonly asUnfrozenTx: ITuple<[PolymeshPrimitivesIdentityId, PalletBridgeBridgeTx]>;
    readonly isExemptedUpdated: boolean;
    readonly asExemptedUpdated: ITuple<[PolymeshPrimitivesIdentityId, PolymeshPrimitivesIdentityId, bool]>;
    readonly isBridgeLimitUpdated: boolean;
    readonly asBridgeLimitUpdated: ITuple<[PolymeshPrimitivesIdentityId, u128, u32]>;
    readonly isTxsHandled: boolean;
    readonly asTxsHandled: Vec<ITuple<[AccountId32, u32, PalletBridgeHandledTxStatus]>>;
    readonly isBridgeTxScheduled: boolean;
    readonly asBridgeTxScheduled: ITuple<[PolymeshPrimitivesIdentityId, PalletBridgeBridgeTx, u32]>;
    readonly isBridgeTxScheduleFailed: boolean;
    readonly asBridgeTxScheduleFailed: ITuple<[PolymeshPrimitivesIdentityId, PalletBridgeBridgeTx, Bytes]>;
    readonly isFreezeAdminAdded: boolean;
    readonly asFreezeAdminAdded: ITuple<[PolymeshPrimitivesIdentityId, AccountId32]>;
    readonly isFreezeAdminRemoved: boolean;
    readonly asFreezeAdminRemoved: ITuple<[PolymeshPrimitivesIdentityId, AccountId32]>;
    readonly isTxRemoved: boolean;
    readonly asTxRemoved: ITuple<[PolymeshPrimitivesIdentityId, PalletBridgeBridgeTx]>;
    readonly type: 'ControllerChanged' | 'AdminChanged' | 'TimelockChanged' | 'Bridged' | 'Frozen' | 'Unfrozen' | 'FrozenTx' | 'UnfrozenTx' | 'ExemptedUpdated' | 'BridgeLimitUpdated' | 'TxsHandled' | 'BridgeTxScheduled' | 'BridgeTxScheduleFailed' | 'FreezeAdminAdded' | 'FreezeAdminRemoved' | 'TxRemoved';
  }

  /** @name PalletBridgeBridgeTx (92) */
  export interface PalletBridgeBridgeTx extends Struct {
    readonly nonce: u32;
    readonly recipient: AccountId32;
    readonly amount: u128;
    readonly txHash: H256;
  }

  /** @name PalletBridgeHandledTxStatus (95) */
  export interface PalletBridgeHandledTxStatus extends Enum {
    readonly isSuccess: boolean;
    readonly isError: boolean;
    readonly asError: Bytes;
    readonly type: 'Success' | 'Error';
  }

  /** @name PalletStakingRawEvent (96) */
  export interface PalletStakingRawEvent extends Enum {
    readonly isEraPayout: boolean;
    readonly asEraPayout: ITuple<[u32, u128, u128]>;
    readonly isReward: boolean;
    readonly asReward: ITuple<[PolymeshPrimitivesIdentityId, AccountId32, u128]>;
    readonly isSlash: boolean;
    readonly asSlash: ITuple<[AccountId32, u128]>;
    readonly isOldSlashingReportDiscarded: boolean;
    readonly asOldSlashingReportDiscarded: u32;
    readonly isStakingElection: boolean;
    readonly asStakingElection: PalletStakingElectionCompute;
    readonly isSolutionStored: boolean;
    readonly asSolutionStored: PalletStakingElectionCompute;
    readonly isBonded: boolean;
    readonly asBonded: ITuple<[PolymeshPrimitivesIdentityId, AccountId32, u128]>;
    readonly isUnbonded: boolean;
    readonly asUnbonded: ITuple<[PolymeshPrimitivesIdentityId, AccountId32, u128]>;
    readonly isNominated: boolean;
    readonly asNominated: ITuple<[PolymeshPrimitivesIdentityId, AccountId32, Vec<AccountId32>]>;
    readonly isWithdrawn: boolean;
    readonly asWithdrawn: ITuple<[AccountId32, u128]>;
    readonly isPermissionedIdentityAdded: boolean;
    readonly asPermissionedIdentityAdded: ITuple<[PolymeshPrimitivesIdentityId, PolymeshPrimitivesIdentityId]>;
    readonly isPermissionedIdentityRemoved: boolean;
    readonly asPermissionedIdentityRemoved: ITuple<[PolymeshPrimitivesIdentityId, PolymeshPrimitivesIdentityId]>;
    readonly isInvalidatedNominators: boolean;
    readonly asInvalidatedNominators: ITuple<[PolymeshPrimitivesIdentityId, AccountId32, Vec<AccountId32>]>;
    readonly isCommissionCapUpdated: boolean;
    readonly asCommissionCapUpdated: ITuple<[PolymeshPrimitivesIdentityId, Perbill, Perbill]>;
    readonly isMinimumBondThresholdUpdated: boolean;
    readonly asMinimumBondThresholdUpdated: ITuple<[Option<PolymeshPrimitivesIdentityId>, u128]>;
    readonly isRewardPaymentSchedulingInterrupted: boolean;
    readonly asRewardPaymentSchedulingInterrupted: ITuple<[AccountId32, u32, SpRuntimeDispatchError]>;
    readonly isSlashingAllowedForChanged: boolean;
    readonly asSlashingAllowedForChanged: PalletStakingSlashingSwitch;
    readonly type: 'EraPayout' | 'Reward' | 'Slash' | 'OldSlashingReportDiscarded' | 'StakingElection' | 'SolutionStored' | 'Bonded' | 'Unbonded' | 'Nominated' | 'Withdrawn' | 'PermissionedIdentityAdded' | 'PermissionedIdentityRemoved' | 'InvalidatedNominators' | 'CommissionCapUpdated' | 'MinimumBondThresholdUpdated' | 'RewardPaymentSchedulingInterrupted' | 'SlashingAllowedForChanged';
  }

  /** @name PalletStakingElectionCompute (97) */
  export interface PalletStakingElectionCompute extends Enum {
    readonly isOnChain: boolean;
    readonly isSigned: boolean;
    readonly isUnsigned: boolean;
    readonly type: 'OnChain' | 'Signed' | 'Unsigned';
  }

  /** @name PalletStakingSlashingSwitch (99) */
  export interface PalletStakingSlashingSwitch extends Enum {
    readonly isValidator: boolean;
    readonly isValidatorAndNominator: boolean;
    readonly isNone: boolean;
    readonly type: 'Validator' | 'ValidatorAndNominator' | 'None';
  }

  /** @name PalletOffencesEvent (100) */
  export interface PalletOffencesEvent extends Enum {
    readonly isOffence: boolean;
    readonly asOffence: {
      readonly kind: U8aFixed;
      readonly timeslot: Bytes;
    } & Struct;
    readonly type: 'Offence';
  }

  /** @name PalletSessionEvent (102) */
  export interface PalletSessionEvent extends Enum {
    readonly isNewSession: boolean;
    readonly asNewSession: {
      readonly sessionIndex: u32;
    } & Struct;
    readonly type: 'NewSession';
  }

  /** @name PalletGrandpaEvent (103) */
  export interface PalletGrandpaEvent extends Enum {
    readonly isNewAuthorities: boolean;
    readonly asNewAuthorities: {
      readonly authoritySet: Vec<ITuple<[SpFinalityGrandpaAppPublic, u64]>>;
    } & Struct;
    readonly isPaused: boolean;
    readonly isResumed: boolean;
    readonly type: 'NewAuthorities' | 'Paused' | 'Resumed';
  }

  /** @name SpFinalityGrandpaAppPublic (106) */
  export interface SpFinalityGrandpaAppPublic extends SpCoreEd25519Public {}

  /** @name SpCoreEd25519Public (107) */
  export interface SpCoreEd25519Public extends U8aFixed {}

  /** @name PalletImOnlineEvent (108) */
  export interface PalletImOnlineEvent extends Enum {
    readonly isHeartbeatReceived: boolean;
    readonly asHeartbeatReceived: {
      readonly authorityId: PalletImOnlineSr25519AppSr25519Public;
    } & Struct;
    readonly isAllGood: boolean;
    readonly isSomeOffline: boolean;
    readonly asSomeOffline: {
      readonly offline: Vec<ITuple<[AccountId32, PalletStakingExposure]>>;
    } & Struct;
    readonly type: 'HeartbeatReceived' | 'AllGood' | 'SomeOffline';
  }

  /** @name PalletImOnlineSr25519AppSr25519Public (109) */
  export interface PalletImOnlineSr25519AppSr25519Public extends SpCoreSr25519Public {}

  /** @name SpCoreSr25519Public (110) */
  export interface SpCoreSr25519Public extends U8aFixed {}

  /** @name PalletStakingExposure (113) */
  export interface PalletStakingExposure extends Struct {
    readonly total: Compact<u128>;
    readonly own: Compact<u128>;
    readonly others: Vec<PalletStakingIndividualExposure>;
  }

  /** @name PalletStakingIndividualExposure (116) */
  export interface PalletStakingIndividualExposure extends Struct {
    readonly who: AccountId32;
    readonly value: Compact<u128>;
  }

  /** @name PalletSudoRawEvent (117) */
  export interface PalletSudoRawEvent extends Enum {
    readonly isSudid: boolean;
    readonly asSudid: Result<Null, SpRuntimeDispatchError>;
    readonly isKeyChanged: boolean;
    readonly asKeyChanged: AccountId32;
    readonly isSudoAsDone: boolean;
    readonly asSudoAsDone: Result<Null, SpRuntimeDispatchError>;
    readonly type: 'Sudid' | 'KeyChanged' | 'SudoAsDone';
  }

  /** @name PolymeshCommonUtilitiesAssetRawEvent (118) */
  export interface PolymeshCommonUtilitiesAssetRawEvent extends Enum {
    readonly isTransfer: boolean;
    readonly asTransfer: ITuple<[PolymeshPrimitivesIdentityId, PolymeshPrimitivesTicker, PolymeshPrimitivesIdentityIdPortfolioId, PolymeshPrimitivesIdentityIdPortfolioId, u128]>;
    readonly isIssued: boolean;
    readonly asIssued: ITuple<[PolymeshPrimitivesIdentityId, PolymeshPrimitivesTicker, PolymeshPrimitivesIdentityId, u128, Bytes, u128]>;
    readonly isRedeemed: boolean;
    readonly asRedeemed: ITuple<[PolymeshPrimitivesIdentityId, PolymeshPrimitivesTicker, PolymeshPrimitivesIdentityId, u128]>;
    readonly isAssetCreated: boolean;
    readonly asAssetCreated: ITuple<[PolymeshPrimitivesIdentityId, PolymeshPrimitivesTicker, bool, PolymeshPrimitivesAssetAssetType, PolymeshPrimitivesIdentityId, bool]>;
    readonly isIdentifiersUpdated: boolean;
    readonly asIdentifiersUpdated: ITuple<[PolymeshPrimitivesIdentityId, PolymeshPrimitivesTicker, Vec<PolymeshPrimitivesAssetIdentifier>]>;
    readonly isDivisibilityChanged: boolean;
    readonly asDivisibilityChanged: ITuple<[PolymeshPrimitivesIdentityId, PolymeshPrimitivesTicker, bool]>;
    readonly isTransferWithData: boolean;
    readonly asTransferWithData: ITuple<[PolymeshPrimitivesIdentityId, PolymeshPrimitivesTicker, PolymeshPrimitivesIdentityId, PolymeshPrimitivesIdentityId, u128, Bytes]>;
    readonly isIsIssuable: boolean;
    readonly asIsIssuable: ITuple<[PolymeshPrimitivesTicker, bool]>;
    readonly isTickerRegistered: boolean;
    readonly asTickerRegistered: ITuple<[PolymeshPrimitivesIdentityId, PolymeshPrimitivesTicker, Option<u64>]>;
    readonly isTickerTransferred: boolean;
    readonly asTickerTransferred: ITuple<[PolymeshPrimitivesIdentityId, PolymeshPrimitivesTicker, PolymeshPrimitivesIdentityId]>;
    readonly isAssetOwnershipTransferred: boolean;
    readonly asAssetOwnershipTransferred: ITuple<[PolymeshPrimitivesIdentityId, PolymeshPrimitivesTicker, PolymeshPrimitivesIdentityId]>;
    readonly isAssetFrozen: boolean;
    readonly asAssetFrozen: ITuple<[PolymeshPrimitivesIdentityId, PolymeshPrimitivesTicker]>;
    readonly isAssetUnfrozen: boolean;
    readonly asAssetUnfrozen: ITuple<[PolymeshPrimitivesIdentityId, PolymeshPrimitivesTicker]>;
    readonly isAssetRenamed: boolean;
    readonly asAssetRenamed: ITuple<[PolymeshPrimitivesIdentityId, PolymeshPrimitivesTicker, Bytes]>;
    readonly isFundingRoundSet: boolean;
    readonly asFundingRoundSet: ITuple<[PolymeshPrimitivesIdentityId, PolymeshPrimitivesTicker, Bytes]>;
    readonly isDocumentAdded: boolean;
    readonly asDocumentAdded: ITuple<[PolymeshPrimitivesIdentityId, PolymeshPrimitivesTicker, u32, PolymeshPrimitivesDocument]>;
    readonly isDocumentRemoved: boolean;
    readonly asDocumentRemoved: ITuple<[PolymeshPrimitivesIdentityId, PolymeshPrimitivesTicker, u32]>;
    readonly isExtensionRemoved: boolean;
    readonly asExtensionRemoved: ITuple<[PolymeshPrimitivesIdentityId, PolymeshPrimitivesTicker, AccountId32]>;
    readonly isClassicTickerClaimed: boolean;
    readonly asClassicTickerClaimed: ITuple<[PolymeshPrimitivesIdentityId, PolymeshPrimitivesTicker, PolymeshPrimitivesEthereumEthereumAddress]>;
    readonly isControllerTransfer: boolean;
    readonly asControllerTransfer: ITuple<[PolymeshPrimitivesIdentityId, PolymeshPrimitivesTicker, PolymeshPrimitivesIdentityIdPortfolioId, u128]>;
    readonly isCustomAssetTypeExists: boolean;
    readonly asCustomAssetTypeExists: ITuple<[PolymeshPrimitivesIdentityId, u32, Bytes]>;
    readonly isCustomAssetTypeRegistered: boolean;
    readonly asCustomAssetTypeRegistered: ITuple<[PolymeshPrimitivesIdentityId, u32, Bytes]>;
    readonly isSetAssetMetadataValue: boolean;
    readonly asSetAssetMetadataValue: ITuple<[PolymeshPrimitivesIdentityId, PolymeshPrimitivesTicker, Bytes, Option<PolymeshPrimitivesAssetMetadataAssetMetadataValueDetail>]>;
    readonly isSetAssetMetadataValueDetails: boolean;
    readonly asSetAssetMetadataValueDetails: ITuple<[PolymeshPrimitivesIdentityId, PolymeshPrimitivesTicker, PolymeshPrimitivesAssetMetadataAssetMetadataValueDetail]>;
    readonly isRegisterAssetMetadataLocalType: boolean;
    readonly asRegisterAssetMetadataLocalType: ITuple<[PolymeshPrimitivesIdentityId, PolymeshPrimitivesTicker, Bytes, u64, PolymeshPrimitivesAssetMetadataAssetMetadataSpec]>;
    readonly isRegisterAssetMetadataGlobalType: boolean;
    readonly asRegisterAssetMetadataGlobalType: ITuple<[Bytes, u64, PolymeshPrimitivesAssetMetadataAssetMetadataSpec]>;
    readonly type: 'Transfer' | 'Issued' | 'Redeemed' | 'AssetCreated' | 'IdentifiersUpdated' | 'DivisibilityChanged' | 'TransferWithData' | 'IsIssuable' | 'TickerRegistered' | 'TickerTransferred' | 'AssetOwnershipTransferred' | 'AssetFrozen' | 'AssetUnfrozen' | 'AssetRenamed' | 'FundingRoundSet' | 'DocumentAdded' | 'DocumentRemoved' | 'ExtensionRemoved' | 'ClassicTickerClaimed' | 'ControllerTransfer' | 'CustomAssetTypeExists' | 'CustomAssetTypeRegistered' | 'SetAssetMetadataValue' | 'SetAssetMetadataValueDetails' | 'RegisterAssetMetadataLocalType' | 'RegisterAssetMetadataGlobalType';
  }

  /** @name PolymeshPrimitivesAssetAssetType (120) */
  export interface PolymeshPrimitivesAssetAssetType extends Enum {
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
    readonly type: 'EquityCommon' | 'EquityPreferred' | 'Commodity' | 'FixedIncome' | 'Reit' | 'Fund' | 'RevenueShareAgreement' | 'StructuredProduct' | 'Derivative' | 'Custom' | 'StableCoin';
  }

  /** @name PolymeshPrimitivesAssetIdentifier (123) */
  export interface PolymeshPrimitivesAssetIdentifier extends Enum {
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

  /** @name PolymeshPrimitivesDocument (128) */
  export interface PolymeshPrimitivesDocument extends Struct {
    readonly uri: Bytes;
    readonly contentHash: PolymeshPrimitivesDocumentHash;
    readonly name: Bytes;
    readonly docType: Option<Bytes>;
    readonly filingDate: Option<u64>;
  }

  /** @name PolymeshPrimitivesDocumentHash (130) */
  export interface PolymeshPrimitivesDocumentHash extends Enum {
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

  /** @name PolymeshPrimitivesEthereumEthereumAddress (139) */
  export interface PolymeshPrimitivesEthereumEthereumAddress extends U8aFixed {}

  /** @name PolymeshPrimitivesAssetMetadataAssetMetadataValueDetail (142) */
  export interface PolymeshPrimitivesAssetMetadataAssetMetadataValueDetail extends Struct {
    readonly expire: Option<u64>;
    readonly lockStatus: PolymeshPrimitivesAssetMetadataAssetMetadataLockStatus;
  }

  /** @name PolymeshPrimitivesAssetMetadataAssetMetadataLockStatus (143) */
  export interface PolymeshPrimitivesAssetMetadataAssetMetadataLockStatus extends Enum {
    readonly isUnlocked: boolean;
    readonly isLocked: boolean;
    readonly isLockedUntil: boolean;
    readonly asLockedUntil: u64;
    readonly type: 'Unlocked' | 'Locked' | 'LockedUntil';
  }

  /** @name PolymeshPrimitivesAssetMetadataAssetMetadataSpec (146) */
  export interface PolymeshPrimitivesAssetMetadataAssetMetadataSpec extends Struct {
    readonly url: Option<Bytes>;
    readonly description: Option<Bytes>;
    readonly typeDef: Option<Bytes>;
  }

  /** @name PalletCorporateActionsDistributionEvent (153) */
  export interface PalletCorporateActionsDistributionEvent extends Enum {
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

  /** @name PolymeshPrimitivesEventOnly (154) */
  export interface PolymeshPrimitivesEventOnly extends PolymeshPrimitivesIdentityId {}

  /** @name PalletCorporateActionsCaId (155) */
  export interface PalletCorporateActionsCaId extends Struct {
    readonly ticker: PolymeshPrimitivesTicker;
    readonly localId: u32;
  }

  /** @name PalletCorporateActionsDistribution (157) */
  export interface PalletCorporateActionsDistribution extends Struct {
    readonly from: PolymeshPrimitivesIdentityIdPortfolioId;
    readonly currency: PolymeshPrimitivesTicker;
    readonly perShare: u128;
    readonly amount: u128;
    readonly remaining: u128;
    readonly reclaimed: bool;
    readonly paymentAt: u64;
    readonly expiresAt: Option<u64>;
  }

  /** @name PolymeshCommonUtilitiesCheckpointEvent (159) */
  export interface PolymeshCommonUtilitiesCheckpointEvent extends Enum {
    readonly isCheckpointCreated: boolean;
    readonly asCheckpointCreated: ITuple<[Option<PolymeshPrimitivesEventOnly>, PolymeshPrimitivesTicker, u64, u128, u64]>;
    readonly isMaximumSchedulesComplexityChanged: boolean;
    readonly asMaximumSchedulesComplexityChanged: ITuple<[PolymeshPrimitivesIdentityId, u64]>;
    readonly isScheduleCreated: boolean;
    readonly asScheduleCreated: ITuple<[PolymeshPrimitivesEventOnly, PolymeshPrimitivesTicker, PolymeshCommonUtilitiesCheckpointStoredSchedule]>;
    readonly isScheduleRemoved: boolean;
    readonly asScheduleRemoved: ITuple<[PolymeshPrimitivesIdentityId, PolymeshPrimitivesTicker, PolymeshCommonUtilitiesCheckpointStoredSchedule]>;
    readonly type: 'CheckpointCreated' | 'MaximumSchedulesComplexityChanged' | 'ScheduleCreated' | 'ScheduleRemoved';
  }

  /** @name PolymeshCommonUtilitiesCheckpointStoredSchedule (162) */
  export interface PolymeshCommonUtilitiesCheckpointStoredSchedule extends Struct {
    readonly schedule: PolymeshPrimitivesCalendarCheckpointSchedule;
    readonly id: u64;
    readonly at: u64;
    readonly remaining: u32;
  }

  /** @name PolymeshPrimitivesCalendarCheckpointSchedule (163) */
  export interface PolymeshPrimitivesCalendarCheckpointSchedule extends Struct {
    readonly start: u64;
    readonly period: PolymeshPrimitivesCalendarCalendarPeriod;
  }

  /** @name PolymeshPrimitivesCalendarCalendarPeriod (164) */
  export interface PolymeshPrimitivesCalendarCalendarPeriod extends Struct {
    readonly unit: PolymeshPrimitivesCalendarCalendarUnit;
    readonly amount: u64;
  }

  /** @name PolymeshPrimitivesCalendarCalendarUnit (165) */
  export interface PolymeshPrimitivesCalendarCalendarUnit extends Enum {
    readonly isSecond: boolean;
    readonly isMinute: boolean;
    readonly isHour: boolean;
    readonly isDay: boolean;
    readonly isWeek: boolean;
    readonly isMonth: boolean;
    readonly isYear: boolean;
    readonly type: 'Second' | 'Minute' | 'Hour' | 'Day' | 'Week' | 'Month' | 'Year';
  }

  /** @name PalletComplianceManagerEvent (167) */
  export interface PalletComplianceManagerEvent extends Enum {
    readonly isComplianceRequirementCreated: boolean;
    readonly asComplianceRequirementCreated: ITuple<[PolymeshPrimitivesIdentityId, PolymeshPrimitivesTicker, PolymeshPrimitivesComplianceManagerComplianceRequirement]>;
    readonly isComplianceRequirementRemoved: boolean;
    readonly asComplianceRequirementRemoved: ITuple<[PolymeshPrimitivesIdentityId, PolymeshPrimitivesTicker, u32]>;
    readonly isAssetComplianceReplaced: boolean;
    readonly asAssetComplianceReplaced: ITuple<[PolymeshPrimitivesIdentityId, PolymeshPrimitivesTicker, Vec<PolymeshPrimitivesComplianceManagerComplianceRequirement>]>;
    readonly isAssetComplianceReset: boolean;
    readonly asAssetComplianceReset: ITuple<[PolymeshPrimitivesIdentityId, PolymeshPrimitivesTicker]>;
    readonly isAssetComplianceResumed: boolean;
    readonly asAssetComplianceResumed: ITuple<[PolymeshPrimitivesIdentityId, PolymeshPrimitivesTicker]>;
    readonly isAssetCompliancePaused: boolean;
    readonly asAssetCompliancePaused: ITuple<[PolymeshPrimitivesIdentityId, PolymeshPrimitivesTicker]>;
    readonly isComplianceRequirementChanged: boolean;
    readonly asComplianceRequirementChanged: ITuple<[PolymeshPrimitivesIdentityId, PolymeshPrimitivesTicker, PolymeshPrimitivesComplianceManagerComplianceRequirement]>;
    readonly isTrustedDefaultClaimIssuerAdded: boolean;
    readonly asTrustedDefaultClaimIssuerAdded: ITuple<[PolymeshPrimitivesIdentityId, PolymeshPrimitivesTicker, PolymeshPrimitivesConditionTrustedIssuer]>;
    readonly isTrustedDefaultClaimIssuerRemoved: boolean;
    readonly asTrustedDefaultClaimIssuerRemoved: ITuple<[PolymeshPrimitivesIdentityId, PolymeshPrimitivesTicker, PolymeshPrimitivesIdentityId]>;
    readonly type: 'ComplianceRequirementCreated' | 'ComplianceRequirementRemoved' | 'AssetComplianceReplaced' | 'AssetComplianceReset' | 'AssetComplianceResumed' | 'AssetCompliancePaused' | 'ComplianceRequirementChanged' | 'TrustedDefaultClaimIssuerAdded' | 'TrustedDefaultClaimIssuerRemoved';
  }

  /** @name PolymeshPrimitivesComplianceManagerComplianceRequirement (168) */
  export interface PolymeshPrimitivesComplianceManagerComplianceRequirement extends Struct {
    readonly senderConditions: Vec<PolymeshPrimitivesCondition>;
    readonly receiverConditions: Vec<PolymeshPrimitivesCondition>;
    readonly id: u32;
  }

  /** @name PolymeshPrimitivesCondition (170) */
  export interface PolymeshPrimitivesCondition extends Struct {
    readonly conditionType: PolymeshPrimitivesConditionConditionType;
    readonly issuers: Vec<PolymeshPrimitivesConditionTrustedIssuer>;
  }

  /** @name PolymeshPrimitivesConditionConditionType (171) */
  export interface PolymeshPrimitivesConditionConditionType extends Enum {
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

  /** @name PolymeshPrimitivesConditionTargetIdentity (173) */
  export interface PolymeshPrimitivesConditionTargetIdentity extends Enum {
    readonly isExternalAgent: boolean;
    readonly isSpecific: boolean;
    readonly asSpecific: PolymeshPrimitivesIdentityId;
    readonly type: 'ExternalAgent' | 'Specific';
  }

  /** @name PolymeshPrimitivesConditionTrustedIssuer (175) */
  export interface PolymeshPrimitivesConditionTrustedIssuer extends Struct {
    readonly issuer: PolymeshPrimitivesIdentityId;
    readonly trustedFor: PolymeshPrimitivesConditionTrustedFor;
  }

  /** @name PolymeshPrimitivesConditionTrustedFor (176) */
  export interface PolymeshPrimitivesConditionTrustedFor extends Enum {
    readonly isAny: boolean;
    readonly isSpecific: boolean;
    readonly asSpecific: Vec<PolymeshPrimitivesIdentityClaimClaimType>;
    readonly type: 'Any' | 'Specific';
  }

  /** @name PolymeshPrimitivesIdentityClaimClaimType (178) */
  export interface PolymeshPrimitivesIdentityClaimClaimType extends Enum {
    readonly isAccredited: boolean;
    readonly isAffiliate: boolean;
    readonly isBuyLockup: boolean;
    readonly isSellLockup: boolean;
    readonly isCustomerDueDiligence: boolean;
    readonly isKnowYourCustomer: boolean;
    readonly isJurisdiction: boolean;
    readonly isExempted: boolean;
    readonly isBlocked: boolean;
    readonly isInvestorUniqueness: boolean;
    readonly isNoType: boolean;
    readonly isInvestorUniquenessV2: boolean;
    readonly type: 'Accredited' | 'Affiliate' | 'BuyLockup' | 'SellLockup' | 'CustomerDueDiligence' | 'KnowYourCustomer' | 'Jurisdiction' | 'Exempted' | 'Blocked' | 'InvestorUniqueness' | 'NoType' | 'InvestorUniquenessV2';
  }

  /** @name PalletCorporateActionsEvent (180) */
  export interface PalletCorporateActionsEvent extends Enum {
    readonly isMaxDetailsLengthChanged: boolean;
    readonly asMaxDetailsLengthChanged: ITuple<[PolymeshPrimitivesIdentityId, u32]>;
    readonly isDefaultTargetIdentitiesChanged: boolean;
    readonly asDefaultTargetIdentitiesChanged: ITuple<[PolymeshPrimitivesIdentityId, PolymeshPrimitivesTicker, PalletCorporateActionsTargetIdentities]>;
    readonly isDefaultWithholdingTaxChanged: boolean;
    readonly asDefaultWithholdingTaxChanged: ITuple<[PolymeshPrimitivesIdentityId, PolymeshPrimitivesTicker, Permill]>;
    readonly isDidWithholdingTaxChanged: boolean;
    readonly asDidWithholdingTaxChanged: ITuple<[PolymeshPrimitivesIdentityId, PolymeshPrimitivesTicker, PolymeshPrimitivesIdentityId, Option<Permill>]>;
    readonly isCaaTransferred: boolean;
    readonly asCaaTransferred: ITuple<[PolymeshPrimitivesIdentityId, PolymeshPrimitivesTicker, PolymeshPrimitivesIdentityId]>;
    readonly isCaInitiated: boolean;
    readonly asCaInitiated: ITuple<[PolymeshPrimitivesEventOnly, PalletCorporateActionsCaId, PalletCorporateActionsCorporateAction, Bytes]>;
    readonly isCaLinkedToDoc: boolean;
    readonly asCaLinkedToDoc: ITuple<[PolymeshPrimitivesIdentityId, PalletCorporateActionsCaId, Vec<u32>]>;
    readonly isCaRemoved: boolean;
    readonly asCaRemoved: ITuple<[PolymeshPrimitivesEventOnly, PalletCorporateActionsCaId]>;
    readonly isRecordDateChanged: boolean;
    readonly asRecordDateChanged: ITuple<[PolymeshPrimitivesEventOnly, PalletCorporateActionsCaId, PalletCorporateActionsCorporateAction]>;
    readonly type: 'MaxDetailsLengthChanged' | 'DefaultTargetIdentitiesChanged' | 'DefaultWithholdingTaxChanged' | 'DidWithholdingTaxChanged' | 'CaaTransferred' | 'CaInitiated' | 'CaLinkedToDoc' | 'CaRemoved' | 'RecordDateChanged';
  }

  /** @name PalletCorporateActionsTargetIdentities (181) */
  export interface PalletCorporateActionsTargetIdentities extends Struct {
    readonly identities: Vec<PolymeshPrimitivesIdentityId>;
    readonly treatment: PalletCorporateActionsTargetTreatment;
  }

  /** @name PalletCorporateActionsTargetTreatment (182) */
  export interface PalletCorporateActionsTargetTreatment extends Enum {
    readonly isInclude: boolean;
    readonly isExclude: boolean;
    readonly type: 'Include' | 'Exclude';
  }

  /** @name PalletCorporateActionsCorporateAction (184) */
  export interface PalletCorporateActionsCorporateAction extends Struct {
    readonly kind: PalletCorporateActionsCaKind;
    readonly declDate: u64;
    readonly recordDate: Option<PalletCorporateActionsRecordDate>;
    readonly targets: PalletCorporateActionsTargetIdentities;
    readonly defaultWithholdingTax: Permill;
    readonly withholdingTax: Vec<ITuple<[PolymeshPrimitivesIdentityId, Permill]>>;
  }

  /** @name PalletCorporateActionsCaKind (185) */
  export interface PalletCorporateActionsCaKind extends Enum {
    readonly isPredictableBenefit: boolean;
    readonly isUnpredictableBenefit: boolean;
    readonly isIssuerNotice: boolean;
    readonly isReorganization: boolean;
    readonly isOther: boolean;
    readonly type: 'PredictableBenefit' | 'UnpredictableBenefit' | 'IssuerNotice' | 'Reorganization' | 'Other';
  }

  /** @name PalletCorporateActionsRecordDate (187) */
  export interface PalletCorporateActionsRecordDate extends Struct {
    readonly date: u64;
    readonly checkpoint: PalletCorporateActionsCaCheckpoint;
  }

  /** @name PalletCorporateActionsCaCheckpoint (188) */
  export interface PalletCorporateActionsCaCheckpoint extends Enum {
    readonly isScheduled: boolean;
    readonly asScheduled: ITuple<[u64, u64]>;
    readonly isExisting: boolean;
    readonly asExisting: u64;
    readonly type: 'Scheduled' | 'Existing';
  }

  /** @name PalletCorporateActionsBallotEvent (193) */
  export interface PalletCorporateActionsBallotEvent extends Enum {
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

  /** @name PalletCorporateActionsBallotBallotTimeRange (194) */
  export interface PalletCorporateActionsBallotBallotTimeRange extends Struct {
    readonly start: u64;
    readonly end: u64;
  }

  /** @name PalletCorporateActionsBallotBallotMeta (195) */
  export interface PalletCorporateActionsBallotBallotMeta extends Struct {
    readonly title: Bytes;
    readonly motions: Vec<PalletCorporateActionsBallotMotion>;
  }

  /** @name PalletCorporateActionsBallotMotion (198) */
  export interface PalletCorporateActionsBallotMotion extends Struct {
    readonly title: Bytes;
    readonly infoLink: Bytes;
    readonly choices: Vec<Bytes>;
  }

  /** @name PalletCorporateActionsBallotBallotVote (204) */
  export interface PalletCorporateActionsBallotBallotVote extends Struct {
    readonly power: u128;
    readonly fallback: Option<u16>;
  }

  /** @name PalletPipsRawEvent (207) */
  export interface PalletPipsRawEvent extends Enum {
    readonly isHistoricalPipsPruned: boolean;
    readonly asHistoricalPipsPruned: ITuple<[PolymeshPrimitivesIdentityId, bool, bool]>;
    readonly isProposalCreated: boolean;
    readonly asProposalCreated: ITuple<[PolymeshPrimitivesIdentityId, PalletPipsProposer, u32, u128, Option<Bytes>, Option<Bytes>, PolymeshCommonUtilitiesMaybeBlock, PalletPipsProposalData]>;
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
    readonly asPendingPipExpiryChanged: ITuple<[PolymeshPrimitivesIdentityId, PolymeshCommonUtilitiesMaybeBlock, PolymeshCommonUtilitiesMaybeBlock]>;
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

  /** @name PalletPipsProposer (208) */
  export interface PalletPipsProposer extends Enum {
    readonly isCommunity: boolean;
    readonly asCommunity: AccountId32;
    readonly isCommittee: boolean;
    readonly asCommittee: PalletPipsCommittee;
    readonly type: 'Community' | 'Committee';
  }

  /** @name PalletPipsCommittee (209) */
  export interface PalletPipsCommittee extends Enum {
    readonly isTechnical: boolean;
    readonly isUpgrade: boolean;
    readonly type: 'Technical' | 'Upgrade';
  }

  /** @name PalletPipsProposalData (213) */
  export interface PalletPipsProposalData extends Enum {
    readonly isHash: boolean;
    readonly asHash: H256;
    readonly isProposal: boolean;
    readonly asProposal: Bytes;
    readonly type: 'Hash' | 'Proposal';
  }

  /** @name PalletPipsProposalState (214) */
  export interface PalletPipsProposalState extends Enum {
    readonly isPending: boolean;
    readonly isRejected: boolean;
    readonly isScheduled: boolean;
    readonly isFailed: boolean;
    readonly isExecuted: boolean;
    readonly isExpired: boolean;
    readonly type: 'Pending' | 'Rejected' | 'Scheduled' | 'Failed' | 'Executed' | 'Expired';
  }

  /** @name PalletPipsSnapshottedPip (217) */
  export interface PalletPipsSnapshottedPip extends Struct {
    readonly id: u32;
    readonly weight: ITuple<[bool, u128]>;
  }

  /** @name PolymeshCommonUtilitiesPortfolioEvent (223) */
  export interface PolymeshCommonUtilitiesPortfolioEvent extends Enum {
    readonly isPortfolioCreated: boolean;
    readonly asPortfolioCreated: ITuple<[PolymeshPrimitivesIdentityId, u64, Bytes]>;
    readonly isPortfolioDeleted: boolean;
    readonly asPortfolioDeleted: ITuple<[PolymeshPrimitivesIdentityId, u64]>;
    readonly isMovedBetweenPortfolios: boolean;
    readonly asMovedBetweenPortfolios: ITuple<[PolymeshPrimitivesIdentityId, PolymeshPrimitivesIdentityIdPortfolioId, PolymeshPrimitivesIdentityIdPortfolioId, PolymeshPrimitivesTicker, u128, Option<PolymeshCommonUtilitiesBalancesMemo>]>;
    readonly isPortfolioRenamed: boolean;
    readonly asPortfolioRenamed: ITuple<[PolymeshPrimitivesIdentityId, u64, Bytes]>;
    readonly isUserPortfolios: boolean;
    readonly asUserPortfolios: ITuple<[PolymeshPrimitivesIdentityId, Vec<ITuple<[u64, Bytes]>>]>;
    readonly isPortfolioCustodianChanged: boolean;
    readonly asPortfolioCustodianChanged: ITuple<[PolymeshPrimitivesIdentityId, PolymeshPrimitivesIdentityIdPortfolioId, PolymeshPrimitivesIdentityId]>;
    readonly type: 'PortfolioCreated' | 'PortfolioDeleted' | 'MovedBetweenPortfolios' | 'PortfolioRenamed' | 'UserPortfolios' | 'PortfolioCustodianChanged';
  }

  /** @name PalletProtocolFeeRawEvent (227) */
  export interface PalletProtocolFeeRawEvent extends Enum {
    readonly isFeeSet: boolean;
    readonly asFeeSet: ITuple<[PolymeshPrimitivesIdentityId, u128]>;
    readonly isCoefficientSet: boolean;
    readonly asCoefficientSet: ITuple<[PolymeshPrimitivesIdentityId, PolymeshPrimitivesPosRatio]>;
    readonly isFeeCharged: boolean;
    readonly asFeeCharged: ITuple<[AccountId32, u128]>;
    readonly type: 'FeeSet' | 'CoefficientSet' | 'FeeCharged';
  }

  /** @name PolymeshPrimitivesPosRatio (228) */
  export interface PolymeshPrimitivesPosRatio extends ITuple<[u32, u32]> {}

  /** @name PalletSchedulerEvent (229) */
  export interface PalletSchedulerEvent extends Enum {
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
      readonly id: Option<Bytes>;
      readonly result: Result<Null, SpRuntimeDispatchError>;
    } & Struct;
    readonly isCallLookupFailed: boolean;
    readonly asCallLookupFailed: {
      readonly task: ITuple<[u32, u32]>;
      readonly id: Option<Bytes>;
      readonly error: FrameSupportScheduleLookupError;
    } & Struct;
    readonly type: 'Scheduled' | 'Canceled' | 'Dispatched' | 'CallLookupFailed';
  }

  /** @name FrameSupportScheduleLookupError (231) */
  export interface FrameSupportScheduleLookupError extends Enum {
    readonly isUnknown: boolean;
    readonly isBadFormat: boolean;
    readonly type: 'Unknown' | 'BadFormat';
  }

  /** @name PalletSettlementRawEvent (232) */
  export interface PalletSettlementRawEvent extends Enum {
    readonly isVenueCreated: boolean;
    readonly asVenueCreated: ITuple<[PolymeshPrimitivesIdentityId, u64, Bytes, PalletSettlementVenueType]>;
    readonly isVenueDetailsUpdated: boolean;
    readonly asVenueDetailsUpdated: ITuple<[PolymeshPrimitivesIdentityId, u64, Bytes]>;
    readonly isVenueTypeUpdated: boolean;
    readonly asVenueTypeUpdated: ITuple<[PolymeshPrimitivesIdentityId, u64, PalletSettlementVenueType]>;
    readonly isInstructionCreated: boolean;
    readonly asInstructionCreated: ITuple<[PolymeshPrimitivesIdentityId, u64, u64, PalletSettlementSettlementType, Option<u64>, Option<u64>, Vec<PalletSettlementLeg>]>;
    readonly isInstructionAffirmed: boolean;
    readonly asInstructionAffirmed: ITuple<[PolymeshPrimitivesIdentityId, PolymeshPrimitivesIdentityIdPortfolioId, u64]>;
    readonly isAffirmationWithdrawn: boolean;
    readonly asAffirmationWithdrawn: ITuple<[PolymeshPrimitivesIdentityId, PolymeshPrimitivesIdentityIdPortfolioId, u64]>;
    readonly isInstructionRejected: boolean;
    readonly asInstructionRejected: ITuple<[PolymeshPrimitivesIdentityId, u64]>;
    readonly isReceiptClaimed: boolean;
    readonly asReceiptClaimed: ITuple<[PolymeshPrimitivesIdentityId, u64, u64, u64, AccountId32, Bytes]>;
    readonly isReceiptValidityChanged: boolean;
    readonly asReceiptValidityChanged: ITuple<[PolymeshPrimitivesIdentityId, AccountId32, u64, bool]>;
    readonly isReceiptUnclaimed: boolean;
    readonly asReceiptUnclaimed: ITuple<[PolymeshPrimitivesIdentityId, u64, u64, u64, AccountId32]>;
    readonly isVenueFiltering: boolean;
    readonly asVenueFiltering: ITuple<[PolymeshPrimitivesIdentityId, PolymeshPrimitivesTicker, bool]>;
    readonly isVenuesAllowed: boolean;
    readonly asVenuesAllowed: ITuple<[PolymeshPrimitivesIdentityId, PolymeshPrimitivesTicker, Vec<u64>]>;
    readonly isVenuesBlocked: boolean;
    readonly asVenuesBlocked: ITuple<[PolymeshPrimitivesIdentityId, PolymeshPrimitivesTicker, Vec<u64>]>;
    readonly isLegFailedExecution: boolean;
    readonly asLegFailedExecution: ITuple<[PolymeshPrimitivesIdentityId, u64, u64]>;
    readonly isInstructionFailed: boolean;
    readonly asInstructionFailed: ITuple<[PolymeshPrimitivesIdentityId, u64]>;
    readonly isInstructionExecuted: boolean;
    readonly asInstructionExecuted: ITuple<[PolymeshPrimitivesIdentityId, u64]>;
    readonly isVenueUnauthorized: boolean;
    readonly asVenueUnauthorized: ITuple<[PolymeshPrimitivesIdentityId, PolymeshPrimitivesTicker, u64]>;
    readonly isSchedulingFailed: boolean;
    readonly asSchedulingFailed: SpRuntimeDispatchError;
    readonly isInstructionRescheduled: boolean;
    readonly asInstructionRescheduled: ITuple<[PolymeshPrimitivesIdentityId, u64]>;
    readonly type: 'VenueCreated' | 'VenueDetailsUpdated' | 'VenueTypeUpdated' | 'InstructionCreated' | 'InstructionAffirmed' | 'AffirmationWithdrawn' | 'InstructionRejected' | 'ReceiptClaimed' | 'ReceiptValidityChanged' | 'ReceiptUnclaimed' | 'VenueFiltering' | 'VenuesAllowed' | 'VenuesBlocked' | 'LegFailedExecution' | 'InstructionFailed' | 'InstructionExecuted' | 'VenueUnauthorized' | 'SchedulingFailed' | 'InstructionRescheduled';
  }

  /** @name PalletSettlementVenueType (235) */
  export interface PalletSettlementVenueType extends Enum {
    readonly isOther: boolean;
    readonly isDistribution: boolean;
    readonly isSto: boolean;
    readonly isExchange: boolean;
    readonly type: 'Other' | 'Distribution' | 'Sto' | 'Exchange';
  }

  /** @name PalletSettlementSettlementType (237) */
  export interface PalletSettlementSettlementType extends Enum {
    readonly isSettleOnAffirmation: boolean;
    readonly isSettleOnBlock: boolean;
    readonly asSettleOnBlock: u32;
    readonly type: 'SettleOnAffirmation' | 'SettleOnBlock';
  }

  /** @name PalletSettlementLeg (239) */
  export interface PalletSettlementLeg extends Struct {
    readonly from: PolymeshPrimitivesIdentityIdPortfolioId;
    readonly to: PolymeshPrimitivesIdentityIdPortfolioId;
    readonly asset: PolymeshPrimitivesTicker;
    readonly amount: u128;
  }

  /** @name PolymeshCommonUtilitiesStatisticsEvent (243) */
  export interface PolymeshCommonUtilitiesStatisticsEvent extends Enum {
    readonly isStatTypesAdded: boolean;
    readonly asStatTypesAdded: ITuple<[PolymeshPrimitivesIdentityId, PolymeshPrimitivesStatisticsAssetScope, Vec<PolymeshPrimitivesStatisticsStatType>]>;
    readonly isStatTypesRemoved: boolean;
    readonly asStatTypesRemoved: ITuple<[PolymeshPrimitivesIdentityId, PolymeshPrimitivesStatisticsAssetScope, Vec<PolymeshPrimitivesStatisticsStatType>]>;
    readonly isAssetStatsUpdated: boolean;
    readonly asAssetStatsUpdated: ITuple<[PolymeshPrimitivesIdentityId, PolymeshPrimitivesStatisticsAssetScope, PolymeshPrimitivesStatisticsStatType, Vec<PolymeshPrimitivesStatisticsStatUpdate>]>;
    readonly isSetAssetTransferCompliance: boolean;
    readonly asSetAssetTransferCompliance: ITuple<[PolymeshPrimitivesIdentityId, PolymeshPrimitivesStatisticsAssetScope, Vec<PolymeshPrimitivesTransferComplianceTransferCondition>]>;
    readonly isTransferConditionExemptionsAdded: boolean;
    readonly asTransferConditionExemptionsAdded: ITuple<[PolymeshPrimitivesIdentityId, PolymeshPrimitivesTransferComplianceTransferConditionExemptKey, Vec<PolymeshPrimitivesIdentityId>]>;
    readonly isTransferConditionExemptionsRemoved: boolean;
    readonly asTransferConditionExemptionsRemoved: ITuple<[PolymeshPrimitivesIdentityId, PolymeshPrimitivesTransferComplianceTransferConditionExemptKey, Vec<PolymeshPrimitivesIdentityId>]>;
    readonly type: 'StatTypesAdded' | 'StatTypesRemoved' | 'AssetStatsUpdated' | 'SetAssetTransferCompliance' | 'TransferConditionExemptionsAdded' | 'TransferConditionExemptionsRemoved';
  }

  /** @name PolymeshPrimitivesStatisticsAssetScope (244) */
  export interface PolymeshPrimitivesStatisticsAssetScope extends Enum {
    readonly isTicker: boolean;
    readonly asTicker: PolymeshPrimitivesTicker;
    readonly type: 'Ticker';
  }

  /** @name PolymeshPrimitivesStatisticsStatType (246) */
  export interface PolymeshPrimitivesStatisticsStatType extends Struct {
    readonly op: PolymeshPrimitivesStatisticsStatOpType;
    readonly claimIssuer: Option<ITuple<[PolymeshPrimitivesIdentityClaimClaimType, PolymeshPrimitivesIdentityId]>>;
  }

  /** @name PolymeshPrimitivesStatisticsStatOpType (247) */
  export interface PolymeshPrimitivesStatisticsStatOpType extends Enum {
    readonly isCount: boolean;
    readonly isBalance: boolean;
    readonly type: 'Count' | 'Balance';
  }

  /** @name PolymeshPrimitivesStatisticsStatUpdate (251) */
  export interface PolymeshPrimitivesStatisticsStatUpdate extends Struct {
    readonly key2: PolymeshPrimitivesStatisticsStat2ndKey;
    readonly value: Option<u128>;
  }

  /** @name PolymeshPrimitivesStatisticsStat2ndKey (252) */
  export interface PolymeshPrimitivesStatisticsStat2ndKey extends Enum {
    readonly isNoClaimStat: boolean;
    readonly isClaim: boolean;
    readonly asClaim: PolymeshPrimitivesStatisticsStatClaim;
    readonly type: 'NoClaimStat' | 'Claim';
  }

  /** @name PolymeshPrimitivesStatisticsStatClaim (253) */
  export interface PolymeshPrimitivesStatisticsStatClaim extends Enum {
    readonly isAccredited: boolean;
    readonly asAccredited: bool;
    readonly isAffiliate: boolean;
    readonly asAffiliate: bool;
    readonly isJurisdiction: boolean;
    readonly asJurisdiction: Option<PolymeshPrimitivesJurisdictionCountryCode>;
    readonly type: 'Accredited' | 'Affiliate' | 'Jurisdiction';
  }

  /** @name PolymeshPrimitivesTransferComplianceTransferCondition (257) */
  export interface PolymeshPrimitivesTransferComplianceTransferCondition extends Enum {
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

  /** @name PolymeshPrimitivesTransferComplianceTransferConditionExemptKey (259) */
  export interface PolymeshPrimitivesTransferComplianceTransferConditionExemptKey extends Struct {
    readonly asset: PolymeshPrimitivesStatisticsAssetScope;
    readonly op: PolymeshPrimitivesStatisticsStatOpType;
    readonly claimType: Option<PolymeshPrimitivesIdentityClaimClaimType>;
  }

  /** @name PalletStoRawEvent (261) */
  export interface PalletStoRawEvent extends Enum {
    readonly isFundraiserCreated: boolean;
    readonly asFundraiserCreated: ITuple<[PolymeshPrimitivesIdentityId, u64, Bytes, PalletStoFundraiser]>;
    readonly isInvested: boolean;
    readonly asInvested: ITuple<[PolymeshPrimitivesIdentityId, u64, PolymeshPrimitivesTicker, PolymeshPrimitivesTicker, u128, u128]>;
    readonly isFundraiserFrozen: boolean;
    readonly asFundraiserFrozen: ITuple<[PolymeshPrimitivesIdentityId, u64]>;
    readonly isFundraiserUnfrozen: boolean;
    readonly asFundraiserUnfrozen: ITuple<[PolymeshPrimitivesIdentityId, u64]>;
    readonly isFundraiserWindowModified: boolean;
    readonly asFundraiserWindowModified: ITuple<[PolymeshPrimitivesEventOnly, u64, u64, Option<u64>, u64, Option<u64>]>;
    readonly isFundraiserClosed: boolean;
    readonly asFundraiserClosed: ITuple<[PolymeshPrimitivesIdentityId, u64]>;
    readonly type: 'FundraiserCreated' | 'Invested' | 'FundraiserFrozen' | 'FundraiserUnfrozen' | 'FundraiserWindowModified' | 'FundraiserClosed';
  }

  /** @name PalletStoFundraiser (264) */
  export interface PalletStoFundraiser extends Struct {
    readonly creator: PolymeshPrimitivesIdentityId;
    readonly offeringPortfolio: PolymeshPrimitivesIdentityIdPortfolioId;
    readonly offeringAsset: PolymeshPrimitivesTicker;
    readonly raisingPortfolio: PolymeshPrimitivesIdentityIdPortfolioId;
    readonly raisingAsset: PolymeshPrimitivesTicker;
    readonly tiers: Vec<PalletStoFundraiserTier>;
    readonly venueId: u64;
    readonly start: u64;
    readonly end: Option<u64>;
    readonly status: PalletStoFundraiserStatus;
    readonly minimumInvestment: u128;
  }

  /** @name PalletStoFundraiserTier (266) */
  export interface PalletStoFundraiserTier extends Struct {
    readonly total: u128;
    readonly price: u128;
    readonly remaining: u128;
  }

  /** @name PalletStoFundraiserStatus (267) */
  export interface PalletStoFundraiserStatus extends Enum {
    readonly isLive: boolean;
    readonly isFrozen: boolean;
    readonly isClosed: boolean;
    readonly isClosedEarly: boolean;
    readonly type: 'Live' | 'Frozen' | 'Closed' | 'ClosedEarly';
  }

  /** @name PalletTreasuryRawEvent (268) */
  export interface PalletTreasuryRawEvent extends Enum {
    readonly isTreasuryDisbursement: boolean;
    readonly asTreasuryDisbursement: ITuple<[PolymeshPrimitivesIdentityId, PolymeshPrimitivesIdentityId, AccountId32, u128]>;
    readonly isTreasuryDisbursementFailed: boolean;
    readonly asTreasuryDisbursementFailed: ITuple<[PolymeshPrimitivesIdentityId, PolymeshPrimitivesIdentityId, AccountId32, u128]>;
    readonly isTreasuryReimbursement: boolean;
    readonly asTreasuryReimbursement: ITuple<[PolymeshPrimitivesIdentityId, u128]>;
    readonly type: 'TreasuryDisbursement' | 'TreasuryDisbursementFailed' | 'TreasuryReimbursement';
  }

  /** @name PalletUtilityEvent (269) */
  export interface PalletUtilityEvent extends Enum {
    readonly isBatchInterrupted: boolean;
    readonly asBatchInterrupted: ITuple<[Vec<u32>, ITuple<[u32, SpRuntimeDispatchError]>]>;
    readonly isBatchOptimisticFailed: boolean;
    readonly asBatchOptimisticFailed: ITuple<[Vec<u32>, Vec<ITuple<[u32, SpRuntimeDispatchError]>>]>;
    readonly isBatchCompleted: boolean;
    readonly asBatchCompleted: Vec<u32>;
    readonly type: 'BatchInterrupted' | 'BatchOptimisticFailed' | 'BatchCompleted';
  }

  /** @name PolymeshCommonUtilitiesBaseEvent (273) */
  export interface PolymeshCommonUtilitiesBaseEvent extends Enum {
    readonly isUnexpectedError: boolean;
    readonly asUnexpectedError: Option<SpRuntimeDispatchError>;
    readonly type: 'UnexpectedError';
  }

  /** @name PolymeshCommonUtilitiesExternalAgentsEvent (275) */
  export interface PolymeshCommonUtilitiesExternalAgentsEvent extends Enum {
    readonly isGroupCreated: boolean;
    readonly asGroupCreated: ITuple<[PolymeshPrimitivesEventOnly, PolymeshPrimitivesTicker, u32, PolymeshPrimitivesSubsetSubsetRestrictionPalletPermissions]>;
    readonly isGroupPermissionsUpdated: boolean;
    readonly asGroupPermissionsUpdated: ITuple<[PolymeshPrimitivesEventOnly, PolymeshPrimitivesTicker, u32, PolymeshPrimitivesSubsetSubsetRestrictionPalletPermissions]>;
    readonly isAgentAdded: boolean;
    readonly asAgentAdded: ITuple<[PolymeshPrimitivesEventOnly, PolymeshPrimitivesTicker, PolymeshPrimitivesAgentAgentGroup]>;
    readonly isAgentRemoved: boolean;
    readonly asAgentRemoved: ITuple<[PolymeshPrimitivesEventOnly, PolymeshPrimitivesTicker, PolymeshPrimitivesIdentityId]>;
    readonly isGroupChanged: boolean;
    readonly asGroupChanged: ITuple<[PolymeshPrimitivesEventOnly, PolymeshPrimitivesTicker, PolymeshPrimitivesIdentityId, PolymeshPrimitivesAgentAgentGroup]>;
    readonly type: 'GroupCreated' | 'GroupPermissionsUpdated' | 'AgentAdded' | 'AgentRemoved' | 'GroupChanged';
  }

  /** @name PolymeshCommonUtilitiesRelayerRawEvent (276) */
  export interface PolymeshCommonUtilitiesRelayerRawEvent extends Enum {
    readonly isAuthorizedPayingKey: boolean;
    readonly asAuthorizedPayingKey: ITuple<[PolymeshPrimitivesEventOnly, AccountId32, AccountId32, u128, u64]>;
    readonly isAcceptedPayingKey: boolean;
    readonly asAcceptedPayingKey: ITuple<[PolymeshPrimitivesEventOnly, AccountId32, AccountId32]>;
    readonly isRemovedPayingKey: boolean;
    readonly asRemovedPayingKey: ITuple<[PolymeshPrimitivesEventOnly, AccountId32, AccountId32]>;
    readonly isUpdatedPolyxLimit: boolean;
    readonly asUpdatedPolyxLimit: ITuple<[PolymeshPrimitivesEventOnly, AccountId32, AccountId32, u128, u128]>;
    readonly type: 'AuthorizedPayingKey' | 'AcceptedPayingKey' | 'RemovedPayingKey' | 'UpdatedPolyxLimit';
  }

  /** @name PalletRewardsRawEvent (277) */
  export interface PalletRewardsRawEvent extends Enum {
    readonly isItnRewardClaimed: boolean;
    readonly asItnRewardClaimed: ITuple<[AccountId32, u128]>;
    readonly type: 'ItnRewardClaimed';
  }

  /** @name PalletContractsEvent (278) */
  export interface PalletContractsEvent extends Enum {
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
    } & Struct;
    readonly isContractEmitted: boolean;
    readonly asContractEmitted: {
      readonly contract: AccountId32;
      readonly data: Bytes;
    } & Struct;
    readonly isCodeRemoved: boolean;
    readonly asCodeRemoved: {
      readonly codeHash: H256;
    } & Struct;
    readonly isContractCodeUpdated: boolean;
    readonly asContractCodeUpdated: {
      readonly contract: AccountId32;
      readonly newCodeHash: H256;
      readonly oldCodeHash: H256;
    } & Struct;
    readonly type: 'Instantiated' | 'Terminated' | 'CodeStored' | 'ContractEmitted' | 'CodeRemoved' | 'ContractCodeUpdated';
  }

  /** @name PolymeshContractsEvent (279) */
  export type PolymeshContractsEvent = Null;

  /** @name PalletPreimageEvent (280) */
  export interface PalletPreimageEvent extends Enum {
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

  /** @name PalletTestUtilsRawEvent (281) */
  export interface PalletTestUtilsRawEvent extends Enum {
    readonly isMockInvestorUIDCreated: boolean;
    readonly asMockInvestorUIDCreated: ITuple<[PolymeshPrimitivesIdentityId, PolymeshPrimitivesCddIdInvestorUid]>;
    readonly isDidStatus: boolean;
    readonly asDidStatus: ITuple<[PolymeshPrimitivesIdentityId, AccountId32]>;
    readonly isCddStatus: boolean;
    readonly asCddStatus: ITuple<[Option<PolymeshPrimitivesIdentityId>, AccountId32, bool]>;
    readonly type: 'MockInvestorUIDCreated' | 'DidStatus' | 'CddStatus';
  }

  /** @name PolymeshPrimitivesCddIdInvestorUid (282) */
  export interface PolymeshPrimitivesCddIdInvestorUid extends U8aFixed {}

  /** @name FrameSystemPhase (283) */
  export interface FrameSystemPhase extends Enum {
    readonly isApplyExtrinsic: boolean;
    readonly asApplyExtrinsic: u32;
    readonly isFinalization: boolean;
    readonly isInitialization: boolean;
    readonly type: 'ApplyExtrinsic' | 'Finalization' | 'Initialization';
  }

  /** @name FrameSystemLastRuntimeUpgradeInfo (286) */
  export interface FrameSystemLastRuntimeUpgradeInfo extends Struct {
    readonly specVersion: Compact<u32>;
    readonly specName: Text;
  }

  /** @name FrameSystemCall (289) */
  export interface FrameSystemCall extends Enum {
    readonly isFillBlock: boolean;
    readonly asFillBlock: {
      readonly ratio: Perbill;
    } & Struct;
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
    readonly type: 'FillBlock' | 'Remark' | 'SetHeapPages' | 'SetCode' | 'SetCodeWithoutChecks' | 'SetStorage' | 'KillStorage' | 'KillPrefix' | 'RemarkWithEvent';
  }

  /** @name FrameSystemLimitsBlockWeights (293) */
  export interface FrameSystemLimitsBlockWeights extends Struct {
    readonly baseBlock: u64;
    readonly maxBlock: u64;
    readonly perClass: FrameSupportWeightsPerDispatchClassWeightsPerClass;
  }

  /** @name FrameSupportWeightsPerDispatchClassWeightsPerClass (294) */
  export interface FrameSupportWeightsPerDispatchClassWeightsPerClass extends Struct {
    readonly normal: FrameSystemLimitsWeightsPerClass;
    readonly operational: FrameSystemLimitsWeightsPerClass;
    readonly mandatory: FrameSystemLimitsWeightsPerClass;
  }

  /** @name FrameSystemLimitsWeightsPerClass (295) */
  export interface FrameSystemLimitsWeightsPerClass extends Struct {
    readonly baseExtrinsic: u64;
    readonly maxExtrinsic: Option<u64>;
    readonly maxTotal: Option<u64>;
    readonly reserved: Option<u64>;
  }

  /** @name FrameSystemLimitsBlockLength (296) */
  export interface FrameSystemLimitsBlockLength extends Struct {
    readonly max: FrameSupportWeightsPerDispatchClassU32;
  }

  /** @name FrameSupportWeightsPerDispatchClassU32 (297) */
  export interface FrameSupportWeightsPerDispatchClassU32 extends Struct {
    readonly normal: u32;
    readonly operational: u32;
    readonly mandatory: u32;
  }

  /** @name FrameSupportWeightsRuntimeDbWeight (298) */
  export interface FrameSupportWeightsRuntimeDbWeight extends Struct {
    readonly read: u64;
    readonly write: u64;
  }

  /** @name SpVersionRuntimeVersion (299) */
  export interface SpVersionRuntimeVersion extends Struct {
    readonly specName: Text;
    readonly implName: Text;
    readonly authoringVersion: u32;
    readonly specVersion: u32;
    readonly implVersion: u32;
    readonly apis: Vec<ITuple<[U8aFixed, u32]>>;
    readonly transactionVersion: u32;
    readonly stateVersion: u8;
  }

  /** @name FrameSystemError (304) */
  export interface FrameSystemError extends Enum {
    readonly isInvalidSpecName: boolean;
    readonly isSpecVersionNeedsToIncrease: boolean;
    readonly isFailedToExtractRuntimeVersion: boolean;
    readonly isNonDefaultComposite: boolean;
    readonly isNonZeroRefCount: boolean;
    readonly isCallFiltered: boolean;
    readonly type: 'InvalidSpecName' | 'SpecVersionNeedsToIncrease' | 'FailedToExtractRuntimeVersion' | 'NonDefaultComposite' | 'NonZeroRefCount' | 'CallFiltered';
  }

  /** @name SpConsensusBabeAppPublic (307) */
  export interface SpConsensusBabeAppPublic extends SpCoreSr25519Public {}

  /** @name SpConsensusBabeDigestsNextConfigDescriptor (310) */
  export interface SpConsensusBabeDigestsNextConfigDescriptor extends Enum {
    readonly isV1: boolean;
    readonly asV1: {
      readonly c: ITuple<[u64, u64]>;
      readonly allowedSlots: SpConsensusBabeAllowedSlots;
    } & Struct;
    readonly type: 'V1';
  }

  /** @name SpConsensusBabeAllowedSlots (312) */
  export interface SpConsensusBabeAllowedSlots extends Enum {
    readonly isPrimarySlots: boolean;
    readonly isPrimaryAndSecondaryPlainSlots: boolean;
    readonly isPrimaryAndSecondaryVRFSlots: boolean;
    readonly type: 'PrimarySlots' | 'PrimaryAndSecondaryPlainSlots' | 'PrimaryAndSecondaryVRFSlots';
  }

  /** @name SpConsensusBabeBabeEpochConfiguration (316) */
  export interface SpConsensusBabeBabeEpochConfiguration extends Struct {
    readonly c: ITuple<[u64, u64]>;
    readonly allowedSlots: SpConsensusBabeAllowedSlots;
  }

  /** @name PalletBabeCall (317) */
  export interface PalletBabeCall extends Enum {
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

  /** @name SpConsensusSlotsEquivocationProof (318) */
  export interface SpConsensusSlotsEquivocationProof extends Struct {
    readonly offender: SpConsensusBabeAppPublic;
    readonly slot: u64;
    readonly firstHeader: SpRuntimeHeader;
    readonly secondHeader: SpRuntimeHeader;
  }

  /** @name SpRuntimeHeader (319) */
  export interface SpRuntimeHeader extends Struct {
    readonly parentHash: H256;
    readonly number: Compact<u32>;
    readonly stateRoot: H256;
    readonly extrinsicsRoot: H256;
    readonly digest: SpRuntimeDigest;
  }

  /** @name SpRuntimeBlakeTwo256 (320) */
  export type SpRuntimeBlakeTwo256 = Null;

  /** @name SpSessionMembershipProof (321) */
  export interface SpSessionMembershipProof extends Struct {
    readonly session: u32;
    readonly trieNodes: Vec<Bytes>;
    readonly validatorCount: u32;
  }

  /** @name PalletBabeError (322) */
  export interface PalletBabeError extends Enum {
    readonly isInvalidEquivocationProof: boolean;
    readonly isInvalidKeyOwnershipProof: boolean;
    readonly isDuplicateOffenceReport: boolean;
    readonly type: 'InvalidEquivocationProof' | 'InvalidKeyOwnershipProof' | 'DuplicateOffenceReport';
  }

  /** @name PalletTimestampCall (323) */
  export interface PalletTimestampCall extends Enum {
    readonly isSet: boolean;
    readonly asSet: {
      readonly now: Compact<u64>;
    } & Struct;
    readonly type: 'Set';
  }

  /** @name PalletIndicesCall (326) */
  export interface PalletIndicesCall extends Enum {
    readonly isClaim: boolean;
    readonly asClaim: {
      readonly index: u32;
    } & Struct;
    readonly isTransfer: boolean;
    readonly asTransfer: {
      readonly new_: AccountId32;
      readonly index: u32;
    } & Struct;
    readonly isFree: boolean;
    readonly asFree: {
      readonly index: u32;
    } & Struct;
    readonly isForceTransfer: boolean;
    readonly asForceTransfer: {
      readonly new_: AccountId32;
      readonly index: u32;
      readonly freeze: bool;
    } & Struct;
    readonly isFreeze: boolean;
    readonly asFreeze: {
      readonly index: u32;
    } & Struct;
    readonly type: 'Claim' | 'Transfer' | 'Free' | 'ForceTransfer' | 'Freeze';
  }

  /** @name PalletIndicesError (327) */
  export interface PalletIndicesError extends Enum {
    readonly isNotAssigned: boolean;
    readonly isNotOwner: boolean;
    readonly isInUse: boolean;
    readonly isNotTransfer: boolean;
    readonly isPermanent: boolean;
    readonly type: 'NotAssigned' | 'NotOwner' | 'InUse' | 'NotTransfer' | 'Permanent';
  }

  /** @name PalletAuthorshipUncleEntryItem (329) */
  export interface PalletAuthorshipUncleEntryItem extends Enum {
    readonly isInclusionHeight: boolean;
    readonly asInclusionHeight: u32;
    readonly isUncle: boolean;
    readonly asUncle: ITuple<[H256, Option<AccountId32>]>;
    readonly type: 'InclusionHeight' | 'Uncle';
  }

  /** @name PalletAuthorshipCall (330) */
  export interface PalletAuthorshipCall extends Enum {
    readonly isSetUncles: boolean;
    readonly asSetUncles: {
      readonly newUncles: Vec<SpRuntimeHeader>;
    } & Struct;
    readonly type: 'SetUncles';
  }

  /** @name PalletAuthorshipError (332) */
  export interface PalletAuthorshipError extends Enum {
    readonly isInvalidUncleParent: boolean;
    readonly isUnclesAlreadySet: boolean;
    readonly isTooManyUncles: boolean;
    readonly isGenesisUncle: boolean;
    readonly isTooHighUncle: boolean;
    readonly isUncleAlreadyIncluded: boolean;
    readonly isOldUncle: boolean;
    readonly type: 'InvalidUncleParent' | 'UnclesAlreadySet' | 'TooManyUncles' | 'GenesisUncle' | 'TooHighUncle' | 'UncleAlreadyIncluded' | 'OldUncle';
  }

  /** @name PalletBalancesBalanceLock (334) */
  export interface PalletBalancesBalanceLock extends Struct {
    readonly id: U8aFixed;
    readonly amount: u128;
    readonly reasons: PolymeshCommonUtilitiesBalancesReasons;
  }

  /** @name PolymeshCommonUtilitiesBalancesReasons (335) */
  export interface PolymeshCommonUtilitiesBalancesReasons extends Enum {
    readonly isFee: boolean;
    readonly isMisc: boolean;
    readonly isAll: boolean;
    readonly type: 'Fee' | 'Misc' | 'All';
  }

  /** @name PalletBalancesCall (336) */
  export interface PalletBalancesCall extends Enum {
    readonly isTransfer: boolean;
    readonly asTransfer: {
      readonly dest: MultiAddress;
      readonly value: Compact<u128>;
    } & Struct;
    readonly isTransferWithMemo: boolean;
    readonly asTransferWithMemo: {
      readonly dest: MultiAddress;
      readonly value: Compact<u128>;
      readonly memo: Option<PolymeshCommonUtilitiesBalancesMemo>;
    } & Struct;
    readonly isDepositBlockRewardReserveBalance: boolean;
    readonly asDepositBlockRewardReserveBalance: {
      readonly value: Compact<u128>;
    } & Struct;
    readonly isSetBalance: boolean;
    readonly asSetBalance: {
      readonly who: MultiAddress;
      readonly newFree: Compact<u128>;
      readonly newReserved: Compact<u128>;
    } & Struct;
    readonly isForceTransfer: boolean;
    readonly asForceTransfer: {
      readonly source: MultiAddress;
      readonly dest: MultiAddress;
      readonly value: Compact<u128>;
    } & Struct;
    readonly isBurnAccountBalance: boolean;
    readonly asBurnAccountBalance: {
      readonly amount: u128;
    } & Struct;
    readonly type: 'Transfer' | 'TransferWithMemo' | 'DepositBlockRewardReserveBalance' | 'SetBalance' | 'ForceTransfer' | 'BurnAccountBalance';
  }

  /** @name PalletBalancesError (338) */
  export interface PalletBalancesError extends Enum {
    readonly isLiquidityRestrictions: boolean;
    readonly isOverflow: boolean;
    readonly isInsufficientBalance: boolean;
    readonly isExistentialDeposit: boolean;
    readonly isReceiverCddMissing: boolean;
    readonly type: 'LiquidityRestrictions' | 'Overflow' | 'InsufficientBalance' | 'ExistentialDeposit' | 'ReceiverCddMissing';
  }

  /** @name PalletTransactionPaymentReleases (340) */
  export interface PalletTransactionPaymentReleases extends Enum {
    readonly isV1Ancient: boolean;
    readonly isV2: boolean;
    readonly type: 'V1Ancient' | 'V2';
  }

  /** @name FrameSupportWeightsWeightToFeeCoefficient (342) */
  export interface FrameSupportWeightsWeightToFeeCoefficient extends Struct {
    readonly coeffInteger: u128;
    readonly coeffFrac: Perbill;
    readonly negative: bool;
    readonly degree: u8;
  }

  /** @name PolymeshPrimitivesIdentityDidRecord (343) */
  export interface PolymeshPrimitivesIdentityDidRecord extends Struct {
    readonly primaryKey: Option<AccountId32>;
  }

  /** @name PalletIdentityClaim1stKey (345) */
  export interface PalletIdentityClaim1stKey extends Struct {
    readonly target: PolymeshPrimitivesIdentityId;
    readonly claimType: PolymeshPrimitivesIdentityClaimClaimType;
  }

  /** @name PalletIdentityClaim2ndKey (346) */
  export interface PalletIdentityClaim2ndKey extends Struct {
    readonly issuer: PolymeshPrimitivesIdentityId;
    readonly scope: Option<PolymeshPrimitivesIdentityClaimScope>;
  }

  /** @name PolymeshPrimitivesSecondaryKeyKeyRecord (348) */
  export interface PolymeshPrimitivesSecondaryKeyKeyRecord extends Enum {
    readonly isPrimaryKey: boolean;
    readonly asPrimaryKey: PolymeshPrimitivesIdentityId;
    readonly isSecondaryKey: boolean;
    readonly asSecondaryKey: ITuple<[PolymeshPrimitivesIdentityId, PolymeshPrimitivesSecondaryKeyPermissions]>;
    readonly isMultiSigSignerKey: boolean;
    readonly asMultiSigSignerKey: AccountId32;
    readonly type: 'PrimaryKey' | 'SecondaryKey' | 'MultiSigSignerKey';
  }

  /** @name PolymeshPrimitivesAuthorization (351) */
  export interface PolymeshPrimitivesAuthorization extends Struct {
    readonly authorizationData: PolymeshPrimitivesAuthorizationAuthorizationData;
    readonly authorizedBy: PolymeshPrimitivesIdentityId;
    readonly expiry: Option<u64>;
    readonly authId: u64;
  }

  /** @name PalletIdentityCall (354) */
  export interface PalletIdentityCall extends Enum {
    readonly isCddRegisterDid: boolean;
    readonly asCddRegisterDid: {
      readonly targetAccount: AccountId32;
      readonly secondaryKeys: Vec<PolymeshPrimitivesSecondaryKey>;
    } & Struct;
    readonly isInvalidateCddClaims: boolean;
    readonly asInvalidateCddClaims: {
      readonly cdd: PolymeshPrimitivesIdentityId;
      readonly disableFrom: u64;
      readonly expiry: Option<u64>;
    } & Struct;
    readonly isRemoveSecondaryKeysOld: boolean;
    readonly asRemoveSecondaryKeysOld: {
      readonly keysToRemove: Vec<PolymeshPrimitivesSecondaryKeySignatory>;
    } & Struct;
    readonly isAcceptPrimaryKey: boolean;
    readonly asAcceptPrimaryKey: {
      readonly rotationAuthId: u64;
      readonly optionalCddAuthId: Option<u64>;
    } & Struct;
    readonly isChangeCddRequirementForMkRotation: boolean;
    readonly asChangeCddRequirementForMkRotation: {
      readonly authRequired: bool;
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
    readonly isSetPermissionToSigner: boolean;
    readonly asSetPermissionToSigner: {
      readonly key: PolymeshPrimitivesSecondaryKeySignatory;
      readonly perms: PolymeshPrimitivesSecondaryKeyPermissions;
    } & Struct;
    readonly isPlaceholderLegacySetPermissionToSigner: boolean;
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
    readonly isAddSecondaryKeysWithAuthorizationOld: boolean;
    readonly asAddSecondaryKeysWithAuthorizationOld: {
      readonly additionalKeys: Vec<PolymeshCommonUtilitiesIdentitySecondaryKeyWithAuthV1>;
      readonly expiresAt: u64;
    } & Struct;
    readonly isAddInvestorUniquenessClaim: boolean;
    readonly asAddInvestorUniquenessClaim: {
      readonly target: PolymeshPrimitivesIdentityId;
      readonly claim: PolymeshPrimitivesIdentityClaimClaim;
      readonly proof: PolymeshPrimitivesInvestorZkproofDataV1InvestorZKProofData;
      readonly expiry: Option<u64>;
    } & Struct;
    readonly isGcAddCddClaim: boolean;
    readonly asGcAddCddClaim: {
      readonly target: PolymeshPrimitivesIdentityId;
    } & Struct;
    readonly isGcRevokeCddClaim: boolean;
    readonly asGcRevokeCddClaim: {
      readonly target: PolymeshPrimitivesIdentityId;
    } & Struct;
    readonly isAddInvestorUniquenessClaimV2: boolean;
    readonly asAddInvestorUniquenessClaimV2: {
      readonly target: PolymeshPrimitivesIdentityId;
      readonly scope: PolymeshPrimitivesIdentityClaimScope;
      readonly claim: PolymeshPrimitivesIdentityClaimClaim;
      readonly proof: ConfidentialIdentityClaimProofsScopeClaimProof;
      readonly expiry: Option<u64>;
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
      readonly optionalCddAuthId: Option<u64>;
    } & Struct;
    readonly isAddSecondaryKeysWithAuthorization: boolean;
    readonly asAddSecondaryKeysWithAuthorization: {
      readonly additionalKeys: Vec<PolymeshCommonUtilitiesIdentitySecondaryKeyWithAuth>;
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
    readonly type: 'CddRegisterDid' | 'InvalidateCddClaims' | 'RemoveSecondaryKeysOld' | 'AcceptPrimaryKey' | 'ChangeCddRequirementForMkRotation' | 'JoinIdentityAsKey' | 'LeaveIdentityAsKey' | 'AddClaim' | 'RevokeClaim' | 'SetPermissionToSigner' | 'PlaceholderLegacySetPermissionToSigner' | 'FreezeSecondaryKeys' | 'UnfreezeSecondaryKeys' | 'AddAuthorization' | 'RemoveAuthorization' | 'AddSecondaryKeysWithAuthorizationOld' | 'AddInvestorUniquenessClaim' | 'GcAddCddClaim' | 'GcRevokeCddClaim' | 'AddInvestorUniquenessClaimV2' | 'RevokeClaimByIndex' | 'RotatePrimaryKeyToSecondary' | 'AddSecondaryKeysWithAuthorization' | 'SetSecondaryKeyPermissions' | 'RemoveSecondaryKeys';
  }

  /** @name PolymeshCommonUtilitiesIdentitySecondaryKeyWithAuthV1 (356) */
  export interface PolymeshCommonUtilitiesIdentitySecondaryKeyWithAuthV1 extends Struct {
    readonly secondaryKey: PolymeshPrimitivesSecondaryKeyV1SecondaryKey;
    readonly authSignature: H512;
  }

  /** @name PolymeshPrimitivesSecondaryKeyV1SecondaryKey (357) */
  export interface PolymeshPrimitivesSecondaryKeyV1SecondaryKey extends Struct {
    readonly signer: PolymeshPrimitivesSecondaryKeySignatory;
    readonly permissions: PolymeshPrimitivesSecondaryKeyPermissions;
  }

  /** @name PolymeshPrimitivesInvestorZkproofDataV1InvestorZKProofData (359) */
  export interface PolymeshPrimitivesInvestorZkproofDataV1InvestorZKProofData extends SchnorrkelSignSignature {}

  /** @name SchnorrkelSignSignature (360) */
  export interface SchnorrkelSignSignature extends Struct {
    readonly r: Curve25519DalekRistrettoCompressedRistretto;
    readonly s: Curve25519DalekScalar;
  }

  /** @name Curve25519DalekRistrettoCompressedRistretto (361) */
  export interface Curve25519DalekRistrettoCompressedRistretto extends U8aFixed {}

  /** @name Curve25519DalekScalar (362) */
  export interface Curve25519DalekScalar extends Struct {
    readonly bytes: U8aFixed;
  }

  /** @name ConfidentialIdentityClaimProofsScopeClaimProof (363) */
  export interface ConfidentialIdentityClaimProofsScopeClaimProof extends Struct {
    readonly proofScopeIdWellformed: ConfidentialIdentitySignSignature;
    readonly proofScopeIdCddIdMatch: ConfidentialIdentityClaimProofsZkProofData;
    readonly scopeId: Curve25519DalekRistrettoRistrettoPoint;
  }

  /** @name ConfidentialIdentitySignSignature (364) */
  export interface ConfidentialIdentitySignSignature extends Struct {
    readonly r: Curve25519DalekRistrettoCompressedRistretto;
    readonly s: Curve25519DalekScalar;
  }

  /** @name ConfidentialIdentityClaimProofsZkProofData (365) */
  export interface ConfidentialIdentityClaimProofsZkProofData extends Struct {
    readonly challengeResponses: Vec<Curve25519DalekScalar>;
    readonly subtractExpressionsRes: Curve25519DalekRistrettoRistrettoPoint;
    readonly blindedScopeDidHash: Curve25519DalekRistrettoRistrettoPoint;
  }

  /** @name Curve25519DalekRistrettoRistrettoPoint (367) */
  export interface Curve25519DalekRistrettoRistrettoPoint extends Curve25519DalekEdwardsEdwardsPoint {}

  /** @name Curve25519DalekEdwardsEdwardsPoint (368) */
  export interface Curve25519DalekEdwardsEdwardsPoint extends Struct {
    readonly x: Curve25519DalekBackendSerialU64FieldFieldElement51;
    readonly y: Curve25519DalekBackendSerialU64FieldFieldElement51;
    readonly z: Curve25519DalekBackendSerialU64FieldFieldElement51;
    readonly t: Curve25519DalekBackendSerialU64FieldFieldElement51;
  }

  /** @name Curve25519DalekBackendSerialU64FieldFieldElement51 (369) */
  export interface Curve25519DalekBackendSerialU64FieldFieldElement51 extends Vec<u64> {}

  /** @name PolymeshCommonUtilitiesIdentitySecondaryKeyWithAuth (372) */
  export interface PolymeshCommonUtilitiesIdentitySecondaryKeyWithAuth extends Struct {
    readonly secondaryKey: PolymeshPrimitivesSecondaryKey;
    readonly authSignature: H512;
  }

  /** @name PalletIdentityError (373) */
  export interface PalletIdentityError extends Enum {
    readonly isAlreadyLinked: boolean;
    readonly isMissingCurrentIdentity: boolean;
    readonly isUnauthorized: boolean;
    readonly isInvalidAccountKey: boolean;
    readonly isUnAuthorizedCddProvider: boolean;
    readonly isInvalidAuthorizationFromOwner: boolean;
    readonly isInvalidAuthorizationFromCddProvider: boolean;
    readonly isNotCddProviderAttestation: boolean;
    readonly isAuthorizationsNotForSameDids: boolean;
    readonly isDidMustAlreadyExist: boolean;
    readonly isCurrentIdentityCannotBeForwarded: boolean;
    readonly isAuthorizationExpired: boolean;
    readonly isTargetHasNoCdd: boolean;
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
    readonly isMultiSigHasBalance: boolean;
    readonly isConfidentialScopeClaimNotAllowed: boolean;
    readonly isInvalidScopeClaim: boolean;
    readonly isClaimVariantNotAllowed: boolean;
    readonly isTargetHasNonZeroBalanceAtScopeId: boolean;
    readonly isCddIdNotUniqueForIdentity: boolean;
    readonly isInvalidCDDId: boolean;
    readonly isClaimAndProofVersionsDoNotMatch: boolean;
    readonly isAccountKeyIsBeingUsed: boolean;
    readonly isCustomScopeTooLong: boolean;
    readonly type: 'AlreadyLinked' | 'MissingCurrentIdentity' | 'Unauthorized' | 'InvalidAccountKey' | 'UnAuthorizedCddProvider' | 'InvalidAuthorizationFromOwner' | 'InvalidAuthorizationFromCddProvider' | 'NotCddProviderAttestation' | 'AuthorizationsNotForSameDids' | 'DidMustAlreadyExist' | 'CurrentIdentityCannotBeForwarded' | 'AuthorizationExpired' | 'TargetHasNoCdd' | 'AuthorizationHasBeenRevoked' | 'InvalidAuthorizationSignature' | 'KeyNotAllowed' | 'NotPrimaryKey' | 'DidDoesNotExist' | 'DidAlreadyExists' | 'SecondaryKeysContainPrimaryKey' | 'FailedToChargeFee' | 'NotASigner' | 'CannotDecodeSignerAccountId' | 'MultiSigHasBalance' | 'ConfidentialScopeClaimNotAllowed' | 'InvalidScopeClaim' | 'ClaimVariantNotAllowed' | 'TargetHasNonZeroBalanceAtScopeId' | 'CddIdNotUniqueForIdentity' | 'InvalidCDDId' | 'ClaimAndProofVersionsDoNotMatch' | 'AccountKeyIsBeingUsed' | 'CustomScopeTooLong';
  }

  /** @name PolymeshCommonUtilitiesGroupInactiveMember (375) */
  export interface PolymeshCommonUtilitiesGroupInactiveMember extends Struct {
    readonly id: PolymeshPrimitivesIdentityId;
    readonly deactivatedAt: u64;
    readonly expiry: Option<u64>;
  }

  /** @name PalletGroupCall (376) */
  export interface PalletGroupCall extends Enum {
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

  /** @name PalletGroupError (377) */
  export interface PalletGroupError extends Enum {
    readonly isOnlyPrimaryKeyAllowed: boolean;
    readonly isDuplicateMember: boolean;
    readonly isNoSuchMember: boolean;
    readonly isLastMemberCannotQuit: boolean;
    readonly isMissingCurrentIdentity: boolean;
    readonly isActiveMembersLimitExceeded: boolean;
    readonly isActiveMembersLimitOverflow: boolean;
    readonly type: 'OnlyPrimaryKeyAllowed' | 'DuplicateMember' | 'NoSuchMember' | 'LastMemberCannotQuit' | 'MissingCurrentIdentity' | 'ActiveMembersLimitExceeded' | 'ActiveMembersLimitOverflow';
  }

  /** @name PalletCommitteeCall (379) */
  export interface PalletCommitteeCall extends Enum {
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
      readonly expiry: PolymeshCommonUtilitiesMaybeBlock;
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

  /** @name PalletMultisigCall (385) */
  export interface PalletMultisigCall extends Enum {
    readonly isCreateMultisig: boolean;
    readonly asCreateMultisig: {
      readonly signers: Vec<PolymeshPrimitivesSecondaryKeySignatory>;
      readonly sigsRequired: u64;
    } & Struct;
    readonly isCreateOrApproveProposalAsIdentity: boolean;
    readonly asCreateOrApproveProposalAsIdentity: {
      readonly multisig: AccountId32;
      readonly proposal: Call;
      readonly expiry: Option<u64>;
      readonly autoClose: bool;
    } & Struct;
    readonly isCreateOrApproveProposalAsKey: boolean;
    readonly asCreateOrApproveProposalAsKey: {
      readonly multisig: AccountId32;
      readonly proposal: Call;
      readonly expiry: Option<u64>;
      readonly autoClose: bool;
    } & Struct;
    readonly isCreateProposalAsIdentity: boolean;
    readonly asCreateProposalAsIdentity: {
      readonly multisig: AccountId32;
      readonly proposal: Call;
      readonly expiry: Option<u64>;
      readonly autoClose: bool;
    } & Struct;
    readonly isCreateProposalAsKey: boolean;
    readonly asCreateProposalAsKey: {
      readonly multisig: AccountId32;
      readonly proposal: Call;
      readonly expiry: Option<u64>;
      readonly autoClose: bool;
    } & Struct;
    readonly isApproveAsIdentity: boolean;
    readonly asApproveAsIdentity: {
      readonly multisig: AccountId32;
      readonly proposalId: u64;
    } & Struct;
    readonly isApproveAsKey: boolean;
    readonly asApproveAsKey: {
      readonly multisig: AccountId32;
      readonly proposalId: u64;
    } & Struct;
    readonly isRejectAsIdentity: boolean;
    readonly asRejectAsIdentity: {
      readonly multisig: AccountId32;
      readonly proposalId: u64;
    } & Struct;
    readonly isRejectAsKey: boolean;
    readonly asRejectAsKey: {
      readonly multisig: AccountId32;
      readonly proposalId: u64;
    } & Struct;
    readonly isAcceptMultisigSignerAsIdentity: boolean;
    readonly asAcceptMultisigSignerAsIdentity: {
      readonly authId: u64;
    } & Struct;
    readonly isAcceptMultisigSignerAsKey: boolean;
    readonly asAcceptMultisigSignerAsKey: {
      readonly authId: u64;
    } & Struct;
    readonly isAddMultisigSigner: boolean;
    readonly asAddMultisigSigner: {
      readonly signer: PolymeshPrimitivesSecondaryKeySignatory;
    } & Struct;
    readonly isRemoveMultisigSigner: boolean;
    readonly asRemoveMultisigSigner: {
      readonly signer: PolymeshPrimitivesSecondaryKeySignatory;
    } & Struct;
    readonly isAddMultisigSignersViaCreator: boolean;
    readonly asAddMultisigSignersViaCreator: {
      readonly multisig: AccountId32;
      readonly signers: Vec<PolymeshPrimitivesSecondaryKeySignatory>;
    } & Struct;
    readonly isRemoveMultisigSignersViaCreator: boolean;
    readonly asRemoveMultisigSignersViaCreator: {
      readonly multisig: AccountId32;
      readonly signers: Vec<PolymeshPrimitivesSecondaryKeySignatory>;
    } & Struct;
    readonly isChangeSigsRequired: boolean;
    readonly asChangeSigsRequired: {
      readonly sigsRequired: u64;
    } & Struct;
    readonly isMakeMultisigSecondary: boolean;
    readonly asMakeMultisigSecondary: {
      readonly multisig: AccountId32;
    } & Struct;
    readonly isMakeMultisigPrimary: boolean;
    readonly asMakeMultisigPrimary: {
      readonly multisig: AccountId32;
      readonly optionalCddAuthId: Option<u64>;
    } & Struct;
    readonly isExecuteScheduledProposal: boolean;
    readonly asExecuteScheduledProposal: {
      readonly multisig: AccountId32;
      readonly proposalId: u64;
      readonly multisigDid: PolymeshPrimitivesIdentityId;
      readonly proposalWeight: u64;
    } & Struct;
    readonly type: 'CreateMultisig' | 'CreateOrApproveProposalAsIdentity' | 'CreateOrApproveProposalAsKey' | 'CreateProposalAsIdentity' | 'CreateProposalAsKey' | 'ApproveAsIdentity' | 'ApproveAsKey' | 'RejectAsIdentity' | 'RejectAsKey' | 'AcceptMultisigSignerAsIdentity' | 'AcceptMultisigSignerAsKey' | 'AddMultisigSigner' | 'RemoveMultisigSigner' | 'AddMultisigSignersViaCreator' | 'RemoveMultisigSignersViaCreator' | 'ChangeSigsRequired' | 'MakeMultisigSecondary' | 'MakeMultisigPrimary' | 'ExecuteScheduledProposal';
  }

  /** @name PalletBridgeCall (386) */
  export interface PalletBridgeCall extends Enum {
    readonly isChangeController: boolean;
    readonly asChangeController: {
      readonly controller: AccountId32;
    } & Struct;
    readonly isChangeAdmin: boolean;
    readonly asChangeAdmin: {
      readonly admin: AccountId32;
    } & Struct;
    readonly isChangeTimelock: boolean;
    readonly asChangeTimelock: {
      readonly timelock: u32;
    } & Struct;
    readonly isFreeze: boolean;
    readonly isUnfreeze: boolean;
    readonly isChangeBridgeLimit: boolean;
    readonly asChangeBridgeLimit: {
      readonly amount: u128;
      readonly duration: u32;
    } & Struct;
    readonly isChangeBridgeExempted: boolean;
    readonly asChangeBridgeExempted: {
      readonly exempted: Vec<ITuple<[PolymeshPrimitivesIdentityId, bool]>>;
    } & Struct;
    readonly isForceHandleBridgeTx: boolean;
    readonly asForceHandleBridgeTx: {
      readonly bridgeTx: PalletBridgeBridgeTx;
    } & Struct;
    readonly isBatchProposeBridgeTx: boolean;
    readonly asBatchProposeBridgeTx: {
      readonly bridgeTxs: Vec<PalletBridgeBridgeTx>;
    } & Struct;
    readonly isProposeBridgeTx: boolean;
    readonly asProposeBridgeTx: {
      readonly bridgeTx: PalletBridgeBridgeTx;
    } & Struct;
    readonly isHandleBridgeTx: boolean;
    readonly asHandleBridgeTx: {
      readonly bridgeTx: PalletBridgeBridgeTx;
    } & Struct;
    readonly isFreezeTxs: boolean;
    readonly asFreezeTxs: {
      readonly bridgeTxs: Vec<PalletBridgeBridgeTx>;
    } & Struct;
    readonly isUnfreezeTxs: boolean;
    readonly asUnfreezeTxs: {
      readonly bridgeTxs: Vec<PalletBridgeBridgeTx>;
    } & Struct;
    readonly isHandleScheduledBridgeTx: boolean;
    readonly asHandleScheduledBridgeTx: {
      readonly bridgeTx: PalletBridgeBridgeTx;
    } & Struct;
    readonly isAddFreezeAdmin: boolean;
    readonly asAddFreezeAdmin: {
      readonly freezeAdmin: AccountId32;
    } & Struct;
    readonly isRemoveFreezeAdmin: boolean;
    readonly asRemoveFreezeAdmin: {
      readonly freezeAdmin: AccountId32;
    } & Struct;
    readonly isRemoveTxs: boolean;
    readonly asRemoveTxs: {
      readonly bridgeTxs: Vec<PalletBridgeBridgeTx>;
    } & Struct;
    readonly type: 'ChangeController' | 'ChangeAdmin' | 'ChangeTimelock' | 'Freeze' | 'Unfreeze' | 'ChangeBridgeLimit' | 'ChangeBridgeExempted' | 'ForceHandleBridgeTx' | 'BatchProposeBridgeTx' | 'ProposeBridgeTx' | 'HandleBridgeTx' | 'FreezeTxs' | 'UnfreezeTxs' | 'HandleScheduledBridgeTx' | 'AddFreezeAdmin' | 'RemoveFreezeAdmin' | 'RemoveTxs';
  }

  /** @name PalletStakingCall (390) */
  export interface PalletStakingCall extends Enum {
    readonly isBond: boolean;
    readonly asBond: {
      readonly controller: MultiAddress;
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
    readonly asSetController: {
      readonly controller: MultiAddress;
    } & Struct;
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
    readonly isAddPermissionedValidator: boolean;
    readonly asAddPermissionedValidator: {
      readonly identity: PolymeshPrimitivesIdentityId;
      readonly intendedCount: Option<u32>;
    } & Struct;
    readonly isRemovePermissionedValidator: boolean;
    readonly asRemovePermissionedValidator: {
      readonly identity: PolymeshPrimitivesIdentityId;
    } & Struct;
    readonly isValidateCddExpiryNominators: boolean;
    readonly asValidateCddExpiryNominators: {
      readonly targets: Vec<AccountId32>;
    } & Struct;
    readonly isSetCommissionCap: boolean;
    readonly asSetCommissionCap: {
      readonly newCap: Perbill;
    } & Struct;
    readonly isSetMinBondThreshold: boolean;
    readonly asSetMinBondThreshold: {
      readonly newValue: u128;
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
    readonly isSetHistoryDepth: boolean;
    readonly asSetHistoryDepth: {
      readonly newHistoryDepth: Compact<u32>;
      readonly eraItemsDeleted: Compact<u32>;
    } & Struct;
    readonly isReapStash: boolean;
    readonly asReapStash: {
      readonly stash: AccountId32;
      readonly numSlashingSpans: u32;
    } & Struct;
    readonly isSubmitElectionSolution: boolean;
    readonly asSubmitElectionSolution: {
      readonly winners: Vec<u16>;
      readonly compact: PalletStakingCompactAssignments;
      readonly score: SpNposElectionsElectionScore;
      readonly era: u32;
      readonly size_: PalletStakingElectionSize;
    } & Struct;
    readonly isSubmitElectionSolutionUnsigned: boolean;
    readonly asSubmitElectionSolutionUnsigned: {
      readonly winners: Vec<u16>;
      readonly compact: PalletStakingCompactAssignments;
      readonly score: SpNposElectionsElectionScore;
      readonly era: u32;
      readonly size_: PalletStakingElectionSize;
    } & Struct;
    readonly isPayoutStakersBySystem: boolean;
    readonly asPayoutStakersBySystem: {
      readonly validatorStash: AccountId32;
      readonly era: u32;
    } & Struct;
    readonly isChangeSlashingAllowedFor: boolean;
    readonly asChangeSlashingAllowedFor: {
      readonly slashingSwitch: PalletStakingSlashingSwitch;
    } & Struct;
    readonly isUpdatePermissionedValidatorIntendedCount: boolean;
    readonly asUpdatePermissionedValidatorIntendedCount: {
      readonly identity: PolymeshPrimitivesIdentityId;
      readonly newIntendedCount: u32;
    } & Struct;
    readonly type: 'Bond' | 'BondExtra' | 'Unbond' | 'WithdrawUnbonded' | 'Validate' | 'Nominate' | 'Chill' | 'SetPayee' | 'SetController' | 'SetValidatorCount' | 'IncreaseValidatorCount' | 'ScaleValidatorCount' | 'AddPermissionedValidator' | 'RemovePermissionedValidator' | 'ValidateCddExpiryNominators' | 'SetCommissionCap' | 'SetMinBondThreshold' | 'ForceNoEras' | 'ForceNewEra' | 'SetInvulnerables' | 'ForceUnstake' | 'ForceNewEraAlways' | 'CancelDeferredSlash' | 'PayoutStakers' | 'Rebond' | 'SetHistoryDepth' | 'ReapStash' | 'SubmitElectionSolution' | 'SubmitElectionSolutionUnsigned' | 'PayoutStakersBySystem' | 'ChangeSlashingAllowedFor' | 'UpdatePermissionedValidatorIntendedCount';
  }

  /** @name PalletStakingRewardDestination (391) */
  export interface PalletStakingRewardDestination extends Enum {
    readonly isStaked: boolean;
    readonly isStash: boolean;
    readonly isController: boolean;
    readonly isAccount: boolean;
    readonly asAccount: AccountId32;
    readonly type: 'Staked' | 'Stash' | 'Controller' | 'Account';
  }

  /** @name PalletStakingValidatorPrefs (392) */
  export interface PalletStakingValidatorPrefs extends Struct {
    readonly commission: Compact<Perbill>;
    readonly blocked: bool;
  }

  /** @name PalletStakingCompactAssignments (398) */
  export interface PalletStakingCompactAssignments extends Struct {
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

  /** @name SpNposElectionsElectionScore (449) */
  export interface SpNposElectionsElectionScore extends Struct {
    readonly minimalStake: u128;
    readonly sumStake: u128;
    readonly sumStakeSquared: u128;
  }

  /** @name PalletStakingElectionSize (450) */
  export interface PalletStakingElectionSize extends Struct {
    readonly validators: Compact<u16>;
    readonly nominators: Compact<u32>;
  }

  /** @name PalletSessionCall (451) */
  export interface PalletSessionCall extends Enum {
    readonly isSetKeys: boolean;
    readonly asSetKeys: {
      readonly keys_: PolymeshRuntimeDevelopRuntimeSessionKeys;
      readonly proof: Bytes;
    } & Struct;
    readonly isPurgeKeys: boolean;
    readonly type: 'SetKeys' | 'PurgeKeys';
  }

  /** @name PolymeshRuntimeDevelopRuntimeSessionKeys (452) */
  export interface PolymeshRuntimeDevelopRuntimeSessionKeys extends Struct {
    readonly grandpa: SpFinalityGrandpaAppPublic;
    readonly babe: SpConsensusBabeAppPublic;
    readonly imOnline: PalletImOnlineSr25519AppSr25519Public;
    readonly authorityDiscovery: SpAuthorityDiscoveryAppPublic;
  }

  /** @name SpAuthorityDiscoveryAppPublic (453) */
  export interface SpAuthorityDiscoveryAppPublic extends SpCoreSr25519Public {}

  /** @name PalletGrandpaCall (454) */
  export interface PalletGrandpaCall extends Enum {
    readonly isReportEquivocation: boolean;
    readonly asReportEquivocation: {
      readonly equivocationProof: SpFinalityGrandpaEquivocationProof;
      readonly keyOwnerProof: SpSessionMembershipProof;
    } & Struct;
    readonly isReportEquivocationUnsigned: boolean;
    readonly asReportEquivocationUnsigned: {
      readonly equivocationProof: SpFinalityGrandpaEquivocationProof;
      readonly keyOwnerProof: SpSessionMembershipProof;
    } & Struct;
    readonly isNoteStalled: boolean;
    readonly asNoteStalled: {
      readonly delay: u32;
      readonly bestFinalizedBlockNumber: u32;
    } & Struct;
    readonly type: 'ReportEquivocation' | 'ReportEquivocationUnsigned' | 'NoteStalled';
  }

  /** @name SpFinalityGrandpaEquivocationProof (455) */
  export interface SpFinalityGrandpaEquivocationProof extends Struct {
    readonly setId: u64;
    readonly equivocation: SpFinalityGrandpaEquivocation;
  }

  /** @name SpFinalityGrandpaEquivocation (456) */
  export interface SpFinalityGrandpaEquivocation extends Enum {
    readonly isPrevote: boolean;
    readonly asPrevote: FinalityGrandpaEquivocationPrevote;
    readonly isPrecommit: boolean;
    readonly asPrecommit: FinalityGrandpaEquivocationPrecommit;
    readonly type: 'Prevote' | 'Precommit';
  }

  /** @name FinalityGrandpaEquivocationPrevote (457) */
  export interface FinalityGrandpaEquivocationPrevote extends Struct {
    readonly roundNumber: u64;
    readonly identity: SpFinalityGrandpaAppPublic;
    readonly first: ITuple<[FinalityGrandpaPrevote, SpFinalityGrandpaAppSignature]>;
    readonly second: ITuple<[FinalityGrandpaPrevote, SpFinalityGrandpaAppSignature]>;
  }

  /** @name FinalityGrandpaPrevote (458) */
  export interface FinalityGrandpaPrevote extends Struct {
    readonly targetHash: H256;
    readonly targetNumber: u32;
  }

  /** @name SpFinalityGrandpaAppSignature (459) */
  export interface SpFinalityGrandpaAppSignature extends SpCoreEd25519Signature {}

  /** @name SpCoreEd25519Signature (460) */
  export interface SpCoreEd25519Signature extends U8aFixed {}

  /** @name FinalityGrandpaEquivocationPrecommit (462) */
  export interface FinalityGrandpaEquivocationPrecommit extends Struct {
    readonly roundNumber: u64;
    readonly identity: SpFinalityGrandpaAppPublic;
    readonly first: ITuple<[FinalityGrandpaPrecommit, SpFinalityGrandpaAppSignature]>;
    readonly second: ITuple<[FinalityGrandpaPrecommit, SpFinalityGrandpaAppSignature]>;
  }

  /** @name FinalityGrandpaPrecommit (463) */
  export interface FinalityGrandpaPrecommit extends Struct {
    readonly targetHash: H256;
    readonly targetNumber: u32;
  }

  /** @name PalletImOnlineCall (465) */
  export interface PalletImOnlineCall extends Enum {
    readonly isHeartbeat: boolean;
    readonly asHeartbeat: {
      readonly heartbeat: PalletImOnlineHeartbeat;
      readonly signature: PalletImOnlineSr25519AppSr25519Signature;
    } & Struct;
    readonly type: 'Heartbeat';
  }

  /** @name PalletImOnlineHeartbeat (466) */
  export interface PalletImOnlineHeartbeat extends Struct {
    readonly blockNumber: u32;
    readonly networkState: SpCoreOffchainOpaqueNetworkState;
    readonly sessionIndex: u32;
    readonly authorityIndex: u32;
    readonly validatorsLen: u32;
  }

  /** @name SpCoreOffchainOpaqueNetworkState (467) */
  export interface SpCoreOffchainOpaqueNetworkState extends Struct {
    readonly peerId: Bytes;
    readonly externalAddresses: Vec<Bytes>;
  }

  /** @name PalletImOnlineSr25519AppSr25519Signature (471) */
  export interface PalletImOnlineSr25519AppSr25519Signature extends SpCoreSr25519Signature {}

  /** @name SpCoreSr25519Signature (472) */
  export interface SpCoreSr25519Signature extends U8aFixed {}

  /** @name PalletSudoCall (473) */
  export interface PalletSudoCall extends Enum {
    readonly isSudo: boolean;
    readonly asSudo: {
      readonly call: Call;
    } & Struct;
    readonly isSudoUncheckedWeight: boolean;
    readonly asSudoUncheckedWeight: {
      readonly call: Call;
      readonly weight: u64;
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
    readonly type: 'Sudo' | 'SudoUncheckedWeight' | 'SetKey' | 'SudoAs';
  }

  /** @name PalletAssetCall (474) */
  export interface PalletAssetCall extends Enum {
    readonly isRegisterTicker: boolean;
    readonly asRegisterTicker: {
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
      readonly name: Bytes;
      readonly ticker: PolymeshPrimitivesTicker;
      readonly divisible: bool;
      readonly assetType: PolymeshPrimitivesAssetAssetType;
      readonly identifiers: Vec<PolymeshPrimitivesAssetIdentifier>;
      readonly fundingRound: Option<Bytes>;
      readonly disableIu: bool;
    } & Struct;
    readonly isFreeze: boolean;
    readonly asFreeze: {
      readonly ticker: PolymeshPrimitivesTicker;
    } & Struct;
    readonly isUnfreeze: boolean;
    readonly asUnfreeze: {
      readonly ticker: PolymeshPrimitivesTicker;
    } & Struct;
    readonly isRenameAsset: boolean;
    readonly asRenameAsset: {
      readonly ticker: PolymeshPrimitivesTicker;
      readonly name: Bytes;
    } & Struct;
    readonly isIssue: boolean;
    readonly asIssue: {
      readonly ticker: PolymeshPrimitivesTicker;
      readonly amount: u128;
    } & Struct;
    readonly isRedeem: boolean;
    readonly asRedeem: {
      readonly ticker: PolymeshPrimitivesTicker;
      readonly value: u128;
    } & Struct;
    readonly isMakeDivisible: boolean;
    readonly asMakeDivisible: {
      readonly ticker: PolymeshPrimitivesTicker;
    } & Struct;
    readonly isAddDocuments: boolean;
    readonly asAddDocuments: {
      readonly docs: Vec<PolymeshPrimitivesDocument>;
      readonly ticker: PolymeshPrimitivesTicker;
    } & Struct;
    readonly isRemoveDocuments: boolean;
    readonly asRemoveDocuments: {
      readonly ids: Vec<u32>;
      readonly ticker: PolymeshPrimitivesTicker;
    } & Struct;
    readonly isSetFundingRound: boolean;
    readonly asSetFundingRound: {
      readonly ticker: PolymeshPrimitivesTicker;
      readonly name: Bytes;
    } & Struct;
    readonly isUpdateIdentifiers: boolean;
    readonly asUpdateIdentifiers: {
      readonly ticker: PolymeshPrimitivesTicker;
      readonly identifiers: Vec<PolymeshPrimitivesAssetIdentifier>;
    } & Struct;
    readonly isClaimClassicTicker: boolean;
    readonly asClaimClassicTicker: {
      readonly ticker: PolymeshPrimitivesTicker;
      readonly ethereumSignature: PolymeshPrimitivesEthereumEcdsaSignature;
    } & Struct;
    readonly isReserveClassicTicker: boolean;
    readonly asReserveClassicTicker: {
      readonly classicTickerImport: PalletAssetClassicTickerImport;
      readonly contractDid: PolymeshPrimitivesIdentityId;
      readonly config: PalletAssetTickerRegistrationConfig;
    } & Struct;
    readonly isControllerTransfer: boolean;
    readonly asControllerTransfer: {
      readonly ticker: PolymeshPrimitivesTicker;
      readonly value: u128;
      readonly fromPortfolio: PolymeshPrimitivesIdentityIdPortfolioId;
    } & Struct;
    readonly isRegisterCustomAssetType: boolean;
    readonly asRegisterCustomAssetType: {
      readonly ty: Bytes;
    } & Struct;
    readonly isCreateAssetWithCustomType: boolean;
    readonly asCreateAssetWithCustomType: {
      readonly name: Bytes;
      readonly ticker: PolymeshPrimitivesTicker;
      readonly divisible: bool;
      readonly customAssetType: Bytes;
      readonly identifiers: Vec<PolymeshPrimitivesAssetIdentifier>;
      readonly fundingRound: Option<Bytes>;
      readonly disableIu: bool;
    } & Struct;
    readonly isSetAssetMetadata: boolean;
    readonly asSetAssetMetadata: {
      readonly ticker: PolymeshPrimitivesTicker;
      readonly key: PolymeshPrimitivesAssetMetadataAssetMetadataKey;
      readonly value: Bytes;
      readonly detail: Option<PolymeshPrimitivesAssetMetadataAssetMetadataValueDetail>;
    } & Struct;
    readonly isSetAssetMetadataDetails: boolean;
    readonly asSetAssetMetadataDetails: {
      readonly ticker: PolymeshPrimitivesTicker;
      readonly key: PolymeshPrimitivesAssetMetadataAssetMetadataKey;
      readonly detail: PolymeshPrimitivesAssetMetadataAssetMetadataValueDetail;
    } & Struct;
    readonly isRegisterAndSetLocalAssetMetadata: boolean;
    readonly asRegisterAndSetLocalAssetMetadata: {
      readonly ticker: PolymeshPrimitivesTicker;
      readonly name: Bytes;
      readonly spec: PolymeshPrimitivesAssetMetadataAssetMetadataSpec;
      readonly value: Bytes;
      readonly detail: Option<PolymeshPrimitivesAssetMetadataAssetMetadataValueDetail>;
    } & Struct;
    readonly isRegisterAssetMetadataLocalType: boolean;
    readonly asRegisterAssetMetadataLocalType: {
      readonly ticker: PolymeshPrimitivesTicker;
      readonly name: Bytes;
      readonly spec: PolymeshPrimitivesAssetMetadataAssetMetadataSpec;
    } & Struct;
    readonly isRegisterAssetMetadataGlobalType: boolean;
    readonly asRegisterAssetMetadataGlobalType: {
      readonly name: Bytes;
      readonly spec: PolymeshPrimitivesAssetMetadataAssetMetadataSpec;
    } & Struct;
    readonly type: 'RegisterTicker' | 'AcceptTickerTransfer' | 'AcceptAssetOwnershipTransfer' | 'CreateAsset' | 'Freeze' | 'Unfreeze' | 'RenameAsset' | 'Issue' | 'Redeem' | 'MakeDivisible' | 'AddDocuments' | 'RemoveDocuments' | 'SetFundingRound' | 'UpdateIdentifiers' | 'ClaimClassicTicker' | 'ReserveClassicTicker' | 'ControllerTransfer' | 'RegisterCustomAssetType' | 'CreateAssetWithCustomType' | 'SetAssetMetadata' | 'SetAssetMetadataDetails' | 'RegisterAndSetLocalAssetMetadata' | 'RegisterAssetMetadataLocalType' | 'RegisterAssetMetadataGlobalType';
  }

  /** @name PolymeshPrimitivesEthereumEcdsaSignature (477) */
  export interface PolymeshPrimitivesEthereumEcdsaSignature extends U8aFixed {}

  /** @name PalletAssetClassicTickerImport (479) */
  export interface PalletAssetClassicTickerImport extends Struct {
    readonly ethOwner: PolymeshPrimitivesEthereumEthereumAddress;
    readonly ticker: PolymeshPrimitivesTicker;
    readonly isContract: bool;
    readonly isCreated: bool;
  }

  /** @name PalletAssetTickerRegistrationConfig (480) */
  export interface PalletAssetTickerRegistrationConfig extends Struct {
    readonly maxTickerLength: u8;
    readonly registrationLength: Option<u64>;
  }

  /** @name PolymeshPrimitivesAssetMetadataAssetMetadataKey (481) */
  export interface PolymeshPrimitivesAssetMetadataAssetMetadataKey extends Enum {
    readonly isGlobal: boolean;
    readonly asGlobal: u64;
    readonly isLocal: boolean;
    readonly asLocal: u64;
    readonly type: 'Global' | 'Local';
  }

  /** @name PalletCorporateActionsDistributionCall (482) */
  export interface PalletCorporateActionsDistributionCall extends Enum {
    readonly isDistribute: boolean;
    readonly asDistribute: {
      readonly caId: PalletCorporateActionsCaId;
      readonly portfolio: Option<u64>;
      readonly currency: PolymeshPrimitivesTicker;
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

  /** @name PalletAssetCheckpointCall (484) */
  export interface PalletAssetCheckpointCall extends Enum {
    readonly isCreateCheckpoint: boolean;
    readonly asCreateCheckpoint: {
      readonly ticker: PolymeshPrimitivesTicker;
    } & Struct;
    readonly isSetSchedulesMaxComplexity: boolean;
    readonly asSetSchedulesMaxComplexity: {
      readonly maxComplexity: u64;
    } & Struct;
    readonly isCreateSchedule: boolean;
    readonly asCreateSchedule: {
      readonly ticker: PolymeshPrimitivesTicker;
      readonly schedule: PalletAssetCheckpointScheduleSpec;
    } & Struct;
    readonly isRemoveSchedule: boolean;
    readonly asRemoveSchedule: {
      readonly ticker: PolymeshPrimitivesTicker;
      readonly id: u64;
    } & Struct;
    readonly type: 'CreateCheckpoint' | 'SetSchedulesMaxComplexity' | 'CreateSchedule' | 'RemoveSchedule';
  }

  /** @name PalletAssetCheckpointScheduleSpec (485) */
  export interface PalletAssetCheckpointScheduleSpec extends Struct {
    readonly start: Option<u64>;
    readonly period: PolymeshPrimitivesCalendarCalendarPeriod;
    readonly remaining: u32;
  }

  /** @name PalletComplianceManagerCall (486) */
  export interface PalletComplianceManagerCall extends Enum {
    readonly isAddComplianceRequirement: boolean;
    readonly asAddComplianceRequirement: {
      readonly ticker: PolymeshPrimitivesTicker;
      readonly senderConditions: Vec<PolymeshPrimitivesCondition>;
      readonly receiverConditions: Vec<PolymeshPrimitivesCondition>;
    } & Struct;
    readonly isRemoveComplianceRequirement: boolean;
    readonly asRemoveComplianceRequirement: {
      readonly ticker: PolymeshPrimitivesTicker;
      readonly id: u32;
    } & Struct;
    readonly isReplaceAssetCompliance: boolean;
    readonly asReplaceAssetCompliance: {
      readonly ticker: PolymeshPrimitivesTicker;
      readonly assetCompliance: Vec<PolymeshPrimitivesComplianceManagerComplianceRequirement>;
    } & Struct;
    readonly isResetAssetCompliance: boolean;
    readonly asResetAssetCompliance: {
      readonly ticker: PolymeshPrimitivesTicker;
    } & Struct;
    readonly isPauseAssetCompliance: boolean;
    readonly asPauseAssetCompliance: {
      readonly ticker: PolymeshPrimitivesTicker;
    } & Struct;
    readonly isResumeAssetCompliance: boolean;
    readonly asResumeAssetCompliance: {
      readonly ticker: PolymeshPrimitivesTicker;
    } & Struct;
    readonly isAddDefaultTrustedClaimIssuer: boolean;
    readonly asAddDefaultTrustedClaimIssuer: {
      readonly ticker: PolymeshPrimitivesTicker;
      readonly issuer: PolymeshPrimitivesConditionTrustedIssuer;
    } & Struct;
    readonly isRemoveDefaultTrustedClaimIssuer: boolean;
    readonly asRemoveDefaultTrustedClaimIssuer: {
      readonly ticker: PolymeshPrimitivesTicker;
      readonly issuer: PolymeshPrimitivesIdentityId;
    } & Struct;
    readonly isChangeComplianceRequirement: boolean;
    readonly asChangeComplianceRequirement: {
      readonly ticker: PolymeshPrimitivesTicker;
      readonly newReq: PolymeshPrimitivesComplianceManagerComplianceRequirement;
    } & Struct;
    readonly type: 'AddComplianceRequirement' | 'RemoveComplianceRequirement' | 'ReplaceAssetCompliance' | 'ResetAssetCompliance' | 'PauseAssetCompliance' | 'ResumeAssetCompliance' | 'AddDefaultTrustedClaimIssuer' | 'RemoveDefaultTrustedClaimIssuer' | 'ChangeComplianceRequirement';
  }

  /** @name PalletCorporateActionsCall (487) */
  export interface PalletCorporateActionsCall extends Enum {
    readonly isSetMaxDetailsLength: boolean;
    readonly asSetMaxDetailsLength: {
      readonly length: u32;
    } & Struct;
    readonly isSetDefaultTargets: boolean;
    readonly asSetDefaultTargets: {
      readonly ticker: PolymeshPrimitivesTicker;
      readonly targets: PalletCorporateActionsTargetIdentities;
    } & Struct;
    readonly isSetDefaultWithholdingTax: boolean;
    readonly asSetDefaultWithholdingTax: {
      readonly ticker: PolymeshPrimitivesTicker;
      readonly tax: Permill;
    } & Struct;
    readonly isSetDidWithholdingTax: boolean;
    readonly asSetDidWithholdingTax: {
      readonly ticker: PolymeshPrimitivesTicker;
      readonly taxedDid: PolymeshPrimitivesIdentityId;
      readonly tax: Option<Permill>;
    } & Struct;
    readonly isInitiateCorporateAction: boolean;
    readonly asInitiateCorporateAction: {
      readonly ticker: PolymeshPrimitivesTicker;
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
      readonly currency: PolymeshPrimitivesTicker;
      readonly perShare: u128;
      readonly amount: u128;
      readonly paymentAt: u64;
      readonly expiresAt: Option<u64>;
    } & Struct;
    readonly type: 'SetMaxDetailsLength' | 'SetDefaultTargets' | 'SetDefaultWithholdingTax' | 'SetDidWithholdingTax' | 'InitiateCorporateAction' | 'LinkCaDoc' | 'RemoveCa' | 'ChangeRecordDate' | 'InitiateCorporateActionAndDistribute';
  }

  /** @name PalletCorporateActionsRecordDateSpec (489) */
  export interface PalletCorporateActionsRecordDateSpec extends Enum {
    readonly isScheduled: boolean;
    readonly asScheduled: u64;
    readonly isExistingSchedule: boolean;
    readonly asExistingSchedule: u64;
    readonly isExisting: boolean;
    readonly asExisting: u64;
    readonly type: 'Scheduled' | 'ExistingSchedule' | 'Existing';
  }

  /** @name PalletCorporateActionsInitiateCorporateActionArgs (492) */
  export interface PalletCorporateActionsInitiateCorporateActionArgs extends Struct {
    readonly ticker: PolymeshPrimitivesTicker;
    readonly kind: PalletCorporateActionsCaKind;
    readonly declDate: u64;
    readonly recordDate: Option<PalletCorporateActionsRecordDateSpec>;
    readonly details: Bytes;
    readonly targets: Option<PalletCorporateActionsTargetIdentities>;
    readonly defaultWithholdingTax: Option<Permill>;
    readonly withholdingTax: Option<Vec<ITuple<[PolymeshPrimitivesIdentityId, Permill]>>>;
  }

  /** @name PalletCorporateActionsBallotCall (493) */
  export interface PalletCorporateActionsBallotCall extends Enum {
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

  /** @name PalletPipsCall (494) */
  export interface PalletPipsCall extends Enum {
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
      readonly expiry: PolymeshCommonUtilitiesMaybeBlock;
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

  /** @name PalletPipsSnapshotResult (497) */
  export interface PalletPipsSnapshotResult extends Enum {
    readonly isApprove: boolean;
    readonly isReject: boolean;
    readonly isSkip: boolean;
    readonly type: 'Approve' | 'Reject' | 'Skip';
  }

  /** @name PalletPortfolioCall (498) */
  export interface PalletPortfolioCall extends Enum {
    readonly isCreatePortfolio: boolean;
    readonly asCreatePortfolio: {
      readonly name: Bytes;
    } & Struct;
    readonly isDeletePortfolio: boolean;
    readonly asDeletePortfolio: {
      readonly num: u64;
    } & Struct;
    readonly isMovePortfolioFunds: boolean;
    readonly asMovePortfolioFunds: {
      readonly from: PolymeshPrimitivesIdentityIdPortfolioId;
      readonly to: PolymeshPrimitivesIdentityIdPortfolioId;
      readonly items: Vec<PalletPortfolioMovePortfolioItem>;
    } & Struct;
    readonly isRenamePortfolio: boolean;
    readonly asRenamePortfolio: {
      readonly num: u64;
      readonly toName: Bytes;
    } & Struct;
    readonly isQuitPortfolioCustody: boolean;
    readonly asQuitPortfolioCustody: {
      readonly pid: PolymeshPrimitivesIdentityIdPortfolioId;
    } & Struct;
    readonly isAcceptPortfolioCustody: boolean;
    readonly asAcceptPortfolioCustody: {
      readonly authId: u64;
    } & Struct;
    readonly type: 'CreatePortfolio' | 'DeletePortfolio' | 'MovePortfolioFunds' | 'RenamePortfolio' | 'QuitPortfolioCustody' | 'AcceptPortfolioCustody';
  }

  /** @name PalletPortfolioMovePortfolioItem (500) */
  export interface PalletPortfolioMovePortfolioItem extends Struct {
    readonly ticker: PolymeshPrimitivesTicker;
    readonly amount: u128;
    readonly memo: Option<PolymeshCommonUtilitiesBalancesMemo>;
  }

  /** @name PalletProtocolFeeCall (501) */
  export interface PalletProtocolFeeCall extends Enum {
    readonly isChangeCoefficient: boolean;
    readonly asChangeCoefficient: {
      readonly coefficient: PolymeshPrimitivesPosRatio;
    } & Struct;
    readonly isChangeBaseFee: boolean;
    readonly asChangeBaseFee: {
      readonly op: PolymeshCommonUtilitiesProtocolFeeProtocolOp;
      readonly baseFee: u128;
    } & Struct;
    readonly type: 'ChangeCoefficient' | 'ChangeBaseFee';
  }

  /** @name PolymeshCommonUtilitiesProtocolFeeProtocolOp (502) */
  export interface PolymeshCommonUtilitiesProtocolFeeProtocolOp extends Enum {
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
    readonly type: 'AssetRegisterTicker' | 'AssetIssue' | 'AssetAddDocuments' | 'AssetCreateAsset' | 'CheckpointCreateSchedule' | 'ComplianceManagerAddComplianceRequirement' | 'IdentityCddRegisterDid' | 'IdentityAddClaim' | 'IdentityAddSecondaryKeysWithAuthorization' | 'PipsPropose' | 'ContractsPutCode' | 'CorporateBallotAttachBallot' | 'CapitalDistributionDistribute';
  }

  /** @name PalletSchedulerCall (503) */
  export interface PalletSchedulerCall extends Enum {
    readonly isSchedule: boolean;
    readonly asSchedule: {
      readonly when: u32;
      readonly maybePeriodic: Option<ITuple<[u32, u32]>>;
      readonly priority: u8;
      readonly call: FrameSupportScheduleMaybeHashed;
    } & Struct;
    readonly isCancel: boolean;
    readonly asCancel: {
      readonly when: u32;
      readonly index: u32;
    } & Struct;
    readonly isScheduleNamed: boolean;
    readonly asScheduleNamed: {
      readonly id: Bytes;
      readonly when: u32;
      readonly maybePeriodic: Option<ITuple<[u32, u32]>>;
      readonly priority: u8;
      readonly call: FrameSupportScheduleMaybeHashed;
    } & Struct;
    readonly isCancelNamed: boolean;
    readonly asCancelNamed: {
      readonly id: Bytes;
    } & Struct;
    readonly isScheduleAfter: boolean;
    readonly asScheduleAfter: {
      readonly after: u32;
      readonly maybePeriodic: Option<ITuple<[u32, u32]>>;
      readonly priority: u8;
      readonly call: FrameSupportScheduleMaybeHashed;
    } & Struct;
    readonly isScheduleNamedAfter: boolean;
    readonly asScheduleNamedAfter: {
      readonly id: Bytes;
      readonly after: u32;
      readonly maybePeriodic: Option<ITuple<[u32, u32]>>;
      readonly priority: u8;
      readonly call: FrameSupportScheduleMaybeHashed;
    } & Struct;
    readonly type: 'Schedule' | 'Cancel' | 'ScheduleNamed' | 'CancelNamed' | 'ScheduleAfter' | 'ScheduleNamedAfter';
  }

  /** @name FrameSupportScheduleMaybeHashed (505) */
  export interface FrameSupportScheduleMaybeHashed extends Enum {
    readonly isValue: boolean;
    readonly asValue: Call;
    readonly isHash: boolean;
    readonly asHash: H256;
    readonly type: 'Value' | 'Hash';
  }

  /** @name PalletSettlementCall (506) */
  export interface PalletSettlementCall extends Enum {
    readonly isCreateVenue: boolean;
    readonly asCreateVenue: {
      readonly details: Bytes;
      readonly signers: Vec<AccountId32>;
      readonly typ: PalletSettlementVenueType;
    } & Struct;
    readonly isUpdateVenueDetails: boolean;
    readonly asUpdateVenueDetails: {
      readonly id: u64;
      readonly details: Bytes;
    } & Struct;
    readonly isUpdateVenueType: boolean;
    readonly asUpdateVenueType: {
      readonly id: u64;
      readonly typ: PalletSettlementVenueType;
    } & Struct;
    readonly isAddInstruction: boolean;
    readonly asAddInstruction: {
      readonly venueId: u64;
      readonly settlementType: PalletSettlementSettlementType;
      readonly tradeDate: Option<u64>;
      readonly valueDate: Option<u64>;
      readonly legs: Vec<PalletSettlementLeg>;
    } & Struct;
    readonly isAddAndAffirmInstruction: boolean;
    readonly asAddAndAffirmInstruction: {
      readonly venueId: u64;
      readonly settlementType: PalletSettlementSettlementType;
      readonly tradeDate: Option<u64>;
      readonly valueDate: Option<u64>;
      readonly legs: Vec<PalletSettlementLeg>;
      readonly portfolios: Vec<PolymeshPrimitivesIdentityIdPortfolioId>;
    } & Struct;
    readonly isAffirmInstruction: boolean;
    readonly asAffirmInstruction: {
      readonly id: u64;
      readonly portfolios: Vec<PolymeshPrimitivesIdentityIdPortfolioId>;
      readonly maxLegsCount: u32;
    } & Struct;
    readonly isWithdrawAffirmation: boolean;
    readonly asWithdrawAffirmation: {
      readonly id: u64;
      readonly portfolios: Vec<PolymeshPrimitivesIdentityIdPortfolioId>;
      readonly maxLegsCount: u32;
    } & Struct;
    readonly isRejectInstruction: boolean;
    readonly asRejectInstruction: {
      readonly id: u64;
      readonly portfolio: PolymeshPrimitivesIdentityIdPortfolioId;
      readonly numOfLegs: u32;
    } & Struct;
    readonly isAffirmWithReceipts: boolean;
    readonly asAffirmWithReceipts: {
      readonly id: u64;
      readonly receiptDetails: Vec<PalletSettlementReceiptDetails>;
      readonly portfolios: Vec<PolymeshPrimitivesIdentityIdPortfolioId>;
      readonly maxLegsCount: u32;
    } & Struct;
    readonly isClaimReceipt: boolean;
    readonly asClaimReceipt: {
      readonly id: u64;
      readonly receiptDetails: PalletSettlementReceiptDetails;
    } & Struct;
    readonly isUnclaimReceipt: boolean;
    readonly asUnclaimReceipt: {
      readonly instructionId: u64;
      readonly legId: u64;
    } & Struct;
    readonly isSetVenueFiltering: boolean;
    readonly asSetVenueFiltering: {
      readonly ticker: PolymeshPrimitivesTicker;
      readonly enabled: bool;
    } & Struct;
    readonly isAllowVenues: boolean;
    readonly asAllowVenues: {
      readonly ticker: PolymeshPrimitivesTicker;
      readonly venues: Vec<u64>;
    } & Struct;
    readonly isDisallowVenues: boolean;
    readonly asDisallowVenues: {
      readonly ticker: PolymeshPrimitivesTicker;
      readonly venues: Vec<u64>;
    } & Struct;
    readonly isChangeReceiptValidity: boolean;
    readonly asChangeReceiptValidity: {
      readonly receiptUid: u64;
      readonly validity: bool;
    } & Struct;
    readonly isExecuteScheduledInstruction: boolean;
    readonly asExecuteScheduledInstruction: {
      readonly id: u64;
      readonly legsCount: u32;
    } & Struct;
    readonly isRescheduleInstruction: boolean;
    readonly asRescheduleInstruction: {
      readonly id: u64;
    } & Struct;
    readonly type: 'CreateVenue' | 'UpdateVenueDetails' | 'UpdateVenueType' | 'AddInstruction' | 'AddAndAffirmInstruction' | 'AffirmInstruction' | 'WithdrawAffirmation' | 'RejectInstruction' | 'AffirmWithReceipts' | 'ClaimReceipt' | 'UnclaimReceipt' | 'SetVenueFiltering' | 'AllowVenues' | 'DisallowVenues' | 'ChangeReceiptValidity' | 'ExecuteScheduledInstruction' | 'RescheduleInstruction';
  }

  /** @name PalletSettlementReceiptDetails (508) */
  export interface PalletSettlementReceiptDetails extends Struct {
    readonly receiptUid: u64;
    readonly legId: u64;
    readonly signer: AccountId32;
    readonly signature: SpRuntimeMultiSignature;
    readonly metadata: Bytes;
  }

  /** @name SpRuntimeMultiSignature (509) */
  export interface SpRuntimeMultiSignature extends Enum {
    readonly isEd25519: boolean;
    readonly asEd25519: SpCoreEd25519Signature;
    readonly isSr25519: boolean;
    readonly asSr25519: SpCoreSr25519Signature;
    readonly isEcdsa: boolean;
    readonly asEcdsa: SpCoreEcdsaSignature;
    readonly type: 'Ed25519' | 'Sr25519' | 'Ecdsa';
  }

  /** @name SpCoreEcdsaSignature (510) */
  export interface SpCoreEcdsaSignature extends U8aFixed {}

  /** @name PalletStatisticsCall (511) */
  export interface PalletStatisticsCall extends Enum {
    readonly isSetActiveAssetStats: boolean;
    readonly asSetActiveAssetStats: {
      readonly asset: PolymeshPrimitivesStatisticsAssetScope;
      readonly statTypes: BTreeSetStatType;
    } & Struct;
    readonly isBatchUpdateAssetStats: boolean;
    readonly asBatchUpdateAssetStats: {
      readonly asset: PolymeshPrimitivesStatisticsAssetScope;
      readonly statType: PolymeshPrimitivesStatisticsStatType;
      readonly values: BTreeSetStatUpdate;
    } & Struct;
    readonly isSetAssetTransferCompliance: boolean;
    readonly asSetAssetTransferCompliance: {
      readonly asset: PolymeshPrimitivesStatisticsAssetScope;
      readonly transferConditions: BTreeSetTransferCondition;
    } & Struct;
    readonly isSetEntitiesExempt: boolean;
    readonly asSetEntitiesExempt: {
      readonly isExempt: bool;
      readonly exemptKey: PolymeshPrimitivesTransferComplianceTransferConditionExemptKey;
      readonly entities: BTreeSetIdentityId;
    } & Struct;
    readonly type: 'SetActiveAssetStats' | 'BatchUpdateAssetStats' | 'SetAssetTransferCompliance' | 'SetEntitiesExempt';
  }

  /** @name BTreeSetStatType (512) */
  export interface BTreeSetStatType extends Vec<PolymeshPrimitivesStatisticsStatType> {}

  /** @name BTreeSetStatUpdate (513) */
  export interface BTreeSetStatUpdate extends Vec<PolymeshPrimitivesStatisticsStatUpdate> {}

  /** @name BTreeSetTransferCondition (514) */
  export interface BTreeSetTransferCondition extends Vec<PolymeshPrimitivesTransferComplianceTransferCondition> {}

  /** @name BTreeSetIdentityId (515) */
  export interface BTreeSetIdentityId extends Vec<PolymeshPrimitivesIdentityId> {}

  /** @name PalletStoCall (516) */
  export interface PalletStoCall extends Enum {
    readonly isCreateFundraiser: boolean;
    readonly asCreateFundraiser: {
      readonly offeringPortfolio: PolymeshPrimitivesIdentityIdPortfolioId;
      readonly offeringAsset: PolymeshPrimitivesTicker;
      readonly raisingPortfolio: PolymeshPrimitivesIdentityIdPortfolioId;
      readonly raisingAsset: PolymeshPrimitivesTicker;
      readonly tiers: Vec<PalletStoPriceTier>;
      readonly venueId: u64;
      readonly start: Option<u64>;
      readonly end: Option<u64>;
      readonly minimumInvestment: u128;
      readonly fundraiserName: Bytes;
    } & Struct;
    readonly isInvest: boolean;
    readonly asInvest: {
      readonly investmentPortfolio: PolymeshPrimitivesIdentityIdPortfolioId;
      readonly fundingPortfolio: PolymeshPrimitivesIdentityIdPortfolioId;
      readonly offeringAsset: PolymeshPrimitivesTicker;
      readonly id: u64;
      readonly purchaseAmount: u128;
      readonly maxPrice: Option<u128>;
      readonly receipt: Option<PalletSettlementReceiptDetails>;
    } & Struct;
    readonly isFreezeFundraiser: boolean;
    readonly asFreezeFundraiser: {
      readonly offeringAsset: PolymeshPrimitivesTicker;
      readonly id: u64;
    } & Struct;
    readonly isUnfreezeFundraiser: boolean;
    readonly asUnfreezeFundraiser: {
      readonly offeringAsset: PolymeshPrimitivesTicker;
      readonly id: u64;
    } & Struct;
    readonly isModifyFundraiserWindow: boolean;
    readonly asModifyFundraiserWindow: {
      readonly offeringAsset: PolymeshPrimitivesTicker;
      readonly id: u64;
      readonly start: u64;
      readonly end: Option<u64>;
    } & Struct;
    readonly isStop: boolean;
    readonly asStop: {
      readonly offeringAsset: PolymeshPrimitivesTicker;
      readonly id: u64;
    } & Struct;
    readonly type: 'CreateFundraiser' | 'Invest' | 'FreezeFundraiser' | 'UnfreezeFundraiser' | 'ModifyFundraiserWindow' | 'Stop';
  }

  /** @name PalletStoPriceTier (518) */
  export interface PalletStoPriceTier extends Struct {
    readonly total: u128;
    readonly price: u128;
  }

  /** @name PalletTreasuryCall (520) */
  export interface PalletTreasuryCall extends Enum {
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

  /** @name PolymeshPrimitivesBeneficiary (522) */
  export interface PolymeshPrimitivesBeneficiary extends Struct {
    readonly id: PolymeshPrimitivesIdentityId;
    readonly amount: u128;
  }

  /** @name PalletUtilityCall (523) */
  export interface PalletUtilityCall extends Enum {
    readonly isBatch: boolean;
    readonly asBatch: {
      readonly calls: Vec<Call>;
    } & Struct;
    readonly isBatchAtomic: boolean;
    readonly asBatchAtomic: {
      readonly calls: Vec<Call>;
    } & Struct;
    readonly isBatchOptimistic: boolean;
    readonly asBatchOptimistic: {
      readonly calls: Vec<Call>;
    } & Struct;
    readonly isRelayTx: boolean;
    readonly asRelayTx: {
      readonly target: AccountId32;
      readonly signature: SpRuntimeMultiSignature;
      readonly call: PalletUtilityUniqueCall;
    } & Struct;
    readonly type: 'Batch' | 'BatchAtomic' | 'BatchOptimistic' | 'RelayTx';
  }

  /** @name PalletUtilityUniqueCall (525) */
  export interface PalletUtilityUniqueCall extends Struct {
    readonly nonce: u64;
    readonly call: Call;
  }

  /** @name PalletBaseCall (526) */
  export type PalletBaseCall = Null;

  /** @name PalletExternalAgentsCall (527) */
  export interface PalletExternalAgentsCall extends Enum {
    readonly isCreateGroup: boolean;
    readonly asCreateGroup: {
      readonly ticker: PolymeshPrimitivesTicker;
      readonly perms: PolymeshPrimitivesSubsetSubsetRestrictionPalletPermissions;
    } & Struct;
    readonly isSetGroupPermissions: boolean;
    readonly asSetGroupPermissions: {
      readonly ticker: PolymeshPrimitivesTicker;
      readonly id: u32;
      readonly perms: PolymeshPrimitivesSubsetSubsetRestrictionPalletPermissions;
    } & Struct;
    readonly isRemoveAgent: boolean;
    readonly asRemoveAgent: {
      readonly ticker: PolymeshPrimitivesTicker;
      readonly agent: PolymeshPrimitivesIdentityId;
    } & Struct;
    readonly isAbdicate: boolean;
    readonly asAbdicate: {
      readonly ticker: PolymeshPrimitivesTicker;
    } & Struct;
    readonly isChangeGroup: boolean;
    readonly asChangeGroup: {
      readonly ticker: PolymeshPrimitivesTicker;
      readonly agent: PolymeshPrimitivesIdentityId;
      readonly group: PolymeshPrimitivesAgentAgentGroup;
    } & Struct;
    readonly isAcceptBecomeAgent: boolean;
    readonly asAcceptBecomeAgent: {
      readonly authId: u64;
    } & Struct;
    readonly isCreateGroupAndAddAuth: boolean;
    readonly asCreateGroupAndAddAuth: {
      readonly ticker: PolymeshPrimitivesTicker;
      readonly perms: PolymeshPrimitivesSubsetSubsetRestrictionPalletPermissions;
      readonly target: PolymeshPrimitivesSecondaryKeySignatory;
    } & Struct;
    readonly isCreateAndChangeCustomGroup: boolean;
    readonly asCreateAndChangeCustomGroup: {
      readonly ticker: PolymeshPrimitivesTicker;
      readonly perms: PolymeshPrimitivesSubsetSubsetRestrictionPalletPermissions;
      readonly agent: PolymeshPrimitivesIdentityId;
    } & Struct;
    readonly type: 'CreateGroup' | 'SetGroupPermissions' | 'RemoveAgent' | 'Abdicate' | 'ChangeGroup' | 'AcceptBecomeAgent' | 'CreateGroupAndAddAuth' | 'CreateAndChangeCustomGroup';
  }

  /** @name PalletRelayerCall (528) */
  export interface PalletRelayerCall extends Enum {
    readonly isSetPayingKey: boolean;
    readonly asSetPayingKey: {
      readonly userKey: AccountId32;
      readonly polyxLimit: u128;
    } & Struct;
    readonly isAcceptPayingKey: boolean;
    readonly asAcceptPayingKey: {
      readonly authId: u64;
    } & Struct;
    readonly isRemovePayingKey: boolean;
    readonly asRemovePayingKey: {
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
    readonly type: 'SetPayingKey' | 'AcceptPayingKey' | 'RemovePayingKey' | 'UpdatePolyxLimit' | 'IncreasePolyxLimit' | 'DecreasePolyxLimit';
  }

  /** @name PalletRewardsCall (529) */
  export interface PalletRewardsCall extends Enum {
    readonly isClaimItnReward: boolean;
    readonly asClaimItnReward: {
      readonly rewardAddress: AccountId32;
      readonly itnAddress: AccountId32;
      readonly signature: SpRuntimeMultiSignature;
    } & Struct;
    readonly isSetItnRewardStatus: boolean;
    readonly asSetItnRewardStatus: {
      readonly itnAddress: AccountId32;
      readonly status: PalletRewardsItnRewardStatus;
    } & Struct;
    readonly type: 'ClaimItnReward' | 'SetItnRewardStatus';
  }

  /** @name PalletRewardsItnRewardStatus (530) */
  export interface PalletRewardsItnRewardStatus extends Enum {
    readonly isUnclaimed: boolean;
    readonly asUnclaimed: u128;
    readonly isClaimed: boolean;
    readonly type: 'Unclaimed' | 'Claimed';
  }

  /** @name PolymeshContractsCall (531) */
  export interface PolymeshContractsCall extends Enum {
    readonly isCall: boolean;
    readonly asCall: {
      readonly contract: AccountId32;
      readonly value: u128;
      readonly gasLimit: u64;
      readonly storageDepositLimit: Option<u128>;
      readonly data: Bytes;
    } & Struct;
    readonly isInstantiateWithCode: boolean;
    readonly asInstantiateWithCode: {
      readonly endowment: u128;
      readonly gasLimit: u64;
      readonly storageDepositLimit: Option<u128>;
      readonly code: Bytes;
      readonly data: Bytes;
      readonly salt: Bytes;
      readonly perms: PolymeshPrimitivesSecondaryKeyPermissions;
    } & Struct;
    readonly isInstantiateWithHash: boolean;
    readonly asInstantiateWithHash: {
      readonly endowment: u128;
      readonly gasLimit: u64;
      readonly storageDepositLimit: Option<u128>;
      readonly codeHash: H256;
      readonly data: Bytes;
      readonly salt: Bytes;
      readonly perms: PolymeshPrimitivesSecondaryKeyPermissions;
    } & Struct;
    readonly type: 'Call' | 'InstantiateWithCode' | 'InstantiateWithHash';
  }

  /** @name PalletPreimageCall (532) */
  export interface PalletPreimageCall extends Enum {
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
    readonly type: 'NotePreimage' | 'UnnotePreimage' | 'RequestPreimage' | 'UnrequestPreimage';
  }

  /** @name PalletTestUtilsCall (533) */
  export interface PalletTestUtilsCall extends Enum {
    readonly isRegisterDid: boolean;
    readonly asRegisterDid: {
      readonly uid: PolymeshPrimitivesCddIdInvestorUid;
      readonly secondaryKeys: Vec<PolymeshPrimitivesSecondaryKey>;
    } & Struct;
    readonly isMockCddRegisterDid: boolean;
    readonly asMockCddRegisterDid: {
      readonly targetAccount: AccountId32;
    } & Struct;
    readonly isGetMyDid: boolean;
    readonly isGetCddOf: boolean;
    readonly asGetCddOf: {
      readonly of: AccountId32;
    } & Struct;
    readonly type: 'RegisterDid' | 'MockCddRegisterDid' | 'GetMyDid' | 'GetCddOf';
  }

  /** @name PalletCommitteePolymeshVotes (534) */
  export interface PalletCommitteePolymeshVotes extends Struct {
    readonly index: u32;
    readonly ayes: Vec<PolymeshPrimitivesIdentityId>;
    readonly nays: Vec<PolymeshPrimitivesIdentityId>;
    readonly expiry: PolymeshCommonUtilitiesMaybeBlock;
  }

  /** @name PalletCommitteeError (536) */
  export interface PalletCommitteeError extends Enum {
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

  /** @name PalletMultisigProposalDetails (546) */
  export interface PalletMultisigProposalDetails extends Struct {
    readonly approvals: u64;
    readonly rejections: u64;
    readonly status: PalletMultisigProposalStatus;
    readonly expiry: Option<u64>;
    readonly autoClose: bool;
  }

  /** @name PalletMultisigProposalStatus (547) */
  export interface PalletMultisigProposalStatus extends Enum {
    readonly isInvalid: boolean;
    readonly isActiveOrExpired: boolean;
    readonly isExecutionSuccessful: boolean;
    readonly isExecutionFailed: boolean;
    readonly isRejected: boolean;
    readonly type: 'Invalid' | 'ActiveOrExpired' | 'ExecutionSuccessful' | 'ExecutionFailed' | 'Rejected';
  }

  /** @name PalletMultisigError (549) */
  export interface PalletMultisigError extends Enum {
    readonly isCddMissing: boolean;
    readonly isProposalMissing: boolean;
    readonly isDecodingError: boolean;
    readonly isNoSigners: boolean;
    readonly isRequiredSignaturesOutOfBounds: boolean;
    readonly isNotASigner: boolean;
    readonly isNoSuchMultisig: boolean;
    readonly isNotEnoughSigners: boolean;
    readonly isNonceOverflow: boolean;
    readonly isAlreadyVoted: boolean;
    readonly isAlreadyASigner: boolean;
    readonly isFailedToChargeFee: boolean;
    readonly isIdentityNotCreator: boolean;
    readonly isChangeNotAllowed: boolean;
    readonly isSignerAlreadyLinkedToMultisig: boolean;
    readonly isSignerAlreadyLinkedToIdentity: boolean;
    readonly isMultisigNotAllowedToLinkToItself: boolean;
    readonly isMissingCurrentIdentity: boolean;
    readonly isNotPrimaryKey: boolean;
    readonly isProposalAlreadyRejected: boolean;
    readonly isProposalExpired: boolean;
    readonly isProposalAlreadyExecuted: boolean;
    readonly isMultisigMissingIdentity: boolean;
    readonly isFailedToSchedule: boolean;
    readonly isTooManySigners: boolean;
    readonly type: 'CddMissing' | 'ProposalMissing' | 'DecodingError' | 'NoSigners' | 'RequiredSignaturesOutOfBounds' | 'NotASigner' | 'NoSuchMultisig' | 'NotEnoughSigners' | 'NonceOverflow' | 'AlreadyVoted' | 'AlreadyASigner' | 'FailedToChargeFee' | 'IdentityNotCreator' | 'ChangeNotAllowed' | 'SignerAlreadyLinkedToMultisig' | 'SignerAlreadyLinkedToIdentity' | 'MultisigNotAllowedToLinkToItself' | 'MissingCurrentIdentity' | 'NotPrimaryKey' | 'ProposalAlreadyRejected' | 'ProposalExpired' | 'ProposalAlreadyExecuted' | 'MultisigMissingIdentity' | 'FailedToSchedule' | 'TooManySigners';
  }

  /** @name PalletBridgeBridgeTxDetail (551) */
  export interface PalletBridgeBridgeTxDetail extends Struct {
    readonly amount: u128;
    readonly status: PalletBridgeBridgeTxStatus;
    readonly executionBlock: u32;
    readonly txHash: H256;
  }

  /** @name PalletBridgeBridgeTxStatus (552) */
  export interface PalletBridgeBridgeTxStatus extends Enum {
    readonly isAbsent: boolean;
    readonly isPending: boolean;
    readonly asPending: u8;
    readonly isFrozen: boolean;
    readonly isTimelocked: boolean;
    readonly isHandled: boolean;
    readonly type: 'Absent' | 'Pending' | 'Frozen' | 'Timelocked' | 'Handled';
  }

  /** @name PalletBridgeError (555) */
  export interface PalletBridgeError extends Enum {
    readonly isControllerNotSet: boolean;
    readonly isBadCaller: boolean;
    readonly isBadAdmin: boolean;
    readonly isNoValidCdd: boolean;
    readonly isProposalAlreadyHandled: boolean;
    readonly isUnauthorized: boolean;
    readonly isFrozen: boolean;
    readonly isNotFrozen: boolean;
    readonly isFrozenTx: boolean;
    readonly isBridgeLimitReached: boolean;
    readonly isOverflow: boolean;
    readonly isDivisionByZero: boolean;
    readonly isTimelockedTx: boolean;
    readonly type: 'ControllerNotSet' | 'BadCaller' | 'BadAdmin' | 'NoValidCdd' | 'ProposalAlreadyHandled' | 'Unauthorized' | 'Frozen' | 'NotFrozen' | 'FrozenTx' | 'BridgeLimitReached' | 'Overflow' | 'DivisionByZero' | 'TimelockedTx';
  }

  /** @name PalletStakingStakingLedger (556) */
  export interface PalletStakingStakingLedger extends Struct {
    readonly stash: AccountId32;
    readonly total: Compact<u128>;
    readonly active: Compact<u128>;
    readonly unlocking: Vec<PalletStakingUnlockChunk>;
    readonly claimedRewards: Vec<u32>;
  }

  /** @name PalletStakingUnlockChunk (558) */
  export interface PalletStakingUnlockChunk extends Struct {
    readonly value: Compact<u128>;
    readonly era: Compact<u32>;
  }

  /** @name PalletStakingNominations (559) */
  export interface PalletStakingNominations extends Struct {
    readonly targets: Vec<AccountId32>;
    readonly submittedIn: u32;
    readonly suppressed: bool;
  }

  /** @name PalletStakingActiveEraInfo (560) */
  export interface PalletStakingActiveEraInfo extends Struct {
    readonly index: u32;
    readonly start: Option<u64>;
  }

  /** @name PalletStakingEraRewardPoints (562) */
  export interface PalletStakingEraRewardPoints extends Struct {
    readonly total: u32;
    readonly individual: BTreeMap<AccountId32, u32>;
  }

  /** @name PalletStakingForcing (565) */
  export interface PalletStakingForcing extends Enum {
    readonly isNotForcing: boolean;
    readonly isForceNew: boolean;
    readonly isForceNone: boolean;
    readonly isForceAlways: boolean;
    readonly type: 'NotForcing' | 'ForceNew' | 'ForceNone' | 'ForceAlways';
  }

  /** @name PalletStakingUnappliedSlash (567) */
  export interface PalletStakingUnappliedSlash extends Struct {
    readonly validator: AccountId32;
    readonly own: u128;
    readonly others: Vec<ITuple<[AccountId32, u128]>>;
    readonly reporters: Vec<AccountId32>;
    readonly payout: u128;
  }

  /** @name PalletStakingSlashingSlashingSpans (571) */
  export interface PalletStakingSlashingSlashingSpans extends Struct {
    readonly spanIndex: u32;
    readonly lastStart: u32;
    readonly lastNonzeroSlash: u32;
    readonly prior: Vec<u32>;
  }

  /** @name PalletStakingSlashingSpanRecord (572) */
  export interface PalletStakingSlashingSpanRecord extends Struct {
    readonly slashed: u128;
    readonly paidOut: u128;
  }

  /** @name PalletStakingElectionResult (573) */
  export interface PalletStakingElectionResult extends Struct {
    readonly electedStashes: Vec<AccountId32>;
    readonly exposures: Vec<ITuple<[AccountId32, PalletStakingExposure]>>;
    readonly compute: PalletStakingElectionCompute;
  }

  /** @name PalletStakingElectionStatus (574) */
  export interface PalletStakingElectionStatus extends Enum {
    readonly isClosed: boolean;
    readonly isOpen: boolean;
    readonly asOpen: u32;
    readonly type: 'Closed' | 'Open';
  }

  /** @name PalletStakingPermissionedIdentityPrefs (575) */
  export interface PalletStakingPermissionedIdentityPrefs extends Struct {
    readonly intendedCount: u32;
    readonly runningCount: u32;
  }

  /** @name PalletStakingReleases (576) */
  export interface PalletStakingReleases extends Enum {
    readonly isV100Ancient: boolean;
    readonly isV200: boolean;
    readonly isV300: boolean;
    readonly isV400: boolean;
    readonly isV500: boolean;
    readonly isV600: boolean;
    readonly isV601: boolean;
    readonly isV700: boolean;
    readonly type: 'V100Ancient' | 'V200' | 'V300' | 'V400' | 'V500' | 'V600' | 'V601' | 'V700';
  }

  /** @name PalletStakingError (577) */
  export interface PalletStakingError extends Enum {
    readonly isNotController: boolean;
    readonly isNotStash: boolean;
    readonly isAlreadyBonded: boolean;
    readonly isAlreadyPaired: boolean;
    readonly isEmptyTargets: boolean;
    readonly isInvalidSlashIndex: boolean;
    readonly isInsufficientValue: boolean;
    readonly isNoMoreChunks: boolean;
    readonly isNoUnlockChunk: boolean;
    readonly isFundedTarget: boolean;
    readonly isInvalidEraToReward: boolean;
    readonly isNotSortedAndUnique: boolean;
    readonly isAlreadyClaimed: boolean;
    readonly isOffchainElectionEarlySubmission: boolean;
    readonly isOffchainElectionWeakSubmission: boolean;
    readonly isSnapshotUnavailable: boolean;
    readonly isOffchainElectionBogusWinnerCount: boolean;
    readonly isOffchainElectionBogusWinner: boolean;
    readonly isOffchainElectionBogusCompact: boolean;
    readonly isOffchainElectionBogusNominator: boolean;
    readonly isOffchainElectionBogusNomination: boolean;
    readonly isOffchainElectionSlashedNomination: boolean;
    readonly isOffchainElectionBogusSelfVote: boolean;
    readonly isOffchainElectionBogusEdge: boolean;
    readonly isOffchainElectionBogusScore: boolean;
    readonly isOffchainElectionBogusElectionSize: boolean;
    readonly isCallNotAllowed: boolean;
    readonly isIncorrectSlashingSpans: boolean;
    readonly isAlreadyExists: boolean;
    readonly isNotExists: boolean;
    readonly isNoChange: boolean;
    readonly isInvalidValidatorIdentity: boolean;
    readonly isInvalidValidatorCommission: boolean;
    readonly isStashIdentityDoesNotExist: boolean;
    readonly isStashIdentityNotPermissioned: boolean;
    readonly isStashIdentityNotCDDed: boolean;
    readonly isHitIntendedValidatorCount: boolean;
    readonly isIntendedCountIsExceedingConsensusLimit: boolean;
    readonly isBondTooSmall: boolean;
    readonly isBadState: boolean;
    readonly isTooManyTargets: boolean;
    readonly isBadTarget: boolean;
    readonly type: 'NotController' | 'NotStash' | 'AlreadyBonded' | 'AlreadyPaired' | 'EmptyTargets' | 'InvalidSlashIndex' | 'InsufficientValue' | 'NoMoreChunks' | 'NoUnlockChunk' | 'FundedTarget' | 'InvalidEraToReward' | 'NotSortedAndUnique' | 'AlreadyClaimed' | 'OffchainElectionEarlySubmission' | 'OffchainElectionWeakSubmission' | 'SnapshotUnavailable' | 'OffchainElectionBogusWinnerCount' | 'OffchainElectionBogusWinner' | 'OffchainElectionBogusCompact' | 'OffchainElectionBogusNominator' | 'OffchainElectionBogusNomination' | 'OffchainElectionSlashedNomination' | 'OffchainElectionBogusSelfVote' | 'OffchainElectionBogusEdge' | 'OffchainElectionBogusScore' | 'OffchainElectionBogusElectionSize' | 'CallNotAllowed' | 'IncorrectSlashingSpans' | 'AlreadyExists' | 'NotExists' | 'NoChange' | 'InvalidValidatorIdentity' | 'InvalidValidatorCommission' | 'StashIdentityDoesNotExist' | 'StashIdentityNotPermissioned' | 'StashIdentityNotCDDed' | 'HitIntendedValidatorCount' | 'IntendedCountIsExceedingConsensusLimit' | 'BondTooSmall' | 'BadState' | 'TooManyTargets' | 'BadTarget';
  }

  /** @name SpStakingOffenceOffenceDetails (578) */
  export interface SpStakingOffenceOffenceDetails extends Struct {
    readonly offender: ITuple<[AccountId32, PalletStakingExposure]>;
    readonly reporters: Vec<AccountId32>;
  }

  /** @name SpCoreCryptoKeyTypeId (583) */
  export interface SpCoreCryptoKeyTypeId extends U8aFixed {}

  /** @name PalletSessionError (584) */
  export interface PalletSessionError extends Enum {
    readonly isInvalidProof: boolean;
    readonly isNoAssociatedValidatorId: boolean;
    readonly isDuplicatedKey: boolean;
    readonly isNoKeys: boolean;
    readonly isNoAccount: boolean;
    readonly type: 'InvalidProof' | 'NoAssociatedValidatorId' | 'DuplicatedKey' | 'NoKeys' | 'NoAccount';
  }

  /** @name PalletGrandpaStoredState (585) */
  export interface PalletGrandpaStoredState extends Enum {
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

  /** @name PalletGrandpaStoredPendingChange (586) */
  export interface PalletGrandpaStoredPendingChange extends Struct {
    readonly scheduledAt: u32;
    readonly delay: u32;
    readonly nextAuthorities: Vec<ITuple<[SpFinalityGrandpaAppPublic, u64]>>;
    readonly forced: Option<u32>;
  }

  /** @name PalletGrandpaError (588) */
  export interface PalletGrandpaError extends Enum {
    readonly isPauseFailed: boolean;
    readonly isResumeFailed: boolean;
    readonly isChangePending: boolean;
    readonly isTooSoon: boolean;
    readonly isInvalidKeyOwnershipProof: boolean;
    readonly isInvalidEquivocationProof: boolean;
    readonly isDuplicateOffenceReport: boolean;
    readonly type: 'PauseFailed' | 'ResumeFailed' | 'ChangePending' | 'TooSoon' | 'InvalidKeyOwnershipProof' | 'InvalidEquivocationProof' | 'DuplicateOffenceReport';
  }

  /** @name PalletImOnlineBoundedOpaqueNetworkState (592) */
  export interface PalletImOnlineBoundedOpaqueNetworkState extends Struct {
    readonly peerId: Bytes;
    readonly externalAddresses: Vec<Bytes>;
  }

  /** @name PalletImOnlineError (596) */
  export interface PalletImOnlineError extends Enum {
    readonly isInvalidKey: boolean;
    readonly isDuplicatedHeartbeat: boolean;
    readonly type: 'InvalidKey' | 'DuplicatedHeartbeat';
  }

  /** @name PalletSudoError (598) */
  export interface PalletSudoError extends Enum {
    readonly isRequireSudo: boolean;
    readonly type: 'RequireSudo';
  }

  /** @name PalletAssetTickerRegistration (599) */
  export interface PalletAssetTickerRegistration extends Struct {
    readonly owner: PolymeshPrimitivesIdentityId;
    readonly expiry: Option<u64>;
  }

  /** @name PalletAssetSecurityToken (600) */
  export interface PalletAssetSecurityToken extends Struct {
    readonly totalSupply: u128;
    readonly ownerDid: PolymeshPrimitivesIdentityId;
    readonly divisible: bool;
    readonly assetType: PolymeshPrimitivesAssetAssetType;
  }

  /** @name PalletAssetAssetOwnershipRelation (604) */
  export interface PalletAssetAssetOwnershipRelation extends Enum {
    readonly isNotOwned: boolean;
    readonly isTickerOwned: boolean;
    readonly isAssetOwned: boolean;
    readonly type: 'NotOwned' | 'TickerOwned' | 'AssetOwned';
  }

  /** @name PalletAssetClassicTickerRegistration (606) */
  export interface PalletAssetClassicTickerRegistration extends Struct {
    readonly ethOwner: PolymeshPrimitivesEthereumEthereumAddress;
    readonly isCreated: bool;
  }

  /** @name PalletAssetError (612) */
  export interface PalletAssetError extends Enum {
    readonly isUnauthorized: boolean;
    readonly isAlreadyArchived: boolean;
    readonly isAlreadyUnArchived: boolean;
    readonly isExtensionAlreadyPresent: boolean;
    readonly isAssetAlreadyCreated: boolean;
    readonly isTickerTooLong: boolean;
    readonly isTickerNotAscii: boolean;
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
    readonly isMaximumTMExtensionLimitReached: boolean;
    readonly isIncompatibleExtensionVersion: boolean;
    readonly isInvalidEthereumSignature: boolean;
    readonly isNoSuchClassicTicker: boolean;
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
    readonly type: 'Unauthorized' | 'AlreadyArchived' | 'AlreadyUnArchived' | 'ExtensionAlreadyPresent' | 'AssetAlreadyCreated' | 'TickerTooLong' | 'TickerNotAscii' | 'TickerAlreadyRegistered' | 'TotalSupplyAboveLimit' | 'NoSuchAsset' | 'AlreadyFrozen' | 'NotAnOwner' | 'BalanceOverflow' | 'TotalSupplyOverflow' | 'InvalidGranularity' | 'NotFrozen' | 'InvalidTransfer' | 'InsufficientBalance' | 'AssetAlreadyDivisible' | 'MaximumTMExtensionLimitReached' | 'IncompatibleExtensionVersion' | 'InvalidEthereumSignature' | 'NoSuchClassicTicker' | 'TickerRegistrationExpired' | 'SenderSameAsReceiver' | 'NoSuchDoc' | 'MaxLengthOfAssetNameExceeded' | 'FundingRoundNameMaxLengthExceeded' | 'InvalidAssetIdentifier' | 'InvestorUniquenessClaimNotAllowed' | 'InvalidCustomAssetTypeId' | 'AssetMetadataNameMaxLengthExceeded' | 'AssetMetadataValueMaxLengthExceeded' | 'AssetMetadataTypeDefMaxLengthExceeded' | 'AssetMetadataKeyIsMissing' | 'AssetMetadataValueIsLocked' | 'AssetMetadataLocalKeyAlreadyExists' | 'AssetMetadataGlobalKeyAlreadyExists';
  }

  /** @name PalletCorporateActionsDistributionError (615) */
  export interface PalletCorporateActionsDistributionError extends Enum {
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

  /** @name PalletAssetCheckpointError (622) */
  export interface PalletAssetCheckpointError extends Enum {
    readonly isNoSuchSchedule: boolean;
    readonly isScheduleNotRemovable: boolean;
    readonly isFailedToComputeNextCheckpoint: boolean;
    readonly isScheduleDurationTooShort: boolean;
    readonly isSchedulesTooComplex: boolean;
    readonly type: 'NoSuchSchedule' | 'ScheduleNotRemovable' | 'FailedToComputeNextCheckpoint' | 'ScheduleDurationTooShort' | 'SchedulesTooComplex';
  }

  /** @name PolymeshPrimitivesComplianceManagerAssetCompliance (623) */
  export interface PolymeshPrimitivesComplianceManagerAssetCompliance extends Struct {
    readonly paused: bool;
    readonly requirements: Vec<PolymeshPrimitivesComplianceManagerComplianceRequirement>;
  }

  /** @name PalletComplianceManagerError (625) */
  export interface PalletComplianceManagerError extends Enum {
    readonly isUnauthorized: boolean;
    readonly isDidNotExist: boolean;
    readonly isInvalidComplianceRequirementId: boolean;
    readonly isIncorrectOperationOnTrustedIssuer: boolean;
    readonly isDuplicateComplianceRequirements: boolean;
    readonly isComplianceRequirementTooComplex: boolean;
    readonly type: 'Unauthorized' | 'DidNotExist' | 'InvalidComplianceRequirementId' | 'IncorrectOperationOnTrustedIssuer' | 'DuplicateComplianceRequirements' | 'ComplianceRequirementTooComplex';
  }

  /** @name PalletCorporateActionsError (628) */
  export interface PalletCorporateActionsError extends Enum {
    readonly isAuthNotCAATransfer: boolean;
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
    readonly type: 'AuthNotCAATransfer' | 'DetailsTooLong' | 'DuplicateDidTax' | 'TooManyDidTaxes' | 'TooManyTargetIds' | 'NoSuchCheckpointId' | 'NoSuchCA' | 'NoRecordDate' | 'RecordDateAfterStart' | 'DeclDateAfterRecordDate' | 'DeclDateInFuture' | 'NotTargetedByCA';
  }

  /** @name PalletCorporateActionsBallotError (630) */
  export interface PalletCorporateActionsBallotError extends Enum {
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

  /** @name PalletPermissionsError (631) */
  export interface PalletPermissionsError extends Enum {
    readonly isUnauthorizedCaller: boolean;
    readonly type: 'UnauthorizedCaller';
  }

  /** @name PalletPipsPipsMetadata (632) */
  export interface PalletPipsPipsMetadata extends Struct {
    readonly id: u32;
    readonly url: Option<Bytes>;
    readonly description: Option<Bytes>;
    readonly createdAt: u32;
    readonly transactionVersion: u32;
    readonly expiry: PolymeshCommonUtilitiesMaybeBlock;
  }

  /** @name PalletPipsDepositInfo (634) */
  export interface PalletPipsDepositInfo extends Struct {
    readonly owner: AccountId32;
    readonly amount: u128;
  }

  /** @name PalletPipsPip (635) */
  export interface PalletPipsPip extends Struct {
    readonly id: u32;
    readonly proposal: Call;
    readonly state: PalletPipsProposalState;
    readonly proposer: PalletPipsProposer;
  }

  /** @name PalletPipsVotingResult (636) */
  export interface PalletPipsVotingResult extends Struct {
    readonly ayesCount: u32;
    readonly ayesStake: u128;
    readonly naysCount: u32;
    readonly naysStake: u128;
  }

  /** @name PalletPipsVote (637) */
  export interface PalletPipsVote extends ITuple<[bool, u128]> {}

  /** @name PalletPipsSnapshotMetadata (638) */
  export interface PalletPipsSnapshotMetadata extends Struct {
    readonly createdAt: u32;
    readonly madeBy: AccountId32;
    readonly id: u32;
  }

  /** @name PalletPipsError (640) */
  export interface PalletPipsError extends Enum {
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
    readonly type: 'RescheduleNotByReleaseCoordinator' | 'NotFromCommunity' | 'NotByCommittee' | 'TooManyActivePips' | 'IncorrectDeposit' | 'InsufficientDeposit' | 'NoSuchProposal' | 'NotACommitteeMember' | 'InvalidFutureBlockNumber' | 'NumberOfVotesExceeded' | 'StakeAmountOfVotesExceeded' | 'MissingCurrentIdentity' | 'IncorrectProposalState' | 'CannotSkipPip' | 'SnapshotResultTooLarge' | 'SnapshotIdMismatch' | 'ScheduledProposalDoesntExist' | 'ProposalNotInScheduledState';
  }

  /** @name PalletPortfolioError (646) */
  export interface PalletPortfolioError extends Enum {
    readonly isPortfolioDoesNotExist: boolean;
    readonly isInsufficientPortfolioBalance: boolean;
    readonly isDestinationIsSamePortfolio: boolean;
    readonly isPortfolioNameAlreadyInUse: boolean;
    readonly isSecondaryKeyNotAuthorizedForPortfolio: boolean;
    readonly isUnauthorizedCustodian: boolean;
    readonly isInsufficientTokensLocked: boolean;
    readonly isPortfolioNotEmpty: boolean;
    readonly isDifferentIdentityPortfolios: boolean;
    readonly type: 'PortfolioDoesNotExist' | 'InsufficientPortfolioBalance' | 'DestinationIsSamePortfolio' | 'PortfolioNameAlreadyInUse' | 'SecondaryKeyNotAuthorizedForPortfolio' | 'UnauthorizedCustodian' | 'InsufficientTokensLocked' | 'PortfolioNotEmpty' | 'DifferentIdentityPortfolios';
  }

  /** @name PalletProtocolFeeError (647) */
  export interface PalletProtocolFeeError extends Enum {
    readonly isInsufficientAccountBalance: boolean;
    readonly isUnHandledImbalances: boolean;
    readonly isInsufficientSubsidyBalance: boolean;
    readonly type: 'InsufficientAccountBalance' | 'UnHandledImbalances' | 'InsufficientSubsidyBalance';
  }

  /** @name PalletSchedulerScheduledV3 (650) */
  export interface PalletSchedulerScheduledV3 extends Struct {
    readonly maybeId: Option<Bytes>;
    readonly priority: u8;
    readonly call: FrameSupportScheduleMaybeHashed;
    readonly maybePeriodic: Option<ITuple<[u32, u32]>>;
    readonly origin: PolymeshRuntimeDevelopRuntimeOriginCaller;
  }

  /** @name PolymeshRuntimeDevelopRuntimeOriginCaller (651) */
  export interface PolymeshRuntimeDevelopRuntimeOriginCaller extends Enum {
    readonly isSystem: boolean;
    readonly asSystem: FrameSupportDispatchRawOrigin;
    readonly isVoid: boolean;
    readonly isPolymeshCommittee: boolean;
    readonly asPolymeshCommittee: PalletCommitteeRawOriginInstance1;
    readonly isTechnicalCommittee: boolean;
    readonly asTechnicalCommittee: PalletCommitteeRawOriginInstance3;
    readonly isUpgradeCommittee: boolean;
    readonly asUpgradeCommittee: PalletCommitteeRawOriginInstance4;
    readonly type: 'System' | 'Void' | 'PolymeshCommittee' | 'TechnicalCommittee' | 'UpgradeCommittee';
  }

  /** @name FrameSupportDispatchRawOrigin (652) */
  export interface FrameSupportDispatchRawOrigin extends Enum {
    readonly isRoot: boolean;
    readonly isSigned: boolean;
    readonly asSigned: AccountId32;
    readonly isNone: boolean;
    readonly type: 'Root' | 'Signed' | 'None';
  }

  /** @name PalletCommitteeRawOriginInstance1 (653) */
  export interface PalletCommitteeRawOriginInstance1 extends Enum {
    readonly isEndorsed: boolean;
    readonly type: 'Endorsed';
  }

  /** @name PalletCommitteeRawOriginInstance3 (654) */
  export interface PalletCommitteeRawOriginInstance3 extends Enum {
    readonly isEndorsed: boolean;
    readonly type: 'Endorsed';
  }

  /** @name PalletCommitteeRawOriginInstance4 (655) */
  export interface PalletCommitteeRawOriginInstance4 extends Enum {
    readonly isEndorsed: boolean;
    readonly type: 'Endorsed';
  }

  /** @name SpCoreVoid (656) */
  export type SpCoreVoid = Null;

  /** @name PalletSchedulerError (657) */
  export interface PalletSchedulerError extends Enum {
    readonly isFailedToSchedule: boolean;
    readonly isNotFound: boolean;
    readonly isTargetBlockNumberInPast: boolean;
    readonly isRescheduleNoChange: boolean;
    readonly type: 'FailedToSchedule' | 'NotFound' | 'TargetBlockNumberInPast' | 'RescheduleNoChange';
  }

  /** @name PalletSettlementVenue (658) */
  export interface PalletSettlementVenue extends Struct {
    readonly creator: PolymeshPrimitivesIdentityId;
    readonly venueType: PalletSettlementVenueType;
  }

  /** @name PalletSettlementInstruction (661) */
  export interface PalletSettlementInstruction extends Struct {
    readonly instructionId: u64;
    readonly venueId: u64;
    readonly status: PalletSettlementInstructionStatus;
    readonly settlementType: PalletSettlementSettlementType;
    readonly createdAt: Option<u64>;
    readonly tradeDate: Option<u64>;
    readonly valueDate: Option<u64>;
  }

  /** @name PalletSettlementInstructionStatus (662) */
  export interface PalletSettlementInstructionStatus extends Enum {
    readonly isUnknown: boolean;
    readonly isPending: boolean;
    readonly isFailed: boolean;
    readonly type: 'Unknown' | 'Pending' | 'Failed';
  }

  /** @name PalletSettlementLegStatus (664) */
  export interface PalletSettlementLegStatus extends Enum {
    readonly isPendingTokenLock: boolean;
    readonly isExecutionPending: boolean;
    readonly isExecutionToBeSkipped: boolean;
    readonly asExecutionToBeSkipped: ITuple<[AccountId32, u64]>;
    readonly type: 'PendingTokenLock' | 'ExecutionPending' | 'ExecutionToBeSkipped';
  }

  /** @name PalletSettlementAffirmationStatus (666) */
  export interface PalletSettlementAffirmationStatus extends Enum {
    readonly isUnknown: boolean;
    readonly isPending: boolean;
    readonly isAffirmed: boolean;
    readonly type: 'Unknown' | 'Pending' | 'Affirmed';
  }

  /** @name PalletSettlementError (670) */
  export interface PalletSettlementError extends Enum {
    readonly isInvalidVenue: boolean;
    readonly isUnauthorized: boolean;
    readonly isNoPendingAffirm: boolean;
    readonly isInstructionNotAffirmed: boolean;
    readonly isInstructionNotPending: boolean;
    readonly isInstructionNotFailed: boolean;
    readonly isLegNotPending: boolean;
    readonly isUnauthorizedSigner: boolean;
    readonly isReceiptAlreadyClaimed: boolean;
    readonly isReceiptNotClaimed: boolean;
    readonly isUnauthorizedVenue: boolean;
    readonly isFailedToLockTokens: boolean;
    readonly isInstructionFailed: boolean;
    readonly isInstructionDatesInvalid: boolean;
    readonly isInstructionSettleBlockPassed: boolean;
    readonly isInvalidSignature: boolean;
    readonly isSameSenderReceiver: boolean;
    readonly isPortfolioMismatch: boolean;
    readonly isSettleOnPastBlock: boolean;
    readonly isNoPortfolioProvided: boolean;
    readonly isUnexpectedAffirmationStatus: boolean;
    readonly isFailedToSchedule: boolean;
    readonly isLegCountTooSmall: boolean;
    readonly isUnknownInstruction: boolean;
    readonly isInstructionHasTooManyLegs: boolean;
    readonly type: 'InvalidVenue' | 'Unauthorized' | 'NoPendingAffirm' | 'InstructionNotAffirmed' | 'InstructionNotPending' | 'InstructionNotFailed' | 'LegNotPending' | 'UnauthorizedSigner' | 'ReceiptAlreadyClaimed' | 'ReceiptNotClaimed' | 'UnauthorizedVenue' | 'FailedToLockTokens' | 'InstructionFailed' | 'InstructionDatesInvalid' | 'InstructionSettleBlockPassed' | 'InvalidSignature' | 'SameSenderReceiver' | 'PortfolioMismatch' | 'SettleOnPastBlock' | 'NoPortfolioProvided' | 'UnexpectedAffirmationStatus' | 'FailedToSchedule' | 'LegCountTooSmall' | 'UnknownInstruction' | 'InstructionHasTooManyLegs';
  }

  /** @name PolymeshPrimitivesStatisticsStat1stKey (672) */
  export interface PolymeshPrimitivesStatisticsStat1stKey extends Struct {
    readonly asset: PolymeshPrimitivesStatisticsAssetScope;
    readonly statType: PolymeshPrimitivesStatisticsStatType;
  }

  /** @name PolymeshPrimitivesTransferComplianceAssetTransferCompliance (673) */
  export interface PolymeshPrimitivesTransferComplianceAssetTransferCompliance extends Struct {
    readonly paused: bool;
    readonly requirements: BTreeSetTransferCondition;
  }

  /** @name PalletStatisticsError (676) */
  export interface PalletStatisticsError extends Enum {
    readonly isInvalidTransfer: boolean;
    readonly isStatTypeMissing: boolean;
    readonly isStatTypeNeededByTransferCondition: boolean;
    readonly isCannotRemoveStatTypeInUse: boolean;
    readonly isStatTypeLimitReached: boolean;
    readonly isTransferConditionLimitReached: boolean;
    readonly type: 'InvalidTransfer' | 'StatTypeMissing' | 'StatTypeNeededByTransferCondition' | 'CannotRemoveStatTypeInUse' | 'StatTypeLimitReached' | 'TransferConditionLimitReached';
  }

  /** @name PalletStoError (678) */
  export interface PalletStoError extends Enum {
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
    readonly type: 'Unauthorized' | 'Overflow' | 'InsufficientTokensRemaining' | 'FundraiserNotFound' | 'FundraiserNotLive' | 'FundraiserClosed' | 'FundraiserExpired' | 'InvalidVenue' | 'InvalidPriceTiers' | 'InvalidOfferingWindow' | 'MaxPriceExceeded' | 'InvestmentAmountTooLow';
  }

  /** @name PalletTreasuryError (679) */
  export interface PalletTreasuryError extends Enum {
    readonly isInsufficientBalance: boolean;
    readonly isInvalidIdentity: boolean;
    readonly type: 'InsufficientBalance' | 'InvalidIdentity';
  }

  /** @name PalletUtilityError (680) */
  export interface PalletUtilityError extends Enum {
    readonly isInvalidSignature: boolean;
    readonly isTargetCddMissing: boolean;
    readonly isInvalidNonce: boolean;
    readonly type: 'InvalidSignature' | 'TargetCddMissing' | 'InvalidNonce';
  }

  /** @name PalletBaseError (681) */
  export interface PalletBaseError extends Enum {
    readonly isTooLong: boolean;
    readonly isCounterOverflow: boolean;
    readonly type: 'TooLong' | 'CounterOverflow';
  }

  /** @name PalletExternalAgentsError (683) */
  export interface PalletExternalAgentsError extends Enum {
    readonly isNoSuchAG: boolean;
    readonly isUnauthorizedAgent: boolean;
    readonly isAlreadyAnAgent: boolean;
    readonly isNotAnAgent: boolean;
    readonly isRemovingLastFullAgent: boolean;
    readonly isSecondaryKeyNotAuthorizedForAsset: boolean;
    readonly type: 'NoSuchAG' | 'UnauthorizedAgent' | 'AlreadyAnAgent' | 'NotAnAgent' | 'RemovingLastFullAgent' | 'SecondaryKeyNotAuthorizedForAsset';
  }

  /** @name PalletRelayerSubsidy (684) */
  export interface PalletRelayerSubsidy extends Struct {
    readonly payingKey: AccountId32;
    readonly remaining: u128;
  }

  /** @name PalletRelayerError (685) */
  export interface PalletRelayerError extends Enum {
    readonly isUserKeyCddMissing: boolean;
    readonly isPayingKeyCddMissing: boolean;
    readonly isNoPayingKey: boolean;
    readonly isNotPayingKey: boolean;
    readonly isNotAuthorizedForPayingKey: boolean;
    readonly isNotAuthorizedForUserKey: boolean;
    readonly isOverflow: boolean;
    readonly type: 'UserKeyCddMissing' | 'PayingKeyCddMissing' | 'NoPayingKey' | 'NotPayingKey' | 'NotAuthorizedForPayingKey' | 'NotAuthorizedForUserKey' | 'Overflow';
  }

  /** @name PalletRewardsError (686) */
  export interface PalletRewardsError extends Enum {
    readonly isUnknownItnAddress: boolean;
    readonly isItnRewardAlreadyClaimed: boolean;
    readonly isInvalidSignature: boolean;
    readonly isUnableToCovertBalance: boolean;
    readonly type: 'UnknownItnAddress' | 'ItnRewardAlreadyClaimed' | 'InvalidSignature' | 'UnableToCovertBalance';
  }

  /** @name PalletContractsWasmPrefabWasmModule (687) */
  export interface PalletContractsWasmPrefabWasmModule extends Struct {
    readonly instructionWeightsVersion: Compact<u32>;
    readonly initial: Compact<u32>;
    readonly maximum: Compact<u32>;
    readonly code: Bytes;
  }

  /** @name PalletContractsWasmOwnerInfo (688) */
  export interface PalletContractsWasmOwnerInfo extends Struct {
    readonly owner: AccountId32;
    readonly deposit: Compact<u128>;
    readonly refcount: Compact<u64>;
  }

  /** @name PalletContractsStorageRawContractInfo (689) */
  export interface PalletContractsStorageRawContractInfo extends Struct {
    readonly trieId: Bytes;
    readonly codeHash: H256;
    readonly storageDeposit: u128;
  }

  /** @name PalletContractsStorageDeletedContract (691) */
  export interface PalletContractsStorageDeletedContract extends Struct {
    readonly trieId: Bytes;
  }

  /** @name PalletContractsSchedule (692) */
  export interface PalletContractsSchedule extends Struct {
    readonly limits: PalletContractsScheduleLimits;
    readonly instructionWeights: PalletContractsScheduleInstructionWeights;
    readonly hostFnWeights: PalletContractsScheduleHostFnWeights;
  }

  /** @name PalletContractsScheduleLimits (693) */
  export interface PalletContractsScheduleLimits extends Struct {
    readonly eventTopics: u32;
    readonly stackHeight: Option<u32>;
    readonly globals: u32;
    readonly parameters: u32;
    readonly memoryPages: u32;
    readonly tableSize: u32;
    readonly brTableSize: u32;
    readonly subjectLen: u32;
    readonly callDepth: u32;
    readonly payloadLen: u32;
    readonly codeLen: u32;
  }

  /** @name PalletContractsScheduleInstructionWeights (694) */
  export interface PalletContractsScheduleInstructionWeights extends Struct {
    readonly version: u32;
    readonly i64const: u32;
    readonly i64load: u32;
    readonly i64store: u32;
    readonly select: u32;
    readonly r_if: u32;
    readonly br: u32;
    readonly brIf: u32;
    readonly brTable: u32;
    readonly brTablePerEntry: u32;
    readonly call: u32;
    readonly callIndirect: u32;
    readonly callIndirectPerParam: u32;
    readonly localGet: u32;
    readonly localSet: u32;
    readonly localTee: u32;
    readonly globalGet: u32;
    readonly globalSet: u32;
    readonly memoryCurrent: u32;
    readonly memoryGrow: u32;
    readonly i64clz: u32;
    readonly i64ctz: u32;
    readonly i64popcnt: u32;
    readonly i64eqz: u32;
    readonly i64extendsi32: u32;
    readonly i64extendui32: u32;
    readonly i32wrapi64: u32;
    readonly i64eq: u32;
    readonly i64ne: u32;
    readonly i64lts: u32;
    readonly i64ltu: u32;
    readonly i64gts: u32;
    readonly i64gtu: u32;
    readonly i64les: u32;
    readonly i64leu: u32;
    readonly i64ges: u32;
    readonly i64geu: u32;
    readonly i64add: u32;
    readonly i64sub: u32;
    readonly i64mul: u32;
    readonly i64divs: u32;
    readonly i64divu: u32;
    readonly i64rems: u32;
    readonly i64remu: u32;
    readonly i64and: u32;
    readonly i64or: u32;
    readonly i64xor: u32;
    readonly i64shl: u32;
    readonly i64shrs: u32;
    readonly i64shru: u32;
    readonly i64rotl: u32;
    readonly i64rotr: u32;
  }

  /** @name PalletContractsScheduleHostFnWeights (695) */
  export interface PalletContractsScheduleHostFnWeights extends Struct {
    readonly caller: u64;
    readonly isContract: u64;
    readonly codeHash: u64;
    readonly ownCodeHash: u64;
    readonly callerIsOrigin: u64;
    readonly address: u64;
    readonly gasLeft: u64;
    readonly balance: u64;
    readonly valueTransferred: u64;
    readonly minimumBalance: u64;
    readonly blockNumber: u64;
    readonly now: u64;
    readonly weightToFee: u64;
    readonly gas: u64;
    readonly input: u64;
    readonly inputPerByte: u64;
    readonly r_return: u64;
    readonly returnPerByte: u64;
    readonly terminate: u64;
    readonly random: u64;
    readonly depositEvent: u64;
    readonly depositEventPerTopic: u64;
    readonly depositEventPerByte: u64;
    readonly debugMessage: u64;
    readonly setStorage: u64;
    readonly setStoragePerNewByte: u64;
    readonly setStoragePerOldByte: u64;
    readonly setCodeHash: u64;
    readonly clearStorage: u64;
    readonly clearStoragePerByte: u64;
    readonly containsStorage: u64;
    readonly containsStoragePerByte: u64;
    readonly getStorage: u64;
    readonly getStoragePerByte: u64;
    readonly takeStorage: u64;
    readonly takeStoragePerByte: u64;
    readonly transfer: u64;
    readonly call: u64;
    readonly delegateCall: u64;
    readonly callTransferSurcharge: u64;
    readonly callPerClonedByte: u64;
    readonly instantiate: u64;
    readonly instantiateTransferSurcharge: u64;
    readonly instantiatePerSaltByte: u64;
    readonly hashSha2256: u64;
    readonly hashSha2256PerByte: u64;
    readonly hashKeccak256: u64;
    readonly hashKeccak256PerByte: u64;
    readonly hashBlake2256: u64;
    readonly hashBlake2256PerByte: u64;
    readonly hashBlake2128: u64;
    readonly hashBlake2128PerByte: u64;
    readonly ecdsaRecover: u64;
    readonly ecdsaToEthAddress: u64;
  }

  /** @name PalletContractsError (696) */
  export interface PalletContractsError extends Enum {
    readonly isInvalidScheduleVersion: boolean;
    readonly isInvalidCallFlags: boolean;
    readonly isOutOfGas: boolean;
    readonly isOutputBufferTooSmall: boolean;
    readonly isTransferFailed: boolean;
    readonly isMaxCallDepthReached: boolean;
    readonly isContractNotFound: boolean;
    readonly isCodeTooLarge: boolean;
    readonly isCodeNotFound: boolean;
    readonly isOutOfBounds: boolean;
    readonly isDecodingFailed: boolean;
    readonly isContractTrapped: boolean;
    readonly isValueTooLarge: boolean;
    readonly isTerminatedWhileReentrant: boolean;
    readonly isInputForwarded: boolean;
    readonly isRandomSubjectTooLong: boolean;
    readonly isTooManyTopics: boolean;
    readonly isDuplicateTopics: boolean;
    readonly isNoChainExtension: boolean;
    readonly isDeletionQueueFull: boolean;
    readonly isDuplicateContract: boolean;
    readonly isTerminatedInConstructor: boolean;
    readonly isDebugMessageInvalidUTF8: boolean;
    readonly isReentranceDenied: boolean;
    readonly isStorageDepositNotEnoughFunds: boolean;
    readonly isStorageDepositLimitExhausted: boolean;
    readonly isCodeInUse: boolean;
    readonly isContractReverted: boolean;
    readonly isCodeRejected: boolean;
    readonly type: 'InvalidScheduleVersion' | 'InvalidCallFlags' | 'OutOfGas' | 'OutputBufferTooSmall' | 'TransferFailed' | 'MaxCallDepthReached' | 'ContractNotFound' | 'CodeTooLarge' | 'CodeNotFound' | 'OutOfBounds' | 'DecodingFailed' | 'ContractTrapped' | 'ValueTooLarge' | 'TerminatedWhileReentrant' | 'InputForwarded' | 'RandomSubjectTooLong' | 'TooManyTopics' | 'DuplicateTopics' | 'NoChainExtension' | 'DeletionQueueFull' | 'DuplicateContract' | 'TerminatedInConstructor' | 'DebugMessageInvalidUTF8' | 'ReentranceDenied' | 'StorageDepositNotEnoughFunds' | 'StorageDepositLimitExhausted' | 'CodeInUse' | 'ContractReverted' | 'CodeRejected';
  }

  /** @name PolymeshContractsError (697) */
  export interface PolymeshContractsError extends Enum {
    readonly isRuntimeCallNotFound: boolean;
    readonly isDataLeftAfterDecoding: boolean;
    readonly isInLenTooLarge: boolean;
    readonly isInstantiatorWithNoIdentity: boolean;
    readonly type: 'RuntimeCallNotFound' | 'DataLeftAfterDecoding' | 'InLenTooLarge' | 'InstantiatorWithNoIdentity';
  }

  /** @name PalletPreimageRequestStatus (698) */
  export interface PalletPreimageRequestStatus extends Enum {
    readonly isUnrequested: boolean;
    readonly asUnrequested: Option<ITuple<[AccountId32, u128]>>;
    readonly isRequested: boolean;
    readonly asRequested: u32;
    readonly type: 'Unrequested' | 'Requested';
  }

  /** @name PalletPreimageError (701) */
  export interface PalletPreimageError extends Enum {
    readonly isTooLarge: boolean;
    readonly isAlreadyNoted: boolean;
    readonly isNotAuthorized: boolean;
    readonly isNotNoted: boolean;
    readonly isRequested: boolean;
    readonly isNotRequested: boolean;
    readonly type: 'TooLarge' | 'AlreadyNoted' | 'NotAuthorized' | 'NotNoted' | 'Requested' | 'NotRequested';
  }

  /** @name PalletTestUtilsError (702) */
  export type PalletTestUtilsError = Null;

  /** @name FrameSystemExtensionsCheckSpecVersion (705) */
  export type FrameSystemExtensionsCheckSpecVersion = Null;

  /** @name FrameSystemExtensionsCheckTxVersion (706) */
  export type FrameSystemExtensionsCheckTxVersion = Null;

  /** @name FrameSystemExtensionsCheckGenesis (707) */
  export type FrameSystemExtensionsCheckGenesis = Null;

  /** @name FrameSystemExtensionsCheckNonce (710) */
  export interface FrameSystemExtensionsCheckNonce extends Compact<u32> {}

  /** @name PolymeshExtensionsCheckWeight (711) */
  export interface PolymeshExtensionsCheckWeight extends FrameSystemExtensionsCheckWeight {}

  /** @name FrameSystemExtensionsCheckWeight (712) */
  export type FrameSystemExtensionsCheckWeight = Null;

  /** @name PalletTransactionPaymentChargeTransactionPayment (713) */
  export interface PalletTransactionPaymentChargeTransactionPayment extends Compact<u128> {}

  /** @name PalletPermissionsStoreCallMetadata (714) */
  export type PalletPermissionsStoreCallMetadata = Null;

  /** @name PolymeshRuntimeDevelopRuntime (715) */
  export type PolymeshRuntimeDevelopRuntime = Null;

} // declare module
