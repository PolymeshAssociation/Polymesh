import { u8, u32, u64 } from "@polkadot/types/primitive";
import { AccountId, Moment } from "@polkadot/types/interfaces/runtime";
import { Option } from "fp-ts/lib/Option";

export interface PortfolioKind {
	Default: string;
	User: PortfolioNumber;
}

export interface TargetIdentity {
	PrimaryIssuanceAgent: string;
	Specific: IdentityId;
}

export interface Claim {
	Accredited: Partial<Scope>;
	Affiliate: Partial<Scope>;
	BuyLockup: Partial<Scope>;
	SellLockup: Partial<Scope>;
	CustomerDueDiligence: CddId;
	KnowYourCustomer: Partial<Scope>;
	Jurisdiction: [CountryCode, Partial<Scope>];
	Exempted: Partial<Scope>;
	Blocked: Partial<Scope>;
	InvestorUniqueness: [Partial<Scope>, ScopeId, CddId];
	NoData: string;
}

export interface ClaimType {
	Accredited: string;
	Affiliate: string;
	BuyLockup: string;
	SellLockup: string;
	CustomerDueDiligence: string;
	KnowYourCustomer: string;
	Jurisdiction: string;
	Exempted: string;
	Blocked: string;
	InvestorUniqueness: string;
	NoData: string;
}

export interface AuthorizationData {
	AttestPrimaryKeyRotation: IdentityId;
	RotatePrimaryKey: IdentityId;
	TransferTicker: Ticker;
	TransferPrimaryIssuanceAgent: Ticker;
	AddMultiSigSigner: AccountId;
	TransferAssetOwnership: Ticker;
	JoinIdentity: Permissions;
	PortfolioCustody: PortfolioId;
	Custom: Ticker;
	NoData: string;
	TransferCorporateActionAgent: Ticker;
}

export interface ConditionType {
	IsPresent: Partial<Claim>;
	IsAbsent: Partial<Claim>;
	IsAnyOf: Partial<Claim>[];
	IsNoneOf: Partial<Claim>[];
	IsIdentity: Partial<TargetIdentity>;
}

export interface TrustedFor {
	Any: string;
	Specific: Partial<ClaimType>[];
}

export type IdentityId = [u8: 32];
export type Ticker = [u8, 12];
export type NonceObject = { nonce: string };
export type PortfolioNumber = u64;
export type ScopeId = [u8, 32];
export type CddId = [u8, 32];
export type PalletName = string;
export type DispatchableName = string;

export type Permissions = {
	asset: Option<Ticker[]>;
	extrinsic: Option<PalletPermissions[]>;
	portfolio: Option<PortfolioId[]>;
};

export type PalletPermissions = {
	pallet_name: PalletName;
	dispatchable_names: Option<DispatchableName[]>;
};

export type PortfolioId = {
	did: IdentityId;
	kind: Partial<PortfolioKind>;
};

export type TickerRegistration = {
	owner: IdentityId;
	expiry: Option<Moment>;
};

export type Authorization = {
	authorization_data: Partial<AuthorizationData>;
	authorized_by: IdentityId;
	expiry: Option<Moment>;
	auth_id: u64;
};
export type Scope = {
	Identity: IdentityId;
	Ticker: Ticker;
	Custom: u8[];
};

export type TrustedIssuer = {
	issuer: IdentityId;
	trusted_for: Partial<TrustedFor>;
};

export type Condition = {
	condition_type: Partial<ConditionType>;
	issuers: TrustedIssuer[];
};

export type ComplianceRequirement = {
	sender_conditions: Condition[];
	receiver_conditions: Condition[];
	id: u32;
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
