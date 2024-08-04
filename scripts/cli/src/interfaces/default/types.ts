// Auto-generated via `yarn polkadot-types-from-defs`, do not edit
/* eslint-disable */

import type { Bytes, Enum, Option, Struct, Text, U8aFixed, Vec, bool, u32, u64 } from '@polkadot/types-codec';
import type { ITuple } from '@polkadot/types-codec/types';
import type { AccountId, Balance, Permill, Weight } from '@polkadot/types/interfaces/runtime';
import type { DispatchError } from '@polkadot/types/interfaces/system';

/** @name AffirmationCount */
export interface AffirmationCount extends Struct {
  readonly sender_asset_count: AssetCount;
  readonly receiver_asset_count: AssetCount;
  readonly offchain_count: u32;
}

/** @name AgentGroup */
export interface AgentGroup extends Enum {
  readonly isFull: boolean;
  readonly isCustom: boolean;
  readonly asCustom: AGId;
  readonly isExceptMeta: boolean;
  readonly isPolymeshV1CAA: boolean;
  readonly isPolymeshV1PIA: boolean;
  readonly type: 'Full' | 'Custom' | 'ExceptMeta' | 'PolymeshV1CAA' | 'PolymeshV1PIA';
}

/** @name AGId */
export interface AGId extends u32 {}

/** @name AssetComplianceResult */
export interface AssetComplianceResult extends Struct {
  readonly paused: bool;
  readonly requirements: Vec<ComplianceRequirementResult>;
  readonly result: bool;
}

/** @name AssetCount */
export interface AssetCount extends Struct {
  readonly fungible_tokens: u32;
  readonly non_fungible_tokens: u32;
  readonly off_chain_assets: u32;
}

/** @name AssetDidResult */
export interface AssetDidResult extends Enum {
  readonly isOk: boolean;
  readonly asOk: IdentityId;
  readonly isErr: boolean;
  readonly asErr: Bytes;
  readonly type: 'Ok' | 'Err';
}

/** @name AssetPermissions */
export interface AssetPermissions extends Enum {
  readonly isWhole: boolean;
  readonly isThese: boolean;
  readonly asThese: Vec<Ticker>;
  readonly isExcept: boolean;
  readonly asExcept: Vec<Ticker>;
  readonly type: 'Whole' | 'These' | 'Except';
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
  readonly isTransferTicker: boolean;
  readonly asTransferTicker: Ticker;
  readonly isAddMultiSigSigner: boolean;
  readonly asAddMultiSigSigner: AccountId;
  readonly isTransferAssetOwnership: boolean;
  readonly asTransferAssetOwnership: Ticker;
  readonly isJoinIdentity: boolean;
  readonly asJoinIdentity: Permissions;
  readonly isPortfolioCustody: boolean;
  readonly asPortfolioCustody: PortfolioId;
  readonly isBecomeAgent: boolean;
  readonly asBecomeAgent: ITuple<[Ticker, AgentGroup]>;
  readonly isAddRelayerPayingKey: boolean;
  readonly asAddRelayerPayingKey: ITuple<[AccountId, AccountId, Balance]>;
  readonly isRotatePrimaryKeyToSecondary: boolean;
  readonly asRotatePrimaryKeyToSecondary: Permissions;
  readonly type: 'AttestPrimaryKeyRotation' | 'RotatePrimaryKey' | 'TransferTicker' | 'AddMultiSigSigner' | 'TransferAssetOwnership' | 'JoinIdentity' | 'PortfolioCustody' | 'BecomeAgent' | 'AddRelayerPayingKey' | 'RotatePrimaryKeyToSecondary';
}

/** @name AuthorizationType */
export interface AuthorizationType extends Enum {
  readonly isAttestPrimaryKeyRotation: boolean;
  readonly isRotatePrimaryKey: boolean;
  readonly isTransferTicker: boolean;
  readonly isAddMultiSigSigner: boolean;
  readonly isTransferAssetOwnership: boolean;
  readonly isJoinIdentity: boolean;
  readonly isPortfolioCustody: boolean;
  readonly isBecomeAgent: boolean;
  readonly isAddRelayerPayingKey: boolean;
  readonly isRotatePrimaryKeyToSecondary: boolean;
  readonly type: 'AttestPrimaryKeyRotation' | 'RotatePrimaryKey' | 'TransferTicker' | 'AddMultiSigSigner' | 'TransferAssetOwnership' | 'JoinIdentity' | 'PortfolioCustody' | 'BecomeAgent' | 'AddRelayerPayingKey' | 'RotatePrimaryKeyToSecondary';
}

/** @name CanTransferGranularReturn */
export interface CanTransferGranularReturn extends Enum {
  readonly isOk: boolean;
  readonly asOk: GranularCanTransferResult;
  readonly isErr: boolean;
  readonly asErr: DispatchError;
  readonly type: 'Ok' | 'Err';
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
  readonly type: 'Ok' | 'Err';
}

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
  readonly isCustom: boolean;
  readonly asCustom: ITuple<[CustomClaimTypeId, Option<Scope>]>;
  readonly type: 'Accredited' | 'Affiliate' | 'BuyLockup' | 'SellLockup' | 'CustomerDueDiligence' | 'KnowYourCustomer' | 'Jurisdiction' | 'Exempted' | 'Blocked' | 'Custom';
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
  readonly isCustom: boolean;
  readonly asCustom: CustomClaimTypeId;
  readonly type: 'Accredited' | 'Affiliate' | 'BuyLockup' | 'SellLockup' | 'CustomerDueDiligence' | 'KnowYourCustomer' | 'Jurisdiction' | 'Exempted' | 'Blocked' | 'Custom';
}

/** @name ComplianceReport */
export interface ComplianceReport extends Struct {
  readonly any_requirement_satistifed: bool;
  readonly paused_compliance: bool;
  readonly requirements: Vec<RequirementReport>;
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

/** @name ConditionReport */
export interface ConditionReport extends Struct {
  readonly satisfied: bool;
  readonly condition: Condition;
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
  readonly type: 'IsPresent' | 'IsAbsent' | 'IsAnyOf' | 'IsNoneOf' | 'IsIdentity';
}

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
  readonly type: 'Af' | 'Ax' | 'Al' | 'Dz' | 'As' | 'Ad' | 'Ao' | 'Ai' | 'Aq' | 'Ag' | 'Ar' | 'Am' | 'Aw' | 'Au' | 'At' | 'Az' | 'Bs' | 'Bh' | 'Bd' | 'Bb' | 'By' | 'Be' | 'Bz' | 'Bj' | 'Bm' | 'Bt' | 'Bo' | 'Ba' | 'Bw' | 'Bv' | 'Br' | 'Vg' | 'Io' | 'Bn' | 'Bg' | 'Bf' | 'Bi' | 'Kh' | 'Cm' | 'Ca' | 'Cv' | 'Ky' | 'Cf' | 'Td' | 'Cl' | 'Cn' | 'Hk' | 'Mo' | 'Cx' | 'Cc' | 'Co' | 'Km' | 'Cg' | 'Cd' | 'Ck' | 'Cr' | 'Ci' | 'Hr' | 'Cu' | 'Cy' | 'Cz' | 'Dk' | 'Dj' | 'Dm' | 'Do' | 'Ec' | 'Eg' | 'Sv' | 'Gq' | 'Er' | 'Ee' | 'Et' | 'Fk' | 'Fo' | 'Fj' | 'Fi' | 'Fr' | 'Gf' | 'Pf' | 'Tf' | 'Ga' | 'Gm' | 'Ge' | 'De' | 'Gh' | 'Gi' | 'Gr' | 'Gl' | 'Gd' | 'Gp' | 'Gu' | 'Gt' | 'Gg' | 'Gn' | 'Gw' | 'Gy' | 'Ht' | 'Hm' | 'Va' | 'Hn' | 'Hu' | 'Is' | 'In' | 'Id' | 'Ir' | 'Iq' | 'Ie' | 'Im' | 'Il' | 'It' | 'Jm' | 'Jp' | 'Je' | 'Jo' | 'Kz' | 'Ke' | 'Ki' | 'Kp' | 'Kr' | 'Kw' | 'Kg' | 'La' | 'Lv' | 'Lb' | 'Ls' | 'Lr' | 'Ly' | 'Li' | 'Lt' | 'Lu' | 'Mk' | 'Mg' | 'Mw' | 'My' | 'Mv' | 'Ml' | 'Mt' | 'Mh' | 'Mq' | 'Mr' | 'Mu' | 'Yt' | 'Mx' | 'Fm' | 'Md' | 'Mc' | 'Mn' | 'Me' | 'Ms' | 'Ma' | 'Mz' | 'Mm' | 'Na' | 'Nr' | 'Np' | 'Nl' | 'An' | 'Nc' | 'Nz' | 'Ni' | 'Ne' | 'Ng' | 'Nu' | 'Nf' | 'Mp' | 'No' | 'Om' | 'Pk' | 'Pw' | 'Ps' | 'Pa' | 'Pg' | 'Py' | 'Pe' | 'Ph' | 'Pn' | 'Pl' | 'Pt' | 'Pr' | 'Qa' | 'Re' | 'Ro' | 'Ru' | 'Rw' | 'Bl' | 'Sh' | 'Kn' | 'Lc' | 'Mf' | 'Pm' | 'Vc' | 'Ws' | 'Sm' | 'St' | 'Sa' | 'Sn' | 'Rs' | 'Sc' | 'Sl' | 'Sg' | 'Sk' | 'Si' | 'Sb' | 'So' | 'Za' | 'Gs' | 'Ss' | 'Es' | 'Lk' | 'Sd' | 'Sr' | 'Sj' | 'Sz' | 'Se' | 'Ch' | 'Sy' | 'Tw' | 'Tj' | 'Tz' | 'Th' | 'Tl' | 'Tg' | 'Tk' | 'To' | 'Tt' | 'Tn' | 'Tr' | 'Tm' | 'Tc' | 'Tv' | 'Ug' | 'Ua' | 'Ae' | 'Gb' | 'Us' | 'Um' | 'Uy' | 'Uz' | 'Vu' | 'Ve' | 'Vn' | 'Vi' | 'Wf' | 'Eh' | 'Ye' | 'Zm' | 'Zw' | 'Bq' | 'Cw' | 'Sx';
}

/** @name CustomClaimTypeId */
export interface CustomClaimTypeId extends u32 {}

/** @name DidStatus */
export interface DidStatus extends Enum {
  readonly isUnknown: boolean;
  readonly isExists: boolean;
  readonly isCddVerified: boolean;
  readonly type: 'Unknown' | 'Exists' | 'CddVerified';
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
  readonly type: 'Whole' | 'These' | 'Except';
}

/** @name ExecuteInstructionInfo */
export interface ExecuteInstructionInfo extends Struct {
  readonly fungible_tokens: u32;
  readonly non_fungible_tokens: u32;
  readonly off_chain_assets: u32;
  readonly consumed_weight: Weight;
  readonly error: Option<Text>;
}

/** @name ExtrinsicPermissions */
export interface ExtrinsicPermissions extends Enum {
  readonly isWhole: boolean;
  readonly isThese: boolean;
  readonly asThese: Vec<PalletPermissions>;
  readonly isExcept: boolean;
  readonly asExcept: Vec<PalletPermissions>;
  readonly type: 'Whole' | 'These' | 'Except';
}

/** @name FungibleLeg */
export interface FungibleLeg extends Struct {
  readonly sender: PortfolioId;
  readonly receiver: PortfolioId;
  readonly ticker: Ticker;
  readonly amount: Balance;
}

/** @name GranularCanTransferResult */
export interface GranularCanTransferResult extends Struct {
  readonly invalid_granularity: bool;
  readonly self_transfer: bool;
  readonly invalid_receiver_cdd: bool;
  readonly invalid_sender_cdd: bool;
  readonly receiver_custodian_error: bool;
  readonly sender_custodian_error: bool;
  readonly sender_insufficient_balance: bool;
  readonly portfolio_validity_result: PortfolioValidityResult;
  readonly asset_frozen: bool;
  readonly transfer_condition_result: Vec<TransferConditionResult>;
  readonly compliance_result: AssetComplianceResult;
  readonly result: bool;
  readonly consumed_weight: Option<Weight>;
}

/** @name IdentityClaim */
export interface IdentityClaim extends Struct {
  readonly claim_issuer: IdentityId;
  readonly issuance_date: Moment;
  readonly last_update_date: Moment;
  readonly expiry: Option<Moment>;
  readonly claim: Claim;
}

/** @name IdentityId */
export interface IdentityId extends U8aFixed {}

/** @name InstructionId */
export interface InstructionId extends u64 {}

/** @name KeyIdentityData */
export interface KeyIdentityData extends Struct {
  readonly identity: IdentityId;
  readonly permissions: Option<Permissions>;
}

/** @name Leg */
export interface Leg extends Enum {
  readonly isFungible: boolean;
  readonly asFungible: FungibleLeg;
  readonly isNonFungible: boolean;
  readonly asNonFungible: NonFungibleLeg;
  readonly isOffChain: boolean;
  readonly asOffChain: OffChainLeg;
  readonly type: 'Fungible' | 'NonFungible' | 'OffChain';
}

/** @name Member */
export interface Member extends Struct {
  readonly id: IdentityId;
  readonly expiry_at: Option<Moment>;
  readonly inactive_from: Option<Moment>;
}

/** @name Moment */
export interface Moment extends u64 {}

/** @name NFTId */
export interface NFTId extends u64 {}

/** @name NFTs */
export interface NFTs extends Struct {
  readonly ticker: Ticker;
  readonly ids: Vec<NFTId>;
}

/** @name NonFungibleLeg */
export interface NonFungibleLeg extends Struct {
  readonly sender: PortfolioId;
  readonly receiver: PortfolioId;
  readonly nfts: NFTs;
}

/** @name OffChainLeg */
export interface OffChainLeg extends Struct {
  readonly sender_identity: IdentityId;
  readonly receiver_identity: IdentityId;
  readonly ticker: Ticker;
  readonly amount: Balance;
}

/** @name PalletName */
export interface PalletName extends Text {}

/** @name PalletPermissions */
export interface PalletPermissions extends Struct {
  readonly pallet_name: PalletName;
  readonly dispatchable_names: DispatchableNames;
}

/** @name Percentage */
export interface Percentage extends Permill {}

/** @name Permissions */
export interface Permissions extends Struct {
  readonly asset: AssetPermissions;
  readonly extrinsic: ExtrinsicPermissions;
  readonly portfolio: PortfolioPermissions;
}

/** @name PipId */
export interface PipId extends u32 {}

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
  readonly type: 'Default' | 'User';
}

/** @name PortfolioNumber */
export interface PortfolioNumber extends u64 {}

/** @name PortfolioPermissions */
export interface PortfolioPermissions extends Enum {
  readonly isWhole: boolean;
  readonly isThese: boolean;
  readonly asThese: Vec<PortfolioId>;
  readonly isExcept: boolean;
  readonly asExcept: Vec<PortfolioId>;
  readonly type: 'Whole' | 'These' | 'Except';
}

/** @name PortfolioValidityResult */
export interface PortfolioValidityResult extends Struct {
  readonly receiver_is_same_portfolio: bool;
  readonly sender_portfolio_does_not_exist: bool;
  readonly receiver_portfolio_does_not_exist: bool;
  readonly sender_insufficient_balance: bool;
  readonly result: bool;
}

/** @name ProtocolOp */
export interface ProtocolOp extends Enum {
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

/** @name RequirementReport */
export interface RequirementReport extends Struct {
  readonly requirement_satisfied: bool;
  readonly id: u32;
  readonly sender_conditions: Vec<ConditionReport>;
  readonly receiver_conditions: Vec<ConditionReport>;
}

/** @name RpcDidRecords */
export interface RpcDidRecords extends Enum {
  readonly isSuccess: boolean;
  readonly asSuccess: RpcDidRecordsSuccess;
  readonly isIdNotFound: boolean;
  readonly asIdNotFound: Bytes;
  readonly type: 'Success' | 'IdNotFound';
}

/** @name RpcDidRecordsSuccess */
export interface RpcDidRecordsSuccess extends Struct {
  readonly primary_key: AccountId;
  readonly secondary_keys: Vec<SecondaryKey>;
}

/** @name Scope */
export interface Scope extends Enum {
  readonly isIdentity: boolean;
  readonly asIdentity: IdentityId;
  readonly isTicker: boolean;
  readonly asTicker: Ticker;
  readonly isCustom: boolean;
  readonly asCustom: Bytes;
  readonly type: 'Identity' | 'Ticker' | 'Custom';
}

/** @name SecondaryKey */
export interface SecondaryKey extends Struct {
  readonly key: AccountId;
  readonly permissions: Permissions;
}

/** @name Signatory */
export interface Signatory extends Enum {
  readonly isIdentity: boolean;
  readonly asIdentity: IdentityId;
  readonly isAccount: boolean;
  readonly asAccount: AccountId;
  readonly type: 'Identity' | 'Account';
}

/** @name StatClaim */
export interface StatClaim extends Enum {
  readonly isAccredited: boolean;
  readonly asAccredited: bool;
  readonly isAffiliate: boolean;
  readonly asAffiliate: bool;
  readonly isJurisdiction: boolean;
  readonly asJurisdiction: Option<CountryCode>;
  readonly type: 'Accredited' | 'Affiliate' | 'Jurisdiction';
}

/** @name TargetIdentity */
export interface TargetIdentity extends Enum {
  readonly isExternalAgent: boolean;
  readonly isSpecific: boolean;
  readonly asSpecific: IdentityId;
  readonly type: 'ExternalAgent' | 'Specific';
}

/** @name Ticker */
export interface Ticker extends U8aFixed {}

/** @name TransferCondition */
export interface TransferCondition extends Enum {
  readonly isMaxInvestorCount: boolean;
  readonly asMaxInvestorCount: u64;
  readonly isMaxInvestorOwnership: boolean;
  readonly asMaxInvestorOwnership: Percentage;
  readonly isClaimCount: boolean;
  readonly asClaimCount: ITuple<[StatClaim, IdentityId, u64, Option<u64>]>;
  readonly isClaimOwnership: boolean;
  readonly asClaimOwnership: ITuple<[StatClaim, IdentityId, Percentage, Percentage]>;
  readonly type: 'MaxInvestorCount' | 'MaxInvestorOwnership' | 'ClaimCount' | 'ClaimOwnership';
}

/** @name TransferConditionResult */
export interface TransferConditionResult extends Struct {
  readonly condition: TransferCondition;
  readonly result: bool;
}

/** @name TrustedFor */
export interface TrustedFor extends Enum {
  readonly isAny: boolean;
  readonly isSpecific: boolean;
  readonly asSpecific: Vec<ClaimType>;
  readonly type: 'Any' | 'Specific';
}

/** @name TrustedIssuer */
export interface TrustedIssuer extends Struct {
  readonly issuer: IdentityId;
  readonly trusted_for: TrustedFor;
}

/** @name VoteCount */
export interface VoteCount extends Enum {
  readonly isProposalFound: boolean;
  readonly asProposalFound: VoteCountProposalFound;
  readonly isProposalNotFound: boolean;
  readonly type: 'ProposalFound' | 'ProposalNotFound';
}

/** @name VoteCountProposalFound */
export interface VoteCountProposalFound extends Struct {
  readonly ayes: u64;
  readonly nays: u64;
}

export type PHANTOM_DEFAULT = 'default';
