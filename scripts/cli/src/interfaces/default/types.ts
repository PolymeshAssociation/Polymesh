// Auto-generated via `yarn polkadot-types-from-defs`, do not edit
/* eslint-disable */

import type { Bytes, Compact, Enum, Option, Struct, Text, U8aFixed, Vec, bool, u16, u32, u64, u8 } from '@polkadot/types';
import type { ITuple } from '@polkadot/types/types';
import type { Signature } from '@polkadot/types/interfaces/extrinsics';
import type { AccountId, Balance, BlockNumber, Call, H256, H512, Hash, MultiAddress, Perbill, Permill } from '@polkadot/types/interfaces/runtime';
import type { AccountInfoWithDualRefCount, DispatchError } from '@polkadot/types/interfaces/system';

/** @name AccountInfo */
export interface AccountInfo extends AccountInfoWithDualRefCount {}

/** @name Address */
export interface Address extends MultiAddress {}

/** @name AffirmationStatus */
export interface AffirmationStatus extends Enum {
  readonly isUnknown: boolean;
  readonly isPending: boolean;
  readonly isAffirmed: boolean;
  readonly isRejected: boolean;
}

/** @name AgentGroup */
export interface AgentGroup extends Enum {
  readonly isFull: boolean;
  readonly isCustom: boolean;
  readonly asCustom: AGId;
  readonly isExceptMeta: boolean;
  readonly isPolymeshV1Caa: boolean;
  readonly isPolymeshV1Pia: boolean;
}

/** @name AGId */
export interface AGId extends u32 {}

/** @name AssetCompliance */
export interface AssetCompliance extends Struct {
  readonly is_paused: bool;
  readonly requirements: Vec<ComplianceRequirement>;
}

/** @name AssetComplianceResult */
export interface AssetComplianceResult extends Struct {
  readonly paused: bool;
  readonly requirements: Vec<ComplianceRequirementResult>;
  readonly result: bool;
}

/** @name AssetDidResult */
export interface AssetDidResult extends Enum {
  readonly isOk: boolean;
  readonly asOk: IdentityId;
  readonly isErr: boolean;
  readonly asErr: Bytes;
}

/** @name AssetIdentifier */
export interface AssetIdentifier extends Enum {
  readonly isCusip: boolean;
  readonly asCusip: U8aFixed;
  readonly isCins: boolean;
  readonly asCins: U8aFixed;
  readonly isIsin: boolean;
  readonly asIsin: U8aFixed;
  readonly isLei: boolean;
  readonly asLei: U8aFixed;
}

/** @name AssetMigrationError */
export interface AssetMigrationError extends Enum {
  readonly isAssetDocumentFail: boolean;
  readonly asAssetDocumentFail: ITuple<[Ticker, DocumentId]>;
}

/** @name AssetName */
export interface AssetName extends Text {}

/** @name AssetOwnershipRelation */
export interface AssetOwnershipRelation extends Enum {
  readonly isNotOwned: boolean;
  readonly isTickerOwned: boolean;
  readonly isAssetOwned: boolean;
}

/** @name AssetPermissions */
export interface AssetPermissions extends Enum {
  readonly isWhole: boolean;
  readonly isThese: boolean;
  readonly asThese: Vec<Ticker>;
  readonly isExcept: boolean;
  readonly asExcept: Vec<Ticker>;
}

/** @name AssetType */
export interface AssetType extends Enum {
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
  readonly asCustom: Bytes;
  readonly isStableCoin: boolean;
}

/** @name Authorization */
export interface Authorization extends Struct {
  readonly authorization_data: AuthorizationData;
  readonly authorized_by: IdentityId;
  readonly expiry: Option<Moment>;
  readonly auth_id: u64;
}

/** @name AuthorizationData */
export interface AuthorizationData extends Enum {
  readonly isAttestPrimaryKeyRotation: boolean;
  readonly asAttestPrimaryKeyRotation: IdentityId;
  readonly isRotatePrimaryKey: boolean;
  readonly asRotatePrimaryKey: IdentityId;
  readonly isTransferTicker: boolean;
  readonly asTransferTicker: Ticker;
  readonly isTransferPrimaryIssuanceAgent: boolean;
  readonly asTransferPrimaryIssuanceAgent: Ticker;
  readonly isAddMultiSigSigner: boolean;
  readonly asAddMultiSigSigner: AccountId;
  readonly isTransferAssetOwnership: boolean;
  readonly asTransferAssetOwnership: Ticker;
  readonly isJoinIdentity: boolean;
  readonly asJoinIdentity: Permissions;
  readonly isPortfolioCustody: boolean;
  readonly asPortfolioCustody: PortfolioId;
  readonly isCustom: boolean;
  readonly asCustom: Ticker;
  readonly isNoData: boolean;
  readonly isTransferCorporateActionAgent: boolean;
  readonly asTransferCorporateActionAgent: Ticker;
  readonly isBecomeAgent: boolean;
  readonly asBecomeAgent: ITuple<[Ticker, AgentGroup]>;
  readonly isAddRelayerPayingKey: boolean;
  readonly asAddRelayerPayingKey: ITuple<[AccountId, AccountId, Balance]>;
}

/** @name AuthorizationNonce */
export interface AuthorizationNonce extends u64 {}

/** @name AuthorizationType */
export interface AuthorizationType extends Enum {
  readonly isAttestPrimaryKeyRotation: boolean;
  readonly isRotatePrimaryKey: boolean;
  readonly isTransferTicker: boolean;
  readonly isAddMultiSigSigner: boolean;
  readonly isTransferAssetOwnership: boolean;
  readonly isJoinIdentity: boolean;
  readonly isPortfolioCustody: boolean;
  readonly isCustom: boolean;
  readonly isNoData: boolean;
}

/** @name BallotMeta */
export interface BallotMeta extends Struct {
  readonly title: BallotTitle;
  readonly motions: Vec<Motion>;
}

/** @name BallotTimeRange */
export interface BallotTimeRange extends Struct {
  readonly start: Moment;
  readonly end: Moment;
}

/** @name BallotTitle */
export interface BallotTitle extends Text {}

/** @name BallotVote */
export interface BallotVote extends Struct {
  readonly power: Balance;
  readonly fallback: Option<u16>;
}

/** @name BatchAddClaimItem */
export interface BatchAddClaimItem extends Struct {
  readonly target: IdentityId;
  readonly claim: Claim;
  readonly expiry: Option<Moment>;
}

/** @name BatchRevokeClaimItem */
export interface BatchRevokeClaimItem extends Struct {
  readonly target: IdentityId;
  readonly claim: Claim;
}

/** @name Beneficiary */
export interface Beneficiary extends Struct {
  readonly id: IdentityId;
  readonly amount: Balance;
}

/** @name BridgeTx */
export interface BridgeTx extends Struct {
  readonly nonce: u32;
  readonly recipient: AccountId;
  readonly value: Balance;
  readonly tx_hash: H256;
}

/** @name BridgeTxDetail */
export interface BridgeTxDetail extends Struct {
  readonly amount: Balance;
  readonly status: BridgeTxStatus;
  readonly execution_block: BlockNumber;
  readonly tx_hash: H256;
}

/** @name BridgeTxStatus */
export interface BridgeTxStatus extends Enum {
  readonly isAbsent: boolean;
  readonly isPending: boolean;
  readonly asPending: u8;
  readonly isFrozen: boolean;
  readonly isTimelocked: boolean;
  readonly isHandled: boolean;
}

/** @name CACheckpoint */
export interface CACheckpoint extends Enum {
  readonly isScheduled: boolean;
  readonly asScheduled: ITuple<[ScheduleId, u64]>;
  readonly isExisting: boolean;
  readonly asExisting: CheckpointId;
}

/** @name CADetails */
export interface CADetails extends Text {}

/** @name CAId */
export interface CAId extends Struct {
  readonly ticker: Ticker;
  readonly local_id: LocalCAId;
}

/** @name CAKind */
export interface CAKind extends Enum {
  readonly isPredictableBenefit: boolean;
  readonly isUnpredictableBenefit: boolean;
  readonly isIssuerNotice: boolean;
  readonly isReorganization: boolean;
  readonly isOther: boolean;
}

/** @name CalendarPeriod */
export interface CalendarPeriod extends Struct {
  readonly unit: CalendarUnit;
  readonly amount: u64;
}

/** @name CalendarUnit */
export interface CalendarUnit extends Enum {
  readonly isSecond: boolean;
  readonly isMinute: boolean;
  readonly isHour: boolean;
  readonly isDay: boolean;
  readonly isWeek: boolean;
  readonly isMonth: boolean;
  readonly isYear: boolean;
}

/** @name CanTransferResult */
export interface CanTransferResult extends Enum {
  readonly isOk: boolean;
  readonly asOk: u8;
  readonly isErr: boolean;
  readonly asErr: Bytes;
}

/** @name CappedFee */
export interface CappedFee extends u64 {}

/** @name CddId */
export interface CddId extends U8aFixed {}

/** @name CddStatus */
export interface CddStatus extends Enum {
  readonly isOk: boolean;
  readonly asOk: IdentityId;
  readonly isErr: boolean;
  readonly asErr: Bytes;
}

/** @name CheckpointId */
export interface CheckpointId extends u64 {}

/** @name CheckpointSchedule */
export interface CheckpointSchedule extends Struct {
  readonly start: Moment;
  readonly period: CalendarPeriod;
}

/** @name ChoiceTitle */
export interface ChoiceTitle extends Text {}

/** @name Claim */
export interface Claim extends Enum {
  readonly isAccredited: boolean;
  readonly asAccredited: Scope;
  readonly isAffiliate: boolean;
  readonly asAffiliate: Scope;
  readonly isBuyLockup: boolean;
  readonly asBuyLockup: Scope;
  readonly isSellLockup: boolean;
  readonly asSellLockup: Scope;
  readonly isCustomerDueDiligence: boolean;
  readonly asCustomerDueDiligence: CddId;
  readonly isKnowYourCustomer: boolean;
  readonly asKnowYourCustomer: Scope;
  readonly isJurisdiction: boolean;
  readonly asJurisdiction: ITuple<[CountryCode, Scope]>;
  readonly isExempted: boolean;
  readonly asExempted: Scope;
  readonly isBlocked: boolean;
  readonly asBlocked: Scope;
  readonly isInvestorUniqueness: boolean;
  readonly asInvestorUniqueness: ITuple<[Scope, ScopeId, CddId]>;
  readonly isNoData: boolean;
  readonly isInvestorUniquenessV2: boolean;
  readonly asInvestorUniquenessV2: CddId;
}

/** @name Claim1stKey */
export interface Claim1stKey extends Struct {
  readonly target: IdentityId;
  readonly claim_type: ClaimType;
}

/** @name Claim2ndKey */
export interface Claim2ndKey extends Struct {
  readonly issuer: IdentityId;
  readonly scope: Option<Scope>;
}

/** @name ClaimType */
export interface ClaimType extends Enum {
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
  readonly isNoData: boolean;
  readonly isInvestorUniquenessV2: boolean;
}

/** @name ClassicTickerImport */
export interface ClassicTickerImport extends Struct {
  readonly eth_owner: EthereumAddress;
  readonly ticker: Ticker;
  readonly is_contract: bool;
  readonly is_created: bool;
}

/** @name ClassicTickerRegistration */
export interface ClassicTickerRegistration extends Struct {
  readonly eth_owner: EthereumAddress;
  readonly is_created: bool;
}

/** @name Committee */
export interface Committee extends Enum {
  readonly isTechnical: boolean;
  readonly isUpgrade: boolean;
}

/** @name ComplianceRequirement */
export interface ComplianceRequirement extends Struct {
  readonly sender_conditions: Vec<Condition>;
  readonly receiver_conditions: Vec<Condition>;
  readonly id: u32;
}

/** @name ComplianceRequirementResult */
export interface ComplianceRequirementResult extends Struct {
  readonly sender_conditions: Vec<ConditionResult>;
  readonly receiver_conditions: Vec<ConditionResult>;
  readonly id: u32;
  readonly result: bool;
}

/** @name Condition */
export interface Condition extends Struct {
  readonly condition_type: ConditionType;
  readonly issuers: Vec<TrustedIssuer>;
}

/** @name ConditionResult */
export interface ConditionResult extends Struct {
  readonly condition: Condition;
  readonly result: bool;
}

/** @name ConditionType */
export interface ConditionType extends Enum {
  readonly isIsPresent: boolean;
  readonly asIsPresent: Claim;
  readonly isIsAbsent: boolean;
  readonly asIsAbsent: Claim;
  readonly isIsAnyOf: boolean;
  readonly asIsAnyOf: Vec<Claim>;
  readonly isIsNoneOf: boolean;
  readonly asIsNoneOf: Vec<Claim>;
  readonly isIsIdentity: boolean;
  readonly asIsIdentity: TargetIdentity;
}

/** @name CorporateAction */
export interface CorporateAction extends Struct {
  readonly kind: CAKind;
  readonly decl_date: Moment;
  readonly record_date: Option<RecordDate>;
  readonly details: Text;
  readonly targets: TargetIdentities;
  readonly default_withholding_tax: Tax;
  readonly withholding_tax: Vec<ITuple<[IdentityId, Tax]>>;
}

/** @name Counter */
export interface Counter extends u64 {}

/** @name CountryCode */
export interface CountryCode extends Enum {
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
}

/** @name DepositInfo */
export interface DepositInfo extends Struct {
  readonly owner: AccountId;
  readonly amount: Balance;
}

/** @name DidRecord */
export interface DidRecord extends Struct {
  readonly primary_key: AccountId;
  readonly secondary_keys: Vec<SecondaryKey>;
}

/** @name DidRecords */
export interface DidRecords extends Enum {
  readonly isSuccess: boolean;
  readonly asSuccess: DidRecordsSuccess;
  readonly isIdNotFound: boolean;
  readonly asIdNotFound: Bytes;
}

/** @name DidRecordsSuccess */
export interface DidRecordsSuccess extends Struct {
  readonly primary_key: AccountId;
  readonly secondary_key: Vec<SecondaryKey>;
}

/** @name DidStatus */
export interface DidStatus extends Enum {
  readonly isUnknown: boolean;
  readonly isExists: boolean;
  readonly isCddVerified: boolean;
}

/** @name DispatchableName */
export interface DispatchableName extends Text {}

/** @name DispatchableNames */
export interface DispatchableNames extends Enum {
  readonly isWhole: boolean;
  readonly isThese: boolean;
  readonly asThese: Vec<DispatchableName>;
  readonly isExcept: boolean;
  readonly asExcept: Vec<DispatchableName>;
}

/** @name Distribution */
export interface Distribution extends Struct {
  readonly from: PortfolioId;
  readonly currency: Ticker;
  readonly per_share: Balance;
  readonly amount: Balance;
  readonly remaining: Balance;
  readonly reclaimed: bool;
  readonly payment_at: Moment;
  readonly expires_at: Option<Moment>;
}

/** @name Document */
export interface Document extends Struct {
  readonly uri: DocumentUri;
  readonly content_hash: DocumentHash;
  readonly name: DocumentName;
  readonly doc_type: Option<DocumentType>;
  readonly filing_date: Option<Moment>;
}

/** @name DocumentHash */
export interface DocumentHash extends Enum {
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
}

/** @name DocumentId */
export interface DocumentId extends u32 {}

/** @name DocumentName */
export interface DocumentName extends Text {}

/** @name DocumentType */
export interface DocumentType extends Text {}

/** @name DocumentUri */
export interface DocumentUri extends Text {}

/** @name EcdsaSignature */
export interface EcdsaSignature extends U8aFixed {}

/** @name ErrorAt */
export interface ErrorAt extends ITuple<[u32, DispatchError]> {}

/** @name EthereumAddress */
export interface EthereumAddress extends U8aFixed {}

/** @name EventCounts */
export interface EventCounts extends Vec<u32> {}

/** @name EventDid */
export interface EventDid extends IdentityId {}

/** @name ExtensionAttributes */
export interface ExtensionAttributes extends Struct {
  readonly usage_fee: Balance;
  readonly version: MetaVersion;
}

/** @name ExtrinsicPermissions */
export interface ExtrinsicPermissions extends Enum {
  readonly isWhole: boolean;
  readonly isThese: boolean;
  readonly asThese: Vec<PalletPermissions>;
  readonly isExcept: boolean;
  readonly asExcept: Vec<PalletPermissions>;
}

/** @name ExtVersion */
export interface ExtVersion extends u32 {}

/** @name FeeOf */
export interface FeeOf extends Balance {}

/** @name FundingRoundName */
export interface FundingRoundName extends Text {}

/** @name Fundraiser */
export interface Fundraiser extends Struct {
  readonly creator: IdentityId;
  readonly offering_portfolio: PortfolioId;
  readonly offering_asset: Ticker;
  readonly raising_portfolio: PortfolioId;
  readonly raising_asset: Ticker;
  readonly tiers: Vec<FundraiserTier>;
  readonly venue_id: u64;
  readonly start: Moment;
  readonly end: Option<Moment>;
  readonly status: FundraiserStatus;
  readonly minimum_investment: Balance;
}

/** @name FundraiserName */
export interface FundraiserName extends Text {}

/** @name FundraiserStatus */
export interface FundraiserStatus extends Enum {
  readonly isLive: boolean;
  readonly isFrozen: boolean;
  readonly isClosed: boolean;
  readonly isClosedEarly: boolean;
}

/** @name FundraiserTier */
export interface FundraiserTier extends Struct {
  readonly total: Balance;
  readonly price: Balance;
  readonly remaining: Balance;
}

/** @name GranularCanTransferResult */
export interface GranularCanTransferResult extends Struct {
  readonly invalid_granularity: bool;
  readonly self_transfer: bool;
  readonly invalid_receiver_cdd: bool;
  readonly invalid_sender_cdd: bool;
  readonly missing_scope_claim: bool;
  readonly receiver_custodian_error: bool;
  readonly sender_custodian_error: bool;
  readonly sender_insufficient_balance: bool;
  readonly portfolio_validity_result: PortfolioValidityResult;
  readonly asset_frozen: bool;
  readonly statistics_result: Vec<TransferManagerResult>;
  readonly compliance_result: AssetComplianceResult;
  readonly result: bool;
}

/** @name HandledTxStatus */
export interface HandledTxStatus extends Enum {
  readonly isSuccess: boolean;
  readonly isError: boolean;
  readonly asError: Text;
}

/** @name IdentityClaim */
export interface IdentityClaim extends Struct {
  readonly claim_issuer: IdentityId;
  readonly issuance_date: Moment;
  readonly last_update_date: Moment;
  readonly expiry: Option<Moment>;
  readonly claim: Claim;
}

/** @name IdentityClaimKey */
export interface IdentityClaimKey extends Struct {
  readonly id: IdentityId;
  readonly claim_type: ClaimType;
}

/** @name IdentityId */
export interface IdentityId extends U8aFixed {}

/** @name IdentityRole */
export interface IdentityRole extends Enum {
  readonly isIssuer: boolean;
  readonly isSimpleTokenIssuer: boolean;
  readonly isValidator: boolean;
  readonly isClaimIssuer: boolean;
  readonly isInvestor: boolean;
  readonly isNodeRunner: boolean;
  readonly isPm: boolean;
  readonly isCddamlClaimIssuer: boolean;
  readonly isAccreditedInvestorClaimIssuer: boolean;
  readonly isVerifiedIdentityClaimIssuer: boolean;
}

/** @name InactiveMember */
export interface InactiveMember extends Struct {
  readonly id: IdentityId;
  readonly deactivated_at: Moment;
  readonly expiry: Option<Moment>;
}

/** @name Instruction */
export interface Instruction extends Struct {
  readonly instruction_id: u64;
  readonly venue_id: u64;
  readonly status: InstructionStatus;
  readonly settlement_type: SettlementType;
  readonly created_at: Option<Moment>;
  readonly trade_date: Option<Moment>;
  readonly value_date: Option<Moment>;
}

/** @name InstructionStatus */
export interface InstructionStatus extends Enum {
  readonly isUnknown: boolean;
  readonly isPending: boolean;
  readonly isFailed: boolean;
}

/** @name InvestorUid */
export interface InvestorUid extends U8aFixed {}

/** @name InvestorZKProofData */
export interface InvestorZKProofData extends Signature {}

/** @name IssueRecipient */
export interface IssueRecipient extends Enum {
  readonly isAccount: boolean;
  readonly asAccount: AccountId;
  readonly isIdentity: boolean;
  readonly asIdentity: IdentityId;
}

/** @name KeyIdentityData */
export interface KeyIdentityData extends Struct {
  readonly identity: IdentityId;
  readonly permissions: Option<Permissions>;
}

/** @name Leg */
export interface Leg extends Struct {
  readonly from: PortfolioId;
  readonly to: PortfolioId;
  readonly asset: Ticker;
  readonly amount: Balance;
}

/** @name LegacyPalletPermissions */
export interface LegacyPalletPermissions extends Struct {
  readonly pallet_name: PalletName;
  readonly total: bool;
  readonly dispatchable_names: Vec<DispatchableName>;
}

/** @name LegacyPermissions */
export interface LegacyPermissions extends Struct {
  readonly asset: Option<Vec<Ticker>>;
  readonly extrinsic: Option<Vec<LegacyPalletPermissions>>;
  readonly portfolio: Option<Vec<PortfolioId>>;
}

/** @name LegStatus */
export interface LegStatus extends Enum {
  readonly isPendingTokenLock: boolean;
  readonly isExecutionPending: boolean;
  readonly isExecutionToBeSkipped: boolean;
  readonly asExecutionToBeSkipped: ITuple<[AccountId, u64]>;
}

/** @name LocalCAId */
export interface LocalCAId extends u32 {}

/** @name LookupSource */
export interface LookupSource extends MultiAddress {}

/** @name MaybeBlock */
export interface MaybeBlock extends Enum {
  readonly isSome: boolean;
  readonly asSome: BlockNumber;
  readonly isNone: boolean;
}

/** @name Memo */
export interface Memo extends U8aFixed {}

/** @name MetaDescription */
export interface MetaDescription extends Text {}

/** @name MetaUrl */
export interface MetaUrl extends Text {}

/** @name MetaVersion */
export interface MetaVersion extends u32 {}

/** @name MigrationError */
export interface MigrationError extends Enum {
  readonly isDecodeKey: boolean;
  readonly asDecodeKey: Bytes;
  readonly isMap: boolean;
  readonly asMap: AssetMigrationError;
}

/** @name Moment */
export interface Moment extends u64 {}

/** @name Motion */
export interface Motion extends Struct {
  readonly title: MotionTitle;
  readonly info_link: MotionInfoLink;
  readonly choices: Vec<ChoiceTitle>;
}

/** @name MotionInfoLink */
export interface MotionInfoLink extends Text {}

/** @name MotionTitle */
export interface MotionTitle extends Text {}

/** @name MovePortfolioItem */
export interface MovePortfolioItem extends Struct {
  readonly ticker: Ticker;
  readonly amount: Balance;
  readonly memo: Option<Memo>;
}

/** @name OffChainSignature */
export interface OffChainSignature extends Enum {
  readonly isEd25519: boolean;
  readonly asEd25519: H512;
  readonly isSr25519: boolean;
  readonly asSr25519: H512;
  readonly isEcdsa: boolean;
  readonly asEcdsa: H512;
}

/** @name PalletName */
export interface PalletName extends Text {}

/** @name PalletPermissions */
export interface PalletPermissions extends Struct {
  readonly pallet_name: PalletName;
  readonly dispatchable_names: DispatchableNames;
}

/** @name Payload */
export interface Payload extends Struct {
  readonly block_number: BlockNumber;
  readonly nominators: Vec<AccountId>;
  readonly public: H256;
}

/** @name PendingTx */
export interface PendingTx extends Struct {
  readonly did: IdentityId;
  readonly bridge_tx: BridgeTx;
}

/** @name Percentage */
export interface Percentage extends Permill {}

/** @name PermissionedIdentityPrefs */
export interface PermissionedIdentityPrefs extends Struct {
  readonly intended_count: u32;
  readonly running_count: u32;
}

/** @name Permissions */
export interface Permissions extends Struct {
  readonly asset: AssetPermissions;
  readonly extrinsic: ExtrinsicPermissions;
  readonly portfolio: PortfolioPermissions;
}

/** @name Pip */
export interface Pip extends Struct {
  readonly id: PipId;
  readonly proposal: Call;
  readonly state: ProposalState;
  readonly proposer: Proposer;
}

/** @name PipDescription */
export interface PipDescription extends Text {}

/** @name PipId */
export interface PipId extends u32 {}

/** @name PipsMetadata */
export interface PipsMetadata extends Struct {
  readonly id: PipId;
  readonly url: Option<Url>;
  readonly description: Option<PipDescription>;
  readonly created_at: BlockNumber;
  readonly transaction_version: u32;
  readonly expiry: MaybeBlock;
}

/** @name PolymeshVotes */
export interface PolymeshVotes extends Struct {
  readonly index: u32;
  readonly ayes: Vec<ITuple<[IdentityId, Balance]>>;
  readonly nays: Vec<ITuple<[IdentityId, Balance]>>;
  readonly end: BlockNumber;
  readonly expiry: MaybeBlock;
}

/** @name PortfolioId */
export interface PortfolioId extends Struct {
  readonly did: IdentityId;
  readonly kind: PortfolioKind;
}

/** @name PortfolioKind */
export interface PortfolioKind extends Enum {
  readonly isDefault: boolean;
  readonly isUser: boolean;
  readonly asUser: PortfolioNumber;
}

/** @name PortfolioName */
export interface PortfolioName extends Text {}

/** @name PortfolioNumber */
export interface PortfolioNumber extends u64 {}

/** @name PortfolioPermissions */
export interface PortfolioPermissions extends Enum {
  readonly isWhole: boolean;
  readonly isThese: boolean;
  readonly asThese: Vec<PortfolioId>;
  readonly isExcept: boolean;
  readonly asExcept: Vec<PortfolioId>;
}

/** @name PortfolioValidityResult */
export interface PortfolioValidityResult extends Struct {
  readonly receiver_is_same_portfolio: bool;
  readonly sender_portfolio_does_not_exist: bool;
  readonly receiver_portfolio_does_not_exist: bool;
  readonly sender_insufficient_balance: bool;
  readonly result: bool;
}

/** @name PosRatio */
export interface PosRatio extends ITuple<[u32, u32]> {}

/** @name PreAuthorizedKeyInfo */
export interface PreAuthorizedKeyInfo extends Struct {
  readonly target_id: IdentityId;
  readonly secondary_key: SecondaryKey;
}

/** @name PriceTier */
export interface PriceTier extends Struct {
  readonly total: Balance;
  readonly price: Balance;
}

/** @name ProportionMatch */
export interface ProportionMatch extends Enum {
  readonly isAtLeast: boolean;
  readonly isMoreThan: boolean;
}

/** @name ProposalData */
export interface ProposalData extends Enum {
  readonly isHash: boolean;
  readonly asHash: Hash;
  readonly isProposal: boolean;
  readonly asProposal: Bytes;
}

/** @name ProposalDetails */
export interface ProposalDetails extends Struct {
  readonly approvals: u64;
  readonly rejections: u64;
  readonly status: ProposalStatus;
  readonly expiry: Option<Moment>;
  readonly auto_close: bool;
}

/** @name ProposalState */
export interface ProposalState extends Enum {
  readonly isPending: boolean;
  readonly isRejected: boolean;
  readonly isScheduled: boolean;
  readonly isFailed: boolean;
  readonly isExecuted: boolean;
  readonly isExpired: boolean;
}

/** @name ProposalStatus */
export interface ProposalStatus extends Enum {
  readonly isInvalid: boolean;
  readonly isActiveOrExpired: boolean;
  readonly isExecutionSuccessful: boolean;
  readonly isExecutionFailed: boolean;
  readonly isRejected: boolean;
}

/** @name Proposer */
export interface Proposer extends Enum {
  readonly isCommunity: boolean;
  readonly asCommunity: AccountId;
  readonly isCommittee: boolean;
  readonly asCommittee: Committee;
}

/** @name ProtocolOp */
export interface ProtocolOp extends Enum {
  readonly isAssetRegisterTicker: boolean;
  readonly isAssetIssue: boolean;
  readonly isAssetAddDocument: boolean;
  readonly isAssetCreateAsset: boolean;
  readonly isAssetCreateCheckpointSchedule: boolean;
  readonly isDividendNew: boolean;
  readonly isComplianceManagerAddComplianceRequirement: boolean;
  readonly isIdentityRegisterDid: boolean;
  readonly isIdentityCddRegisterDid: boolean;
  readonly isIdentityAddClaim: boolean;
  readonly isIdentitySetPrimaryKey: boolean;
  readonly isIdentityAddSecondaryKeysWithAuthorization: boolean;
  readonly isPipsPropose: boolean;
  readonly isVotingAddBallot: boolean;
  readonly isContractsPutCode: boolean;
  readonly isBallotAttachBallot: boolean;
  readonly isDistributionDistribute: boolean;
}

/** @name ProverTickerKey */
export interface ProverTickerKey extends Struct {
  readonly prover: IdentityId;
  readonly ticker: Ticker;
}

/** @name Receipt */
export interface Receipt extends Struct {
  readonly receipt_uid: u64;
  readonly from: PortfolioId;
  readonly to: PortfolioId;
  readonly asset: Ticker;
  readonly amount: Balance;
}

/** @name ReceiptDetails */
export interface ReceiptDetails extends Struct {
  readonly receipt_uid: u64;
  readonly leg_id: u64;
  readonly signer: AccountId;
  readonly signature: OffChainSignature;
  readonly metadata: ReceiptMetadata;
}

/** @name ReceiptMetadata */
export interface ReceiptMetadata extends Text {}

/** @name RecordDate */
export interface RecordDate extends Struct {
  readonly date: Moment;
  readonly checkpoint: CACheckpoint;
}

/** @name RecordDateSpec */
export interface RecordDateSpec extends Enum {
  readonly isScheduled: boolean;
  readonly asScheduled: Moment;
  readonly isExistingSchedule: boolean;
  readonly asExistingSchedule: ScheduleId;
  readonly isExisting: boolean;
  readonly asExisting: CheckpointId;
}

/** @name RestrictionResult */
export interface RestrictionResult extends Enum {
  readonly isValid: boolean;
  readonly isInvalid: boolean;
  readonly isForceValid: boolean;
}

/** @name RistrettoPoint */
export interface RistrettoPoint extends U8aFixed {}

/** @name Scalar */
export interface Scalar extends U8aFixed {}

/** @name ScheduleId */
export interface ScheduleId extends u64 {}

/** @name ScheduleSpec */
export interface ScheduleSpec extends Struct {
  readonly start: Option<Moment>;
  readonly period: CalendarPeriod;
  readonly remaining: u32;
}

/** @name Scope */
export interface Scope extends Enum {
  readonly isIdentity: boolean;
  readonly asIdentity: IdentityId;
  readonly isTicker: boolean;
  readonly asTicker: Ticker;
  readonly isCustom: boolean;
  readonly asCustom: Bytes;
}

/** @name ScopeClaimProof */
export interface ScopeClaimProof extends Struct {
  readonly proof_scope_id_wellformed: Signature;
  readonly proof_scope_id_cdd_id_match: ZkProofData;
  readonly scope_id: RistrettoPoint;
}

/** @name ScopeId */
export interface ScopeId extends U8aFixed {}

/** @name SecondaryKey */
export interface SecondaryKey extends Struct {
  readonly signer: Signatory;
  readonly permissions: Permissions;
}

/** @name SecondaryKeyWithAuth */
export interface SecondaryKeyWithAuth extends Struct {
  readonly secondary_key: SecondaryKey;
  readonly auth_signature: Signature;
}

/** @name SecurityToken */
export interface SecurityToken extends Struct {
  readonly name: AssetName;
  readonly total_supply: Balance;
  readonly owner_did: IdentityId;
  readonly divisible: bool;
  readonly asset_type: AssetType;
}

/** @name SettlementType */
export interface SettlementType extends Enum {
  readonly isSettleOnAffirmation: boolean;
  readonly isSettleOnBlock: boolean;
  readonly asSettleOnBlock: BlockNumber;
}

/** @name Signatory */
export interface Signatory extends Enum {
  readonly isIdentity: boolean;
  readonly asIdentity: IdentityId;
  readonly isAccount: boolean;
  readonly asAccount: AccountId;
}

/** @name SimpleTokenRecord */
export interface SimpleTokenRecord extends Struct {
  readonly ticker: Ticker;
  readonly total_supply: Balance;
  readonly owner_did: IdentityId;
}

/** @name SkippedCount */
export interface SkippedCount extends u8 {}

/** @name SlashingSwitch */
export interface SlashingSwitch extends Enum {
  readonly isValidator: boolean;
  readonly isValidatorAndNominator: boolean;
  readonly isNone: boolean;
}

/** @name SmartExtension */
export interface SmartExtension extends Struct {
  readonly extension_type: SmartExtensionType;
  readonly extension_name: SmartExtensionName;
  readonly extension_id: AccountId;
  readonly is_archive: bool;
}

/** @name SmartExtensionName */
export interface SmartExtensionName extends Text {}

/** @name SmartExtensionType */
export interface SmartExtensionType extends Enum {
  readonly isTransferManager: boolean;
  readonly isOfferings: boolean;
  readonly isSmartWallet: boolean;
  readonly isCustom: boolean;
  readonly asCustom: Bytes;
}

/** @name SnapshotId */
export interface SnapshotId extends u32 {}

/** @name SnapshotMetadata */
export interface SnapshotMetadata extends Struct {
  readonly created_at: BlockNumber;
  readonly made_by: AccountId;
  readonly id: SnapshotId;
}

/** @name SnapshotResult */
export interface SnapshotResult extends Enum {
  readonly isApprove: boolean;
  readonly isReject: boolean;
  readonly isSkip: boolean;
}

/** @name SnapshottedPip */
export interface SnapshottedPip extends Struct {
  readonly id: PipId;
  readonly weight: ITuple<[bool, Balance]>;
}

/** @name StoredSchedule */
export interface StoredSchedule extends Struct {
  readonly schedule: CheckpointSchedule;
  readonly id: ScheduleId;
  readonly at: Moment;
  readonly remaining: u32;
}

/** @name Subsidy */
export interface Subsidy extends Struct {
  readonly paying_key: AccountId;
  readonly remaining: Balance;
}

/** @name TargetIdAuthorization */
export interface TargetIdAuthorization extends Struct {
  readonly target_id: IdentityId;
  readonly nonce: u64;
  readonly expires_at: Moment;
}

/** @name TargetIdentities */
export interface TargetIdentities extends Struct {
  readonly identities: Vec<IdentityId>;
  readonly treatment: TargetTreatment;
}

/** @name TargetIdentity */
export interface TargetIdentity extends Enum {
  readonly isExternalAgent: boolean;
  readonly isSpecific: boolean;
  readonly asSpecific: IdentityId;
}

/** @name TargetTreatment */
export interface TargetTreatment extends Enum {
  readonly isInclude: boolean;
  readonly isExclude: boolean;
}

/** @name Tax */
export interface Tax extends Permill {}

/** @name TemplateDetails */
export interface TemplateDetails extends Struct {
  readonly instantiation_fee: Balance;
  readonly owner: IdentityId;
  readonly frozen: bool;
}

/** @name TemplateMetadata */
export interface TemplateMetadata extends Struct {
  readonly url: Option<MetaUrl>;
  readonly se_type: SmartExtensionType;
  readonly usage_fee: Balance;
  readonly description: MetaDescription;
  readonly version: MetaVersion;
}

/** @name Ticker */
export interface Ticker extends U8aFixed {}

/** @name TickerRangeProof */
export interface TickerRangeProof extends Struct {
  readonly initial_message: U8aFixed;
  readonly final_response: Bytes;
  readonly max_two_exp: u32;
}

/** @name TickerRegistration */
export interface TickerRegistration extends Struct {
  readonly owner: IdentityId;
  readonly expiry: Option<Moment>;
}

/** @name TickerRegistrationConfig */
export interface TickerRegistrationConfig extends Struct {
  readonly max_ticker_length: u8;
  readonly registration_length: Option<Moment>;
}

/** @name TickerTransferApproval */
export interface TickerTransferApproval extends Struct {
  readonly authorized_by: IdentityId;
  readonly next_ticker: Option<Ticker>;
  readonly previous_ticker: Option<Ticker>;
}

/** @name TransferManager */
export interface TransferManager extends Enum {
  readonly isCountTransferManager: boolean;
  readonly asCountTransferManager: Counter;
  readonly isPercentageTransferManager: boolean;
  readonly asPercentageTransferManager: Percentage;
}

/** @name TransferManagerResult */
export interface TransferManagerResult extends Struct {
  readonly tm: TransferManager;
  readonly result: bool;
}

/** @name TrustedFor */
export interface TrustedFor extends Enum {
  readonly isAny: boolean;
  readonly isSpecific: boolean;
  readonly asSpecific: Vec<ClaimType>;
}

/** @name TrustedIssuer */
export interface TrustedIssuer extends Struct {
  readonly issuer: IdentityId;
  readonly trusted_for: TrustedFor;
}

/** @name UniqueCall */
export interface UniqueCall extends Struct {
  readonly nonce: u64;
  readonly call: Call;
}

/** @name Url */
export interface Url extends Text {}

/** @name ValidatorPrefsWithBlocked */
export interface ValidatorPrefsWithBlocked extends Struct {
  readonly commission: Compact<Perbill>;
}

/** @name Venue */
export interface Venue extends Struct {
  readonly creator: IdentityId;
  readonly instructions: Vec<u64>;
  readonly details: VenueDetails;
  readonly venue_type: VenueType;
}

/** @name VenueDetails */
export interface VenueDetails extends Text {}

/** @name VenueType */
export interface VenueType extends Enum {
  readonly isOther: boolean;
  readonly isDistribution: boolean;
  readonly isSto: boolean;
  readonly isExchange: boolean;
}

/** @name Version */
export interface Version extends u8 {}

/** @name Vote */
export interface Vote extends ITuple<[bool, Balance]> {}

/** @name VoteByPip */
export interface VoteByPip extends Struct {
  readonly pip: PipId;
  readonly vote: Vote;
}

/** @name VoteCount */
export interface VoteCount extends Enum {
  readonly isProposalFound: boolean;
  readonly asProposalFound: VoteCountProposalFound;
  readonly isProposalNotFound: boolean;
  readonly asProposalNotFound: Bytes;
}

/** @name VoteCountProposalFound */
export interface VoteCountProposalFound extends Struct {
  readonly ayes: u64;
  readonly nays: u64;
}

/** @name VotingResult */
export interface VotingResult extends Struct {
  readonly ayes_count: u32;
  readonly ayes_stake: Balance;
  readonly nays_count: u32;
  readonly nays_stake: Balance;
}

/** @name WeightToFeeCoefficient */
export interface WeightToFeeCoefficient extends Struct {
  readonly coeffInteger: Balance;
  readonly coeffFrac: Perbill;
  readonly negative: bool;
  readonly degree: u8;
}

/** @name ZkProofData */
export interface ZkProofData extends Struct {
  readonly challenge_responses: Vec<Scalar>;
  readonly subtract_expressions_res: RistrettoPoint;
  readonly blinded_scope_did_hash: RistrettoPoint;
}

export type PHANTOM_DEFAULT = 'default';
