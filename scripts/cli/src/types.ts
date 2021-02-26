import type { AccountId } from "@polkadot/types/interfaces/runtime";
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
export interface Signatory {
	Identity?: IdentityId;
	Account?: AccountId;
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

export type IdentityId = number;
export type Ticker = string;
export type NonceObject = { nonce: string };
export type PortfolioNumber = number;
export type ScopeId = string;
export type CddId = string;
export type PalletName = string;
export type DispatchableName = string;
export type Expiry = number | undefined;
export type DocumentName = string;
export type DocumentUri = string;

export type MovePortfolioItem = {
	ticker: Ticker;
	amount: number;
};

export type Permissions = {
	asset?: Ticker[];
	extrinsic?: PalletPermissions[] | LegacyPalletPermissions[];
	portfolio?: PortfolioId[];
};

export type PalletPermissions = {
	pallet_name: PalletName;
	dispatchable_names?: DispatchableName[];
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

export enum CountryCode {
	AF,
	AX,
	AL,
	DZ,
	AS,
	AD,
	AO,
	AI,
	AQ,
	AG,
	AR,
	AM,
	AW,
	AU,
	AT,
	AZ,
	BS,
	BH,
	BD,
	BB,
	BY,
	BE,
	BZ,
	BJ,
	BM,
	BT,
	BO,
	BA,
	BW,
	BV,
	BR,
	VG,
	IO,
	BN,
	BG,
	BF,
	BI,
	KH,
	CM,
	CA,
	CV,
	KY,
	CF,
	TD,
	CL,
	CN,
	HK,
	MO,
	CX,
	CC,
	CO,
	KM,
	CG,
	CD,
	CK,
	CR,
	CI,
	HR,
	CU,
	CY,
	CZ,
	DK,
	DJ,
	DM,
	DO,
	EC,
	EG,
	SV,
	GQ,
	ER,
	EE,
	ET,
	FK,
	FO,
	FJ,
	FI,
	FR,
	GF,
	PF,
	TF,
	GA,
	GM,
	GE,
	DE,
	GH,
	GI,
	GR,
	GL,
	GD,
	GP,
	GU,
	GT,
	GG,
	GN,
	GW,
	GY,
	HT,
	HM,
	VA,
	HN,
	HU,
	IS,
	IN,
	ID,
	IR,
	IQ,
	IE,
	IM,
	IL,
	IT,
	JM,
	JP,
	JE,
	JO,
	KZ,
	KE,
	KI,
	KP,
	KR,
	KW,
	KG,
	LA,
	LV,
	LB,
	LS,
	LR,
	LY,
	LI,
	LT,
	LU,
	MK,
	MG,
	MW,
	MY,
	MV,
	ML,
	MT,
	MH,
	MQ,
	MR,
	MU,
	YT,
	MX,
	FM,
	MD,
	MC,
	MN,
	ME,
	MS,
	MA,
	MZ,
	MM,
	NA,
	NR,
	NP,
	NL,
	AN,
	NC,
	NZ,
	NI,
	NE,
	NG,
	NU,
	NF,
	MP,
	NO,
	OM,
	PK,
	PW,
	PS,
	PA,
	PG,
	PY,
	PE,
	PH,
	PN,
	PL,
	PT,
	PR,
	QA,
	RE,
	RO,
	RU,
	RW,
	BL,
	SH,
	KN,
	LC,
	MF,
	PM,
	VC,
	WS,
	SM,
	ST,
	SA,
	SN,
	RS,
	SC,
	SL,
	SG,
	SK,
	SI,
	SB,
	SO,
	ZA,
	GS,
	SS,
	ES,
	LK,
	SD,
	SR,
	SJ,
	SZ,
	SE,
	CH,
	SY,
	TW,
	TJ,
	TZ,
	TH,
	TL,
	TG,
	TK,
	TO,
	TT,
	TN,
	TR,
	TM,
	TC,
	TV,
	UG,
	UA,
	AE,
	GB,
	US,
	UM,
	UY,
	UZ,
	VU,
	VE,
	VN,
	VI,
	WF,
	EH,
	YE,
	ZM,
	ZW,
}
