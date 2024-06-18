import type { AccountId } from "@polkadot/types/interfaces/runtime";
import type { IdentityId, CountryCode } from "../src/interfaces";

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
  BecomeAgent?: (string | AgentGroup)[];
}

export interface AgentGroup {
  Full?: string;
  Custom?: number;
  ExceptMeta?: string;
  PolymeshV1CAA?: string;
  PolymeshV1PIA?: string;
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
export type Signatory = { Identity: IdentityId } | { Account: AccountId | Uint8Array};
export type venueType = "Other" | "Distribution" | "Sto" | "Exchange" | number | Uint8Array;

export type Claim =
  | { Accredited: Scope }
  | { Affiliate: Scope }
  | { BuyLockup: Scope }
  | { SellLockup: Scope }
  | { CustomerDueDiligence: CddId }
  | { KnowYourCustomer: Scope }
  | { Jurisdiction: [CountryCode, Scope] }
  | { Exempted: Scope }
  | { Blocked: Scope };

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
export type ExtrinsicPermissions = SubsetRestriction<[PalletName, PalletPermissions]>;
export type PortfolioPermissions = SubsetRestriction<PortfolioId>;

export type Permissions = {
  asset: AssetPermissions;
  extrinsic: ExtrinsicPermissions;
  portfolio: PortfolioPermissions;
};

export type PalletPermissions = {
  extrinsics: SubsetRestriction<DispatchableName>;
};

export type PortfolioId = {
  did: IdentityId;
  kind: PortfolioKind;
};

export type Authorization = {
  authorization_data: AuthorizationData;
  authorized_by: IdentityId;
  expiry: Expiry;
  auth_id: number;
};

export type PriceTier = {
  total: number;
  price: number;
}
