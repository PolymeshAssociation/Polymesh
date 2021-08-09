import type { AccountId } from "@polkadot/types/interfaces/runtime";
import type { IdentityId, CountryCode } from "../src/interfaces";
import type { AnyNumber } from "@polkadot/types/types";
//import { Option } from "fp-ts/lib/Option";
//import type { Option, Vec } from '@polkadot/types/codec';

export interface Document {
  uri: DocumentUri;
  content_hash: DocumentHash;
  name: DocumentName;
  doc_type?: string;
  filing_date?: number;
}
export interface DocumentHash {
  None?: string;
  H512?: [Uint8Array, 64];
  H384?: [Uint8Array, 48];
  H320?: [Uint8Array, 40];
  H256?: [Uint8Array, 32];
  H224?: [Uint8Array, 28];
  H192?: [Uint8Array, 24];
  H160?: [Uint8Array, 20];
  H128?: [Uint8Array, 16];
}
export interface Scope {
  Identity?: IdentityId;
  Ticker?: Ticker;
  Custom?: number[];
}
export interface PortfolioKind {
  Default?: string;
  User?: PortfolioNumber;
}

export interface TargetIdentity {
  PrimaryIssuanceAgent?: string;
  Specific?: IdentityId;
}

export interface Claim {
  Accredited?: Scope;
  Affiliate?: Scope;
  BuyLockup?: Scope;
  SellLockup?: Scope;
  CustomerDueDiligence?: CddId;
  KnowYourCustomer?: Scope;
  Jurisdiction?: [CountryCode, Scope];
  Exempted?: Scope;
  Blocked?: Scope;
  InvestorUniqueness?: [Scope, ScopeId, CddId];
  NoData?: string;
}

export interface ClaimType {
  Accredited?: string;
  Affiliate?: string;
  BuyLockup?: string;
  SellLockup?: string;
  CustomerDueDiligence?: string;
  KnowYourCustomer?: string;
  Jurisdiction?: string;
  Exempted?: string;
  Blocked?: string;
  InvestorUniqueness?: string;
  NoData?: string;
}

export interface AuthorizationData {
	AttestPrimaryKeyRotation?: IdentityId;
	RotatePrimaryKey?: IdentityId;
	TransferTicker?: Ticker;
	TransferPrimaryIssuanceAgent?: Ticker;
	AddMultiSigSigner?: AccountId;
	TransferAssetOwnership?: Ticker;
	JoinIdentity?: Permissions;
	PortfolioCustody?: PortfolioId;
	Custom?: Ticker;
	NoData?: string;
	TransferCorporateActionAgent?: Ticker;
	BecomeAgent?: { Ticker: Ticker; AgentGroup: AgentGroup };
}

export interface AgentGroup {
	Full?: string;
	Custom?: number;
	ExceptMeta?: string;
	PolymeshV1CAA?: string;
	PolymeshV1PIA?: string;
}

export interface ConditionType {
  IsPresent?: Claim;
  IsAbsent?: Claim;
  IsAnyOf?: Claim[];
  IsNoneOf?: Claim[];
  IsIdentity?: TargetIdentity;
}

export interface TrustedFor {
  Any?: string;
  Specific?: ClaimType[];
}

export type Ticker = string;
export type NonceObject = { nonce: string };
export type PortfolioNumber = number;
export type ScopeId = string;
export type CddId = string;
export type PalletName = string;
export type DispatchableName = string;
export type Expiry = string | object | Uint8Array | null;
export type DocumentName = string;
export type DocumentUri = string;
export type Signatory = { Identity: IdentityId } | { Account: AccountId };

export type MovePortfolioItem = {
  ticker: Ticker;
  amount: number;
};
export type Whole = {
  Whole: undefined;
};
export type These<T> = {
  These: T[];
};
export type Except<T> = {
  Except: T[];
};
export type SubsetRestriction<T> = Whole | These<T> | Except<T>;
export type AssetPermissions = SubsetRestriction<Ticker>;
export type ExtrinsicPermissions = SubsetRestriction<PalletPermissions>;
export type PortfolioPermissions = SubsetRestriction<PortfolioId>;

export type Permissions = {
  asset: AssetPermissions;
  extrinsic: ExtrinsicPermissions;
  portfolio: PortfolioPermissions;
};

export type PalletPermissions = {
  pallet_name: PalletName;
  dispatchable_names: SubsetRestriction<DispatchableName>;
};

export type LegacyPalletPermissions = {
  pallet_name: PalletName;
  total: Boolean;
  dispatchable_names: DispatchableName[];
};

export type PortfolioId = {
  did: IdentityId;
  kind: PortfolioKind;
};

export type TickerRegistration = {
  owner: IdentityId;
  expiry: Expiry;
};

export type Authorization = {
  authorization_data: AuthorizationData;
  authorized_by: IdentityId;
  expiry: Expiry;
  auth_id: number;
};

export type TrustedIssuer = {
  issuer: IdentityId;
  trusted_for: TrustedFor;
};

export type Condition = {
  condition_type: ConditionType;
  issuers: TrustedIssuer[];
};

export type ComplianceRequirement = {
  sender_conditions: Condition[];
  receiver_conditions: Condition[];
  id: number;
};

export type AssetCompliance = {
  is_paused: Boolean;
  requirements: ComplianceRequirement[];
};
