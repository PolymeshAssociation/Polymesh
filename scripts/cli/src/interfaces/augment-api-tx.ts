// Auto-generated via `yarn polkadot-types-from-chain`, do not edit
/* eslint-disable */

import type { ApiTypes } from '@polkadot/api-base/types';
import type { Bytes, Compact, Option, Vec, bool, u128, u16, u32, u64, u8 } from '@polkadot/types-codec';
import type { AnyNumber, ITuple } from '@polkadot/types-codec/types';
import type { AccountId32, Call, H256, MultiAddress, Perbill, Percent, Permill } from '@polkadot/types/interfaces/runtime';
import type { BTreeSetIdentityId, BTreeSetStatType, BTreeSetStatUpdate, BTreeSetTransferCondition, ConfidentialIdentityClaimProofsScopeClaimProof, FrameSupportScheduleMaybeHashed, PalletAssetCheckpointScheduleSpec, PalletAssetClassicTickerImport, PalletAssetTickerRegistrationConfig, PalletBridgeBridgeTx, PalletCorporateActionsBallotBallotMeta, PalletCorporateActionsBallotBallotTimeRange, PalletCorporateActionsBallotBallotVote, PalletCorporateActionsCaId, PalletCorporateActionsCaKind, PalletCorporateActionsInitiateCorporateActionArgs, PalletCorporateActionsRecordDateSpec, PalletCorporateActionsTargetIdentities, PalletImOnlineHeartbeat, PalletImOnlineSr25519AppSr25519Signature, PalletPipsSnapshotResult, PalletPortfolioMovePortfolioItem, PalletRewardsItnRewardStatus, PalletSettlementLeg, PalletSettlementReceiptDetails, PalletSettlementSettlementType, PalletSettlementVenueType, PalletStakingCompactAssignments, PalletStakingElectionSize, PalletStakingRewardDestination, PalletStakingSlashingSwitch, PalletStakingValidatorPrefs, PalletStoPriceTier, PalletUtilityUniqueCall, PolymeshCommonUtilitiesBalancesMemo, PolymeshCommonUtilitiesIdentitySecondaryKeyWithAuth, PolymeshCommonUtilitiesIdentitySecondaryKeyWithAuthV1, PolymeshCommonUtilitiesMaybeBlock, PolymeshCommonUtilitiesProtocolFeeProtocolOp, PolymeshPrimitivesAgentAgentGroup, PolymeshPrimitivesAssetAssetType, PolymeshPrimitivesAssetIdentifier, PolymeshPrimitivesAssetMetadataAssetMetadataKey, PolymeshPrimitivesAssetMetadataAssetMetadataSpec, PolymeshPrimitivesAssetMetadataAssetMetadataValueDetail, PolymeshPrimitivesAuthorizationAuthorizationData, PolymeshPrimitivesBeneficiary, PolymeshPrimitivesCddIdInvestorUid, PolymeshPrimitivesComplianceManagerComplianceRequirement, PolymeshPrimitivesCondition, PolymeshPrimitivesConditionTrustedIssuer, PolymeshPrimitivesDocument, PolymeshPrimitivesEthereumEcdsaSignature, PolymeshPrimitivesIdentityClaimClaim, PolymeshPrimitivesIdentityClaimClaimType, PolymeshPrimitivesIdentityClaimScope, PolymeshPrimitivesIdentityId, PolymeshPrimitivesIdentityIdPortfolioId, PolymeshPrimitivesInvestorZkproofDataV1InvestorZKProofData, PolymeshPrimitivesPosRatio, PolymeshPrimitivesSecondaryKey, PolymeshPrimitivesSecondaryKeyPermissions, PolymeshPrimitivesSecondaryKeySignatory, PolymeshPrimitivesStatisticsAssetScope, PolymeshPrimitivesStatisticsStatType, PolymeshPrimitivesSubsetSubsetRestrictionPalletPermissions, PolymeshPrimitivesTicker, PolymeshPrimitivesTransferComplianceTransferConditionExemptKey, PolymeshRuntimeDevelopRuntimeSessionKeys, SpConsensusBabeDigestsNextConfigDescriptor, SpConsensusSlotsEquivocationProof, SpFinalityGrandpaEquivocationProof, SpNposElectionsElectionScore, SpRuntimeHeader, SpRuntimeMultiSignature, SpSessionMembershipProof } from '@polkadot/types/lookup';

declare module '@polkadot/api-base/types/submittable' {
  export interface AugmentedSubmittables<ApiType extends ApiTypes> {
    asset: {
      /**
       * This function is used to accept a token ownership transfer.
       * NB: To reject the transfer, call remove auth function in identity module.
       * 
       * # Arguments
       * * `origin` It contains the secondary key of the caller (i.e. who signed the transaction to execute this function).
       * * `auth_id` Authorization ID of the token ownership transfer authorization.
       **/
      acceptAssetOwnershipTransfer: AugmentedSubmittable<(authId: u64 | AnyNumber | Uint8Array) => SubmittableExtrinsic<ApiType>, [u64]>;
      /**
       * Accepts a ticker transfer.
       * 
       * Consumes the authorization `auth_id` (see `pallet_identity::consume_auth`).
       * NB: To reject the transfer, call remove auth function in identity module.
       * 
       * # Arguments
       * * `origin` It contains the secondary key of the caller (i.e. who signed the transaction to execute this function).
       * * `auth_id` Authorization ID of ticker transfer authorization.
       * 
       * ## Errors
       * - `AuthorizationError::BadType` if `auth_id` is not a valid ticket transfer authorization.
       * 
       **/
      acceptTickerTransfer: AugmentedSubmittable<(authId: u64 | AnyNumber | Uint8Array) => SubmittableExtrinsic<ApiType>, [u64]>;
      /**
       * Add documents for a given token.
       * 
       * # Arguments
       * * `origin` is a signer that has permissions to act as an agent of `ticker`.
       * * `ticker` Ticker of the token.
       * * `docs` Documents to be attached to `ticker`.
       * 
       * # Permissions
       * * Asset
       **/
      addDocuments: AugmentedSubmittable<(docs: Vec<PolymeshPrimitivesDocument> | (PolymeshPrimitivesDocument | { uri?: any; contentHash?: any; name?: any; docType?: any; filingDate?: any } | string | Uint8Array)[], ticker: PolymeshPrimitivesTicker | string | Uint8Array) => SubmittableExtrinsic<ApiType>, [Vec<PolymeshPrimitivesDocument>, PolymeshPrimitivesTicker]>;
      /**
       * Claim a systematically reserved Polymath Classic (PMC) `ticker`
       * and transfer it to the `origin`'s identity.
       * 
       * To verify that the `origin` is in control of the Ethereum account on the books,
       * an `ethereum_signature` containing the `origin`'s DID as the message
       * must be provided by that Ethereum account.
       * 
       * # Errors
       * - `NoSuchClassicTicker` if this is not a systematically reserved PMC ticker.
       * - `TickerAlreadyRegistered` if the ticker was already registered, e.g., by `origin`.
       * - `TickerRegistrationExpired` if the ticker's registration has expired.
       * - `BadOrigin` if not signed.
       * - `InvalidEthereumSignature` if the `ethereum_signature` is not valid.
       * - `NotAnOwner` if the ethereum account is not the owner of the PMC ticker.
       **/
      claimClassicTicker: AugmentedSubmittable<(ticker: PolymeshPrimitivesTicker | string | Uint8Array, ethereumSignature: PolymeshPrimitivesEthereumEcdsaSignature | string | Uint8Array) => SubmittableExtrinsic<ApiType>, [PolymeshPrimitivesTicker, PolymeshPrimitivesEthereumEcdsaSignature]>;
      /**
       * Forces a transfer of token from `from_portfolio` to the caller's default portfolio.
       * 
       * # Arguments
       * * `origin` Must be an external agent with appropriate permissions for a given ticker.
       * * `ticker` Ticker symbol of the asset.
       * * `value`  Amount of tokens need to force transfer.
       * * `from_portfolio` From whom portfolio tokens gets transferred.
       **/
      controllerTransfer: AugmentedSubmittable<(ticker: PolymeshPrimitivesTicker | string | Uint8Array, value: u128 | AnyNumber | Uint8Array, fromPortfolio: PolymeshPrimitivesIdentityIdPortfolioId | { did?: any; kind?: any } | string | Uint8Array) => SubmittableExtrinsic<ApiType>, [PolymeshPrimitivesTicker, u128, PolymeshPrimitivesIdentityIdPortfolioId]>;
      /**
       * Initializes a new security token, with the initiating account as its owner.
       * The total supply will initially be zero. To mint tokens, use `issue`.
       * 
       * # Arguments
       * * `origin` - contains the secondary key of the caller (i.e. who signed the transaction to execute this function).
       * * `name` - the name of the token.
       * * `ticker` - the ticker symbol of the token.
       * * `divisible` - a boolean to identify the divisibility status of the token.
       * * `asset_type` - the asset type.
       * * `identifiers` - a vector of asset identifiers.
       * * `funding_round` - name of the funding round.
       * * `disable_iu` - whether or not investor uniqueness enforcement should be disabled.
       * This cannot be changed after creating the asset.
       * 
       * ## Errors
       * - `InvalidAssetIdentifier` if any of `identifiers` are invalid.
       * - `MaxLengthOfAssetNameExceeded` if `name`'s length exceeds `T::AssetNameMaxLength`.
       * - `FundingRoundNameMaxLengthExceeded` if the name of the funding round is longer that
       * `T::FundingRoundNameMaxLength`.
       * - `AssetAlreadyCreated` if asset was already created.
       * - `TickerTooLong` if `ticker`'s length is greater than `config.max_ticker_length` chain
       * parameter.
       * - `TickerNotAscii` if `ticker` is not yet registered, and contains non-ascii printable characters (from code 32 to 126) or any character after first occurrence of `\0`.
       * 
       * ## Permissions
       * * Portfolio
       **/
      createAsset: AugmentedSubmittable<(name: Bytes | string | Uint8Array, ticker: PolymeshPrimitivesTicker | string | Uint8Array, divisible: bool | boolean | Uint8Array, assetType: PolymeshPrimitivesAssetAssetType | { EquityCommon: any } | { EquityPreferred: any } | { Commodity: any } | { FixedIncome: any } | { REIT: any } | { Fund: any } | { RevenueShareAgreement: any } | { StructuredProduct: any } | { Derivative: any } | { Custom: any } | { StableCoin: any } | string | Uint8Array, identifiers: Vec<PolymeshPrimitivesAssetIdentifier> | (PolymeshPrimitivesAssetIdentifier | { CUSIP: any } | { CINS: any } | { ISIN: any } | { LEI: any } | { FIGI: any } | string | Uint8Array)[], fundingRound: Option<Bytes> | null | object | string | Uint8Array, disableIu: bool | boolean | Uint8Array) => SubmittableExtrinsic<ApiType>, [Bytes, PolymeshPrimitivesTicker, bool, PolymeshPrimitivesAssetAssetType, Vec<PolymeshPrimitivesAssetIdentifier>, Option<Bytes>, bool]>;
      /**
       * Utility extrinsic to batch `create_asset` and `register_custom_asset_type`.
       **/
      createAssetWithCustomType: AugmentedSubmittable<(name: Bytes | string | Uint8Array, ticker: PolymeshPrimitivesTicker | string | Uint8Array, divisible: bool | boolean | Uint8Array, customAssetType: Bytes | string | Uint8Array, identifiers: Vec<PolymeshPrimitivesAssetIdentifier> | (PolymeshPrimitivesAssetIdentifier | { CUSIP: any } | { CINS: any } | { ISIN: any } | { LEI: any } | { FIGI: any } | string | Uint8Array)[], fundingRound: Option<Bytes> | null | object | string | Uint8Array, disableIu: bool | boolean | Uint8Array) => SubmittableExtrinsic<ApiType>, [Bytes, PolymeshPrimitivesTicker, bool, Bytes, Vec<PolymeshPrimitivesAssetIdentifier>, Option<Bytes>, bool]>;
      /**
       * Freezes transfers and minting of a given token.
       * 
       * # Arguments
       * * `origin` - the secondary key of the sender.
       * * `ticker` - the ticker of the token.
       * 
       * ## Errors
       * - `AlreadyFrozen` if `ticker` is already frozen.
       * 
       * # Permissions
       * * Asset
       **/
      freeze: AugmentedSubmittable<(ticker: PolymeshPrimitivesTicker | string | Uint8Array) => SubmittableExtrinsic<ApiType>, [PolymeshPrimitivesTicker]>;
      /**
       * Issue, or mint, new tokens to the caller,
       * which must be an authorized external agent.
       * 
       * # Arguments
       * * `origin` is a signer that has permissions to act as an agent of `ticker`.
       * * `ticker` of the token.
       * * `amount` of tokens that get issued.
       * 
       * # Permissions
       * * Asset
       * * Portfolio
       **/
      issue: AugmentedSubmittable<(ticker: PolymeshPrimitivesTicker | string | Uint8Array, amount: u128 | AnyNumber | Uint8Array) => SubmittableExtrinsic<ApiType>, [PolymeshPrimitivesTicker, u128]>;
      /**
       * Makes an indivisible token divisible.
       * 
       * # Arguments
       * * `origin` is a signer that has permissions to act as an agent of `ticker`.
       * * `ticker` Ticker of the token.
       * 
       * ## Errors
       * - `AssetAlreadyDivisible` if `ticker` is already divisible.
       * 
       * # Permissions
       * * Asset
       **/
      makeDivisible: AugmentedSubmittable<(ticker: PolymeshPrimitivesTicker | string | Uint8Array) => SubmittableExtrinsic<ApiType>, [PolymeshPrimitivesTicker]>;
      /**
       * Redeems existing tokens by reducing the balance of the caller's default portfolio and the total supply of the token
       * 
       * # Arguments
       * * `origin` is a signer that has permissions to act as an agent of `ticker`.
       * * `ticker` Ticker of the token.
       * * `value` Amount of tokens to redeem.
       * 
       * # Errors
       * - `Unauthorized` If called by someone without the appropriate external agent permissions
       * - `InvalidGranularity` If the amount is not divisible by 10^6 for non-divisible tokens
       * - `InsufficientPortfolioBalance` If the caller's default portfolio doesn't have enough free balance
       * 
       * # Permissions
       * * Asset
       * * Portfolio
       **/
      redeem: AugmentedSubmittable<(ticker: PolymeshPrimitivesTicker | string | Uint8Array, value: u128 | AnyNumber | Uint8Array) => SubmittableExtrinsic<ApiType>, [PolymeshPrimitivesTicker, u128]>;
      /**
       * Registers and set local asset metadata.
       * 
       * # Arguments
       * * `origin` is a signer that has permissions to act as an agent of `ticker`.
       * * `ticker` Ticker of the token.
       * * `name` Metadata name.
       * * `spec` Metadata type definition.
       * * `value` Metadata value.
       * * `details` Optional Metadata value details (expire, lock status).
       * 
       * # Errors
       * * `AssetMetadataLocalKeyAlreadyExists` if a local metadata type with `name` already exists for `ticker`.
       * * `AssetMetadataNameMaxLengthExceeded` if the metadata `name` exceeds the maximum length.
       * * `AssetMetadataTypeDefMaxLengthExceeded` if the metadata `spec` type definition exceeds the maximum length.
       * * `AssetMetadataValueMaxLengthExceeded` if the metadata value exceeds the maximum length.
       * 
       * # Permissions
       * * Agent
       * * Asset
       **/
      registerAndSetLocalAssetMetadata: AugmentedSubmittable<(ticker: PolymeshPrimitivesTicker | string | Uint8Array, name: Bytes | string | Uint8Array, spec: PolymeshPrimitivesAssetMetadataAssetMetadataSpec | { url?: any; description?: any; typeDef?: any } | string | Uint8Array, value: Bytes | string | Uint8Array, detail: Option<PolymeshPrimitivesAssetMetadataAssetMetadataValueDetail> | null | object | string | Uint8Array) => SubmittableExtrinsic<ApiType>, [PolymeshPrimitivesTicker, Bytes, PolymeshPrimitivesAssetMetadataAssetMetadataSpec, Bytes, Option<PolymeshPrimitivesAssetMetadataAssetMetadataValueDetail>]>;
      /**
       * Registers asset metadata global type.
       * 
       * # Arguments
       * * `origin` is a signer that has permissions to act as an agent of `ticker`.
       * * `name` Metadata name.
       * * `spec` Metadata type definition.
       * 
       * # Errors
       * * `AssetMetadataGlobalKeyAlreadyExists` if a globa metadata type with `name` already exists.
       * * `AssetMetadataNameMaxLengthExceeded` if the metadata `name` exceeds the maximum length.
       * * `AssetMetadataTypeDefMaxLengthExceeded` if the metadata `spec` type definition exceeds the maximum length.
       **/
      registerAssetMetadataGlobalType: AugmentedSubmittable<(name: Bytes | string | Uint8Array, spec: PolymeshPrimitivesAssetMetadataAssetMetadataSpec | { url?: any; description?: any; typeDef?: any } | string | Uint8Array) => SubmittableExtrinsic<ApiType>, [Bytes, PolymeshPrimitivesAssetMetadataAssetMetadataSpec]>;
      /**
       * Registers asset metadata local type.
       * 
       * # Arguments
       * * `origin` is a signer that has permissions to act as an agent of `ticker`.
       * * `ticker` Ticker of the token.
       * * `name` Metadata name.
       * * `spec` Metadata type definition.
       * 
       * # Errors
       * * `AssetMetadataLocalKeyAlreadyExists` if a local metadata type with `name` already exists for `ticker`.
       * * `AssetMetadataNameMaxLengthExceeded` if the metadata `name` exceeds the maximum length.
       * * `AssetMetadataTypeDefMaxLengthExceeded` if the metadata `spec` type definition exceeds the maximum length.
       * 
       * # Permissions
       * * Agent
       * * Asset
       **/
      registerAssetMetadataLocalType: AugmentedSubmittable<(ticker: PolymeshPrimitivesTicker | string | Uint8Array, name: Bytes | string | Uint8Array, spec: PolymeshPrimitivesAssetMetadataAssetMetadataSpec | { url?: any; description?: any; typeDef?: any } | string | Uint8Array) => SubmittableExtrinsic<ApiType>, [PolymeshPrimitivesTicker, Bytes, PolymeshPrimitivesAssetMetadataAssetMetadataSpec]>;
      /**
       * Registers a custom asset type.
       * 
       * The provided `ty` will be bound to an ID in storage.
       * The ID can then be used in `AssetType::Custom`.
       * Should the `ty` already exist in storage, no second ID is assigned to it.
       * 
       * # Arguments
       * * `origin` who called the extrinsic.
       * * `ty` contains the string representation of the asset type.
       **/
      registerCustomAssetType: AugmentedSubmittable<(ty: Bytes | string | Uint8Array) => SubmittableExtrinsic<ApiType>, [Bytes]>;
      /**
       * Registers a new ticker or extends validity of an existing ticker.
       * NB: Ticker validity does not get carry forward when renewing ticker.
       * 
       * # Arguments
       * * `origin` It contains the secondary key of the caller (i.e. who signed the transaction to execute this function).
       * * `ticker` ticker to register.
       * 
       * # Permissions
       * * Asset
       **/
      registerTicker: AugmentedSubmittable<(ticker: PolymeshPrimitivesTicker | string | Uint8Array) => SubmittableExtrinsic<ApiType>, [PolymeshPrimitivesTicker]>;
      /**
       * Remove documents for a given token.
       * 
       * # Arguments
       * * `origin` is a signer that has permissions to act as an agent of `ticker`.
       * * `ticker` Ticker of the token.
       * * `ids` Documents ids to be removed from `ticker`.
       * 
       * # Permissions
       * * Asset
       **/
      removeDocuments: AugmentedSubmittable<(ids: Vec<u32> | (u32 | AnyNumber | Uint8Array)[], ticker: PolymeshPrimitivesTicker | string | Uint8Array) => SubmittableExtrinsic<ApiType>, [Vec<u32>, PolymeshPrimitivesTicker]>;
      /**
       * Renames a given token.
       * 
       * # Arguments
       * * `origin` - the secondary key of the sender.
       * * `ticker` - the ticker of the token.
       * * `name` - the new name of the token.
       * 
       * ## Errors
       * - `MaxLengthOfAssetNameExceeded` if length of `name` is greater than
       * `T::AssetNameMaxLength`.
       * 
       * # Permissions
       * * Asset
       **/
      renameAsset: AugmentedSubmittable<(ticker: PolymeshPrimitivesTicker | string | Uint8Array, name: Bytes | string | Uint8Array) => SubmittableExtrinsic<ApiType>, [PolymeshPrimitivesTicker, Bytes]>;
      /**
       * Reserve a Polymath Classic (PMC) ticker.
       * Must be called by root, and assigns the ticker to a systematic DID.
       * 
       * # Arguments
       * * `origin` which must be root.
       * * `classic_ticker_import` specification for the PMC ticker.
       * * `contract_did` to reserve the ticker to if `classic_ticker_import.is_contract` holds.
       * * `config` to use for expiry and ticker length.
       * 
       * # Errors
       * * `AssetAlreadyCreated` if `classic_ticker_import.ticker` was created as an asset.
       * * `TickerTooLong` if the `config` considers the `classic_ticker_import.ticker` too long.
       * * `TickerAlreadyRegistered` if `classic_ticker_import.ticker` was already registered.
       **/
      reserveClassicTicker: AugmentedSubmittable<(classicTickerImport: PalletAssetClassicTickerImport | { ethOwner?: any; ticker?: any; isContract?: any; isCreated?: any } | string | Uint8Array, contractDid: PolymeshPrimitivesIdentityId | string | Uint8Array, config: PalletAssetTickerRegistrationConfig | { maxTickerLength?: any; registrationLength?: any } | string | Uint8Array) => SubmittableExtrinsic<ApiType>, [PalletAssetClassicTickerImport, PolymeshPrimitivesIdentityId, PalletAssetTickerRegistrationConfig]>;
      /**
       * Set asset metadata value.
       * 
       * # Arguments
       * * `origin` is a signer that has permissions to act as an agent of `ticker`.
       * * `ticker` Ticker of the token.
       * * `key` Metadata key.
       * * `value` Metadata value.
       * * `details` Optional Metadata value details (expire, lock status).
       * 
       * # Errors
       * * `AssetMetadataKeyIsMissing` if the metadata type key doesn't exist.
       * * `AssetMetadataValueIsLocked` if the metadata value for `key` is locked.
       * * `AssetMetadataValueMaxLengthExceeded` if the metadata value exceeds the maximum length.
       * 
       * # Permissions
       * * Agent
       * * Asset
       **/
      setAssetMetadata: AugmentedSubmittable<(ticker: PolymeshPrimitivesTicker | string | Uint8Array, key: PolymeshPrimitivesAssetMetadataAssetMetadataKey | { Global: any } | { Local: any } | string | Uint8Array, value: Bytes | string | Uint8Array, detail: Option<PolymeshPrimitivesAssetMetadataAssetMetadataValueDetail> | null | object | string | Uint8Array) => SubmittableExtrinsic<ApiType>, [PolymeshPrimitivesTicker, PolymeshPrimitivesAssetMetadataAssetMetadataKey, Bytes, Option<PolymeshPrimitivesAssetMetadataAssetMetadataValueDetail>]>;
      /**
       * Set asset metadata value details (expire, lock status).
       * 
       * # Arguments
       * * `origin` is a signer that has permissions to act as an agent of `ticker`.
       * * `ticker` Ticker of the token.
       * * `key` Metadata key.
       * * `details` Metadata value details (expire, lock status).
       * 
       * # Errors
       * * `AssetMetadataKeyIsMissing` if the metadata type key doesn't exist.
       * * `AssetMetadataValueIsLocked` if the metadata value for `key` is locked.
       * 
       * # Permissions
       * * Agent
       * * Asset
       **/
      setAssetMetadataDetails: AugmentedSubmittable<(ticker: PolymeshPrimitivesTicker | string | Uint8Array, key: PolymeshPrimitivesAssetMetadataAssetMetadataKey | { Global: any } | { Local: any } | string | Uint8Array, detail: PolymeshPrimitivesAssetMetadataAssetMetadataValueDetail | { expire?: any; lockStatus?: any } | string | Uint8Array) => SubmittableExtrinsic<ApiType>, [PolymeshPrimitivesTicker, PolymeshPrimitivesAssetMetadataAssetMetadataKey, PolymeshPrimitivesAssetMetadataAssetMetadataValueDetail]>;
      /**
       * Sets the name of the current funding round.
       * 
       * # Arguments
       * * `origin` - a signer that has permissions to act as an agent of `ticker`.
       * * `ticker` - the ticker of the token.
       * * `name` - the desired name of the current funding round.
       * 
       * ## Errors
       * - `FundingRoundNameMaxLengthExceeded` if length of `name` is greater than
       * `T::FundingRoundNameMaxLength`.
       * 
       * # Permissions
       * * Asset
       **/
      setFundingRound: AugmentedSubmittable<(ticker: PolymeshPrimitivesTicker | string | Uint8Array, name: Bytes | string | Uint8Array) => SubmittableExtrinsic<ApiType>, [PolymeshPrimitivesTicker, Bytes]>;
      /**
       * Unfreezes transfers and minting of a given token.
       * 
       * # Arguments
       * * `origin` - the secondary key of the sender.
       * * `ticker` - the ticker of the frozen token.
       * 
       * ## Errors
       * - `NotFrozen` if `ticker` is not frozen yet.
       * 
       * # Permissions
       * * Asset
       **/
      unfreeze: AugmentedSubmittable<(ticker: PolymeshPrimitivesTicker | string | Uint8Array) => SubmittableExtrinsic<ApiType>, [PolymeshPrimitivesTicker]>;
      /**
       * Updates the asset identifiers.
       * 
       * # Arguments
       * * `origin` - a signer that has permissions to act as an agent of `ticker`.
       * * `ticker` - the ticker of the token.
       * * `identifiers` - the asset identifiers to be updated in the form of a vector of pairs
       * of `IdentifierType` and `AssetIdentifier` value.
       * 
       * ## Errors
       * - `InvalidAssetIdentifier` if `identifiers` contains any invalid identifier.
       * 
       * # Permissions
       * * Asset
       **/
      updateIdentifiers: AugmentedSubmittable<(ticker: PolymeshPrimitivesTicker | string | Uint8Array, identifiers: Vec<PolymeshPrimitivesAssetIdentifier> | (PolymeshPrimitivesAssetIdentifier | { CUSIP: any } | { CINS: any } | { ISIN: any } | { LEI: any } | { FIGI: any } | string | Uint8Array)[]) => SubmittableExtrinsic<ApiType>, [PolymeshPrimitivesTicker, Vec<PolymeshPrimitivesAssetIdentifier>]>;
      /**
       * Generic tx
       **/
      [key: string]: SubmittableExtrinsicFunction<ApiType>;
    };
    authorship: {
      /**
       * Provide a set of uncles.
       **/
      setUncles: AugmentedSubmittable<(newUncles: Vec<SpRuntimeHeader> | (SpRuntimeHeader | { parentHash?: any; number?: any; stateRoot?: any; extrinsicsRoot?: any; digest?: any } | string | Uint8Array)[]) => SubmittableExtrinsic<ApiType>, [Vec<SpRuntimeHeader>]>;
      /**
       * Generic tx
       **/
      [key: string]: SubmittableExtrinsicFunction<ApiType>;
    };
    babe: {
      /**
       * Plan an epoch config change. The epoch config change is recorded and will be enacted on
       * the next call to `enact_epoch_change`. The config will be activated one epoch after.
       * Multiple calls to this method will replace any existing planned config change that had
       * not been enacted yet.
       **/
      planConfigChange: AugmentedSubmittable<(config: SpConsensusBabeDigestsNextConfigDescriptor | { V1: any } | string | Uint8Array) => SubmittableExtrinsic<ApiType>, [SpConsensusBabeDigestsNextConfigDescriptor]>;
      /**
       * Report authority equivocation/misbehavior. This method will verify
       * the equivocation proof and validate the given key ownership proof
       * against the extracted offender. If both are valid, the offence will
       * be reported.
       **/
      reportEquivocation: AugmentedSubmittable<(equivocationProof: SpConsensusSlotsEquivocationProof | { offender?: any; slot?: any; firstHeader?: any; secondHeader?: any } | string | Uint8Array, keyOwnerProof: SpSessionMembershipProof | { session?: any; trieNodes?: any; validatorCount?: any } | string | Uint8Array) => SubmittableExtrinsic<ApiType>, [SpConsensusSlotsEquivocationProof, SpSessionMembershipProof]>;
      /**
       * Report authority equivocation/misbehavior. This method will verify
       * the equivocation proof and validate the given key ownership proof
       * against the extracted offender. If both are valid, the offence will
       * be reported.
       * This extrinsic must be called unsigned and it is expected that only
       * block authors will call it (validated in `ValidateUnsigned`), as such
       * if the block author is defined it will be defined as the equivocation
       * reporter.
       **/
      reportEquivocationUnsigned: AugmentedSubmittable<(equivocationProof: SpConsensusSlotsEquivocationProof | { offender?: any; slot?: any; firstHeader?: any; secondHeader?: any } | string | Uint8Array, keyOwnerProof: SpSessionMembershipProof | { session?: any; trieNodes?: any; validatorCount?: any } | string | Uint8Array) => SubmittableExtrinsic<ApiType>, [SpConsensusSlotsEquivocationProof, SpSessionMembershipProof]>;
      /**
       * Generic tx
       **/
      [key: string]: SubmittableExtrinsicFunction<ApiType>;
    };
    balances: {
      /**
       * Burns the given amount of tokens from the caller's free, unlocked balance.
       **/
      burnAccountBalance: AugmentedSubmittable<(amount: u128 | AnyNumber | Uint8Array) => SubmittableExtrinsic<ApiType>, [u128]>;
      /**
       * Move some POLYX from balance of self to balance of BRR.
       **/
      depositBlockRewardReserveBalance: AugmentedSubmittable<(value: Compact<u128> | AnyNumber | Uint8Array) => SubmittableExtrinsic<ApiType>, [Compact<u128>]>;
      /**
       * Exactly as `transfer`, except the origin must be root and the source account may be
       * specified.
       * 
       * # <weight>
       * - Same as transfer, but additional read and write because the source account is
       * not assumed to be in the overlay.
       * # </weight>
       **/
      forceTransfer: AugmentedSubmittable<(source: MultiAddress | { Id: any } | { Index: any } | { Raw: any } | { Address32: any } | { Address20: any } | string | Uint8Array, dest: MultiAddress | { Id: any } | { Index: any } | { Raw: any } | { Address32: any } | { Address20: any } | string | Uint8Array, value: Compact<u128> | AnyNumber | Uint8Array) => SubmittableExtrinsic<ApiType>, [MultiAddress, MultiAddress, Compact<u128>]>;
      /**
       * Set the balances of a given account.
       * 
       * This will alter `FreeBalance` and `ReservedBalance` in storage. it will
       * also decrease the total issuance of the system (`TotalIssuance`).
       * 
       * The dispatch origin for this call is `root`.
       **/
      setBalance: AugmentedSubmittable<(who: MultiAddress | { Id: any } | { Index: any } | { Raw: any } | { Address32: any } | { Address20: any } | string | Uint8Array, newFree: Compact<u128> | AnyNumber | Uint8Array, newReserved: Compact<u128> | AnyNumber | Uint8Array) => SubmittableExtrinsic<ApiType>, [MultiAddress, Compact<u128>, Compact<u128>]>;
      /**
       * Transfer some liquid free balance to another account.
       * 
       * `transfer` will set the `FreeBalance` of the sender and receiver.
       * It will decrease the total issuance of the system by the `TransferFee`.
       * 
       * The dispatch origin for this call must be `Signed` by the transactor.
       * 
       * # <weight>
       * - Dependent on arguments but not critical, given proper implementations for
       * input config types. See related functions below.
       * - It contains a limited number of reads and writes internally and no complex computation.
       * 
       * Related functions:
       * 
       * - `ensure_can_withdraw` is always called internally but has a bounded complexity.
       * - Transferring balances to accounts that did not exist before will cause
       * `T::OnNewAccount::on_new_account` to be called.
       * ---------------------------------
       * - Base Weight: 73.64 µs, worst case scenario (account created, account removed)
       * - DB Weight: 1 Read and 1 Write to destination account.
       * - Origin account is already in memory, so no DB operations for them.
       * # </weight>
       **/
      transfer: AugmentedSubmittable<(dest: MultiAddress | { Id: any } | { Index: any } | { Raw: any } | { Address32: any } | { Address20: any } | string | Uint8Array, value: Compact<u128> | AnyNumber | Uint8Array) => SubmittableExtrinsic<ApiType>, [MultiAddress, Compact<u128>]>;
      /**
       * Transfer the native currency with the help of identifier string
       * this functionality can help to differentiate the transfers.
       * 
       * # <weight>
       * - Base Weight: 73.64 µs, worst case scenario (account created, account removed)
       * - DB Weight: 1 Read and 1 Write to destination account.
       * - Origin account is already in memory, so no DB operations for them.
       * # </weight>
       **/
      transferWithMemo: AugmentedSubmittable<(dest: MultiAddress | { Id: any } | { Index: any } | { Raw: any } | { Address32: any } | { Address20: any } | string | Uint8Array, value: Compact<u128> | AnyNumber | Uint8Array, memo: Option<PolymeshCommonUtilitiesBalancesMemo> | null | object | string | Uint8Array) => SubmittableExtrinsic<ApiType>, [MultiAddress, Compact<u128>, Option<PolymeshCommonUtilitiesBalancesMemo>]>;
      /**
       * Generic tx
       **/
      [key: string]: SubmittableExtrinsicFunction<ApiType>;
    };
    base: {
      /**
       * Generic tx
       **/
      [key: string]: SubmittableExtrinsicFunction<ApiType>;
    };
    bridge: {
      /**
       * Add a freeze admin.
       * 
       * ## Errors
       * - `BadAdmin` if `origin` is not `Self::admin()` account.
       **/
      addFreezeAdmin: AugmentedSubmittable<(freezeAdmin: AccountId32 | string | Uint8Array) => SubmittableExtrinsic<ApiType>, [AccountId32]>;
      /**
       * Proposes a vector of bridge transactions. The vector is processed until the first
       * proposal which causes an error, in which case the error is returned and the rest of
       * proposals are not processed.
       * 
       * ## Errors
       * - `ControllerNotSet` if `Controllers` was not set.
       * 
       * # Weight
       * `500_000_000 + 7_000_000 * bridge_txs.len()`
       **/
      batchProposeBridgeTx: AugmentedSubmittable<(bridgeTxs: Vec<PalletBridgeBridgeTx> | (PalletBridgeBridgeTx | { nonce?: any; recipient?: any; amount?: any; txHash?: any } | string | Uint8Array)[]) => SubmittableExtrinsic<ApiType>, [Vec<PalletBridgeBridgeTx>]>;
      /**
       * Changes the bridge admin key.
       * 
       * ## Errors
       * - `BadAdmin` if `origin` is not `Self::admin()` account.
       **/
      changeAdmin: AugmentedSubmittable<(admin: AccountId32 | string | Uint8Array) => SubmittableExtrinsic<ApiType>, [AccountId32]>;
      /**
       * Changes the bridge limit exempted list.
       * 
       * ## Errors
       * - `BadAdmin` if `origin` is not `Self::admin()` account.
       **/
      changeBridgeExempted: AugmentedSubmittable<(exempted: Vec<ITuple<[PolymeshPrimitivesIdentityId, bool]>> | ([PolymeshPrimitivesIdentityId | string | Uint8Array, bool | boolean | Uint8Array])[]) => SubmittableExtrinsic<ApiType>, [Vec<ITuple<[PolymeshPrimitivesIdentityId, bool]>>]>;
      /**
       * Changes the bridge limits.
       * 
       * ## Errors
       * - `BadAdmin` if `origin` is not `Self::admin()` account.
       * - `DivisionByZero` if `duration` is zero.
       **/
      changeBridgeLimit: AugmentedSubmittable<(amount: u128 | AnyNumber | Uint8Array, duration: u32 | AnyNumber | Uint8Array) => SubmittableExtrinsic<ApiType>, [u128, u32]>;
      /**
       * Changes the controller account as admin.
       * 
       * ## Errors
       * - `BadAdmin` if `origin` is not `Self::admin()` account.
       **/
      changeController: AugmentedSubmittable<(controller: AccountId32 | string | Uint8Array) => SubmittableExtrinsic<ApiType>, [AccountId32]>;
      /**
       * Changes the timelock period.
       * 
       * ## Errors
       * - `BadAdmin` if `origin` is not `Self::admin()` account.
       **/
      changeTimelock: AugmentedSubmittable<(timelock: u32 | AnyNumber | Uint8Array) => SubmittableExtrinsic<ApiType>, [u32]>;
      /**
       * Forces handling a transaction by bypassing the bridge limit and timelock.
       * 
       * ## Errors
       * - `BadAdmin` if `origin` is not `Self::admin()` account.
       * - `NoValidCdd` if `bridge_tx.recipient` does not have a valid CDD claim.
       **/
      forceHandleBridgeTx: AugmentedSubmittable<(bridgeTx: PalletBridgeBridgeTx | { nonce?: any; recipient?: any; amount?: any; txHash?: any } | string | Uint8Array) => SubmittableExtrinsic<ApiType>, [PalletBridgeBridgeTx]>;
      /**
       * Freezes transaction handling in the bridge module if it is not already frozen. When the
       * bridge is frozen, attempted transactions get postponed instead of getting handled.
       * 
       * ## Errors
       * - `BadAdmin` if `origin` is not `Self::admin()` account.
       **/
      freeze: AugmentedSubmittable<() => SubmittableExtrinsic<ApiType>, []>;
      /**
       * Freezes given bridge transactions.
       * If any bridge txn is already handled then this function will just ignore it and process next one.
       * 
       * ## Errors
       * - `BadAdmin` if `origin` is not `Self::admin()` account.
       * 
       * # Weight
       * `400_000_000 + 2_000_000 * bridge_txs.len()`
       **/
      freezeTxs: AugmentedSubmittable<(bridgeTxs: Vec<PalletBridgeBridgeTx> | (PalletBridgeBridgeTx | { nonce?: any; recipient?: any; amount?: any; txHash?: any } | string | Uint8Array)[]) => SubmittableExtrinsic<ApiType>, [Vec<PalletBridgeBridgeTx>]>;
      /**
       * Handles an approved bridge transaction proposal.
       * 
       * ## Errors
       * - `BadCaller` if `origin` is not `Self::controller` or  `Self::admin`.
       * - `TimelockedTx` if the transaction status is `Timelocked`.
       * - `ProposalAlreadyHandled` if the transaction status is `Handled`.
       **/
      handleBridgeTx: AugmentedSubmittable<(bridgeTx: PalletBridgeBridgeTx | { nonce?: any; recipient?: any; amount?: any; txHash?: any } | string | Uint8Array) => SubmittableExtrinsic<ApiType>, [PalletBridgeBridgeTx]>;
      /**
       * Root callable extrinsic, used as an internal call to handle a scheduled timelocked bridge transaction.
       * 
       * # Errors
       * - `BadOrigin` if `origin` is not root.
       * - `ProposalAlreadyHandled` if transaction status is `Handled`.
       * - `FrozenTx` if transaction status is `Frozen`.
       **/
      handleScheduledBridgeTx: AugmentedSubmittable<(bridgeTx: PalletBridgeBridgeTx | { nonce?: any; recipient?: any; amount?: any; txHash?: any } | string | Uint8Array) => SubmittableExtrinsic<ApiType>, [PalletBridgeBridgeTx]>;
      /**
       * Proposes a bridge transaction, which amounts to making a multisig proposal for the
       * bridge transaction if the transaction is new or approving an existing proposal if the
       * transaction has already been proposed.
       * 
       * ## Errors
       * - `ControllerNotSet` if `Controllers` was not set.
       **/
      proposeBridgeTx: AugmentedSubmittable<(bridgeTx: PalletBridgeBridgeTx | { nonce?: any; recipient?: any; amount?: any; txHash?: any } | string | Uint8Array) => SubmittableExtrinsic<ApiType>, [PalletBridgeBridgeTx]>;
      /**
       * Remove a freeze admin.
       * 
       * ## Errors
       * - `BadAdmin` if `origin` is not `Self::admin()` account.
       **/
      removeFreezeAdmin: AugmentedSubmittable<(freezeAdmin: AccountId32 | string | Uint8Array) => SubmittableExtrinsic<ApiType>, [AccountId32]>;
      /**
       * Remove given bridge transactions.
       * 
       * ## Errors
       * - `BadAdmin` if `origin` is not `Self::admin()` account.
       * - `NotFrozen` if a tx in `bridge_txs` is not frozen.
       **/
      removeTxs: AugmentedSubmittable<(bridgeTxs: Vec<PalletBridgeBridgeTx> | (PalletBridgeBridgeTx | { nonce?: any; recipient?: any; amount?: any; txHash?: any } | string | Uint8Array)[]) => SubmittableExtrinsic<ApiType>, [Vec<PalletBridgeBridgeTx>]>;
      /**
       * Unfreezes transaction handling in the bridge module if it is frozen.
       * 
       * ## Errors
       * - `BadAdmin` if `origin` is not `Self::admin()` account.
       **/
      unfreeze: AugmentedSubmittable<() => SubmittableExtrinsic<ApiType>, []>;
      /**
       * Unfreezes given bridge transactions.
       * If any bridge txn is already handled then this function will just ignore it and process next one.
       * 
       * ## Errors
       * - `BadAdmin` if `origin` is not `Self::admin()` account.
       * 
       * # Weight
       * `400_000_000 + 7_000_000 * bridge_txs.len()`
       **/
      unfreezeTxs: AugmentedSubmittable<(bridgeTxs: Vec<PalletBridgeBridgeTx> | (PalletBridgeBridgeTx | { nonce?: any; recipient?: any; amount?: any; txHash?: any } | string | Uint8Array)[]) => SubmittableExtrinsic<ApiType>, [Vec<PalletBridgeBridgeTx>]>;
      /**
       * Generic tx
       **/
      [key: string]: SubmittableExtrinsicFunction<ApiType>;
    };
    capitalDistribution: {
      /**
       * Claim a benefit of the capital distribution attached to `ca_id`.
       * 
       * Taxes are withheld as specified by the CA.
       * Post-tax earnings are then transferred to the default portfolio of the `origin`'s DID.
       * 
       * All benefits are rounded by truncation, down to first integer below.
       * Moreover, before post-tax earnings, in indivisible currencies are transferred,
       * they are rounded down to a whole unit.
       * 
       * ## Arguments
       * - `origin` which must be a holder of the asset and eligible for the distribution.
       * - `ca_id` identifies the CA to start a capital distribution for.
       * 
       * # Errors
       * - `HolderAlreadyPaid` if `origin`'s DID has already received its benefit.
       * - `NoSuchDistribution` if there's no capital distribution for `ca_id`.
       * - `CannotClaimBeforeStart` if `now < payment_at`.
       * - `CannotClaimAfterExpiry` if `now > expiry_at.unwrap()`.
       * - `NoSuchCA` if `ca_id` does not identify an existing CA.
       * - `NotTargetedByCA` if the CA does not target `origin`'s DID.
       * - `BalanceAmountProductOverflowed` if `ba = balance * amount` would overflow.
       * - `BalanceAmountProductSupplyDivisionFailed` if `ba * supply` would overflow.
       * - Other errors can occur if the compliance manager rejects the transfer.
       **/
      claim: AugmentedSubmittable<(caId: PalletCorporateActionsCaId | { ticker?: any; localId?: any } | string | Uint8Array) => SubmittableExtrinsic<ApiType>, [PalletCorporateActionsCaId]>;
      /**
       * Start and attach a capital distribution, to the CA identified by `ca_id`,
       * with `amount` funds in `currency` withdrawn from `portfolio` belonging to `origin`'s DID.
       * 
       * The distribution will commence at `payment_at` and expire at `expires_at`,
       * if provided, or if `None`, then there's no expiry.
       * 
       * The funds will be locked in `portfolio` from when `distribute` is called.
       * When there's no expiry, some funds may be locked indefinitely in `portfolio`,
       * due to claimants not withdrawing or no benefits being pushed to them.
       * For indivisible currencies, unlocked amounts, of less than one whole unit,
       * will not be transferable from `portfolio`.
       * However, if we imagine that users `Alice` and `Bob` both are entitled to 1.5 units,
       * and only receive `1` units each, then `0.5 + 0.5 = 1` units are left in `portfolio`,
       * which is now transferrable.
       * 
       * ## Arguments
       * - `origin` is a signer that has permissions to act as an agent of `ca_id.ticker`.
       * - `ca_id` identifies the CA to start a capital distribution for.
       * - `portfolio` specifies the portfolio number of the agent to distribute `amount` from.
       * - `currency` to withdraw and distribute from the `portfolio`.
       * - `per_share` amount of `currency` to withdraw and distribute.
       * Specified as a per-million, i.e. `1 / 10^6`th of one `currency` token.
       * - `amount` of `currency` to withdraw and distribute at most.
       * - `payment_at` specifies when benefits may first be pushed or claimed.
       * - `expires_at` specifies, if provided, when remaining benefits are forfeit
       * and may be reclaimed by `origin`.
       * 
       * # Errors
       * - `UnauthorizedAgent` if `origin` is not agent-permissioned for `ticker`.
       * - `ExpiryBeforePayment` if `expires_at.unwrap() <= payment_at`.
       * - `NoSuchCA` if `ca_id` does not identify an existing CA.
       * - `NoRecordDate` if CA has no record date.
       * - `RecordDateAfterStart` if CA's record date > payment_at.
       * - `UnauthorizedCustodian` if the caller is not the custodian of `portfolio`.
       * - `InsufficientPortfolioBalance` if `portfolio` has less than `amount` of `currency`.
       * - `InsufficientBalance` if the protocol fee couldn't be charged.
       * - `CANotBenefit` if the CA is not of kind PredictableBenefit/UnpredictableBenefit
       * - `DistributionAmountIsZero` if the `amount` is zero.
       * - `DistributionPerShareIsZero` if the `per_share` is zero.
       * 
       * # Permissions
       * * Asset
       * * Portfolio
       **/
      distribute: AugmentedSubmittable<(caId: PalletCorporateActionsCaId | { ticker?: any; localId?: any } | string | Uint8Array, portfolio: Option<u64> | null | object | string | Uint8Array, currency: PolymeshPrimitivesTicker | string | Uint8Array, perShare: u128 | AnyNumber | Uint8Array, amount: u128 | AnyNumber | Uint8Array, paymentAt: u64 | AnyNumber | Uint8Array, expiresAt: Option<u64> | null | object | string | Uint8Array) => SubmittableExtrinsic<ApiType>, [PalletCorporateActionsCaId, Option<u64>, PolymeshPrimitivesTicker, u128, u128, u64, Option<u64>]>;
      /**
       * Push benefit of an ongoing distribution to the given `holder`.
       * 
       * Taxes are withheld as specified by the CA.
       * Post-tax earnings are then transferred to the default portfolio of the `origin`'s DID.
       * 
       * All benefits are rounded by truncation, down to first integer below.
       * Moreover, before post-tax earnings, in indivisible currencies are transferred,
       * they are rounded down to a whole unit.
       * 
       * ## Arguments
       * - `origin` is a signer that has permissions to act as an agent of `ca_id.ticker`.
       * - `ca_id` identifies the CA with a capital distributions to push benefits for.
       * - `holder` to push benefits to.
       * 
       * # Errors
       * - `UnauthorizedAgent` if `origin` is not agent-permissioned for `ticker`.
       * - `NoSuchDistribution` if there's no capital distribution for `ca_id`.
       * - `CannotClaimBeforeStart` if `now < payment_at`.
       * - `CannotClaimAfterExpiry` if `now > expiry_at.unwrap()`.
       * - `NoSuchCA` if `ca_id` does not identify an existing CA.
       * - `NotTargetedByCA` if the CA does not target `holder`.
       * - `BalanceAmountProductOverflowed` if `ba = balance * amount` would overflow.
       * - `BalanceAmountProductSupplyDivisionFailed` if `ba * supply` would overflow.
       * - Other errors can occur if the compliance manager rejects the transfer.
       **/
      pushBenefit: AugmentedSubmittable<(caId: PalletCorporateActionsCaId | { ticker?: any; localId?: any } | string | Uint8Array, holder: PolymeshPrimitivesIdentityId | string | Uint8Array) => SubmittableExtrinsic<ApiType>, [PalletCorporateActionsCaId, PolymeshPrimitivesIdentityId]>;
      /**
       * Assuming a distribution has expired,
       * unlock the remaining amount in the distributor portfolio.
       * 
       * ## Arguments
       * - `origin` which must be the creator of the capital distribution tied to `ca_id`.
       * - `ca_id` identifies the CA with a capital distribution to reclaim for.
       * 
       * # Errors
       * - `NoSuchDistribution` if there's no capital distribution for `ca_id`.
       * - `AlreadyReclaimed` if this function has already been called successfully.
       * - `NotExpired` if `now < expiry`.
       **/
      reclaim: AugmentedSubmittable<(caId: PalletCorporateActionsCaId | { ticker?: any; localId?: any } | string | Uint8Array) => SubmittableExtrinsic<ApiType>, [PalletCorporateActionsCaId]>;
      /**
       * Removes a distribution that hasn't started yet,
       * unlocking the full amount in the distributor portfolio.
       * 
       * ## Arguments
       * - `origin` is a signer that has permissions to act as an agent of `ca_id.ticker`.
       * - `ca_id` identifies the CA with a not-yet-started capital distribution to remove.
       * 
       * # Errors
       * - `UnauthorizedAgent` if `origin` is not agent-permissioned for `ticker`.
       * - `NoSuchDistribution` if there's no capital distribution for `ca_id`.
       * - `DistributionStarted` if `payment_at <= now`.
       **/
      removeDistribution: AugmentedSubmittable<(caId: PalletCorporateActionsCaId | { ticker?: any; localId?: any } | string | Uint8Array) => SubmittableExtrinsic<ApiType>, [PalletCorporateActionsCaId]>;
      /**
       * Generic tx
       **/
      [key: string]: SubmittableExtrinsicFunction<ApiType>;
    };
    cddServiceProviders: {
      /**
       * Allows the calling member to *unilaterally quit* without this being subject to a GC
       * vote.
       * 
       * # Arguments
       * * `origin` - Member of committee who wants to quit.
       * 
       * # Error
       * 
       * * Only primary key can abdicate.
       * * Last member of a group cannot abdicate.
       **/
      abdicateMembership: AugmentedSubmittable<() => SubmittableExtrinsic<ApiType>, []>;
      /**
       * Adds a member `who` to the group. May only be called from `AddOrigin` or root.
       * 
       * # Arguments
       * * `origin` - Origin representing `AddOrigin` or root
       * * `who` - IdentityId to be added to the group.
       **/
      addMember: AugmentedSubmittable<(who: PolymeshPrimitivesIdentityId | string | Uint8Array) => SubmittableExtrinsic<ApiType>, [PolymeshPrimitivesIdentityId]>;
      /**
       * Disables a member at specific moment.
       * 
       * Please note that if member is already revoked (a "valid member"), its revocation
       * time-stamp will be updated.
       * 
       * Any disabled member should NOT allow to act like an active member of the group. For
       * instance, a disabled CDD member should NOT be able to generate a CDD claim. However any
       * generated claim issued before `at` would be considered as a valid one.
       * 
       * If you want to invalidate any generated claim, you should use `Self::remove_member`.
       * 
       * # Arguments
       * * `at` - Revocation time-stamp.
       * * `who` - Target member of the group.
       * * `expiry` - Time-stamp when `who` is removed from CDD. As soon as it is expired, the
       * generated claims will be "invalid" as `who` is not considered a member of the group.
       **/
      disableMember: AugmentedSubmittable<(who: PolymeshPrimitivesIdentityId | string | Uint8Array, expiry: Option<u64> | null | object | string | Uint8Array, at: Option<u64> | null | object | string | Uint8Array) => SubmittableExtrinsic<ApiType>, [PolymeshPrimitivesIdentityId, Option<u64>, Option<u64>]>;
      /**
       * Removes a member `who` from the set. May only be called from `RemoveOrigin` or root.
       * 
       * Any claim previously generated by this member is not valid as a group claim. For
       * instance, if a CDD member group generated a claim for a target identity and then it is
       * removed, that claim will be invalid.  In case you want to keep the validity of generated
       * claims, you have to use `Self::disable_member` function
       * 
       * # Arguments
       * * `origin` - Origin representing `RemoveOrigin` or root
       * * `who` - IdentityId to be removed from the group.
       **/
      removeMember: AugmentedSubmittable<(who: PolymeshPrimitivesIdentityId | string | Uint8Array) => SubmittableExtrinsic<ApiType>, [PolymeshPrimitivesIdentityId]>;
      /**
       * Changes the membership to a new set, disregarding the existing membership.
       * May only be called from `ResetOrigin` or root.
       * 
       * # Arguments
       * * `origin` - Origin representing `ResetOrigin` or root
       * * `members` - New set of identities
       **/
      resetMembers: AugmentedSubmittable<(members: Vec<PolymeshPrimitivesIdentityId> | (PolymeshPrimitivesIdentityId | string | Uint8Array)[]) => SubmittableExtrinsic<ApiType>, [Vec<PolymeshPrimitivesIdentityId>]>;
      /**
       * Change this group's limit for how many concurrent active members they may be.
       * 
       * # Arguments
       * * `limit` - the number of active members there may be concurrently.
       **/
      setActiveMembersLimit: AugmentedSubmittable<(limit: u32 | AnyNumber | Uint8Array) => SubmittableExtrinsic<ApiType>, [u32]>;
      /**
       * Swaps out one member `remove` for another member `add`.
       * 
       * May only be called from `SwapOrigin` or root.
       * 
       * # Arguments
       * * `origin` - Origin representing `SwapOrigin` or root
       * * `remove` - IdentityId to be removed from the group.
       * * `add` - IdentityId to be added in place of `remove`.
       **/
      swapMember: AugmentedSubmittable<(remove: PolymeshPrimitivesIdentityId | string | Uint8Array, add: PolymeshPrimitivesIdentityId | string | Uint8Array) => SubmittableExtrinsic<ApiType>, [PolymeshPrimitivesIdentityId, PolymeshPrimitivesIdentityId]>;
      /**
       * Generic tx
       **/
      [key: string]: SubmittableExtrinsicFunction<ApiType>;
    };
    checkpoint: {
      /**
       * Creates a single checkpoint at the current time.
       * 
       * # Arguments
       * - `origin` is a signer that has permissions to act as an agent of `ticker`.
       * - `ticker` to create the checkpoint for.
       * 
       * # Errors
       * - `UnauthorizedAgent` if the DID of `origin` isn't a permissioned agent for `ticker`.
       * - `CounterOverflow` if the total checkpoint counter would overflow.
       **/
      createCheckpoint: AugmentedSubmittable<(ticker: PolymeshPrimitivesTicker | string | Uint8Array) => SubmittableExtrinsic<ApiType>, [PolymeshPrimitivesTicker]>;
      /**
       * Creates a schedule generating checkpoints
       * in the future at either a fixed time or at intervals.
       * 
       * The schedule starts out with `strong_ref_count(schedule_id) <- 0`.
       * 
       * # Arguments
       * - `origin` is a signer that has permissions to act as owner of `ticker`.
       * - `ticker` to create the schedule for.
       * - `schedule` that will generate checkpoints.
       * 
       * # Errors
       * - `UnauthorizedAgent` if the DID of `origin` isn't a permissioned agent for `ticker`.
       * - `ScheduleDurationTooShort` if the schedule duration is too short.
       * - `InsufficientAccountBalance` if the protocol fee could not be charged.
       * - `CounterOverflow` if the schedule ID or total checkpoint counters would overflow.
       * - `FailedToComputeNextCheckpoint` if the next checkpoint for `schedule` is in the past.
       * 
       * # Permissions
       * * Asset
       **/
      createSchedule: AugmentedSubmittable<(ticker: PolymeshPrimitivesTicker | string | Uint8Array, schedule: PalletAssetCheckpointScheduleSpec | { start?: any; period?: any; remaining?: any } | string | Uint8Array) => SubmittableExtrinsic<ApiType>, [PolymeshPrimitivesTicker, PalletAssetCheckpointScheduleSpec]>;
      /**
       * Removes the checkpoint schedule of an asset identified by `id`.
       * 
       * # Arguments
       * - `origin` is a signer that has permissions to act as owner of `ticker`.
       * - `ticker` to remove the schedule from.
       * - `id` of the schedule, when it was created by `created_schedule`.
       * 
       * # Errors
       * - `UnauthorizedAgent` if the DID of `origin` isn't a permissioned agent for `ticker`.
       * - `NoCheckpointSchedule` if `id` does not identify a schedule for this `ticker`.
       * - `ScheduleNotRemovable` if `id` exists but is not removable.
       * 
       * # Permissions
       * * Asset
       **/
      removeSchedule: AugmentedSubmittable<(ticker: PolymeshPrimitivesTicker | string | Uint8Array, id: u64 | AnyNumber | Uint8Array) => SubmittableExtrinsic<ApiType>, [PolymeshPrimitivesTicker, u64]>;
      /**
       * Sets the max complexity of a schedule set for an arbitrary ticker to `max_complexity`.
       * The new maximum is not enforced retroactively,
       * and only applies once new schedules are made.
       * 
       * Must be called as a PIP (requires "root").
       * 
       * # Arguments
       * - `origin` is the root origin.
       * - `max_complexity` allowed for an arbitrary ticker's schedule set.
       **/
      setSchedulesMaxComplexity: AugmentedSubmittable<(maxComplexity: u64 | AnyNumber | Uint8Array) => SubmittableExtrinsic<ApiType>, [u64]>;
      /**
       * Generic tx
       **/
      [key: string]: SubmittableExtrinsicFunction<ApiType>;
    };
    committeeMembership: {
      /**
       * Allows the calling member to *unilaterally quit* without this being subject to a GC
       * vote.
       * 
       * # Arguments
       * * `origin` - Member of committee who wants to quit.
       * 
       * # Error
       * 
       * * Only primary key can abdicate.
       * * Last member of a group cannot abdicate.
       **/
      abdicateMembership: AugmentedSubmittable<() => SubmittableExtrinsic<ApiType>, []>;
      /**
       * Adds a member `who` to the group. May only be called from `AddOrigin` or root.
       * 
       * # Arguments
       * * `origin` - Origin representing `AddOrigin` or root
       * * `who` - IdentityId to be added to the group.
       **/
      addMember: AugmentedSubmittable<(who: PolymeshPrimitivesIdentityId | string | Uint8Array) => SubmittableExtrinsic<ApiType>, [PolymeshPrimitivesIdentityId]>;
      /**
       * Disables a member at specific moment.
       * 
       * Please note that if member is already revoked (a "valid member"), its revocation
       * time-stamp will be updated.
       * 
       * Any disabled member should NOT allow to act like an active member of the group. For
       * instance, a disabled CDD member should NOT be able to generate a CDD claim. However any
       * generated claim issued before `at` would be considered as a valid one.
       * 
       * If you want to invalidate any generated claim, you should use `Self::remove_member`.
       * 
       * # Arguments
       * * `at` - Revocation time-stamp.
       * * `who` - Target member of the group.
       * * `expiry` - Time-stamp when `who` is removed from CDD. As soon as it is expired, the
       * generated claims will be "invalid" as `who` is not considered a member of the group.
       **/
      disableMember: AugmentedSubmittable<(who: PolymeshPrimitivesIdentityId | string | Uint8Array, expiry: Option<u64> | null | object | string | Uint8Array, at: Option<u64> | null | object | string | Uint8Array) => SubmittableExtrinsic<ApiType>, [PolymeshPrimitivesIdentityId, Option<u64>, Option<u64>]>;
      /**
       * Removes a member `who` from the set. May only be called from `RemoveOrigin` or root.
       * 
       * Any claim previously generated by this member is not valid as a group claim. For
       * instance, if a CDD member group generated a claim for a target identity and then it is
       * removed, that claim will be invalid.  In case you want to keep the validity of generated
       * claims, you have to use `Self::disable_member` function
       * 
       * # Arguments
       * * `origin` - Origin representing `RemoveOrigin` or root
       * * `who` - IdentityId to be removed from the group.
       **/
      removeMember: AugmentedSubmittable<(who: PolymeshPrimitivesIdentityId | string | Uint8Array) => SubmittableExtrinsic<ApiType>, [PolymeshPrimitivesIdentityId]>;
      /**
       * Changes the membership to a new set, disregarding the existing membership.
       * May only be called from `ResetOrigin` or root.
       * 
       * # Arguments
       * * `origin` - Origin representing `ResetOrigin` or root
       * * `members` - New set of identities
       **/
      resetMembers: AugmentedSubmittable<(members: Vec<PolymeshPrimitivesIdentityId> | (PolymeshPrimitivesIdentityId | string | Uint8Array)[]) => SubmittableExtrinsic<ApiType>, [Vec<PolymeshPrimitivesIdentityId>]>;
      /**
       * Change this group's limit for how many concurrent active members they may be.
       * 
       * # Arguments
       * * `limit` - the number of active members there may be concurrently.
       **/
      setActiveMembersLimit: AugmentedSubmittable<(limit: u32 | AnyNumber | Uint8Array) => SubmittableExtrinsic<ApiType>, [u32]>;
      /**
       * Swaps out one member `remove` for another member `add`.
       * 
       * May only be called from `SwapOrigin` or root.
       * 
       * # Arguments
       * * `origin` - Origin representing `SwapOrigin` or root
       * * `remove` - IdentityId to be removed from the group.
       * * `add` - IdentityId to be added in place of `remove`.
       **/
      swapMember: AugmentedSubmittable<(remove: PolymeshPrimitivesIdentityId | string | Uint8Array, add: PolymeshPrimitivesIdentityId | string | Uint8Array) => SubmittableExtrinsic<ApiType>, [PolymeshPrimitivesIdentityId, PolymeshPrimitivesIdentityId]>;
      /**
       * Generic tx
       **/
      [key: string]: SubmittableExtrinsicFunction<ApiType>;
    };
    complianceManager: {
      /**
       * Adds a compliance requirement to an asset's compliance by ticker.
       * If the compliance requirement is a duplicate, it does nothing.
       * 
       * # Arguments
       * * origin - Signer of the dispatchable. It should be the owner of the ticker
       * * ticker - Symbol of the asset
       * * sender_conditions - Sender transfer conditions.
       * * receiver_conditions - Receiver transfer conditions.
       * 
       * # Permissions
       * * Asset
       **/
      addComplianceRequirement: AugmentedSubmittable<(ticker: PolymeshPrimitivesTicker | string | Uint8Array, senderConditions: Vec<PolymeshPrimitivesCondition> | (PolymeshPrimitivesCondition | { conditionType?: any; issuers?: any } | string | Uint8Array)[], receiverConditions: Vec<PolymeshPrimitivesCondition> | (PolymeshPrimitivesCondition | { conditionType?: any; issuers?: any } | string | Uint8Array)[]) => SubmittableExtrinsic<ApiType>, [PolymeshPrimitivesTicker, Vec<PolymeshPrimitivesCondition>, Vec<PolymeshPrimitivesCondition>]>;
      /**
       * Adds another default trusted claim issuer at the ticker level.
       * 
       * # Arguments
       * * origin - Signer of the dispatchable. It should be the owner of the ticker.
       * * ticker - Symbol of the asset.
       * * issuer - IdentityId of the trusted claim issuer.
       * 
       * # Permissions
       * * Asset
       **/
      addDefaultTrustedClaimIssuer: AugmentedSubmittable<(ticker: PolymeshPrimitivesTicker | string | Uint8Array, issuer: PolymeshPrimitivesConditionTrustedIssuer | { issuer?: any; trustedFor?: any } | string | Uint8Array) => SubmittableExtrinsic<ApiType>, [PolymeshPrimitivesTicker, PolymeshPrimitivesConditionTrustedIssuer]>;
      /**
       * Modify an existing compliance requirement of a given ticker.
       * 
       * # Arguments
       * * origin - Signer of the dispatchable. It should be the owner of the ticker.
       * * ticker - Symbol of the asset.
       * * new_req - Compliance requirement.
       * 
       * # Permissions
       * * Asset
       **/
      changeComplianceRequirement: AugmentedSubmittable<(ticker: PolymeshPrimitivesTicker | string | Uint8Array, newReq: PolymeshPrimitivesComplianceManagerComplianceRequirement | { senderConditions?: any; receiverConditions?: any; id?: any } | string | Uint8Array) => SubmittableExtrinsic<ApiType>, [PolymeshPrimitivesTicker, PolymeshPrimitivesComplianceManagerComplianceRequirement]>;
      /**
       * Pauses the verification of conditions for `ticker` during transfers.
       * 
       * # Arguments
       * * origin - Signer of the dispatchable. It should be the owner of the ticker
       * * ticker - Symbol of the asset
       * 
       * # Permissions
       * * Asset
       **/
      pauseAssetCompliance: AugmentedSubmittable<(ticker: PolymeshPrimitivesTicker | string | Uint8Array) => SubmittableExtrinsic<ApiType>, [PolymeshPrimitivesTicker]>;
      /**
       * Removes a compliance requirement from an asset's compliance.
       * 
       * # Arguments
       * * origin - Signer of the dispatchable. It should be the owner of the ticker
       * * ticker - Symbol of the asset
       * * id - Compliance requirement id which is need to be removed
       * 
       * # Permissions
       * * Asset
       **/
      removeComplianceRequirement: AugmentedSubmittable<(ticker: PolymeshPrimitivesTicker | string | Uint8Array, id: u32 | AnyNumber | Uint8Array) => SubmittableExtrinsic<ApiType>, [PolymeshPrimitivesTicker, u32]>;
      /**
       * Removes the given `issuer` from the set of default trusted claim issuers at the ticker level.
       * 
       * # Arguments
       * * origin - Signer of the dispatchable. It should be the owner of the ticker.
       * * ticker - Symbol of the asset.
       * * issuer - IdentityId of the trusted claim issuer.
       * 
       * # Permissions
       * * Asset
       **/
      removeDefaultTrustedClaimIssuer: AugmentedSubmittable<(ticker: PolymeshPrimitivesTicker | string | Uint8Array, issuer: PolymeshPrimitivesIdentityId | string | Uint8Array) => SubmittableExtrinsic<ApiType>, [PolymeshPrimitivesTicker, PolymeshPrimitivesIdentityId]>;
      /**
       * Replaces an asset's compliance by ticker with a new compliance.
       * 
       * Compliance requirements will be sorted (ascending by id) before
       * replacing the current requirements.
       * 
       * # Arguments
       * * `ticker` - the asset ticker,
       * * `asset_compliance - the new asset compliance.
       * 
       * # Errors
       * * `Unauthorized` if `origin` is not the owner of the ticker.
       * * `DuplicateAssetCompliance` if `asset_compliance` contains multiple entries with the same `requirement_id`.
       * 
       * # Permissions
       * * Asset
       **/
      replaceAssetCompliance: AugmentedSubmittable<(ticker: PolymeshPrimitivesTicker | string | Uint8Array, assetCompliance: Vec<PolymeshPrimitivesComplianceManagerComplianceRequirement> | (PolymeshPrimitivesComplianceManagerComplianceRequirement | { senderConditions?: any; receiverConditions?: any; id?: any } | string | Uint8Array)[]) => SubmittableExtrinsic<ApiType>, [PolymeshPrimitivesTicker, Vec<PolymeshPrimitivesComplianceManagerComplianceRequirement>]>;
      /**
       * Removes an asset's compliance
       * 
       * # Arguments
       * * origin - Signer of the dispatchable. It should be the owner of the ticker
       * * ticker - Symbol of the asset
       * 
       * # Permissions
       * * Asset
       **/
      resetAssetCompliance: AugmentedSubmittable<(ticker: PolymeshPrimitivesTicker | string | Uint8Array) => SubmittableExtrinsic<ApiType>, [PolymeshPrimitivesTicker]>;
      /**
       * Resumes the verification of conditions for `ticker` during transfers.
       * 
       * # Arguments
       * * origin - Signer of the dispatchable. It should be the owner of the ticker
       * * ticker - Symbol of the asset
       * 
       * # Permissions
       * * Asset
       **/
      resumeAssetCompliance: AugmentedSubmittable<(ticker: PolymeshPrimitivesTicker | string | Uint8Array) => SubmittableExtrinsic<ApiType>, [PolymeshPrimitivesTicker]>;
      /**
       * Generic tx
       **/
      [key: string]: SubmittableExtrinsicFunction<ApiType>;
    };
    corporateAction: {
      /**
       * Changes the record date of the CA identified by `ca_id`.
       * 
       * ## Arguments
       * - `origin` which must be an external agent of `ca_id.ticker` with relevant permissions.
       * - `ca_id` of the CA to alter.
       * - `record_date`, if any, to calculate the impact of the CA.
       * If provided, this results in a scheduled balance snapshot ("checkpoint") at the date.
       * 
       * # Errors
       * - `UnauthorizedAgent` if `origin` is not agent-permissioned for `ticker`.
       * - `NoSuchCA` if `id` does not identify an existing CA.
       * - When `record_date.is_some()`, other errors due to checkpoint scheduling may occur.
       * 
       * # Permissions
       * * Asset
       **/
      changeRecordDate: AugmentedSubmittable<(caId: PalletCorporateActionsCaId | { ticker?: any; localId?: any } | string | Uint8Array, recordDate: Option<PalletCorporateActionsRecordDateSpec> | null | object | string | Uint8Array) => SubmittableExtrinsic<ApiType>, [PalletCorporateActionsCaId, Option<PalletCorporateActionsRecordDateSpec>]>;
      /**
       * Initiates a CA for `ticker` of `kind` with `details` and other provided arguments.
       * 
       * ## Arguments
       * - `origin` which must be an external agent of `ticker` with relevant permissions.
       * - `ticker` that the CA is made for.
       * - `kind` of CA being initiated.
       * - `decl_date` of CA bring initialized.
       * - `record_date`, if any, to calculate the impact of this CA.
       * If provided, this results in a scheduled balance snapshot ("checkpoint") at the date.
       * - `details` of the CA in free-text form, up to a certain number of bytes in length.
       * - `targets`, if any, which this CA is relevant/irrelevant to.
       * Overrides, if provided, the default at the asset level (`set_default_targets`).
       * - `default_withholding_tax`, if any, is the default withholding tax to use for this CA.
       * Overrides, if provided, the default at the asset level (`set_default_withholding_tax`).
       * - `withholding_tax`, if any, provides per-DID withholding tax overrides.
       * Overrides, if provided, the default at the asset level (`set_did_withholding_tax`).
       * 
       * # Errors
       * - `DetailsTooLong` if `details.len()` goes beyond `max_details_length`.
       * - `UnauthorizedAgent` if `origin` is not agent-permissioned for `ticker`.
       * - `CounterOverflow` in the unlikely event that so many CAs were created for this `ticker`,
       * that integer overflow would have occured if instead allowed.
       * - `TooManyDidTaxes` if `withholding_tax.unwrap().len()` would go over the limit `MaxDidWhts`.
       * - `DuplicateDidTax` if a DID is included more than once in `wt`.
       * - `TooManyTargetIds` if `targets.unwrap().identities.len() > T::MaxTargetIds::get()`.
       * - `DeclDateInFuture` if the declaration date is not in the past.
       * - When `record_date.is_some()`, other errors due to checkpoint scheduling may occur.
       * 
       * # Permissions
       * * Asset
       **/
      initiateCorporateAction: AugmentedSubmittable<(ticker: PolymeshPrimitivesTicker | string | Uint8Array, kind: PalletCorporateActionsCaKind | 'PredictableBenefit' | 'UnpredictableBenefit' | 'IssuerNotice' | 'Reorganization' | 'Other' | number | Uint8Array, declDate: u64 | AnyNumber | Uint8Array, recordDate: Option<PalletCorporateActionsRecordDateSpec> | null | object | string | Uint8Array, details: Bytes | string | Uint8Array, targets: Option<PalletCorporateActionsTargetIdentities> | null | object | string | Uint8Array, defaultWithholdingTax: Option<Permill> | null | object | string | Uint8Array, withholdingTax: Option<Vec<ITuple<[PolymeshPrimitivesIdentityId, Permill]>>> | null | object | string | Uint8Array) => SubmittableExtrinsic<ApiType>, [PolymeshPrimitivesTicker, PalletCorporateActionsCaKind, u64, Option<PalletCorporateActionsRecordDateSpec>, Bytes, Option<PalletCorporateActionsTargetIdentities>, Option<Permill>, Option<Vec<ITuple<[PolymeshPrimitivesIdentityId, Permill]>>>]>;
      /**
       * Utility extrinsic to batch `initiate_corporate_action` and `distribute`
       **/
      initiateCorporateActionAndDistribute: AugmentedSubmittable<(caArgs: PalletCorporateActionsInitiateCorporateActionArgs | { ticker?: any; kind?: any; declDate?: any; recordDate?: any; details?: any; targets?: any; defaultWithholdingTax?: any; withholdingTax?: any } | string | Uint8Array, portfolio: Option<u64> | null | object | string | Uint8Array, currency: PolymeshPrimitivesTicker | string | Uint8Array, perShare: u128 | AnyNumber | Uint8Array, amount: u128 | AnyNumber | Uint8Array, paymentAt: u64 | AnyNumber | Uint8Array, expiresAt: Option<u64> | null | object | string | Uint8Array) => SubmittableExtrinsic<ApiType>, [PalletCorporateActionsInitiateCorporateActionArgs, Option<u64>, PolymeshPrimitivesTicker, u128, u128, u64, Option<u64>]>;
      /**
       * Link the given CA `id` to the given `docs`.
       * Any previous links for the CA are removed in favor of `docs`.
       * 
       * The workflow here is to add the documents and initiating the CA in any order desired.
       * Once both exist, they can now be linked together.
       * 
       * ## Arguments
       * - `origin` which must be an external agent of `id.ticker` with relevant permissions.
       * - `id` of the CA to associate with `docs`.
       * - `docs` to associate with the CA with `id`.
       * 
       * # Errors
       * - `UnauthorizedAgent` if `origin` is not agent-permissioned for `ticker`.
       * - `NoSuchCA` if `id` does not identify an existing CA.
       * - `NoSuchDoc` if any of `docs` does not identify an existing document.
       * 
       * # Permissions
       * * Asset
       **/
      linkCaDoc: AugmentedSubmittable<(id: PalletCorporateActionsCaId | { ticker?: any; localId?: any } | string | Uint8Array, docs: Vec<u32> | (u32 | AnyNumber | Uint8Array)[]) => SubmittableExtrinsic<ApiType>, [PalletCorporateActionsCaId, Vec<u32>]>;
      /**
       * Removes the CA identified by `ca_id`.
       * 
       * Associated data, such as document links, ballots,
       * and capital distributions are also removed.
       * 
       * Any schedule associated with the record date will see
       * `strong_ref_count(schedule_id)` decremented.
       * 
       * ## Arguments
       * - `origin` which must be an external agent of `ca_id.ticker` with relevant permissions.
       * - `ca_id` of the CA to remove.
       * 
       * # Errors
       * - `UnauthorizedAgent` if `origin` is not agent-permissioned for `ticker`.
       * - `NoSuchCA` if `id` does not identify an existing CA.
       * 
       * # Permissions
       * * Asset
       **/
      removeCa: AugmentedSubmittable<(caId: PalletCorporateActionsCaId | { ticker?: any; localId?: any } | string | Uint8Array) => SubmittableExtrinsic<ApiType>, [PalletCorporateActionsCaId]>;
      /**
       * Set the default CA `TargetIdentities` to `targets`.
       * 
       * ## Arguments
       * - `origin` which must be an external agent of `ticker` with relevant permissions.
       * - `ticker` for which the default identities are changing.
       * - `targets` the default target identities for a CA.
       * 
       * ## Errors
       * - `UnauthorizedAgent` if `origin` is not agent-permissioned for `ticker`.
       * - `TooManyTargetIds` if `targets.identities.len() > T::MaxTargetIds::get()`.
       * 
       * # Permissions
       * * Asset
       **/
      setDefaultTargets: AugmentedSubmittable<(ticker: PolymeshPrimitivesTicker | string | Uint8Array, targets: PalletCorporateActionsTargetIdentities | { identities?: any; treatment?: any } | string | Uint8Array) => SubmittableExtrinsic<ApiType>, [PolymeshPrimitivesTicker, PalletCorporateActionsTargetIdentities]>;
      /**
       * Set the default withholding tax for all DIDs and CAs relevant to this `ticker`.
       * 
       * ## Arguments
       * - `origin` which must be an external agent of `ticker` with relevant permissions.
       * - `ticker` that the withholding tax will apply to.
       * - `tax` that should be withheld when distributing dividends, etc.
       * 
       * ## Errors
       * - `UnauthorizedAgent` if `origin` is not agent-permissioned for `ticker`.
       * 
       * # Permissions
       * * Asset
       **/
      setDefaultWithholdingTax: AugmentedSubmittable<(ticker: PolymeshPrimitivesTicker | string | Uint8Array, tax: Permill | AnyNumber | Uint8Array) => SubmittableExtrinsic<ApiType>, [PolymeshPrimitivesTicker, Permill]>;
      /**
       * Set the withholding tax of `ticker` for `taxed_did` to `tax`.
       * If `Some(tax)`, this overrides the default withholding tax of `ticker` to `tax` for `taxed_did`.
       * Otherwise, if `None`, the default withholding tax will be used.
       * 
       * ## Arguments
       * - `origin` which must be an external agent of `ticker` with relevant permissions.
       * - `ticker` that the withholding tax will apply to.
       * - `taxed_did` that will have its withholding tax updated.
       * - `tax` that should be withheld when distributing dividends, etc.
       * 
       * ## Errors
       * - `UnauthorizedAgent` if `origin` is not agent-permissioned for `ticker`.
       * - `TooManyDidTaxes` if `Some(tax)` and adding the override would go over the limit `MaxDidWhts`.
       * 
       * # Permissions
       * * Asset
       **/
      setDidWithholdingTax: AugmentedSubmittable<(ticker: PolymeshPrimitivesTicker | string | Uint8Array, taxedDid: PolymeshPrimitivesIdentityId | string | Uint8Array, tax: Option<Permill> | null | object | string | Uint8Array) => SubmittableExtrinsic<ApiType>, [PolymeshPrimitivesTicker, PolymeshPrimitivesIdentityId, Option<Permill>]>;
      /**
       * Set the max `length` of `details` in terms of bytes.
       * May only be called via a PIP.
       **/
      setMaxDetailsLength: AugmentedSubmittable<(length: u32 | AnyNumber | Uint8Array) => SubmittableExtrinsic<ApiType>, [u32]>;
      /**
       * Generic tx
       **/
      [key: string]: SubmittableExtrinsicFunction<ApiType>;
    };
    corporateBallot: {
      /**
       * Attach a corporate ballot to the CA identified by `ca_id`.
       * 
       * The ballot will admit votes within `range`.
       * The ballot's metadata is provided by `meta`,
       * which includes the ballot title, the motions, their choices, etc.
       * See the `BallotMeta` for more.
       * 
       * ## Arguments
       * - `origin` is a signer that has permissions to act as an agent of `ca_id.ticker`.
       * - `ca_id` identifies the CA to attach the ballot to.
       * - `range` specifies when voting starts and ends.
       * - `meta` specifies the ballot's metadata as aforementioned.
       * - `rcv` specifies whether RCV is enabled for this ballot.
       * 
       * # Errors
       * - `UnauthorizedAgent` if `origin` is not agent-permissioned for `ticker`.
       * - `NoSuchCA` if `ca_id` does not identify an existing CA.
       * - `CANotNotice` if the CA is not of the `IssuerNotice` kind.
       * - `StartAfterEnd` if `range.start > range.end`.
       * - `NowAfterEnd` if `now > range.end` where `now` is the current timestamp.
       * - `NoRecordDate` if CA has no record date.
       * - `RecordDateAfterStart` if `date > range.start` where `date` is the CA's record date.
       * - `AlreadyExists` if there's a ballot already.
       * - `NumberOfChoicesOverflow` if the total choice in `meta` overflows `usize`.
       * - `TooLong` if any of the embedded strings in `meta` are too long.
       * - `InsufficientBalance` if the protocol fee couldn't be charged.
       **/
      attachBallot: AugmentedSubmittable<(caId: PalletCorporateActionsCaId | { ticker?: any; localId?: any } | string | Uint8Array, range: PalletCorporateActionsBallotBallotTimeRange | { start?: any; end?: any } | string | Uint8Array, meta: PalletCorporateActionsBallotBallotMeta | { title?: any; motions?: any } | string | Uint8Array, rcv: bool | boolean | Uint8Array) => SubmittableExtrinsic<ApiType>, [PalletCorporateActionsCaId, PalletCorporateActionsBallotBallotTimeRange, PalletCorporateActionsBallotBallotMeta, bool]>;
      /**
       * Amend the end date of the ballot of the CA identified by `ca_id`.
       * 
       * ## Arguments
       * - `origin` is a signer that has permissions to act as an agent of `ca_id.ticker`.
       * - `ca_id` identifies the attached ballot's CA.
       * - `end` specifies the new end date of the ballot.
       * 
       * # Errors
       * - `UnauthorizedAgent` if `origin` is not agent-permissioned for `ticker`.
       * - `NoSuchBallot` if `ca_id` does not identify a ballot.
       * - `VotingAlreadyStarted` if `start >= now`, where `now` is the current time.
       * - `StartAfterEnd` if `start > end`.
       **/
      changeEnd: AugmentedSubmittable<(caId: PalletCorporateActionsCaId | { ticker?: any; localId?: any } | string | Uint8Array, end: u64 | AnyNumber | Uint8Array) => SubmittableExtrinsic<ApiType>, [PalletCorporateActionsCaId, u64]>;
      /**
       * Amend the metadata (title, motions, etc.) of the ballot of the CA identified by `ca_id`.
       * 
       * ## Arguments
       * - `origin` is a signer that has permissions to act as an agent of `ca_id.ticker`.
       * - `ca_id` identifies the attached ballot's CA.
       * - `meta` specifies the new metadata.
       * 
       * # Errors
       * - `UnauthorizedAgent` if `origin` is not agent-permissioned for `ticker`.
       * - `NoSuchBallot` if `ca_id` does not identify a ballot.
       * - `VotingAlreadyStarted` if `start >= now`, where `now` is the current time.
       * - `NumberOfChoicesOverflow` if the total choice in `meta` overflows `usize`.
       * - `TooLong` if any of the embedded strings in `meta` are too long.
       **/
      changeMeta: AugmentedSubmittable<(caId: PalletCorporateActionsCaId | { ticker?: any; localId?: any } | string | Uint8Array, meta: PalletCorporateActionsBallotBallotMeta | { title?: any; motions?: any } | string | Uint8Array) => SubmittableExtrinsic<ApiType>, [PalletCorporateActionsCaId, PalletCorporateActionsBallotBallotMeta]>;
      /**
       * Amend RCV support for the ballot of the CA identified by `ca_id`.
       * 
       * ## Arguments
       * - `origin` is a signer that has permissions to act as an agent of `ca_id.ticker`.
       * - `ca_id` identifies the attached ballot's CA.
       * - `rcv` specifies if RCV is to be supported or not.
       * 
       * # Errors
       * - `UnauthorizedAgent` if `origin` is not agent-permissioned for `ticker`.
       * - `NoSuchBallot` if `ca_id` does not identify a ballot.
       * - `VotingAlreadyStarted` if `start >= now`, where `now` is the current time.
       **/
      changeRcv: AugmentedSubmittable<(caId: PalletCorporateActionsCaId | { ticker?: any; localId?: any } | string | Uint8Array, rcv: bool | boolean | Uint8Array) => SubmittableExtrinsic<ApiType>, [PalletCorporateActionsCaId, bool]>;
      /**
       * Remove the ballot of the CA identified by `ca_id`.
       * 
       * ## Arguments
       * - `origin` is a signer that has permissions to act as an agent of `ca_id.ticker`.
       * - `ca_id` identifies the attached ballot's CA.
       * 
       * # Errors
       * - `UnauthorizedAgent` if `origin` is not agent-permissioned for `ticker`.
       * - `NoSuchBallot` if `ca_id` does not identify a ballot.
       * - `VotingAlreadyStarted` if `start >= now`, where `now` is the current time.
       **/
      removeBallot: AugmentedSubmittable<(caId: PalletCorporateActionsCaId | { ticker?: any; localId?: any } | string | Uint8Array) => SubmittableExtrinsic<ApiType>, [PalletCorporateActionsCaId]>;
      /**
       * Cast `votes` in the ballot attached to the CA identified by `ca_id`.
       * 
       * ## Arguments
       * - `origin` which must be a permissioned signer targeted by the CA.
       * - `ca_id` identifies the attached ballot's CA.
       * - `votes` specifies the balances to assign to each choice in the ballot.
       * The full voting power of `origin`'s DID may be used for each motion in the ballot.
       * 
       * # Errors
       * - `NoSuchBallot` if `ca_id` does not identify a ballot.
       * - `VotingNotStarted` if the voting period hasn't commenced yet.
       * - `VotingAlreadyEnded` if the voting period has ended.
       * - `WrongVoteCount` if the number of choices in the ballot does not match `votes.len()`.
       * - `NoSuchCA` if `ca_id` does not identify an existing CA.
       * - `NotTargetedByCA` if the CA does not target `origin`'s DID.
       * - `InsufficientVotes` if the voting power used for any motion in `votes`
       * exceeds `origin`'s DID's voting power.
       **/
      vote: AugmentedSubmittable<(caId: PalletCorporateActionsCaId | { ticker?: any; localId?: any } | string | Uint8Array, votes: Vec<PalletCorporateActionsBallotBallotVote> | (PalletCorporateActionsBallotBallotVote | { power?: any; fallback?: any } | string | Uint8Array)[]) => SubmittableExtrinsic<ApiType>, [PalletCorporateActionsCaId, Vec<PalletCorporateActionsBallotBallotVote>]>;
      /**
       * Generic tx
       **/
      [key: string]: SubmittableExtrinsicFunction<ApiType>;
    };
    externalAgents: {
      /**
       * Abdicate agentship for `ticker`.
       * 
       * # Arguments
       * - `ticker` of which the caller is an agent.
       * 
       * # Errors
       * - `NotAnAgent` if the caller is not an agent of `ticker`.
       * - `RemovingLastFullAgent` if the caller is the last full agent.
       * 
       * # Permissions
       * * Asset
       **/
      abdicate: AugmentedSubmittable<(ticker: PolymeshPrimitivesTicker | string | Uint8Array) => SubmittableExtrinsic<ApiType>, [PolymeshPrimitivesTicker]>;
      /**
       * Accept an authorization by an agent "Alice" who issued `auth_id`
       * to also become an agent of the ticker Alice specified.
       * 
       * # Arguments
       * - `auth_id` identifying the authorization to accept.
       * 
       * # Errors
       * - `AuthorizationError::Invalid` if `auth_id` does not exist for the given caller.
       * - `AuthorizationError::Expired` if `auth_id` is for an auth that has expired.
       * - `AuthorizationError::BadType` if `auth_id` was not for a `BecomeAgent` auth type.
       * - `UnauthorizedAgent` if "Alice" is not permissioned to provide the auth.
       * - `NoSuchAG` if the group referred to a custom that does not exist.
       * - `AlreadyAnAgent` if the caller is already an agent of the ticker.
       * 
       * # Permissions
       * * Agent
       **/
      acceptBecomeAgent: AugmentedSubmittable<(authId: u64 | AnyNumber | Uint8Array) => SubmittableExtrinsic<ApiType>, [u64]>;
      /**
       * Change the agent group that `agent` belongs to in `ticker`.
       * 
       * # Arguments
       * - `ticker` that has the `agent`.
       * - `agent` of `ticker` to change the group for.
       * - `group` that `agent` will belong to in `ticker`.
       * 
       * # Errors
       * - `UnauthorizedAgent` if `origin` was not authorized as an agent to call this.
       * - `NoSuchAG` if `id` does not identify a custom AG.
       * - `NotAnAgent` if `agent` is not an agent of `ticker`.
       * - `RemovingLastFullAgent` if `agent` was a `Full` one and is being demoted.
       * 
       * # Permissions
       * * Asset
       * * Agent
       **/
      changeGroup: AugmentedSubmittable<(ticker: PolymeshPrimitivesTicker | string | Uint8Array, agent: PolymeshPrimitivesIdentityId | string | Uint8Array, group: PolymeshPrimitivesAgentAgentGroup | { Full: any } | { Custom: any } | { ExceptMeta: any } | { PolymeshV1CAA: any } | { PolymeshV1PIA: any } | string | Uint8Array) => SubmittableExtrinsic<ApiType>, [PolymeshPrimitivesTicker, PolymeshPrimitivesIdentityId, PolymeshPrimitivesAgentAgentGroup]>;
      /**
       * Utility extrinsic to batch `create_group` and  `change_group` for custom groups only.
       * 
       * # Permissions
       * * Asset
       * * Agent
       **/
      createAndChangeCustomGroup: AugmentedSubmittable<(ticker: PolymeshPrimitivesTicker | string | Uint8Array, perms: PolymeshPrimitivesSubsetSubsetRestrictionPalletPermissions | { Whole: any } | { These: any } | { Except: any } | string | Uint8Array, agent: PolymeshPrimitivesIdentityId | string | Uint8Array) => SubmittableExtrinsic<ApiType>, [PolymeshPrimitivesTicker, PolymeshPrimitivesSubsetSubsetRestrictionPalletPermissions, PolymeshPrimitivesIdentityId]>;
      /**
       * Creates a custom agent group (AG) for the given `ticker`.
       * 
       * The AG will have the permissions as given by `perms`.
       * This new AG is then assigned `id = AGIdSequence::get() + 1` as its `AGId`,
       * which you can use as `AgentGroup::Custom(id)` when adding agents for `ticker`.
       * 
       * # Arguments
       * - `ticker` to add the custom group for.
       * - `perms` that the new AG will have.
       * 
       * # Errors
       * - `UnauthorizedAgent` if `origin` was not authorized as an agent to call this.
       * - `TooLong` if `perms` had some string or list length that was too long.
       * - `CounterOverflow` if `AGIdSequence::get() + 1` would exceed `u32::MAX`.
       * 
       * # Permissions
       * * Asset
       * * Agent
       **/
      createGroup: AugmentedSubmittable<(ticker: PolymeshPrimitivesTicker | string | Uint8Array, perms: PolymeshPrimitivesSubsetSubsetRestrictionPalletPermissions | { Whole: any } | { These: any } | { Except: any } | string | Uint8Array) => SubmittableExtrinsic<ApiType>, [PolymeshPrimitivesTicker, PolymeshPrimitivesSubsetSubsetRestrictionPalletPermissions]>;
      /**
       * Utility extrinsic to batch `create_group` and  `add_auth`.
       * 
       * # Permissions
       * * Asset
       * * Agent
       **/
      createGroupAndAddAuth: AugmentedSubmittable<(ticker: PolymeshPrimitivesTicker | string | Uint8Array, perms: PolymeshPrimitivesSubsetSubsetRestrictionPalletPermissions | { Whole: any } | { These: any } | { Except: any } | string | Uint8Array, target: PolymeshPrimitivesSecondaryKeySignatory | { Identity: any } | { Account: any } | string | Uint8Array) => SubmittableExtrinsic<ApiType>, [PolymeshPrimitivesTicker, PolymeshPrimitivesSubsetSubsetRestrictionPalletPermissions, PolymeshPrimitivesSecondaryKeySignatory]>;
      /**
       * Remove the given `agent` from `ticker`.
       * 
       * # Arguments
       * - `ticker` that has the `agent` to remove.
       * - `agent` of `ticker` to remove.
       * 
       * # Errors
       * - `UnauthorizedAgent` if `origin` was not authorized as an agent to call this.
       * - `NotAnAgent` if `agent` is not an agent of `ticker`.
       * - `RemovingLastFullAgent` if `agent` is the last full one.
       * 
       * # Permissions
       * * Asset
       * * Agent
       **/
      removeAgent: AugmentedSubmittable<(ticker: PolymeshPrimitivesTicker | string | Uint8Array, agent: PolymeshPrimitivesIdentityId | string | Uint8Array) => SubmittableExtrinsic<ApiType>, [PolymeshPrimitivesTicker, PolymeshPrimitivesIdentityId]>;
      /**
       * Updates the permissions of the custom AG identified by `id`, for the given `ticker`.
       * 
       * # Arguments
       * - `ticker` the custom AG belongs to.
       * - `id` for the custom AG within `ticker`.
       * - `perms` to update the custom AG to.
       * 
       * # Errors
       * - `UnauthorizedAgent` if `origin` was not authorized as an agent to call this.
       * - `TooLong` if `perms` had some string or list length that was too long.
       * - `NoSuchAG` if `id` does not identify a custom AG.
       * 
       * # Permissions
       * * Asset
       * * Agent
       **/
      setGroupPermissions: AugmentedSubmittable<(ticker: PolymeshPrimitivesTicker | string | Uint8Array, id: u32 | AnyNumber | Uint8Array, perms: PolymeshPrimitivesSubsetSubsetRestrictionPalletPermissions | { Whole: any } | { These: any } | { Except: any } | string | Uint8Array) => SubmittableExtrinsic<ApiType>, [PolymeshPrimitivesTicker, u32, PolymeshPrimitivesSubsetSubsetRestrictionPalletPermissions]>;
      /**
       * Generic tx
       **/
      [key: string]: SubmittableExtrinsicFunction<ApiType>;
    };
    grandpa: {
      /**
       * Note that the current authority set of the GRANDPA finality gadget has
       * stalled. This will trigger a forced authority set change at the beginning
       * of the next session, to be enacted `delay` blocks after that. The delay
       * should be high enough to safely assume that the block signalling the
       * forced change will not be re-orged (e.g. 1000 blocks). The GRANDPA voters
       * will start the new authority set using the given finalized block as base.
       * Only callable by root.
       **/
      noteStalled: AugmentedSubmittable<(delay: u32 | AnyNumber | Uint8Array, bestFinalizedBlockNumber: u32 | AnyNumber | Uint8Array) => SubmittableExtrinsic<ApiType>, [u32, u32]>;
      /**
       * Report voter equivocation/misbehavior. This method will verify the
       * equivocation proof and validate the given key ownership proof
       * against the extracted offender. If both are valid, the offence
       * will be reported.
       **/
      reportEquivocation: AugmentedSubmittable<(equivocationProof: SpFinalityGrandpaEquivocationProof | { setId?: any; equivocation?: any } | string | Uint8Array, keyOwnerProof: SpSessionMembershipProof | { session?: any; trieNodes?: any; validatorCount?: any } | string | Uint8Array) => SubmittableExtrinsic<ApiType>, [SpFinalityGrandpaEquivocationProof, SpSessionMembershipProof]>;
      /**
       * Report voter equivocation/misbehavior. This method will verify the
       * equivocation proof and validate the given key ownership proof
       * against the extracted offender. If both are valid, the offence
       * will be reported.
       * 
       * This extrinsic must be called unsigned and it is expected that only
       * block authors will call it (validated in `ValidateUnsigned`), as such
       * if the block author is defined it will be defined as the equivocation
       * reporter.
       **/
      reportEquivocationUnsigned: AugmentedSubmittable<(equivocationProof: SpFinalityGrandpaEquivocationProof | { setId?: any; equivocation?: any } | string | Uint8Array, keyOwnerProof: SpSessionMembershipProof | { session?: any; trieNodes?: any; validatorCount?: any } | string | Uint8Array) => SubmittableExtrinsic<ApiType>, [SpFinalityGrandpaEquivocationProof, SpSessionMembershipProof]>;
      /**
       * Generic tx
       **/
      [key: string]: SubmittableExtrinsicFunction<ApiType>;
    };
    identity: {
      /**
       * Call this with the new primary key. By invoking this method, caller accepts authorization
       * to become the new primary key of the issuing identity. If a CDD service provider approved
       * this change (or this is not required), primary key of the DID is updated.
       * 
       * The caller (new primary key) must be either a secondary key of the issuing identity, or
       * unlinked to any identity.
       * 
       * Differs from rotate_primary_key_to_secondary in that it will unlink the old primary key
       * instead of leaving it as a secondary key.
       * 
       * # Arguments
       * * `owner_auth_id` Authorization from the owner who initiated the change
       * * `cdd_auth_id` Authorization from a CDD service provider
       **/
      acceptPrimaryKey: AugmentedSubmittable<(rotationAuthId: u64 | AnyNumber | Uint8Array, optionalCddAuthId: Option<u64> | null | object | string | Uint8Array) => SubmittableExtrinsic<ApiType>, [u64, Option<u64>]>;
      /**
       * Adds an authorization.
       **/
      addAuthorization: AugmentedSubmittable<(target: PolymeshPrimitivesSecondaryKeySignatory | { Identity: any } | { Account: any } | string | Uint8Array, data: PolymeshPrimitivesAuthorizationAuthorizationData | { AttestPrimaryKeyRotation: any } | { RotatePrimaryKey: any } | { TransferTicker: any } | { AddMultiSigSigner: any } | { TransferAssetOwnership: any } | { JoinIdentity: any } | { PortfolioCustody: any } | { BecomeAgent: any } | { AddRelayerPayingKey: any } | { RotatePrimaryKeyToSecondary: any } | string | Uint8Array, expiry: Option<u64> | null | object | string | Uint8Array) => SubmittableExtrinsic<ApiType>, [PolymeshPrimitivesSecondaryKeySignatory, PolymeshPrimitivesAuthorizationAuthorizationData, Option<u64>]>;
      /**
       * Adds a new claim record or edits an existing one.
       * 
       * Only called by did_issuer's secondary key.
       **/
      addClaim: AugmentedSubmittable<(target: PolymeshPrimitivesIdentityId | string | Uint8Array, claim: PolymeshPrimitivesIdentityClaimClaim | { Accredited: any } | { Affiliate: any } | { BuyLockup: any } | { SellLockup: any } | { CustomerDueDiligence: any } | { KnowYourCustomer: any } | { Jurisdiction: any } | { Exempted: any } | { Blocked: any } | { InvestorUniqueness: any } | { NoData: any } | { InvestorUniquenessV2: any } | string | Uint8Array, expiry: Option<u64> | null | object | string | Uint8Array) => SubmittableExtrinsic<ApiType>, [PolymeshPrimitivesIdentityId, PolymeshPrimitivesIdentityClaimClaim, Option<u64>]>;
      /**
       * Add `Claim::InvestorUniqueness` claim for a given target identity.
       * 
       * # <weight>
       * Weight of the this extrinsic is depend on the computation that used to validate
       * the proof of claim, which will be a constant independent of user inputs.
       * # </weight>
       * 
       * # Arguments
       * * origin - Who provides the claim to the user? In this case, it's the user's account id as the user provides.
       * * target - `IdentityId` to which the claim gets assigned.
       * * claim - `InvestorUniqueness` claim details.
       * * proof - To validate the self attestation.
       * * expiry - Expiry of claim.
       * 
       * # Errors
       * * `DidMustAlreadyExist` Target should already been a part of the ecosystem.
       * * `ClaimVariantNotAllowed` When origin trying to pass claim variant other than `InvestorUniqueness`.
       * * `ConfidentialScopeClaimNotAllowed` When issuer is different from target or CDD_ID is invalid for given user.
       * * `InvalidScopeClaim When proof is invalid.
       * * `InvalidCDDId` when you are not the owner of that CDD_ID.
       **/
      addInvestorUniquenessClaim: AugmentedSubmittable<(target: PolymeshPrimitivesIdentityId | string | Uint8Array, claim: PolymeshPrimitivesIdentityClaimClaim | { Accredited: any } | { Affiliate: any } | { BuyLockup: any } | { SellLockup: any } | { CustomerDueDiligence: any } | { KnowYourCustomer: any } | { Jurisdiction: any } | { Exempted: any } | { Blocked: any } | { InvestorUniqueness: any } | { NoData: any } | { InvestorUniquenessV2: any } | string | Uint8Array, proof: PolymeshPrimitivesInvestorZkproofDataV1InvestorZKProofData | { r?: any; s?: any } | string | Uint8Array, expiry: Option<u64> | null | object | string | Uint8Array) => SubmittableExtrinsic<ApiType>, [PolymeshPrimitivesIdentityId, PolymeshPrimitivesIdentityClaimClaim, PolymeshPrimitivesInvestorZkproofDataV1InvestorZKProofData, Option<u64>]>;
      addInvestorUniquenessClaimV2: AugmentedSubmittable<(target: PolymeshPrimitivesIdentityId | string | Uint8Array, scope: PolymeshPrimitivesIdentityClaimScope | { Identity: any } | { Ticker: any } | { Custom: any } | string | Uint8Array, claim: PolymeshPrimitivesIdentityClaimClaim | { Accredited: any } | { Affiliate: any } | { BuyLockup: any } | { SellLockup: any } | { CustomerDueDiligence: any } | { KnowYourCustomer: any } | { Jurisdiction: any } | { Exempted: any } | { Blocked: any } | { InvestorUniqueness: any } | { NoData: any } | { InvestorUniquenessV2: any } | string | Uint8Array, proof: ConfidentialIdentityClaimProofsScopeClaimProof | { proofScopeIdWellformed?: any; proofScopeIdCddIdMatch?: any; scopeId?: any } | string | Uint8Array, expiry: Option<u64> | null | object | string | Uint8Array) => SubmittableExtrinsic<ApiType>, [PolymeshPrimitivesIdentityId, PolymeshPrimitivesIdentityClaimScope, PolymeshPrimitivesIdentityClaimClaim, ConfidentialIdentityClaimProofsScopeClaimProof, Option<u64>]>;
      /**
       * Adds secondary keys to target identity `id`.
       * 
       * Keys are directly added to identity because each of them has an authorization.
       * 
       * # Arguments:
       * - `origin` which must be the primary key of the identity `id`.
       * - `id` to which new secondary keys will be added.
       * - `additional_keys` which includes secondary keys,
       * coupled with authorization data, to add to target identity.
       * 
       * # Errors
       * - Can only called by primary key owner.
       * - Keys should be able to linked to any identity.
       **/
      addSecondaryKeysWithAuthorization: AugmentedSubmittable<(additionalKeys: Vec<PolymeshCommonUtilitiesIdentitySecondaryKeyWithAuth> | (PolymeshCommonUtilitiesIdentitySecondaryKeyWithAuth | { secondaryKey?: any; authSignature?: any } | string | Uint8Array)[], expiresAt: u64 | AnyNumber | Uint8Array) => SubmittableExtrinsic<ApiType>, [Vec<PolymeshCommonUtilitiesIdentitySecondaryKeyWithAuth>, u64]>;
      /**
       * Deprecated. Use `add_secondary_keys_with_authorization` instead.
       **/
      addSecondaryKeysWithAuthorizationOld: AugmentedSubmittable<(additionalKeys: Vec<PolymeshCommonUtilitiesIdentitySecondaryKeyWithAuthV1> | (PolymeshCommonUtilitiesIdentitySecondaryKeyWithAuthV1 | { secondaryKey?: any; authSignature?: any } | string | Uint8Array)[], expiresAt: u64 | AnyNumber | Uint8Array) => SubmittableExtrinsic<ApiType>, [Vec<PolymeshCommonUtilitiesIdentitySecondaryKeyWithAuthV1>, u64]>;
      /**
       * Register `target_account` with a new Identity.
       * 
       * # Failure
       * - `origin` has to be a active CDD provider. Inactive CDD providers cannot add new
       * claims.
       * - `target_account` (primary key of the new Identity) can be linked to just one and only
       * one identity.
       * - External secondary keys can be linked to just one identity.
       * 
       * # Weight
       * `7_000_000_000 + 600_000 * secondary_keys.len()`
       **/
      cddRegisterDid: AugmentedSubmittable<(targetAccount: AccountId32 | string | Uint8Array, secondaryKeys: Vec<PolymeshPrimitivesSecondaryKey> | (PolymeshPrimitivesSecondaryKey | { key?: any; permissions?: any } | string | Uint8Array)[]) => SubmittableExtrinsic<ApiType>, [AccountId32, Vec<PolymeshPrimitivesSecondaryKey>]>;
      /**
       * Set if CDD authorization is required for updating primary key of an identity.
       * Callable via root (governance)
       * 
       * # Arguments
       * * `auth_required` CDD Authorization required or not
       **/
      changeCddRequirementForMkRotation: AugmentedSubmittable<(authRequired: bool | boolean | Uint8Array) => SubmittableExtrinsic<ApiType>, [bool]>;
      /**
       * It disables all secondary keys at `did` identity.
       * 
       * # Errors
       * 
       **/
      freezeSecondaryKeys: AugmentedSubmittable<() => SubmittableExtrinsic<ApiType>, []>;
      /**
       * Assuming this is executed by the GC voting majority, adds a new cdd claim record.
       **/
      gcAddCddClaim: AugmentedSubmittable<(target: PolymeshPrimitivesIdentityId | string | Uint8Array) => SubmittableExtrinsic<ApiType>, [PolymeshPrimitivesIdentityId]>;
      /**
       * Assuming this is executed by the GC voting majority, removes an existing cdd claim record.
       **/
      gcRevokeCddClaim: AugmentedSubmittable<(target: PolymeshPrimitivesIdentityId | string | Uint8Array) => SubmittableExtrinsic<ApiType>, [PolymeshPrimitivesIdentityId]>;
      /**
       * Invalidates any claim generated by `cdd` from `disable_from` timestamps.
       * 
       * You can also define an expiration time,
       * which will invalidate all claims generated by that `cdd` and remove it as CDD member group.
       **/
      invalidateCddClaims: AugmentedSubmittable<(cdd: PolymeshPrimitivesIdentityId | string | Uint8Array, disableFrom: u64 | AnyNumber | Uint8Array, expiry: Option<u64> | null | object | string | Uint8Array) => SubmittableExtrinsic<ApiType>, [PolymeshPrimitivesIdentityId, u64, Option<u64>]>;
      /**
       * Join an identity as a secondary key.
       **/
      joinIdentityAsKey: AugmentedSubmittable<(authId: u64 | AnyNumber | Uint8Array) => SubmittableExtrinsic<ApiType>, [u64]>;
      /**
       * Leave the secondary key's identity.
       **/
      leaveIdentityAsKey: AugmentedSubmittable<() => SubmittableExtrinsic<ApiType>, []>;
      /**
       * Placeholder for removed `legacy_set_permission_to_signer`.
       **/
      placeholderLegacySetPermissionToSigner: AugmentedSubmittable<() => SubmittableExtrinsic<ApiType>, []>;
      /**
       * Removes an authorization.
       * _auth_issuer_pays determines whether the issuer of the authorisation pays the transaction fee
       **/
      removeAuthorization: AugmentedSubmittable<(target: PolymeshPrimitivesSecondaryKeySignatory | { Identity: any } | { Account: any } | string | Uint8Array, authId: u64 | AnyNumber | Uint8Array, authIssuerPays: bool | boolean | Uint8Array) => SubmittableExtrinsic<ApiType>, [PolymeshPrimitivesSecondaryKeySignatory, u64, bool]>;
      /**
       * Removes specified secondary keys of a DID if present.
       * 
       * # Errors
       * 
       * The extrinsic can only called by primary key owner.
       **/
      removeSecondaryKeys: AugmentedSubmittable<(keysToRemove: Vec<AccountId32> | (AccountId32 | string | Uint8Array)[]) => SubmittableExtrinsic<ApiType>, [Vec<AccountId32>]>;
      /**
       * Deprecated. Use `remove_secondary_keys` instead.
       **/
      removeSecondaryKeysOld: AugmentedSubmittable<(keysToRemove: Vec<PolymeshPrimitivesSecondaryKeySignatory> | (PolymeshPrimitivesSecondaryKeySignatory | { Identity: any } | { Account: any } | string | Uint8Array)[]) => SubmittableExtrinsic<ApiType>, [Vec<PolymeshPrimitivesSecondaryKeySignatory>]>;
      /**
       * Marks the specified claim as revoked.
       **/
      revokeClaim: AugmentedSubmittable<(target: PolymeshPrimitivesIdentityId | string | Uint8Array, claim: PolymeshPrimitivesIdentityClaimClaim | { Accredited: any } | { Affiliate: any } | { BuyLockup: any } | { SellLockup: any } | { CustomerDueDiligence: any } | { KnowYourCustomer: any } | { Jurisdiction: any } | { Exempted: any } | { Blocked: any } | { InvestorUniqueness: any } | { NoData: any } | { InvestorUniquenessV2: any } | string | Uint8Array) => SubmittableExtrinsic<ApiType>, [PolymeshPrimitivesIdentityId, PolymeshPrimitivesIdentityClaimClaim]>;
      /**
       * Revokes a specific claim using its [Claim Unique Index](/pallet_identity/index.html#claim-unique-index) composed by `target`,
       * `claim_type`, and `scope`.
       * 
       * Please note that `origin` must be the issuer of the target claim.
       * 
       * # Errors
       * - `TargetHasNonZeroBalanceAtScopeId` when you try to revoke a `InvestorUniqueness*`
       * claim, and `target` identity still have any balance on the given `scope`.
       **/
      revokeClaimByIndex: AugmentedSubmittable<(target: PolymeshPrimitivesIdentityId | string | Uint8Array, claimType: PolymeshPrimitivesIdentityClaimClaimType | 'Accredited' | 'Affiliate' | 'BuyLockup' | 'SellLockup' | 'CustomerDueDiligence' | 'KnowYourCustomer' | 'Jurisdiction' | 'Exempted' | 'Blocked' | 'InvestorUniqueness' | 'NoType' | 'InvestorUniquenessV2' | number | Uint8Array, scope: Option<PolymeshPrimitivesIdentityClaimScope> | null | object | string | Uint8Array) => SubmittableExtrinsic<ApiType>, [PolymeshPrimitivesIdentityId, PolymeshPrimitivesIdentityClaimClaimType, Option<PolymeshPrimitivesIdentityClaimScope>]>;
      /**
       * Call this with the new primary key. By invoking this method, caller accepts authorization
       * to become the new primary key of the issuing identity. If a CDD service provider approved
       * this change, (or this is not required), primary key of the DID is updated.
       * 
       * The caller (new primary key) must be either a secondary key of the issuing identity, or
       * unlinked to any identity.
       * 
       * Differs from accept_primary_key in that it will leave the old primary key as a secondary
       * key with the permissions specified in the corresponding RotatePrimaryKeyToSecondary authorization
       * instead of unlinking the old primary key.
       * 
       * # Arguments
       * * `owner_auth_id` Authorization from the owner who initiated the change
       * * `cdd_auth_id` Authorization from a CDD service provider
       **/
      rotatePrimaryKeyToSecondary: AugmentedSubmittable<(authId: u64 | AnyNumber | Uint8Array, optionalCddAuthId: Option<u64> | null | object | string | Uint8Array) => SubmittableExtrinsic<ApiType>, [u64, Option<u64>]>;
      /**
       * Deprecated. Use `set_secondary_key_permissions` instead.
       **/
      setPermissionToSigner: AugmentedSubmittable<(key: PolymeshPrimitivesSecondaryKeySignatory | { Identity: any } | { Account: any } | string | Uint8Array, perms: PolymeshPrimitivesSecondaryKeyPermissions | { asset?: any; extrinsic?: any; portfolio?: any } | string | Uint8Array) => SubmittableExtrinsic<ApiType>, [PolymeshPrimitivesSecondaryKeySignatory, PolymeshPrimitivesSecondaryKeyPermissions]>;
      /**
       * Sets permissions for an specific `target_key` key.
       * 
       * Only the primary key of an identity is able to set secondary key permissions.
       **/
      setSecondaryKeyPermissions: AugmentedSubmittable<(key: AccountId32 | string | Uint8Array, perms: PolymeshPrimitivesSecondaryKeyPermissions | { asset?: any; extrinsic?: any; portfolio?: any } | string | Uint8Array) => SubmittableExtrinsic<ApiType>, [AccountId32, PolymeshPrimitivesSecondaryKeyPermissions]>;
      /**
       * Re-enables all secondary keys of the caller's identity.
       **/
      unfreezeSecondaryKeys: AugmentedSubmittable<() => SubmittableExtrinsic<ApiType>, []>;
      /**
       * Generic tx
       **/
      [key: string]: SubmittableExtrinsicFunction<ApiType>;
    };
    imOnline: {
      /**
       * # <weight>
       * - Complexity: `O(K + E)` where K is length of `Keys` (heartbeat.validators_len) and E is
       * length of `heartbeat.network_state.external_address`
       * - `O(K)`: decoding of length `K`
       * - `O(E)`: decoding/encoding of length `E`
       * - DbReads: pallet_session `Validators`, pallet_session `CurrentIndex`, `Keys`,
       * `ReceivedHeartbeats`
       * - DbWrites: `ReceivedHeartbeats`
       * # </weight>
       **/
      heartbeat: AugmentedSubmittable<(heartbeat: PalletImOnlineHeartbeat | { blockNumber?: any; networkState?: any; sessionIndex?: any; authorityIndex?: any; validatorsLen?: any } | string | Uint8Array, signature: PalletImOnlineSr25519AppSr25519Signature | string | Uint8Array) => SubmittableExtrinsic<ApiType>, [PalletImOnlineHeartbeat, PalletImOnlineSr25519AppSr25519Signature]>;
      /**
       * Generic tx
       **/
      [key: string]: SubmittableExtrinsicFunction<ApiType>;
    };
    indices: {
      /**
       * Assign an previously unassigned index.
       * 
       * Payment: `Deposit` is reserved from the sender account.
       * 
       * The dispatch origin for this call must be _Signed_.
       * 
       * - `index`: the index to be claimed. This must not be in use.
       * 
       * Emits `IndexAssigned` if successful.
       * 
       * # <weight>
       * - `O(1)`.
       * - One storage mutation (codec `O(1)`).
       * - One reserve operation.
       * - One event.
       * -------------------
       * - DB Weight: 1 Read/Write (Accounts)
       * # </weight>
       **/
      claim: AugmentedSubmittable<(index: u32 | AnyNumber | Uint8Array) => SubmittableExtrinsic<ApiType>, [u32]>;
      /**
       * Force an index to an account. This doesn't require a deposit. If the index is already
       * held, then any deposit is reimbursed to its current owner.
       * 
       * The dispatch origin for this call must be _Root_.
       * 
       * - `index`: the index to be (re-)assigned.
       * - `new`: the new owner of the index. This function is a no-op if it is equal to sender.
       * - `freeze`: if set to `true`, will freeze the index so it cannot be transferred.
       * 
       * Emits `IndexAssigned` if successful.
       * 
       * # <weight>
       * - `O(1)`.
       * - One storage mutation (codec `O(1)`).
       * - Up to one reserve operation.
       * - One event.
       * -------------------
       * - DB Weight:
       * - Reads: Indices Accounts, System Account (original owner)
       * - Writes: Indices Accounts, System Account (original owner)
       * # </weight>
       **/
      forceTransfer: AugmentedSubmittable<(updated: AccountId32 | string | Uint8Array, index: u32 | AnyNumber | Uint8Array, freeze: bool | boolean | Uint8Array) => SubmittableExtrinsic<ApiType>, [AccountId32, u32, bool]>;
      /**
       * Free up an index owned by the sender.
       * 
       * Payment: Any previous deposit placed for the index is unreserved in the sender account.
       * 
       * The dispatch origin for this call must be _Signed_ and the sender must own the index.
       * 
       * - `index`: the index to be freed. This must be owned by the sender.
       * 
       * Emits `IndexFreed` if successful.
       * 
       * # <weight>
       * - `O(1)`.
       * - One storage mutation (codec `O(1)`).
       * - One reserve operation.
       * - One event.
       * -------------------
       * - DB Weight: 1 Read/Write (Accounts)
       * # </weight>
       **/
      free: AugmentedSubmittable<(index: u32 | AnyNumber | Uint8Array) => SubmittableExtrinsic<ApiType>, [u32]>;
      /**
       * Freeze an index so it will always point to the sender account. This consumes the
       * deposit.
       * 
       * The dispatch origin for this call must be _Signed_ and the signing account must have a
       * non-frozen account `index`.
       * 
       * - `index`: the index to be frozen in place.
       * 
       * Emits `IndexFrozen` if successful.
       * 
       * # <weight>
       * - `O(1)`.
       * - One storage mutation (codec `O(1)`).
       * - Up to one slash operation.
       * - One event.
       * -------------------
       * - DB Weight: 1 Read/Write (Accounts)
       * # </weight>
       **/
      freeze: AugmentedSubmittable<(index: u32 | AnyNumber | Uint8Array) => SubmittableExtrinsic<ApiType>, [u32]>;
      /**
       * Assign an index already owned by the sender to another account. The balance reservation
       * is effectively transferred to the new account.
       * 
       * The dispatch origin for this call must be _Signed_.
       * 
       * - `index`: the index to be re-assigned. This must be owned by the sender.
       * - `new`: the new owner of the index. This function is a no-op if it is equal to sender.
       * 
       * Emits `IndexAssigned` if successful.
       * 
       * # <weight>
       * - `O(1)`.
       * - One storage mutation (codec `O(1)`).
       * - One transfer operation.
       * - One event.
       * -------------------
       * - DB Weight:
       * - Reads: Indices Accounts, System Account (recipient)
       * - Writes: Indices Accounts, System Account (recipient)
       * # </weight>
       **/
      transfer: AugmentedSubmittable<(updated: AccountId32 | string | Uint8Array, index: u32 | AnyNumber | Uint8Array) => SubmittableExtrinsic<ApiType>, [AccountId32, u32]>;
      /**
       * Generic tx
       **/
      [key: string]: SubmittableExtrinsicFunction<ApiType>;
    };
    multiSig: {
      /**
       * Accepts a multisig signer authorization given to signer's identity.
       * 
       * # Arguments
       * * `auth_id` - Auth id of the authorization.
       **/
      acceptMultisigSignerAsIdentity: AugmentedSubmittable<(authId: u64 | AnyNumber | Uint8Array) => SubmittableExtrinsic<ApiType>, [u64]>;
      /**
       * Accepts a multisig signer authorization given to signer's key (AccountId).
       * 
       * # Arguments
       * * `auth_id` - Auth id of the authorization.
       **/
      acceptMultisigSignerAsKey: AugmentedSubmittable<(authId: u64 | AnyNumber | Uint8Array) => SubmittableExtrinsic<ApiType>, [u64]>;
      /**
       * Adds a signer to the multisig. This must be called by the multisig itself.
       * 
       * # Arguments
       * * `signer` - Signatory to add.
       **/
      addMultisigSigner: AugmentedSubmittable<(signer: PolymeshPrimitivesSecondaryKeySignatory | { Identity: any } | { Account: any } | string | Uint8Array) => SubmittableExtrinsic<ApiType>, [PolymeshPrimitivesSecondaryKeySignatory]>;
      /**
       * Adds a signer to the multisig. This must be called by the creator identity of the
       * multisig.
       * 
       * # Arguments
       * * `multisig` - Address of the multi sig
       * * `signers` - Signatories to add.
       * 
       * # Weight
       * `900_000_000 + 3_000_000 * signers.len()`
       **/
      addMultisigSignersViaCreator: AugmentedSubmittable<(multisig: AccountId32 | string | Uint8Array, signers: Vec<PolymeshPrimitivesSecondaryKeySignatory> | (PolymeshPrimitivesSecondaryKeySignatory | { Identity: any } | { Account: any } | string | Uint8Array)[]) => SubmittableExtrinsic<ApiType>, [AccountId32, Vec<PolymeshPrimitivesSecondaryKeySignatory>]>;
      /**
       * Approves a multisig proposal using the caller's identity.
       * 
       * # Arguments
       * * `multisig` - MultiSig address.
       * * `proposal_id` - Proposal id to approve.
       * If quorum is reached, the proposal will be immediately executed.
       **/
      approveAsIdentity: AugmentedSubmittable<(multisig: AccountId32 | string | Uint8Array, proposalId: u64 | AnyNumber | Uint8Array) => SubmittableExtrinsic<ApiType>, [AccountId32, u64]>;
      /**
       * Approves a multisig proposal using the caller's secondary key (`AccountId`).
       * 
       * # Arguments
       * * `multisig` - MultiSig address.
       * * `proposal_id` - Proposal id to approve.
       * If quorum is reached, the proposal will be immediately executed.
       **/
      approveAsKey: AugmentedSubmittable<(multisig: AccountId32 | string | Uint8Array, proposalId: u64 | AnyNumber | Uint8Array) => SubmittableExtrinsic<ApiType>, [AccountId32, u64]>;
      /**
       * Changes the number of signatures required by a multisig. This must be called by the
       * multisig itself.
       * 
       * # Arguments
       * * `sigs_required` - New number of required signatures.
       **/
      changeSigsRequired: AugmentedSubmittable<(sigsRequired: u64 | AnyNumber | Uint8Array) => SubmittableExtrinsic<ApiType>, [u64]>;
      /**
       * Creates a multisig
       * 
       * # Arguments
       * * `signers` - Signers of the multisig (They need to accept authorization before they are actually added).
       * * `sigs_required` - Number of sigs required to process a multi-sig tx.
       **/
      createMultisig: AugmentedSubmittable<(signers: Vec<PolymeshPrimitivesSecondaryKeySignatory> | (PolymeshPrimitivesSecondaryKeySignatory | { Identity: any } | { Account: any } | string | Uint8Array)[], sigsRequired: u64 | AnyNumber | Uint8Array) => SubmittableExtrinsic<ApiType>, [Vec<PolymeshPrimitivesSecondaryKeySignatory>, u64]>;
      /**
       * Creates a multisig proposal if it hasn't been created or approves it if it has.
       * 
       * # Arguments
       * * `multisig` - MultiSig address.
       * * `proposal` - Proposal to be voted on.
       * * `expiry` - Optional proposal expiry time.
       * * `auto_close` - Close proposal on receiving enough reject votes.
       * If this is 1 out of `m` multisig, the proposal will be immediately executed.
       **/
      createOrApproveProposalAsIdentity: AugmentedSubmittable<(multisig: AccountId32 | string | Uint8Array, proposal: Call | { callIndex?: any; args?: any } | string | Uint8Array, expiry: Option<u64> | null | object | string | Uint8Array, autoClose: bool | boolean | Uint8Array) => SubmittableExtrinsic<ApiType>, [AccountId32, Call, Option<u64>, bool]>;
      /**
       * Creates a multisig proposal if it hasn't been created or approves it if it has.
       * 
       * # Arguments
       * * `multisig` - MultiSig address.
       * * `proposal` - Proposal to be voted on.
       * * `expiry` - Optional proposal expiry time.
       * * `auto_close` - Close proposal on receiving enough reject votes.
       * If this is 1 out of `m` multisig, the proposal will be immediately executed.
       **/
      createOrApproveProposalAsKey: AugmentedSubmittable<(multisig: AccountId32 | string | Uint8Array, proposal: Call | { callIndex?: any; args?: any } | string | Uint8Array, expiry: Option<u64> | null | object | string | Uint8Array, autoClose: bool | boolean | Uint8Array) => SubmittableExtrinsic<ApiType>, [AccountId32, Call, Option<u64>, bool]>;
      /**
       * Creates a multisig proposal
       * 
       * # Arguments
       * * `multisig` - MultiSig address.
       * * `proposal` - Proposal to be voted on.
       * * `expiry` - Optional proposal expiry time.
       * * `auto_close` - Close proposal on receiving enough reject votes.
       * If this is 1 out of `m` multisig, the proposal will be immediately executed.
       **/
      createProposalAsIdentity: AugmentedSubmittable<(multisig: AccountId32 | string | Uint8Array, proposal: Call | { callIndex?: any; args?: any } | string | Uint8Array, expiry: Option<u64> | null | object | string | Uint8Array, autoClose: bool | boolean | Uint8Array) => SubmittableExtrinsic<ApiType>, [AccountId32, Call, Option<u64>, bool]>;
      /**
       * Creates a multisig proposal
       * 
       * # Arguments
       * * `multisig` - MultiSig address.
       * * `proposal` - Proposal to be voted on.
       * * `expiry` - Optional proposal expiry time.
       * * `auto_close` - Close proposal on receiving enough reject votes.
       * If this is 1 out of `m` multisig, the proposal will be immediately executed.
       **/
      createProposalAsKey: AugmentedSubmittable<(multisig: AccountId32 | string | Uint8Array, proposal: Call | { callIndex?: any; args?: any } | string | Uint8Array, expiry: Option<u64> | null | object | string | Uint8Array, autoClose: bool | boolean | Uint8Array) => SubmittableExtrinsic<ApiType>, [AccountId32, Call, Option<u64>, bool]>;
      /**
       * Root callable extrinsic, used as an internal call for executing scheduled multisig proposal.
       **/
      executeScheduledProposal: AugmentedSubmittable<(multisig: AccountId32 | string | Uint8Array, proposalId: u64 | AnyNumber | Uint8Array, multisigDid: PolymeshPrimitivesIdentityId | string | Uint8Array, proposalWeight: u64 | AnyNumber | Uint8Array) => SubmittableExtrinsic<ApiType>, [AccountId32, u64, PolymeshPrimitivesIdentityId, u64]>;
      /**
       * Adds a multisig as the primary key of the current did if the current DID is the creator
       * of the multisig.
       * 
       * # Arguments
       * * `multi_sig` - multi sig address
       **/
      makeMultisigPrimary: AugmentedSubmittable<(multisig: AccountId32 | string | Uint8Array, optionalCddAuthId: Option<u64> | null | object | string | Uint8Array) => SubmittableExtrinsic<ApiType>, [AccountId32, Option<u64>]>;
      /**
       * Adds a multisig as a secondary key of current did if the current did is the creator of the
       * multisig.
       * 
       * # Arguments
       * * `multisig` - multi sig address
       **/
      makeMultisigSecondary: AugmentedSubmittable<(multisig: AccountId32 | string | Uint8Array) => SubmittableExtrinsic<ApiType>, [AccountId32]>;
      /**
       * Rejects a multisig proposal using the caller's identity.
       * 
       * # Arguments
       * * `multisig` - MultiSig address.
       * * `proposal_id` - Proposal id to reject.
       * If quorum is reached, the proposal will be immediately executed.
       **/
      rejectAsIdentity: AugmentedSubmittable<(multisig: AccountId32 | string | Uint8Array, proposalId: u64 | AnyNumber | Uint8Array) => SubmittableExtrinsic<ApiType>, [AccountId32, u64]>;
      /**
       * Rejects a multisig proposal using the caller's secondary key (`AccountId`).
       * 
       * # Arguments
       * * `multisig` - MultiSig address.
       * * `proposal_id` - Proposal id to reject.
       * If quorum is reached, the proposal will be immediately executed.
       **/
      rejectAsKey: AugmentedSubmittable<(multisig: AccountId32 | string | Uint8Array, proposalId: u64 | AnyNumber | Uint8Array) => SubmittableExtrinsic<ApiType>, [AccountId32, u64]>;
      /**
       * Removes a signer from the multisig. This must be called by the multisig itself.
       * 
       * # Arguments
       * * `signer` - Signatory to remove.
       **/
      removeMultisigSigner: AugmentedSubmittable<(signer: PolymeshPrimitivesSecondaryKeySignatory | { Identity: any } | { Account: any } | string | Uint8Array) => SubmittableExtrinsic<ApiType>, [PolymeshPrimitivesSecondaryKeySignatory]>;
      /**
       * Removes a signer from the multisig.
       * This must be called by the creator identity of the multisig.
       * 
       * # Arguments
       * * `multisig` - Address of the multisig.
       * * `signers` - Signatories to remove.
       * 
       * # Weight
       * `900_000_000 + 3_000_000 * signers.len()`
       **/
      removeMultisigSignersViaCreator: AugmentedSubmittable<(multisig: AccountId32 | string | Uint8Array, signers: Vec<PolymeshPrimitivesSecondaryKeySignatory> | (PolymeshPrimitivesSecondaryKeySignatory | { Identity: any } | { Account: any } | string | Uint8Array)[]) => SubmittableExtrinsic<ApiType>, [AccountId32, Vec<PolymeshPrimitivesSecondaryKeySignatory>]>;
      /**
       * Generic tx
       **/
      [key: string]: SubmittableExtrinsicFunction<ApiType>;
    };
    pips: {
      /**
       * Approves the pending committee PIP given by the `id`.
       * 
       * # Errors
       * * `BadOrigin` unless a GC voting majority executes this function.
       * * `NoSuchProposal` if the PIP with `id` doesn't exist.
       * * `IncorrectProposalState` if the proposal isn't pending.
       * * `NotByCommittee` if the proposal isn't by a committee.
       **/
      approveCommitteeProposal: AugmentedSubmittable<(id: u32 | AnyNumber | Uint8Array) => SubmittableExtrinsic<ApiType>, [u32]>;
      /**
       * Clears the snapshot and emits the event `SnapshotCleared`.
       * 
       * # Errors
       * * `NotACommitteeMember` - triggered when a non-GC-member executes the function.
       **/
      clearSnapshot: AugmentedSubmittable<() => SubmittableExtrinsic<ApiType>, []>;
      /**
       * Enacts `results` for the PIPs in the snapshot queue.
       * The snapshot will be available for further enactments until it is cleared.
       * 
       * The `results` are encoded a list of `(id, result)` where `result` is applied to `id`.
       * Note that the snapshot priority queue is encoded with the *lowest priority first*.
       * so `results = [(id, Approve)]` will approve `SnapshotQueue[SnapshotQueue.len() - 1]`.
       * 
       * # Errors
       * * `BadOrigin` - unless a GC voting majority executes this function.
       * * `CannotSkipPip` - a given PIP has already been skipped too many times.
       * * `SnapshotResultTooLarge` - on len(results) > len(snapshot_queue).
       * * `SnapshotIdMismatch` - if:
       * ```text
       * ∃ (i ∈ 0..SnapshotQueue.len()).
       * results[i].0 ≠ SnapshotQueue[SnapshotQueue.len() - i].id
       * ```
       * This is protects against clearing queue while GC is voting.
       **/
      enactSnapshotResults: AugmentedSubmittable<(results: Vec<ITuple<[u32, PalletPipsSnapshotResult]>> | ([u32 | AnyNumber | Uint8Array, PalletPipsSnapshotResult | 'Approve' | 'Reject' | 'Skip' | number | Uint8Array])[]) => SubmittableExtrinsic<ApiType>, [Vec<ITuple<[u32, PalletPipsSnapshotResult]>>]>;
      /**
       * Internal dispatchable that handles execution of a PIP.
       **/
      executeScheduledPip: AugmentedSubmittable<(id: u32 | AnyNumber | Uint8Array) => SubmittableExtrinsic<ApiType>, [u32]>;
      /**
       * Internal dispatchable that handles expiration of a PIP.
       **/
      expireScheduledPip: AugmentedSubmittable<(did: PolymeshPrimitivesIdentityId | string | Uint8Array, id: u32 | AnyNumber | Uint8Array) => SubmittableExtrinsic<ApiType>, [PolymeshPrimitivesIdentityId, u32]>;
      /**
       * A network member creates a PIP by submitting a dispatchable which
       * changes the network in someway. A minimum deposit is required to open a new proposal.
       * 
       * # Arguments
       * * `proposer` is either a signing key or committee.
       * Used to understand whether this is a committee proposal and verified against `origin`.
       * * `proposal` a dispatchable call
       * * `deposit` minimum deposit value, which is ignored if `proposer` is a committee.
       * * `url` a link to a website for proposal discussion
       **/
      propose: AugmentedSubmittable<(proposal: Call | { callIndex?: any; args?: any } | string | Uint8Array, deposit: u128 | AnyNumber | Uint8Array, url: Option<Bytes> | null | object | string | Uint8Array, description: Option<Bytes> | null | object | string | Uint8Array) => SubmittableExtrinsic<ApiType>, [Call, u128, Option<Bytes>, Option<Bytes>]>;
      /**
       * Prune the PIP given by the `id`, refunding any funds not already refunded.
       * The PIP may not be active
       * 
       * This function is intended for storage garbage collection purposes.
       * 
       * # Errors
       * * `BadOrigin` unless a GC voting majority executes this function.
       * * `NoSuchProposal` if the PIP with `id` doesn't exist.
       * * `IncorrectProposalState` if the proposal is active.
       **/
      pruneProposal: AugmentedSubmittable<(id: u32 | AnyNumber | Uint8Array) => SubmittableExtrinsic<ApiType>, [u32]>;
      /**
       * Rejects the PIP given by the `id`, refunding any bonded funds,
       * assuming it hasn't been cancelled or executed.
       * Note that proposals scheduled-for-execution can also be rejected.
       * 
       * # Errors
       * * `BadOrigin` unless a GC voting majority executes this function.
       * * `NoSuchProposal` if the PIP with `id` doesn't exist.
       * * `IncorrectProposalState` if the proposal was cancelled or executed.
       **/
      rejectProposal: AugmentedSubmittable<(id: u32 | AnyNumber | Uint8Array) => SubmittableExtrinsic<ApiType>, [u32]>;
      /**
       * Updates the execution schedule of the PIP given by `id`.
       * 
       * # Arguments
       * * `until` defines the future block where the enactment period will finished.
       * `None` value means that enactment period is going to finish in the next block.
       * 
       * # Errors
       * * `RescheduleNotByReleaseCoordinator` unless triggered by release coordinator.
       * * `IncorrectProposalState` unless the proposal was in a scheduled state.
       **/
      rescheduleExecution: AugmentedSubmittable<(id: u32 | AnyNumber | Uint8Array, until: Option<u32> | null | object | string | Uint8Array) => SubmittableExtrinsic<ApiType>, [u32, Option<u32>]>;
      /**
       * Change the maximum number of active PIPs before community members cannot propose anything.
       * Can only be called by root.
       * 
       * # Arguments
       * * `limit` of concurrent active PIPs.
       **/
      setActivePipLimit: AugmentedSubmittable<(limit: u32 | AnyNumber | Uint8Array) => SubmittableExtrinsic<ApiType>, [u32]>;
      /**
       * Change the default enactment period.
       * Can only be called by root.
       * 
       * # Arguments
       * * `duration` the new default enactment period it takes for a scheduled PIP to be executed.
       **/
      setDefaultEnactmentPeriod: AugmentedSubmittable<(duration: u32 | AnyNumber | Uint8Array) => SubmittableExtrinsic<ApiType>, [u32]>;
      /**
       * Change the maximum skip count (`max_pip_skip_count`).
       * Can only be called by root.
       * 
       * # Arguments
       * * `max` skips before a PIP cannot be skipped by GC anymore.
       **/
      setMaxPipSkipCount: AugmentedSubmittable<(max: u8 | AnyNumber | Uint8Array) => SubmittableExtrinsic<ApiType>, [u8]>;
      /**
       * Change the minimum proposal deposit amount required to start a proposal.
       * Can only be called by root.
       * 
       * # Arguments
       * * `deposit` the new min deposit required to start a proposal
       **/
      setMinProposalDeposit: AugmentedSubmittable<(deposit: u128 | AnyNumber | Uint8Array) => SubmittableExtrinsic<ApiType>, [u128]>;
      /**
       * Change the amount of blocks after which a pending PIP is expired.
       * If `expiry` is `None` then PIPs never expire.
       * Can only be called by root.
       * 
       * # Arguments
       * * `expiry` the block-time it takes for a still-`Pending` PIP to expire.
       **/
      setPendingPipExpiry: AugmentedSubmittable<(expiry: PolymeshCommonUtilitiesMaybeBlock | { Some: any } | { None: any } | string | Uint8Array) => SubmittableExtrinsic<ApiType>, [PolymeshCommonUtilitiesMaybeBlock]>;
      /**
       * Change whether completed PIPs are pruned.
       * Can only be called by root.
       * 
       * # Arguments
       * * `prune` specifies whether completed PIPs should be pruned.
       **/
      setPruneHistoricalPips: AugmentedSubmittable<(prune: bool | boolean | Uint8Array) => SubmittableExtrinsic<ApiType>, [bool]>;
      /**
       * Takes a new snapshot of the current list of active && pending PIPs.
       * The PIPs are then sorted into a priority queue based on each PIP's weight.
       * 
       * # Errors
       * * `NotACommitteeMember` - triggered when a non-GC-member executes the function.
       **/
      snapshot: AugmentedSubmittable<() => SubmittableExtrinsic<ApiType>, []>;
      /**
       * Vote either in favor (`aye_or_nay` == true) or against a PIP with `id`.
       * The "convinction" or strength of the vote is given by `deposit`, which is reserved.
       * 
       * Note that `vote` is *not* additive.
       * That is, `vote(id, true, 50)` followed by `vote(id, true, 40)`
       * will first reserve `50` and then refund `50 - 10`, ending up with `40` in deposit.
       * To add atop of existing votes, you'll need `existing_deposit + addition`.
       * 
       * # Arguments
       * * `id`, proposal id
       * * `aye_or_nay`, a bool representing for or against vote
       * * `deposit`, the "conviction" with which the vote is made.
       * 
       * # Errors
       * * `NoSuchProposal` if `id` doesn't reference a valid PIP.
       * * `NotFromCommunity` if proposal was made by a committee.
       * * `IncorrectProposalState` if PIP isn't pending.
       * * `InsufficientDeposit` if `origin` cannot reserve `deposit - old_deposit`.
       **/
      vote: AugmentedSubmittable<(id: u32 | AnyNumber | Uint8Array, ayeOrNay: bool | boolean | Uint8Array, deposit: u128 | AnyNumber | Uint8Array) => SubmittableExtrinsic<ApiType>, [u32, bool, u128]>;
      /**
       * Generic tx
       **/
      [key: string]: SubmittableExtrinsicFunction<ApiType>;
    };
    polymeshCommittee: {
      /**
       * Changes the time after which a proposal expires.
       * 
       * # Arguments
       * * `expiry` - The new expiry time.
       **/
      setExpiresAfter: AugmentedSubmittable<(expiry: PolymeshCommonUtilitiesMaybeBlock | { Some: any } | { None: any } | string | Uint8Array) => SubmittableExtrinsic<ApiType>, [PolymeshCommonUtilitiesMaybeBlock]>;
      /**
       * Changes the release coordinator.
       * 
       * # Arguments
       * * `id` - The DID of the new release coordinator.
       * 
       * # Errors
       * * `NotAMember`, If the new coordinator `id` is not part of the committee.
       **/
      setReleaseCoordinator: AugmentedSubmittable<(id: PolymeshPrimitivesIdentityId | string | Uint8Array) => SubmittableExtrinsic<ApiType>, [PolymeshPrimitivesIdentityId]>;
      /**
       * Change the vote threshold the determines the winning proposal.
       * For e.g., for a simple majority use (1, 2) which represents the in-equation ">= 1/2".
       * 
       * # Arguments
       * * `n` - Numerator of the fraction representing vote threshold.
       * * `d` - Denominator of the fraction representing vote threshold.
       **/
      setVoteThreshold: AugmentedSubmittable<(n: u32 | AnyNumber | Uint8Array, d: u32 | AnyNumber | Uint8Array) => SubmittableExtrinsic<ApiType>, [u32, u32]>;
      /**
       * Votes `approve`ingly (or not, if `false`)
       * on an existing `proposal` given by its hash, `index`.
       * 
       * # Arguments
       * * `proposal` - A hash of the proposal to be voted on.
       * * `index` - The proposal index.
       * * `approve` - If `true` than this is a `for` vote, and `against` otherwise.
       * 
       * # Errors
       * * `NotAMember`, if the `origin` is not a member of this committee.
       **/
      vote: AugmentedSubmittable<(proposal: H256 | string | Uint8Array, index: u32 | AnyNumber | Uint8Array, approve: bool | boolean | Uint8Array) => SubmittableExtrinsic<ApiType>, [H256, u32, bool]>;
      /**
       * Proposes to the committee that `call` should be executed in its name.
       * Alternatively, if the hash of `call` has already been recorded, i.e., already proposed,
       * then this call counts as a vote, i.e., as if `vote_by_hash` was called.
       * 
       * # Weight
       * 
       * The weight of this dispatchable is that of `call` as well as the complexity
       * for recording the vote itself.
       * 
       * # Arguments
       * * `approve` - is this an approving vote?
       * If the proposal doesn't exist, passing `false` will result in error `FirstVoteReject`.
       * * `call` - the call to propose for execution.
       * 
       * # Errors
       * * `FirstVoteReject`, if `call` hasn't been proposed and `approve == false`.
       * * `NotAMember`, if the `origin` is not a member of this committee.
       **/
      voteOrPropose: AugmentedSubmittable<(approve: bool | boolean | Uint8Array, call: Call | { callIndex?: any; args?: any } | string | Uint8Array) => SubmittableExtrinsic<ApiType>, [bool, Call]>;
      /**
       * Generic tx
       **/
      [key: string]: SubmittableExtrinsicFunction<ApiType>;
    };
    polymeshContracts: {
      /**
       * Calls the `contract` through its address with the given `data`.
       * 
       * The contract is endowed with `value` POLYX,
       * but note that this is distinct from gas fees which are limited with `gas_limit`.
       * 
       * The contract may optionally call back into the runtime,
       * executing extrinsics such as e.g., `create_asset`.
       * During such runtime calls, the current identity will be the one that instantiate the `contract`.
       * This restriction exists for security purposes.
       * 
       * # Arguments
       * - `contract` to call.
       * - `value` in POLYX to transfer to the contract.
       * - `gas_limit` that limits how much gas execution can consume, erroring above it.
       * - `data` to be interpreted by the smart contract call.
       * 
       * # Errors
       * - All the errors in `pallet_contracts::Call::call` can also happen here.
       * - `ContractNotFound` if `contract` doesn't exist or isn't a contract.
       * - CDD/Permissions are checked, unlike in `pallet_contracts`.
       **/
      call: AugmentedSubmittable<(contract: AccountId32 | string | Uint8Array, value: u128 | AnyNumber | Uint8Array, gasLimit: u64 | AnyNumber | Uint8Array, storageDepositLimit: Option<u128> | null | object | string | Uint8Array, data: Bytes | string | Uint8Array) => SubmittableExtrinsic<ApiType>, [AccountId32, u128, u64, Option<u128>, Bytes]>;
      /**
       * Instantiates a smart contract defining it with the given `code` and `salt`.
       * 
       * The contract will be attached as a secondary key,
       * with `perms` as its permissions, to `origin`'s identity.
       * 
       * The contract is transferred `endowment` amount of POLYX.
       * This is distinct from the `gas_limit`,
       * which controls how much gas the deployment code may at most consume.
       * 
       * # Arguments
       * - `endowment` amount of POLYX to transfer to the contract.
       * - `gas_limit` for how much gas the `deploy` code in the contract may at most consume.
       * - `code` with the WASM binary defining the smart contract.
       * - `salt` used for contract address derivation.
       * By varying this, the same `code` can be used under the same identity.
       * 
       * # Errors
       * - All the errors in `pallet_contracts::Call::instantiate_with_code` can also happen here.
       * - CDD/Permissions are checked, unlike in `pallet_contracts`.
       * - Errors that arise when adding a new secondary key can also occur here.
       **/
      instantiateWithCode: AugmentedSubmittable<(endowment: u128 | AnyNumber | Uint8Array, gasLimit: u64 | AnyNumber | Uint8Array, storageDepositLimit: Option<u128> | null | object | string | Uint8Array, code: Bytes | string | Uint8Array, data: Bytes | string | Uint8Array, salt: Bytes | string | Uint8Array, perms: PolymeshPrimitivesSecondaryKeyPermissions | { asset?: any; extrinsic?: any; portfolio?: any } | string | Uint8Array) => SubmittableExtrinsic<ApiType>, [u128, u64, Option<u128>, Bytes, Bytes, Bytes, PolymeshPrimitivesSecondaryKeyPermissions]>;
      /**
       * Instantiates a smart contract defining using the given `code_hash` and `salt`.
       * 
       * Unlike `instantiate_with_code`,
       * this assumes that at least one contract with the same WASM code has already been uploaded.
       * 
       * The contract will be attached as a secondary key,
       * with `perms` as its permissions, to `origin`'s identity.
       * 
       * The contract is transferred `endowment` amount of POLYX.
       * This is distinct from the `gas_limit`,
       * which controls how much gas the deployment code may at most consume.
       * 
       * # Arguments
       * - `endowment` amount of POLYX to transfer to the contract.
       * - `gas_limit` for how much gas the `deploy` code in the contract may at most consume.
       * - `code_hash` of an already uploaded WASM binary.
       * - `salt` used for contract address derivation.
       * By varying this, the same `code` can be used under the same identity.
       * 
       * # Errors
       * - All the errors in `pallet_contracts::Call::instantiate` can also happen here.
       * - CDD/Permissions are checked, unlike in `pallet_contracts`.
       * - Errors that arise when adding a new secondary key can also occur here.
       **/
      instantiateWithHash: AugmentedSubmittable<(endowment: u128 | AnyNumber | Uint8Array, gasLimit: u64 | AnyNumber | Uint8Array, storageDepositLimit: Option<u128> | null | object | string | Uint8Array, codeHash: H256 | string | Uint8Array, data: Bytes | string | Uint8Array, salt: Bytes | string | Uint8Array, perms: PolymeshPrimitivesSecondaryKeyPermissions | { asset?: any; extrinsic?: any; portfolio?: any } | string | Uint8Array) => SubmittableExtrinsic<ApiType>, [u128, u64, Option<u128>, H256, Bytes, Bytes, PolymeshPrimitivesSecondaryKeyPermissions]>;
      /**
       * Generic tx
       **/
      [key: string]: SubmittableExtrinsicFunction<ApiType>;
    };
    portfolio: {
      acceptPortfolioCustody: AugmentedSubmittable<(authId: u64 | AnyNumber | Uint8Array) => SubmittableExtrinsic<ApiType>, [u64]>;
      /**
       * Creates a portfolio with the given `name`.
       **/
      createPortfolio: AugmentedSubmittable<(name: Bytes | string | Uint8Array) => SubmittableExtrinsic<ApiType>, [Bytes]>;
      /**
       * Deletes a user portfolio. A portfolio can be deleted only if it has no funds.
       * 
       * # Errors
       * * `PortfolioDoesNotExist` if `num` doesn't reference a valid portfolio.
       * * `PortfolioNotEmpty` if the portfolio still holds any asset
       * 
       * # Permissions
       * * Portfolio
       **/
      deletePortfolio: AugmentedSubmittable<(num: u64 | AnyNumber | Uint8Array) => SubmittableExtrinsic<ApiType>, [u64]>;
      /**
       * Moves a token amount from one portfolio of an identity to another portfolio of the same
       * identity. Must be called by the custodian of the sender.
       * Funds from deleted portfolios can also be recovered via this method.
       * 
       * A short memo can be added to to each token amount moved.
       * 
       * # Errors
       * * `PortfolioDoesNotExist` if one or both of the portfolios reference an invalid portfolio.
       * * `destination_is_same_portfolio` if both sender and receiver portfolio are the same
       * * `DifferentIdentityPortfolios` if the sender and receiver portfolios belong to different identities
       * * `UnauthorizedCustodian` if the caller is not the custodian of the from portfolio
       * * `InsufficientPortfolioBalance` if the sender does not have enough free balance
       * 
       * # Permissions
       * * Portfolio
       **/
      movePortfolioFunds: AugmentedSubmittable<(from: PolymeshPrimitivesIdentityIdPortfolioId | { did?: any; kind?: any } | string | Uint8Array, to: PolymeshPrimitivesIdentityIdPortfolioId | { did?: any; kind?: any } | string | Uint8Array, items: Vec<PalletPortfolioMovePortfolioItem> | (PalletPortfolioMovePortfolioItem | { ticker?: any; amount?: any; memo?: any } | string | Uint8Array)[]) => SubmittableExtrinsic<ApiType>, [PolymeshPrimitivesIdentityIdPortfolioId, PolymeshPrimitivesIdentityIdPortfolioId, Vec<PalletPortfolioMovePortfolioItem>]>;
      /**
       * When called by the custodian of `portfolio_id`,
       * allows returning the custody of the portfolio to the portfolio owner unilaterally.
       * 
       * # Errors
       * * `UnauthorizedCustodian` if the caller is not the current custodian of `portfolio_id`.
       * 
       * # Permissions
       * * Portfolio
       **/
      quitPortfolioCustody: AugmentedSubmittable<(pid: PolymeshPrimitivesIdentityIdPortfolioId | { did?: any; kind?: any } | string | Uint8Array) => SubmittableExtrinsic<ApiType>, [PolymeshPrimitivesIdentityIdPortfolioId]>;
      /**
       * Renames a non-default portfolio.
       * 
       * # Errors
       * * `PortfolioDoesNotExist` if `num` doesn't reference a valid portfolio.
       * 
       * # Permissions
       * * Portfolio
       **/
      renamePortfolio: AugmentedSubmittable<(num: u64 | AnyNumber | Uint8Array, toName: Bytes | string | Uint8Array) => SubmittableExtrinsic<ApiType>, [u64, Bytes]>;
      /**
       * Generic tx
       **/
      [key: string]: SubmittableExtrinsicFunction<ApiType>;
    };
    preimage: {
      /**
       * Register a preimage on-chain.
       * 
       * If the preimage was previously requested, no fees or deposits are taken for providing
       * the preimage. Otherwise, a deposit is taken proportional to the size of the preimage.
       **/
      notePreimage: AugmentedSubmittable<(bytes: Bytes | string | Uint8Array) => SubmittableExtrinsic<ApiType>, [Bytes]>;
      /**
       * Request a preimage be uploaded to the chain without paying any fees or deposits.
       * 
       * If the preimage requests has already been provided on-chain, we unreserve any deposit
       * a user may have paid, and take the control of the preimage out of their hands.
       **/
      requestPreimage: AugmentedSubmittable<(hash: H256 | string | Uint8Array) => SubmittableExtrinsic<ApiType>, [H256]>;
      /**
       * Clear an unrequested preimage from the runtime storage.
       **/
      unnotePreimage: AugmentedSubmittable<(hash: H256 | string | Uint8Array) => SubmittableExtrinsic<ApiType>, [H256]>;
      /**
       * Clear a previously made request for a preimage.
       * 
       * NOTE: THIS MUST NOT BE CALLED ON `hash` MORE TIMES THAN `request_preimage`.
       **/
      unrequestPreimage: AugmentedSubmittable<(hash: H256 | string | Uint8Array) => SubmittableExtrinsic<ApiType>, [H256]>;
      /**
       * Generic tx
       **/
      [key: string]: SubmittableExtrinsicFunction<ApiType>;
    };
    protocolFee: {
      /**
       * Changes the a base fee for the root origin.
       * 
       * # Errors
       * * `BadOrigin` - Only root allowed.
       **/
      changeBaseFee: AugmentedSubmittable<(op: PolymeshCommonUtilitiesProtocolFeeProtocolOp | 'AssetRegisterTicker' | 'AssetIssue' | 'AssetAddDocuments' | 'AssetCreateAsset' | 'CheckpointCreateSchedule' | 'ComplianceManagerAddComplianceRequirement' | 'IdentityCddRegisterDid' | 'IdentityAddClaim' | 'IdentityAddSecondaryKeysWithAuthorization' | 'PipsPropose' | 'ContractsPutCode' | 'CorporateBallotAttachBallot' | 'CapitalDistributionDistribute' | number | Uint8Array, baseFee: u128 | AnyNumber | Uint8Array) => SubmittableExtrinsic<ApiType>, [PolymeshCommonUtilitiesProtocolFeeProtocolOp, u128]>;
      /**
       * Changes the fee coefficient for the root origin.
       * 
       * # Errors
       * * `BadOrigin` - Only root allowed.
       **/
      changeCoefficient: AugmentedSubmittable<(coefficient: PolymeshPrimitivesPosRatio) => SubmittableExtrinsic<ApiType>, [PolymeshPrimitivesPosRatio]>;
      /**
       * Generic tx
       **/
      [key: string]: SubmittableExtrinsicFunction<ApiType>;
    };
    relayer: {
      /**
       * Accepts a `paying_key` authorization.
       * 
       * # Arguments
       * - `auth_id` the authorization id to accept a `paying_key`.
       * 
       * # Errors
       * - `AuthorizationError::Invalid` if `auth_id` does not exist for the given caller.
       * - `AuthorizationError::Expired` if `auth_id` the authorization has expired.
       * - `AuthorizationError::BadType` if `auth_id` was not a `AddRelayerPayingKey` authorization.
       * - `NotAuthorizedForUserKey` if `origin` is not authorized to accept the authorization for the `user_key`.
       * - `NotAuthorizedForPayingKey` if the authorization was created an identity different from the `paying_key`'s identity.
       * - `UserKeyCddMissing` if the `user_key` is not attached to a CDD'd identity.
       * - `PayingKeyCddMissing` if the `paying_key` is not attached to a CDD'd identity.
       * - `UnauthorizedCaller` if `origin` is not authorized to call this extrinsic.
       **/
      acceptPayingKey: AugmentedSubmittable<(authId: u64 | AnyNumber | Uint8Array) => SubmittableExtrinsic<ApiType>, [u64]>;
      /**
       * Decrease the available POLYX for a `user_key`.
       * 
       * # Arguments
       * - `user_key` the user key of the subsidy to update the available POLYX.
       * - `amount` the amount of POLYX to remove from the subsidy of `user_key`.
       * 
       * # Errors
       * - `NoPayingKey` if the `user_key` doesn't have a `paying_key`.
       * - `NotPayingKey` if `origin` doesn't match the current `paying_key`.
       * - `UnauthorizedCaller` if `origin` is not authorized to call this extrinsic.
       * - `Overlow` if the subsidy has less then `amount` POLYX remaining.
       **/
      decreasePolyxLimit: AugmentedSubmittable<(userKey: AccountId32 | string | Uint8Array, amount: u128 | AnyNumber | Uint8Array) => SubmittableExtrinsic<ApiType>, [AccountId32, u128]>;
      /**
       * Increase the available POLYX for a `user_key`.
       * 
       * # Arguments
       * - `user_key` the user key of the subsidy to update the available POLYX.
       * - `amount` the amount of POLYX to add to the subsidy of `user_key`.
       * 
       * # Errors
       * - `NoPayingKey` if the `user_key` doesn't have a `paying_key`.
       * - `NotPayingKey` if `origin` doesn't match the current `paying_key`.
       * - `UnauthorizedCaller` if `origin` is not authorized to call this extrinsic.
       * - `Overlow` if the subsidy's remaining POLYX would have overflowed `u128::MAX`.
       **/
      increasePolyxLimit: AugmentedSubmittable<(userKey: AccountId32 | string | Uint8Array, amount: u128 | AnyNumber | Uint8Array) => SubmittableExtrinsic<ApiType>, [AccountId32, u128]>;
      /**
       * Removes the `paying_key` from a `user_key`.
       * 
       * # Arguments
       * - `user_key` the user key to remove the subsidy from.
       * - `paying_key` the paying key that was subsidising the `user_key`.
       * 
       * # Errors
       * - `NotAuthorizedForUserKey` if `origin` is not authorized to remove the subsidy for the `user_key`.
       * - `NoPayingKey` if the `user_key` doesn't have a `paying_key`.
       * - `NotPayingKey` if the `paying_key` doesn't match the current `paying_key`.
       * - `UnauthorizedCaller` if `origin` is not authorized to call this extrinsic.
       **/
      removePayingKey: AugmentedSubmittable<(userKey: AccountId32 | string | Uint8Array, payingKey: AccountId32 | string | Uint8Array) => SubmittableExtrinsic<ApiType>, [AccountId32, AccountId32]>;
      /**
       * Creates an authorization to allow `user_key` to accept the caller (`origin == paying_key`) as their subsidiser.
       * 
       * # Arguments
       * - `user_key` the user key to subsidise.
       * - `polyx_limit` the initial POLYX limit for this subsidy.
       * 
       * # Errors
       * - `UnauthorizedCaller` if `origin` is not authorized to call this extrinsic.
       **/
      setPayingKey: AugmentedSubmittable<(userKey: AccountId32 | string | Uint8Array, polyxLimit: u128 | AnyNumber | Uint8Array) => SubmittableExtrinsic<ApiType>, [AccountId32, u128]>;
      /**
       * Updates the available POLYX for a `user_key`.
       * 
       * # Arguments
       * - `user_key` the user key of the subsidy to update the available POLYX.
       * - `polyx_limit` the amount of POLYX available for subsidising the `user_key`.
       * 
       * # Errors
       * - `NoPayingKey` if the `user_key` doesn't have a `paying_key`.
       * - `NotPayingKey` if `origin` doesn't match the current `paying_key`.
       * - `UnauthorizedCaller` if `origin` is not authorized to call this extrinsic.
       **/
      updatePolyxLimit: AugmentedSubmittable<(userKey: AccountId32 | string | Uint8Array, polyxLimit: u128 | AnyNumber | Uint8Array) => SubmittableExtrinsic<ApiType>, [AccountId32, u128]>;
      /**
       * Generic tx
       **/
      [key: string]: SubmittableExtrinsicFunction<ApiType>;
    };
    rewards: {
      /**
       * Claim an ITN reward.
       * 
       * ## Arguments
       * * `itn_address` specifying the awarded address on ITN.
       * * `signature` authenticating the claim to the reward.
       * The signature should contain `reward_address` followed by the suffix `"claim_itn_reward"`,
       * and must have been signed by `itn_address`.
       * 
       * # Errors
       * * `InsufficientBalance` - Itn rewards has insufficient funds to issue the reward.
       * * `InvalidSignature` - `signature` had an invalid signer or invalid message.
       * * `ItnRewardAlreadyClaimed` - Reward issued to the `itn_address` has already been claimed.
       * * `UnknownItnAddress` - `itn_address` is not in the rewards table and has no reward to be claimed.
       **/
      claimItnReward: AugmentedSubmittable<(rewardAddress: AccountId32 | string | Uint8Array, itnAddress: AccountId32 | string | Uint8Array, signature: SpRuntimeMultiSignature | { Ed25519: any } | { Sr25519: any } | { Ecdsa: any } | string | Uint8Array) => SubmittableExtrinsic<ApiType>, [AccountId32, AccountId32, SpRuntimeMultiSignature]>;
      setItnRewardStatus: AugmentedSubmittable<(itnAddress: AccountId32 | string | Uint8Array, status: PalletRewardsItnRewardStatus | { Unclaimed: any } | { Claimed: any } | string | Uint8Array) => SubmittableExtrinsic<ApiType>, [AccountId32, PalletRewardsItnRewardStatus]>;
      /**
       * Generic tx
       **/
      [key: string]: SubmittableExtrinsicFunction<ApiType>;
    };
    scheduler: {
      /**
       * Cancel an anonymously scheduled task.
       **/
      cancel: AugmentedSubmittable<(when: u32 | AnyNumber | Uint8Array, index: u32 | AnyNumber | Uint8Array) => SubmittableExtrinsic<ApiType>, [u32, u32]>;
      /**
       * Cancel a named scheduled task.
       **/
      cancelNamed: AugmentedSubmittable<(id: Bytes | string | Uint8Array) => SubmittableExtrinsic<ApiType>, [Bytes]>;
      /**
       * Anonymously schedule a task.
       **/
      schedule: AugmentedSubmittable<(when: u32 | AnyNumber | Uint8Array, maybePeriodic: Option<ITuple<[u32, u32]>> | null | object | string | Uint8Array, priority: u8 | AnyNumber | Uint8Array, call: FrameSupportScheduleMaybeHashed | { Value: any } | { Hash: any } | string | Uint8Array) => SubmittableExtrinsic<ApiType>, [u32, Option<ITuple<[u32, u32]>>, u8, FrameSupportScheduleMaybeHashed]>;
      /**
       * Anonymously schedule a task after a delay.
       * 
       * # <weight>
       * Same as [`schedule`].
       * # </weight>
       **/
      scheduleAfter: AugmentedSubmittable<(after: u32 | AnyNumber | Uint8Array, maybePeriodic: Option<ITuple<[u32, u32]>> | null | object | string | Uint8Array, priority: u8 | AnyNumber | Uint8Array, call: FrameSupportScheduleMaybeHashed | { Value: any } | { Hash: any } | string | Uint8Array) => SubmittableExtrinsic<ApiType>, [u32, Option<ITuple<[u32, u32]>>, u8, FrameSupportScheduleMaybeHashed]>;
      /**
       * Schedule a named task.
       **/
      scheduleNamed: AugmentedSubmittable<(id: Bytes | string | Uint8Array, when: u32 | AnyNumber | Uint8Array, maybePeriodic: Option<ITuple<[u32, u32]>> | null | object | string | Uint8Array, priority: u8 | AnyNumber | Uint8Array, call: FrameSupportScheduleMaybeHashed | { Value: any } | { Hash: any } | string | Uint8Array) => SubmittableExtrinsic<ApiType>, [Bytes, u32, Option<ITuple<[u32, u32]>>, u8, FrameSupportScheduleMaybeHashed]>;
      /**
       * Schedule a named task after a delay.
       * 
       * # <weight>
       * Same as [`schedule_named`](Self::schedule_named).
       * # </weight>
       **/
      scheduleNamedAfter: AugmentedSubmittable<(id: Bytes | string | Uint8Array, after: u32 | AnyNumber | Uint8Array, maybePeriodic: Option<ITuple<[u32, u32]>> | null | object | string | Uint8Array, priority: u8 | AnyNumber | Uint8Array, call: FrameSupportScheduleMaybeHashed | { Value: any } | { Hash: any } | string | Uint8Array) => SubmittableExtrinsic<ApiType>, [Bytes, u32, Option<ITuple<[u32, u32]>>, u8, FrameSupportScheduleMaybeHashed]>;
      /**
       * Generic tx
       **/
      [key: string]: SubmittableExtrinsicFunction<ApiType>;
    };
    session: {
      /**
       * Removes any session key(s) of the function caller.
       * 
       * This doesn't take effect until the next session.
       * 
       * The dispatch origin of this function must be Signed and the account must be either be
       * convertible to a validator ID using the chain's typical addressing system (this usually
       * means being a controller account) or directly convertible into a validator ID (which
       * usually means being a stash account).
       * 
       * # <weight>
       * - Complexity: `O(1)` in number of key types. Actual cost depends on the number of length
       * of `T::Keys::key_ids()` which is fixed.
       * - DbReads: `T::ValidatorIdOf`, `NextKeys`, `origin account`
       * - DbWrites: `NextKeys`, `origin account`
       * - DbWrites per key id: `KeyOwner`
       * # </weight>
       **/
      purgeKeys: AugmentedSubmittable<() => SubmittableExtrinsic<ApiType>, []>;
      /**
       * Sets the session key(s) of the function caller to `keys`.
       * Allows an account to set its session key prior to becoming a validator.
       * This doesn't take effect until the next session.
       * 
       * The dispatch origin of this function must be signed.
       * 
       * # <weight>
       * - Complexity: `O(1)`. Actual cost depends on the number of length of
       * `T::Keys::key_ids()` which is fixed.
       * - DbReads: `origin account`, `T::ValidatorIdOf`, `NextKeys`
       * - DbWrites: `origin account`, `NextKeys`
       * - DbReads per key id: `KeyOwner`
       * - DbWrites per key id: `KeyOwner`
       * # </weight>
       **/
      setKeys: AugmentedSubmittable<(keys: PolymeshRuntimeDevelopRuntimeSessionKeys | { grandpa?: any; babe?: any; imOnline?: any; authorityDiscovery?: any } | string | Uint8Array, proof: Bytes | string | Uint8Array) => SubmittableExtrinsic<ApiType>, [PolymeshRuntimeDevelopRuntimeSessionKeys, Bytes]>;
      /**
       * Generic tx
       **/
      [key: string]: SubmittableExtrinsicFunction<ApiType>;
    };
    settlement: {
      /**
       * Adds and affirms a new instruction.
       * 
       * # Arguments
       * * `venue_id` - ID of the venue this instruction belongs to.
       * * `settlement_type` - Defines if the instruction should be settled
       * in the next block after receiving all affirmations or waiting till a specific block.
       * * `trade_date` - Optional date from which people can interact with this instruction.
       * * `value_date` - Optional date after which the instruction should be settled (not enforced)
       * * `legs` - Legs included in this instruction.
       * * `portfolios` - Portfolios that the sender controls and wants to use in this affirmations.
       * 
       * # Permissions
       * * Portfolio
       **/
      addAndAffirmInstruction: AugmentedSubmittable<(venueId: u64 | AnyNumber | Uint8Array, settlementType: PalletSettlementSettlementType | { SettleOnAffirmation: any } | { SettleOnBlock: any } | string | Uint8Array, tradeDate: Option<u64> | null | object | string | Uint8Array, valueDate: Option<u64> | null | object | string | Uint8Array, legs: Vec<PalletSettlementLeg> | (PalletSettlementLeg | { from?: any; to?: any; asset?: any; amount?: any } | string | Uint8Array)[], portfolios: Vec<PolymeshPrimitivesIdentityIdPortfolioId> | (PolymeshPrimitivesIdentityIdPortfolioId | { did?: any; kind?: any } | string | Uint8Array)[]) => SubmittableExtrinsic<ApiType>, [u64, PalletSettlementSettlementType, Option<u64>, Option<u64>, Vec<PalletSettlementLeg>, Vec<PolymeshPrimitivesIdentityIdPortfolioId>]>;
      /**
       * Adds a new instruction.
       * 
       * # Arguments
       * * `venue_id` - ID of the venue this instruction belongs to.
       * * `settlement_type` - Defines if the instruction should be settled
       * in the next block after receiving all affirmations or waiting till a specific block.
       * * `trade_date` - Optional date from which people can interact with this instruction.
       * * `value_date` - Optional date after which the instruction should be settled (not enforced)
       * * `legs` - Legs included in this instruction.
       * 
       * # Weight
       * `950_000_000 + 1_000_000 * legs.len()`
       **/
      addInstruction: AugmentedSubmittable<(venueId: u64 | AnyNumber | Uint8Array, settlementType: PalletSettlementSettlementType | { SettleOnAffirmation: any } | { SettleOnBlock: any } | string | Uint8Array, tradeDate: Option<u64> | null | object | string | Uint8Array, valueDate: Option<u64> | null | object | string | Uint8Array, legs: Vec<PalletSettlementLeg> | (PalletSettlementLeg | { from?: any; to?: any; asset?: any; amount?: any } | string | Uint8Array)[]) => SubmittableExtrinsic<ApiType>, [u64, PalletSettlementSettlementType, Option<u64>, Option<u64>, Vec<PalletSettlementLeg>]>;
      /**
       * Provide affirmation to an existing instruction.
       * 
       * # Arguments
       * * `id` - Instruction id to affirm.
       * * `portfolios` - Portfolios that the sender controls and wants to affirm this instruction.
       * * `max_legs_count` - Number of legs that need to be  affirmed.
       * 
       * # Permissions
       * * Portfolio
       **/
      affirmInstruction: AugmentedSubmittable<(id: u64 | AnyNumber | Uint8Array, portfolios: Vec<PolymeshPrimitivesIdentityIdPortfolioId> | (PolymeshPrimitivesIdentityIdPortfolioId | { did?: any; kind?: any } | string | Uint8Array)[], maxLegsCount: u32 | AnyNumber | Uint8Array) => SubmittableExtrinsic<ApiType>, [u64, Vec<PolymeshPrimitivesIdentityIdPortfolioId>, u32]>;
      /**
       * Accepts an instruction and claims a signed receipt.
       * 
       * # Arguments
       * * `id` - Target instruction id.
       * * `leg_id` - Target leg id for the receipt
       * * `receipt_uid` - Receipt ID generated by the signer.
       * * `signer` - Signer of the receipt.
       * * `signed_data` - Signed receipt.
       * * `portfolios` - Portfolios that the sender controls and wants to accept this instruction with
       * 
       * # Permissions
       * * Portfolio
       **/
      affirmWithReceipts: AugmentedSubmittable<(id: u64 | AnyNumber | Uint8Array, receiptDetails: Vec<PalletSettlementReceiptDetails> | (PalletSettlementReceiptDetails | { receiptUid?: any; legId?: any; signer?: any; signature?: any; metadata?: any } | string | Uint8Array)[], portfolios: Vec<PolymeshPrimitivesIdentityIdPortfolioId> | (PolymeshPrimitivesIdentityIdPortfolioId | { did?: any; kind?: any } | string | Uint8Array)[], maxLegsCount: u32 | AnyNumber | Uint8Array) => SubmittableExtrinsic<ApiType>, [u64, Vec<PalletSettlementReceiptDetails>, Vec<PolymeshPrimitivesIdentityIdPortfolioId>, u32]>;
      /**
       * Allows additional venues to create instructions involving an asset.
       * 
       * * `ticker` - Ticker of the token in question.
       * * `venues` - Array of venues that are allowed to create instructions for the token in question.
       * 
       * # Permissions
       * * Asset
       **/
      allowVenues: AugmentedSubmittable<(ticker: PolymeshPrimitivesTicker | string | Uint8Array, venues: Vec<u64> | (u64 | AnyNumber | Uint8Array)[]) => SubmittableExtrinsic<ApiType>, [PolymeshPrimitivesTicker, Vec<u64>]>;
      /**
       * Marks a receipt issued by the caller as claimed or not claimed.
       * This allows the receipt issuer to invalidate an already issued receipt or revalidate an already claimed receipt.
       * 
       * * `receipt_uid` - Unique ID of the receipt.
       * * `validity` - New validity of the receipt.
       **/
      changeReceiptValidity: AugmentedSubmittable<(receiptUid: u64 | AnyNumber | Uint8Array, validity: bool | boolean | Uint8Array) => SubmittableExtrinsic<ApiType>, [u64, bool]>;
      /**
       * Claims a signed receipt.
       * 
       * # Arguments
       * * `id` - Target instruction id for the receipt.
       * * `leg_id` - Target leg id for the receipt
       * * `receipt_uid` - Receipt ID generated by the signer.
       * * `signer` - Signer of the receipt.
       * * `signed_data` - Signed receipt.
       * 
       * # Permissions
       * * Portfolio
       **/
      claimReceipt: AugmentedSubmittable<(id: u64 | AnyNumber | Uint8Array, receiptDetails: PalletSettlementReceiptDetails | { receiptUid?: any; legId?: any; signer?: any; signature?: any; metadata?: any } | string | Uint8Array) => SubmittableExtrinsic<ApiType>, [u64, PalletSettlementReceiptDetails]>;
      /**
       * Registers a new venue.
       * 
       * * `details` - Extra details about a venue
       * * `signers` - Array of signers that are allowed to sign receipts for this venue
       * * `typ` - Type of venue being created
       **/
      createVenue: AugmentedSubmittable<(details: Bytes | string | Uint8Array, signers: Vec<AccountId32> | (AccountId32 | string | Uint8Array)[], typ: PalletSettlementVenueType | 'Other' | 'Distribution' | 'Sto' | 'Exchange' | number | Uint8Array) => SubmittableExtrinsic<ApiType>, [Bytes, Vec<AccountId32>, PalletSettlementVenueType]>;
      /**
       * Revokes permission given to venues for creating instructions involving a particular asset.
       * 
       * * `ticker` - Ticker of the token in question.
       * * `venues` - Array of venues that are no longer allowed to create instructions for the token in question.
       * 
       * # Permissions
       * * Asset
       **/
      disallowVenues: AugmentedSubmittable<(ticker: PolymeshPrimitivesTicker | string | Uint8Array, venues: Vec<u64> | (u64 | AnyNumber | Uint8Array)[]) => SubmittableExtrinsic<ApiType>, [PolymeshPrimitivesTicker, Vec<u64>]>;
      /**
       * Root callable extrinsic, used as an internal call to execute a scheduled settlement instruction.
       **/
      executeScheduledInstruction: AugmentedSubmittable<(id: u64 | AnyNumber | Uint8Array, legsCount: u32 | AnyNumber | Uint8Array) => SubmittableExtrinsic<ApiType>, [u64, u32]>;
      /**
       * Rejects an existing instruction.
       * 
       * # Arguments
       * * `id` - Instruction id to reject.
       * * `portfolio` - Portfolio to reject the instruction.
       * * `num_of_legs` - Number of legs in the instruction.
       * 
       * # Permissions
       * * Portfolio
       **/
      rejectInstruction: AugmentedSubmittable<(id: u64 | AnyNumber | Uint8Array, portfolio: PolymeshPrimitivesIdentityIdPortfolioId | { did?: any; kind?: any } | string | Uint8Array, numOfLegs: u32 | AnyNumber | Uint8Array) => SubmittableExtrinsic<ApiType>, [u64, PolymeshPrimitivesIdentityIdPortfolioId, u32]>;
      /**
       * Reschedules a failed instruction.
       * 
       * # Arguments
       * * `id` - Target instruction id to reschedule.
       * 
       * # Permissions
       * * Portfolio
       * 
       * # Errors
       * * `InstructionNotFailed` - Instruction not in a failed state or does not exist.
       **/
      rescheduleInstruction: AugmentedSubmittable<(id: u64 | AnyNumber | Uint8Array) => SubmittableExtrinsic<ApiType>, [u64]>;
      /**
       * Enables or disabled venue filtering for a token.
       * 
       * # Arguments
       * * `ticker` - Ticker of the token in question.
       * * `enabled` - Boolean that decides if the filtering should be enabled.
       * 
       * # Permissions
       * * Asset
       **/
      setVenueFiltering: AugmentedSubmittable<(ticker: PolymeshPrimitivesTicker | string | Uint8Array, enabled: bool | boolean | Uint8Array) => SubmittableExtrinsic<ApiType>, [PolymeshPrimitivesTicker, bool]>;
      /**
       * Unclaims a previously claimed receipt.
       * 
       * # Arguments
       * * `instruction_id` - Target instruction id for the receipt.
       * * `leg_id` - Target leg id for the receipt
       * 
       * # Permissions
       * * Portfolio
       **/
      unclaimReceipt: AugmentedSubmittable<(instructionId: u64 | AnyNumber | Uint8Array, legId: u64 | AnyNumber | Uint8Array) => SubmittableExtrinsic<ApiType>, [u64, u64]>;
      /**
       * Edit a venue's details.
       * 
       * * `id` specifies the ID of the venue to edit.
       * * `details` specifies the updated venue details.
       **/
      updateVenueDetails: AugmentedSubmittable<(id: u64 | AnyNumber | Uint8Array, details: Bytes | string | Uint8Array) => SubmittableExtrinsic<ApiType>, [u64, Bytes]>;
      /**
       * Edit a venue's type.
       * 
       * * `id` specifies the ID of the venue to edit.
       * * `type` specifies the new type of the venue.
       **/
      updateVenueType: AugmentedSubmittable<(id: u64 | AnyNumber | Uint8Array, typ: PalletSettlementVenueType | 'Other' | 'Distribution' | 'Sto' | 'Exchange' | number | Uint8Array) => SubmittableExtrinsic<ApiType>, [u64, PalletSettlementVenueType]>;
      /**
       * Withdraw an affirmation for a given instruction.
       * 
       * # Arguments
       * * `id` - Instruction id for that affirmation get withdrawn.
       * * `portfolios` - Portfolios that the sender controls and wants to withdraw affirmation.
       * * `max_legs_count` - Number of legs that need to be un-affirmed.
       * 
       * # Permissions
       * * Portfolio
       **/
      withdrawAffirmation: AugmentedSubmittable<(id: u64 | AnyNumber | Uint8Array, portfolios: Vec<PolymeshPrimitivesIdentityIdPortfolioId> | (PolymeshPrimitivesIdentityIdPortfolioId | { did?: any; kind?: any } | string | Uint8Array)[], maxLegsCount: u32 | AnyNumber | Uint8Array) => SubmittableExtrinsic<ApiType>, [u64, Vec<PolymeshPrimitivesIdentityIdPortfolioId>, u32]>;
      /**
       * Generic tx
       **/
      [key: string]: SubmittableExtrinsicFunction<ApiType>;
    };
    staking: {
      /**
       * Governance committee on 2/3 rds majority can introduce a new potential identity
       * to the pool of permissioned entities who can run validators. Staking module uses `PermissionedIdentity`
       * to ensure validators have completed KYB compliance and considers them for validation.
       * 
       * # Arguments
       * * origin Required origin for adding a potential validator.
       * * identity Validator's IdentityId.
       * * intended_count No. of validators given identity intends to run.
       **/
      addPermissionedValidator: AugmentedSubmittable<(identity: PolymeshPrimitivesIdentityId | string | Uint8Array, intendedCount: Option<u32> | null | object | string | Uint8Array) => SubmittableExtrinsic<ApiType>, [PolymeshPrimitivesIdentityId, Option<u32>]>;
      /**
       * Take the origin account as a stash and lock up `value` of its balance. `controller` will
       * be the account that controls it.
       * 
       * `value` must be more than the `minimum_balance` specified by `T::Currency`.
       * 
       * The dispatch origin for this call must be _Signed_ by the stash account.
       * 
       * Emits `Bonded`.
       * 
       * # <weight>
       * - Independent of the arguments. Moderate complexity.
       * - O(1).
       * - Three extra DB entries.
       * 
       * NOTE: Two of the storage writes (`Self::bonded`, `Self::payee`) are _never_ cleaned
       * unless the `origin` falls below _existential deposit_ and gets removed as dust.
       * ------------------
       * Weight: O(1)
       * DB Weight:
       * - Read: Bonded, Ledger, [Origin Account], Current Era, History Depth, Locks
       * - Write: Bonded, Payee, [Origin Account], Locks, Ledger
       * # </weight>
       **/
      bond: AugmentedSubmittable<(controller: MultiAddress | { Id: any } | { Index: any } | { Raw: any } | { Address32: any } | { Address20: any } | string | Uint8Array, value: Compact<u128> | AnyNumber | Uint8Array, payee: PalletStakingRewardDestination | { Staked: any } | { Stash: any } | { Controller: any } | { Account: any } | string | Uint8Array) => SubmittableExtrinsic<ApiType>, [MultiAddress, Compact<u128>, PalletStakingRewardDestination]>;
      /**
       * Add some extra amount that have appeared in the stash `free_balance` into the balance up
       * for staking.
       * 
       * Use this if there are additional funds in your stash account that you wish to bond.
       * Unlike [`bond`] or [`unbond`] this function does not impose any limitation on the amount
       * that can be added.
       * 
       * The dispatch origin for this call must be _Signed_ by the stash, not the controller and
       * it can be only called when [`EraElectionStatus`] is `Closed`.
       * 
       * Emits `Bonded`.
       * 
       * # <weight>
       * - Independent of the arguments. Insignificant complexity.
       * - O(1).
       * - One DB entry.
       * ------------
       * DB Weight:
       * - Read: Era Election Status, Bonded, Ledger, [Origin Account], Locks
       * - Write: [Origin Account], Locks, Ledger
       * # </weight>
       **/
      bondExtra: AugmentedSubmittable<(maxAdditional: Compact<u128> | AnyNumber | Uint8Array) => SubmittableExtrinsic<ApiType>, [Compact<u128>]>;
      /**
       * Cancel enactment of a deferred slash.
       * 
       * Can be called by the `T::SlashCancelOrigin`.
       * 
       * Parameters: era and indices of the slashes for that era to kill.
       * 
       * # <weight>
       * Complexity: O(U + S)
       * with U unapplied slashes weighted with U=1000
       * and S is the number of slash indices to be canceled.
       * - Read: Unapplied Slashes
       * - Write: Unapplied Slashes
       * # </weight>
       **/
      cancelDeferredSlash: AugmentedSubmittable<(era: u32 | AnyNumber | Uint8Array, slashIndices: Vec<u32> | (u32 | AnyNumber | Uint8Array)[]) => SubmittableExtrinsic<ApiType>, [u32, Vec<u32>]>;
      /**
       * Switch slashing status on the basis of given `SlashingSwitch`. Can only be called by root.
       * 
       * # Arguments
       * * origin - AccountId of root.
       * * slashing_switch - Switch used to set the targets for slashing.
       **/
      changeSlashingAllowedFor: AugmentedSubmittable<(slashingSwitch: PalletStakingSlashingSwitch | 'Validator' | 'ValidatorAndNominator' | 'None' | number | Uint8Array) => SubmittableExtrinsic<ApiType>, [PalletStakingSlashingSwitch]>;
      /**
       * Declare no desire to either validate or nominate.
       * 
       * Effects will be felt at the beginning of the next era.
       * 
       * The dispatch origin for this call must be _Signed_ by the controller, not the stash.
       * And, it can be only called when [`EraElectionStatus`] is `Closed`.
       * 
       * # <weight>
       * - Independent of the arguments. Insignificant complexity.
       * - Contains one read.
       * - Writes are limited to the `origin` account key.
       * --------
       * Weight: O(1)
       * DB Weight:
       * - Read: EraElectionStatus, Ledger
       * - Write: Validators, Nominators
       * # </weight>
       **/
      chill: AugmentedSubmittable<() => SubmittableExtrinsic<ApiType>, []>;
      /**
       * Force there to be a new era at the end of the next session. After this, it will be
       * reset to normal (non-forced) behaviour.
       * 
       * The dispatch origin must be Root.
       * 
       * # <weight>
       * - No arguments.
       * - Weight: O(1)
       * - Write ForceEra
       * # </weight>
       **/
      forceNewEra: AugmentedSubmittable<() => SubmittableExtrinsic<ApiType>, []>;
      /**
       * Force there to be a new era at the end of sessions indefinitely.
       * 
       * The dispatch origin must be Root.
       * 
       * # <weight>
       * - Weight: O(1)
       * - Write: ForceEra
       * # </weight>
       **/
      forceNewEraAlways: AugmentedSubmittable<() => SubmittableExtrinsic<ApiType>, []>;
      /**
       * Force there to be no new eras indefinitely.
       * 
       * The dispatch origin must be Root.
       * 
       * # <weight>
       * - No arguments.
       * - Weight: O(1)
       * - Write: ForceEra
       * # </weight>
       **/
      forceNoEras: AugmentedSubmittable<() => SubmittableExtrinsic<ApiType>, []>;
      /**
       * Force a current staker to become completely unstaked, immediately.
       * 
       * The dispatch origin must be Root.
       * 
       * # <weight>
       * O(S) where S is the number of slashing spans to be removed
       * Reads: Bonded, Slashing Spans, Account, Locks
       * Writes: Bonded, Slashing Spans (if S > 0), Ledger, Payee, Validators, Nominators, Account, Locks
       * Writes Each: SpanSlash * S
       * # </weight>
       **/
      forceUnstake: AugmentedSubmittable<(stash: AccountId32 | string | Uint8Array, numSlashingSpans: u32 | AnyNumber | Uint8Array) => SubmittableExtrinsic<ApiType>, [AccountId32, u32]>;
      /**
       * Increments the ideal number of validators.
       * 
       * The dispatch origin must be Root.
       * 
       * # <weight>
       * Same as [`set_validator_count`].
       * # </weight>
       **/
      increaseValidatorCount: AugmentedSubmittable<(additional: Compact<u32> | AnyNumber | Uint8Array) => SubmittableExtrinsic<ApiType>, [Compact<u32>]>;
      /**
       * Declare the desire to nominate `targets` for the origin controller.
       * 
       * Effects will be felt at the beginning of the next era. This can only be called when
       * [`EraElectionStatus`] is `Closed`.
       * 
       * The dispatch origin for this call must be _Signed_ by the controller, not the stash.
       * And, it can be only called when [`EraElectionStatus`] is `Closed`.
       * 
       * # <weight>
       * - The transaction's complexity is proportional to the size of `targets` (N)
       * which is capped at CompactAssignments::LIMIT (MAX_NOMINATIONS).
       * - Both the reads and writes follow a similar pattern.
       * ---------
       * Weight: O(N)
       * where N is the number of targets
       * DB Weight:
       * - Reads: Era Election Status, Ledger, Current Era
       * - Writes: Validators, Nominators
       * # </weight>
       **/
      nominate: AugmentedSubmittable<(targets: Vec<MultiAddress> | (MultiAddress | { Id: any } | { Index: any } | { Raw: any } | { Address32: any } | { Address20: any } | string | Uint8Array)[]) => SubmittableExtrinsic<ApiType>, [Vec<MultiAddress>]>;
      /**
       * Pay out all the stakers behind a single validator for a single era.
       * 
       * - `validator_stash` is the stash account of the validator. Their nominators, up to
       * `T::MaxNominatorRewardedPerValidator`, will also receive their rewards.
       * - `era` may be any era between `[current_era - history_depth; current_era]`.
       * 
       * The origin of this call must be _Signed_. Any account can call this function, even if
       * it is not one of the stakers.
       * 
       * This can only be called when [`EraElectionStatus`] is `Closed`.
       * 
       * # <weight>
       * - Time complexity: at most O(MaxNominatorRewardedPerValidator).
       * - Contains a limited number of reads and writes.
       * -----------
       * N is the Number of payouts for the validator (including the validator)
       * Weight:
       * - Reward Destination Staked: O(N)
       * - Reward Destination Controller (Creating): O(N)
       * DB Weight:
       * - Read: EraElectionStatus, CurrentEra, HistoryDepth, ErasValidatorReward,
       * ErasStakersClipped, ErasRewardPoints, ErasValidatorPrefs (8 items)
       * - Read Each: Bonded, Ledger, Payee, Locks, System Account (5 items)
       * - Write Each: System Account, Locks, Ledger (3 items)
       * # </weight>
       **/
      payoutStakers: AugmentedSubmittable<(validatorStash: AccountId32 | string | Uint8Array, era: u32 | AnyNumber | Uint8Array) => SubmittableExtrinsic<ApiType>, [AccountId32, u32]>;
      /**
       * System version of `payout_stakers()`. Only be called by the root origin.
       **/
      payoutStakersBySystem: AugmentedSubmittable<(validatorStash: AccountId32 | string | Uint8Array, era: u32 | AnyNumber | Uint8Array) => SubmittableExtrinsic<ApiType>, [AccountId32, u32]>;
      /**
       * Remove all data structure concerning a staker/stash once its balance is at the minimum.
       * This is essentially equivalent to `withdraw_unbonded` except it can be called by anyone
       * and the target `stash` must have no funds left beyond the ED.
       * 
       * This can be called from any origin.
       * 
       * - `stash`: The stash account to reap. Its balance must be zero.
       * 
       * # <weight>
       * Complexity: O(S) where S is the number of slashing spans on the account.
       * DB Weight:
       * - Reads: Stash Account, Bonded, Slashing Spans, Locks
       * - Writes: Bonded, Slashing Spans (if S > 0), Ledger, Payee, Validators, Nominators, Stash Account, Locks
       * - Writes Each: SpanSlash * S
       * # </weight>
       **/
      reapStash: AugmentedSubmittable<(stash: AccountId32 | string | Uint8Array, numSlashingSpans: u32 | AnyNumber | Uint8Array) => SubmittableExtrinsic<ApiType>, [AccountId32, u32]>;
      /**
       * Rebond a portion of the stash scheduled to be unlocked.
       * 
       * The dispatch origin must be signed by the controller, and it can be only called when
       * [`EraElectionStatus`] is `Closed`.
       * 
       * # <weight>
       * - Time complexity: O(L), where L is unlocking chunks
       * - Bounded by `MAX_UNLOCKING_CHUNKS`.
       * - Storage changes: Can't increase storage, only decrease it.
       * ---------------
       * - DB Weight:
       * - Reads: EraElectionStatus, Ledger, Locks, [Origin Account]
       * - Writes: [Origin Account], Locks, Ledger
       * # </weight>
       **/
      rebond: AugmentedSubmittable<(value: Compact<u128> | AnyNumber | Uint8Array) => SubmittableExtrinsic<ApiType>, [Compact<u128>]>;
      /**
       * Remove an identity from the pool of (wannabe) validator identities. Effects are known in the next session.
       * Staking module checks `PermissionedIdentity` to ensure validators have
       * completed KYB compliance
       * 
       * # Arguments
       * * origin Required origin for removing a potential validator.
       * * identity Validator's IdentityId.
       **/
      removePermissionedValidator: AugmentedSubmittable<(identity: PolymeshPrimitivesIdentityId | string | Uint8Array) => SubmittableExtrinsic<ApiType>, [PolymeshPrimitivesIdentityId]>;
      /**
       * Scale up the ideal number of validators by a factor.
       * 
       * The dispatch origin must be Root.
       * 
       * # <weight>
       * Same as [`set_validator_count`].
       * # </weight>
       **/
      scaleValidatorCount: AugmentedSubmittable<(factor: Percent | AnyNumber | Uint8Array) => SubmittableExtrinsic<ApiType>, [Percent]>;
      /**
       * Changes commission rate which applies to all validators. Only Governance
       * committee is allowed to change this value.
       * 
       * # Arguments
       * * `new_cap` the new commission cap.
       **/
      setCommissionCap: AugmentedSubmittable<(newCap: Perbill | AnyNumber | Uint8Array) => SubmittableExtrinsic<ApiType>, [Perbill]>;
      /**
       * (Re-)set the controller of a stash.
       * 
       * Effects will be felt at the beginning of the next era.
       * 
       * The dispatch origin for this call must be _Signed_ by the stash, not the controller.
       * 
       * # <weight>
       * - Independent of the arguments. Insignificant complexity.
       * - Contains a limited number of reads.
       * - Writes are limited to the `origin` account key.
       * ----------
       * Weight: O(1)
       * DB Weight:
       * - Read: Bonded, Ledger New Controller, Ledger Old Controller
       * - Write: Bonded, Ledger New Controller, Ledger Old Controller
       * # </weight>
       **/
      setController: AugmentedSubmittable<(controller: MultiAddress | { Id: any } | { Index: any } | { Raw: any } | { Address32: any } | { Address20: any } | string | Uint8Array) => SubmittableExtrinsic<ApiType>, [MultiAddress]>;
      /**
       * Set `HistoryDepth` value. This function will delete any history information
       * when `HistoryDepth` is reduced.
       * 
       * Parameters:
       * - `new_history_depth`: The new history depth you would like to set.
       * - `era_items_deleted`: The number of items that will be deleted by this dispatch.
       * This should report all the storage items that will be deleted by clearing old
       * era history. Needed to report an accurate weight for the dispatch. Trusted by
       * `Root` to report an accurate number.
       * 
       * Origin must be root.
       * 
       * # <weight>
       * - E: Number of history depths removed, i.e. 10 -> 7 = 3
       * - Weight: O(E)
       * - DB Weight:
       * - Reads: Current Era, History Depth
       * - Writes: History Depth
       * - Clear Prefix Each: Era Stakers, EraStakersClipped, ErasValidatorPrefs
       * - Writes Each: ErasValidatorReward, ErasRewardPoints, ErasTotalStake, ErasStartSessionIndex
       * # </weight>
       **/
      setHistoryDepth: AugmentedSubmittable<(newHistoryDepth: Compact<u32> | AnyNumber | Uint8Array, eraItemsDeleted: Compact<u32> | AnyNumber | Uint8Array) => SubmittableExtrinsic<ApiType>, [Compact<u32>, Compact<u32>]>;
      /**
       * Set the validators who cannot be slashed (if any).
       * 
       * The dispatch origin must be Root.
       * 
       * # <weight>
       * - O(V)
       * - Write: Invulnerables
       * # </weight>
       **/
      setInvulnerables: AugmentedSubmittable<(invulnerables: Vec<AccountId32> | (AccountId32 | string | Uint8Array)[]) => SubmittableExtrinsic<ApiType>, [Vec<AccountId32>]>;
      /**
       * Changes min bond value to be used in validate(). Only Governance
       * committee is allowed to change this value.
       * 
       * # Arguments
       * * `new_value` the new minimum
       **/
      setMinBondThreshold: AugmentedSubmittable<(newValue: u128 | AnyNumber | Uint8Array) => SubmittableExtrinsic<ApiType>, [u128]>;
      /**
       * (Re-)set the payment target for a controller.
       * 
       * Effects will be felt at the beginning of the next era.
       * 
       * The dispatch origin for this call must be _Signed_ by the controller, not the stash.
       * 
       * # <weight>
       * - Independent of the arguments. Insignificant complexity.
       * - Contains a limited number of reads.
       * - Writes are limited to the `origin` account key.
       * ---------
       * - Weight: O(1)
       * - DB Weight:
       * - Read: Ledger
       * - Write: Payee
       * # </weight>
       **/
      setPayee: AugmentedSubmittable<(payee: PalletStakingRewardDestination | { Staked: any } | { Stash: any } | { Controller: any } | { Account: any } | string | Uint8Array) => SubmittableExtrinsic<ApiType>, [PalletStakingRewardDestination]>;
      /**
       * Sets the ideal number of validators.
       * 
       * The dispatch origin must be Root.
       * 
       * # <weight>
       * Weight: O(1)
       * Write: Validator Count
       * # </weight>
       **/
      setValidatorCount: AugmentedSubmittable<(updated: Compact<u32> | AnyNumber | Uint8Array) => SubmittableExtrinsic<ApiType>, [Compact<u32>]>;
      /**
       * Submit an election result to the chain. If the solution:
       * 
       * 1. is valid.
       * 2. has a better score than a potentially existing solution on chain.
       * 
       * then, it will be _put_ on chain.
       * 
       * A solution consists of two pieces of data:
       * 
       * 1. `winners`: a flat vector of all the winners of the round.
       * 2. `assignments`: the compact version of an assignment vector that encodes the edge
       * weights.
       * 
       * Both of which may be computed using _phragmen_, or any other algorithm.
       * 
       * Additionally, the submitter must provide:
       * 
       * - The `score` that they claim their solution has.
       * 
       * Both validators and nominators will be represented by indices in the solution. The
       * indices should respect the corresponding types ([`ValidatorIndex`] and
       * [`NominatorIndex`]). Moreover, they should be valid when used to index into
       * [`SnapshotValidators`] and [`SnapshotNominators`]. Any invalid index will cause the
       * solution to be rejected. These two storage items are set during the election window and
       * may be used to determine the indices.
       * 
       * A solution is valid if:
       * 
       * 0. It is submitted when [`EraElectionStatus`] is `Open`.
       * 1. Its claimed score is equal to the score computed on-chain.
       * 2. Presents the correct number of winners.
       * 3. All indexes must be value according to the snapshot vectors. All edge values must
       * also be correct and should not overflow the granularity of the ratio type (i.e. 256
       * or billion).
       * 4. For each edge, all targets are actually nominated by the voter.
       * 5. Has correct self-votes.
       * 
       * A solutions score is consisted of 3 parameters:
       * 
       * 1. `min { support.total }` for each support of a winner. This value should be maximized.
       * 2. `sum { support.total }` for each support of a winner. This value should be minimized.
       * 3. `sum { support.total^2 }` for each support of a winner. This value should be
       * minimized (to ensure less variance)
       * 
       * # <weight>
       * The transaction is assumed to be the longest path, a better solution.
       * - Initial solution is almost the same.
       * - Worse solution is retraced in pre-dispatch-checks which sets its own weight.
       * # </weight>
       **/
      submitElectionSolution: AugmentedSubmittable<(winners: Vec<u16> | (u16 | AnyNumber | Uint8Array)[], compact: PalletStakingCompactAssignments | { votes1?: any; votes2?: any; votes3?: any; votes4?: any; votes5?: any; votes6?: any; votes7?: any; votes8?: any; votes9?: any; votes10?: any; votes11?: any; votes12?: any; votes13?: any; votes14?: any; votes15?: any; votes16?: any } | string | Uint8Array, score: SpNposElectionsElectionScore | { minimalStake?: any; sumStake?: any; sumStakeSquared?: any } | string | Uint8Array, era: u32 | AnyNumber | Uint8Array, size: PalletStakingElectionSize | { validators?: any; nominators?: any } | string | Uint8Array) => SubmittableExtrinsic<ApiType>, [Vec<u16>, PalletStakingCompactAssignments, SpNposElectionsElectionScore, u32, PalletStakingElectionSize]>;
      /**
       * Unsigned version of `submit_election_solution`.
       * 
       * Note that this must pass the [`ValidateUnsigned`] check which only allows transactions
       * from the local node to be included. In other words, only the block author can include a
       * transaction in the block.
       * 
       * # <weight>
       * See [`submit_election_solution`].
       * # </weight>
       **/
      submitElectionSolutionUnsigned: AugmentedSubmittable<(winners: Vec<u16> | (u16 | AnyNumber | Uint8Array)[], compact: PalletStakingCompactAssignments | { votes1?: any; votes2?: any; votes3?: any; votes4?: any; votes5?: any; votes6?: any; votes7?: any; votes8?: any; votes9?: any; votes10?: any; votes11?: any; votes12?: any; votes13?: any; votes14?: any; votes15?: any; votes16?: any } | string | Uint8Array, score: SpNposElectionsElectionScore | { minimalStake?: any; sumStake?: any; sumStakeSquared?: any } | string | Uint8Array, era: u32 | AnyNumber | Uint8Array, size: PalletStakingElectionSize | { validators?: any; nominators?: any } | string | Uint8Array) => SubmittableExtrinsic<ApiType>, [Vec<u16>, PalletStakingCompactAssignments, SpNposElectionsElectionScore, u32, PalletStakingElectionSize]>;
      /**
       * Schedule a portion of the stash to be unlocked ready for transfer out after the bond
       * period ends. If this leaves an amount actively bonded less than
       * T::Currency::minimum_balance(), then it is increased to the full amount.
       * 
       * Once the unlock period is done, you can call `withdraw_unbonded` to actually move
       * the funds out of management ready for transfer.
       * 
       * No more than a limited number of unlocking chunks (see `MAX_UNLOCKING_CHUNKS`)
       * can co-exists at the same time. In that case, [`Call::withdraw_unbonded`] need
       * to be called first to remove some of the chunks (if possible).
       * 
       * The dispatch origin for this call must be _Signed_ by the controller, not the stash.
       * And, it can be only called when [`EraElectionStatus`] is `Closed`.
       * 
       * Emits `Unbonded`.
       * 
       * See also [`Call::withdraw_unbonded`].
       * 
       * # <weight>
       * - Independent of the arguments. Limited but potentially exploitable complexity.
       * - Contains a limited number of reads.
       * - Each call (requires the remainder of the bonded balance to be above `minimum_balance`)
       * will cause a new entry to be inserted into a vector (`Ledger.unlocking`) kept in storage.
       * The only way to clean the aforementioned storage item is also user-controlled via
       * `withdraw_unbonded`.
       * - One DB entry.
       * ----------
       * Weight: O(1)
       * DB Weight:
       * - Read: EraElectionStatus, Ledger, CurrentEra, Locks, \[Origin Account\]
       * - Write: Locks, Ledger, \[Origin Account\]
       * </weight>
       **/
      unbond: AugmentedSubmittable<(value: Compact<u128> | AnyNumber | Uint8Array) => SubmittableExtrinsic<ApiType>, [Compact<u128>]>;
      /**
       * Update the intended validator count for a given DID.
       * 
       * # Arguments
       * * origin which must be the required origin for adding a potential validator.
       * * identity to add as a validator.
       * * new_intended_count New value of intended count.
       **/
      updatePermissionedValidatorIntendedCount: AugmentedSubmittable<(identity: PolymeshPrimitivesIdentityId | string | Uint8Array, newIntendedCount: u32 | AnyNumber | Uint8Array) => SubmittableExtrinsic<ApiType>, [PolymeshPrimitivesIdentityId, u32]>;
      /**
       * Declare the desire to validate for the origin controller.
       * 
       * Effects will be felt at the beginning of the next era.
       * 
       * The dispatch origin for this call must be _Signed_ by the controller, not the stash.
       * And, it can be only called when [`EraElectionStatus`] is `Closed`.
       * 
       * # <weight>
       * - Independent of the arguments. Insignificant complexity.
       * - Contains a limited number of reads.
       * - Writes are limited to the `origin` account key.
       * -----------
       * Weight: O(1)
       * DB Weight:
       * - Read: Era Election Status, Ledger
       * - Write: Nominators, Validators
       * # </weight>
       **/
      validate: AugmentedSubmittable<(prefs: PalletStakingValidatorPrefs | { commission?: any; blocked?: any } | string | Uint8Array) => SubmittableExtrinsic<ApiType>, [PalletStakingValidatorPrefs]>;
      /**
       * Validate the nominators CDD expiry time.
       * 
       * If an account from a given set of address is nominating then
       * check the CDD expiry time of it and if it is expired
       * then the account should be unbonded and removed from the nominating process.
       * 
       * #<weight>
       * - Depends on passed list of AccountId.
       * - Depends on the no. of claim issuers an accountId has for the CDD expiry.
       * #</weight>
       **/
      validateCddExpiryNominators: AugmentedSubmittable<(targets: Vec<AccountId32> | (AccountId32 | string | Uint8Array)[]) => SubmittableExtrinsic<ApiType>, [Vec<AccountId32>]>;
      /**
       * Remove any unlocked chunks from the `unlocking` queue from our management.
       * 
       * This essentially frees up that balance to be used by the stash account to do
       * whatever it wants.
       * 
       * The dispatch origin for this call must be _Signed_ by the controller, not the stash.
       * And, it can be only called when [`EraElectionStatus`] is `Closed`.
       * 
       * Emits `Withdrawn`.
       * 
       * See also [`Call::unbond`].
       * 
       * # <weight>
       * - Could be dependent on the `origin` argument and how much `unlocking` chunks exist.
       * It implies `consolidate_unlocked` which loops over `Ledger.unlocking`, which is
       * indirectly user-controlled. See [`unbond`] for more detail.
       * - Contains a limited number of reads, yet the size of which could be large based on `ledger`.
       * - Writes are limited to the `origin` account key.
       * ---------------
       * Complexity O(S) where S is the number of slashing spans to remove
       * Update:
       * - Reads: EraElectionStatus, Ledger, Current Era, Locks, [Origin Account]
       * - Writes: [Origin Account], Locks, Ledger
       * Kill:
       * - Reads: EraElectionStatus, Ledger, Current Era, Bonded, Slashing Spans, [Origin
       * Account], Locks
       * - Writes: Bonded, Slashing Spans (if S > 0), Ledger, Payee, Validators, Nominators,
       * [Origin Account], Locks
       * - Writes Each: SpanSlash * S
       * NOTE: Weight annotation is the kill scenario, we refund otherwise.
       * # </weight>
       **/
      withdrawUnbonded: AugmentedSubmittable<(numSlashingSpans: u32 | AnyNumber | Uint8Array) => SubmittableExtrinsic<ApiType>, [u32]>;
      /**
       * Generic tx
       **/
      [key: string]: SubmittableExtrinsicFunction<ApiType>;
    };
    statistics: {
      /**
       * Allow a trusted issuer to init/resync ticker/company stats.
       * 
       * # Arguments
       * - `origin` - a signer that has permissions to act as an agent of `asset`.
       * - `asset` - the asset to change the active stats on.
       * - `stat_type` - stat type to update.
       * - `values` - Updated values for `stat_type`.
       * 
       * # Errors
       * - `StatTypeMissing` - `stat_type` is not enabled for the `asset`.
       * - `UnauthorizedAgent` if `origin` is not agent-permissioned for `asset`.
       * 
       * # Permissions
       * - Agent
       * - Asset
       **/
      batchUpdateAssetStats: AugmentedSubmittable<(asset: PolymeshPrimitivesStatisticsAssetScope | { Ticker: any } | string | Uint8Array, statType: PolymeshPrimitivesStatisticsStatType | { op?: any; claimIssuer?: any } | string | Uint8Array, values: BTreeSetStatUpdate) => SubmittableExtrinsic<ApiType>, [PolymeshPrimitivesStatisticsAssetScope, PolymeshPrimitivesStatisticsStatType, BTreeSetStatUpdate]>;
      /**
       * Set the active asset stat_types.
       * 
       * # Arguments
       * - `origin` - a signer that has permissions to act as an agent of `asset`.
       * - `asset` - the asset to change the active stats on.
       * - `stat_types` - the new stat types to replace any existing types.
       * 
       * # Errors
       * - `StatTypeLimitReached` - too many stat types enabled for the `asset`.
       * - `CannotRemoveStatTypeInUse` - can not remove a stat type that is in use by transfer conditions.
       * - `UnauthorizedAgent` if `origin` is not agent-permissioned for `asset`.
       * 
       * # Permissions
       * - Agent
       * - Asset
       **/
      setActiveAssetStats: AugmentedSubmittable<(asset: PolymeshPrimitivesStatisticsAssetScope | { Ticker: any } | string | Uint8Array, statTypes: BTreeSetStatType) => SubmittableExtrinsic<ApiType>, [PolymeshPrimitivesStatisticsAssetScope, BTreeSetStatType]>;
      /**
       * Set asset transfer compliance rules.
       * 
       * # Arguments
       * - `origin` - a signer that has permissions to act as an agent of `asset`.
       * - `asset` - the asset to change the active stats on.
       * - `transfer_conditions` - the new transfer condition to replace any existing conditions.
       * 
       * # Errors
       * - `TransferConditionLimitReached` - too many transfer condititon enabled for `asset`.
       * - `StatTypeMissing` - a transfer condition requires a stat type that is not enabled for the `asset`.
       * - `UnauthorizedAgent` if `origin` is not agent-permissioned for `asset`.
       * 
       * # Permissions
       * - Agent
       * - Asset
       **/
      setAssetTransferCompliance: AugmentedSubmittable<(asset: PolymeshPrimitivesStatisticsAssetScope | { Ticker: any } | string | Uint8Array, transferConditions: BTreeSetTransferCondition) => SubmittableExtrinsic<ApiType>, [PolymeshPrimitivesStatisticsAssetScope, BTreeSetTransferCondition]>;
      /**
       * Set/unset entities exempt from an asset's transfer compliance rules.
       * 
       * # Arguments
       * - `origin` - a signer that has permissions to act as an agent of `exempt_key.asset`.
       * - `is_exempt` - enable/disable exemption for `entities`.
       * - `exempt_key` - the asset and stat type to exempt the `entities` from.
       * - `entities` - the entities to set/unset the exemption for.
       * 
       * # Errors
       * - `UnauthorizedAgent` if `origin` is not agent-permissioned for `asset`.
       * 
       * # Permissions
       * - Agent
       * - Asset
       **/
      setEntitiesExempt: AugmentedSubmittable<(isExempt: bool | boolean | Uint8Array, exemptKey: PolymeshPrimitivesTransferComplianceTransferConditionExemptKey | { asset?: any; op?: any; claimType?: any } | string | Uint8Array, entities: BTreeSetIdentityId) => SubmittableExtrinsic<ApiType>, [bool, PolymeshPrimitivesTransferComplianceTransferConditionExemptKey, BTreeSetIdentityId]>;
      /**
       * Generic tx
       **/
      [key: string]: SubmittableExtrinsicFunction<ApiType>;
    };
    sto: {
      /**
       * Create a new fundraiser.
       * 
       * * `offering_portfolio` - Portfolio containing the `offering_asset`.
       * * `offering_asset` - Asset being offered.
       * * `raising_portfolio` - Portfolio containing the `raising_asset`.
       * * `raising_asset` - Asset being exchanged for `offering_asset` on investment.
       * * `tiers` - Price tiers to charge investors on investment.
       * * `venue_id` - Venue to handle settlement.
       * * `start` - Fundraiser start time, if `None` the fundraiser will start immediately.
       * * `end` - Fundraiser end time, if `None` the fundraiser will never expire.
       * * `minimum_investment` - Minimum amount of `raising_asset` that an investor needs to spend to invest in this raise.
       * * `fundraiser_name` - Fundraiser name, only used in the UIs.
       * 
       * # Permissions
       * * Asset
       * * Portfolio
       **/
      createFundraiser: AugmentedSubmittable<(offeringPortfolio: PolymeshPrimitivesIdentityIdPortfolioId | { did?: any; kind?: any } | string | Uint8Array, offeringAsset: PolymeshPrimitivesTicker | string | Uint8Array, raisingPortfolio: PolymeshPrimitivesIdentityIdPortfolioId | { did?: any; kind?: any } | string | Uint8Array, raisingAsset: PolymeshPrimitivesTicker | string | Uint8Array, tiers: Vec<PalletStoPriceTier> | (PalletStoPriceTier | { total?: any; price?: any } | string | Uint8Array)[], venueId: u64 | AnyNumber | Uint8Array, start: Option<u64> | null | object | string | Uint8Array, end: Option<u64> | null | object | string | Uint8Array, minimumInvestment: u128 | AnyNumber | Uint8Array, fundraiserName: Bytes | string | Uint8Array) => SubmittableExtrinsic<ApiType>, [PolymeshPrimitivesIdentityIdPortfolioId, PolymeshPrimitivesTicker, PolymeshPrimitivesIdentityIdPortfolioId, PolymeshPrimitivesTicker, Vec<PalletStoPriceTier>, u64, Option<u64>, Option<u64>, u128, Bytes]>;
      /**
       * Freeze a fundraiser.
       * 
       * * `offering_asset` - Asset to freeze.
       * * `id` - ID of the fundraiser to freeze.
       * 
       * # Permissions
       * * Asset
       **/
      freezeFundraiser: AugmentedSubmittable<(offeringAsset: PolymeshPrimitivesTicker | string | Uint8Array, id: u64 | AnyNumber | Uint8Array) => SubmittableExtrinsic<ApiType>, [PolymeshPrimitivesTicker, u64]>;
      /**
       * Invest in a fundraiser.
       * 
       * * `investment_portfolio` - Portfolio that `offering_asset` will be deposited in.
       * * `funding_portfolio` - Portfolio that will fund the investment.
       * * `offering_asset` - Asset to invest in.
       * * `id` - ID of the fundraiser to invest in.
       * * `purchase_amount` - Amount of `offering_asset` to purchase.
       * * `max_price` - Maximum price to pay per unit of `offering_asset`, If `None`there are no constraints on price.
       * * `receipt` - Off-chain receipt to use instead of on-chain balance in `funding_portfolio`.
       * 
       * # Permissions
       * * Portfolio
       **/
      invest: AugmentedSubmittable<(investmentPortfolio: PolymeshPrimitivesIdentityIdPortfolioId | { did?: any; kind?: any } | string | Uint8Array, fundingPortfolio: PolymeshPrimitivesIdentityIdPortfolioId | { did?: any; kind?: any } | string | Uint8Array, offeringAsset: PolymeshPrimitivesTicker | string | Uint8Array, id: u64 | AnyNumber | Uint8Array, purchaseAmount: u128 | AnyNumber | Uint8Array, maxPrice: Option<u128> | null | object | string | Uint8Array, receipt: Option<PalletSettlementReceiptDetails> | null | object | string | Uint8Array) => SubmittableExtrinsic<ApiType>, [PolymeshPrimitivesIdentityIdPortfolioId, PolymeshPrimitivesIdentityIdPortfolioId, PolymeshPrimitivesTicker, u64, u128, Option<u128>, Option<PalletSettlementReceiptDetails>]>;
      /**
       * Modify the time window a fundraiser is active
       * 
       * * `offering_asset` - Asset to modify.
       * * `id` - ID of the fundraiser to modify.
       * * `start` - New start of the fundraiser.
       * * `end` - New end of the fundraiser to modify.
       * 
       * # Permissions
       * * Asset
       **/
      modifyFundraiserWindow: AugmentedSubmittable<(offeringAsset: PolymeshPrimitivesTicker | string | Uint8Array, id: u64 | AnyNumber | Uint8Array, start: u64 | AnyNumber | Uint8Array, end: Option<u64> | null | object | string | Uint8Array) => SubmittableExtrinsic<ApiType>, [PolymeshPrimitivesTicker, u64, u64, Option<u64>]>;
      /**
       * Stop a fundraiser.
       * 
       * * `offering_asset` - Asset to stop.
       * * `id` - ID of the fundraiser to stop.
       * 
       * # Permissions
       * * Asset
       **/
      stop: AugmentedSubmittable<(offeringAsset: PolymeshPrimitivesTicker | string | Uint8Array, id: u64 | AnyNumber | Uint8Array) => SubmittableExtrinsic<ApiType>, [PolymeshPrimitivesTicker, u64]>;
      /**
       * Unfreeze a fundraiser.
       * 
       * * `offering_asset` - Asset to unfreeze.
       * * `id` - ID of the fundraiser to unfreeze.
       * 
       * # Permissions
       * * Asset
       **/
      unfreezeFundraiser: AugmentedSubmittable<(offeringAsset: PolymeshPrimitivesTicker | string | Uint8Array, id: u64 | AnyNumber | Uint8Array) => SubmittableExtrinsic<ApiType>, [PolymeshPrimitivesTicker, u64]>;
      /**
       * Generic tx
       **/
      [key: string]: SubmittableExtrinsicFunction<ApiType>;
    };
    sudo: {
      /**
       * Authenticates the current sudo key and sets the given AccountId (`new`) as the new sudo key.
       * 
       * The dispatch origin for this call must be _Signed_.
       * 
       * # <weight>
       * - O(1).
       * - Limited storage reads.
       * - One DB change.
       * # </weight>
       **/
      setKey: AugmentedSubmittable<(updated: MultiAddress | { Id: any } | { Index: any } | { Raw: any } | { Address32: any } | { Address20: any } | string | Uint8Array) => SubmittableExtrinsic<ApiType>, [MultiAddress]>;
      /**
       * Authenticates the sudo key and dispatches a function call with `Root` origin.
       * 
       * The dispatch origin for this call must be _Signed_.
       * 
       * # <weight>
       * - O(1).
       * - Limited storage reads.
       * - One DB write (event).
       * - Weight of derivative `call` execution + 10,000.
       * # </weight>
       **/
      sudo: AugmentedSubmittable<(call: Call | { callIndex?: any; args?: any } | string | Uint8Array) => SubmittableExtrinsic<ApiType>, [Call]>;
      /**
       * Authenticates the sudo key and dispatches a function call with `Signed` origin from
       * a given account.
       * 
       * The dispatch origin for this call must be _Signed_.
       * 
       * # <weight>
       * - O(1).
       * - Limited storage reads.
       * - One DB write (event).
       * - Weight of derivative `call` execution + 10,000.
       * # </weight>
       **/
      sudoAs: AugmentedSubmittable<(who: MultiAddress | { Id: any } | { Index: any } | { Raw: any } | { Address32: any } | { Address20: any } | string | Uint8Array, call: Call | { callIndex?: any; args?: any } | string | Uint8Array) => SubmittableExtrinsic<ApiType>, [MultiAddress, Call]>;
      /**
       * Authenticates the sudo key and dispatches a function call with `Root` origin.
       * This function does not check the weight of the call, and instead allows the
       * Sudo user to specify the weight of the call.
       * 
       * The dispatch origin for this call must be _Signed_.
       * 
       * # <weight>
       * - O(1).
       * - The weight of this call is defined by the caller.
       * # </weight>
       **/
      sudoUncheckedWeight: AugmentedSubmittable<(call: Call | { callIndex?: any; args?: any } | string | Uint8Array, weight: u64 | AnyNumber | Uint8Array) => SubmittableExtrinsic<ApiType>, [Call, u64]>;
      /**
       * Generic tx
       **/
      [key: string]: SubmittableExtrinsicFunction<ApiType>;
    };
    system: {
      /**
       * A dispatch that will fill the block weight up to the given ratio.
       **/
      fillBlock: AugmentedSubmittable<(ratio: Perbill | AnyNumber | Uint8Array) => SubmittableExtrinsic<ApiType>, [Perbill]>;
      /**
       * Kill all storage items with a key that starts with the given prefix.
       * 
       * **NOTE:** We rely on the Root origin to provide us the number of subkeys under
       * the prefix we are removing to accurately calculate the weight of this function.
       **/
      killPrefix: AugmentedSubmittable<(prefix: Bytes | string | Uint8Array, subkeys: u32 | AnyNumber | Uint8Array) => SubmittableExtrinsic<ApiType>, [Bytes, u32]>;
      /**
       * Kill some items from storage.
       **/
      killStorage: AugmentedSubmittable<(keys: Vec<Bytes> | (Bytes | string | Uint8Array)[]) => SubmittableExtrinsic<ApiType>, [Vec<Bytes>]>;
      /**
       * Make some on-chain remark.
       * 
       * # <weight>
       * - `O(1)`
       * # </weight>
       **/
      remark: AugmentedSubmittable<(remark: Bytes | string | Uint8Array) => SubmittableExtrinsic<ApiType>, [Bytes]>;
      /**
       * Make some on-chain remark and emit event.
       **/
      remarkWithEvent: AugmentedSubmittable<(remark: Bytes | string | Uint8Array) => SubmittableExtrinsic<ApiType>, [Bytes]>;
      /**
       * Set the new runtime code.
       * 
       * # <weight>
       * - `O(C + S)` where `C` length of `code` and `S` complexity of `can_set_code`
       * - 1 call to `can_set_code`: `O(S)` (calls `sp_io::misc::runtime_version` which is
       * expensive).
       * - 1 storage write (codec `O(C)`).
       * - 1 digest item.
       * - 1 event.
       * The weight of this function is dependent on the runtime, but generally this is very
       * expensive. We will treat this as a full block.
       * # </weight>
       **/
      setCode: AugmentedSubmittable<(code: Bytes | string | Uint8Array) => SubmittableExtrinsic<ApiType>, [Bytes]>;
      /**
       * Set the new runtime code without doing any checks of the given `code`.
       * 
       * # <weight>
       * - `O(C)` where `C` length of `code`
       * - 1 storage write (codec `O(C)`).
       * - 1 digest item.
       * - 1 event.
       * The weight of this function is dependent on the runtime. We will treat this as a full
       * block. # </weight>
       **/
      setCodeWithoutChecks: AugmentedSubmittable<(code: Bytes | string | Uint8Array) => SubmittableExtrinsic<ApiType>, [Bytes]>;
      /**
       * Set the number of pages in the WebAssembly environment's heap.
       **/
      setHeapPages: AugmentedSubmittable<(pages: u64 | AnyNumber | Uint8Array) => SubmittableExtrinsic<ApiType>, [u64]>;
      /**
       * Set some items of storage.
       **/
      setStorage: AugmentedSubmittable<(items: Vec<ITuple<[Bytes, Bytes]>> | ([Bytes | string | Uint8Array, Bytes | string | Uint8Array])[]) => SubmittableExtrinsic<ApiType>, [Vec<ITuple<[Bytes, Bytes]>>]>;
      /**
       * Generic tx
       **/
      [key: string]: SubmittableExtrinsicFunction<ApiType>;
    };
    technicalCommittee: {
      /**
       * Changes the time after which a proposal expires.
       * 
       * # Arguments
       * * `expiry` - The new expiry time.
       **/
      setExpiresAfter: AugmentedSubmittable<(expiry: PolymeshCommonUtilitiesMaybeBlock | { Some: any } | { None: any } | string | Uint8Array) => SubmittableExtrinsic<ApiType>, [PolymeshCommonUtilitiesMaybeBlock]>;
      /**
       * Changes the release coordinator.
       * 
       * # Arguments
       * * `id` - The DID of the new release coordinator.
       * 
       * # Errors
       * * `NotAMember`, If the new coordinator `id` is not part of the committee.
       **/
      setReleaseCoordinator: AugmentedSubmittable<(id: PolymeshPrimitivesIdentityId | string | Uint8Array) => SubmittableExtrinsic<ApiType>, [PolymeshPrimitivesIdentityId]>;
      /**
       * Change the vote threshold the determines the winning proposal.
       * For e.g., for a simple majority use (1, 2) which represents the in-equation ">= 1/2".
       * 
       * # Arguments
       * * `n` - Numerator of the fraction representing vote threshold.
       * * `d` - Denominator of the fraction representing vote threshold.
       **/
      setVoteThreshold: AugmentedSubmittable<(n: u32 | AnyNumber | Uint8Array, d: u32 | AnyNumber | Uint8Array) => SubmittableExtrinsic<ApiType>, [u32, u32]>;
      /**
       * Votes `approve`ingly (or not, if `false`)
       * on an existing `proposal` given by its hash, `index`.
       * 
       * # Arguments
       * * `proposal` - A hash of the proposal to be voted on.
       * * `index` - The proposal index.
       * * `approve` - If `true` than this is a `for` vote, and `against` otherwise.
       * 
       * # Errors
       * * `NotAMember`, if the `origin` is not a member of this committee.
       **/
      vote: AugmentedSubmittable<(proposal: H256 | string | Uint8Array, index: u32 | AnyNumber | Uint8Array, approve: bool | boolean | Uint8Array) => SubmittableExtrinsic<ApiType>, [H256, u32, bool]>;
      /**
       * Proposes to the committee that `call` should be executed in its name.
       * Alternatively, if the hash of `call` has already been recorded, i.e., already proposed,
       * then this call counts as a vote, i.e., as if `vote_by_hash` was called.
       * 
       * # Weight
       * 
       * The weight of this dispatchable is that of `call` as well as the complexity
       * for recording the vote itself.
       * 
       * # Arguments
       * * `approve` - is this an approving vote?
       * If the proposal doesn't exist, passing `false` will result in error `FirstVoteReject`.
       * * `call` - the call to propose for execution.
       * 
       * # Errors
       * * `FirstVoteReject`, if `call` hasn't been proposed and `approve == false`.
       * * `NotAMember`, if the `origin` is not a member of this committee.
       **/
      voteOrPropose: AugmentedSubmittable<(approve: bool | boolean | Uint8Array, call: Call | { callIndex?: any; args?: any } | string | Uint8Array) => SubmittableExtrinsic<ApiType>, [bool, Call]>;
      /**
       * Generic tx
       **/
      [key: string]: SubmittableExtrinsicFunction<ApiType>;
    };
    technicalCommitteeMembership: {
      /**
       * Allows the calling member to *unilaterally quit* without this being subject to a GC
       * vote.
       * 
       * # Arguments
       * * `origin` - Member of committee who wants to quit.
       * 
       * # Error
       * 
       * * Only primary key can abdicate.
       * * Last member of a group cannot abdicate.
       **/
      abdicateMembership: AugmentedSubmittable<() => SubmittableExtrinsic<ApiType>, []>;
      /**
       * Adds a member `who` to the group. May only be called from `AddOrigin` or root.
       * 
       * # Arguments
       * * `origin` - Origin representing `AddOrigin` or root
       * * `who` - IdentityId to be added to the group.
       **/
      addMember: AugmentedSubmittable<(who: PolymeshPrimitivesIdentityId | string | Uint8Array) => SubmittableExtrinsic<ApiType>, [PolymeshPrimitivesIdentityId]>;
      /**
       * Disables a member at specific moment.
       * 
       * Please note that if member is already revoked (a "valid member"), its revocation
       * time-stamp will be updated.
       * 
       * Any disabled member should NOT allow to act like an active member of the group. For
       * instance, a disabled CDD member should NOT be able to generate a CDD claim. However any
       * generated claim issued before `at` would be considered as a valid one.
       * 
       * If you want to invalidate any generated claim, you should use `Self::remove_member`.
       * 
       * # Arguments
       * * `at` - Revocation time-stamp.
       * * `who` - Target member of the group.
       * * `expiry` - Time-stamp when `who` is removed from CDD. As soon as it is expired, the
       * generated claims will be "invalid" as `who` is not considered a member of the group.
       **/
      disableMember: AugmentedSubmittable<(who: PolymeshPrimitivesIdentityId | string | Uint8Array, expiry: Option<u64> | null | object | string | Uint8Array, at: Option<u64> | null | object | string | Uint8Array) => SubmittableExtrinsic<ApiType>, [PolymeshPrimitivesIdentityId, Option<u64>, Option<u64>]>;
      /**
       * Removes a member `who` from the set. May only be called from `RemoveOrigin` or root.
       * 
       * Any claim previously generated by this member is not valid as a group claim. For
       * instance, if a CDD member group generated a claim for a target identity and then it is
       * removed, that claim will be invalid.  In case you want to keep the validity of generated
       * claims, you have to use `Self::disable_member` function
       * 
       * # Arguments
       * * `origin` - Origin representing `RemoveOrigin` or root
       * * `who` - IdentityId to be removed from the group.
       **/
      removeMember: AugmentedSubmittable<(who: PolymeshPrimitivesIdentityId | string | Uint8Array) => SubmittableExtrinsic<ApiType>, [PolymeshPrimitivesIdentityId]>;
      /**
       * Changes the membership to a new set, disregarding the existing membership.
       * May only be called from `ResetOrigin` or root.
       * 
       * # Arguments
       * * `origin` - Origin representing `ResetOrigin` or root
       * * `members` - New set of identities
       **/
      resetMembers: AugmentedSubmittable<(members: Vec<PolymeshPrimitivesIdentityId> | (PolymeshPrimitivesIdentityId | string | Uint8Array)[]) => SubmittableExtrinsic<ApiType>, [Vec<PolymeshPrimitivesIdentityId>]>;
      /**
       * Change this group's limit for how many concurrent active members they may be.
       * 
       * # Arguments
       * * `limit` - the number of active members there may be concurrently.
       **/
      setActiveMembersLimit: AugmentedSubmittable<(limit: u32 | AnyNumber | Uint8Array) => SubmittableExtrinsic<ApiType>, [u32]>;
      /**
       * Swaps out one member `remove` for another member `add`.
       * 
       * May only be called from `SwapOrigin` or root.
       * 
       * # Arguments
       * * `origin` - Origin representing `SwapOrigin` or root
       * * `remove` - IdentityId to be removed from the group.
       * * `add` - IdentityId to be added in place of `remove`.
       **/
      swapMember: AugmentedSubmittable<(remove: PolymeshPrimitivesIdentityId | string | Uint8Array, add: PolymeshPrimitivesIdentityId | string | Uint8Array) => SubmittableExtrinsic<ApiType>, [PolymeshPrimitivesIdentityId, PolymeshPrimitivesIdentityId]>;
      /**
       * Generic tx
       **/
      [key: string]: SubmittableExtrinsicFunction<ApiType>;
    };
    testUtils: {
      /**
       * Emits an event with caller's identity and CDD status.
       **/
      getCddOf: AugmentedSubmittable<(of: AccountId32 | string | Uint8Array) => SubmittableExtrinsic<ApiType>, [AccountId32]>;
      /**
       * Emits an event with caller's identity.
       **/
      getMyDid: AugmentedSubmittable<() => SubmittableExtrinsic<ApiType>, []>;
      /**
       * Registers a new Identity for the `target_account` and issues a CDD claim to it.
       * The Investor UID is generated deterministically by the hash of the generated DID and
       * then we fix it to be compliant with UUID v4.
       * 
       * # See
       * - [RFC 4122: UUID](https://tools.ietf.org/html/rfc4122)
       * 
       * # Failure
       * - `origin` has to be an active CDD provider. Inactive CDD providers cannot add new
       * claims.
       * - `target_account` (primary key of the new Identity) can be linked to just one and only
       * one identity.
       **/
      mockCddRegisterDid: AugmentedSubmittable<(targetAccount: AccountId32 | string | Uint8Array) => SubmittableExtrinsic<ApiType>, [AccountId32]>;
      /**
       * Generates a new `IdentityID` for the caller, and issues a self-generated CDD claim.
       * 
       * The caller account will be the primary key of that identity.
       * For each account of `secondary_keys`, a new `JoinIdentity` authorization is created, so
       * each of them will need to accept it before become part of this new `IdentityID`.
       * 
       * # Errors
       * - `AlreadyLinked` if the caller account or if any of the given `secondary_keys` has already linked to an `IdentityID`
       * - `SecondaryKeysContainPrimaryKey` if `secondary_keys` contains the caller account.
       * - `DidAlreadyExists` if auto-generated DID already exists.
       **/
      registerDid: AugmentedSubmittable<(uid: PolymeshPrimitivesCddIdInvestorUid | string | Uint8Array, secondaryKeys: Vec<PolymeshPrimitivesSecondaryKey> | (PolymeshPrimitivesSecondaryKey | { key?: any; permissions?: any } | string | Uint8Array)[]) => SubmittableExtrinsic<ApiType>, [PolymeshPrimitivesCddIdInvestorUid, Vec<PolymeshPrimitivesSecondaryKey>]>;
      /**
       * Generic tx
       **/
      [key: string]: SubmittableExtrinsicFunction<ApiType>;
    };
    timestamp: {
      /**
       * Set the current time.
       * 
       * This call should be invoked exactly once per block. It will panic at the finalization
       * phase, if this call hasn't been invoked by that time.
       * 
       * The timestamp should be greater than the previous one by the amount specified by
       * `MinimumPeriod`.
       * 
       * The dispatch origin for this call must be `Inherent`.
       * 
       * # <weight>
       * - `O(1)` (Note that implementations of `OnTimestampSet` must also be `O(1)`)
       * - 1 storage read and 1 storage mutation (codec `O(1)`). (because of `DidUpdate::take` in
       * `on_finalize`)
       * - 1 event handler `on_timestamp_set`. Must be `O(1)`.
       * # </weight>
       **/
      set: AugmentedSubmittable<(now: Compact<u64> | AnyNumber | Uint8Array) => SubmittableExtrinsic<ApiType>, [Compact<u64>]>;
      /**
       * Generic tx
       **/
      [key: string]: SubmittableExtrinsicFunction<ApiType>;
    };
    treasury: {
      /**
       * It transfers balances from treasury to each of beneficiaries and the specific amount
       * for each of them.
       * 
       * # Error
       * * `BadOrigin`: Only root can execute transaction.
       * * `InsufficientBalance`: If treasury balances is not enough to cover all beneficiaries.
       * * `InvalidIdentity`: If one of the beneficiaries has an invalid identity.
       **/
      disbursement: AugmentedSubmittable<(beneficiaries: Vec<PolymeshPrimitivesBeneficiary> | (PolymeshPrimitivesBeneficiary | { id?: any; amount?: any } | string | Uint8Array)[]) => SubmittableExtrinsic<ApiType>, [Vec<PolymeshPrimitivesBeneficiary>]>;
      /**
       * It transfers the specific `amount` from `origin` account into treasury.
       * 
       * Only accounts which are associated to an identity can make a donation to treasury.
       **/
      reimbursement: AugmentedSubmittable<(amount: u128 | AnyNumber | Uint8Array) => SubmittableExtrinsic<ApiType>, [u128]>;
      /**
       * Generic tx
       **/
      [key: string]: SubmittableExtrinsicFunction<ApiType>;
    };
    upgradeCommittee: {
      /**
       * Changes the time after which a proposal expires.
       * 
       * # Arguments
       * * `expiry` - The new expiry time.
       **/
      setExpiresAfter: AugmentedSubmittable<(expiry: PolymeshCommonUtilitiesMaybeBlock | { Some: any } | { None: any } | string | Uint8Array) => SubmittableExtrinsic<ApiType>, [PolymeshCommonUtilitiesMaybeBlock]>;
      /**
       * Changes the release coordinator.
       * 
       * # Arguments
       * * `id` - The DID of the new release coordinator.
       * 
       * # Errors
       * * `NotAMember`, If the new coordinator `id` is not part of the committee.
       **/
      setReleaseCoordinator: AugmentedSubmittable<(id: PolymeshPrimitivesIdentityId | string | Uint8Array) => SubmittableExtrinsic<ApiType>, [PolymeshPrimitivesIdentityId]>;
      /**
       * Change the vote threshold the determines the winning proposal.
       * For e.g., for a simple majority use (1, 2) which represents the in-equation ">= 1/2".
       * 
       * # Arguments
       * * `n` - Numerator of the fraction representing vote threshold.
       * * `d` - Denominator of the fraction representing vote threshold.
       **/
      setVoteThreshold: AugmentedSubmittable<(n: u32 | AnyNumber | Uint8Array, d: u32 | AnyNumber | Uint8Array) => SubmittableExtrinsic<ApiType>, [u32, u32]>;
      /**
       * Votes `approve`ingly (or not, if `false`)
       * on an existing `proposal` given by its hash, `index`.
       * 
       * # Arguments
       * * `proposal` - A hash of the proposal to be voted on.
       * * `index` - The proposal index.
       * * `approve` - If `true` than this is a `for` vote, and `against` otherwise.
       * 
       * # Errors
       * * `NotAMember`, if the `origin` is not a member of this committee.
       **/
      vote: AugmentedSubmittable<(proposal: H256 | string | Uint8Array, index: u32 | AnyNumber | Uint8Array, approve: bool | boolean | Uint8Array) => SubmittableExtrinsic<ApiType>, [H256, u32, bool]>;
      /**
       * Proposes to the committee that `call` should be executed in its name.
       * Alternatively, if the hash of `call` has already been recorded, i.e., already proposed,
       * then this call counts as a vote, i.e., as if `vote_by_hash` was called.
       * 
       * # Weight
       * 
       * The weight of this dispatchable is that of `call` as well as the complexity
       * for recording the vote itself.
       * 
       * # Arguments
       * * `approve` - is this an approving vote?
       * If the proposal doesn't exist, passing `false` will result in error `FirstVoteReject`.
       * * `call` - the call to propose for execution.
       * 
       * # Errors
       * * `FirstVoteReject`, if `call` hasn't been proposed and `approve == false`.
       * * `NotAMember`, if the `origin` is not a member of this committee.
       **/
      voteOrPropose: AugmentedSubmittable<(approve: bool | boolean | Uint8Array, call: Call | { callIndex?: any; args?: any } | string | Uint8Array) => SubmittableExtrinsic<ApiType>, [bool, Call]>;
      /**
       * Generic tx
       **/
      [key: string]: SubmittableExtrinsicFunction<ApiType>;
    };
    upgradeCommitteeMembership: {
      /**
       * Allows the calling member to *unilaterally quit* without this being subject to a GC
       * vote.
       * 
       * # Arguments
       * * `origin` - Member of committee who wants to quit.
       * 
       * # Error
       * 
       * * Only primary key can abdicate.
       * * Last member of a group cannot abdicate.
       **/
      abdicateMembership: AugmentedSubmittable<() => SubmittableExtrinsic<ApiType>, []>;
      /**
       * Adds a member `who` to the group. May only be called from `AddOrigin` or root.
       * 
       * # Arguments
       * * `origin` - Origin representing `AddOrigin` or root
       * * `who` - IdentityId to be added to the group.
       **/
      addMember: AugmentedSubmittable<(who: PolymeshPrimitivesIdentityId | string | Uint8Array) => SubmittableExtrinsic<ApiType>, [PolymeshPrimitivesIdentityId]>;
      /**
       * Disables a member at specific moment.
       * 
       * Please note that if member is already revoked (a "valid member"), its revocation
       * time-stamp will be updated.
       * 
       * Any disabled member should NOT allow to act like an active member of the group. For
       * instance, a disabled CDD member should NOT be able to generate a CDD claim. However any
       * generated claim issued before `at` would be considered as a valid one.
       * 
       * If you want to invalidate any generated claim, you should use `Self::remove_member`.
       * 
       * # Arguments
       * * `at` - Revocation time-stamp.
       * * `who` - Target member of the group.
       * * `expiry` - Time-stamp when `who` is removed from CDD. As soon as it is expired, the
       * generated claims will be "invalid" as `who` is not considered a member of the group.
       **/
      disableMember: AugmentedSubmittable<(who: PolymeshPrimitivesIdentityId | string | Uint8Array, expiry: Option<u64> | null | object | string | Uint8Array, at: Option<u64> | null | object | string | Uint8Array) => SubmittableExtrinsic<ApiType>, [PolymeshPrimitivesIdentityId, Option<u64>, Option<u64>]>;
      /**
       * Removes a member `who` from the set. May only be called from `RemoveOrigin` or root.
       * 
       * Any claim previously generated by this member is not valid as a group claim. For
       * instance, if a CDD member group generated a claim for a target identity and then it is
       * removed, that claim will be invalid.  In case you want to keep the validity of generated
       * claims, you have to use `Self::disable_member` function
       * 
       * # Arguments
       * * `origin` - Origin representing `RemoveOrigin` or root
       * * `who` - IdentityId to be removed from the group.
       **/
      removeMember: AugmentedSubmittable<(who: PolymeshPrimitivesIdentityId | string | Uint8Array) => SubmittableExtrinsic<ApiType>, [PolymeshPrimitivesIdentityId]>;
      /**
       * Changes the membership to a new set, disregarding the existing membership.
       * May only be called from `ResetOrigin` or root.
       * 
       * # Arguments
       * * `origin` - Origin representing `ResetOrigin` or root
       * * `members` - New set of identities
       **/
      resetMembers: AugmentedSubmittable<(members: Vec<PolymeshPrimitivesIdentityId> | (PolymeshPrimitivesIdentityId | string | Uint8Array)[]) => SubmittableExtrinsic<ApiType>, [Vec<PolymeshPrimitivesIdentityId>]>;
      /**
       * Change this group's limit for how many concurrent active members they may be.
       * 
       * # Arguments
       * * `limit` - the number of active members there may be concurrently.
       **/
      setActiveMembersLimit: AugmentedSubmittable<(limit: u32 | AnyNumber | Uint8Array) => SubmittableExtrinsic<ApiType>, [u32]>;
      /**
       * Swaps out one member `remove` for another member `add`.
       * 
       * May only be called from `SwapOrigin` or root.
       * 
       * # Arguments
       * * `origin` - Origin representing `SwapOrigin` or root
       * * `remove` - IdentityId to be removed from the group.
       * * `add` - IdentityId to be added in place of `remove`.
       **/
      swapMember: AugmentedSubmittable<(remove: PolymeshPrimitivesIdentityId | string | Uint8Array, add: PolymeshPrimitivesIdentityId | string | Uint8Array) => SubmittableExtrinsic<ApiType>, [PolymeshPrimitivesIdentityId, PolymeshPrimitivesIdentityId]>;
      /**
       * Generic tx
       **/
      [key: string]: SubmittableExtrinsicFunction<ApiType>;
    };
    utility: {
      /**
       * Dispatch multiple calls from the sender's origin.
       * 
       * This will execute until the first one fails and then stop.
       * 
       * May be called from root or a signed origin.
       * 
       * # Parameters
       * - `calls`: The calls to be dispatched from the same origin.
       * 
       * # Weight
       * - The sum of the weights of the `calls`.
       * - One event.
       * 
       * This will return `Ok` in all circumstances except an unsigned origin. To determine the success of the batch, an
       * event is deposited. If a call failed and the batch was interrupted, then the
       * `BatchInterrupted` event is deposited, along with the number of successful calls made
       * and the error of the failed call. If all were successful, then the `BatchCompleted`
       * event is deposited.
       **/
      batch: AugmentedSubmittable<(calls: Vec<Call> | (Call | { callIndex?: any; args?: any } | string | Uint8Array)[]) => SubmittableExtrinsic<ApiType>, [Vec<Call>]>;
      /**
       * Dispatch multiple calls from the sender's origin.
       * 
       * This will execute all calls, in order, stopping at the first failure,
       * in which case the state changes are rolled back.
       * On failure, an event `BatchInterrupted(failure_idx, error)` is deposited.
       * 
       * May be called from root or a signed origin.
       * 
       * # Parameters
       * - `calls`: The calls to be dispatched from the same origin.
       * 
       * # Weight
       * - The sum of the weights of the `calls`.
       * - One event.
       * 
       * This will return `Ok` in all circumstances except an unsigned origin.
       * To determine the success of the batch, an event is deposited.
       * If any call failed, then `BatchInterrupted` is deposited.
       * If all were successful, then the `BatchCompleted` event is deposited.
       **/
      batchAtomic: AugmentedSubmittable<(calls: Vec<Call> | (Call | { callIndex?: any; args?: any } | string | Uint8Array)[]) => SubmittableExtrinsic<ApiType>, [Vec<Call>]>;
      /**
       * Dispatch multiple calls from the sender's origin.
       * 
       * This will execute all calls, in order, irrespective of failures.
       * Any failures will be available in a `BatchOptimisticFailed` event.
       * 
       * May be called from root or a signed origin.
       * 
       * # Parameters
       * - `calls`: The calls to be dispatched from the same origin.
       * 
       * 
       * # Weight
       * - The sum of the weights of the `calls`.
       * - One event.
       * 
       * This will return `Ok` in all circumstances except an unsigned origin.
       * To determine the success of the batch, an event is deposited.
       * If any call failed, then `BatchOptimisticFailed` is deposited,
       * with a vector of event counts for each call as well as a vector
       * of errors.
       * If all were successful, then the `BatchCompleted` event is deposited.
       **/
      batchOptimistic: AugmentedSubmittable<(calls: Vec<Call> | (Call | { callIndex?: any; args?: any } | string | Uint8Array)[]) => SubmittableExtrinsic<ApiType>, [Vec<Call>]>;
      /**
       * Relay a call for a target from an origin
       * 
       * Relaying in this context refers to the ability of origin to make a call on behalf of
       * target.
       * 
       * Fees are charged to origin
       * 
       * # Parameters
       * - `target`: Account to be relayed
       * - `signature`: Signature from target authorizing the relay
       * - `call`: Call to be relayed on behalf of target
       * 
       **/
      relayTx: AugmentedSubmittable<(target: AccountId32 | string | Uint8Array, signature: SpRuntimeMultiSignature | { Ed25519: any } | { Sr25519: any } | { Ecdsa: any } | string | Uint8Array, call: PalletUtilityUniqueCall | { nonce?: any; call?: any } | string | Uint8Array) => SubmittableExtrinsic<ApiType>, [AccountId32, SpRuntimeMultiSignature, PalletUtilityUniqueCall]>;
      /**
       * Generic tx
       **/
      [key: string]: SubmittableExtrinsicFunction<ApiType>;
    };
  } // AugmentedSubmittables
} // declare module
