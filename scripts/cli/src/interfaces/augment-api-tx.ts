// Auto-generated via `yarn polkadot-types-from-chain`, do not edit
/* eslint-disable */

// import type lookup before we augment - in some environments
// this is required to allow for ambient/previous definitions
import '@polkadot/api-base/types/submittable';

import type { ApiTypes, AugmentedSubmittable, SubmittableExtrinsic, SubmittableExtrinsicFunction } from '@polkadot/api-base/types';
import type { BTreeSet, Bytes, Compact, Option, U8aFixed, Vec, bool, u128, u16, u32, u64, u8 } from '@polkadot/types-codec';
import type { AnyNumber, IMethod, ITuple } from '@polkadot/types-codec/types';
import type { AccountId32, Call, H256, MultiAddress, Perbill, Percent, Permill } from '@polkadot/types/interfaces/runtime';
import type { PalletBalancesAdjustmentDirection, PalletContractsWasmDeterminism, PalletCorporateActionsBallotBallotMeta, PalletCorporateActionsBallotBallotTimeRange, PalletCorporateActionsBallotBallotVote, PalletCorporateActionsCaId, PalletCorporateActionsCaKind, PalletCorporateActionsInitiateCorporateActionArgs, PalletCorporateActionsRecordDateSpec, PalletCorporateActionsTargetIdentities, PalletElectionProviderMultiPhaseRawSolution, PalletElectionProviderMultiPhaseSolutionOrSnapshotSize, PalletImOnlineHeartbeat, PalletImOnlineSr25519AppSr25519Signature, PalletPipsSnapshotResult, PalletStakingPalletConfigOpPerbill, PalletStakingPalletConfigOpPercent, PalletStakingPalletConfigOpU128, PalletStakingPalletConfigOpU32, PalletStakingRewardDestination, PalletStakingUnlockChunk, PalletStakingValidatorPrefs, PalletStoFundingMethod, PalletStoPriceTier, PalletValidatorsSlashingSwitch, PolymeshContractsApi, PolymeshContractsChainExtensionExtrinsicId, PolymeshContractsNextUpgrade, PolymeshPrimitivesAgentAgentGroup, PolymeshPrimitivesAssetAssetHolder, PolymeshPrimitivesAssetAssetHolderKind, PolymeshPrimitivesAssetAssetId, PolymeshPrimitivesAssetAssetType, PolymeshPrimitivesAssetIdentifier, PolymeshPrimitivesAssetMetadataAssetMetadataKey, PolymeshPrimitivesAssetMetadataAssetMetadataSpec, PolymeshPrimitivesAssetMetadataAssetMetadataValueDetail, PolymeshPrimitivesAssetNonFungibleType, PolymeshPrimitivesAuthorizationAuthorizationData, PolymeshPrimitivesBeneficiary, PolymeshPrimitivesCheckpointScheduleCheckpoints, PolymeshPrimitivesComplianceManagerComplianceRequirement, PolymeshPrimitivesCondition, PolymeshPrimitivesConditionTrustedIssuer, PolymeshPrimitivesDocument, PolymeshPrimitivesIdentityClaimClaim, PolymeshPrimitivesIdentityClaimClaimType, PolymeshPrimitivesIdentityClaimScope, PolymeshPrimitivesIdentityCreateChildIdentityWithAuth, PolymeshPrimitivesIdentityId, PolymeshPrimitivesIdentityIdPortfolioId, PolymeshPrimitivesIdentitySecondaryKeyWithAuth, PolymeshPrimitivesMaybeBlock, PolymeshPrimitivesMemo, PolymeshPrimitivesNftNfTs, PolymeshPrimitivesNftNftCollectionKeys, PolymeshPrimitivesNftNftMetadataAttribute, PolymeshPrimitivesPortfolioFund, PolymeshPrimitivesPosRatio, PolymeshPrimitivesProtocolFeeProtocolOp, PolymeshPrimitivesSecondaryKey, PolymeshPrimitivesSecondaryKeyExtrinsicPermissions, PolymeshPrimitivesSecondaryKeyPermissions, PolymeshPrimitivesSecondaryKeySignatory, PolymeshPrimitivesSettlementAffirmationCount, PolymeshPrimitivesSettlementAssetCount, PolymeshPrimitivesSettlementLeg, PolymeshPrimitivesSettlementReceiptDetails, PolymeshPrimitivesSettlementSettlementType, PolymeshPrimitivesSettlementVenueType, PolymeshPrimitivesStatisticsStatType, PolymeshPrimitivesStatisticsStatUpdate, PolymeshPrimitivesTicker, PolymeshPrimitivesTransferComplianceTransferCondition, PolymeshPrimitivesTransferComplianceTransferConditionExemptKey, PolymeshRuntimeDevelopRuntimeOriginCaller, PolymeshRuntimeDevelopRuntimeSessionKeys, SpConsensusBabeDigestsNextConfigDescriptor, SpConsensusBeefyDoubleVotingProof, SpConsensusBeefyForkVotingProof, SpConsensusBeefyFutureBlockVotingProof, SpConsensusGrandpaEquivocationProof, SpConsensusSlotsEquivocationProof, SpNposElectionsElectionScore, SpNposElectionsSupport, SpRuntimeMultiSignature, SpSessionMembershipProof, SpWeightsWeightV2Weight } from '@polkadot/types/lookup';

export type __AugmentedSubmittable = AugmentedSubmittable<() => unknown>;
export type __SubmittableExtrinsic<ApiType extends ApiTypes> = SubmittableExtrinsic<ApiType>;
export type __SubmittableExtrinsicFunction<ApiType extends ApiTypes> = SubmittableExtrinsicFunction<ApiType>;

declare module '@polkadot/api-base/types/submittable' {
  interface AugmentedSubmittables<ApiType extends ApiTypes> {
    asset: {
      /**
       * Accepts an asset ownership transfer.
       * 
       * Consumes the authorization `auth_id` (see `pallet_identity::consume_auth`).
       * NB: To reject the transfer, call remove auth function in identity module.
       * 
       * # Arguments
       * * `origin` - The origin of the call, which can be the primary or secondary key of an identity.
       * * `auth_id` - Authorization ID of the asset ownership transfer authorization.
       * 
       * # Events
       * * `AssetOwnershipTransferred` - When a asset ownership is successfully transferred.
       * 
       * # Errors
       * * `TickerRegistrationNotFound` - If the ticker registration is not found.
       * * `TickerIsAlreadyLinkedToAnAsset` - If the ticker is already linked to an asset.
       **/
      acceptAssetOwnershipTransfer: AugmentedSubmittable<(authId: u64 | AnyNumber | Uint8Array) => SubmittableExtrinsic<ApiType>, [u64]>;
      /**
       * Accepts a ticker transfer.
       * 
       * Consumes the authorization `auth_id` (see `pallet_identity::consume_auth`).
       * NB: To reject the transfer, call remove auth function in identity module.
       * 
       * # Arguments
       * * `origin` - The origin of the call, which can be the primary or secondary key of an identity.
       * * `auth_id` - Authorization ID of ticker transfer authorization.
       * 
       * # Events
       * * `TickerTransferred` - When a ticker is successfully transferred.
       * 
       * # Errors
       * * `TickerRegistrationNotFound` - If the ticker registration is not found.
       * * `TickerIsAlreadyLinkedToAnAsset` - If the ticker is already linked to an asset.
       **/
      acceptTickerTransfer: AugmentedSubmittable<(authId: u64 | AnyNumber | Uint8Array) => SubmittableExtrinsic<ApiType>, [u64]>;
      /**
       * Add documents for a given asset.
       * 
       * This function allows the asset issuer or an external agent to add documents to an asset.
       * 
       * # Arguments
       * * `origin` - The origin of the call, which can be the primary or secondary key of an identity.
       * * `asset_id` - The [`AssetId`] associated to the asset.
       * * `docs` - Documents to be attached to the asset.
       * 
       * # Permissions
       * * Asset
       * 
       * # Events
       * * `DocumentAdded` - When a document is successfully added to an asset.
       * 
       * # Errors
       * * `CounterOverflow` - If the document ID counter overflows.
       **/
      addDocuments: AugmentedSubmittable<(docs: Vec<PolymeshPrimitivesDocument> | (PolymeshPrimitivesDocument | { uri?: any; contentHash?: any; name?: any; docType?: any; filingDate?: any } | string | Uint8Array)[], assetId: PolymeshPrimitivesAssetAssetId | string | Uint8Array) => SubmittableExtrinsic<ApiType>, [Vec<PolymeshPrimitivesDocument>, PolymeshPrimitivesAssetAssetId]>;
      /**
       * Sets all identities in the `mediators` set as mandatory mediators for any instruction transferring `asset_id`.
       * 
       * This function allows the asset issuer or an external agent to add mandatory mediators for an asset.
       * 
       * # Arguments
       * * `origin` - The origin of the call, which can be the primary or secondary key of an identity.
       * * `asset_id` - The [`AssetId`] of the asset that will require the mediators.
       * * `mediators` - A set of [`IdentityId`] of all the mandatory mediators for the given ticker.
       * 
       * # Permissions
       * * Asset
       * 
       * # Events
       * * `AssetMediatorsAdded` - When the mandatory mediators are successfully added.
       * 
       * # Errors
       * * `NumberOfAssetMediatorsExceeded` - If the number of mandatory mediators exceeds the maximum allowed limit.
       **/
      addMandatoryMediators: AugmentedSubmittable<(assetId: PolymeshPrimitivesAssetAssetId | string | Uint8Array, mediators: BTreeSet<PolymeshPrimitivesIdentityId>) => SubmittableExtrinsic<ApiType>, [PolymeshPrimitivesAssetAssetId, BTreeSet<PolymeshPrimitivesIdentityId>]>;
      /**
       * Forces a transfer of tokens from `asset_holder` to the caller's default portfolio.
       * 
       * This function allows the asset issuer or an external agent to force a transfer of tokens from one portfolio to another.
       * 
       * # Arguments
       * * `origin` - The origin of the call, which can be the primary or secondary key of an identity.
       * * `asset_id` - The [`AssetId`] associated to the asset.
       * * `value` - The [`Balance`] of tokens that will be transferred.
       * * `asset_holder` - The [`AssetHolder`] that will have its balance reduced.
       * 
       * # Permissions
       * * Asset
       * * Portfolio
       * 
       * # Events
       * * `ControllerTransfer` - When tokens are successfully transferred.
       * 
       * # Errors
       * * `UnexpectedNonFungibleToken` - If the asset is a non-fungible token.
       * * `InvalidGranularity` - If the amount to transfer does not meet the granularity requirements.
       * * `TotalSupplyOverflow` - If the total supply exceeds the maximum allowed limit.
       **/
      controllerTransfer: AugmentedSubmittable<(assetId: PolymeshPrimitivesAssetAssetId | string | Uint8Array, value: u128 | AnyNumber | Uint8Array, assetHolder: PolymeshPrimitivesAssetAssetHolder | { Portfolio: any } | { Account: any } | string | Uint8Array) => SubmittableExtrinsic<ApiType>, [PolymeshPrimitivesAssetAssetId, u128, PolymeshPrimitivesAssetAssetHolder]>;
      /**
       * Creates a new asset.
       * 
       * The total supply will initially be zero. To mint tokens, use [`Pallet::issue`].
       * 
       * # Arguments
       * * `origin` - The origin of the call, which can be the primary or secondary key of an identity.
       * * `asset_name` - The [`AssetName`] of the new asset.
       * * `divisible` - Sets [`AssetDetails::divisible`], where `true` means the asset is divisible.
       * * `asset_type` - The [`AssetType`] of the new asset.
       * * `asset_identifiers` - A vector of [`AssetIdentifier`].
       * * `funding_round_name` - The name of the funding round ([`FundingRoundName`]).
       * 
       * # Events
       * * `AssetCreated` - When a new asset is successfully created.
       * 
       * # Errors
       * * `MaxLengthOfAssetNameExceeded` - If the asset name length exceeds the maximum allowed length.
       * * `InvalidCustomAssetTypeId` - If the custom asset type ID is invalid.
       * * `InvalidAssetIdentifier` - If any of the asset identifiers are invalid.
       **/
      createAsset: AugmentedSubmittable<(assetName: Bytes | string | Uint8Array, divisible: bool | boolean | Uint8Array, assetType: PolymeshPrimitivesAssetAssetType | { EquityCommon: any } | { EquityPreferred: any } | { Commodity: any } | { FixedIncome: any } | { REIT: any } | { Fund: any } | { RevenueShareAgreement: any } | { StructuredProduct: any } | { Derivative: any } | { Custom: any } | { StableCoin: any } | { NonFungible: any } | string | Uint8Array, assetIdentifiers: Vec<PolymeshPrimitivesAssetIdentifier> | (PolymeshPrimitivesAssetIdentifier | { CUSIP: any } | { CINS: any } | { ISIN: any } | { LEI: any } | { FIGI: any } | string | Uint8Array)[], fundingRoundName: Option<Bytes> | null | Uint8Array | Bytes | string) => SubmittableExtrinsic<ApiType>, [Bytes, bool, PolymeshPrimitivesAssetAssetType, Vec<PolymeshPrimitivesAssetIdentifier>, Option<Bytes>]>;
      /**
       * Creates a new asset with a new custom asset type.
       * 
       * The total supply will initially be zero. To mint tokens, use [`Pallet::issue`].
       * 
       * # Arguments
       * * `origin` - The origin of the call, which can be the primary or secondary key of an identity.
       * * `asset_name` - The [`AssetName`] of the new asset.
       * * `divisible` - Sets [`AssetDetails::divisible`], where `true` means the asset is divisible.
       * * `custom_asset_type` - The custom asset type of the asset.
       * * `asset_identifiers` - A vector of [`AssetIdentifier`].
       * * `funding_round_name` - The name of the funding round ([`FundingRoundName`]).
       * 
       * # Events
       * * `AssetCreated` - When a new asset is successfully created.
       * * `CustomAssetTypeRegistered` - When a new custom asset type is successfully registered.
       * * `CustomAssetTypeAlreadyRegistered` - When the custom asset type is already registered.
       * 
       * # Errors
       * * `MaxLengthOfAssetNameExceeded` - If the asset name length exceeds the maximum allowed length.
       * * `InvalidCustomAssetTypeId` - If the custom asset type ID is invalid.
       * * `InvalidAssetIdentifier` - If any of the asset identifiers are invalid.
       * * `TooLong` - If the custom asset type length exceeds the maximum allowed length.
       **/
      createAssetWithCustomType: AugmentedSubmittable<(assetName: Bytes | string | Uint8Array, divisible: bool | boolean | Uint8Array, customAssetType: Bytes | string | Uint8Array, assetIdentifiers: Vec<PolymeshPrimitivesAssetIdentifier> | (PolymeshPrimitivesAssetIdentifier | { CUSIP: any } | { CINS: any } | { ISIN: any } | { LEI: any } | { FIGI: any } | string | Uint8Array)[], fundingRoundName: Option<Bytes> | null | Uint8Array | Bytes | string) => SubmittableExtrinsic<ApiType>, [Bytes, bool, Bytes, Vec<PolymeshPrimitivesAssetIdentifier>, Option<Bytes>]>;
      /**
       * Pre-approves the receivement of the asset for all identities.
       * 
       * This function allows the root origin to pre-approve the receivement of an asset for all identities.
       * 
       * # Arguments
       * * `origin` - The root origin.
       * * `asset_id` - The [`AssetId`] that will be exempt from affirmation.
       * 
       * # Events
       * * `AssetAffirmationExemption` - When the asset is successfully exempted from affirmation.
       **/
      exemptAssetAffirmation: AugmentedSubmittable<(assetId: PolymeshPrimitivesAssetAssetId | string | Uint8Array) => SubmittableExtrinsic<ApiType>, [PolymeshPrimitivesAssetAssetId]>;
      /**
       * Freezes transfers of a given asset.
       * 
       * This function allows the asset issuer or an external agent to freeze transfers of a given asset.
       * 
       * # Arguments
       * * `origin` - The origin of the call, which can be the primary or secondary key of an identity.
       * * `asset_id` - The asset to freeze.
       * 
       * # Permissions
       * * Asset
       * 
       * # Events
       * * `AssetFrozen` - When an asset is successfully frozen.
       * 
       * # Errors
       * * `NoSuchAsset` - If the asset does not exist.
       * * `AlreadyFrozen` - If the asset is already frozen.
       **/
      freeze: AugmentedSubmittable<(assetId: PolymeshPrimitivesAssetAssetId | string | Uint8Array) => SubmittableExtrinsic<ApiType>, [PolymeshPrimitivesAssetAssetId]>;
      /**
       * Issue (i.e mint) new tokens to the caller, which must be an authorized external agent.
       * 
       * This function allows the asset issuer or an external agent to mint new tokens for a given asset.
       * 
       * # Arguments
       * * `origin`: A signer that has permissions to act as an agent of `ticker`.
       * * `asset_id`: the [`AssetId`] associated to the asset.
       * * `amount`: The amount of tokens that will be issued.
       * * `asset_holder_kind`: The [`AssetHolderKind`] of the portfolio that will receive the minted tokens.
       * 
       * # Permissions
       * * Asset
       * * Portfolio
       * 
       * # Events
       * * `AssetBalanceUpdated` - When the asset balance is successfully updated.
       * 
       * # Errors
       * * `UnexpectedNonFungibleToken` - If the asset is a non-fungible token.
       * * `InvalidGranularity` - If the amount to issue does not meet the granularity requirements.
       * * `TotalSupplyOverflow` - If the total supply exceeds the maximum allowed limit.
       **/
      issue: AugmentedSubmittable<(assetId: PolymeshPrimitivesAssetAssetId | string | Uint8Array, amount: u128 | AnyNumber | Uint8Array, assetHolderKind: PolymeshPrimitivesAssetAssetHolderKind | { Account: any } | { DefaultPortfolio: any } | { UserPortfolio: any } | string | Uint8Array) => SubmittableExtrinsic<ApiType>, [PolymeshPrimitivesAssetAssetId, u128, PolymeshPrimitivesAssetAssetHolderKind]>;
      /**
       * Establishes a connection between a ticker and an AssetId.
       * 
       * This function allows the asset issuer or an external agent to link a ticker to an asset.
       * 
       * # Arguments
       * * `origin` - The origin of the call, which can be the primary or secondary key of an identity.
       * * `ticker` - The [`Ticker`] that will be linked to the given `asset_id`.
       * * `asset_id` - The [`AssetId`] that will be connected to `ticker`.
       * 
       * # Permissions
       * * Asset
       * 
       * # Events
       * * `TickerLinkedToAsset` - When the ticker is successfully linked to the asset.
       * 
       * # Errors
       * * `TickerNotRegisteredToCaller` - If the ticker is not registered to the caller.
       * * `TickerRegistrationExpired` - If the ticker registration has expired.
       * * `TickerRegistrationNotFound` - If the ticker registration is not found.
       * * `TickerIsAlreadyLinkedToAnAsset` - If the ticker is already linked to an asset.
       * * `AssetIsAlreadyLinkedToATicker` - If the asset is already linked to a ticker.
       **/
      linkTickerToAssetId: AugmentedSubmittable<(ticker: PolymeshPrimitivesTicker | string | Uint8Array, assetId: PolymeshPrimitivesAssetAssetId | string | Uint8Array) => SubmittableExtrinsic<ApiType>, [PolymeshPrimitivesTicker, PolymeshPrimitivesAssetAssetId]>;
      /**
       * If the asset associated to `asset_id` is indivisible, sets [`AssetDetails::divisible`] to true.
       * 
       * This function allows the asset issuer or an external agent to make an indivisible asset divisible.
       * 
       * # Arguments
       * * `origin` - The origin of the call, which can be the primary or secondary key of an identity.
       * * `asset_id` - The [`AssetId`] associated to the asset.
       * 
       * # Permissions
       * * Asset
       * 
       * # Events
       * * `DivisibilityChanged` - When the divisibility of an asset is successfully changed.
       * 
       * # Errors
       * * `NoSuchAsset` - If the asset does not exist.
       * * `UnexpectedNonFungibleToken` - If the asset is a non-fungible token.
       * * `AssetAlreadyDivisible` - If the asset is already divisible.
       **/
      makeDivisible: AugmentedSubmittable<(assetId: PolymeshPrimitivesAssetAssetId | string | Uint8Array) => SubmittableExtrinsic<ApiType>, [PolymeshPrimitivesAssetAssetId]>;
      /**
       * Pre-approves the receivement of an asset.
       * 
       * # Arguments
       * * `origin` - The origin of the call, which can be the primary or secondary key of an identity.
       * * `asset_id` - The [`AssetId`] that will be exempt from affirmation.
       * 
       * # Permissions
       * * Asset
       * 
       * # Events
       * * `PreApprovedAsset` - When the asset is successfully pre-approved for receivement.
       **/
      preApproveAsset: AugmentedSubmittable<(assetId: PolymeshPrimitivesAssetAssetId | string | Uint8Array) => SubmittableExtrinsic<ApiType>, [PolymeshPrimitivesAssetAssetId]>;
      /**
       * Receiver affirms a pending asset transfer.
       * 
       * If someone tries to transfer asset to an account that requires receiver affirmations, then the receiver will need to affirm the transfer
       * before the transfer is executed.
       * 
       * # Arguments
       * * `origin` - The origin of the call, which will be the receiver of the assets.
       * * `transfer_id` - The [`InstructionId`] associated to the pending transfer.
       * 
       * # Permissions
       * * Asset
       * * Portfolio
       * 
       * # Events
       * * `InstructionAffirmed` - The asset transfer settlement was affirmed by the caller as the receiver.
       * * `InstructionExecuted` - The asset transfer settlement was executed successfully.
       * * `AssetBalanceUpdated` - The asset balance was updated for both the sender and receiver portfolios.
       * 
       * # Errors
       * * `UnknownInstruction` - If the instruction associated to the given transfer ID does not exist.
       * * `InvalidTransfer` - If the transfer validation check fails.
       **/
      receiverAffirmAssetTransfer: AugmentedSubmittable<(transferId: u64 | AnyNumber | Uint8Array) => SubmittableExtrinsic<ApiType>, [u64]>;
      /**
       * Redeems (i.e burns) existing tokens by reducing the balance of the caller's portfolio and the total supply of the asset.
       * 
       * This function allows the asset issuer or an external agent to redeem tokens from a given asset.
       * 
       * # Arguments
       * * `origin`: is a signer that has permissions to act as an agent of `asset_id`.
       * * `asset_id`: the [`AssetId`] associated to the asset.
       * * `value`: amount of tokens to redeem.
       * * `asset_holder_kind`: the [`AssetHolderKind`] that will have its balance reduced.
       * 
       * # Permissions
       * * Asset
       * * Portfolio
       * 
       * # Events
       * * `AssetBalanceUpdated` - When the asset balance is successfully updated.
       * 
       * # Errors
       * * `UnexpectedNonFungibleToken` - If the asset is a non-fungible token.
       * * `InvalidGranularity` - If the value to redeem does not meet the granularity requirements.
       * * `TotalSupplyOverflow` - If the total supply exceeds the maximum allowed limit.
       **/
      redeem: AugmentedSubmittable<(assetId: PolymeshPrimitivesAssetAssetId | string | Uint8Array, value: u128 | AnyNumber | Uint8Array, assetHolderKind: PolymeshPrimitivesAssetAssetHolderKind | { Account: any } | { DefaultPortfolio: any } | { UserPortfolio: any } | string | Uint8Array) => SubmittableExtrinsic<ApiType>, [PolymeshPrimitivesAssetAssetId, u128, PolymeshPrimitivesAssetAssetHolderKind]>;
      /**
       * Registers and set local asset metadata.
       * 
       * This function allows the asset issuer or an external agent to register and set local metadata for an asset.
       * 
       * # Arguments
       * * `origin` - The origin of the call, which can be the primary or secondary key of an identity.
       * * `asset_id` - The [`AssetId`] associated to the asset.
       * * `name` - The [`AssetMetadataName`].
       * * `spec` - The asset metadata specifications ([`AssetMetadataSpec`]).
       * * `value` - The [`AssetMetadataValue`] of the given metadata key.
       * * `details` - Optional [`AssetMetadataValueDetail`] (expire, lock status).
       * 
       * # Permissions
       * * Asset
       * 
       * # Events
       * * `RegisterAssetMetadataLocalType` - When the local asset metadata type is successfully registered.
       * * `SetAssetMetadataValue` - When the asset metadata value is successfully set.
       * 
       * # Errors
       * * `AssetMetadataLocalKeyAlreadyExists` - If the local metadata key already exists.
       * * `AssetMetadataValueIsLocked` - If the metadata value is locked.
       * * `AssetMetadataValueMaxLengthExceeded` - If the metadata value length exceeds the maximum allowed length.
       **/
      registerAndSetLocalAssetMetadata: AugmentedSubmittable<(assetId: PolymeshPrimitivesAssetAssetId | string | Uint8Array, name: Bytes | string | Uint8Array, spec: PolymeshPrimitivesAssetMetadataAssetMetadataSpec | { url?: any; description?: any; typeDef?: any } | string | Uint8Array, value: Bytes | string | Uint8Array, detail: Option<PolymeshPrimitivesAssetMetadataAssetMetadataValueDetail> | null | Uint8Array | PolymeshPrimitivesAssetMetadataAssetMetadataValueDetail | { expire?: any; lockStatus?: any } | string) => SubmittableExtrinsic<ApiType>, [PolymeshPrimitivesAssetAssetId, Bytes, PolymeshPrimitivesAssetMetadataAssetMetadataSpec, Bytes, Option<PolymeshPrimitivesAssetMetadataAssetMetadataValueDetail>]>;
      /**
       * Registers asset metadata global type.
       * 
       * This function allows the root origin to register a global metadata type.
       * 
       * # Arguments
       * * `origin` - The root origin.
       * * `name` - The [`AssetMetadataName`].
       * * `spec` - The asset metadata specifications ([`AssetMetadataSpec`]).
       * 
       * # Events
       * * `RegisterAssetMetadataGlobalType` - When the global asset metadata type is successfully registered.
       * 
       * # Errors
       * * `AssetMetadataGlobalKeyAlreadyExists` - If the global metadata key already exists.
       * * `AssetMetadataNameMaxLengthExceeded` - If the metadata name length exceeds the maximum allowed length.
       * * `AssetMetadataTypeDefMaxLengthExceeded` - If the metadata type definition length exceeds the maximum allowed length.
       **/
      registerAssetMetadataGlobalType: AugmentedSubmittable<(name: Bytes | string | Uint8Array, spec: PolymeshPrimitivesAssetMetadataAssetMetadataSpec | { url?: any; description?: any; typeDef?: any } | string | Uint8Array) => SubmittableExtrinsic<ApiType>, [Bytes, PolymeshPrimitivesAssetMetadataAssetMetadataSpec]>;
      /**
       * Registers asset metadata local type.
       * 
       * This function allows the asset issuer or an external agent to register a local metadata type for an asset.
       * 
       * # Arguments
       * * `origin` - The origin of the call, which can be the primary or secondary key of an identity.
       * * `asset_id` - The [`AssetId`] associated to the asset.
       * * `name` - The [`AssetMetadataName`].
       * * `spec` - The asset metadata specifications ([`AssetMetadataSpec`]).
       * 
       * # Permissions
       * * Asset
       * 
       * # Events
       * * `RegisterAssetMetadataLocalType` - When the local asset metadata type is successfully registered.
       * 
       * # Errors
       * * `AssetMetadataLocalKeyAlreadyExists` - If the local metadata key already exists.
       * * `AssetMetadataNameMaxLengthExceeded` - If the metadata name length exceeds the maximum allowed length.
       * * `AssetMetadataTypeDefMaxLengthExceeded` - If the metadata type definition length exceeds the maximum allowed length.
       **/
      registerAssetMetadataLocalType: AugmentedSubmittable<(assetId: PolymeshPrimitivesAssetAssetId | string | Uint8Array, name: Bytes | string | Uint8Array, spec: PolymeshPrimitivesAssetMetadataAssetMetadataSpec | { url?: any; description?: any; typeDef?: any } | string | Uint8Array) => SubmittableExtrinsic<ApiType>, [PolymeshPrimitivesAssetAssetId, Bytes, PolymeshPrimitivesAssetMetadataAssetMetadataSpec]>;
      /**
       * Registers a custom asset type.
       * 
       * # Arguments
       * * `origin` - The origin of the call, which can be the primary or secondary key of an identity.
       * * `ty` - Contains the string representation of the asset type.
       * 
       * # Events
       * * `CustomAssetTypeRegistered` - When a new custom asset type is successfully registered.
       * * `CustomAssetTypeAlreadyRegistered` - When the custom asset type is already registered.
       * 
       * # Errors
       * * `TooLong` - If the custom asset type length exceeds the maximum allowed length.
       **/
      registerCustomAssetType: AugmentedSubmittable<(ty: Bytes | string | Uint8Array) => SubmittableExtrinsic<ApiType>, [Bytes]>;
      /**
       * Registers a unique ticker or extends the validity of an existing ticker.
       * 
       * This function allows the caller to register a new ticker or extend the registration
       * of an existing ticker. The ticker validity does not carry forward when renewing.
       * 
       * # Arguments
       * * `origin` - The origin of the call, which can be the primary or secondary key of an identity.
       * * `ticker` - The ticker to register.
       * 
       * # Events
       * * `TickerRegistered` - When a ticker is successfully registered.
       * 
       * # Errors
       * * `TickerAlreadyRegistered` - If the ticker is already registered.
       * * `TickerTooLong` - If the ticker length exceeds the maximum allowed length.
       * * `InvalidTickerCharacter` - If the ticker contains invalid characters.
       **/
      registerUniqueTicker: AugmentedSubmittable<(ticker: PolymeshPrimitivesTicker | string | Uint8Array) => SubmittableExtrinsic<ApiType>, [PolymeshPrimitivesTicker]>;
      /**
       * Reject a pending asset transfer.
       * 
       * If someone tries to transfer asset to an account that requires receiver affirmations, then the receiver can reject the transfer.
       * The sender can also reject the transfer before the receiver affirms it.
       * 
       * # Arguments
       * * `origin` - The origin of the call, which can be either the sender or receiver.
       * * `transfer_id` - The [`InstructionId`] associated to the pending transfer.
       * 
       * # Permissions
       * * Asset
       * * Portfolio
       * 
       * # Events
       * * `InstructionRejected` - The asset transfer settlement was rejected by the caller.
       * 
       * # Errors
       * * `InvalidInstructionStatusForRejection` - Either the instruction doesn't exist or it has already been executed or rejected.
       **/
      rejectAssetTransfer: AugmentedSubmittable<(transferId: u64 | AnyNumber | Uint8Array) => SubmittableExtrinsic<ApiType>, [u64]>;
      /**
       * Removes the pre-approval of the asset for all identities.
       * 
       * This function allows the root origin to remove the pre-approval of an asset for all identities.
       * 
       * # Arguments
       * * `origin` - The root origin.
       * * `asset_id` - The [`AssetId`] that will have its exemption removed.
       * 
       * # Events
       * * `RemoveAssetAffirmationExemption` - When the asset's affirmation exemption is successfully removed.
       **/
      removeAssetAffirmationExemption: AugmentedSubmittable<(assetId: PolymeshPrimitivesAssetAssetId | string | Uint8Array) => SubmittableExtrinsic<ApiType>, [PolymeshPrimitivesAssetAssetId]>;
      /**
       * Removes the pre-approval of an asset.
       * 
       * # Arguments
       * * `origin` - The origin of the call, which can be the primary or secondary key of an identity.
       * * `asset_id` - The [`AssetId`] that will have its exemption removed.
       * 
       * # Permissions
       * * Asset
       * 
       * # Events
       * * `RemovePreApprovedAsset` - When the asset's pre-approval is successfully removed.
       **/
      removeAssetPreApproval: AugmentedSubmittable<(assetId: PolymeshPrimitivesAssetAssetId | string | Uint8Array) => SubmittableExtrinsic<ApiType>, [PolymeshPrimitivesAssetAssetId]>;
      /**
       * Remove documents for a given asset.
       * 
       * This function allows the asset issuer or an external agent to remove documents from an asset.
       * 
       * # Arguments
       * * `origin` - The origin of the call, which can be the primary or secondary key of an identity.
       * * `asset_id` - The [`AssetId`] associated to the asset.
       * * `docs_id` - A vector of all [`DocumentId`] that will be removed from the asset.
       * 
       * # Permissions
       * * Asset
       * 
       * # Events
       * * `DocumentRemoved` - When a document is successfully removed from an asset.
       **/
      removeDocuments: AugmentedSubmittable<(docsId: Vec<u32> | (u32 | AnyNumber | Uint8Array)[], assetId: PolymeshPrimitivesAssetAssetId | string | Uint8Array) => SubmittableExtrinsic<ApiType>, [Vec<u32>, PolymeshPrimitivesAssetAssetId]>;
      /**
       * Removes the asset metadata key and value of a local key.
       * 
       * This function allows the asset issuer or an external agent to remove a local metadata key and its value for an asset.
       * 
       * # Arguments
       * * `origin` - The origin of the call, which can be the primary or secondary key of an identity.
       * * `asset_id` - The [`AssetId`] associated to the local metadata key.
       * * `local_key` - The [`AssetMetadataLocalKey`] that will be removed.
       * 
       * # Permissions
       * * Asset
       * 
       * # Events
       * * `LocalMetadataKeyDeleted` - When the local metadata key is successfully deleted.
       * 
       * # Errors
       * * `AssetMetadataKeyIsMissing` - If the local metadata key is missing.
       * * `AssetMetadataValueIsLocked` - If the metadata value is locked.
       * * `AssetMetadataKeyBelongsToNFTCollection` - If the metadata key belongs to an NFT collection.
       **/
      removeLocalMetadataKey: AugmentedSubmittable<(assetId: PolymeshPrimitivesAssetAssetId | string | Uint8Array, localKey: u64 | AnyNumber | Uint8Array) => SubmittableExtrinsic<ApiType>, [PolymeshPrimitivesAssetAssetId, u64]>;
      /**
       * Removes all identities in the `mediators` set from the mandatory mediators list for the given `asset_id`.
       * 
       * This function allows the asset issuer or an external agent to remove mandatory mediators for an asset.
       * 
       * # Arguments
       * * `origin` - The origin of the call, which can be the primary or secondary key of an identity.
       * * `asset_id` - The [`AssetId`] of the asset that will have mediators removed.
       * * `mediators` - A set of [`IdentityId`] of all the mediators that will be removed from the mandatory mediators list.
       * 
       * # Permissions
       * * Asset
       * 
       * # Events
       * * `AssetMediatorsRemoved` - When the mandatory mediators are successfully removed.
       **/
      removeMandatoryMediators: AugmentedSubmittable<(assetId: PolymeshPrimitivesAssetAssetId | string | Uint8Array, mediators: BTreeSet<PolymeshPrimitivesIdentityId>) => SubmittableExtrinsic<ApiType>, [PolymeshPrimitivesAssetAssetId, BTreeSet<PolymeshPrimitivesIdentityId>]>;
      /**
       * Removes the asset metadata value of a metadata key.
       * 
       * This function allows the asset issuer or an external agent to remove a metadata value for an asset.
       * 
       * # Arguments
       * * `origin` - The origin of the call, which can be the primary or secondary key of an identity.
       * * `asset_id` - The [`AssetId`] associated to the metadata key.
       * * `metadata_key` - The [`AssetMetadataKey`] that will have its value deleted.
       * 
       * # Permissions
       * * Asset
       * 
       * # Events
       * * `MetadataValueDeleted` - When the metadata value is successfully deleted.
       * 
       * # Errors
       * * `AssetMetadataKeyIsMissing` - If the metadata key is missing.
       * * `AssetMetadataValueIsLocked` - If the metadata value is locked.
       **/
      removeMetadataValue: AugmentedSubmittable<(assetId: PolymeshPrimitivesAssetAssetId | string | Uint8Array, metadataKey: PolymeshPrimitivesAssetMetadataAssetMetadataKey | { Global: any } | { Local: any } | string | Uint8Array) => SubmittableExtrinsic<ApiType>, [PolymeshPrimitivesAssetAssetId, PolymeshPrimitivesAssetMetadataAssetMetadataKey]>;
      /**
       * Updates the [`AssetName`] associated to an asset.
       * 
       * This function allows the asset issuer or an external agent to update the name of an asset.
       * 
       * # Arguments
       * * `origin` - The origin of the call, which can be the primary or secondary key of an identity.
       * * `asset_id` - The [`AssetId`] associated to the asset.
       * * `asset_name` - The new [`AssetName`] that will be associated to the asset.
       * 
       * # Permissions
       * * Asset
       * 
       * # Events
       * * `AssetRenamed` - When an asset is successfully renamed.
       * 
       * # Errors
       * * `MaxLengthOfAssetNameExceeded` - If the asset name length exceeds the maximum allowed length.
       * * `NoSuchAsset` - If the asset does not exist.
       **/
      renameAsset: AugmentedSubmittable<(assetId: PolymeshPrimitivesAssetAssetId | string | Uint8Array, assetName: Bytes | string | Uint8Array) => SubmittableExtrinsic<ApiType>, [PolymeshPrimitivesAssetAssetId, Bytes]>;
      /**
       * Set asset metadata value.
       * 
       * This function allows the asset issuer or an external agent to set metadata for an asset.
       * 
       * # Arguments
       * * `origin` - The origin of the call, which can be the primary or secondary key of an identity.
       * * `asset_id` - The [`AssetId`] associated to the asset.
       * * `key` - The [`AssetMetadataKey`] associated to the asset.
       * * `value` - The [`AssetMetadataValue`] of the given metadata key.
       * * `details` - Optional [`AssetMetadataValueDetail`] (expire, lock status).
       * 
       * # Permissions
       * * Asset
       * 
       * # Events
       * * `SetAssetMetadataValue` - When the asset metadata value is successfully set.
       * 
       * # Errors
       * * `AssetMetadataKeyIsMissing` - If the metadata key is missing.
       * * `AssetMetadataValueIsLocked` - If the metadata value is locked.
       * * `AssetMetadataValueMaxLengthExceeded` - If the metadata value length exceeds the maximum allowed length.
       **/
      setAssetMetadata: AugmentedSubmittable<(assetId: PolymeshPrimitivesAssetAssetId | string | Uint8Array, key: PolymeshPrimitivesAssetMetadataAssetMetadataKey | { Global: any } | { Local: any } | string | Uint8Array, value: Bytes | string | Uint8Array, detail: Option<PolymeshPrimitivesAssetMetadataAssetMetadataValueDetail> | null | Uint8Array | PolymeshPrimitivesAssetMetadataAssetMetadataValueDetail | { expire?: any; lockStatus?: any } | string) => SubmittableExtrinsic<ApiType>, [PolymeshPrimitivesAssetAssetId, PolymeshPrimitivesAssetMetadataAssetMetadataKey, Bytes, Option<PolymeshPrimitivesAssetMetadataAssetMetadataValueDetail>]>;
      /**
       * Set asset metadata value details (expire, lock status).
       * 
       * This function allows the asset issuer or an external agent to set metadata details for an asset.
       * 
       * # Arguments
       * * `origin` - The origin of the call, which can be the primary or secondary key of an identity.
       * * `asset_id` - The [`AssetId`] associated to the asset.
       * * `key` - The [`AssetMetadataKey`] associated to the asset.
       * * `details` - The [`AssetMetadataValueDetail`] (expire, lock status) that will be associated to the asset.
       * 
       * # Permissions
       * * Asset
       * 
       * # Events
       * * `SetAssetMetadataValueDetails` - When the asset metadata value details are successfully set.
       * 
       * # Errors
       * * `AssetMetadataKeyIsMissing` - If the metadata key is missing.
       * * `AssetMetadataValueIsLocked` - If the metadata value is locked.
       * * `AssetMetadataValueIsEmpty` - If the metadata value is empty.
       **/
      setAssetMetadataDetails: AugmentedSubmittable<(assetId: PolymeshPrimitivesAssetAssetId | string | Uint8Array, key: PolymeshPrimitivesAssetMetadataAssetMetadataKey | { Global: any } | { Local: any } | string | Uint8Array, detail: PolymeshPrimitivesAssetMetadataAssetMetadataValueDetail | { expire?: any; lockStatus?: any } | string | Uint8Array) => SubmittableExtrinsic<ApiType>, [PolymeshPrimitivesAssetAssetId, PolymeshPrimitivesAssetMetadataAssetMetadataKey, PolymeshPrimitivesAssetMetadataAssetMetadataValueDetail]>;
      /**
       * Sets the name of the current funding round.
       * 
       * This function allows the asset issuer or an external agent to set the name of the current funding round for an asset.
       * 
       * # Arguments
       * * `origin` - The origin of the call, which can be the primary or secondary key of an identity.
       * * `asset_id` - The [`AssetId`] associated to the asset.
       * * `funding_round_name` - The [`FundingRoundName`] of the current funding round.
       * 
       * # Permissions
       * * Asset
       * 
       * # Events
       * * `FundingRoundSet` - When the funding round name is successfully set.
       * 
       * # Errors
       * * `FundingRoundNameMaxLengthExceeded` - If the funding round name length exceeds the maximum allowed length.
       **/
      setFundingRound: AugmentedSubmittable<(assetId: PolymeshPrimitivesAssetAssetId | string | Uint8Array, fundingRoundName: Bytes | string | Uint8Array) => SubmittableExtrinsic<ApiType>, [PolymeshPrimitivesAssetAssetId, Bytes]>;
      /**
       * Sets whether the caller's identity requires receiver affirmation for all incoming transfers.
       * 
       * When `require` is `true`, the identity opts in to mandatory receiver affirmation — all incoming
       * transfers must be explicitly affirmed. When `false` (the default), transfers are
       * auto-affirmed for this identity.
       * 
       * # Arguments
       * * `origin` - The origin of the call, which can be the primary or secondary key of an identity.
       * * `require` - Whether to require mandatory receiver affirmation.
       * 
       * # Events
       * * `MandatoryReceiverAffirmationSet` - When the mandatory receiver affirmation flag is updated.
       **/
      setMandatoryReceiverAffirmation: AugmentedSubmittable<(require: bool | boolean | Uint8Array) => SubmittableExtrinsic<ApiType>, [bool]>;
      /**
       * Transfer assets from the caller's default portfolio to the target address's default portfolio.
       * 
       * The settlement engine is used to perform the asset transfer.
       * 
       * # Arguments
       * * `origin` - The origin of the call, which will be the sender of the assets.
       * * `asset_id` - The [`AssetId`] associated to the asset.
       * * `to` - The target address to which the assets will be sent.
       * * `amount` - The [`Balance`] of tokens that will be transferred.
       * * `memo` - An optional [`Memo`] that can be attached to the transfer instruction.
       * 
       * # Permissions
       * * Asset
       * * Portfolio
       * 
       * # Events
       * * `CreatedAssetTransfer` - When an asset transfer instruction is created.
       * * `InstructionCreated` - The asset transfer settlement was created.
       * * `InstructionAffirmed` - The asset transfer settlement was affirmed by the caller as the sender.
       * * `InstructionAutomaticallyAffirmed` - If the receiver pre-approved the asset, the instruction is automatically affirmed for the receiver.
       * * `InstructionExecuted` - The asset transfer settlement was executed successfully (if the receiver pre-approved the asset).
       * 
       * # Errors
       * * `InsufficientBalance` - If the sender's balance is not sufficient to cover the transfer amount.
       * * `InvalidTransfer` - If the transfer validation check fails.
       * * `ReceiverIdentityNotFound` - If the receiver's identity is not found.
       * * `UnexpectedOFFChainAsset` - If the asset could not be found on-chain.
       * * `MissingIdentity` - The caller doesn't have an identity.
       **/
      transferAsset: AugmentedSubmittable<(assetId: PolymeshPrimitivesAssetAssetId | string | Uint8Array, to: AccountId32 | string | Uint8Array, amount: u128 | AnyNumber | Uint8Array, memo: Option<PolymeshPrimitivesMemo> | null | Uint8Array | PolymeshPrimitivesMemo | string) => SubmittableExtrinsic<ApiType>, [PolymeshPrimitivesAssetAssetId, AccountId32, u128, Option<PolymeshPrimitivesMemo>]>;
      /**
       * Unfreezes transfers of a given asset.
       * 
       * This function allows the asset issuer or an external agent to unfreeze transfers of a given asset.
       * 
       * # Arguments
       * * `origin` - The origin of the call, which can be the primary or secondary key of an identity.
       * * `asset_id` - The asset to unfreeze.
       * 
       * # Permissions
       * * Asset
       * 
       * # Events
       * * `AssetUnfrozen` - When an asset is successfully unfrozen.
       * 
       * # Errors
       * * `NoSuchAsset` - If the asset does not exist.
       * * `NotFrozen` - If the asset is not frozen.
       **/
      unfreeze: AugmentedSubmittable<(assetId: PolymeshPrimitivesAssetAssetId | string | Uint8Array) => SubmittableExtrinsic<ApiType>, [PolymeshPrimitivesAssetAssetId]>;
      /**
       * Removes the link between a ticker and an asset.
       * 
       * This function allows the asset issuer or an external agent to unlink a ticker from an asset.
       * 
       * # Arguments
       * * `origin` - The origin of the call, which can be the primary or secondary key of an identity.
       * * `ticker` - The [`Ticker`] that will be unlinked from the given `asset_id`.
       * * `asset_id` - The [`AssetId`] that will be unlinked from `ticker`.
       * 
       * # Permissions
       * * Asset
       * 
       * # Events
       * * `TickerUnlinkedFromAsset` - When the ticker is successfully unlinked from the asset.
       * 
       * # Errors
       * * `TickerNotRegisteredToCaller` - If the ticker is not registered to the caller.
       * * `TickerRegistrationNotFound` - If the ticker registration is not found.
       * * `TickerIsNotLinkedToTheAsset` - If the ticker is not linked to the asset.
       **/
      unlinkTickerFromAssetId: AugmentedSubmittable<(ticker: PolymeshPrimitivesTicker | string | Uint8Array, assetId: PolymeshPrimitivesAssetAssetId | string | Uint8Array) => SubmittableExtrinsic<ApiType>, [PolymeshPrimitivesTicker, PolymeshPrimitivesAssetAssetId]>;
      /**
       * Updates the type of an asset.
       * 
       * This function allows the asset issuer or an external agent to update the type of an asset.
       * 
       * # Arguments
       * * `origin` - The origin of the call, which can be the primary or secondary key of an identity.
       * * `asset_id` - The [`AssetId`] associated to the asset.
       * * `asset_type` - The new [`AssetType`] of the asset.
       * 
       * # Permissions
       * * Asset
       * 
       * # Events
       * * `AssetTypeChanged` - When the asset type is successfully changed.
       * 
       * # Errors
       * * `NoSuchAsset` - If the asset does not exist.
       * * `InvalidCustomAssetTypeId` - If the custom asset type ID is invalid.
       * * `IncompatibleAssetTypeUpdate` - If the new asset type is incompatible with the existing asset type.
       **/
      updateAssetType: AugmentedSubmittable<(assetId: PolymeshPrimitivesAssetAssetId | string | Uint8Array, assetType: PolymeshPrimitivesAssetAssetType | { EquityCommon: any } | { EquityPreferred: any } | { Commodity: any } | { FixedIncome: any } | { REIT: any } | { Fund: any } | { RevenueShareAgreement: any } | { StructuredProduct: any } | { Derivative: any } | { Custom: any } | { StableCoin: any } | { NonFungible: any } | string | Uint8Array) => SubmittableExtrinsic<ApiType>, [PolymeshPrimitivesAssetAssetId, PolymeshPrimitivesAssetAssetType]>;
      /**
       * Updates the global metadata specification.
       * 
       * # Arguments
       * * `origin` - The origin of the call, which can be the primary or secondary key of an identity.
       * * `asset_metadata_name` - The [`AssetMetadataName`] associated with the global metadata.
       * * `asset_metadata_spec` - The new [`AssetMetadataSpec`] that will be associated with the global metadata.
       * 
       * # Events
       * * `GlobalMetadataSpecUpdated` - When the global metadata specification is successfully updated.
       * 
       * # Errors
       * * `BadOrigin` - If the origin is not authorized.
       * * `TooLong` - If the metadata url or description length exceeds the maximum allowed length.
       * * `AssetMetadataTypeDefMaxLengthExceeded` - If the metadata type definition length exceeds the maximum allowed length.
       **/
      updateGlobalMetadataSpec: AugmentedSubmittable<(assetMetadataName: Bytes | string | Uint8Array, assetMetadataSpec: PolymeshPrimitivesAssetMetadataAssetMetadataSpec | { url?: any; description?: any; typeDef?: any } | string | Uint8Array) => SubmittableExtrinsic<ApiType>, [Bytes, PolymeshPrimitivesAssetMetadataAssetMetadataSpec]>;
      /**
       * Updates the asset identifiers associated to the asset.
       * 
       * This function allows the asset issuer or an external agent to update the asset identifiers for an asset.
       * 
       * # Arguments
       * * `origin` - The origin of the call, which can be the primary or secondary key of an identity.
       * * `asset_id` - The [`AssetId`] associated to the asset.
       * * `asset_identifiers` - A vector of [`AssetIdentifier`] that will be associated to the asset.
       * 
       * # Permissions
       * * Asset
       * 
       * # Events
       * * `IdentifiersUpdated` - When the asset identifiers are successfully updated.
       * 
       * # Errors
       * * `InvalidAssetIdentifier` - If any of the asset identifiers are invalid.
       **/
      updateIdentifiers: AugmentedSubmittable<(assetId: PolymeshPrimitivesAssetAssetId | string | Uint8Array, assetIdentifiers: Vec<PolymeshPrimitivesAssetIdentifier> | (PolymeshPrimitivesAssetIdentifier | { CUSIP: any } | { CINS: any } | { ISIN: any } | { LEI: any } | { FIGI: any } | string | Uint8Array)[]) => SubmittableExtrinsic<ApiType>, [PolymeshPrimitivesAssetAssetId, Vec<PolymeshPrimitivesAssetIdentifier>]>;
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
       * Burn the specified liquid free balance from the origin account.
       * 
       * If the origin's account ends up below the existential deposit as a result
       * of the burn and `keep_alive` is false, the account will be reaped.
       * 
       * Unlike sending funds to a _burn_ address, which merely makes the funds inaccessible,
       * this `burn` operation will reduce total issuance by the amount _burned_.
       **/
      burn: AugmentedSubmittable<(value: Compact<u128> | AnyNumber | Uint8Array, keepAlive: bool | boolean | Uint8Array) => SubmittableExtrinsic<ApiType>, [Compact<u128>, bool]>;
      /**
       * Adjust the total issuance in a saturating way.
       * 
       * Can only be called by root and always needs a positive `delta`.
       * 
       * # Example
       **/
      forceAdjustTotalIssuance: AugmentedSubmittable<(direction: PalletBalancesAdjustmentDirection | 'Increase' | 'Decrease' | number | Uint8Array, delta: Compact<u128> | AnyNumber | Uint8Array) => SubmittableExtrinsic<ApiType>, [PalletBalancesAdjustmentDirection, Compact<u128>]>;
      /**
       * Set the regular balance of a given account.
       * 
       * The dispatch origin for this call is `root`.
       **/
      forceSetBalance: AugmentedSubmittable<(who: MultiAddress | { Id: any } | { Index: any } | { Raw: any } | { Address32: any } | { Address20: any } | string | Uint8Array, newFree: Compact<u128> | AnyNumber | Uint8Array) => SubmittableExtrinsic<ApiType>, [MultiAddress, Compact<u128>]>;
      /**
       * Exactly as `transfer_allow_death`, except the origin must be root and the source account
       * may be specified.
       **/
      forceTransfer: AugmentedSubmittable<(source: MultiAddress | { Id: any } | { Index: any } | { Raw: any } | { Address32: any } | { Address20: any } | string | Uint8Array, dest: MultiAddress | { Id: any } | { Index: any } | { Raw: any } | { Address32: any } | { Address20: any } | string | Uint8Array, value: Compact<u128> | AnyNumber | Uint8Array) => SubmittableExtrinsic<ApiType>, [MultiAddress, MultiAddress, Compact<u128>]>;
      /**
       * Unreserve some balance from a user by force.
       * 
       * Can only be called by ROOT.
       **/
      forceUnreserve: AugmentedSubmittable<(who: MultiAddress | { Id: any } | { Index: any } | { Raw: any } | { Address32: any } | { Address20: any } | string | Uint8Array, amount: u128 | AnyNumber | Uint8Array) => SubmittableExtrinsic<ApiType>, [MultiAddress, u128]>;
      /**
       * Transfer the entire transferable balance from the caller account.
       * 
       * NOTE: This function only attempts to transfer _transferable_ balances. This means that
       * any locked, reserved, or existential deposits (when `keep_alive` is `true`), will not be
       * transferred by this function. To ensure that this function results in a killed account,
       * you might need to prepare the account by removing any reference counters, storage
       * deposits, etc...
       * 
       * The dispatch origin of this call must be Signed.
       * 
       * - `dest`: The recipient of the transfer.
       * - `keep_alive`: A boolean to determine if the `transfer_all` operation should send all
       * of the funds the account has, causing the sender account to be killed (false), or
       * transfer everything except at least the existential deposit, which will guarantee to
       * keep the sender account alive (true).
       **/
      transferAll: AugmentedSubmittable<(dest: MultiAddress | { Id: any } | { Index: any } | { Raw: any } | { Address32: any } | { Address20: any } | string | Uint8Array, keepAlive: bool | boolean | Uint8Array) => SubmittableExtrinsic<ApiType>, [MultiAddress, bool]>;
      /**
       * Transfer some liquid free balance to another account.
       * 
       * `transfer_allow_death` will set the `FreeBalance` of the sender and receiver.
       * If the sender's account is below the existential deposit as a result
       * of the transfer, the account will be reaped.
       * 
       * The dispatch origin for this call must be `Signed` by the transactor.
       **/
      transferAllowDeath: AugmentedSubmittable<(dest: MultiAddress | { Id: any } | { Index: any } | { Raw: any } | { Address32: any } | { Address20: any } | string | Uint8Array, value: Compact<u128> | AnyNumber | Uint8Array) => SubmittableExtrinsic<ApiType>, [MultiAddress, Compact<u128>]>;
      /**
       * Same as the [`transfer_allow_death`] call, but with a check that the transfer will not
       * kill the origin account.
       * 
       * 99% of the time you want [`transfer_allow_death`] instead.
       * 
       * [`transfer_allow_death`]: struct.Pallet.html#method.transfer
       **/
      transferKeepAlive: AugmentedSubmittable<(dest: MultiAddress | { Id: any } | { Index: any } | { Raw: any } | { Address32: any } | { Address20: any } | string | Uint8Array, value: Compact<u128> | AnyNumber | Uint8Array) => SubmittableExtrinsic<ApiType>, [MultiAddress, Compact<u128>]>;
      /**
       * Transfer some liquid free balance to another account.
       * 
       * `transfer` will set the `FreeBalance` of the sender and receiver.
       * If the sender's account is below the existential deposit as a result
       * of the transfer, the account will be reaped.
       * 
       * The dispatch origin for this call must be `Signed` by the transactor.
       * 
       * ## Complexity
       * - Dependent on arguments but not critical, given proper implementations for input config
       * types. See related functions below.
       * - It contains a limited number of reads and writes internally and no complex
       * computation.
       * 
       * Related functions:
       * 
       * - `ensure_can_withdraw` is always called internally but has a bounded complexity.
       * - Transferring balances to accounts that did not exist before will cause
       * `T::OnNewAccount::on_new_account` to be called.
       * - Removing enough funds from an account will trigger `T::DustRemoval::on_unbalanced`.
       * - `transfer_keep_alive` works the same way as `transfer`, but has an additional check
       * that the transfer will not kill the origin account.
       **/
      transferWithMemo: AugmentedSubmittable<(dest: MultiAddress | { Id: any } | { Index: any } | { Raw: any } | { Address32: any } | { Address20: any } | string | Uint8Array, value: Compact<u128> | AnyNumber | Uint8Array, memo: Option<PolymeshPrimitivesMemo> | null | Uint8Array | PolymeshPrimitivesMemo | string) => SubmittableExtrinsic<ApiType>, [MultiAddress, Compact<u128>, Option<PolymeshPrimitivesMemo>]>;
      /**
       * Upgrade a specified account.
       * 
       * - `origin`: Must be `Signed`.
       * - `who`: The account to be upgraded.
       * 
       * This will waive the transaction fee if at least all but 10% of the accounts needed to
       * be upgraded. (We let some not have to be upgraded just in order to allow for the
       * possibility of churn).
       **/
      upgradeAccounts: AugmentedSubmittable<(who: Vec<AccountId32> | (AccountId32 | string | Uint8Array)[]) => SubmittableExtrinsic<ApiType>, [Vec<AccountId32>]>;
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
    beefy: {
      /**
       * Report voter equivocation/misbehavior. This method will verify the
       * equivocation proof and validate the given key ownership proof
       * against the extracted offender. If both are valid, the offence
       * will be reported.
       **/
      reportDoubleVoting: AugmentedSubmittable<(equivocationProof: SpConsensusBeefyDoubleVotingProof | { first?: any; second?: any } | string | Uint8Array, keyOwnerProof: SpSessionMembershipProof | { session?: any; trieNodes?: any; validatorCount?: any } | string | Uint8Array) => SubmittableExtrinsic<ApiType>, [SpConsensusBeefyDoubleVotingProof, SpSessionMembershipProof]>;
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
      reportDoubleVotingUnsigned: AugmentedSubmittable<(equivocationProof: SpConsensusBeefyDoubleVotingProof | { first?: any; second?: any } | string | Uint8Array, keyOwnerProof: SpSessionMembershipProof | { session?: any; trieNodes?: any; validatorCount?: any } | string | Uint8Array) => SubmittableExtrinsic<ApiType>, [SpConsensusBeefyDoubleVotingProof, SpSessionMembershipProof]>;
      /**
       * Report fork voting equivocation. This method will verify the equivocation proof
       * and validate the given key ownership proof against the extracted offender.
       * If both are valid, the offence will be reported.
       **/
      reportForkVoting: AugmentedSubmittable<(equivocationProof: SpConsensusBeefyForkVotingProof | { vote?: any; ancestryProof?: any; header?: any } | string | Uint8Array, keyOwnerProof: SpSessionMembershipProof | { session?: any; trieNodes?: any; validatorCount?: any } | string | Uint8Array) => SubmittableExtrinsic<ApiType>, [SpConsensusBeefyForkVotingProof, SpSessionMembershipProof]>;
      /**
       * Report fork voting equivocation. This method will verify the equivocation proof
       * and validate the given key ownership proof against the extracted offender.
       * If both are valid, the offence will be reported.
       * 
       * This extrinsic must be called unsigned and it is expected that only
       * block authors will call it (validated in `ValidateUnsigned`), as such
       * if the block author is defined it will be defined as the equivocation
       * reporter.
       **/
      reportForkVotingUnsigned: AugmentedSubmittable<(equivocationProof: SpConsensusBeefyForkVotingProof | { vote?: any; ancestryProof?: any; header?: any } | string | Uint8Array, keyOwnerProof: SpSessionMembershipProof | { session?: any; trieNodes?: any; validatorCount?: any } | string | Uint8Array) => SubmittableExtrinsic<ApiType>, [SpConsensusBeefyForkVotingProof, SpSessionMembershipProof]>;
      /**
       * Report future block voting equivocation. This method will verify the equivocation proof
       * and validate the given key ownership proof against the extracted offender.
       * If both are valid, the offence will be reported.
       **/
      reportFutureBlockVoting: AugmentedSubmittable<(equivocationProof: SpConsensusBeefyFutureBlockVotingProof | { vote?: any } | string | Uint8Array, keyOwnerProof: SpSessionMembershipProof | { session?: any; trieNodes?: any; validatorCount?: any } | string | Uint8Array) => SubmittableExtrinsic<ApiType>, [SpConsensusBeefyFutureBlockVotingProof, SpSessionMembershipProof]>;
      /**
       * Report future block voting equivocation. This method will verify the equivocation proof
       * and validate the given key ownership proof against the extracted offender.
       * If both are valid, the offence will be reported.
       * 
       * This extrinsic must be called unsigned and it is expected that only
       * block authors will call it (validated in `ValidateUnsigned`), as such
       * if the block author is defined it will be defined as the equivocation
       * reporter.
       **/
      reportFutureBlockVotingUnsigned: AugmentedSubmittable<(equivocationProof: SpConsensusBeefyFutureBlockVotingProof | { vote?: any } | string | Uint8Array, keyOwnerProof: SpSessionMembershipProof | { session?: any; trieNodes?: any; validatorCount?: any } | string | Uint8Array) => SubmittableExtrinsic<ApiType>, [SpConsensusBeefyFutureBlockVotingProof, SpSessionMembershipProof]>;
      /**
       * Reset BEEFY consensus by setting a new BEEFY genesis at `delay_in_blocks` blocks in the
       * future.
       * 
       * Note: `delay_in_blocks` has to be at least 1.
       **/
      setNewGenesis: AugmentedSubmittable<(delayInBlocks: u32 | AnyNumber | Uint8Array) => SubmittableExtrinsic<ApiType>, [u32]>;
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
      claim: AugmentedSubmittable<(caId: PalletCorporateActionsCaId | { assetId?: any; localId?: any } | string | Uint8Array) => SubmittableExtrinsic<ApiType>, [PalletCorporateActionsCaId]>;
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
       * - `origin` is a signer that has permissions to act as an agent of `ca_id.asset_id`.
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
       * - `UnauthorizedAgent` if `origin` is not agent-permissioned for `asset_id`.
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
      distribute: AugmentedSubmittable<(caId: PalletCorporateActionsCaId | { assetId?: any; localId?: any } | string | Uint8Array, portfolio: Option<u64> | null | Uint8Array | u64 | AnyNumber, currency: PolymeshPrimitivesAssetAssetId | string | Uint8Array, perShare: u128 | AnyNumber | Uint8Array, amount: u128 | AnyNumber | Uint8Array, paymentAt: u64 | AnyNumber | Uint8Array, expiresAt: Option<u64> | null | Uint8Array | u64 | AnyNumber) => SubmittableExtrinsic<ApiType>, [PalletCorporateActionsCaId, Option<u64>, PolymeshPrimitivesAssetAssetId, u128, u128, u64, Option<u64>]>;
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
       * - `origin` is a signer that has permissions to act as an agent of `ca_id.asset_id`.
       * - `ca_id` identifies the CA with a capital distributions to push benefits for.
       * - `holder` to push benefits to.
       * 
       * # Errors
       * - `UnauthorizedAgent` if `origin` is not agent-permissioned for `asset_id`.
       * - `NoSuchDistribution` if there's no capital distribution for `ca_id`.
       * - `CannotClaimBeforeStart` if `now < payment_at`.
       * - `CannotClaimAfterExpiry` if `now > expiry_at.unwrap()`.
       * - `NoSuchCA` if `ca_id` does not identify an existing CA.
       * - `NotTargetedByCA` if the CA does not target `holder`.
       * - `BalanceAmountProductOverflowed` if `ba = balance * amount` would overflow.
       * - `BalanceAmountProductSupplyDivisionFailed` if `ba * supply` would overflow.
       * - Other errors can occur if the compliance manager rejects the transfer.
       **/
      pushBenefit: AugmentedSubmittable<(caId: PalletCorporateActionsCaId | { assetId?: any; localId?: any } | string | Uint8Array, holder: PolymeshPrimitivesIdentityId | string | Uint8Array) => SubmittableExtrinsic<ApiType>, [PalletCorporateActionsCaId, PolymeshPrimitivesIdentityId]>;
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
      reclaim: AugmentedSubmittable<(caId: PalletCorporateActionsCaId | { assetId?: any; localId?: any } | string | Uint8Array) => SubmittableExtrinsic<ApiType>, [PalletCorporateActionsCaId]>;
      /**
       * Removes a distribution that hasn't started yet,
       * unlocking the full amount in the distributor portfolio.
       * 
       * ## Arguments
       * - `origin` is a signer that has permissions to act as an agent of `ca_id.asset_id`.
       * - `ca_id` identifies the CA with a not-yet-started capital distribution to remove.
       * 
       * # Errors
       * - `UnauthorizedAgent` if `origin` is not agent-permissioned for `asset_id`.
       * - `NoSuchDistribution` if there's no capital distribution for `ca_id`.
       * - `DistributionStarted` if `payment_at <= now`.
       **/
      removeDistribution: AugmentedSubmittable<(caId: PalletCorporateActionsCaId | { assetId?: any; localId?: any } | string | Uint8Array) => SubmittableExtrinsic<ApiType>, [PalletCorporateActionsCaId]>;
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
       * - `origin` is a signer that has permissions to act as an agent of `asset_id`.
       * - `asset_id` to create the checkpoint for.
       * 
       * # Errors
       * - `UnauthorizedAgent` if the DID of `origin` isn't a permissioned agent for `asset_id`.
       * - `CounterOverflow` if the total checkpoint counter would overflow.
       **/
      createCheckpoint: AugmentedSubmittable<(assetId: PolymeshPrimitivesAssetAssetId | string | Uint8Array) => SubmittableExtrinsic<ApiType>, [PolymeshPrimitivesAssetAssetId]>;
      /**
       * Creates a schedule generating checkpoints
       * in the future at either a fixed time or at intervals.
       * 
       * The schedule starts out with `strong_ref_count(schedule_id) <- 0`.
       * 
       * # Arguments
       * - `origin` is a signer that has permissions to act as owner of `asset_id`.
       * - `asset_id` to create the schedule for.
       * - `schedule` that will generate checkpoints.
       * 
       * # Errors
       * - `UnauthorizedAgent` if the DID of `origin` isn't a permissioned agent for `asset_id`.
       * - `InsufficientAccountBalance` if the protocol fee could not be charged.
       * - `CounterOverflow` if the schedule ID or total checkpoint counters would overflow.
       * 
       * # Permissions
       * * Asset
       **/
      createSchedule: AugmentedSubmittable<(assetId: PolymeshPrimitivesAssetAssetId | string | Uint8Array, schedule: PolymeshPrimitivesCheckpointScheduleCheckpoints | { pending?: any } | string | Uint8Array) => SubmittableExtrinsic<ApiType>, [PolymeshPrimitivesAssetAssetId, PolymeshPrimitivesCheckpointScheduleCheckpoints]>;
      /**
       * Removes the checkpoint schedule of an asset identified by `id`.
       * 
       * # Arguments
       * - `origin` is a signer that has permissions to act as owner of `asset_id`.
       * - `asset_id` to remove the schedule from.
       * - `id` of the schedule, when it was created by `created_schedule`.
       * 
       * # Errors
       * - `UnauthorizedAgent` if the DID of `origin` isn't a permissioned agent for `asset_id`.
       * - `NoCheckpointSchedule` if `id` does not identify a schedule for this `asset_id`.
       * - `ScheduleNotRemovable` if `id` exists but is not removable.
       * 
       * # Permissions
       * * Asset
       **/
      removeSchedule: AugmentedSubmittable<(assetId: PolymeshPrimitivesAssetAssetId | string | Uint8Array, id: u64 | AnyNumber | Uint8Array) => SubmittableExtrinsic<ApiType>, [PolymeshPrimitivesAssetAssetId, u64]>;
      /**
       * Sets the max complexity of a schedule set for an arbitrary asset_id to `max_complexity`.
       * The new maximum is not enforced retroactively,
       * and only applies once new schedules are made.
       * 
       * Must be called as a PIP (requires "root").
       * 
       * # Arguments
       * - `origin` is the root origin.
       * - `max_complexity` allowed for an arbitrary asset's schedule set.
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
      disableMember: AugmentedSubmittable<(who: PolymeshPrimitivesIdentityId | string | Uint8Array, expiry: Option<u64> | null | Uint8Array | u64 | AnyNumber, at: Option<u64> | null | Uint8Array | u64 | AnyNumber) => SubmittableExtrinsic<ApiType>, [PolymeshPrimitivesIdentityId, Option<u64>, Option<u64>]>;
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
       * Adds a compliance requirement to an asset given by `asset_id`.
       * If there are duplicate ClaimTypes for a particular trusted issuer, duplicates are removed.
       * 
       * # Arguments
       * * origin - Signer of the dispatchable. It should be the owner of the asset
       * * asset_id - Symbol of the asset
       * * sender_conditions - Sender transfer conditions.
       * * receiver_conditions - Receiver transfer conditions.
       * 
       * # Permissions
       * * Asset
       **/
      addComplianceRequirement: AugmentedSubmittable<(assetId: PolymeshPrimitivesAssetAssetId | string | Uint8Array, senderConditions: Vec<PolymeshPrimitivesCondition> | (PolymeshPrimitivesCondition | { conditionType?: any; issuers?: any } | string | Uint8Array)[], receiverConditions: Vec<PolymeshPrimitivesCondition> | (PolymeshPrimitivesCondition | { conditionType?: any; issuers?: any } | string | Uint8Array)[]) => SubmittableExtrinsic<ApiType>, [PolymeshPrimitivesAssetAssetId, Vec<PolymeshPrimitivesCondition>, Vec<PolymeshPrimitivesCondition>]>;
      /**
       * Adds another default trusted claim issuer at the asset level.
       * 
       * # Arguments
       * * origin - Signer of the dispatchable. It should be the owner of the asset.
       * * asset_id - Symbol of the asset.
       * * issuer - IdentityId of the trusted claim issuer.
       * 
       * # Permissions
       * * Asset
       **/
      addDefaultTrustedClaimIssuer: AugmentedSubmittable<(assetId: PolymeshPrimitivesAssetAssetId | string | Uint8Array, issuer: PolymeshPrimitivesConditionTrustedIssuer | { issuer?: any; trustedFor?: any } | string | Uint8Array) => SubmittableExtrinsic<ApiType>, [PolymeshPrimitivesAssetAssetId, PolymeshPrimitivesConditionTrustedIssuer]>;
      /**
       * Modify an existing compliance requirement of a given asset.
       * 
       * # Arguments
       * * origin - Signer of the dispatchable. It should be the owner of the asset.
       * * asset_id - Symbol of the asset.
       * * new_req - Compliance requirement.
       * 
       * # Permissions
       * * Asset
       **/
      changeComplianceRequirement: AugmentedSubmittable<(assetId: PolymeshPrimitivesAssetAssetId | string | Uint8Array, newReq: PolymeshPrimitivesComplianceManagerComplianceRequirement | { senderConditions?: any; receiverConditions?: any; id?: any } | string | Uint8Array) => SubmittableExtrinsic<ApiType>, [PolymeshPrimitivesAssetAssetId, PolymeshPrimitivesComplianceManagerComplianceRequirement]>;
      /**
       * Pauses the verification of conditions for `asset_id` during transfers.
       * 
       * # Arguments
       * * origin - Signer of the dispatchable. It should be the owner of the asset
       * * asset_id - Symbol of the asset
       * 
       * # Permissions
       * * Asset
       **/
      pauseAssetCompliance: AugmentedSubmittable<(assetId: PolymeshPrimitivesAssetAssetId | string | Uint8Array) => SubmittableExtrinsic<ApiType>, [PolymeshPrimitivesAssetAssetId]>;
      /**
       * Removes a compliance requirement from an asset's compliance.
       * 
       * # Arguments
       * * origin - Signer of the dispatchable. It should be the owner of the asset
       * * asset_id - Symbol of the asset
       * * id - Compliance requirement id which is need to be removed
       * 
       * # Permissions
       * * Asset
       **/
      removeComplianceRequirement: AugmentedSubmittable<(assetId: PolymeshPrimitivesAssetAssetId | string | Uint8Array, id: u32 | AnyNumber | Uint8Array) => SubmittableExtrinsic<ApiType>, [PolymeshPrimitivesAssetAssetId, u32]>;
      /**
       * Removes the given `issuer` from the set of default trusted claim issuers at the asset level.
       * 
       * # Arguments
       * * origin - Signer of the dispatchable. It should be the owner of the asset.
       * * asset_id - Symbol of the asset.
       * * issuer - IdentityId of the trusted claim issuer.
       * 
       * # Permissions
       * * Asset
       **/
      removeDefaultTrustedClaimIssuer: AugmentedSubmittable<(assetId: PolymeshPrimitivesAssetAssetId | string | Uint8Array, issuer: PolymeshPrimitivesIdentityId | string | Uint8Array) => SubmittableExtrinsic<ApiType>, [PolymeshPrimitivesAssetAssetId, PolymeshPrimitivesIdentityId]>;
      /**
       * Replaces an asset's compliance with a new compliance.
       * 
       * Compliance requirements will be sorted (ascending by id) before
       * replacing the current requirements.
       * 
       * # Arguments
       * * `asset_id` - the asset asset_id,
       * * `asset_compliance - the new asset compliance.
       * 
       * # Errors
       * * `Unauthorized` if `origin` is not the owner of the asset_id.
       * * `DuplicateAssetCompliance` if `asset_compliance` contains multiple entries with the same `requirement_id`.
       * 
       * # Permissions
       * * Asset
       **/
      replaceAssetCompliance: AugmentedSubmittable<(assetId: PolymeshPrimitivesAssetAssetId | string | Uint8Array, assetCompliance: Vec<PolymeshPrimitivesComplianceManagerComplianceRequirement> | (PolymeshPrimitivesComplianceManagerComplianceRequirement | { senderConditions?: any; receiverConditions?: any; id?: any } | string | Uint8Array)[]) => SubmittableExtrinsic<ApiType>, [PolymeshPrimitivesAssetAssetId, Vec<PolymeshPrimitivesComplianceManagerComplianceRequirement>]>;
      /**
       * Removes an asset's compliance
       * 
       * # Arguments
       * * origin - Signer of the dispatchable. It should be the owner of the asset
       * * asset_id - Symbol of the asset
       * 
       * # Permissions
       * * Asset
       **/
      resetAssetCompliance: AugmentedSubmittable<(assetId: PolymeshPrimitivesAssetAssetId | string | Uint8Array) => SubmittableExtrinsic<ApiType>, [PolymeshPrimitivesAssetAssetId]>;
      /**
       * Resumes the verification of conditions for `asset_id` during transfers.
       * 
       * # Arguments
       * * origin - Signer of the dispatchable. It should be the owner of the asset
       * * asset_id - Symbol of the asset
       * 
       * # Permissions
       * * Asset
       **/
      resumeAssetCompliance: AugmentedSubmittable<(assetId: PolymeshPrimitivesAssetAssetId | string | Uint8Array) => SubmittableExtrinsic<ApiType>, [PolymeshPrimitivesAssetAssetId]>;
      /**
       * Generic tx
       **/
      [key: string]: SubmittableExtrinsicFunction<ApiType>;
    };
    contracts: {
      /**
       * Makes a call to an account, optionally transferring some balance.
       * 
       * # Parameters
       * 
       * * `dest`: Address of the contract to call.
       * * `value`: The balance to transfer from the `origin` to `dest`.
       * * `gas_limit`: The gas limit enforced when executing the constructor.
       * * `storage_deposit_limit`: The maximum amount of balance that can be charged from the
       * caller to pay for the storage consumed.
       * * `data`: The input data to pass to the contract.
       * 
       * * If the account is a smart-contract account, the associated code will be
       * executed and any value will be transferred.
       * * If the account is a regular account, any value will be transferred.
       * * If no account exists and the call value is not less than `existential_deposit`,
       * a regular account will be created and any value will be transferred.
       **/
      call: AugmentedSubmittable<(dest: MultiAddress | { Id: any } | { Index: any } | { Raw: any } | { Address32: any } | { Address20: any } | string | Uint8Array, value: Compact<u128> | AnyNumber | Uint8Array, gasLimit: SpWeightsWeightV2Weight | { refTime?: any; proofSize?: any } | string | Uint8Array, storageDepositLimit: Option<Compact<u128>> | null | Uint8Array | Compact<u128> | AnyNumber, data: Bytes | string | Uint8Array) => SubmittableExtrinsic<ApiType>, [MultiAddress, Compact<u128>, SpWeightsWeightV2Weight, Option<Compact<u128>>, Bytes]>;
      /**
       * Deprecated version if [`Self::call`] for use in an in-storage `Call`.
       **/
      callOldWeight: AugmentedSubmittable<(dest: MultiAddress | { Id: any } | { Index: any } | { Raw: any } | { Address32: any } | { Address20: any } | string | Uint8Array, value: Compact<u128> | AnyNumber | Uint8Array, gasLimit: Compact<u64> | AnyNumber | Uint8Array, storageDepositLimit: Option<Compact<u128>> | null | Uint8Array | Compact<u128> | AnyNumber, data: Bytes | string | Uint8Array) => SubmittableExtrinsic<ApiType>, [MultiAddress, Compact<u128>, Compact<u64>, Option<Compact<u128>>, Bytes]>;
      /**
       * Instantiates a contract from a previously deployed wasm binary.
       * 
       * This function is identical to [`Self::instantiate_with_code`] but without the
       * code deployment step. Instead, the `code_hash` of an on-chain deployed wasm binary
       * must be supplied.
       **/
      instantiate: AugmentedSubmittable<(value: Compact<u128> | AnyNumber | Uint8Array, gasLimit: SpWeightsWeightV2Weight | { refTime?: any; proofSize?: any } | string | Uint8Array, storageDepositLimit: Option<Compact<u128>> | null | Uint8Array | Compact<u128> | AnyNumber, codeHash: H256 | string | Uint8Array, data: Bytes | string | Uint8Array, salt: Bytes | string | Uint8Array) => SubmittableExtrinsic<ApiType>, [Compact<u128>, SpWeightsWeightV2Weight, Option<Compact<u128>>, H256, Bytes, Bytes]>;
      /**
       * Deprecated version if [`Self::instantiate`] for use in an in-storage `Call`.
       **/
      instantiateOldWeight: AugmentedSubmittable<(value: Compact<u128> | AnyNumber | Uint8Array, gasLimit: Compact<u64> | AnyNumber | Uint8Array, storageDepositLimit: Option<Compact<u128>> | null | Uint8Array | Compact<u128> | AnyNumber, codeHash: H256 | string | Uint8Array, data: Bytes | string | Uint8Array, salt: Bytes | string | Uint8Array) => SubmittableExtrinsic<ApiType>, [Compact<u128>, Compact<u64>, Option<Compact<u128>>, H256, Bytes, Bytes]>;
      /**
       * Instantiates a new contract from the supplied `code` optionally transferring
       * some balance.
       * 
       * This dispatchable has the same effect as calling [`Self::upload_code`] +
       * [`Self::instantiate`]. Bundling them together provides efficiency gains. Please
       * also check the documentation of [`Self::upload_code`].
       * 
       * # Parameters
       * 
       * * `value`: The balance to transfer from the `origin` to the newly created contract.
       * * `gas_limit`: The gas limit enforced when executing the constructor.
       * * `storage_deposit_limit`: The maximum amount of balance that can be charged/reserved
       * from the caller to pay for the storage consumed.
       * * `code`: The contract code to deploy in raw bytes.
       * * `data`: The input data to pass to the contract constructor.
       * * `salt`: Used for the address derivation. See [`Pallet::contract_address`].
       * 
       * Instantiation is executed as follows:
       * 
       * - The supplied `code` is deployed, and a `code_hash` is created for that code.
       * - If the `code_hash` already exists on the chain the underlying `code` will be shared.
       * - The destination address is computed based on the sender, code_hash and the salt.
       * - The smart-contract account is created at the computed address.
       * - The `value` is transferred to the new account.
       * - The `deploy` function is executed in the context of the newly-created account.
       **/
      instantiateWithCode: AugmentedSubmittable<(value: Compact<u128> | AnyNumber | Uint8Array, gasLimit: SpWeightsWeightV2Weight | { refTime?: any; proofSize?: any } | string | Uint8Array, storageDepositLimit: Option<Compact<u128>> | null | Uint8Array | Compact<u128> | AnyNumber, code: Bytes | string | Uint8Array, data: Bytes | string | Uint8Array, salt: Bytes | string | Uint8Array) => SubmittableExtrinsic<ApiType>, [Compact<u128>, SpWeightsWeightV2Weight, Option<Compact<u128>>, Bytes, Bytes, Bytes]>;
      /**
       * Deprecated version if [`Self::instantiate_with_code`] for use in an in-storage `Call`.
       **/
      instantiateWithCodeOldWeight: AugmentedSubmittable<(value: Compact<u128> | AnyNumber | Uint8Array, gasLimit: Compact<u64> | AnyNumber | Uint8Array, storageDepositLimit: Option<Compact<u128>> | null | Uint8Array | Compact<u128> | AnyNumber, code: Bytes | string | Uint8Array, data: Bytes | string | Uint8Array, salt: Bytes | string | Uint8Array) => SubmittableExtrinsic<ApiType>, [Compact<u128>, Compact<u64>, Option<Compact<u128>>, Bytes, Bytes, Bytes]>;
      /**
       * When a migration is in progress, this dispatchable can be used to run migration steps.
       * Calls that contribute to advancing the migration have their fees waived, as it's helpful
       * for the chain. Note that while the migration is in progress, the pallet will also
       * leverage the `on_idle` hooks to run migration steps.
       **/
      migrate: AugmentedSubmittable<(weightLimit: SpWeightsWeightV2Weight | { refTime?: any; proofSize?: any } | string | Uint8Array) => SubmittableExtrinsic<ApiType>, [SpWeightsWeightV2Weight]>;
      /**
       * Remove the code stored under `code_hash` and refund the deposit to its owner.
       * 
       * A code can only be removed by its original uploader (its owner) and only if it is
       * not used by any contract.
       **/
      removeCode: AugmentedSubmittable<(codeHash: H256 | string | Uint8Array) => SubmittableExtrinsic<ApiType>, [H256]>;
      /**
       * Privileged function that changes the code of an existing contract.
       * 
       * This takes care of updating refcounts and all other necessary operations. Returns
       * an error if either the `code_hash` or `dest` do not exist.
       * 
       * # Note
       * 
       * This does **not** change the address of the contract in question. This means
       * that the contract address is no longer derived from its code hash after calling
       * this dispatchable.
       **/
      setCode: AugmentedSubmittable<(dest: MultiAddress | { Id: any } | { Index: any } | { Raw: any } | { Address32: any } | { Address20: any } | string | Uint8Array, codeHash: H256 | string | Uint8Array) => SubmittableExtrinsic<ApiType>, [MultiAddress, H256]>;
      /**
       * Upload new `code` without instantiating a contract from it.
       * 
       * If the code does not already exist a deposit is reserved from the caller
       * and unreserved only when [`Self::remove_code`] is called. The size of the reserve
       * depends on the size of the supplied `code`.
       * 
       * If the code already exists in storage it will still return `Ok` and upgrades
       * the in storage version to the current
       * [`InstructionWeights::version`](InstructionWeights).
       * 
       * - `determinism`: If this is set to any other value but [`Determinism::Enforced`] then
       * the only way to use this code is to delegate call into it from an offchain execution.
       * Set to [`Determinism::Enforced`] if in doubt.
       * 
       * # Note
       * 
       * Anyone can instantiate a contract from any uploaded code and thus prevent its removal.
       * To avoid this situation a constructor could employ access control so that it can
       * only be instantiated by permissioned entities. The same is true when uploading
       * through [`Self::instantiate_with_code`].
       * 
       * Use [`Determinism::Relaxed`] exclusively for non-deterministic code. If the uploaded
       * code is deterministic, specifying [`Determinism::Relaxed`] will be disregarded and
       * result in higher gas costs.
       **/
      uploadCode: AugmentedSubmittable<(code: Bytes | string | Uint8Array, storageDepositLimit: Option<Compact<u128>> | null | Uint8Array | Compact<u128> | AnyNumber, determinism: PalletContractsWasmDeterminism | 'Enforced' | 'Relaxed' | number | Uint8Array) => SubmittableExtrinsic<ApiType>, [Bytes, Option<Compact<u128>>, PalletContractsWasmDeterminism]>;
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
       * - `origin` which must be an external agent of `ca_id.asset_id` with relevant permissions.
       * - `ca_id` of the CA to alter.
       * - `record_date`, if any, to calculate the impact of the CA.
       * If provided, this results in a scheduled balance snapshot ("checkpoint") at the date.
       * 
       * # Errors
       * - `UnauthorizedAgent` if `origin` is not agent-permissioned for `asset_id`.
       * - `NoSuchCA` if `id` does not identify an existing CA.
       * - When `record_date.is_some()`, other errors due to checkpoint scheduling may occur.
       * 
       * # Permissions
       * * Asset
       **/
      changeRecordDate: AugmentedSubmittable<(caId: PalletCorporateActionsCaId | { assetId?: any; localId?: any } | string | Uint8Array, recordDate: Option<PalletCorporateActionsRecordDateSpec> | null | Uint8Array | PalletCorporateActionsRecordDateSpec | { Scheduled: any } | { ExistingSchedule: any } | { Existing: any } | string) => SubmittableExtrinsic<ApiType>, [PalletCorporateActionsCaId, Option<PalletCorporateActionsRecordDateSpec>]>;
      /**
       * Initiates a CA for `asset_id` of `kind` with `details` and other provided arguments.
       * 
       * ## Arguments
       * - `origin` which must be an external agent of `asset_id` with relevant permissions.
       * - `asset_id` that the CA is made for.
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
       * - `UnauthorizedAgent` if `origin` is not agent-permissioned for `asset_id`.
       * - `CounterOverflow` in the unlikely event that so many CAs were created for this `asset_id`,
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
      initiateCorporateAction: AugmentedSubmittable<(assetId: PolymeshPrimitivesAssetAssetId | string | Uint8Array, kind: PalletCorporateActionsCaKind | 'PredictableBenefit' | 'UnpredictableBenefit' | 'IssuerNotice' | 'Reorganization' | 'Other' | number | Uint8Array, declDate: u64 | AnyNumber | Uint8Array, recordDate: Option<PalletCorporateActionsRecordDateSpec> | null | Uint8Array | PalletCorporateActionsRecordDateSpec | { Scheduled: any } | { ExistingSchedule: any } | { Existing: any } | string, details: Bytes | string | Uint8Array, targets: Option<PalletCorporateActionsTargetIdentities> | null | Uint8Array | PalletCorporateActionsTargetIdentities | { identities?: any; treatment?: any } | string, defaultWithholdingTax: Option<Permill> | null | Uint8Array | Permill | AnyNumber, withholdingTax: Option<Vec<ITuple<[PolymeshPrimitivesIdentityId, Permill]>>> | null | Uint8Array | Vec<ITuple<[PolymeshPrimitivesIdentityId, Permill]>> | ([PolymeshPrimitivesIdentityId | string | Uint8Array, Permill | AnyNumber | Uint8Array])[]) => SubmittableExtrinsic<ApiType>, [PolymeshPrimitivesAssetAssetId, PalletCorporateActionsCaKind, u64, Option<PalletCorporateActionsRecordDateSpec>, Bytes, Option<PalletCorporateActionsTargetIdentities>, Option<Permill>, Option<Vec<ITuple<[PolymeshPrimitivesIdentityId, Permill]>>>]>;
      initiateCorporateActionAndBallot: AugmentedSubmittable<(caArgs: PalletCorporateActionsInitiateCorporateActionArgs | { assetId?: any; kind?: any; declDate?: any; recordDate?: any; details?: any; targets?: any; defaultWithholdingTax?: any; withholdingTax?: any } | string | Uint8Array, ballotTimeRange: PalletCorporateActionsBallotBallotTimeRange | { start?: any; end?: any } | string | Uint8Array, ballotMeta: PalletCorporateActionsBallotBallotMeta | { title?: any; motions?: any } | string | Uint8Array, rcv: bool | boolean | Uint8Array) => SubmittableExtrinsic<ApiType>, [PalletCorporateActionsInitiateCorporateActionArgs, PalletCorporateActionsBallotBallotTimeRange, PalletCorporateActionsBallotBallotMeta, bool]>;
      /**
       * Utility extrinsic to batch `initiate_corporate_action` and `distribute`
       **/
      initiateCorporateActionAndDistribute: AugmentedSubmittable<(caArgs: PalletCorporateActionsInitiateCorporateActionArgs | { assetId?: any; kind?: any; declDate?: any; recordDate?: any; details?: any; targets?: any; defaultWithholdingTax?: any; withholdingTax?: any } | string | Uint8Array, portfolio: Option<u64> | null | Uint8Array | u64 | AnyNumber, currency: PolymeshPrimitivesAssetAssetId | string | Uint8Array, perShare: u128 | AnyNumber | Uint8Array, amount: u128 | AnyNumber | Uint8Array, paymentAt: u64 | AnyNumber | Uint8Array, expiresAt: Option<u64> | null | Uint8Array | u64 | AnyNumber) => SubmittableExtrinsic<ApiType>, [PalletCorporateActionsInitiateCorporateActionArgs, Option<u64>, PolymeshPrimitivesAssetAssetId, u128, u128, u64, Option<u64>]>;
      /**
       * Link the given CA `id` to the given `docs`.
       * Any previous links for the CA are removed in favor of `docs`.
       * 
       * The workflow here is to add the documents and initiating the CA in any order desired.
       * Once both exist, they can now be linked together.
       * 
       * ## Arguments
       * - `origin` which must be an external agent of `id.asset_id` with relevant permissions.
       * - `id` of the CA to associate with `docs`.
       * - `docs` to associate with the CA with `id`.
       * 
       * # Errors
       * - `UnauthorizedAgent` if `origin` is not agent-permissioned for `asset_id`.
       * - `NoSuchCA` if `id` does not identify an existing CA.
       * - `NoSuchDoc` if any of `docs` does not identify an existing document.
       * 
       * # Permissions
       * * Asset
       **/
      linkCaDoc: AugmentedSubmittable<(id: PalletCorporateActionsCaId | { assetId?: any; localId?: any } | string | Uint8Array, docs: Vec<u32> | (u32 | AnyNumber | Uint8Array)[]) => SubmittableExtrinsic<ApiType>, [PalletCorporateActionsCaId, Vec<u32>]>;
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
       * - `origin` which must be an external agent of `ca_id.asset_id` with relevant permissions.
       * - `ca_id` of the CA to remove.
       * 
       * # Errors
       * - `UnauthorizedAgent` if `origin` is not agent-permissioned for `asset_id`.
       * - `NoSuchCA` if `id` does not identify an existing CA.
       * 
       * # Permissions
       * * Asset
       **/
      removeCa: AugmentedSubmittable<(caId: PalletCorporateActionsCaId | { assetId?: any; localId?: any } | string | Uint8Array) => SubmittableExtrinsic<ApiType>, [PalletCorporateActionsCaId]>;
      /**
       * Set the default CA `TargetIdentities` to `targets`.
       * 
       * ## Arguments
       * - `origin` which must be an external agent of `asset_id` with relevant permissions.
       * - `asset_id` for which the default identities are changing.
       * - `targets` the default target identities for a CA.
       * 
       * ## Errors
       * - `UnauthorizedAgent` if `origin` is not agent-permissioned for `asset_id`.
       * - `TooManyTargetIds` if `targets.identities.len() > T::MaxTargetIds::get()`.
       * 
       * # Permissions
       * * Asset
       **/
      setDefaultTargets: AugmentedSubmittable<(assetId: PolymeshPrimitivesAssetAssetId | string | Uint8Array, targets: PalletCorporateActionsTargetIdentities | { identities?: any; treatment?: any } | string | Uint8Array) => SubmittableExtrinsic<ApiType>, [PolymeshPrimitivesAssetAssetId, PalletCorporateActionsTargetIdentities]>;
      /**
       * Set the default withholding tax for all DIDs and CAs relevant to this `asset_id`.
       * 
       * ## Arguments
       * - `origin` which must be an external agent of `asset_id` with relevant permissions.
       * - `asset_id` that the withholding tax will apply to.
       * - `tax` that should be withheld when distributing dividends, etc.
       * 
       * ## Errors
       * - `UnauthorizedAgent` if `origin` is not agent-permissioned for `asset_id`.
       * 
       * # Permissions
       * * Asset
       **/
      setDefaultWithholdingTax: AugmentedSubmittable<(assetId: PolymeshPrimitivesAssetAssetId | string | Uint8Array, tax: Permill | AnyNumber | Uint8Array) => SubmittableExtrinsic<ApiType>, [PolymeshPrimitivesAssetAssetId, Permill]>;
      /**
       * Set the withholding tax of `asset_id` for `taxed_did` to `tax`.
       * If `Some(tax)`, this overrides the default withholding tax of `asset_id` to `tax` for `taxed_did`.
       * Otherwise, if `None`, the default withholding tax will be used.
       * 
       * ## Arguments
       * - `origin` which must be an external agent of `asset_id` with relevant permissions.
       * - `asset_id` that the withholding tax will apply to.
       * - `taxed_did` that will have its withholding tax updated.
       * - `tax` that should be withheld when distributing dividends, etc.
       * 
       * ## Errors
       * - `UnauthorizedAgent` if `origin` is not agent-permissioned for `asset_id`.
       * - `TooManyDidTaxes` if `Some(tax)` and adding the override would go over the limit `MaxDidWhts`.
       * 
       * # Permissions
       * * Asset
       **/
      setDidWithholdingTax: AugmentedSubmittable<(assetId: PolymeshPrimitivesAssetAssetId | string | Uint8Array, taxedDid: PolymeshPrimitivesIdentityId | string | Uint8Array, tax: Option<Permill> | null | Uint8Array | Permill | AnyNumber) => SubmittableExtrinsic<ApiType>, [PolymeshPrimitivesAssetAssetId, PolymeshPrimitivesIdentityId, Option<Permill>]>;
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
       * - `origin` is a signer that has permissions to act as an agent of `ca_id.asset_id`.
       * - `ca_id` identifies the CA to attach the ballot to.
       * - `range` specifies when voting starts and ends.
       * - `meta` specifies the ballot's metadata as aforementioned.
       * - `rcv` specifies whether RCV is enabled for this ballot.
       * 
       * # Errors
       * - `UnauthorizedAgent` if `origin` is not agent-permissioned for `asset_id`.
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
      attachBallot: AugmentedSubmittable<(caId: PalletCorporateActionsCaId | { assetId?: any; localId?: any } | string | Uint8Array, range: PalletCorporateActionsBallotBallotTimeRange | { start?: any; end?: any } | string | Uint8Array, meta: PalletCorporateActionsBallotBallotMeta | { title?: any; motions?: any } | string | Uint8Array, rcv: bool | boolean | Uint8Array) => SubmittableExtrinsic<ApiType>, [PalletCorporateActionsCaId, PalletCorporateActionsBallotBallotTimeRange, PalletCorporateActionsBallotBallotMeta, bool]>;
      /**
       * Amend the end date of the ballot of the CA identified by `ca_id`.
       * 
       * ## Arguments
       * - `origin` is a signer that has permissions to act as an agent of `ca_id.asset_id`.
       * - `ca_id` identifies the attached ballot's CA.
       * - `end` specifies the new end date of the ballot.
       * 
       * # Errors
       * - `UnauthorizedAgent` if `origin` is not agent-permissioned for `asset_id`.
       * - `NoSuchBallot` if `ca_id` does not identify a ballot.
       * - `VotingAlreadyStarted` if `start >= now`, where `now` is the current time.
       * - `StartAfterEnd` if `start > end`.
       **/
      changeEnd: AugmentedSubmittable<(caId: PalletCorporateActionsCaId | { assetId?: any; localId?: any } | string | Uint8Array, end: u64 | AnyNumber | Uint8Array) => SubmittableExtrinsic<ApiType>, [PalletCorporateActionsCaId, u64]>;
      /**
       * Amend the metadata (title, motions, etc.) of the ballot of the CA identified by `ca_id`.
       * 
       * ## Arguments
       * - `origin` is a signer that has permissions to act as an agent of `ca_id.asset_id`.
       * - `ca_id` identifies the attached ballot's CA.
       * - `meta` specifies the new metadata.
       * 
       * # Errors
       * - `UnauthorizedAgent` if `origin` is not agent-permissioned for `asset_id`.
       * - `NoSuchBallot` if `ca_id` does not identify a ballot.
       * - `VotingAlreadyStarted` if `start >= now`, where `now` is the current time.
       * - `NumberOfChoicesOverflow` if the total choice in `meta` overflows `usize`.
       * - `TooLong` if any of the embedded strings in `meta` are too long.
       **/
      changeMeta: AugmentedSubmittable<(caId: PalletCorporateActionsCaId | { assetId?: any; localId?: any } | string | Uint8Array, meta: PalletCorporateActionsBallotBallotMeta | { title?: any; motions?: any } | string | Uint8Array) => SubmittableExtrinsic<ApiType>, [PalletCorporateActionsCaId, PalletCorporateActionsBallotBallotMeta]>;
      /**
       * Amend RCV support for the ballot of the CA identified by `ca_id`.
       * 
       * ## Arguments
       * - `origin` is a signer that has permissions to act as an agent of `ca_id.asset_id`.
       * - `ca_id` identifies the attached ballot's CA.
       * - `rcv` specifies if RCV is to be supported or not.
       * 
       * # Errors
       * - `UnauthorizedAgent` if `origin` is not agent-permissioned for `asset_id`.
       * - `NoSuchBallot` if `ca_id` does not identify a ballot.
       * - `VotingAlreadyStarted` if `start >= now`, where `now` is the current time.
       **/
      changeRcv: AugmentedSubmittable<(caId: PalletCorporateActionsCaId | { assetId?: any; localId?: any } | string | Uint8Array, rcv: bool | boolean | Uint8Array) => SubmittableExtrinsic<ApiType>, [PalletCorporateActionsCaId, bool]>;
      /**
       * Remove the ballot of the CA identified by `ca_id`.
       * 
       * ## Arguments
       * - `origin` is a signer that has permissions to act as an agent of `ca_id.asset_id`.
       * - `ca_id` identifies the attached ballot's CA.
       * 
       * # Errors
       * - `UnauthorizedAgent` if `origin` is not agent-permissioned for `asset_id`.
       * - `NoSuchBallot` if `ca_id` does not identify a ballot.
       * - `VotingAlreadyStarted` if `start >= now`, where `now` is the current time.
       **/
      removeBallot: AugmentedSubmittable<(caId: PalletCorporateActionsCaId | { assetId?: any; localId?: any } | string | Uint8Array) => SubmittableExtrinsic<ApiType>, [PalletCorporateActionsCaId]>;
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
      vote: AugmentedSubmittable<(caId: PalletCorporateActionsCaId | { assetId?: any; localId?: any } | string | Uint8Array, votes: Vec<PalletCorporateActionsBallotBallotVote> | (PalletCorporateActionsBallotBallotVote | { power?: any; fallback?: any } | string | Uint8Array)[]) => SubmittableExtrinsic<ApiType>, [PalletCorporateActionsCaId, Vec<PalletCorporateActionsBallotBallotVote>]>;
      /**
       * Generic tx
       **/
      [key: string]: SubmittableExtrinsicFunction<ApiType>;
    };
    didRegistrars: {
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
      disableMember: AugmentedSubmittable<(who: PolymeshPrimitivesIdentityId | string | Uint8Array, expiry: Option<u64> | null | Uint8Array | u64 | AnyNumber, at: Option<u64> | null | Uint8Array | u64 | AnyNumber) => SubmittableExtrinsic<ApiType>, [PolymeshPrimitivesIdentityId, Option<u64>, Option<u64>]>;
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
    electionProviderMultiPhase: {
      /**
       * Trigger the governance fallback.
       * 
       * This can only be called when [`Phase::Emergency`] is enabled, as an alternative to
       * calling [`Call::set_emergency_election_result`].
       **/
      governanceFallback: AugmentedSubmittable<() => SubmittableExtrinsic<ApiType>, []>;
      /**
       * Set a solution in the queue, to be handed out to the client of this pallet in the next
       * call to `ElectionProvider::elect`.
       * 
       * This can only be set by `T::ForceOrigin`, and only when the phase is `Emergency`.
       * 
       * The solution is not checked for any feasibility and is assumed to be trustworthy, as any
       * feasibility check itself can in principle cause the election process to fail (due to
       * memory/weight constrains).
       **/
      setEmergencyElectionResult: AugmentedSubmittable<(supports: Vec<ITuple<[AccountId32, SpNposElectionsSupport]>> | ([AccountId32 | string | Uint8Array, SpNposElectionsSupport | { total?: any; voters?: any } | string | Uint8Array])[]) => SubmittableExtrinsic<ApiType>, [Vec<ITuple<[AccountId32, SpNposElectionsSupport]>>]>;
      /**
       * Set a new value for `MinimumUntrustedScore`.
       * 
       * Dispatch origin must be aligned with `T::ForceOrigin`.
       * 
       * This check can be turned off by setting the value to `None`.
       **/
      setMinimumUntrustedScore: AugmentedSubmittable<(maybeNextScore: Option<SpNposElectionsElectionScore> | null | Uint8Array | SpNposElectionsElectionScore | { minimalStake?: any; sumStake?: any; sumStakeSquared?: any } | string) => SubmittableExtrinsic<ApiType>, [Option<SpNposElectionsElectionScore>]>;
      /**
       * Submit a solution for the signed phase.
       * 
       * The dispatch origin fo this call must be __signed__.
       * 
       * The solution is potentially queued, based on the claimed score and processed at the end
       * of the signed phase.
       * 
       * A deposit is reserved and recorded for the solution. Based on the outcome, the solution
       * might be rewarded, slashed, or get all or a part of the deposit back.
       **/
      submit: AugmentedSubmittable<(rawSolution: PalletElectionProviderMultiPhaseRawSolution | { solution?: any; score?: any; round?: any } | string | Uint8Array) => SubmittableExtrinsic<ApiType>, [PalletElectionProviderMultiPhaseRawSolution]>;
      /**
       * Submit a solution for the unsigned phase.
       * 
       * The dispatch origin fo this call must be __none__.
       * 
       * This submission is checked on the fly. Moreover, this unsigned solution is only
       * validated when submitted to the pool from the **local** node. Effectively, this means
       * that only active validators can submit this transaction when authoring a block (similar
       * to an inherent).
       * 
       * To prevent any incorrect solution (and thus wasted time/weight), this transaction will
       * panic if the solution submitted by the validator is invalid in any way, effectively
       * putting their authoring reward at risk.
       * 
       * No deposit or reward is associated with this submission.
       **/
      submitUnsigned: AugmentedSubmittable<(rawSolution: PalletElectionProviderMultiPhaseRawSolution | { solution?: any; score?: any; round?: any } | string | Uint8Array, witness: PalletElectionProviderMultiPhaseSolutionOrSnapshotSize | { voters?: any; targets?: any } | string | Uint8Array) => SubmittableExtrinsic<ApiType>, [PalletElectionProviderMultiPhaseRawSolution, PalletElectionProviderMultiPhaseSolutionOrSnapshotSize]>;
      /**
       * Generic tx
       **/
      [key: string]: SubmittableExtrinsicFunction<ApiType>;
    };
    externalAgents: {
      /**
       * Abdicate agentship for `asset_id`.
       * 
       * # Arguments
       * - `assetID` the [`AssetId] of which the caller is an agent.
       * 
       * # Errors
       * - `NotAnAgent` if the caller is not an agent of `asset_id`.
       * - `RemovingLastFullAgent` if the caller is the last full agent.
       * 
       * # Permissions
       * * Asset
       **/
      abdicate: AugmentedSubmittable<(assetId: PolymeshPrimitivesAssetAssetId | string | Uint8Array) => SubmittableExtrinsic<ApiType>, [PolymeshPrimitivesAssetAssetId]>;
      /**
       * Accept an authorization by an agent "Alice" who issued `auth_id`
       * to also become an agent of the asset Alice specified.
       * 
       * # Arguments
       * - `auth_id` identifying the authorization to accept.
       * 
       * # Errors
       * - `Error::InvalidAuthorization` if `auth_id` does not exist for the given caller.
       * - `Error::AuthorizationExpired` if `auth_id` is for an auth that has expired.
       * - `Error::BadAuthorizationType` if `auth_id` was not for a `BecomeAgent` auth type.
       * - `UnauthorizedAgent` if "Alice" is not permissioned to provide the auth.
       * - `NoSuchAG` if the group referred to a custom that does not exist.
       * - `AlreadyAnAgent` if the caller is already an agent of the asset.
       * 
       * # Permissions
       * * Agent
       **/
      acceptBecomeAgent: AugmentedSubmittable<(authId: u64 | AnyNumber | Uint8Array) => SubmittableExtrinsic<ApiType>, [u64]>;
      /**
       * Change the agent group that `agent` belongs to in `asset_id`.
       * 
       * # Arguments
       * - `assetID` the [`AssetId] that has the `agent`.
       * - `agent` of `asset_id` to change the group for.
       * - `group` that `agent` will belong to in `asset_id`.
       * 
       * # Errors
       * - `UnauthorizedAgent` if `origin` was not authorized as an agent to call this.
       * - `NoSuchAG` if `id` does not identify a custom AG.
       * - `NotAnAgent` if `agent` is not an agent of `asset_id`.
       * - `RemovingLastFullAgent` if `agent` was a `Full` one and is being demoted.
       * 
       * # Permissions
       * * Asset
       * * Agent
       **/
      changeGroup: AugmentedSubmittable<(assetId: PolymeshPrimitivesAssetAssetId | string | Uint8Array, agent: PolymeshPrimitivesIdentityId | string | Uint8Array, group: PolymeshPrimitivesAgentAgentGroup | { Full: any } | { Custom: any } | { ExceptMeta: any } | { PolymeshV1CAA: any } | { PolymeshV1PIA: any } | string | Uint8Array) => SubmittableExtrinsic<ApiType>, [PolymeshPrimitivesAssetAssetId, PolymeshPrimitivesIdentityId, PolymeshPrimitivesAgentAgentGroup]>;
      /**
       * Utility extrinsic to batch `create_group` and  `change_group` for custom groups only.
       * 
       * # Permissions
       * * Asset
       * * Agent
       **/
      createAndChangeCustomGroup: AugmentedSubmittable<(assetId: PolymeshPrimitivesAssetAssetId | string | Uint8Array, perms: PolymeshPrimitivesSecondaryKeyExtrinsicPermissions | { Whole: any } | { These: any } | { Except: any } | string | Uint8Array, agent: PolymeshPrimitivesIdentityId | string | Uint8Array) => SubmittableExtrinsic<ApiType>, [PolymeshPrimitivesAssetAssetId, PolymeshPrimitivesSecondaryKeyExtrinsicPermissions, PolymeshPrimitivesIdentityId]>;
      /**
       * Creates a custom agent group (AG) for the given `asset_id`.
       * 
       * The AG will have the permissions as given by `perms`.
       * This new AG is then assigned `id = AGIdSequence::get() + 1` as its `AGId`,
       * which you can use as `AgentGroup::Custom(id)` when adding agents for `asset_id`.
       * 
       * # Arguments
       * - `assetID` the [`AssetId] to add the custom group for.
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
      createGroup: AugmentedSubmittable<(assetId: PolymeshPrimitivesAssetAssetId | string | Uint8Array, perms: PolymeshPrimitivesSecondaryKeyExtrinsicPermissions | { Whole: any } | { These: any } | { Except: any } | string | Uint8Array) => SubmittableExtrinsic<ApiType>, [PolymeshPrimitivesAssetAssetId, PolymeshPrimitivesSecondaryKeyExtrinsicPermissions]>;
      /**
       * Utility extrinsic to batch `create_group` and  `add_auth`.
       * 
       * # Permissions
       * * Asset
       * * Agent
       **/
      createGroupAndAddAuth: AugmentedSubmittable<(assetId: PolymeshPrimitivesAssetAssetId | string | Uint8Array, perms: PolymeshPrimitivesSecondaryKeyExtrinsicPermissions | { Whole: any } | { These: any } | { Except: any } | string | Uint8Array, target: PolymeshPrimitivesIdentityId | string | Uint8Array, expiry: Option<u64> | null | Uint8Array | u64 | AnyNumber) => SubmittableExtrinsic<ApiType>, [PolymeshPrimitivesAssetAssetId, PolymeshPrimitivesSecondaryKeyExtrinsicPermissions, PolymeshPrimitivesIdentityId, Option<u64>]>;
      /**
       * Remove the given `agent` from `asset_id`.
       * 
       * # Arguments
       * - `assetID` the [`AssetId] that has the `agent` to remove.
       * - `agent` of `asset_id` to remove.
       * 
       * # Errors
       * - `UnauthorizedAgent` if `origin` was not authorized as an agent to call this.
       * - `NotAnAgent` if `agent` is not an agent of `asset_id`.
       * - `RemovingLastFullAgent` if `agent` is the last full one.
       * 
       * # Permissions
       * * Asset
       * * Agent
       **/
      removeAgent: AugmentedSubmittable<(assetId: PolymeshPrimitivesAssetAssetId | string | Uint8Array, agent: PolymeshPrimitivesIdentityId | string | Uint8Array) => SubmittableExtrinsic<ApiType>, [PolymeshPrimitivesAssetAssetId, PolymeshPrimitivesIdentityId]>;
      /**
       * Updates the permissions of the custom AG identified by `id`, for the given `asset_id`.
       * 
       * # Arguments
       * - `assetID` the [`AssetId] the custom AG belongs to.
       * - `id` for the custom AG within `asset_id`.
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
      setGroupPermissions: AugmentedSubmittable<(assetId: PolymeshPrimitivesAssetAssetId | string | Uint8Array, id: u32 | AnyNumber | Uint8Array, perms: PolymeshPrimitivesSecondaryKeyExtrinsicPermissions | { Whole: any } | { These: any } | { Except: any } | string | Uint8Array) => SubmittableExtrinsic<ApiType>, [PolymeshPrimitivesAssetAssetId, u32, PolymeshPrimitivesSecondaryKeyExtrinsicPermissions]>;
      /**
       * Generic tx
       **/
      [key: string]: SubmittableExtrinsicFunction<ApiType>;
    };
    grandpa: {
      /**
       * Note that the current authority set of the GRANDPA finality gadget has stalled.
       * 
       * This will trigger a forced authority set change at the beginning of the next session, to
       * be enacted `delay` blocks after that. The `delay` should be high enough to safely assume
       * that the block signalling the forced change will not be re-orged e.g. 1000 blocks.
       * The block production rate (which may be slowed down because of finality lagging) should
       * be taken into account when choosing the `delay`. The GRANDPA voters based on the new
       * authority will start voting on top of `best_finalized_block_number` for new finalized
       * blocks. `best_finalized_block_number` should be the highest of the latest finalized
       * block of all validators of the new authority set.
       * 
       * Only callable by root.
       **/
      noteStalled: AugmentedSubmittable<(delay: u32 | AnyNumber | Uint8Array, bestFinalizedBlockNumber: u32 | AnyNumber | Uint8Array) => SubmittableExtrinsic<ApiType>, [u32, u32]>;
      /**
       * Report voter equivocation/misbehavior. This method will verify the
       * equivocation proof and validate the given key ownership proof
       * against the extracted offender. If both are valid, the offence
       * will be reported.
       **/
      reportEquivocation: AugmentedSubmittable<(equivocationProof: SpConsensusGrandpaEquivocationProof | { setId?: any; equivocation?: any } | string | Uint8Array, keyOwnerProof: SpSessionMembershipProof | { session?: any; trieNodes?: any; validatorCount?: any } | string | Uint8Array) => SubmittableExtrinsic<ApiType>, [SpConsensusGrandpaEquivocationProof, SpSessionMembershipProof]>;
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
      reportEquivocationUnsigned: AugmentedSubmittable<(equivocationProof: SpConsensusGrandpaEquivocationProof | { setId?: any; equivocation?: any } | string | Uint8Array, keyOwnerProof: SpSessionMembershipProof | { session?: any; trieNodes?: any; validatorCount?: any } | string | Uint8Array) => SubmittableExtrinsic<ApiType>, [SpConsensusGrandpaEquivocationProof, SpSessionMembershipProof]>;
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
       * * `rotation_auth_id` Authorization from the owner who initiated the change
       **/
      acceptPrimaryKey: AugmentedSubmittable<(rotationAuthId: u64 | AnyNumber | Uint8Array) => SubmittableExtrinsic<ApiType>, [u64]>;
      /**
       * Adds an authorization.
       **/
      addAuthorization: AugmentedSubmittable<(target: PolymeshPrimitivesSecondaryKeySignatory | { Identity: any } | { Account: any } | string | Uint8Array, data: PolymeshPrimitivesAuthorizationAuthorizationData | { AttestPrimaryKeyRotation: any } | { RotatePrimaryKey: any } | { TransferTicker: any } | { AddMultiSigSigner: any } | { TransferAssetOwnership: any } | { JoinIdentity: any } | { PortfolioCustody: any } | { BecomeAgent: any } | { OldAddRelayerPayingKey: any } | { RotatePrimaryKeyToSecondary: any } | string | Uint8Array, expiry: Option<u64> | null | Uint8Array | u64 | AnyNumber) => SubmittableExtrinsic<ApiType>, [PolymeshPrimitivesSecondaryKeySignatory, PolymeshPrimitivesAuthorizationAuthorizationData, Option<u64>]>;
      /**
       * Adds a new claim record or edits an existing one.
       * 
       * Only called by did_issuer's secondary key.
       **/
      addClaim: AugmentedSubmittable<(target: PolymeshPrimitivesIdentityId | string | Uint8Array, claim: PolymeshPrimitivesIdentityClaimClaim | { Accredited: any } | { Affiliate: any } | { BuyLockup: any } | { SellLockup: any } | { CustomerDueDiligence: any } | { KnowYourCustomer: any } | { Jurisdiction: any } | { Exempted: any } | { Blocked: any } | { Custom: any } | string | Uint8Array, expiry: Option<u64> | null | Uint8Array | u64 | AnyNumber) => SubmittableExtrinsic<ApiType>, [PolymeshPrimitivesIdentityId, PolymeshPrimitivesIdentityClaimClaim, Option<u64>]>;
      /**
       * Adds secondary keys to target identity `id`.
       * 
       * Keys are directly added to identity because each of them has an authorization.
       * 
       * # Arguments
       * * `origin` which must be the primary key of the identity `id`.
       * * `additional_keys` which includes secondary keys,
       * coupled with authorization data, to add to target identity.
       * * `expires_at` expiration time for the authorizations.
       * 
       * # Errors
       * * Can only called by primary key owner.
       * * Keys should be able to linked to any identity.
       **/
      addSecondaryKeysWithAuthorization: AugmentedSubmittable<(additionalKeys: Vec<PolymeshPrimitivesIdentitySecondaryKeyWithAuth> | (PolymeshPrimitivesIdentitySecondaryKeyWithAuth | { secondaryKey?: any; authSignature?: any } | string | Uint8Array)[], expiresAt: u64 | AnyNumber | Uint8Array) => SubmittableExtrinsic<ApiType>, [Vec<PolymeshPrimitivesIdentitySecondaryKeyWithAuth>, u64]>;
      /**
       * Register `target_account` with a new Identity.
       * 
       * # Failure
       * - `origin` has to be an active DID registrar. Inactive DID registrars cannot register new
       * identities.
       * - `target_account` (primary key of the new Identity) can be linked to just one and only
       * one identity.
       * - External secondary keys can be linked to just one identity.
       **/
      cddRegisterDid: AugmentedSubmittable<(targetAccount: AccountId32 | string | Uint8Array, secondaryKeys: Vec<PolymeshPrimitivesSecondaryKey> | (PolymeshPrimitivesSecondaryKey | { key?: any; permissions?: any } | string | Uint8Array)[]) => SubmittableExtrinsic<ApiType>, [AccountId32, Vec<PolymeshPrimitivesSecondaryKey>]>;
      /**
       * Register `target_account` with a new Identity and issue a CDD claim with a blank CddId
       * 
       * # Failure
       * - `origin` has to be an active DID registrar. Inactive DID registrars cannot register new
       * identities.
       * - `target_account` (primary key of the new Identity) can be linked to just one and only
       * one identity.
       * - External secondary keys can be linked to just one identity.
       **/
      cddRegisterDidWithCdd: AugmentedSubmittable<(targetAccount: AccountId32 | string | Uint8Array, secondaryKeys: Vec<PolymeshPrimitivesSecondaryKey> | (PolymeshPrimitivesSecondaryKey | { key?: any; permissions?: any } | string | Uint8Array)[], expiry: Option<u64> | null | Uint8Array | u64 | AnyNumber) => SubmittableExtrinsic<ApiType>, [AccountId32, Vec<PolymeshPrimitivesSecondaryKey>, Option<u64>]>;
      /**
       * Create a child identities.
       * 
       * The new primary key for each child identity will need to sign (off-chain)
       * an authorization.
       * 
       * Only the primary key can create child identities.
       * 
       * # Arguments
       * - `child_keys` the keys that will become primary keys of their own child identity.
       * 
       * # Errors
       * - `KeyNotAllowed` only the primary key can create a new identity.
       * - `AlreadyLinked` one of the keys is already linked to an identity.
       * - `DuplicateKey` one of the keys is included multiple times.
       * - `IsChildIdentity` the caller's identity is already a child identity and can't create child identities.
       **/
      createChildIdentities: AugmentedSubmittable<(childKeys: Vec<PolymeshPrimitivesIdentityCreateChildIdentityWithAuth> | (PolymeshPrimitivesIdentityCreateChildIdentityWithAuth | { key?: any; authSignature?: any } | string | Uint8Array)[], expiresAt: u64 | AnyNumber | Uint8Array) => SubmittableExtrinsic<ApiType>, [Vec<PolymeshPrimitivesIdentityCreateChildIdentityWithAuth>, u64]>;
      /**
       * Create a child identity and make the `secondary_key` it's primary key.
       * 
       * Only the primary key can create child identities.
       * 
       * # Arguments
       * - `secondary_key` the secondary key that will become the primary key of the new identity.
       * 
       * # Errors
       * - `KeyNotAllowed` only the primary key can create a new identity.
       * - `NotASigner` the `secondary_key` is not a secondary key of the caller's identity.
       * - `AccountKeyIsBeingUsed` the `secondary_key` can't be unlinked from it's current identity.
       * - `IsChildIdentity` the caller's identity is already a child identity and can't create child identities.
       **/
      createChildIdentity: AugmentedSubmittable<(secondaryKey: AccountId32 | string | Uint8Array) => SubmittableExtrinsic<ApiType>, [AccountId32]>;
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
       * Join an identity as a secondary key.
       **/
      joinIdentityAsKey: AugmentedSubmittable<(authId: u64 | AnyNumber | Uint8Array) => SubmittableExtrinsic<ApiType>, [u64]>;
      /**
       * Leave the secondary key's identity.
       **/
      leaveIdentityAsKey: AugmentedSubmittable<() => SubmittableExtrinsic<ApiType>, []>;
      /**
       * Register custom claim type.
       * 
       * # Errors
       * * `CustomClaimTypeAlreadyExists` The type that is being registered already exists.
       * * `CounterOverflow` CustomClaimTypeId has overflowed.
       * * `TooLong` The type being registered is too lang.
       **/
      registerCustomClaimType: AugmentedSubmittable<(ty: Bytes | string | Uint8Array) => SubmittableExtrinsic<ApiType>, [Bytes]>;
      /**
       * Register a new DID for the target account.
       * 
       * Caller must be a DID registrar (formerly CDD provider).
       * No CDD claim is added - DID existence is sufficient for onboarding.
       * 
       * Note: This extrinsic does NOT support secondary keys.
       * Use the deprecated `cdd_register_did` if secondary keys are needed.
       * 
       * # Failure
       * - `origin` has to be an active DID registrar. Inactive DID registrars cannot register new
       * identities.
       * - `target_account` (primary key of the new Identity) can be linked to just one and only
       * one identity.
       **/
      registerDid: AugmentedSubmittable<(targetAccount: AccountId32 | string | Uint8Array) => SubmittableExtrinsic<ApiType>, [AccountId32]>;
      /**
       * Removes an authorization.
       * `_auth_issuer_pays` determines whether the issuer of the authorisation pays the transaction fee
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
       * Marks the specified claim as revoked.
       **/
      revokeClaim: AugmentedSubmittable<(target: PolymeshPrimitivesIdentityId | string | Uint8Array, claim: PolymeshPrimitivesIdentityClaimClaim | { Accredited: any } | { Affiliate: any } | { BuyLockup: any } | { SellLockup: any } | { CustomerDueDiligence: any } | { KnowYourCustomer: any } | { Jurisdiction: any } | { Exempted: any } | { Blocked: any } | { Custom: any } | string | Uint8Array) => SubmittableExtrinsic<ApiType>, [PolymeshPrimitivesIdentityId, PolymeshPrimitivesIdentityClaimClaim]>;
      /**
       * Revokes a specific claim using its [Claim Unique Index](/pallet_identity/index.html#claim-unique-index) composed by `target`,
       * `claim_type`, and `scope`.
       * 
       * Please note that `origin` must be the issuer of the target claim.
       **/
      revokeClaimByIndex: AugmentedSubmittable<(target: PolymeshPrimitivesIdentityId | string | Uint8Array, claimType: PolymeshPrimitivesIdentityClaimClaimType | { Accredited: any } | { Affiliate: any } | { BuyLockup: any } | { SellLockup: any } | { CustomerDueDiligence: any } | { KnowYourCustomer: any } | { Jurisdiction: any } | { Exempted: any } | { Blocked: any } | { Custom: any } | string | Uint8Array, scope: Option<PolymeshPrimitivesIdentityClaimScope> | null | Uint8Array | PolymeshPrimitivesIdentityClaimScope | { Identity: any } | { Asset: any } | { Custom: any } | string) => SubmittableExtrinsic<ApiType>, [PolymeshPrimitivesIdentityId, PolymeshPrimitivesIdentityClaimClaimType, Option<PolymeshPrimitivesIdentityClaimScope>]>;
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
       * * `auth_id` Authorization from the owner who initiated the change
       **/
      rotatePrimaryKeyToSecondary: AugmentedSubmittable<(authId: u64 | AnyNumber | Uint8Array) => SubmittableExtrinsic<ApiType>, [u64]>;
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
       * Unlink a child identity from it's parent identity.
       * 
       * Only the primary key of the parent or child identities can unlink the identities.
       * 
       * # Arguments
       * - `child_did` the child identity to unlink from its parent identity.
       * 
       * # Errors
       * - `KeyNotAllowed` only the primary key of either the parent or child identity can unlink the identities.
       * - `NoParentIdentity` the identity `child_did` doesn't have a parent identity.
       * - `NotParentOrChildIdentity` the caller's identity isn't the parent or child identity.
       **/
      unlinkChildIdentity: AugmentedSubmittable<(childDid: PolymeshPrimitivesIdentityId | string | Uint8Array) => SubmittableExtrinsic<ApiType>, [PolymeshPrimitivesIdentityId]>;
      /**
       * Generic tx
       **/
      [key: string]: SubmittableExtrinsicFunction<ApiType>;
    };
    imOnline: {
      /**
       * ## Complexity:
       * - `O(K)` where K is length of `Keys` (heartbeat.validators_len)
       * - `O(K)`: decoding of length `K`
       **/
      heartbeat: AugmentedSubmittable<(heartbeat: PalletImOnlineHeartbeat | { blockNumber?: any; sessionIndex?: any; authorityIndex?: any; validatorsLen?: any } | string | Uint8Array, signature: PalletImOnlineSr25519AppSr25519Signature | string | Uint8Array) => SubmittableExtrinsic<ApiType>, [PalletImOnlineHeartbeat, PalletImOnlineSr25519AppSr25519Signature]>;
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
       * ## Complexity
       * - `O(1)`.
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
       * ## Complexity
       * - `O(1)`.
       **/
      forceTransfer: AugmentedSubmittable<(updated: MultiAddress | { Id: any } | { Index: any } | { Raw: any } | { Address32: any } | { Address20: any } | string | Uint8Array, index: u32 | AnyNumber | Uint8Array, freeze: bool | boolean | Uint8Array) => SubmittableExtrinsic<ApiType>, [MultiAddress, u32, bool]>;
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
       * ## Complexity
       * - `O(1)`.
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
       * ## Complexity
       * - `O(1)`.
       **/
      freeze: AugmentedSubmittable<(index: u32 | AnyNumber | Uint8Array) => SubmittableExtrinsic<ApiType>, [u32]>;
      /**
       * Poke the deposit reserved for an index.
       * 
       * The dispatch origin for this call must be _Signed_ and the signing account must have a
       * non-frozen account `index`.
       * 
       * The transaction fees is waived if the deposit is changed after poking/reconsideration.
       * 
       * - `index`: the index whose deposit is to be poked/reconsidered.
       * 
       * Emits `DepositPoked` if successful.
       **/
      pokeDeposit: AugmentedSubmittable<(index: u32 | AnyNumber | Uint8Array) => SubmittableExtrinsic<ApiType>, [u32]>;
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
       * ## Complexity
       * - `O(1)`.
       **/
      transfer: AugmentedSubmittable<(updated: MultiAddress | { Id: any } | { Index: any } | { Raw: any } | { Address32: any } | { Address20: any } | string | Uint8Array, index: u32 | AnyNumber | Uint8Array) => SubmittableExtrinsic<ApiType>, [MultiAddress, u32]>;
      /**
       * Generic tx
       **/
      [key: string]: SubmittableExtrinsicFunction<ApiType>;
    };
    multiSig: {
      /**
       * Accepts a multisig signer authorization given to signer's key (AccountId).
       * 
       * # Arguments
       * * `auth_id` - Auth id of the authorization.
       **/
      acceptMultisigSigner: AugmentedSubmittable<(authId: u64 | AnyNumber | Uint8Array) => SubmittableExtrinsic<ApiType>, [u64]>;
      /**
       * Add an admin identity to the multisig.  This must be called by the multisig itself.
       **/
      addAdmin: AugmentedSubmittable<(adminDid: PolymeshPrimitivesIdentityId | string | Uint8Array) => SubmittableExtrinsic<ApiType>, [PolymeshPrimitivesIdentityId]>;
      /**
       * Adds signers to the multisig.  This must be called by the multisig itself.
       * 
       * # Arguments
       * * `signers` - Signers to add.
       **/
      addMultisigSigners: AugmentedSubmittable<(signers: Vec<AccountId32> | (AccountId32 | string | Uint8Array)[]) => SubmittableExtrinsic<ApiType>, [Vec<AccountId32>]>;
      /**
       * Adds a signer to the multisig.  This must be called by the admin identity of the
       * multisig.
       * 
       * # Arguments
       * * `multisig` - Address of the multi sig
       * * `signers` - Signers to add.
       * 
       **/
      addMultisigSignersViaAdmin: AugmentedSubmittable<(multisig: AccountId32 | string | Uint8Array, signers: Vec<AccountId32> | (AccountId32 | string | Uint8Array)[]) => SubmittableExtrinsic<ApiType>, [AccountId32, Vec<AccountId32>]>;
      /**
       * Approves a multisig proposal using the caller's secondary key (`AccountId`).
       * 
       * # Arguments
       * * `multisig` - MultiSig address.
       * * `proposal_id` - Proposal id to approve.
       * * `max_weight` - The maximum weight to execute the proposal.
       * 
       * If quorum is reached, the proposal will be immediately executed.
       **/
      approve: AugmentedSubmittable<(multisig: AccountId32 | string | Uint8Array, proposalId: u64 | AnyNumber | Uint8Array, maxWeight: Option<SpWeightsWeightV2Weight> | null | Uint8Array | SpWeightsWeightV2Weight | { refTime?: any; proofSize?: any } | string) => SubmittableExtrinsic<ApiType>, [AccountId32, u64, Option<SpWeightsWeightV2Weight>]>;
      /**
       * Approves a multisig join identity proposal.
       * 
       * # Arguments
       * * `multisig` - MultiSig address.
       * * `auth_id` - The join identity authorization to approve.
       * 
       * If quorum is reached, the join identity proposal will be immediately executed.
       **/
      approveJoinIdentity: AugmentedSubmittable<(multisig: AccountId32 | string | Uint8Array, authId: u64 | AnyNumber | Uint8Array) => SubmittableExtrinsic<ApiType>, [AccountId32, u64]>;
      /**
       * Changes the number of signatures required by a multisig.  This must be called by the
       * multisig itself.
       * 
       * # Arguments
       * * `sigs_required` - New number of required signatures.
       **/
      changeSigsRequired: AugmentedSubmittable<(sigsRequired: u64 | AnyNumber | Uint8Array) => SubmittableExtrinsic<ApiType>, [u64]>;
      /**
       * Changes the number of signatures required by a multisig.  This must be called by the admin of the multisig.
       * 
       * # Arguments
       * * `multisig` - The account identifier ([`AccountId`]) for the multi signature account.
       * * `signatures_required` - The number of required signatures.
       **/
      changeSigsRequiredViaAdmin: AugmentedSubmittable<(multisig: AccountId32 | string | Uint8Array, signaturesRequired: u64 | AnyNumber | Uint8Array) => SubmittableExtrinsic<ApiType>, [AccountId32, u64]>;
      /**
       * Creates a multisig
       * 
       * # Arguments
       * * `signers` - Signers of the multisig (They need to accept authorization before they are actually added).
       * * `sigs_required` - Number of sigs required to process a multi-sig tx.
       * * `permissions` - optional custom permissions.  Only the primary key can provide custom permissions.
       **/
      createMultisig: AugmentedSubmittable<(signers: Vec<AccountId32> | (AccountId32 | string | Uint8Array)[], sigsRequired: u64 | AnyNumber | Uint8Array, permissions: Option<PolymeshPrimitivesSecondaryKeyPermissions> | null | Uint8Array | PolymeshPrimitivesSecondaryKeyPermissions | { asset?: any; extrinsic?: any; portfolio?: any } | string) => SubmittableExtrinsic<ApiType>, [Vec<AccountId32>, u64, Option<PolymeshPrimitivesSecondaryKeyPermissions>]>;
      /**
       * Creates a multisig proposal
       * 
       * # Arguments
       * * `multisig` - MultiSig address.
       * * `proposal` - Proposal to be voted on.
       * * `expiry` - Optional proposal expiry time.
       * 
       * If this is 1 out of `m` multisig, the proposal will be immediately executed.
       **/
      createProposal: AugmentedSubmittable<(multisig: AccountId32 | string | Uint8Array, proposal: Call | IMethod | string | Uint8Array, expiry: Option<u64> | null | Uint8Array | u64 | AnyNumber) => SubmittableExtrinsic<ApiType>, [AccountId32, Call, Option<u64>]>;
      /**
       * Accept a JoinIdentity authorization for this multisig.  This must be called by the multisig itself.
       **/
      joinIdentity: AugmentedSubmittable<(authId: u64 | AnyNumber | Uint8Array) => SubmittableExtrinsic<ApiType>, [u64]>;
      /**
       * Rejects a multisig proposal using the caller's secondary key (`AccountId`).
       * 
       * # Arguments
       * * `multisig` - MultiSig address.
       * * `proposal_id` - Proposal id to reject.
       * If quorum is reached, the proposal will be immediately executed.
       **/
      reject: AugmentedSubmittable<(multisig: AccountId32 | string | Uint8Array, proposalId: u64 | AnyNumber | Uint8Array) => SubmittableExtrinsic<ApiType>, [AccountId32, u64]>;
      /**
       * Removes the admin identity from the `multisig`.  This must be called by the multisig itself.
       **/
      removeAdmin: AugmentedSubmittable<() => SubmittableExtrinsic<ApiType>, []>;
      /**
       * Removes the admin identity from the `multisig`.  This must be called by the admin of the multisig.
       **/
      removeAdminViaAdmin: AugmentedSubmittable<(multisig: AccountId32 | string | Uint8Array) => SubmittableExtrinsic<ApiType>, [AccountId32]>;
      /**
       * Removes signers from the multisig.  This must be called by the multisig itself.
       * 
       * # Arguments
       * * `signers` - Signers to remove.
       **/
      removeMultisigSigners: AugmentedSubmittable<(signers: Vec<AccountId32> | (AccountId32 | string | Uint8Array)[]) => SubmittableExtrinsic<ApiType>, [Vec<AccountId32>]>;
      /**
       * Removes a signer from the multisig.
       * This must be called by the admin identity of the multisig.
       * 
       * # Arguments
       * * `multisig` - Address of the multisig.
       * * `signers` - Signers to remove.
       * 
       **/
      removeMultisigSignersViaAdmin: AugmentedSubmittable<(multisig: AccountId32 | string | Uint8Array, signers: Vec<AccountId32> | (AccountId32 | string | Uint8Array)[]) => SubmittableExtrinsic<ApiType>, [AccountId32, Vec<AccountId32>]>;
      /**
       * Removes the paying identity from the `multisig`.  This must be called by the multisig itself.
       **/
      removePayer: AugmentedSubmittable<() => SubmittableExtrinsic<ApiType>, []>;
      /**
       * Removes the paying identity from the `multisig`.  This must be called by the paying identity of the multisig.
       **/
      removePayerViaPayer: AugmentedSubmittable<(multisig: AccountId32 | string | Uint8Array) => SubmittableExtrinsic<ApiType>, [AccountId32]>;
      /**
       * Generic tx
       **/
      [key: string]: SubmittableExtrinsicFunction<ApiType>;
    };
    nft: {
      /**
       * Forces the transfer of NFTs from a given portfolio to the caller's portfolio.
       * 
       * # Arguments
       * * `origin` - is a signer that has permissions to act as an agent of `asset_id`.
       * * `nft_id` - the [`NFTId`] of the NFT to be transferred.
       * * `source` - the [`AssetHolder`] that currently holds the NFT.
       * * `callers_holdings_kind` - the [`AssetHolderKind`] of the caller's portfolio.
       * 
       * # Permissions
       * * Asset
       * * Portfolio
       **/
      controllerTransfer: AugmentedSubmittable<(nfts: PolymeshPrimitivesNftNfTs | { assetId?: any; ids?: any } | string | Uint8Array, source: PolymeshPrimitivesAssetAssetHolder | { Portfolio: any } | { Account: any } | string | Uint8Array, callersHoldingsKind: PolymeshPrimitivesAssetAssetHolderKind | { Account: any } | { DefaultPortfolio: any } | { UserPortfolio: any } | string | Uint8Array) => SubmittableExtrinsic<ApiType>, [PolymeshPrimitivesNftNfTs, PolymeshPrimitivesAssetAssetHolder, PolymeshPrimitivesAssetAssetHolderKind]>;
      /**
       * Cretes a new `NFTCollection`.
       * 
       * # Arguments
       * * `origin` - contains the secondary key of the caller (i.e. who signed the transaction to execute this function).
       * * `asset_id` - optional [`AssetId`] associated to the new collection. `None` will create a new asset.
       * * `nft_type` - in case the asset hasn't been created yet, one will be created with the given type.
       * * `collection_keys` - all mandatory metadata keys that the tokens in the collection must have.
       * 
       * ## Errors
       * - `CollectionAlredyRegistered` - if the asset_id is already associated to an NFT collection.
       * - `InvalidAssetType` - if the associated asset is not of type NFT.
       * - `MaxNumberOfKeysExceeded` - if the number of metadata keys for the collection is greater than the maximum allowed.
       * - `UnregisteredMetadataKey` - if any of the metadata keys needed for the collection has not been registered.
       * - `DuplicateMetadataKey` - if a duplicate metadata keys has been passed as input.
       * 
       * # Permissions
       * * Asset
       **/
      createNftCollection: AugmentedSubmittable<(assetId: Option<PolymeshPrimitivesAssetAssetId> | null | Uint8Array | PolymeshPrimitivesAssetAssetId | string, nftType: Option<PolymeshPrimitivesAssetNonFungibleType> | null | Uint8Array | PolymeshPrimitivesAssetNonFungibleType | { Derivative: any } | { FixedIncome: any } | { Invoice: any } | { Custom: any } | string, collectionKeys: PolymeshPrimitivesNftNftCollectionKeys) => SubmittableExtrinsic<ApiType>, [Option<PolymeshPrimitivesAssetAssetId>, Option<PolymeshPrimitivesAssetNonFungibleType>, PolymeshPrimitivesNftNftCollectionKeys]>;
      /**
       * Issues an NFT to the caller.
       * 
       * # Arguments
       * * `origin` - is a signer that has permissions to act as an agent of `asset_id`.
       * * `asset_id` - the [`AssetId`] of the NFT collection.
       * * `nft_metadata_attributes` - all mandatory metadata keys and values for the NFT.
       * - `holdings_kind` - the [`AssetHolderKind`] that will receive the minted nft.
       * 
       * ## Errors
       * - `CollectionNotFound` - if the collection associated to the given asset_id has not been created.
       * - `InvalidMetadataAttribute` - if the number of attributes is not equal to the number set in the collection or attempting to set a value for a key not definied in the collection.
       * - `DuplicateMetadataKey` - if a duplicate metadata keys has been passed as input.
       * 
       * 
       * # Permissions
       * * Asset
       * * Portfolio
       **/
      issueNft: AugmentedSubmittable<(assetId: PolymeshPrimitivesAssetAssetId | string | Uint8Array, nftMetadataAttributes: Vec<PolymeshPrimitivesNftNftMetadataAttribute> | (PolymeshPrimitivesNftNftMetadataAttribute | { key?: any; value?: any } | string | Uint8Array)[], holdingsKind: PolymeshPrimitivesAssetAssetHolderKind | { Account: any } | { DefaultPortfolio: any } | { UserPortfolio: any } | string | Uint8Array) => SubmittableExtrinsic<ApiType>, [PolymeshPrimitivesAssetAssetId, Vec<PolymeshPrimitivesNftNftMetadataAttribute>, PolymeshPrimitivesAssetAssetHolderKind]>;
      /**
       * Redeems the given NFT from the caller's portfolio.
       * 
       * # Arguments
       * * `origin` - is a signer that has permissions to act as an agent of `asset_id`.
       * * `asset_id` - the [`AssetId`] of the NFT collection.
       * * `nft_id` - the id of the NFT to be burned.
       * * `holdings_kind` - the [`AssetHolderKind`] that contains the nft.
       * 
       * ## Errors
       * - `CollectionNotFound` - if the collection associated to the given asset_id has not been created.
       * - `NFTNotFound` - if the given NFT does not exist in the portfolio.
       * 
       * # Permissions
       * * Asset
       * * Portfolio
       **/
      redeemNft: AugmentedSubmittable<(assetId: PolymeshPrimitivesAssetAssetId | string | Uint8Array, nftId: u64 | AnyNumber | Uint8Array, holdingsKind: PolymeshPrimitivesAssetAssetHolderKind | { Account: any } | { DefaultPortfolio: any } | { UserPortfolio: any } | string | Uint8Array, numberOfKeys: Option<u8> | null | Uint8Array | u8 | AnyNumber) => SubmittableExtrinsic<ApiType>, [PolymeshPrimitivesAssetAssetId, u64, PolymeshPrimitivesAssetAssetHolderKind, Option<u8>]>;
      /**
       * Generic tx
       **/
      [key: string]: SubmittableExtrinsicFunction<ApiType>;
    };
    pips: {
      /**
       * Approves the pending committee PIP given by the `id`.
       * 
       * This function can only be called by a Governance Committee (GC) voting majority.
       * 
       * # Arguments
       * * `origin` - The origin of the call, which must be a GC voting majority.
       * * `id` - The proposal ID of the PIP to be approved.
       * 
       * # Errors
       * * `BadOrigin` - If the call is not made by a GC voting majority.
       * * `NoSuchProposal` - If the PIP with the given `id` does not exist.
       * * `IncorrectProposalState` - If the proposal is not in a pending state.
       * * `NotByCommittee` - If the proposal was not made by a committee.
       * 
       * # Notes
       * This function schedules the PIP for execution if all checks pass.
       **/
      approveCommitteeProposal: AugmentedSubmittable<(id: u32 | AnyNumber | Uint8Array) => SubmittableExtrinsic<ApiType>, [u32]>;
      /**
       * Clears the snapshot and emits the event `SnapshotCleared`.
       * 
       * This function can only be called by a Governance Committee (GC) member.
       * 
       * # Arguments
       * * `origin` - The origin of the call, which must be a GC member.
       * 
       * # Events
       * * `SnapshotCleared` - Emitted when the snapshot is successfully cleared, containing the ID of the cleared snapshot.
       * 
       * # Errors
       * * `NotACommitteeMember` - If the call is not made by a GC member.
       **/
      clearSnapshot: AugmentedSubmittable<() => SubmittableExtrinsic<ApiType>, []>;
      /**
       * Enacts the results for the PIPs in the snapshot queue.
       * 
       * The snapshot will be available for further enactments until it is cleared.
       * 
       * The `results` parameter is a list of `(id, result)` tuples where `result` is applied to the PIP with the given `id`.
       * Note that the snapshot priority queue is encoded with the *lowest priority first*.
       * For example, `results = [(id, Approve)]` will approve `SnapshotQueue[SnapshotQueue.len() - 1]`.
       * 
       * # Arguments
       * * `origin` - The origin of the call, which must be a GC voting majority.
       * * `results` - A vector of tuples where each tuple contains a PIP ID and a `SnapshotResult` (either `Approve`, `Reject`, or `Skip`).
       * 
       * # Events
       * * `SnapshotResultsEnacted` - Emitted when the snapshot results are successfully enacted, containing the ID of the snapshot and the actions taken.
       * 
       * # Errors
       * * `BadOrigin` - If the call is not made by a GC voting majority.
       * * `CannotSkipPip` - If a given PIP has already been skipped too many times.
       * * `SnapshotResultTooLarge` - If the length of `results` is greater than the length of the snapshot queue.
       * * `SnapshotIdMismatch` - If there is a mismatch between the PIP IDs in `results` and the snapshot queue.
       * 
       * # Notes
       * This function will:
       * - Update the skip counts for PIPs that are skipped.
       * - Reject PIPs that are marked for rejection and refund any bonded funds.
       * - Approve PIPs that are marked for approval and schedule them for execution.
       **/
      enactSnapshotResults: AugmentedSubmittable<(results: Vec<ITuple<[u32, PalletPipsSnapshotResult]>> | ([u32 | AnyNumber | Uint8Array, PalletPipsSnapshotResult | 'Approve' | 'Reject' | 'Skip' | number | Uint8Array])[]) => SubmittableExtrinsic<ApiType>, [Vec<ITuple<[u32, PalletPipsSnapshotResult]>>]>;
      /**
       * Executes a scheduled PIP (Polymesh Improvement Proposal).
       * 
       * # Arguments
       * * `origin` - The origin of the call, which must be the root.
       * * `id` - The unique identifier of the PIP to be executed.
       * 
       * # Errors
       * * `BadOrigin` - If the call is not made by the root origin.
       * 
       * # Notes
       * This function will:
       * - Remove the PIP from the scheduling queue.
       * - Execute the proposal associated with the PIP.
       **/
      executeScheduledPip: AugmentedSubmittable<(id: u32 | AnyNumber | Uint8Array) => SubmittableExtrinsic<ApiType>, [u32]>;
      /**
       * Expires a scheduled PIP (Polymesh Improvement Proposal).
       * 
       * # Arguments
       * * `origin` - The origin of the call, which must be the root.
       * * `did` - The identity ID of the entity initiating the expiration.
       * * `id` - The unique identifier of the PIP to be expired.
       * 
       * # Errors
       * * `BadOrigin` - If the call is not made by the root origin.
       * 
       * # Notes
       * This function will:
       * - Check if the PIP is in a pending state.
       * - Unsnapshot the PIP if it was part of a snapshot.
       * - Prune the PIP data if it is in an expired state.
       **/
      expireScheduledPip: AugmentedSubmittable<(did: PolymeshPrimitivesIdentityId | string | Uint8Array, id: u32 | AnyNumber | Uint8Array) => SubmittableExtrinsic<ApiType>, [PolymeshPrimitivesIdentityId, u32]>;
      /**
       * Proposes a new PIP by submitting a dispatchable which changes the network.
       * 
       * # Arguments
       * * `origin` - The origin of the call.
       * * `proposal` - The dispatchable call.
       * * `deposit` - The deposit amount for the proposal.
       * * `url` - A link to a website for proposal discussion.
       * * `description` - A short description of the proposal.
       * 
       * # Events
       * * `ProposalCreated`.
       * 
       * # Errors
       * * `IncorrectDeposit` - If the deposit amount is less than the required minimum.
       * * `TooManyActivePips` - If the number of active PIPs exceeds the maximum.
       **/
      propose: AugmentedSubmittable<(proposal: Call | IMethod | string | Uint8Array, deposit: u128 | AnyNumber | Uint8Array, url: Option<Bytes> | null | Uint8Array | Bytes | string, description: Option<Bytes> | null | Uint8Array | Bytes | string) => SubmittableExtrinsic<ApiType>, [Call, u128, Option<Bytes>, Option<Bytes>]>;
      /**
       * Prunes the PIP given by the `id`. The PIP must not be active.
       * 
       * This function is intended for storage garbage collection purposes and can only be called by a Governance Committee (GC) voting majority.
       * 
       * # Arguments
       * * `origin` - The origin of the call, which must be a GC voting majority.
       * * `id` - The proposal ID of the PIP to be pruned.
       * 
       * # Errors
       * * `BadOrigin` - If the call is not made by a GC voting majority.
       * * `NoSuchProposal` - If the PIP with the given `id` does not exist.
       * * `IncorrectProposalState` - If the proposal is active.
       * 
       * # Notes
       * This function will remove the PIP from storage and refund any remaining bonded funds.
       **/
      pruneProposal: AugmentedSubmittable<(id: u32 | AnyNumber | Uint8Array) => SubmittableExtrinsic<ApiType>, [u32]>;
      /**
       * Rejects the PIP given by the `id`. Bonded funds will be refunded, assuming it hasn't
       * been cancelled or executed.
       * 
       * This function can only be called by a Governance Committee (GC) voting majority.
       * 
       * # Arguments
       * * `origin` - The origin of the call, which must be a GC voting majority.
       * * `id` - The proposal ID of the PIP to be rejected.
       * 
       * # Errors
       * * `BadOrigin` - If the call is not made by a GC voting majority.
       * * `NoSuchProposal` - If the PIP with the given `id` does not exist.
       * * `IncorrectProposalState` - If the proposal was cancelled or executed.
       * 
       * # Notes
       * This function will unschedule the PIP if it was scheduled for execution and will
       * unsnapshot the PIP if it was part of a snapshot. It will also handle the rejection
       * of the proposal and refund any bonded funds.
       **/
      rejectProposal: AugmentedSubmittable<(id: u32 | AnyNumber | Uint8Array) => SubmittableExtrinsic<ApiType>, [u32]>;
      /**
       * Updates the execution schedule of the PIP given by `id`.
       * This function can only be called by the release coordinator.
       * 
       * # Arguments
       * * `origin` - The origin of the call, which must be the release coordinator.
       * * `id` - The proposal ID of the PIP to be rescheduled.
       * * `until` - An optional future block number where the enactment period will finish.
       * If `None`, the enactment period will finish in the next block.
       * 
       * # Errors
       * * `RescheduleNotByReleaseCoordinator` - If the call is not made by the release coordinator.
       * * `IncorrectProposalState` - If the proposal is not in a scheduled state.
       * * `InvalidFutureBlockNumber` - If the provided block number is not a valid future block number.
       **/
      rescheduleExecution: AugmentedSubmittable<(id: u32 | AnyNumber | Uint8Array, until: Option<u32> | null | Uint8Array | u32 | AnyNumber) => SubmittableExtrinsic<ApiType>, [u32, Option<u32>]>;
      /**
       * Sets the limit on the number of active PIPs. This function can only be called by the root origin.
       * 
       * # Arguments
       * * `origin` - The origin of the call, which must be the root.
       * * `limit` - The new limit on the number of active PIPs.
       * 
       * # Events
       * * `ActivePipLimitChanged` - Emitted when the active PIP limit is changed, containing the old and new values.
       * 
       * # Errors
       * * `BadOrigin` - If the call is not made by the root origin.
       **/
      setActivePipLimit: AugmentedSubmittable<(limit: u32 | AnyNumber | Uint8Array) => SubmittableExtrinsic<ApiType>, [u32]>;
      /**
       * Changes the default enactment period. This function can only be called by the root origin.
       * 
       * # Arguments
       * * `origin` - The origin of the call, which must be the root.
       * * `period` - The new default enactment period.
       * 
       * # Events
       * * `DefaultEnactmentPeriodChanged` - Emitted when the default enactment period is changed, containing the old and new values.
       * 
       * # Errors
       * * `BadOrigin` - If the call is not made by the root origin.
       **/
      setDefaultEnactmentPeriod: AugmentedSubmittable<(duration: u32 | AnyNumber | Uint8Array) => SubmittableExtrinsic<ApiType>, [u32]>;
      /**
       * Sets the maximum number of times a PIP can be skipped. This function can only be called by the root origin.
       * 
       * # Arguments
       * * `origin` - The origin of the call, which must be the root.
       * * `max` - The new maximum skip count for PIPs.
       * 
       * # Events
       * * `MaxPipSkipCountChanged` - Emitted when the maximum PIP skip count is changed, containing the old and new values.
       * 
       * # Errors
       * * `BadOrigin` - If the call is not made by the root origin.
       **/
      setMaxPipSkipCount: AugmentedSubmittable<(max: u8 | AnyNumber | Uint8Array) => SubmittableExtrinsic<ApiType>, [u8]>;
      /**
       * Changes the minimum proposal deposit amount required to start a proposal. This function can only be called by the root origin.
       * 
       * # Arguments
       * * `origin` - The origin of the call, which must be the root.
       * * `deposit` - The new minimum deposit required to start a proposal.
       * 
       * # Events
       * * `MinimumProposalDepositChanged` - Emitted when the minimum proposal deposit is changed, containing the old and new values.
       * 
       * # Errors
       * * `BadOrigin` - If the call is not made by the root origin.
       **/
      setMinProposalDeposit: AugmentedSubmittable<(deposit: u128 | AnyNumber | Uint8Array) => SubmittableExtrinsic<ApiType>, [u128]>;
      /**
       * Sets the expiry duration (in blocks) for pending PIPs. This function can only be called by the root origin.
       * 
       * # Arguments
       * * `origin` - The origin of the call, which must be the root.
       * * `expiry` - The new expiry duration for pending PIPs. If `None`, PIPs never expire.
       * 
       * # Events
       * * `PendingPipExpiryChanged` - Emitted when the pending PIP expiry duration is changed, containing the old and new values.
       * 
       * # Errors
       * * `BadOrigin` - If the call is not made by the root origin.
       **/
      setPendingPipExpiry: AugmentedSubmittable<(expiry: PolymeshPrimitivesMaybeBlock | { Some: any } | { None: any } | string | Uint8Array) => SubmittableExtrinsic<ApiType>, [PolymeshPrimitivesMaybeBlock]>;
      /**
       * Sets the pruning setting for historical PIPs. This function can only be called by the root origin.
       * 
       * # Arguments
       * * `origin` - The origin of the call, which must be the root.
       * * `prune` - A boolean flag indicating whether completed PIPs should be pruned (`true`) or retained (`false`).
       * 
       * # Events
       * * `HistoricalPipsPruned` - Emitted when the pruning setting is changed, containing the old and new values.
       * 
       * # Errors
       * * `BadOrigin` - If the call is not made by the root origin.
       **/
      setPruneHistoricalPips: AugmentedSubmittable<(prune: bool | boolean | Uint8Array) => SubmittableExtrinsic<ApiType>, [bool]>;
      /**
       * Takes a new snapshot of the current list of active and pending PIPs.
       * The PIPs are then sorted into a priority queue based on each PIP's weight.
       * 
       * This function can only be called by a Governance Committee (GC) member.
       * 
       * # Arguments
       * * `origin` - The origin of the call, which must be a GC member.
       * 
       * # Events
       * * `SnapshotTaken` - Emitted when a snapshot is successfully taken, containing the ID of the snapshot and the queue of PIPs.
       * 
       * # Errors
       * * `NotACommitteeMember` - If the call is not made by a GC member.
       **/
      snapshot: AugmentedSubmittable<() => SubmittableExtrinsic<ApiType>, []>;
      /**
       * Casts a vote either in favor or against a PIP with `id`.
       * The "conviction" or strength of the vote is given by `deposit`, which is reserved.
       * 
       * Note that `vote` is *not* additive.
       * That is, `vote(id, true, 50)` followed by `vote(id, true, 40)`
       * will first reserve `50` and then refund `50 - 10`, ending up with `40` in deposit.
       * To add atop of existing votes, you'll need `existing_deposit + addition`.
       * 
       * # Arguments
       * * `origin` - The origin of the call.
       * * `id` - The proposal ID to vote on.
       * * `aye_or_nay` - A boolean representing a vote in favor (`true`) or against (`false`).
       * * `deposit` - The "conviction" or strength of the vote, represented by the amount of deposit.
       * 
       * # Events
       * * `Voted` - Emitted when a vote is successfully cast.
       * 
       * # Errors
       * * `NoSuchProposal` - If the `id` does not reference a valid PIP.
       * * `NotFromCommunity` - If the proposal was made by a committee.
       * * `IncorrectProposalState` - If the PIP is not in a pending state.
       * * `InsufficientDeposit` - If the `origin` cannot reserve the required deposit.
       * * `IncorrectDeposit` - If the deposit amount is less than the required minimum.
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
      setExpiresAfter: AugmentedSubmittable<(expiry: PolymeshPrimitivesMaybeBlock | { Some: any } | { None: any } | string | Uint8Array) => SubmittableExtrinsic<ApiType>, [PolymeshPrimitivesMaybeBlock]>;
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
      voteOrPropose: AugmentedSubmittable<(approve: bool | boolean | Uint8Array, call: Call | IMethod | string | Uint8Array) => SubmittableExtrinsic<ApiType>, [bool, Call]>;
      /**
       * Generic tx
       **/
      [key: string]: SubmittableExtrinsicFunction<ApiType>;
    };
    polymeshContracts: {
      /**
       * Instantiates a smart contract defining it with the given `code` and `salt`.
       * 
       * The contract will be attached as a primary key of a newly created child identity of the caller.
       * 
       * # Arguments
       * - `endowment`: Amount of POLYX to transfer to the contract.
       * - `gas_limit`: For how much gas the `deploy` code in the contract may at most consume.
       * - `storage_deposit_limit`: The maximum amount of balance that can be charged/reserved from the caller to pay for the storage consumed.
       * - `code`: The WASM binary defining the smart contract.
       * - `data`: The input data to pass to the contract constructor.
       * - `salt`: Used for contract address derivation. By varying this, the same `code` can be used under the same identity.
       * 
       **/
      instantiateWithCodeAsPrimaryKey: AugmentedSubmittable<(endowment: u128 | AnyNumber | Uint8Array, gasLimit: SpWeightsWeightV2Weight | { refTime?: any; proofSize?: any } | string | Uint8Array, storageDepositLimit: Option<u128> | null | Uint8Array | u128 | AnyNumber, code: Bytes | string | Uint8Array, data: Bytes | string | Uint8Array, salt: Bytes | string | Uint8Array) => SubmittableExtrinsic<ApiType>, [u128, SpWeightsWeightV2Weight, Option<u128>, Bytes, Bytes, Bytes]>;
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
       * - `storage_deposit_limit` The maximum amount of balance that can be charged/reserved
       * from the caller to pay for the storage consumed.
       * - `code` with the WASM binary defining the smart contract.
       * - `data` The input data to pass to the contract constructor.
       * - `salt` used for contract address derivation.
       * By varying this, the same `code` can be used under the same identity.
       * - `perms` that the new secondary key will have.
       * 
       * # Errors
       * - All the errors in `pallet_contracts::Call::instantiate_with_code` can also happen here.
       * - CDD/Permissions are checked, unlike in `pallet_contracts`.
       * - Errors that arise when adding a new secondary key can also occur here.
       **/
      instantiateWithCodePerms: AugmentedSubmittable<(endowment: u128 | AnyNumber | Uint8Array, gasLimit: SpWeightsWeightV2Weight | { refTime?: any; proofSize?: any } | string | Uint8Array, storageDepositLimit: Option<u128> | null | Uint8Array | u128 | AnyNumber, code: Bytes | string | Uint8Array, data: Bytes | string | Uint8Array, salt: Bytes | string | Uint8Array, perms: PolymeshPrimitivesSecondaryKeyPermissions | { asset?: any; extrinsic?: any; portfolio?: any } | string | Uint8Array) => SubmittableExtrinsic<ApiType>, [u128, SpWeightsWeightV2Weight, Option<u128>, Bytes, Bytes, Bytes, PolymeshPrimitivesSecondaryKeyPermissions]>;
      /**
       * Instantiates a smart contract defining using the given `code_hash` and `salt`.
       * 
       * Unlike `instantiate_with_code`, this assumes that at least one contract with the same WASM code has already been uploaded.
       * 
       * The contract will be attached as a primary key of a newly created child identity of the caller.
       * 
       * # Arguments
       * - `endowment`: amount of POLYX to transfer to the contract.
       * - `gas_limit`: for how much gas the `deploy` code in the contract may at most consume.
       * - `storage_deposit_limit`: The maximum amount of balance that can be charged/reserved from the caller to pay for the storage consumed.
       * - `code_hash`: of an already uploaded WASM binary.
       * - `data`: The input data to pass to the contract constructor.
       * - `salt`: used for contract address derivation. By varying this, the same `code` can be used under the same identity.
       * 
       **/
      instantiateWithHashAsPrimaryKey: AugmentedSubmittable<(endowment: u128 | AnyNumber | Uint8Array, gasLimit: SpWeightsWeightV2Weight | { refTime?: any; proofSize?: any } | string | Uint8Array, storageDepositLimit: Option<u128> | null | Uint8Array | u128 | AnyNumber, codeHash: H256 | string | Uint8Array, data: Bytes | string | Uint8Array, salt: Bytes | string | Uint8Array) => SubmittableExtrinsic<ApiType>, [u128, SpWeightsWeightV2Weight, Option<u128>, H256, Bytes, Bytes]>;
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
       * - `storage_deposit_limit` The maximum amount of balance that can be charged/reserved
       * from the caller to pay for the storage consumed.
       * - `code_hash` of an already uploaded WASM binary.
       * - `data` The input data to pass to the contract constructor.
       * - `salt` used for contract address derivation.
       * By varying this, the same `code` can be used under the same identity.
       * - `perms` that the new secondary key will have.
       * 
       * # Errors
       * - All the errors in `pallet_contracts::Call::instantiate` can also happen here.
       * - CDD/Permissions are checked, unlike in `pallet_contracts`.
       * - Errors that arise when adding a new secondary key can also occur here.
       **/
      instantiateWithHashPerms: AugmentedSubmittable<(endowment: u128 | AnyNumber | Uint8Array, gasLimit: SpWeightsWeightV2Weight | { refTime?: any; proofSize?: any } | string | Uint8Array, storageDepositLimit: Option<u128> | null | Uint8Array | u128 | AnyNumber, codeHash: H256 | string | Uint8Array, data: Bytes | string | Uint8Array, salt: Bytes | string | Uint8Array, perms: PolymeshPrimitivesSecondaryKeyPermissions | { asset?: any; extrinsic?: any; portfolio?: any } | string | Uint8Array) => SubmittableExtrinsic<ApiType>, [u128, SpWeightsWeightV2Weight, Option<u128>, H256, Bytes, Bytes, PolymeshPrimitivesSecondaryKeyPermissions]>;
      /**
       * Update CallRuntime whitelist.
       * 
       * # Arguments
       * 
       * # Errors
       **/
      updateCallRuntimeWhitelist: AugmentedSubmittable<(updates: Vec<ITuple<[PolymeshContractsChainExtensionExtrinsicId, bool]>> | ([PolymeshContractsChainExtensionExtrinsicId, bool | boolean | Uint8Array])[]) => SubmittableExtrinsic<ApiType>, [Vec<ITuple<[PolymeshContractsChainExtensionExtrinsicId, bool]>>]>;
      upgradeApi: AugmentedSubmittable<(api: PolymeshContractsApi | { desc?: any; major?: any } | string | Uint8Array, nextUpgrade: PolymeshContractsNextUpgrade | { chainVersion?: any; apiHash?: any } | string | Uint8Array) => SubmittableExtrinsic<ApiType>, [PolymeshContractsApi, PolymeshContractsNextUpgrade]>;
      /**
       * Generic tx
       **/
      [key: string]: SubmittableExtrinsicFunction<ApiType>;
    };
    polymeshTransactionPayment: {
      setDisableFees: AugmentedSubmittable<(value: bool | boolean | Uint8Array) => SubmittableExtrinsic<ApiType>, [bool]>;
      /**
       * Generic tx
       **/
      [key: string]: SubmittableExtrinsicFunction<ApiType>;
    };
    portfolio: {
      acceptPortfolioCustody: AugmentedSubmittable<(authId: u64 | AnyNumber | Uint8Array) => SubmittableExtrinsic<ApiType>, [u64]>;
      /**
       * Adds an identity that will be allowed to create and take custody of a portfolio under the caller's identity.
       * 
       * # Arguments
       * * `trusted_identity` - the [`IdentityId`] that will be allowed to call `create_custody_portfolio`.
       * 
       **/
      allowIdentityToCreatePortfolios: AugmentedSubmittable<(trustedIdentity: PolymeshPrimitivesIdentityId | string | Uint8Array) => SubmittableExtrinsic<ApiType>, [PolymeshPrimitivesIdentityId]>;
      /**
       * Creates a portfolio under the `portfolio_owner_id` identity and transfers its custody to the caller's identity.
       * 
       * # Arguments
       * * `portfolio_owner_id` - the [`IdentityId`] that will own the new portfolio.
       * * `portfolio_name` - the [`PortfolioName`] of the new portfolio.
       * 
       **/
      createCustodyPortfolio: AugmentedSubmittable<(portfolioOwnerId: PolymeshPrimitivesIdentityId | string | Uint8Array, portfolioName: Bytes | string | Uint8Array) => SubmittableExtrinsic<ApiType>, [PolymeshPrimitivesIdentityId, Bytes]>;
      /**
       * Creates a portfolio with the given `name`.
       **/
      createPortfolio: AugmentedSubmittable<(name: Bytes | string | Uint8Array) => SubmittableExtrinsic<ApiType>, [Bytes]>;
      /**
       * Deletes a user portfolio. A portfolio can be deleted only if it has no funds.
       * 
       * # Errors
       * * `PortfolioDoesNotExist` if `portfolio_number` doesn't reference a valid portfolio.
       * * `PortfolioNotEmpty` if the portfolio still holds any asset
       * 
       * # Permissions
       * * Portfolio
       **/
      deletePortfolio: AugmentedSubmittable<(portfolioNumber: u64 | AnyNumber | Uint8Array) => SubmittableExtrinsic<ApiType>, [u64]>;
      /**
       * Moves fungigle an non-fungible tokens from one portfolio of an identity to another portfolio of the same
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
       * * `NoDuplicateAssetsAllowed` the same asset can't be repeated in the items vector.
       * * `InvalidTransferNFTNotOwned` if the caller is trying to move an NFT he doesn't own.
       * * `InvalidTransferNFTIsLocked` if the caller is trying to move a locked NFT.
       * 
       * # Permissions
       * * Portfolio
       **/
      movePortfolioFunds: AugmentedSubmittable<(from: PolymeshPrimitivesIdentityIdPortfolioId | { did?: any; kind?: any } | string | Uint8Array, to: PolymeshPrimitivesIdentityIdPortfolioId | { did?: any; kind?: any } | string | Uint8Array, funds: Vec<PolymeshPrimitivesPortfolioFund> | (PolymeshPrimitivesPortfolioFund | { description?: any; memo?: any } | string | Uint8Array)[]) => SubmittableExtrinsic<ApiType>, [PolymeshPrimitivesIdentityIdPortfolioId, PolymeshPrimitivesIdentityIdPortfolioId, Vec<PolymeshPrimitivesPortfolioFund>]>;
      /**
       * Pre-approves the receivement of an asset to a portfolio.
       * 
       * # Arguments
       * * `origin` - the secondary key of the sender.
       * * `asset_id` - the [`AssetId`] that will be exempt from affirmation.
       * * `portfolio_id` - the [`PortfolioId`] that can receive `asset_id` without affirmation.
       * 
       * # Permissions
       * * Portfolio
       **/
      preApprovePortfolio: AugmentedSubmittable<(assetId: PolymeshPrimitivesAssetAssetId | string | Uint8Array, portfolioId: PolymeshPrimitivesIdentityIdPortfolioId | { did?: any; kind?: any } | string | Uint8Array) => SubmittableExtrinsic<ApiType>, [PolymeshPrimitivesAssetAssetId, PolymeshPrimitivesIdentityIdPortfolioId]>;
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
       * Removes the pre approval of an asset to a portfolio.
       * 
       * # Arguments
       * * `origin` - the secondary key of the sender.
       * * `asset_id` - the [`AssetId`] that will be exempt from affirmation.
       * * `portfolio_id` - the [`PortfolioId`] that can receive `asset_id` without affirmation.
       * 
       * # Permissions
       * * Portfolio
       **/
      removePortfolioPreApproval: AugmentedSubmittable<(assetId: PolymeshPrimitivesAssetAssetId | string | Uint8Array, portfolioId: PolymeshPrimitivesIdentityIdPortfolioId | { did?: any; kind?: any } | string | Uint8Array) => SubmittableExtrinsic<ApiType>, [PolymeshPrimitivesAssetAssetId, PolymeshPrimitivesIdentityIdPortfolioId]>;
      /**
       * Renames a non-default portfolio.
       * 
       * # Errors
       * * `PortfolioDoesNotExist` if `portfolio_number` doesn't reference a valid portfolio.
       * 
       * # Permissions
       * * Portfolio
       **/
      renamePortfolio: AugmentedSubmittable<(portfolioNumber: u64 | AnyNumber | Uint8Array, newPortfolioName: Bytes | string | Uint8Array) => SubmittableExtrinsic<ApiType>, [u64, Bytes]>;
      /**
       * Removes permission of an identity to create and take custody of a portfolio under the caller's identity.
       * 
       * # Arguments
       * * `identity` - the [`IdentityId`] that will have the permissions to call `create_custody_portfolio` revoked.
       * 
       **/
      revokeCreatePortfoliosPermission: AugmentedSubmittable<(identity: PolymeshPrimitivesIdentityId | string | Uint8Array) => SubmittableExtrinsic<ApiType>, [PolymeshPrimitivesIdentityId]>;
      /**
       * Generic tx
       **/
      [key: string]: SubmittableExtrinsicFunction<ApiType>;
    };
    preimage: {
      /**
       * Ensure that the bulk of pre-images is upgraded.
       * 
       * The caller pays no fee if at least 90% of pre-images were successfully updated.
       **/
      ensureUpdated: AugmentedSubmittable<(hashes: Vec<H256> | (H256 | string | Uint8Array)[]) => SubmittableExtrinsic<ApiType>, [Vec<H256>]>;
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
       * 
       * If `len` is provided, then it will be a much cheaper operation.
       * 
       * - `hash`: The hash of the preimage to be removed from the store.
       * - `len`: The length of the preimage of `hash`.
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
      changeBaseFee: AugmentedSubmittable<(op: PolymeshPrimitivesProtocolFeeProtocolOp | 'AssetRegisterTicker' | 'AssetIssue' | 'AssetAddDocuments' | 'AssetCreateAsset' | 'CheckpointCreateSchedule' | 'ComplianceManagerAddComplianceRequirement' | 'IdentityCddRegisterDid' | 'IdentityAddClaim' | 'IdentityAddSecondaryKeysWithAuthorization' | 'PipsPropose' | 'ContractsPutCode' | 'CorporateBallotAttachBallot' | 'CapitalDistributionDistribute' | 'NFTCreateCollection' | 'NFTMint' | 'IdentityCreateChildIdentity' | number | Uint8Array, baseFee: u128 | AnyNumber | Uint8Array) => SubmittableExtrinsic<ApiType>, [PolymeshPrimitivesProtocolFeeProtocolOp, u128]>;
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
       * Accepts a subsidy from a `paying_key`.
       * 
       * # Arguments
       * - `paying_key` the paying key that is subsidising the caller's `user_key`.
       **/
      acceptSubsidy: AugmentedSubmittable<(payingKey: AccountId32 | string | Uint8Array) => SubmittableExtrinsic<ApiType>, [AccountId32]>;
      /**
       * Approve a subsidy for a `user_key`.
       * 
       * # Arguments
       * - `user_key` the user key to subsidise.
       * - `polyx_limit` the initial POLYX limit for this subsidy.
       * 
       **/
      approveSubsidy: AugmentedSubmittable<(userKey: AccountId32 | string | Uint8Array, polyxLimit: u128 | AnyNumber | Uint8Array) => SubmittableExtrinsic<ApiType>, [AccountId32, u128]>;
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
       * - `Overlow` if the subsidy's remaining POLYX would have overflowed `u128::MAX`.
       **/
      increasePolyxLimit: AugmentedSubmittable<(userKey: AccountId32 | string | Uint8Array, amount: u128 | AnyNumber | Uint8Array) => SubmittableExtrinsic<ApiType>, [AccountId32, u128]>;
      /**
       * Relay a call for a target from an origin
       * 
       * Relaying in this context refers to the ability of origin to make a call on behalf of
       * target.
       * 
       * Transaction and protocol fees are charged to origin.
       * 
       * # Parameters
       * - `target`: Account to be relayed
       * - `signature`: Signature from target authorizing the relay
       * - `call`: Call to be relayed on behalf of target
       **/
      relayTx: AugmentedSubmittable<(target: AccountId32 | string | Uint8Array, signature: SpRuntimeMultiSignature | { Ed25519: any } | { Sr25519: any } | { Ecdsa: any } | { Eth: any } | string | Uint8Array, call: Call | IMethod | string | Uint8Array, expiresAt: u64 | AnyNumber | Uint8Array) => SubmittableExtrinsic<ApiType>, [AccountId32, SpRuntimeMultiSignature, Call, u64]>;
      /**
       * Removes the `paying_key` from a `user_key`.
       * 
       * This can only be called by either the `user_key` or the `paying_key`.
       * 
       * # Arguments
       * - `user_key` the user key to remove the subsidy from.
       * - `paying_key` the paying key that was subsidising the `user_key`.
       * 
       * # Errors
       * - `NoPayingKey` if the `user_key` doesn't have a `paying_key`.
       * - `NotPayingKey` if the `paying_key` doesn't match the current `paying_key`.
       **/
      removeSubsidy: AugmentedSubmittable<(userKey: AccountId32 | string | Uint8Array, payingKey: AccountId32 | string | Uint8Array) => SubmittableExtrinsic<ApiType>, [AccountId32, AccountId32]>;
      /**
       * Revoke a previously approved subsidy.
       * 
       * # Arguments
       * - `user_key` the user key to revoke the subsidy for.
       **/
      revokeSubsidy: AugmentedSubmittable<(userKey: AccountId32 | string | Uint8Array) => SubmittableExtrinsic<ApiType>, [AccountId32]>;
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
       **/
      updatePolyxLimit: AugmentedSubmittable<(userKey: AccountId32 | string | Uint8Array, polyxLimit: u128 | AnyNumber | Uint8Array) => SubmittableExtrinsic<ApiType>, [AccountId32, u128]>;
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
      cancelNamed: AugmentedSubmittable<(id: U8aFixed | string | Uint8Array) => SubmittableExtrinsic<ApiType>, [U8aFixed]>;
      /**
       * Removes the retry configuration of a task.
       **/
      cancelRetry: AugmentedSubmittable<(task: ITuple<[u32, u32]> | [u32 | AnyNumber | Uint8Array, u32 | AnyNumber | Uint8Array]) => SubmittableExtrinsic<ApiType>, [ITuple<[u32, u32]>]>;
      /**
       * Cancel the retry configuration of a named task.
       **/
      cancelRetryNamed: AugmentedSubmittable<(id: U8aFixed | string | Uint8Array) => SubmittableExtrinsic<ApiType>, [U8aFixed]>;
      /**
       * Anonymously schedule a task.
       **/
      schedule: AugmentedSubmittable<(when: u32 | AnyNumber | Uint8Array, maybePeriodic: Option<ITuple<[u32, u32]>> | null | Uint8Array | ITuple<[u32, u32]> | [u32 | AnyNumber | Uint8Array, u32 | AnyNumber | Uint8Array], priority: u8 | AnyNumber | Uint8Array, call: Call | IMethod | string | Uint8Array) => SubmittableExtrinsic<ApiType>, [u32, Option<ITuple<[u32, u32]>>, u8, Call]>;
      /**
       * Anonymously schedule a task after a delay.
       **/
      scheduleAfter: AugmentedSubmittable<(after: u32 | AnyNumber | Uint8Array, maybePeriodic: Option<ITuple<[u32, u32]>> | null | Uint8Array | ITuple<[u32, u32]> | [u32 | AnyNumber | Uint8Array, u32 | AnyNumber | Uint8Array], priority: u8 | AnyNumber | Uint8Array, call: Call | IMethod | string | Uint8Array) => SubmittableExtrinsic<ApiType>, [u32, Option<ITuple<[u32, u32]>>, u8, Call]>;
      /**
       * Schedule a named task.
       **/
      scheduleNamed: AugmentedSubmittable<(id: U8aFixed | string | Uint8Array, when: u32 | AnyNumber | Uint8Array, maybePeriodic: Option<ITuple<[u32, u32]>> | null | Uint8Array | ITuple<[u32, u32]> | [u32 | AnyNumber | Uint8Array, u32 | AnyNumber | Uint8Array], priority: u8 | AnyNumber | Uint8Array, call: Call | IMethod | string | Uint8Array) => SubmittableExtrinsic<ApiType>, [U8aFixed, u32, Option<ITuple<[u32, u32]>>, u8, Call]>;
      /**
       * Schedule a named task after a delay.
       **/
      scheduleNamedAfter: AugmentedSubmittable<(id: U8aFixed | string | Uint8Array, after: u32 | AnyNumber | Uint8Array, maybePeriodic: Option<ITuple<[u32, u32]>> | null | Uint8Array | ITuple<[u32, u32]> | [u32 | AnyNumber | Uint8Array, u32 | AnyNumber | Uint8Array], priority: u8 | AnyNumber | Uint8Array, call: Call | IMethod | string | Uint8Array) => SubmittableExtrinsic<ApiType>, [U8aFixed, u32, Option<ITuple<[u32, u32]>>, u8, Call]>;
      /**
       * Set a retry configuration for a task so that, in case its scheduled run fails, it will
       * be retried after `period` blocks, for a total amount of `retries` retries or until it
       * succeeds.
       * 
       * Tasks which need to be scheduled for a retry are still subject to weight metering and
       * agenda space, same as a regular task. If a periodic task fails, it will be scheduled
       * normally while the task is retrying.
       * 
       * Tasks scheduled as a result of a retry for a periodic task are unnamed, non-periodic
       * clones of the original task. Their retry configuration will be derived from the
       * original task's configuration, but will have a lower value for `remaining` than the
       * original `total_retries`.
       **/
      setRetry: AugmentedSubmittable<(task: ITuple<[u32, u32]> | [u32 | AnyNumber | Uint8Array, u32 | AnyNumber | Uint8Array], retries: u8 | AnyNumber | Uint8Array, period: u32 | AnyNumber | Uint8Array) => SubmittableExtrinsic<ApiType>, [ITuple<[u32, u32]>, u8, u32]>;
      /**
       * Set a retry configuration for a named task so that, in case its scheduled run fails, it
       * will be retried after `period` blocks, for a total amount of `retries` retries or until
       * it succeeds.
       * 
       * Tasks which need to be scheduled for a retry are still subject to weight metering and
       * agenda space, same as a regular task. If a periodic task fails, it will be scheduled
       * normally while the task is retrying.
       * 
       * Tasks scheduled as a result of a retry for a periodic task are unnamed, non-periodic
       * clones of the original task. Their retry configuration will be derived from the
       * original task's configuration, but will have a lower value for `remaining` than the
       * original `total_retries`.
       **/
      setRetryNamed: AugmentedSubmittable<(id: U8aFixed | string | Uint8Array, retries: u8 | AnyNumber | Uint8Array, period: u32 | AnyNumber | Uint8Array) => SubmittableExtrinsic<ApiType>, [U8aFixed, u8, u32]>;
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
       * ## Complexity
       * - `O(1)` in number of key types. Actual cost depends on the number of length of
       * `T::Keys::key_ids()` which is fixed.
       **/
      purgeKeys: AugmentedSubmittable<() => SubmittableExtrinsic<ApiType>, []>;
      /**
       * Sets the session key(s) of the function caller to `keys`.
       * Allows an account to set its session key prior to becoming a validator.
       * This doesn't take effect until the next session.
       * 
       * The dispatch origin of this function must be signed.
       * 
       * ## Complexity
       * - `O(1)`. Actual cost depends on the number of length of `T::Keys::key_ids()` which is
       * fixed.
       **/
      setKeys: AugmentedSubmittable<(keys: PolymeshRuntimeDevelopRuntimeSessionKeys | { grandpa?: any; babe?: any; imOnline?: any; authorityDiscovery?: any; beefy?: any } | string | Uint8Array, proof: Bytes | string | Uint8Array) => SubmittableExtrinsic<ApiType>, [PolymeshRuntimeDevelopRuntimeSessionKeys, Bytes]>;
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
       * * `venue_id`: The [`VenueId`] of the venue this instruction belongs to.
       * * `settlement_type`: The [`SettlementType`] specifying when the instruction should be settled.
       * * `trade_date`: Optional date from which people can interact with this instruction.
       * * `value_date`: Optional date after which the instruction should be settled (not enforced).
       * * `legs`: A vector of all [`Leg`] included in this instruction.
       * * `holder_set`: A set of [`AssetHolder`] under the caller's control and intended for affirmation.
       * * `memo`: An optional [`Memo`] field for this instruction.
       * 
       * # Permissions
       * * Portfolio
       **/
      addAndAffirmInstruction: AugmentedSubmittable<(venueId: Option<u64> | null | Uint8Array | u64 | AnyNumber, settlementType: PolymeshPrimitivesSettlementSettlementType | { SettleOnAffirmation: any } | { SettleOnBlock: any } | { SettleManual: any } | { SettleAfterLock: any } | string | Uint8Array, tradeDate: Option<u64> | null | Uint8Array | u64 | AnyNumber, valueDate: Option<u64> | null | Uint8Array | u64 | AnyNumber, legs: Vec<PolymeshPrimitivesSettlementLeg> | (PolymeshPrimitivesSettlementLeg | { Fungible: any } | { NonFungible: any } | { OffChain: any } | string | Uint8Array)[], holderSet: BTreeSet<PolymeshPrimitivesAssetAssetHolder>, instructionMemo: Option<PolymeshPrimitivesMemo> | null | Uint8Array | PolymeshPrimitivesMemo | string) => SubmittableExtrinsic<ApiType>, [Option<u64>, PolymeshPrimitivesSettlementSettlementType, Option<u64>, Option<u64>, Vec<PolymeshPrimitivesSettlementLeg>, BTreeSet<PolymeshPrimitivesAssetAssetHolder>, Option<PolymeshPrimitivesMemo>]>;
      /**
       * Adds and affirms a new instruction with mediators.
       * 
       * # Arguments
       * * `venue_id`: The [`VenueId`] of the venue this instruction belongs to.
       * * `settlement_type`: The [`SettlementType`] specifying when the instruction should be settled.
       * * `trade_date`: Optional date from which people can interact with this instruction.
       * * `value_date`: Optional date after which the instruction should be settled (not enforced).
       * * `legs`: A vector of all [`Leg`] included in this instruction.
       * * `holder_set`: A set of [`AssetHolder`] under the caller's control and intended for affirmation.
       * * `instruction_memo`: An optional [`Memo`] field for this instruction.
       * * `mediators`: A set of [`IdentityId`] of all the mandatory mediators for the instruction.
       * 
       * # Permissions
       * * Portfolio
       **/
      addAndAffirmWithMediators: AugmentedSubmittable<(venueId: Option<u64> | null | Uint8Array | u64 | AnyNumber, settlementType: PolymeshPrimitivesSettlementSettlementType | { SettleOnAffirmation: any } | { SettleOnBlock: any } | { SettleManual: any } | { SettleAfterLock: any } | string | Uint8Array, tradeDate: Option<u64> | null | Uint8Array | u64 | AnyNumber, valueDate: Option<u64> | null | Uint8Array | u64 | AnyNumber, legs: Vec<PolymeshPrimitivesSettlementLeg> | (PolymeshPrimitivesSettlementLeg | { Fungible: any } | { NonFungible: any } | { OffChain: any } | string | Uint8Array)[], holderSet: BTreeSet<PolymeshPrimitivesAssetAssetHolder>, instructionMemo: Option<PolymeshPrimitivesMemo> | null | Uint8Array | PolymeshPrimitivesMemo | string, mediators: BTreeSet<PolymeshPrimitivesIdentityId>) => SubmittableExtrinsic<ApiType>, [Option<u64>, PolymeshPrimitivesSettlementSettlementType, Option<u64>, Option<u64>, Vec<PolymeshPrimitivesSettlementLeg>, BTreeSet<PolymeshPrimitivesAssetAssetHolder>, Option<PolymeshPrimitivesMemo>, BTreeSet<PolymeshPrimitivesIdentityId>]>;
      /**
       * Adds a new instruction.
       * 
       * # Arguments
       * * `venue_id`: The optional [`VenueId`] of the venue this instruction belongs to.
       * * `settlement_type`: The [`SettlementType`] specifying when the instruction should be settled.
       * * `trade_date`: Optional date from which people can interact with this instruction.
       * * `value_date`: Optional date after which the instruction should be settled (not enforced).
       * * `legs`: A vector of all [`Leg`] included in this instruction.
       * * `memo`: An optional [`Memo`] field for this instruction.
       **/
      addInstruction: AugmentedSubmittable<(venueId: Option<u64> | null | Uint8Array | u64 | AnyNumber, settlementType: PolymeshPrimitivesSettlementSettlementType | { SettleOnAffirmation: any } | { SettleOnBlock: any } | { SettleManual: any } | { SettleAfterLock: any } | string | Uint8Array, tradeDate: Option<u64> | null | Uint8Array | u64 | AnyNumber, valueDate: Option<u64> | null | Uint8Array | u64 | AnyNumber, legs: Vec<PolymeshPrimitivesSettlementLeg> | (PolymeshPrimitivesSettlementLeg | { Fungible: any } | { NonFungible: any } | { OffChain: any } | string | Uint8Array)[], instructionMemo: Option<PolymeshPrimitivesMemo> | null | Uint8Array | PolymeshPrimitivesMemo | string) => SubmittableExtrinsic<ApiType>, [Option<u64>, PolymeshPrimitivesSettlementSettlementType, Option<u64>, Option<u64>, Vec<PolymeshPrimitivesSettlementLeg>, Option<PolymeshPrimitivesMemo>]>;
      /**
       * Adds a new instruction with mediators.
       * 
       * # Arguments
       * * `venue_id`: The [`VenueId`] of the venue this instruction belongs to.
       * * `settlement_type`: The [`SettlementType`] specifying when the instruction should be settled.
       * * `trade_date`: Optional date from which people can interact with this instruction.
       * * `value_date`: Optional date after which the instruction should be settled (not enforced).
       * * `legs`: A vector of all [`Leg`] included in this instruction.
       * * `instruction_memo`: An optional [`Memo`] field for this instruction.
       * * `mediators`: A set of [`IdentityId`] of all the mandatory mediators for the instruction.
       **/
      addInstructionWithMediators: AugmentedSubmittable<(venueId: Option<u64> | null | Uint8Array | u64 | AnyNumber, settlementType: PolymeshPrimitivesSettlementSettlementType | { SettleOnAffirmation: any } | { SettleOnBlock: any } | { SettleManual: any } | { SettleAfterLock: any } | string | Uint8Array, tradeDate: Option<u64> | null | Uint8Array | u64 | AnyNumber, valueDate: Option<u64> | null | Uint8Array | u64 | AnyNumber, legs: Vec<PolymeshPrimitivesSettlementLeg> | (PolymeshPrimitivesSettlementLeg | { Fungible: any } | { NonFungible: any } | { OffChain: any } | string | Uint8Array)[], instructionMemo: Option<PolymeshPrimitivesMemo> | null | Uint8Array | PolymeshPrimitivesMemo | string, mediators: BTreeSet<PolymeshPrimitivesIdentityId>) => SubmittableExtrinsic<ApiType>, [Option<u64>, PolymeshPrimitivesSettlementSettlementType, Option<u64>, Option<u64>, Vec<PolymeshPrimitivesSettlementLeg>, Option<PolymeshPrimitivesMemo>, BTreeSet<PolymeshPrimitivesIdentityId>]>;
      /**
       * Provide affirmation to an existing instruction.
       * 
       * # Arguments
       * * `id` - the [`InstructionId`] of the instruction being affirmed.
       * * `holder_set` - a set of [`AssetHolder`] under the caller's control and intended for affirmation.
       * 
       * # Permissions
       * * Portfolio
       **/
      affirmInstruction: AugmentedSubmittable<(id: u64 | AnyNumber | Uint8Array, holderSet: BTreeSet<PolymeshPrimitivesAssetAssetHolder>) => SubmittableExtrinsic<ApiType>, [u64, BTreeSet<PolymeshPrimitivesAssetAssetHolder>]>;
      /**
       * Affirms the instruction as a mediator - should only be called by mediators, otherwise it will fail.
       * 
       * # Arguments
       * * `origin`: The secondary key of the sender.
       * * `instruction_id`: The [`InstructionId`] that will be affirmed by the mediator.
       * * `expiry`: An Optional value for defining when the affirmation will expire (None means it will always be valid).
       **/
      affirmInstructionAsMediator: AugmentedSubmittable<(instructionId: u64 | AnyNumber | Uint8Array, expiry: Option<u64> | null | Uint8Array | u64 | AnyNumber) => SubmittableExtrinsic<ApiType>, [u64, Option<u64>]>;
      /**
       * Provide affirmation to an existing instruction.
       * 
       * # Arguments
       * * `id` - the [`InstructionId`] of the instruction being affirmed.
       * * `holder_set` - a vector of [`AssetHolder`] under the caller's control and intended for affirmation.
       * * `number_of_assets` - an optional [`AffirmationCount`] that will be used for a precise fee estimation before executing the extrinsic.
       * 
       * Note: calling the rpc method `get_affirmation_count` returns an instance of [`AffirmationCount`].
       * 
       * # Permissions
       * * Portfolio
       **/
      affirmInstructionWithCount: AugmentedSubmittable<(id: u64 | AnyNumber | Uint8Array, holderSet: BTreeSet<PolymeshPrimitivesAssetAssetHolder>, numberOfAssets: Option<PolymeshPrimitivesSettlementAffirmationCount> | null | Uint8Array | PolymeshPrimitivesSettlementAffirmationCount | { senderAssetCount?: any; receiverAssetCount?: any; offchainCount?: any } | string) => SubmittableExtrinsic<ApiType>, [u64, BTreeSet<PolymeshPrimitivesAssetAssetHolder>, Option<PolymeshPrimitivesSettlementAffirmationCount>]>;
      /**
       * Affirms an instruction using receipts for offchain transfers.
       * 
       * # Arguments
       * * `id` - the [`InstructionId`] of the instruction being affirmed.
       * * `receipt_details` - a vector of [`ReceiptDetails`], which contain the details about the offchain transfer.
       * * `holder_set` - a vector of [`AssetHolder`] under the caller's control and intended for affirmation.
       * 
       * # Permissions
       * * Portfolio/Account
       **/
      affirmWithReceipts: AugmentedSubmittable<(id: u64 | AnyNumber | Uint8Array, receiptDetails: Vec<PolymeshPrimitivesSettlementReceiptDetails> | (PolymeshPrimitivesSettlementReceiptDetails | { uid?: any; instructionId?: any; legId?: any; signer?: any; signature?: any; expiresAt?: any; metadata?: any } | string | Uint8Array)[], holderSet: BTreeSet<PolymeshPrimitivesAssetAssetHolder>) => SubmittableExtrinsic<ApiType>, [u64, Vec<PolymeshPrimitivesSettlementReceiptDetails>, BTreeSet<PolymeshPrimitivesAssetAssetHolder>]>;
      /**
       * Affirms an instruction using receipts for offchain transfers.
       * 
       * # Arguments
       * * `id` - the [`InstructionId`] of the instruction being affirmed.
       * * `receipt_details` - a vector of [`ReceiptDetails`], which contain the details about the offchain transfer.
       * * `holder_set` - a vector of [`AssetHolder`] under the caller's control and intended for affirmation.
       * * `number_of_assets` - an optional [`AffirmationCount`] that will be used for a precise fee estimation before executing the extrinsic.
       * 
       * Note: calling the rpc method `get_affirmation_count` returns an instance of [`AffirmationCount`].
       * 
       * # Permissions
       * * Portfolio
       **/
      affirmWithReceiptsWithCount: AugmentedSubmittable<(id: u64 | AnyNumber | Uint8Array, receiptDetails: Vec<PolymeshPrimitivesSettlementReceiptDetails> | (PolymeshPrimitivesSettlementReceiptDetails | { uid?: any; instructionId?: any; legId?: any; signer?: any; signature?: any; expiresAt?: any; metadata?: any } | string | Uint8Array)[], holderSet: BTreeSet<PolymeshPrimitivesAssetAssetHolder>, numberOfAssets: Option<PolymeshPrimitivesSettlementAffirmationCount> | null | Uint8Array | PolymeshPrimitivesSettlementAffirmationCount | { senderAssetCount?: any; receiverAssetCount?: any; offchainCount?: any } | string) => SubmittableExtrinsic<ApiType>, [u64, Vec<PolymeshPrimitivesSettlementReceiptDetails>, BTreeSet<PolymeshPrimitivesAssetAssetHolder>, Option<PolymeshPrimitivesSettlementAffirmationCount>]>;
      /**
       * Allows additional venues to create instructions involving an asset.
       * 
       * * `asset_id` - AssetId of the token in question.
       * * `venues` - Array of venues that are allowed to create instructions for the token in question.
       * 
       * # Permissions
       * * Asset
       **/
      allowVenues: AugmentedSubmittable<(assetId: PolymeshPrimitivesAssetAssetId | string | Uint8Array, venues: Vec<u64> | (u64 | AnyNumber | Uint8Array)[]) => SubmittableExtrinsic<ApiType>, [PolymeshPrimitivesAssetAssetId, Vec<u64>]>;
      /**
       * Registers a new venue.
       * 
       * * `details` - Extra details about a venue
       * * `signers` - Array of signers that are allowed to sign receipts for this venue
       * * `typ` - Type of venue being created
       **/
      createVenue: AugmentedSubmittable<(details: Bytes | string | Uint8Array, signers: Vec<AccountId32> | (AccountId32 | string | Uint8Array)[], typ: PolymeshPrimitivesSettlementVenueType | 'Other' | 'Distribution' | 'Sto' | 'Exchange' | number | Uint8Array) => SubmittableExtrinsic<ApiType>, [Bytes, Vec<AccountId32>, PolymeshPrimitivesSettlementVenueType]>;
      /**
       * Revokes permission given to venues for creating instructions involving a particular asset.
       * 
       * * `asset_id` - AssetId of the token in question.
       * * `venues` - Array of venues that are no longer allowed to create instructions for the token in question.
       * 
       * # Permissions
       * * Asset
       **/
      disallowVenues: AugmentedSubmittable<(assetId: PolymeshPrimitivesAssetAssetId | string | Uint8Array, venues: Vec<u64> | (u64 | AnyNumber | Uint8Array)[]) => SubmittableExtrinsic<ApiType>, [PolymeshPrimitivesAssetAssetId, Vec<u64>]>;
      /**
       * Manually executes an instruction.
       * 
       * # Arguments
       * * `id`: The [`InstructionId`] of the instruction to be executed.
       * * `asset_holder`:  One of the caller's [`AssetHolder`] which is also a counter patry in the instruction.
       * If None, the caller must be the venue creator or a counter party in a [`Leg::OffChain`].
       * * `fungible_transfers`: The number of fungible legs in the instruction.
       * * `nfts_transfers`: The number of nfts being transferred in the instruction.
       * * `offchain_transfers`: The number of offchain legs in the instruction.
       * * `weight_limit`: An optional maximum [`Weight`] value to be charged for executing the instruction.
       * If the `weight_limit` is less than the required amount, the instruction will fail execution.
       * 
       * Note: calling the rpc method `get_execute_instruction_info` returns an instance of [`ExecuteInstructionInfo`], which contains the count parameters.
       **/
      executeManualInstruction: AugmentedSubmittable<(id: u64 | AnyNumber | Uint8Array, assetHolder: Option<PolymeshPrimitivesAssetAssetHolder> | null | Uint8Array | PolymeshPrimitivesAssetAssetHolder | { Portfolio: any } | { Account: any } | string, fungibleTransfers: u32 | AnyNumber | Uint8Array, nftsTransfers: u32 | AnyNumber | Uint8Array, offchainTransfers: u32 | AnyNumber | Uint8Array, weightLimit: Option<SpWeightsWeightV2Weight> | null | Uint8Array | SpWeightsWeightV2Weight | { refTime?: any; proofSize?: any } | string) => SubmittableExtrinsic<ApiType>, [u64, Option<PolymeshPrimitivesAssetAssetHolder>, u32, u32, u32, Option<SpWeightsWeightV2Weight>]>;
      /**
       * Root callable extrinsic, used as an internal call to execute a scheduled settlement instruction.
       **/
      executeScheduledInstruction: AugmentedSubmittable<(id: u64 | AnyNumber | Uint8Array, weightLimit: SpWeightsWeightV2Weight | { refTime?: any; proofSize?: any } | string | Uint8Array) => SubmittableExtrinsic<ApiType>, [u64, SpWeightsWeightV2Weight]>;
      /**
       * Moves the instruction status to `LockedForExecution`. This function must be called by a
       * mediator of the instruction and will only suceed if the following conditions are met:
       * - All affirmations have been received.
       * - Instruction is pending or has failed at least one time.
       * - All mediator's affirmations are still valid.
       * - All assets are in the allowed venue list.
       * - All senders have the right amount of assets being transferred.
       * - All senders and receivers are compliant and have valid CDD claims.
       * - All assets' statistics are still valid.
       * - There are no frozen assets.
       * 
       * # Arguments
       * * `origin` - The origin of the call, specifying the caller.
       * * `inst_id` - The [`InstructionId`] of the instruction to be locked.
       * * `weight_limit` - A maximum [`Weight`] value to be charged for locking the instruction.
       **/
      lockInstruction: AugmentedSubmittable<(instId: u64 | AnyNumber | Uint8Array, weightLimit: SpWeightsWeightV2Weight | { refTime?: any; proofSize?: any } | string | Uint8Array) => SubmittableExtrinsic<ApiType>, [u64, SpWeightsWeightV2Weight]>;
      /**
       * Rejects an existing instruction.
       * 
       * # Arguments
       * * `id` - the [`InstructionId`] of the instruction being rejected.
       * * `asset_holder` - the [`AssetHolder`] that belongs to the instruction and is rejecting it.
       * 
       * # Permissions
       * * Portfolio
       **/
      rejectInstruction: AugmentedSubmittable<(id: u64 | AnyNumber | Uint8Array, assetHolder: PolymeshPrimitivesAssetAssetHolder | { Portfolio: any } | { Account: any } | string | Uint8Array) => SubmittableExtrinsic<ApiType>, [u64, PolymeshPrimitivesAssetAssetHolder]>;
      /**
       * Rejects an existing instruction - should only be called by mediators, otherwise it will fail.
       * 
       * # Arguments
       * * `instruction_id` - the [`InstructionId`] of the instruction being rejected.
       * * `number_of_assets` - an optional [`AssetCount`] that will be used for a precise fee estimation before executing the extrinsic.
       * 
       * Note: calling the rpc method `get_execute_instruction_info` returns an instance of [`ExecuteInstructionInfo`], which contain the asset count.
       **/
      rejectInstructionAsMediator: AugmentedSubmittable<(instructionId: u64 | AnyNumber | Uint8Array, numberOfAssets: Option<PolymeshPrimitivesSettlementAssetCount> | null | Uint8Array | PolymeshPrimitivesSettlementAssetCount | { fungible?: any; nonFungible?: any; offChain?: any } | string) => SubmittableExtrinsic<ApiType>, [u64, Option<PolymeshPrimitivesSettlementAssetCount>]>;
      /**
       * Rejects an existing instruction.
       * 
       * # Arguments
       * * `id` - the [`InstructionId`] of the instruction being rejected.
       * * `asset_holder` - the [`AssetHolder`] that belongs to the instruction and is rejecting it.
       * * `number_of_assets` - an optional [`AssetCount`] that will be used for a precise fee estimation before executing the extrinsic.
       * 
       * Note: calling the rpc method `get_execute_instruction_info` returns an instance of [`ExecuteInstructionInfo`], which contain the asset count.
       * 
       * # Permissions
       * * Portfolio
       **/
      rejectInstructionWithCount: AugmentedSubmittable<(id: u64 | AnyNumber | Uint8Array, assetHolder: PolymeshPrimitivesAssetAssetHolder | { Portfolio: any } | { Account: any } | string | Uint8Array, numberOfAssets: Option<PolymeshPrimitivesSettlementAssetCount> | null | Uint8Array | PolymeshPrimitivesSettlementAssetCount | { fungible?: any; nonFungible?: any; offChain?: any } | string) => SubmittableExtrinsic<ApiType>, [u64, PolymeshPrimitivesAssetAssetHolder, Option<PolymeshPrimitivesSettlementAssetCount>]>;
      /**
       * Enables or disabled venue filtering for a token.
       * 
       * # Arguments
       * * `asset_id` - AssetId of the token in question.
       * * `enabled` - Boolean that decides if the filtering should be enabled.
       * 
       * # Permissions
       * * Asset
       **/
      setVenueFiltering: AugmentedSubmittable<(assetId: PolymeshPrimitivesAssetAssetId | string | Uint8Array, enabled: bool | boolean | Uint8Array) => SubmittableExtrinsic<ApiType>, [PolymeshPrimitivesAssetAssetId, bool]>;
      /**
       * Edit a venue's details.
       * 
       * * `id` specifies the ID of the venue to edit.
       * * `details` specifies the updated venue details.
       **/
      updateVenueDetails: AugmentedSubmittable<(id: u64 | AnyNumber | Uint8Array, details: Bytes | string | Uint8Array) => SubmittableExtrinsic<ApiType>, [u64, Bytes]>;
      /**
       * Edit a venue's signers.
       * * `id` specifies the ID of the venue to edit.
       * * `signers` specifies the signers to add/remove.
       * * `add_signers` specifies the update type add/remove of venue where add is true and remove is false.
       **/
      updateVenueSigners: AugmentedSubmittable<(id: u64 | AnyNumber | Uint8Array, signers: Vec<AccountId32> | (AccountId32 | string | Uint8Array)[], addSigners: bool | boolean | Uint8Array) => SubmittableExtrinsic<ApiType>, [u64, Vec<AccountId32>, bool]>;
      /**
       * Edit a venue's type.
       * 
       * * `id` specifies the ID of the venue to edit.
       * * `type` specifies the new type of the venue.
       **/
      updateVenueType: AugmentedSubmittable<(id: u64 | AnyNumber | Uint8Array, typ: PolymeshPrimitivesSettlementVenueType | 'Other' | 'Distribution' | 'Sto' | 'Exchange' | number | Uint8Array) => SubmittableExtrinsic<ApiType>, [u64, PolymeshPrimitivesSettlementVenueType]>;
      /**
       * Withdraw an affirmation for a given instruction.
       * 
       * # Arguments
       * * `id` - the [`InstructionId`] of the instruction getting an affirmation withdrawn.
       * * `holder_set` - a set of [`AssetHolder`] under the caller's control and intended for affirmation withdrawal.
       * 
       * # Permissions
       * * Portfolio
       **/
      withdrawAffirmation: AugmentedSubmittable<(id: u64 | AnyNumber | Uint8Array, holderSet: BTreeSet<PolymeshPrimitivesAssetAssetHolder>) => SubmittableExtrinsic<ApiType>, [u64, BTreeSet<PolymeshPrimitivesAssetAssetHolder>]>;
      /**
       * Removes the mediator's affirmation for the instruction - should only be called by mediators, otherwise it will fail.
       * 
       * # Arguments
       * * `origin`: The secondary key of the sender.
       * * `instruction_id`: The [`InstructionId`] that will have the affirmation removed.
       **/
      withdrawAffirmationAsMediator: AugmentedSubmittable<(instructionId: u64 | AnyNumber | Uint8Array) => SubmittableExtrinsic<ApiType>, [u64]>;
      /**
       * Withdraw an affirmation for a given instruction.
       * 
       * # Arguments
       * * `id` - the [`InstructionId`] of the instruction getting an affirmation withdrawn.
       * * `holder_set` - a set of [`AssetHolder`] under the caller's control and intended for affirmation withdrawal.
       * * `number_of_assets` - an optional [`AffirmationCount`] that will be used for a precise fee estimation before executing the extrinsic.
       * 
       * Note: calling the rpc method `get_affirmation_count` returns an instance of [`AffirmationCount`].
       * 
       * # Permissions
       * * Portfolio
       **/
      withdrawAffirmationWithCount: AugmentedSubmittable<(id: u64 | AnyNumber | Uint8Array, holderSet: BTreeSet<PolymeshPrimitivesAssetAssetHolder>, numberOfAssets: Option<PolymeshPrimitivesSettlementAffirmationCount> | null | Uint8Array | PolymeshPrimitivesSettlementAffirmationCount | { senderAssetCount?: any; receiverAssetCount?: any; offchainCount?: any } | string) => SubmittableExtrinsic<ApiType>, [u64, BTreeSet<PolymeshPrimitivesAssetAssetHolder>, Option<PolymeshPrimitivesSettlementAffirmationCount>]>;
      /**
       * Generic tx
       **/
      [key: string]: SubmittableExtrinsicFunction<ApiType>;
    };
    staking: {
      /**
       * Take the origin account as a stash and lock up `value` of its balance. `controller` will
       * be the account that controls it.
       * 
       * `value` must be more than the `minimum_balance` specified by `T::Currency`.
       * 
       * The dispatch origin for this call must be _Signed_ by the stash account.
       * 
       * Emits `Bonded`.
       * ## Complexity
       * - Independent of the arguments. Moderate complexity.
       * - O(1).
       * - Three extra DB entries.
       * 
       * NOTE: Two of the storage writes (`Self::bonded`, `Self::payee`) are _never_ cleaned
       * unless the `origin` falls below _existential deposit_ (or equal to 0) and gets removed
       * as dust.
       **/
      bond: AugmentedSubmittable<(value: Compact<u128> | AnyNumber | Uint8Array, payee: PalletStakingRewardDestination | { Staked: any } | { Stash: any } | { Controller: any } | { Account: any } | { None: any } | string | Uint8Array) => SubmittableExtrinsic<ApiType>, [Compact<u128>, PalletStakingRewardDestination]>;
      /**
       * Add some extra amount that have appeared in the stash `free_balance` into the balance up
       * for staking.
       * 
       * The dispatch origin for this call must be _Signed_ by the stash, not the controller.
       * 
       * Use this if there are additional funds in your stash account that you wish to bond.
       * Unlike [`bond`](Self::bond) or [`unbond`](Self::unbond) this function does not impose
       * any limitation on the amount that can be added.
       * 
       * Emits `Bonded`.
       * 
       * ## Complexity
       * - Independent of the arguments. Insignificant complexity.
       * - O(1).
       **/
      bondExtra: AugmentedSubmittable<(maxAdditional: Compact<u128> | AnyNumber | Uint8Array) => SubmittableExtrinsic<ApiType>, [Compact<u128>]>;
      /**
       * Cancel enactment of a deferred slash.
       * 
       * Can be called by the `T::AdminOrigin`.
       * 
       * Parameters: era and indices of the slashes for that era to kill.
       * They **must** be sorted in ascending order, *and* unique.
       **/
      cancelDeferredSlash: AugmentedSubmittable<(era: u32 | AnyNumber | Uint8Array, slashIndices: Vec<u32> | (u32 | AnyNumber | Uint8Array)[]) => SubmittableExtrinsic<ApiType>, [u32, Vec<u32>]>;
      /**
       * Declare no desire to either validate or nominate.
       * 
       * Effects will be felt at the beginning of the next era.
       * 
       * The dispatch origin for this call must be _Signed_ by the controller, not the stash.
       * 
       * ## Complexity
       * - Independent of the arguments. Insignificant complexity.
       * - Contains one read.
       * - Writes are limited to the `origin` account key.
       **/
      chill: AugmentedSubmittable<() => SubmittableExtrinsic<ApiType>, []>;
      /**
       * Declare a `controller` to stop participating as either a validator or nominator.
       * 
       * Effects will be felt at the beginning of the next era.
       * 
       * The dispatch origin for this call must be _Signed_, but can be called by anyone.
       * 
       * If the caller is the same as the controller being targeted, then no further checks are
       * enforced, and this function behaves just like `chill`.
       * 
       * If the caller is different than the controller being targeted, the following conditions
       * must be met:
       * 
       * * `controller` must belong to a nominator who has become non-decodable,
       * 
       * Or:
       * 
       * * A `ChillThreshold` must be set and checked which defines how close to the max
       * nominators or validators we must reach before users can start chilling one-another.
       * * A `MaxNominatorCount` and `MaxValidatorCount` must be set which is used to determine
       * how close we are to the threshold.
       * * A `MinNominatorBond` and `MinValidatorBond` must be set and checked, which determines
       * if this is a person that should be chilled because they have not met the threshold
       * bond required.
       * 
       * This can be helpful if bond requirements are updated, and we need to remove old users
       * who do not satisfy these requirements.
       **/
      chillOther: AugmentedSubmittable<(stash: AccountId32 | string | Uint8Array) => SubmittableExtrinsic<ApiType>, [AccountId32]>;
      /**
       * Updates a batch of controller accounts to their corresponding stash account if they are
       * not the same. Ignores any controller accounts that do not exist, and does not operate if
       * the stash and controller are already the same.
       * 
       * Effects will be felt instantly (as soon as this function is completed successfully).
       * 
       * The dispatch origin must be `T::AdminOrigin`.
       **/
      deprecateControllerBatch: AugmentedSubmittable<(controllers: Vec<AccountId32> | (AccountId32 | string | Uint8Array)[]) => SubmittableExtrinsic<ApiType>, [Vec<AccountId32>]>;
      /**
       * Force a validator to have at least the minimum commission. This will not affect a
       * validator who already has a commission greater than or equal to the minimum. Any account
       * can call this.
       **/
      forceApplyMinCommission: AugmentedSubmittable<(validatorStash: AccountId32 | string | Uint8Array) => SubmittableExtrinsic<ApiType>, [AccountId32]>;
      /**
       * Force there to be a new era at the end of the next session. After this, it will be
       * reset to normal (non-forced) behaviour.
       * 
       * The dispatch origin must be Root.
       * 
       * # Warning
       * 
       * The election process starts multiple blocks before the end of the era.
       * If this is called just before a new era is triggered, the election process may not
       * have enough blocks to get a result.
       * 
       * ## Complexity
       * - No arguments.
       * - Weight: O(1)
       **/
      forceNewEra: AugmentedSubmittable<() => SubmittableExtrinsic<ApiType>, []>;
      /**
       * Force there to be a new era at the end of sessions indefinitely.
       * 
       * The dispatch origin must be Root.
       * 
       * # Warning
       * 
       * The election process starts multiple blocks before the end of the era.
       * If this is called just before a new era is triggered, the election process may not
       * have enough blocks to get a result.
       **/
      forceNewEraAlways: AugmentedSubmittable<() => SubmittableExtrinsic<ApiType>, []>;
      /**
       * Force there to be no new eras indefinitely.
       * 
       * The dispatch origin must be Root.
       * 
       * # Warning
       * 
       * The election process starts multiple blocks before the end of the era.
       * Thus the election process may be ongoing when this is called. In this case the
       * election will continue until the next era is triggered.
       * 
       * ## Complexity
       * - No arguments.
       * - Weight: O(1)
       **/
      forceNoEras: AugmentedSubmittable<() => SubmittableExtrinsic<ApiType>, []>;
      /**
       * Force a current staker to become completely unstaked, immediately.
       * 
       * The dispatch origin must be Root.
       * 
       * ## Parameters
       * 
       * - `num_slashing_spans`: Refer to comments on [`Call::withdraw_unbonded`] for more
       * details.
       **/
      forceUnstake: AugmentedSubmittable<(stash: AccountId32 | string | Uint8Array, numSlashingSpans: u32 | AnyNumber | Uint8Array) => SubmittableExtrinsic<ApiType>, [AccountId32, u32]>;
      /**
       * Increments the ideal number of validators up to maximum of
       * `ElectionProviderBase::MaxWinners`.
       * 
       * The dispatch origin must be Root.
       * 
       * ## Complexity
       * Same as [`Self::set_validator_count`].
       **/
      increaseValidatorCount: AugmentedSubmittable<(additional: Compact<u32> | AnyNumber | Uint8Array) => SubmittableExtrinsic<ApiType>, [Compact<u32>]>;
      /**
       * Remove the given nominations from the calling validator.
       * 
       * Effects will be felt at the beginning of the next era.
       * 
       * The dispatch origin for this call must be _Signed_ by the controller, not the stash.
       * 
       * - `who`: A list of nominator stash accounts who are nominating this validator which
       * should no longer be nominating this validator.
       * 
       * Note: Making this call only makes sense if you first set the validator preferences to
       * block any further nominations.
       **/
      kick: AugmentedSubmittable<(who: Vec<MultiAddress> | (MultiAddress | { Id: any } | { Index: any } | { Raw: any } | { Address32: any } | { Address20: any } | string | Uint8Array)[]) => SubmittableExtrinsic<ApiType>, [Vec<MultiAddress>]>;
      /**
       * This function allows governance to manually slash a validator and is a
       * **fallback mechanism**.
       * 
       * The dispatch origin must be `T::AdminOrigin`.
       * 
       * ## Parameters
       * - `validator_stash` - The stash account of the validator to slash.
       * - `era` - The era in which the validator was in the active set.
       * - `slash_fraction` - The percentage of the stake to slash, expressed as a Perbill.
       * 
       * ## Behavior
       * 
       * The slash will be applied using the standard slashing mechanics, respecting the
       * configured `SlashDeferDuration`.
       * 
       * This means:
       * - If the validator was already slashed by a higher percentage for the same era, this
       * slash will have no additional effect.
       * - If the validator was previously slashed by a lower percentage, only the difference
       * will be applied.
       * - The slash will be deferred by `SlashDeferDuration` eras before being enacted.
       **/
      manualSlash: AugmentedSubmittable<(validatorStash: AccountId32 | string | Uint8Array, era: u32 | AnyNumber | Uint8Array, slashFraction: Perbill | AnyNumber | Uint8Array) => SubmittableExtrinsic<ApiType>, [AccountId32, u32, Perbill]>;
      /**
       * Removes the legacy Staking locks if they exist.
       * 
       * This removes the legacy lock on the stake with [`Config::OldCurrency`] and creates a
       * hold on it if needed. If all stake cannot be held, the best effort is made to hold as
       * much as possible. The remaining stake is forced withdrawn from the ledger.
       * 
       * The fee is waived if the migration is successful.
       **/
      migrateCurrency: AugmentedSubmittable<(stash: AccountId32 | string | Uint8Array) => SubmittableExtrinsic<ApiType>, [AccountId32]>;
      /**
       * Declare the desire to nominate `targets` for the origin controller.
       * 
       * Effects will be felt at the beginning of the next era.
       * 
       * The dispatch origin for this call must be _Signed_ by the controller, not the stash.
       * 
       * ## Complexity
       * - The transaction's complexity is proportional to the size of `targets` (N)
       * which is capped at CompactAssignments::LIMIT (T::MaxNominations).
       * - Both the reads and writes follow a similar pattern.
       **/
      nominate: AugmentedSubmittable<(targets: Vec<MultiAddress> | (MultiAddress | { Id: any } | { Index: any } | { Raw: any } | { Address32: any } | { Address20: any } | string | Uint8Array)[]) => SubmittableExtrinsic<ApiType>, [Vec<MultiAddress>]>;
      /**
       * Pay out next page of the stakers behind a validator for the given era.
       * 
       * - `validator_stash` is the stash account of the validator.
       * - `era` may be any era between `[current_era - history_depth; current_era]`.
       * 
       * The origin of this call must be _Signed_. Any account can call this function, even if
       * it is not one of the stakers.
       * 
       * The reward payout could be paged in case there are too many nominators backing the
       * `validator_stash`. This call will payout unpaid pages in an ascending order. To claim a
       * specific page, use `payout_stakers_by_page`.`
       * 
       * If all pages are claimed, it returns an error `InvalidPage`.
       **/
      payoutStakers: AugmentedSubmittable<(validatorStash: AccountId32 | string | Uint8Array, era: u32 | AnyNumber | Uint8Array) => SubmittableExtrinsic<ApiType>, [AccountId32, u32]>;
      /**
       * Pay out a page of the stakers behind a validator for the given era and page.
       * 
       * - `validator_stash` is the stash account of the validator.
       * - `era` may be any era between `[current_era - history_depth; current_era]`.
       * - `page` is the page index of nominators to pay out with value between 0 and
       * `num_nominators / T::MaxExposurePageSize`.
       * 
       * The origin of this call must be _Signed_. Any account can call this function, even if
       * it is not one of the stakers.
       * 
       * If a validator has more than [`Config::MaxExposurePageSize`] nominators backing
       * them, then the list of nominators is paged, with each page being capped at
       * [`Config::MaxExposurePageSize`.] If a validator has more than one page of nominators,
       * the call needs to be made for each page separately in order for all the nominators
       * backing a validator to receive the reward. The nominators are not sorted across pages
       * and so it should not be assumed the highest staker would be on the topmost page and vice
       * versa. If rewards are not claimed in [`Config::HistoryDepth`] eras, they are lost.
       **/
      payoutStakersByPage: AugmentedSubmittable<(validatorStash: AccountId32 | string | Uint8Array, era: u32 | AnyNumber | Uint8Array, page: u32 | AnyNumber | Uint8Array) => SubmittableExtrinsic<ApiType>, [AccountId32, u32, u32]>;
      /**
       * Remove all data structures concerning a staker/stash once it is at a state where it can
       * be considered `dust` in the staking system. The requirements are:
       * 
       * 1. the `total_balance` of the stash is below existential deposit.
       * 2. or, the `ledger.total` of the stash is below existential deposit.
       * 3. or, existential deposit is zero and either `total_balance` or `ledger.total` is zero.
       * 
       * The former can happen in cases like a slash; the latter when a fully unbonded account
       * is still receiving staking rewards in `RewardDestination::Staked`.
       * 
       * It can be called by anyone, as long as `stash` meets the above requirements.
       * 
       * Refunds the transaction fees upon successful execution.
       * 
       * ## Parameters
       * 
       * - `num_slashing_spans`: Refer to comments on [`Call::withdraw_unbonded`] for more
       * details.
       **/
      reapStash: AugmentedSubmittable<(stash: AccountId32 | string | Uint8Array, numSlashingSpans: u32 | AnyNumber | Uint8Array) => SubmittableExtrinsic<ApiType>, [AccountId32, u32]>;
      /**
       * Rebond a portion of the stash scheduled to be unlocked.
       * 
       * The dispatch origin must be signed by the controller.
       * 
       * ## Complexity
       * - Time complexity: O(L), where L is unlocking chunks
       * - Bounded by `MaxUnlockingChunks`.
       **/
      rebond: AugmentedSubmittable<(value: Compact<u128> | AnyNumber | Uint8Array) => SubmittableExtrinsic<ApiType>, [Compact<u128>]>;
      /**
       * Restores the state of a ledger which is in an inconsistent state.
       * 
       * The requirements to restore a ledger are the following:
       * * The stash is bonded; or
       * * The stash is not bonded but it has a staking lock left behind; or
       * * If the stash has an associated ledger and its state is inconsistent; or
       * * If the ledger is not corrupted *but* its staking lock is out of sync.
       * 
       * The `maybe_*` input parameters will overwrite the corresponding data and metadata of the
       * ledger associated with the stash. If the input parameters are not set, the ledger will
       * be reset values from on-chain state.
       **/
      restoreLedger: AugmentedSubmittable<(stash: AccountId32 | string | Uint8Array, maybeController: Option<AccountId32> | null | Uint8Array | AccountId32 | string, maybeTotal: Option<u128> | null | Uint8Array | u128 | AnyNumber, maybeUnlocking: Option<Vec<PalletStakingUnlockChunk>> | null | Uint8Array | Vec<PalletStakingUnlockChunk> | (PalletStakingUnlockChunk | { value?: any; era?: any } | string | Uint8Array)[]) => SubmittableExtrinsic<ApiType>, [AccountId32, Option<AccountId32>, Option<u128>, Option<Vec<PalletStakingUnlockChunk>>]>;
      /**
       * Scale up the ideal number of validators by a factor up to maximum of
       * `ElectionProviderBase::MaxWinners`.
       * 
       * The dispatch origin must be Root.
       * 
       * ## Complexity
       * Same as [`Self::set_validator_count`].
       **/
      scaleValidatorCount: AugmentedSubmittable<(factor: Percent | AnyNumber | Uint8Array) => SubmittableExtrinsic<ApiType>, [Percent]>;
      /**
       * (Re-)sets the controller of a stash to the stash itself. This function previously
       * accepted a `controller` argument to set the controller to an account other than the
       * stash itself. This functionality has now been removed, now only setting the controller
       * to the stash, if it is not already.
       * 
       * Effects will be felt instantly (as soon as this function is completed successfully).
       * 
       * The dispatch origin for this call must be _Signed_ by the stash, not the controller.
       * 
       * ## Complexity
       * O(1)
       * - Independent of the arguments. Insignificant complexity.
       * - Contains a limited number of reads.
       * - Writes are limited to the `origin` account key.
       **/
      setController: AugmentedSubmittable<() => SubmittableExtrinsic<ApiType>, []>;
      /**
       * Set the validators who cannot be slashed (if any).
       * 
       * The dispatch origin must be Root.
       **/
      setInvulnerables: AugmentedSubmittable<(invulnerables: Vec<AccountId32> | (AccountId32 | string | Uint8Array)[]) => SubmittableExtrinsic<ApiType>, [Vec<AccountId32>]>;
      /**
       * Sets the minimum amount of commission that each validators must maintain.
       * 
       * This call has lower privilege requirements than `set_staking_config` and can be called
       * by the `T::AdminOrigin`. Root can always call this.
       **/
      setMinCommission: AugmentedSubmittable<(updated: Perbill | AnyNumber | Uint8Array) => SubmittableExtrinsic<ApiType>, [Perbill]>;
      /**
       * (Re-)set the payment target for a controller.
       * 
       * Effects will be felt instantly (as soon as this function is completed successfully).
       * 
       * The dispatch origin for this call must be _Signed_ by the controller, not the stash.
       * 
       * ## Complexity
       * - O(1)
       * - Independent of the arguments. Insignificant complexity.
       * - Contains a limited number of reads.
       * - Writes are limited to the `origin` account key.
       * ---------
       **/
      setPayee: AugmentedSubmittable<(payee: PalletStakingRewardDestination | { Staked: any } | { Stash: any } | { Controller: any } | { Account: any } | { None: any } | string | Uint8Array) => SubmittableExtrinsic<ApiType>, [PalletStakingRewardDestination]>;
      /**
       * Update the various staking configurations .
       * 
       * * `min_nominator_bond`: The minimum active bond needed to be a nominator.
       * * `min_validator_bond`: The minimum active bond needed to be a validator.
       * * `max_nominator_count`: The max number of users who can be a nominator at once. When
       * set to `None`, no limit is enforced.
       * * `max_validator_count`: The max number of users who can be a validator at once. When
       * set to `None`, no limit is enforced.
       * * `chill_threshold`: The ratio of `max_nominator_count` or `max_validator_count` which
       * should be filled in order for the `chill_other` transaction to work.
       * * `min_commission`: The minimum amount of commission that each validators must maintain.
       * This is checked only upon calling `validate`. Existing validators are not affected.
       * 
       * RuntimeOrigin must be Root to call this function.
       * 
       * NOTE: Existing nominators and validators will not be affected by this update.
       * to kick people under the new limits, `chill_other` should be called.
       **/
      setStakingConfigs: AugmentedSubmittable<(minNominatorBond: PalletStakingPalletConfigOpU128 | { Noop: any } | { Set: any } | { Remove: any } | string | Uint8Array, minValidatorBond: PalletStakingPalletConfigOpU128 | { Noop: any } | { Set: any } | { Remove: any } | string | Uint8Array, maxNominatorCount: PalletStakingPalletConfigOpU32 | { Noop: any } | { Set: any } | { Remove: any } | string | Uint8Array, maxValidatorCount: PalletStakingPalletConfigOpU32 | { Noop: any } | { Set: any } | { Remove: any } | string | Uint8Array, chillThreshold: PalletStakingPalletConfigOpPercent | { Noop: any } | { Set: any } | { Remove: any } | string | Uint8Array, minCommission: PalletStakingPalletConfigOpPerbill | { Noop: any } | { Set: any } | { Remove: any } | string | Uint8Array, maxStakedRewards: PalletStakingPalletConfigOpPercent | { Noop: any } | { Set: any } | { Remove: any } | string | Uint8Array) => SubmittableExtrinsic<ApiType>, [PalletStakingPalletConfigOpU128, PalletStakingPalletConfigOpU128, PalletStakingPalletConfigOpU32, PalletStakingPalletConfigOpU32, PalletStakingPalletConfigOpPercent, PalletStakingPalletConfigOpPerbill, PalletStakingPalletConfigOpPercent]>;
      /**
       * Sets the ideal number of validators.
       * 
       * The dispatch origin must be Root.
       * 
       * ## Complexity
       * O(1)
       **/
      setValidatorCount: AugmentedSubmittable<(updated: Compact<u32> | AnyNumber | Uint8Array) => SubmittableExtrinsic<ApiType>, [Compact<u32>]>;
      /**
       * Schedule a portion of the stash to be unlocked ready for transfer out after the bond
       * period ends. If this leaves an amount actively bonded less than
       * [`asset::existential_deposit`], then it is increased to the full amount.
       * 
       * The stash may be chilled if the ledger total amount falls to 0 after unbonding.
       * 
       * The dispatch origin for this call must be _Signed_ by the controller, not the stash.
       * 
       * Once the unlock period is done, you can call `withdraw_unbonded` to actually move
       * the funds out of management ready for transfer.
       * 
       * No more than a limited number of unlocking chunks (see `MaxUnlockingChunks`)
       * can co-exists at the same time. If there are no unlocking chunks slots available
       * [`Call::withdraw_unbonded`] is called to remove some of the chunks (if possible).
       * 
       * If a user encounters the `InsufficientBond` error when calling this extrinsic,
       * they should call `chill` first in order to free up their bonded funds.
       * 
       * Emits `Unbonded`.
       * 
       * See also [`Call::withdraw_unbonded`].
       **/
      unbond: AugmentedSubmittable<(value: Compact<u128> | AnyNumber | Uint8Array) => SubmittableExtrinsic<ApiType>, [Compact<u128>]>;
      /**
       * Migrates an account's `RewardDestination::Controller` to
       * `RewardDestination::Account(controller)`.
       * 
       * Effects will be felt instantly (as soon as this function is completed successfully).
       * 
       * This will waive the transaction fee if the `payee` is successfully migrated.
       **/
      updatePayee: AugmentedSubmittable<(controller: AccountId32 | string | Uint8Array) => SubmittableExtrinsic<ApiType>, [AccountId32]>;
      /**
       * Declare the desire to validate for the origin controller.
       * 
       * Effects will be felt at the beginning of the next era.
       * 
       * The dispatch origin for this call must be _Signed_ by the controller, not the stash.
       **/
      validate: AugmentedSubmittable<(prefs: PalletStakingValidatorPrefs | { commission?: any; blocked?: any } | string | Uint8Array) => SubmittableExtrinsic<ApiType>, [PalletStakingValidatorPrefs]>;
      /**
       * Remove any unlocked chunks from the `unlocking` queue from our management.
       * 
       * This essentially frees up that balance to be used by the stash account to do whatever
       * it wants.
       * 
       * The dispatch origin for this call must be _Signed_ by the controller.
       * 
       * Emits `Withdrawn`.
       * 
       * See also [`Call::unbond`].
       * 
       * ## Parameters
       * 
       * - `num_slashing_spans` indicates the number of metadata slashing spans to clear when
       * this call results in a complete removal of all the data related to the stash account.
       * In this case, the `num_slashing_spans` must be larger or equal to the number of
       * slashing spans associated with the stash account in the [`SlashingSpans`] storage type,
       * otherwise the call will fail. The call weight is directly proportional to
       * `num_slashing_spans`.
       * 
       * ## Complexity
       * O(S) where S is the number of slashing spans to remove
       * NOTE: Weight annotation is the kill scenario, we refund otherwise.
       **/
      withdrawUnbonded: AugmentedSubmittable<(numSlashingSpans: u32 | AnyNumber | Uint8Array) => SubmittableExtrinsic<ApiType>, [u32]>;
      /**
       * Generic tx
       **/
      [key: string]: SubmittableExtrinsicFunction<ApiType>;
    };
    statistics: {
      /**
       * Allow a trusted issuer to init/resync asset/company stats.
       * 
       * # Arguments
       * - `origin` - a signer that has permissions to act as an agent of `asset_id`.
       * - `asset_id` - the [`AssetId`] to change the active stats on.
       * - `stat_type` - stat type to update.
       * - `values` - Updated values for `stat_type`.
       * 
       * # Errors
       * - `StatTypeMissing` - `stat_type` is not enabled for the `asset_id`.
       * - `UnauthorizedAgent` if `origin` is not agent-permissioned for `asset_id`.
       * 
       * # Permissions
       * - Agent
       * - Asset
       **/
      batchUpdateAssetStats: AugmentedSubmittable<(assetId: PolymeshPrimitivesAssetAssetId | string | Uint8Array, statType: PolymeshPrimitivesStatisticsStatType | { operationType?: any; claimIssuer?: any } | string | Uint8Array, values: BTreeSet<PolymeshPrimitivesStatisticsStatUpdate>) => SubmittableExtrinsic<ApiType>, [PolymeshPrimitivesAssetAssetId, PolymeshPrimitivesStatisticsStatType, BTreeSet<PolymeshPrimitivesStatisticsStatUpdate>]>;
      /**
       * Set the active asset stat_types.
       * 
       * # Arguments
       * - `origin` - a signer that has permissions to act as an agent of `asset_id`.
       * - `asset_id` - the [`AssetId`] to change the active stats on.
       * - `stat_types` - the new stat types to replace any existing types.
       * 
       * # Errors
       * - `StatTypeLimitReached` - too many stat types enabled for the `asset_id`.
       * - `CannotRemoveStatTypeInUse` - can not remove a stat type that is in use by transfer conditions.
       * - `UnauthorizedAgent` if `origin` is not agent-permissioned for `asset_id`.
       * 
       * # Permissions
       * - Agent
       * - Asset
       **/
      setActiveAssetStats: AugmentedSubmittable<(assetId: PolymeshPrimitivesAssetAssetId | string | Uint8Array, statTypes: BTreeSet<PolymeshPrimitivesStatisticsStatType>) => SubmittableExtrinsic<ApiType>, [PolymeshPrimitivesAssetAssetId, BTreeSet<PolymeshPrimitivesStatisticsStatType>]>;
      /**
       * Set asset transfer compliance rules.
       * 
       * # Arguments
       * - `origin` - a signer that has permissions to act as an agent of `asset_id`.
       * - `asset_id` - the [`AssetId`] to change the active stats on.
       * - `transfer_conditions` - the new transfer condition to replace any existing conditions.
       * 
       * # Errors
       * - `TransferConditionLimitReached` - too many transfer condititon enabled for `asset_id`.
       * - `StatTypeMissing` - a transfer condition requires a stat type that is not enabled for the `asset_id`.
       * - `UnauthorizedAgent` if `origin` is not agent-permissioned for `asset_id`.
       * 
       * # Permissions
       * - Agent
       * - Asset
       **/
      setAssetTransferCompliance: AugmentedSubmittable<(assetId: PolymeshPrimitivesAssetAssetId | string | Uint8Array, transferConditions: BTreeSet<PolymeshPrimitivesTransferComplianceTransferCondition>) => SubmittableExtrinsic<ApiType>, [PolymeshPrimitivesAssetAssetId, BTreeSet<PolymeshPrimitivesTransferComplianceTransferCondition>]>;
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
      setEntitiesExempt: AugmentedSubmittable<(isExempt: bool | boolean | Uint8Array, exemptKey: PolymeshPrimitivesTransferComplianceTransferConditionExemptKey | { assetId?: any; op?: any; claimType?: any } | string | Uint8Array, entities: BTreeSet<PolymeshPrimitivesIdentityId>) => SubmittableExtrinsic<ApiType>, [bool, PolymeshPrimitivesTransferComplianceTransferConditionExemptKey, BTreeSet<PolymeshPrimitivesIdentityId>]>;
      /**
       * Generic tx
       **/
      [key: string]: SubmittableExtrinsicFunction<ApiType>;
    };
    sto: {
      /**
       * Create a new fundraiser for a security token offering.
       * 
       * This function creates a tiered pricing fundraiser where investors can purchase
       * tokens at different price points. The fundraiser uses Polymesh's settlement
       * infrastructure to ensure compliant and secure token transfers.
       * 
       * # Parameters
       * * `offering_portfolio` - Portfolio containing the tokens being offered for sale
       * * `offering_asset` - Asset ID of the security token being sold
       * * `raising_portfolio` - Portfolio that will receive the raised funds
       * * `raising_asset` - Asset ID of the payment token (e.g., POLYX, stablecoin)
       * * `tiers` - Vector of price tiers (1-10 tiers), each with total amount and price per unit
       * * `venue_id` - STO venue ID for handling settlements (must be owned by caller)
       * * `start` - Optional start time; if `None`, fundraiser begins immediately
       * * `end` - Optional end time; if `None`, fundraiser runs indefinitely
       * * `minimum_investment` - Minimum amount of `raising_asset` required per investment
       * * `fundraiser_name` - Human-readable name for UI display (length limited)
       * 
       * # Permissions Required
       * * **Asset Agent**: Caller must be an authorized external agent for `offering_asset`
       * * **Portfolio Custody**: Caller must have custody of both `offering_portfolio` and `raising_portfolio`
       * * **Venue Ownership**: The specified `venue_id` must be an STO venue owned by the caller
       * 
       * # Errors
       * * `InvalidVenue` - Venue doesn't exist, wrong type, or not owned by caller
       * * `InvalidPriceTiers` - Invalid tier configuration (0 tiers, >10 tiers, zero amounts)
       * * `InvalidOfferingWindow` - Start time is after end time
       * * `Overflow` - Total offering amount calculation overflowed
       **/
      createFundraiser: AugmentedSubmittable<(offeringPortfolio: PolymeshPrimitivesIdentityIdPortfolioId | { did?: any; kind?: any } | string | Uint8Array, offeringAsset: PolymeshPrimitivesAssetAssetId | string | Uint8Array, raisingPortfolio: PolymeshPrimitivesIdentityIdPortfolioId | { did?: any; kind?: any } | string | Uint8Array, raisingAsset: PolymeshPrimitivesAssetAssetId | string | Uint8Array, tiers: Vec<PalletStoPriceTier> | (PalletStoPriceTier | { total?: any; price?: any } | string | Uint8Array)[], venueId: u64 | AnyNumber | Uint8Array, start: Option<u64> | null | Uint8Array | u64 | AnyNumber, end: Option<u64> | null | Uint8Array | u64 | AnyNumber, minimumInvestment: u128 | AnyNumber | Uint8Array, fundraiserName: Bytes | string | Uint8Array) => SubmittableExtrinsic<ApiType>, [PolymeshPrimitivesIdentityIdPortfolioId, PolymeshPrimitivesAssetAssetId, PolymeshPrimitivesIdentityIdPortfolioId, PolymeshPrimitivesAssetAssetId, Vec<PalletStoPriceTier>, u64, Option<u64>, Option<u64>, u128, Bytes]>;
      /**
       * Enable off-chain funding support for a fundraiser.
       * 
       * This function allows a fundraiser to accept off-chain payments through
       * cryptographically signed receipts. Once enabled, investors can use the
       * `invest` function with `FundingMethod::OffChain` to provide payment
       * receipts instead of on-chain portfolio transfers.
       * 
       * # Parameters
       * * `offering_asset` - Asset ID associated with the fundraiser
       * * `fundraiser_id` - Unique identifier of the fundraiser to enable off-chain funding for
       * * `ticker` - Ticker symbol of the off-chain asset that will be accepted as payment
       * 
       * # Permissions Required
       * * **Asset Agent**: Caller must be an authorized external agent for `offering_asset`
       * OR be the original creator of the fundraiser
       * 
       * # Errors
       * * `FundraiserNotFound` - Specified fundraiser doesn't exist
       * * `FundraiserClosed` - Fundraiser has been permanently closed
       * * `Unauthorized` - Caller lacks required permissions
       **/
      enableOffchainFunding: AugmentedSubmittable<(offeringAsset: PolymeshPrimitivesAssetAssetId | string | Uint8Array, fundraiserId: u64 | AnyNumber | Uint8Array, ticker: PolymeshPrimitivesTicker | string | Uint8Array) => SubmittableExtrinsic<ApiType>, [PolymeshPrimitivesAssetAssetId, u64, PolymeshPrimitivesTicker]>;
      /**
       * Temporarily freeze a fundraiser to prevent new investments.
       * 
       * When a fundraiser is frozen, it cannot accept new investments but remains
       * otherwise intact. This is useful for pausing activity while resolving
       * issues or during maintenance periods. The fundraiser can be unfrozen
       * later to resume normal operations.
       * 
       * # Parameters
       * * `offering_asset` - Asset ID associated with the fundraiser to freeze
       * * `fundraiser_id` - Unique identifier of the fundraiser to freeze
       * 
       * # Permissions Required
       * * **Asset Agent**: Caller must be an authorized external agent for `offering_asset`
       * 
       * # Errors
       * * `FundraiserNotFound` - Specified fundraiser doesn't exist
       * * `FundraiserClosed` - Fundraiser has already been permanently closed
       * * `Unauthorized` - Caller lacks required asset agent permissions
       **/
      freezeFundraiser: AugmentedSubmittable<(offeringAsset: PolymeshPrimitivesAssetAssetId | string | Uint8Array, fundraiserId: u64 | AnyNumber | Uint8Array) => SubmittableExtrinsic<ApiType>, [PolymeshPrimitivesAssetAssetId, u64]>;
      /**
       * Invest in a fundraiser using on-chain or off-chain funding.
       * 
       * This function allows investors to purchase tokens from an active fundraiser.
       * The investment is processed through multiple price tiers in order, starting
       * with the lowest-priced tier. The purchase creates a settlement instruction
       * that transfers tokens and payment between the appropriate portfolios.
       * 
       * # Parameters
       * * `offering_asset` - Asset ID of the security token being purchased
       * * `fundraiser_id` - Unique identifier of the fundraiser to invest in
       * * `investment_portfolio` - Portfolio where purchased tokens will be deposited
       * * `funding` - Payment method: either `OnChain(portfolio_id)` for on-chain assets
       * or `OffChain(receipt_details)` for off-chain receipts with signature verification
       * * `purchase_amount` - Number of `offering_asset` tokens to purchase
       * * `max_price` - Optional maximum price per token; if specified, investment fails
       * if the blended price across tiers exceeds this limit
       * 
       * # Permissions Required
       * * **Portfolio Custody**: Caller must have custody of `investment_portfolio`
       * * **Funding Portfolio**: If using on-chain funding, caller must have custody
       * of the funding portfolio specified in the `FundingMethod`
       * 
       * # Errors
       * * `FundraiserNotFound` - Specified fundraiser doesn't exist
       * * `FundraiserNotLive` - Fundraiser is frozen or closed
       * * `FundraiserExpired` - Current time is outside fundraiser's active window
       * * `InsufficientTokensRemaining` - Not enough tokens available across all tiers
       * * `InvestmentAmountTooLow` - Total cost is below minimum investment threshold
       * * `MaxPriceExceeded` - Blended price exceeds investor's maximum price limit
       * * `OffchainFundingNotAllowed` - Off-chain funding not enabled for this fundraiser
       * * `InvalidSignature` - Off-chain receipt signature verification failed
       **/
      invest: AugmentedSubmittable<(offeringAsset: PolymeshPrimitivesAssetAssetId | string | Uint8Array, fundraiserId: u64 | AnyNumber | Uint8Array, investmentPortfolio: PolymeshPrimitivesIdentityIdPortfolioId | { did?: any; kind?: any } | string | Uint8Array, funding: PalletStoFundingMethod | { OnChain: any } | { OffChain: any } | string | Uint8Array, purchaseAmount: u128 | AnyNumber | Uint8Array, maxPrice: Option<u128> | null | Uint8Array | u128 | AnyNumber) => SubmittableExtrinsic<ApiType>, [PolymeshPrimitivesAssetAssetId, u64, PolymeshPrimitivesIdentityIdPortfolioId, PalletStoFundingMethod, u128, Option<u128>]>;
      /**
       * Modify the time window when a fundraiser is active for investments.
       * 
       * This function allows authorized agents to update the start and end times
       * of an active fundraiser. This can be useful for extending fundraising
       * periods, adjusting launch timing, or responding to market conditions.
       * The fundraiser must not be permanently closed to modify its window.
       * 
       * # Parameters
       * * `offering_asset` - Asset ID associated with the fundraiser to modify
       * * `fundraiser_id` - Unique identifier of the fundraiser to modify
       * * `start` - New start time for the fundraiser (can be in the past or future)
       * * `end` - New optional end time; if `None`, the fundraiser runs indefinitely
       * 
       * # Permissions Required
       * * **Asset Agent**: Caller must be an authorized external agent for `offering_asset`
       * 
       * # Errors
       * * `FundraiserNotFound` - Specified fundraiser doesn't exist
       * * `FundraiserClosed` - Fundraiser has been permanently closed
       * * `FundraiserExpired` - Fundraiser has already expired (past its original end time)
       * * `InvalidOfferingWindow` - New start time is after new end time
       * * `Unauthorized` - Caller lacks required asset agent permissions
       **/
      modifyFundraiserWindow: AugmentedSubmittable<(offeringAsset: PolymeshPrimitivesAssetAssetId | string | Uint8Array, fundraiserId: u64 | AnyNumber | Uint8Array, start: u64 | AnyNumber | Uint8Array, end: Option<u64> | null | Uint8Array | u64 | AnyNumber) => SubmittableExtrinsic<ApiType>, [PolymeshPrimitivesAssetAssetId, u64, u64, Option<u64>]>;
      /**
       * Permanently stop a fundraiser and unlock remaining tokens.
       * 
       * This function permanently closes a fundraiser, preventing any further
       * investments. Any remaining tokens that haven't been sold are unlocked
       * and returned to the offering portfolio. Once stopped, a fundraiser
       * cannot be restarted.
       * 
       * # Parameters
       * * `offering_asset` - Asset ID associated with the fundraiser to stop
       * * `fundraiser_id` - Unique identifier of the fundraiser to stop
       * 
       * # Permissions Required
       * * **Asset Agent**: Caller must be an authorized external agent for `offering_asset`
       * OR be the original creator of the fundraiser
       * 
       * # Errors
       * * `FundraiserNotFound` - Specified fundraiser doesn't exist
       * * `FundraiserClosed` - Fundraiser has already been permanently closed
       * * `Unauthorized` - Caller lacks required permissions
       **/
      stop: AugmentedSubmittable<(offeringAsset: PolymeshPrimitivesAssetAssetId | string | Uint8Array, fundraiserId: u64 | AnyNumber | Uint8Array) => SubmittableExtrinsic<ApiType>, [PolymeshPrimitivesAssetAssetId, u64]>;
      /**
       * Resume a frozen fundraiser to allow new investments.
       * 
       * This function unfreezes a previously frozen fundraiser, returning it to
       * the Live status where it can accept new investments. The fundraiser
       * must not be permanently closed for this operation to succeed.
       * 
       * # Parameters
       * * `offering_asset` - Asset ID associated with the fundraiser to unfreeze
       * * `fundraiser_id` - Unique identifier of the fundraiser to unfreeze
       * 
       * # Permissions Required
       * * **Asset Agent**: Caller must be an authorized external agent for `offering_asset`
       * 
       * # Errors
       * * `FundraiserNotFound` - Specified fundraiser doesn't exist
       * * `FundraiserClosed` - Fundraiser has been permanently closed and cannot be unfrozen
       * * `Unauthorized` - Caller lacks required asset agent permissions
       **/
      unfreezeFundraiser: AugmentedSubmittable<(offeringAsset: PolymeshPrimitivesAssetAssetId | string | Uint8Array, fundraiserId: u64 | AnyNumber | Uint8Array) => SubmittableExtrinsic<ApiType>, [PolymeshPrimitivesAssetAssetId, u64]>;
      /**
       * Generic tx
       **/
      [key: string]: SubmittableExtrinsicFunction<ApiType>;
    };
    sudo: {
      /**
       * Permanently removes the sudo key.
       * 
       * **This cannot be un-done.**
       **/
      removeKey: AugmentedSubmittable<() => SubmittableExtrinsic<ApiType>, []>;
      /**
       * Authenticates the current sudo key and sets the given AccountId (`new`) as the new sudo
       * key.
       **/
      setKey: AugmentedSubmittable<(updated: MultiAddress | { Id: any } | { Index: any } | { Raw: any } | { Address32: any } | { Address20: any } | string | Uint8Array) => SubmittableExtrinsic<ApiType>, [MultiAddress]>;
      /**
       * Authenticates the sudo key and dispatches a function call with `Root` origin.
       **/
      sudo: AugmentedSubmittable<(call: Call | IMethod | string | Uint8Array) => SubmittableExtrinsic<ApiType>, [Call]>;
      /**
       * Authenticates the sudo key and dispatches a function call with `Signed` origin from
       * a given account.
       * 
       * The dispatch origin for this call must be _Signed_.
       **/
      sudoAs: AugmentedSubmittable<(who: MultiAddress | { Id: any } | { Index: any } | { Raw: any } | { Address32: any } | { Address20: any } | string | Uint8Array, call: Call | IMethod | string | Uint8Array) => SubmittableExtrinsic<ApiType>, [MultiAddress, Call]>;
      /**
       * Authenticates the sudo key and dispatches a function call with `Root` origin.
       * This function does not check the weight of the call, and instead allows the
       * Sudo user to specify the weight of the call.
       * 
       * The dispatch origin for this call must be _Signed_.
       **/
      sudoUncheckedWeight: AugmentedSubmittable<(call: Call | IMethod | string | Uint8Array, weight: SpWeightsWeightV2Weight | { refTime?: any; proofSize?: any } | string | Uint8Array) => SubmittableExtrinsic<ApiType>, [Call, SpWeightsWeightV2Weight]>;
      /**
       * Generic tx
       **/
      [key: string]: SubmittableExtrinsicFunction<ApiType>;
    };
    system: {
      /**
       * Provide the preimage (runtime binary) `code` for an upgrade that has been authorized.
       * 
       * If the authorization required a version check, this call will ensure the spec name
       * remains unchanged and that the spec version has increased.
       * 
       * Depending on the runtime's `OnSetCode` configuration, this function may directly apply
       * the new `code` in the same block or attempt to schedule the upgrade.
       * 
       * All origins are allowed.
       **/
      applyAuthorizedUpgrade: AugmentedSubmittable<(code: Bytes | string | Uint8Array) => SubmittableExtrinsic<ApiType>, [Bytes]>;
      /**
       * Authorize an upgrade to a given `code_hash` for the runtime. The runtime can be supplied
       * later.
       * 
       * This call requires Root origin.
       **/
      authorizeUpgrade: AugmentedSubmittable<(codeHash: H256 | string | Uint8Array) => SubmittableExtrinsic<ApiType>, [H256]>;
      /**
       * Authorize an upgrade to a given `code_hash` for the runtime. The runtime can be supplied
       * later.
       * 
       * WARNING: This authorizes an upgrade that will take place without any safety checks, for
       * example that the spec name remains the same and that the version number increases. Not
       * recommended for normal use. Use `authorize_upgrade` instead.
       * 
       * This call requires Root origin.
       **/
      authorizeUpgradeWithoutChecks: AugmentedSubmittable<(codeHash: H256 | string | Uint8Array) => SubmittableExtrinsic<ApiType>, [H256]>;
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
       * Can be executed by every `origin`.
       **/
      remark: AugmentedSubmittable<(remark: Bytes | string | Uint8Array) => SubmittableExtrinsic<ApiType>, [Bytes]>;
      /**
       * Make some on-chain remark and emit event.
       **/
      remarkWithEvent: AugmentedSubmittable<(remark: Bytes | string | Uint8Array) => SubmittableExtrinsic<ApiType>, [Bytes]>;
      /**
       * Set the new runtime code.
       **/
      setCode: AugmentedSubmittable<(code: Bytes | string | Uint8Array) => SubmittableExtrinsic<ApiType>, [Bytes]>;
      /**
       * Set the new runtime code without doing any checks of the given `code`.
       * 
       * Note that runtime upgrades will not run if this is called with a not-increasing spec
       * version!
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
      setExpiresAfter: AugmentedSubmittable<(expiry: PolymeshPrimitivesMaybeBlock | { Some: any } | { None: any } | string | Uint8Array) => SubmittableExtrinsic<ApiType>, [PolymeshPrimitivesMaybeBlock]>;
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
      voteOrPropose: AugmentedSubmittable<(approve: bool | boolean | Uint8Array, call: Call | IMethod | string | Uint8Array) => SubmittableExtrinsic<ApiType>, [bool, Call]>;
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
      disableMember: AugmentedSubmittable<(who: PolymeshPrimitivesIdentityId | string | Uint8Array, expiry: Option<u64> | null | Uint8Array | u64 | AnyNumber, at: Option<u64> | null | Uint8Array | u64 | AnyNumber) => SubmittableExtrinsic<ApiType>, [PolymeshPrimitivesIdentityId, Option<u64>, Option<u64>]>;
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
    timestamp: {
      /**
       * Set the current time.
       * 
       * This call should be invoked exactly once per block. It will panic at the finalization
       * phase, if this call hasn't been invoked by that time.
       * 
       * The timestamp should be greater than the previous one by the amount specified by
       * [`Config::MinimumPeriod`].
       * 
       * The dispatch origin for this call must be _None_.
       * 
       * This dispatch class is _Mandatory_ to ensure it gets executed in the block. Be aware
       * that changing the complexity of this call could result exhausting the resources in a
       * block to execute any other calls.
       * 
       * ## Complexity
       * - `O(1)` (Note that implementations of `OnTimestampSet` must also be `O(1)`)
       * - 1 storage read and 1 storage mutation (codec `O(1)` because of `DidUpdate::take` in
       * `on_finalize`)
       * - 1 event handler `on_timestamp_set`. Must be `O(1)`.
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
      setExpiresAfter: AugmentedSubmittable<(expiry: PolymeshPrimitivesMaybeBlock | { Some: any } | { None: any } | string | Uint8Array) => SubmittableExtrinsic<ApiType>, [PolymeshPrimitivesMaybeBlock]>;
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
      voteOrPropose: AugmentedSubmittable<(approve: bool | boolean | Uint8Array, call: Call | IMethod | string | Uint8Array) => SubmittableExtrinsic<ApiType>, [bool, Call]>;
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
      disableMember: AugmentedSubmittable<(who: PolymeshPrimitivesIdentityId | string | Uint8Array, expiry: Option<u64> | null | Uint8Array | u64 | AnyNumber, at: Option<u64> | null | Uint8Array | u64 | AnyNumber) => SubmittableExtrinsic<ApiType>, [PolymeshPrimitivesIdentityId, Option<u64>, Option<u64>]>;
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
       * Send a call through an indexed pseudonym of the sender.
       * 
       * Filter from origin are passed along. The call will be dispatched with an origin which
       * use the same filter as the origin of this call.
       * 
       * The dispatch origin for this call must be _Signed_.
       **/
      asDerivative: AugmentedSubmittable<(index: u16 | AnyNumber | Uint8Array, call: Call | IMethod | string | Uint8Array) => SubmittableExtrinsic<ApiType>, [u16, Call]>;
      /**
       * Send a batch of dispatch calls.
       * 
       * May be called from any origin except `None`.
       * 
       * - `calls`: The calls to be dispatched from the same origin. The number of call must not
       * exceed the constant: `batched_calls_limit` (available in constant metadata).
       * 
       * If origin is root then the calls are dispatched without checking origin filter. (This
       * includes bypassing `frame_system::Config::BaseCallFilter`).
       * 
       * ## Complexity
       * - O(C) where C is the number of calls to be batched.
       * 
       * This will return `Ok` in all circumstances. To determine the success of the batch, an
       * event is deposited. If a call failed and the batch was interrupted, then the
       * `BatchInterrupted` event is deposited, along with the number of successful calls made
       * and the error of the failed call. If all were successful, then the `BatchCompleted`
       * event is deposited.
       **/
      batch: AugmentedSubmittable<(calls: Vec<Call> | (Call | IMethod | string | Uint8Array)[]) => SubmittableExtrinsic<ApiType>, [Vec<Call>]>;
      /**
       * Send a batch of dispatch calls and atomically execute them.
       * The whole transaction will rollback and fail if any of the calls failed.
       * 
       * May be called from any origin except `None`.
       * 
       * - `calls`: The calls to be dispatched from the same origin. The number of call must not
       * exceed the constant: `batched_calls_limit` (available in constant metadata).
       * 
       * If origin is root then the calls are dispatched without checking origin filter. (This
       * includes bypassing `frame_system::Config::BaseCallFilter`).
       * 
       * ## Complexity
       * - O(C) where C is the number of calls to be batched.
       **/
      batchAll: AugmentedSubmittable<(calls: Vec<Call> | (Call | IMethod | string | Uint8Array)[]) => SubmittableExtrinsic<ApiType>, [Vec<Call>]>;
      /**
       * Dispatches a function call with a provided origin.
       * 
       * The dispatch origin for this call must be _Root_.
       * 
       * ## Complexity
       * - O(1).
       **/
      dispatchAs: AugmentedSubmittable<(asOrigin: PolymeshRuntimeDevelopRuntimeOriginCaller | { system: any } | { PolymeshCommittee: any } | { TechnicalCommittee: any } | { UpgradeCommittee: any } | string | Uint8Array, call: Call | IMethod | string | Uint8Array) => SubmittableExtrinsic<ApiType>, [PolymeshRuntimeDevelopRuntimeOriginCaller, Call]>;
      /**
       * Send a batch of dispatch calls.
       * Unlike `batch`, it allows errors and won't interrupt.
       * 
       * May be called from any origin except `None`.
       * 
       * - `calls`: The calls to be dispatched from the same origin. The number of call must not
       * exceed the constant: `batched_calls_limit` (available in constant metadata).
       * 
       * If origin is root then the calls are dispatch without checking origin filter. (This
       * includes bypassing `frame_system::Config::BaseCallFilter`).
       * 
       * ## Complexity
       * - O(C) where C is the number of calls to be batched.
       **/
      forceBatch: AugmentedSubmittable<(calls: Vec<Call> | (Call | IMethod | string | Uint8Array)[]) => SubmittableExtrinsic<ApiType>, [Vec<Call>]>;
      /**
       * Dispatch a function call with a specified weight.
       * 
       * This function does not check the weight of the call, and instead allows the
       * Root origin to specify the weight of the call.
       * 
       * The dispatch origin for this call must be _Root_.
       **/
      withWeight: AugmentedSubmittable<(call: Call | IMethod | string | Uint8Array, weight: SpWeightsWeightV2Weight | { refTime?: any; proofSize?: any } | string | Uint8Array) => SubmittableExtrinsic<ApiType>, [Call, SpWeightsWeightV2Weight]>;
      /**
       * Generic tx
       **/
      [key: string]: SubmittableExtrinsicFunction<ApiType>;
    };
    validators: {
      /**
       * Adds a permissioned identity and sets its preferences.
       * 
       * The dispatch origin must be Root.
       **/
      addPermissionedValidator: AugmentedSubmittable<(identity: PolymeshPrimitivesIdentityId | string | Uint8Array, intendedCount: Option<u32> | null | Uint8Array | u32 | AnyNumber) => SubmittableExtrinsic<ApiType>, [PolymeshPrimitivesIdentityId, Option<u32>]>;
      /**
       * Switch slashing status on the basis of given `slashing_switch`. Can only be called by root.
       **/
      changeSlashingAllowedFor: AugmentedSubmittable<(slashingSwitch: PalletValidatorsSlashingSwitch | 'Validator' | 'ValidatorAndNominator' | 'None' | number | Uint8Array) => SubmittableExtrinsic<ApiType>, [PalletValidatorsSlashingSwitch]>;
      /**
       * Governance council forcefully chills a validator. Effects will be felt at the beginning of the next era.
       **/
      chillFromGovernance: AugmentedSubmittable<(identity: PolymeshPrimitivesIdentityId | string | Uint8Array, stashKeys: Vec<AccountId32> | (AccountId32 | string | Uint8Array)[]) => SubmittableExtrinsic<ApiType>, [PolymeshPrimitivesIdentityId, Vec<AccountId32>]>;
      payoutStakersBySystem: AugmentedSubmittable<(validatorStash: AccountId32 | string | Uint8Array, era: u32 | AnyNumber | Uint8Array) => SubmittableExtrinsic<ApiType>, [AccountId32, u32]>;
      /**
       * Remove an identity from the pool of (wannabe) validator identities. Effects are known in the next session.
       * 
       * The dispatch origin must be Root.
       * 
       * # Arguments
       * * origin Required origin for removing a potential validator.
       * * identity Validator's IdentityId.
       **/
      removePermissionedValidator: AugmentedSubmittable<(identity: PolymeshPrimitivesIdentityId | string | Uint8Array) => SubmittableExtrinsic<ApiType>, [PolymeshPrimitivesIdentityId]>;
      /**
       * Changes commission rate which applies to all validators. Only Governance
       * committee is allowed to change this value.
       * 
       * # Arguments
       * * `new_cap` the new commission cap.
       **/
      setCommissionCap: AugmentedSubmittable<(newCap: Perbill | AnyNumber | Uint8Array) => SubmittableExtrinsic<ApiType>, [Perbill]>;
      /**
       * Sets the intended count to `new_intended_count` for the given `identity`.
       **/
      updatePermissionedValidatorIntendedCount: AugmentedSubmittable<(identity: PolymeshPrimitivesIdentityId | string | Uint8Array, newIntendedCount: u32 | AnyNumber | Uint8Array) => SubmittableExtrinsic<ApiType>, [PolymeshPrimitivesIdentityId, u32]>;
      /**
       * Generic tx
       **/
      [key: string]: SubmittableExtrinsicFunction<ApiType>;
    };
  } // AugmentedSubmittables
} // declare module
