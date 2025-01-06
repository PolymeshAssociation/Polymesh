// Auto-generated via `yarn polkadot-types-from-chain`, do not edit
/* eslint-disable */

// import type lookup before we augment - in some environments
// this is required to allow for ambient/previous definitions
import '@polkadot/api-base/types/submittable';

import type { ApiTypes, AugmentedSubmittable, SubmittableExtrinsic, SubmittableExtrinsicFunction } from '@polkadot/api-base/types';
import type { BTreeSet, Bytes, Compact, Option, U8aFixed, Vec, bool, u128, u16, u32, u64, u8 } from '@polkadot/types-codec';
import type { AnyNumber, IMethod, ITuple } from '@polkadot/types-codec/types';
import type { AccountId32, Call, H256, MultiAddress, Perbill, Percent, Permill } from '@polkadot/types/interfaces/runtime';
import type { PalletContractsWasmDeterminism, PalletCorporateActionsBallotBallotMeta, PalletCorporateActionsBallotBallotTimeRange, PalletCorporateActionsBallotBallotVote, PalletCorporateActionsCaId, PalletCorporateActionsCaKind, PalletCorporateActionsInitiateCorporateActionArgs, PalletCorporateActionsRecordDateSpec, PalletCorporateActionsTargetIdentities, PalletElectionProviderMultiPhaseRawSolution, PalletElectionProviderMultiPhaseSolutionOrSnapshotSize, PalletImOnlineHeartbeat, PalletImOnlineSr25519AppSr25519Signature, PalletPipsSnapshotResult, PalletStakingPalletConfigOpPerbill, PalletStakingPalletConfigOpPercent, PalletStakingPalletConfigOpU128, PalletStakingPalletConfigOpU32, PalletStakingRewardDestination, PalletStakingValidatorPrefs, PalletStoFundingMethod, PalletStoPriceTier, PalletUtilityUniqueCall, PalletValidatorsSlashingSwitch, PolymeshCommonUtilitiesCheckpointScheduleCheckpoints, PolymeshCommonUtilitiesIdentityCreateChildIdentityWithAuth, PolymeshCommonUtilitiesIdentitySecondaryKeyWithAuth, PolymeshCommonUtilitiesProtocolFeeProtocolOp, PolymeshContractsApi, PolymeshContractsChainExtensionExtrinsicId, PolymeshContractsNextUpgrade, PolymeshPrimitivesAgentAgentGroup, PolymeshPrimitivesAssetAssetId, PolymeshPrimitivesAssetAssetType, PolymeshPrimitivesAssetIdentifier, PolymeshPrimitivesAssetMetadataAssetMetadataKey, PolymeshPrimitivesAssetMetadataAssetMetadataSpec, PolymeshPrimitivesAssetMetadataAssetMetadataValueDetail, PolymeshPrimitivesAssetNonFungibleType, PolymeshPrimitivesAuthorizationAuthorizationData, PolymeshPrimitivesBeneficiary, PolymeshPrimitivesComplianceManagerComplianceRequirement, PolymeshPrimitivesCondition, PolymeshPrimitivesConditionTrustedIssuer, PolymeshPrimitivesDocument, PolymeshPrimitivesIdentityClaimClaim, PolymeshPrimitivesIdentityClaimClaimType, PolymeshPrimitivesIdentityClaimScope, PolymeshPrimitivesIdentityId, PolymeshPrimitivesIdentityIdPortfolioId, PolymeshPrimitivesIdentityIdPortfolioKind, PolymeshPrimitivesMaybeBlock, PolymeshPrimitivesMemo, PolymeshPrimitivesNftNfTs, PolymeshPrimitivesNftNftCollectionKeys, PolymeshPrimitivesNftNftMetadataAttribute, PolymeshPrimitivesPortfolioFund, PolymeshPrimitivesPosRatio, PolymeshPrimitivesSecondaryKey, PolymeshPrimitivesSecondaryKeyExtrinsicPermissions, PolymeshPrimitivesSecondaryKeyPermissions, PolymeshPrimitivesSecondaryKeySignatory, PolymeshPrimitivesSettlementAffirmationCount, PolymeshPrimitivesSettlementAssetCount, PolymeshPrimitivesSettlementLeg, PolymeshPrimitivesSettlementReceiptDetails, PolymeshPrimitivesSettlementSettlementType, PolymeshPrimitivesSettlementVenueType, PolymeshPrimitivesStatisticsStatType, PolymeshPrimitivesStatisticsStatUpdate, PolymeshPrimitivesTicker, PolymeshPrimitivesTransferComplianceTransferCondition, PolymeshPrimitivesTransferComplianceTransferConditionExemptKey, PolymeshRuntimeDevelopRuntimeOriginCaller, PolymeshRuntimeDevelopRuntimeSessionKeys, SpConsensusBabeDigestsNextConfigDescriptor, SpConsensusGrandpaEquivocationProof, SpConsensusSlotsEquivocationProof, SpNposElectionsElectionScore, SpNposElectionsSupport, SpRuntimeMultiSignature, SpSessionMembershipProof, SpWeightsWeightV2Weight } from '@polkadot/types/lookup';

export type __AugmentedSubmittable = AugmentedSubmittable<() => unknown>;
export type __SubmittableExtrinsic<ApiType extends ApiTypes> = SubmittableExtrinsic<ApiType>;
export type __SubmittableExtrinsicFunction<ApiType extends ApiTypes> = SubmittableExtrinsicFunction<ApiType>;

declare module '@polkadot/api-base/types/submittable' {
  interface AugmentedSubmittables<ApiType extends ApiTypes> {
    asset: {
      /**
       * See [`Pallet::accept_asset_ownership_transfer`].
       **/
      acceptAssetOwnershipTransfer: AugmentedSubmittable<(authId: u64 | AnyNumber | Uint8Array) => SubmittableExtrinsic<ApiType>, [u64]>;
      /**
       * See [`Pallet::accept_ticker_transfer`].
       **/
      acceptTickerTransfer: AugmentedSubmittable<(authId: u64 | AnyNumber | Uint8Array) => SubmittableExtrinsic<ApiType>, [u64]>;
      /**
       * See [`Pallet::add_documents`].
       **/
      addDocuments: AugmentedSubmittable<(docs: Vec<PolymeshPrimitivesDocument> | (PolymeshPrimitivesDocument | { uri?: any; contentHash?: any; name?: any; docType?: any; filingDate?: any } | string | Uint8Array)[], assetId: PolymeshPrimitivesAssetAssetId | string | Uint8Array) => SubmittableExtrinsic<ApiType>, [Vec<PolymeshPrimitivesDocument>, PolymeshPrimitivesAssetAssetId]>;
      /**
       * See [`Pallet::add_mandatory_mediators`].
       **/
      addMandatoryMediators: AugmentedSubmittable<(assetId: PolymeshPrimitivesAssetAssetId | string | Uint8Array, mediators: BTreeSet<PolymeshPrimitivesIdentityId>) => SubmittableExtrinsic<ApiType>, [PolymeshPrimitivesAssetAssetId, BTreeSet<PolymeshPrimitivesIdentityId>]>;
      /**
       * See [`Pallet::controller_transfer`].
       **/
      controllerTransfer: AugmentedSubmittable<(assetId: PolymeshPrimitivesAssetAssetId | string | Uint8Array, value: u128 | AnyNumber | Uint8Array, fromPortfolio: PolymeshPrimitivesIdentityIdPortfolioId | { did?: any; kind?: any } | string | Uint8Array) => SubmittableExtrinsic<ApiType>, [PolymeshPrimitivesAssetAssetId, u128, PolymeshPrimitivesIdentityIdPortfolioId]>;
      /**
       * See [`Pallet::create_asset`].
       **/
      createAsset: AugmentedSubmittable<(assetName: Bytes | string | Uint8Array, divisible: bool | boolean | Uint8Array, assetType: PolymeshPrimitivesAssetAssetType | { EquityCommon: any } | { EquityPreferred: any } | { Commodity: any } | { FixedIncome: any } | { REIT: any } | { Fund: any } | { RevenueShareAgreement: any } | { StructuredProduct: any } | { Derivative: any } | { Custom: any } | { StableCoin: any } | { NonFungible: any } | string | Uint8Array, assetIdentifiers: Vec<PolymeshPrimitivesAssetIdentifier> | (PolymeshPrimitivesAssetIdentifier | { CUSIP: any } | { CINS: any } | { ISIN: any } | { LEI: any } | { FIGI: any } | string | Uint8Array)[], fundingRoundName: Option<Bytes> | null | Uint8Array | Bytes | string) => SubmittableExtrinsic<ApiType>, [Bytes, bool, PolymeshPrimitivesAssetAssetType, Vec<PolymeshPrimitivesAssetIdentifier>, Option<Bytes>]>;
      /**
       * See [`Pallet::create_asset_with_custom_type`].
       **/
      createAssetWithCustomType: AugmentedSubmittable<(assetName: Bytes | string | Uint8Array, divisible: bool | boolean | Uint8Array, customAssetType: Bytes | string | Uint8Array, assetIdentifiers: Vec<PolymeshPrimitivesAssetIdentifier> | (PolymeshPrimitivesAssetIdentifier | { CUSIP: any } | { CINS: any } | { ISIN: any } | { LEI: any } | { FIGI: any } | string | Uint8Array)[], fundingRoundName: Option<Bytes> | null | Uint8Array | Bytes | string) => SubmittableExtrinsic<ApiType>, [Bytes, bool, Bytes, Vec<PolymeshPrimitivesAssetIdentifier>, Option<Bytes>]>;
      /**
       * See [`Pallet::exempt_asset_affirmation`].
       **/
      exemptAssetAffirmation: AugmentedSubmittable<(assetId: PolymeshPrimitivesAssetAssetId | string | Uint8Array) => SubmittableExtrinsic<ApiType>, [PolymeshPrimitivesAssetAssetId]>;
      /**
       * See [`Pallet::freeze`].
       **/
      freeze: AugmentedSubmittable<(assetId: PolymeshPrimitivesAssetAssetId | string | Uint8Array) => SubmittableExtrinsic<ApiType>, [PolymeshPrimitivesAssetAssetId]>;
      /**
       * See [`Pallet::issue`].
       **/
      issue: AugmentedSubmittable<(assetId: PolymeshPrimitivesAssetAssetId | string | Uint8Array, amount: u128 | AnyNumber | Uint8Array, portfolioKind: PolymeshPrimitivesIdentityIdPortfolioKind | { Default: any } | { User: any } | string | Uint8Array) => SubmittableExtrinsic<ApiType>, [PolymeshPrimitivesAssetAssetId, u128, PolymeshPrimitivesIdentityIdPortfolioKind]>;
      /**
       * See [`Pallet::link_ticker_to_asset_id`].
       **/
      linkTickerToAssetId: AugmentedSubmittable<(ticker: PolymeshPrimitivesTicker | string | Uint8Array, assetId: PolymeshPrimitivesAssetAssetId | string | Uint8Array) => SubmittableExtrinsic<ApiType>, [PolymeshPrimitivesTicker, PolymeshPrimitivesAssetAssetId]>;
      /**
       * See [`Pallet::make_divisible`].
       **/
      makeDivisible: AugmentedSubmittable<(assetId: PolymeshPrimitivesAssetAssetId | string | Uint8Array) => SubmittableExtrinsic<ApiType>, [PolymeshPrimitivesAssetAssetId]>;
      /**
       * See [`Pallet::pre_approve_asset`].
       **/
      preApproveAsset: AugmentedSubmittable<(assetId: PolymeshPrimitivesAssetAssetId | string | Uint8Array) => SubmittableExtrinsic<ApiType>, [PolymeshPrimitivesAssetAssetId]>;
      /**
       * See [`Pallet::redeem`].
       **/
      redeem: AugmentedSubmittable<(assetId: PolymeshPrimitivesAssetAssetId | string | Uint8Array, value: u128 | AnyNumber | Uint8Array, portfolioKind: PolymeshPrimitivesIdentityIdPortfolioKind | { Default: any } | { User: any } | string | Uint8Array) => SubmittableExtrinsic<ApiType>, [PolymeshPrimitivesAssetAssetId, u128, PolymeshPrimitivesIdentityIdPortfolioKind]>;
      /**
       * See [`Pallet::register_and_set_local_asset_metadata`].
       **/
      registerAndSetLocalAssetMetadata: AugmentedSubmittable<(assetId: PolymeshPrimitivesAssetAssetId | string | Uint8Array, name: Bytes | string | Uint8Array, spec: PolymeshPrimitivesAssetMetadataAssetMetadataSpec | { url?: any; description?: any; typeDef?: any } | string | Uint8Array, value: Bytes | string | Uint8Array, detail: Option<PolymeshPrimitivesAssetMetadataAssetMetadataValueDetail> | null | Uint8Array | PolymeshPrimitivesAssetMetadataAssetMetadataValueDetail | { expire?: any; lockStatus?: any } | string) => SubmittableExtrinsic<ApiType>, [PolymeshPrimitivesAssetAssetId, Bytes, PolymeshPrimitivesAssetMetadataAssetMetadataSpec, Bytes, Option<PolymeshPrimitivesAssetMetadataAssetMetadataValueDetail>]>;
      /**
       * See [`Pallet::register_asset_metadata_global_type`].
       **/
      registerAssetMetadataGlobalType: AugmentedSubmittable<(name: Bytes | string | Uint8Array, spec: PolymeshPrimitivesAssetMetadataAssetMetadataSpec | { url?: any; description?: any; typeDef?: any } | string | Uint8Array) => SubmittableExtrinsic<ApiType>, [Bytes, PolymeshPrimitivesAssetMetadataAssetMetadataSpec]>;
      /**
       * See [`Pallet::register_asset_metadata_local_type`].
       **/
      registerAssetMetadataLocalType: AugmentedSubmittable<(assetId: PolymeshPrimitivesAssetAssetId | string | Uint8Array, name: Bytes | string | Uint8Array, spec: PolymeshPrimitivesAssetMetadataAssetMetadataSpec | { url?: any; description?: any; typeDef?: any } | string | Uint8Array) => SubmittableExtrinsic<ApiType>, [PolymeshPrimitivesAssetAssetId, Bytes, PolymeshPrimitivesAssetMetadataAssetMetadataSpec]>;
      /**
       * See [`Pallet::register_custom_asset_type`].
       **/
      registerCustomAssetType: AugmentedSubmittable<(ty: Bytes | string | Uint8Array) => SubmittableExtrinsic<ApiType>, [Bytes]>;
      /**
       * See [`Pallet::register_unique_ticker`].
       **/
      registerUniqueTicker: AugmentedSubmittable<(ticker: PolymeshPrimitivesTicker | string | Uint8Array) => SubmittableExtrinsic<ApiType>, [PolymeshPrimitivesTicker]>;
      /**
       * See [`Pallet::remove_asset_affirmation_exemption`].
       **/
      removeAssetAffirmationExemption: AugmentedSubmittable<(assetId: PolymeshPrimitivesAssetAssetId | string | Uint8Array) => SubmittableExtrinsic<ApiType>, [PolymeshPrimitivesAssetAssetId]>;
      /**
       * See [`Pallet::remove_asset_pre_approval`].
       **/
      removeAssetPreApproval: AugmentedSubmittable<(assetId: PolymeshPrimitivesAssetAssetId | string | Uint8Array) => SubmittableExtrinsic<ApiType>, [PolymeshPrimitivesAssetAssetId]>;
      /**
       * See [`Pallet::remove_documents`].
       **/
      removeDocuments: AugmentedSubmittable<(docsId: Vec<u32> | (u32 | AnyNumber | Uint8Array)[], assetId: PolymeshPrimitivesAssetAssetId | string | Uint8Array) => SubmittableExtrinsic<ApiType>, [Vec<u32>, PolymeshPrimitivesAssetAssetId]>;
      /**
       * See [`Pallet::remove_local_metadata_key`].
       **/
      removeLocalMetadataKey: AugmentedSubmittable<(assetId: PolymeshPrimitivesAssetAssetId | string | Uint8Array, localKey: u64 | AnyNumber | Uint8Array) => SubmittableExtrinsic<ApiType>, [PolymeshPrimitivesAssetAssetId, u64]>;
      /**
       * See [`Pallet::remove_mandatory_mediators`].
       **/
      removeMandatoryMediators: AugmentedSubmittable<(assetId: PolymeshPrimitivesAssetAssetId | string | Uint8Array, mediators: BTreeSet<PolymeshPrimitivesIdentityId>) => SubmittableExtrinsic<ApiType>, [PolymeshPrimitivesAssetAssetId, BTreeSet<PolymeshPrimitivesIdentityId>]>;
      /**
       * See [`Pallet::remove_metadata_value`].
       **/
      removeMetadataValue: AugmentedSubmittable<(assetId: PolymeshPrimitivesAssetAssetId | string | Uint8Array, metadataKey: PolymeshPrimitivesAssetMetadataAssetMetadataKey | { Global: any } | { Local: any } | string | Uint8Array) => SubmittableExtrinsic<ApiType>, [PolymeshPrimitivesAssetAssetId, PolymeshPrimitivesAssetMetadataAssetMetadataKey]>;
      /**
       * See [`Pallet::rename_asset`].
       **/
      renameAsset: AugmentedSubmittable<(assetId: PolymeshPrimitivesAssetAssetId | string | Uint8Array, assetName: Bytes | string | Uint8Array) => SubmittableExtrinsic<ApiType>, [PolymeshPrimitivesAssetAssetId, Bytes]>;
      /**
       * See [`Pallet::set_asset_metadata`].
       **/
      setAssetMetadata: AugmentedSubmittable<(assetId: PolymeshPrimitivesAssetAssetId | string | Uint8Array, key: PolymeshPrimitivesAssetMetadataAssetMetadataKey | { Global: any } | { Local: any } | string | Uint8Array, value: Bytes | string | Uint8Array, detail: Option<PolymeshPrimitivesAssetMetadataAssetMetadataValueDetail> | null | Uint8Array | PolymeshPrimitivesAssetMetadataAssetMetadataValueDetail | { expire?: any; lockStatus?: any } | string) => SubmittableExtrinsic<ApiType>, [PolymeshPrimitivesAssetAssetId, PolymeshPrimitivesAssetMetadataAssetMetadataKey, Bytes, Option<PolymeshPrimitivesAssetMetadataAssetMetadataValueDetail>]>;
      /**
       * See [`Pallet::set_asset_metadata_details`].
       **/
      setAssetMetadataDetails: AugmentedSubmittable<(assetId: PolymeshPrimitivesAssetAssetId | string | Uint8Array, key: PolymeshPrimitivesAssetMetadataAssetMetadataKey | { Global: any } | { Local: any } | string | Uint8Array, detail: PolymeshPrimitivesAssetMetadataAssetMetadataValueDetail | { expire?: any; lockStatus?: any } | string | Uint8Array) => SubmittableExtrinsic<ApiType>, [PolymeshPrimitivesAssetAssetId, PolymeshPrimitivesAssetMetadataAssetMetadataKey, PolymeshPrimitivesAssetMetadataAssetMetadataValueDetail]>;
      /**
       * See [`Pallet::set_funding_round`].
       **/
      setFundingRound: AugmentedSubmittable<(assetId: PolymeshPrimitivesAssetAssetId | string | Uint8Array, fundingRoundName: Bytes | string | Uint8Array) => SubmittableExtrinsic<ApiType>, [PolymeshPrimitivesAssetAssetId, Bytes]>;
      /**
       * See [`Pallet::unfreeze`].
       **/
      unfreeze: AugmentedSubmittable<(assetId: PolymeshPrimitivesAssetAssetId | string | Uint8Array) => SubmittableExtrinsic<ApiType>, [PolymeshPrimitivesAssetAssetId]>;
      /**
       * See [`Pallet::unlink_ticker_from_asset_id`].
       **/
      unlinkTickerFromAssetId: AugmentedSubmittable<(ticker: PolymeshPrimitivesTicker | string | Uint8Array, assetId: PolymeshPrimitivesAssetAssetId | string | Uint8Array) => SubmittableExtrinsic<ApiType>, [PolymeshPrimitivesTicker, PolymeshPrimitivesAssetAssetId]>;
      /**
       * See [`Pallet::update_asset_type`].
       **/
      updateAssetType: AugmentedSubmittable<(assetId: PolymeshPrimitivesAssetAssetId | string | Uint8Array, assetType: PolymeshPrimitivesAssetAssetType | { EquityCommon: any } | { EquityPreferred: any } | { Commodity: any } | { FixedIncome: any } | { REIT: any } | { Fund: any } | { RevenueShareAgreement: any } | { StructuredProduct: any } | { Derivative: any } | { Custom: any } | { StableCoin: any } | { NonFungible: any } | string | Uint8Array) => SubmittableExtrinsic<ApiType>, [PolymeshPrimitivesAssetAssetId, PolymeshPrimitivesAssetAssetType]>;
      /**
       * See [`Pallet::update_global_metadata_spec`].
       **/
      updateGlobalMetadataSpec: AugmentedSubmittable<(assetMetadataName: Bytes | string | Uint8Array, assetMetadataSpec: PolymeshPrimitivesAssetMetadataAssetMetadataSpec | { url?: any; description?: any; typeDef?: any } | string | Uint8Array) => SubmittableExtrinsic<ApiType>, [Bytes, PolymeshPrimitivesAssetMetadataAssetMetadataSpec]>;
      /**
       * See [`Pallet::update_identifiers`].
       **/
      updateIdentifiers: AugmentedSubmittable<(assetId: PolymeshPrimitivesAssetAssetId | string | Uint8Array, assetIdentifiers: Vec<PolymeshPrimitivesAssetIdentifier> | (PolymeshPrimitivesAssetIdentifier | { CUSIP: any } | { CINS: any } | { ISIN: any } | { LEI: any } | { FIGI: any } | string | Uint8Array)[]) => SubmittableExtrinsic<ApiType>, [PolymeshPrimitivesAssetAssetId, Vec<PolymeshPrimitivesAssetIdentifier>]>;
      /**
       * Generic tx
       **/
      [key: string]: SubmittableExtrinsicFunction<ApiType>;
    };
    babe: {
      /**
       * See [`Pallet::plan_config_change`].
       **/
      planConfigChange: AugmentedSubmittable<(config: SpConsensusBabeDigestsNextConfigDescriptor | { V1: any } | string | Uint8Array) => SubmittableExtrinsic<ApiType>, [SpConsensusBabeDigestsNextConfigDescriptor]>;
      /**
       * See [`Pallet::report_equivocation`].
       **/
      reportEquivocation: AugmentedSubmittable<(equivocationProof: SpConsensusSlotsEquivocationProof | { offender?: any; slot?: any; firstHeader?: any; secondHeader?: any } | string | Uint8Array, keyOwnerProof: SpSessionMembershipProof | { session?: any; trieNodes?: any; validatorCount?: any } | string | Uint8Array) => SubmittableExtrinsic<ApiType>, [SpConsensusSlotsEquivocationProof, SpSessionMembershipProof]>;
      /**
       * See [`Pallet::report_equivocation_unsigned`].
       **/
      reportEquivocationUnsigned: AugmentedSubmittable<(equivocationProof: SpConsensusSlotsEquivocationProof | { offender?: any; slot?: any; firstHeader?: any; secondHeader?: any } | string | Uint8Array, keyOwnerProof: SpSessionMembershipProof | { session?: any; trieNodes?: any; validatorCount?: any } | string | Uint8Array) => SubmittableExtrinsic<ApiType>, [SpConsensusSlotsEquivocationProof, SpSessionMembershipProof]>;
      /**
       * Generic tx
       **/
      [key: string]: SubmittableExtrinsicFunction<ApiType>;
    };
    balances: {
      /**
       * See [`Pallet::burn_account_balance`].
       **/
      burnAccountBalance: AugmentedSubmittable<(amount: u128 | AnyNumber | Uint8Array) => SubmittableExtrinsic<ApiType>, [u128]>;
      /**
       * See [`Pallet::deposit_block_reward_reserve_balance`].
       **/
      depositBlockRewardReserveBalance: AugmentedSubmittable<(value: Compact<u128> | AnyNumber | Uint8Array) => SubmittableExtrinsic<ApiType>, [Compact<u128>]>;
      /**
       * See [`Pallet::force_transfer`].
       **/
      forceTransfer: AugmentedSubmittable<(source: MultiAddress | { Id: any } | { Index: any } | { Raw: any } | { Address32: any } | { Address20: any } | string | Uint8Array, dest: MultiAddress | { Id: any } | { Index: any } | { Raw: any } | { Address32: any } | { Address20: any } | string | Uint8Array, value: Compact<u128> | AnyNumber | Uint8Array) => SubmittableExtrinsic<ApiType>, [MultiAddress, MultiAddress, Compact<u128>]>;
      /**
       * See [`Pallet::set_balance`].
       **/
      setBalance: AugmentedSubmittable<(who: MultiAddress | { Id: any } | { Index: any } | { Raw: any } | { Address32: any } | { Address20: any } | string | Uint8Array, newFree: Compact<u128> | AnyNumber | Uint8Array, newReserved: Compact<u128> | AnyNumber | Uint8Array) => SubmittableExtrinsic<ApiType>, [MultiAddress, Compact<u128>, Compact<u128>]>;
      /**
       * See [`Pallet::transfer`].
       **/
      transfer: AugmentedSubmittable<(dest: MultiAddress | { Id: any } | { Index: any } | { Raw: any } | { Address32: any } | { Address20: any } | string | Uint8Array, value: Compact<u128> | AnyNumber | Uint8Array) => SubmittableExtrinsic<ApiType>, [MultiAddress, Compact<u128>]>;
      /**
       * See [`Pallet::transfer_with_memo`].
       **/
      transferWithMemo: AugmentedSubmittable<(dest: MultiAddress | { Id: any } | { Index: any } | { Raw: any } | { Address32: any } | { Address20: any } | string | Uint8Array, value: Compact<u128> | AnyNumber | Uint8Array, memo: Option<PolymeshPrimitivesMemo> | null | Uint8Array | PolymeshPrimitivesMemo | string) => SubmittableExtrinsic<ApiType>, [MultiAddress, Compact<u128>, Option<PolymeshPrimitivesMemo>]>;
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
    capitalDistribution: {
      /**
       * See [`Pallet::claim`].
       **/
      claim: AugmentedSubmittable<(caId: PalletCorporateActionsCaId | { assetId?: any; localId?: any } | string | Uint8Array) => SubmittableExtrinsic<ApiType>, [PalletCorporateActionsCaId]>;
      /**
       * See [`Pallet::distribute`].
       **/
      distribute: AugmentedSubmittable<(caId: PalletCorporateActionsCaId | { assetId?: any; localId?: any } | string | Uint8Array, portfolio: Option<u64> | null | Uint8Array | u64 | AnyNumber, currency: PolymeshPrimitivesAssetAssetId | string | Uint8Array, perShare: u128 | AnyNumber | Uint8Array, amount: u128 | AnyNumber | Uint8Array, paymentAt: u64 | AnyNumber | Uint8Array, expiresAt: Option<u64> | null | Uint8Array | u64 | AnyNumber) => SubmittableExtrinsic<ApiType>, [PalletCorporateActionsCaId, Option<u64>, PolymeshPrimitivesAssetAssetId, u128, u128, u64, Option<u64>]>;
      /**
       * See [`Pallet::push_benefit`].
       **/
      pushBenefit: AugmentedSubmittable<(caId: PalletCorporateActionsCaId | { assetId?: any; localId?: any } | string | Uint8Array, holder: PolymeshPrimitivesIdentityId | string | Uint8Array) => SubmittableExtrinsic<ApiType>, [PalletCorporateActionsCaId, PolymeshPrimitivesIdentityId]>;
      /**
       * See [`Pallet::reclaim`].
       **/
      reclaim: AugmentedSubmittable<(caId: PalletCorporateActionsCaId | { assetId?: any; localId?: any } | string | Uint8Array) => SubmittableExtrinsic<ApiType>, [PalletCorporateActionsCaId]>;
      /**
       * See [`Pallet::remove_distribution`].
       **/
      removeDistribution: AugmentedSubmittable<(caId: PalletCorporateActionsCaId | { assetId?: any; localId?: any } | string | Uint8Array) => SubmittableExtrinsic<ApiType>, [PalletCorporateActionsCaId]>;
      /**
       * Generic tx
       **/
      [key: string]: SubmittableExtrinsicFunction<ApiType>;
    };
    cddServiceProviders: {
      /**
       * See [`Pallet::abdicate_membership`].
       **/
      abdicateMembership: AugmentedSubmittable<() => SubmittableExtrinsic<ApiType>, []>;
      /**
       * See [`Pallet::add_member`].
       **/
      addMember: AugmentedSubmittable<(who: PolymeshPrimitivesIdentityId | string | Uint8Array) => SubmittableExtrinsic<ApiType>, [PolymeshPrimitivesIdentityId]>;
      /**
       * See [`Pallet::disable_member`].
       **/
      disableMember: AugmentedSubmittable<(who: PolymeshPrimitivesIdentityId | string | Uint8Array, expiry: Option<u64> | null | Uint8Array | u64 | AnyNumber, at: Option<u64> | null | Uint8Array | u64 | AnyNumber) => SubmittableExtrinsic<ApiType>, [PolymeshPrimitivesIdentityId, Option<u64>, Option<u64>]>;
      /**
       * See [`Pallet::remove_member`].
       **/
      removeMember: AugmentedSubmittable<(who: PolymeshPrimitivesIdentityId | string | Uint8Array) => SubmittableExtrinsic<ApiType>, [PolymeshPrimitivesIdentityId]>;
      /**
       * See [`Pallet::reset_members`].
       **/
      resetMembers: AugmentedSubmittable<(members: Vec<PolymeshPrimitivesIdentityId> | (PolymeshPrimitivesIdentityId | string | Uint8Array)[]) => SubmittableExtrinsic<ApiType>, [Vec<PolymeshPrimitivesIdentityId>]>;
      /**
       * See [`Pallet::set_active_members_limit`].
       **/
      setActiveMembersLimit: AugmentedSubmittable<(limit: u32 | AnyNumber | Uint8Array) => SubmittableExtrinsic<ApiType>, [u32]>;
      /**
       * See [`Pallet::swap_member`].
       **/
      swapMember: AugmentedSubmittable<(remove: PolymeshPrimitivesIdentityId | string | Uint8Array, add: PolymeshPrimitivesIdentityId | string | Uint8Array) => SubmittableExtrinsic<ApiType>, [PolymeshPrimitivesIdentityId, PolymeshPrimitivesIdentityId]>;
      /**
       * Generic tx
       **/
      [key: string]: SubmittableExtrinsicFunction<ApiType>;
    };
    checkpoint: {
      /**
       * See [`Pallet::create_checkpoint`].
       **/
      createCheckpoint: AugmentedSubmittable<(assetId: PolymeshPrimitivesAssetAssetId | string | Uint8Array) => SubmittableExtrinsic<ApiType>, [PolymeshPrimitivesAssetAssetId]>;
      /**
       * See [`Pallet::create_schedule`].
       **/
      createSchedule: AugmentedSubmittable<(assetId: PolymeshPrimitivesAssetAssetId | string | Uint8Array, schedule: PolymeshCommonUtilitiesCheckpointScheduleCheckpoints | { pending?: any } | string | Uint8Array) => SubmittableExtrinsic<ApiType>, [PolymeshPrimitivesAssetAssetId, PolymeshCommonUtilitiesCheckpointScheduleCheckpoints]>;
      /**
       * See [`Pallet::remove_schedule`].
       **/
      removeSchedule: AugmentedSubmittable<(assetId: PolymeshPrimitivesAssetAssetId | string | Uint8Array, id: u64 | AnyNumber | Uint8Array) => SubmittableExtrinsic<ApiType>, [PolymeshPrimitivesAssetAssetId, u64]>;
      /**
       * See [`Pallet::set_schedules_max_complexity`].
       **/
      setSchedulesMaxComplexity: AugmentedSubmittable<(maxComplexity: u64 | AnyNumber | Uint8Array) => SubmittableExtrinsic<ApiType>, [u64]>;
      /**
       * Generic tx
       **/
      [key: string]: SubmittableExtrinsicFunction<ApiType>;
    };
    committeeMembership: {
      /**
       * See [`Pallet::abdicate_membership`].
       **/
      abdicateMembership: AugmentedSubmittable<() => SubmittableExtrinsic<ApiType>, []>;
      /**
       * See [`Pallet::add_member`].
       **/
      addMember: AugmentedSubmittable<(who: PolymeshPrimitivesIdentityId | string | Uint8Array) => SubmittableExtrinsic<ApiType>, [PolymeshPrimitivesIdentityId]>;
      /**
       * See [`Pallet::disable_member`].
       **/
      disableMember: AugmentedSubmittable<(who: PolymeshPrimitivesIdentityId | string | Uint8Array, expiry: Option<u64> | null | Uint8Array | u64 | AnyNumber, at: Option<u64> | null | Uint8Array | u64 | AnyNumber) => SubmittableExtrinsic<ApiType>, [PolymeshPrimitivesIdentityId, Option<u64>, Option<u64>]>;
      /**
       * See [`Pallet::remove_member`].
       **/
      removeMember: AugmentedSubmittable<(who: PolymeshPrimitivesIdentityId | string | Uint8Array) => SubmittableExtrinsic<ApiType>, [PolymeshPrimitivesIdentityId]>;
      /**
       * See [`Pallet::reset_members`].
       **/
      resetMembers: AugmentedSubmittable<(members: Vec<PolymeshPrimitivesIdentityId> | (PolymeshPrimitivesIdentityId | string | Uint8Array)[]) => SubmittableExtrinsic<ApiType>, [Vec<PolymeshPrimitivesIdentityId>]>;
      /**
       * See [`Pallet::set_active_members_limit`].
       **/
      setActiveMembersLimit: AugmentedSubmittable<(limit: u32 | AnyNumber | Uint8Array) => SubmittableExtrinsic<ApiType>, [u32]>;
      /**
       * See [`Pallet::swap_member`].
       **/
      swapMember: AugmentedSubmittable<(remove: PolymeshPrimitivesIdentityId | string | Uint8Array, add: PolymeshPrimitivesIdentityId | string | Uint8Array) => SubmittableExtrinsic<ApiType>, [PolymeshPrimitivesIdentityId, PolymeshPrimitivesIdentityId]>;
      /**
       * Generic tx
       **/
      [key: string]: SubmittableExtrinsicFunction<ApiType>;
    };
    complianceManager: {
      /**
       * See [`Pallet::add_compliance_requirement`].
       **/
      addComplianceRequirement: AugmentedSubmittable<(assetId: PolymeshPrimitivesAssetAssetId | string | Uint8Array, senderConditions: Vec<PolymeshPrimitivesCondition> | (PolymeshPrimitivesCondition | { conditionType?: any; issuers?: any } | string | Uint8Array)[], receiverConditions: Vec<PolymeshPrimitivesCondition> | (PolymeshPrimitivesCondition | { conditionType?: any; issuers?: any } | string | Uint8Array)[]) => SubmittableExtrinsic<ApiType>, [PolymeshPrimitivesAssetAssetId, Vec<PolymeshPrimitivesCondition>, Vec<PolymeshPrimitivesCondition>]>;
      /**
       * See [`Pallet::add_default_trusted_claim_issuer`].
       **/
      addDefaultTrustedClaimIssuer: AugmentedSubmittable<(assetId: PolymeshPrimitivesAssetAssetId | string | Uint8Array, issuer: PolymeshPrimitivesConditionTrustedIssuer | { issuer?: any; trustedFor?: any } | string | Uint8Array) => SubmittableExtrinsic<ApiType>, [PolymeshPrimitivesAssetAssetId, PolymeshPrimitivesConditionTrustedIssuer]>;
      /**
       * See [`Pallet::change_compliance_requirement`].
       **/
      changeComplianceRequirement: AugmentedSubmittable<(assetId: PolymeshPrimitivesAssetAssetId | string | Uint8Array, newReq: PolymeshPrimitivesComplianceManagerComplianceRequirement | { senderConditions?: any; receiverConditions?: any; id?: any } | string | Uint8Array) => SubmittableExtrinsic<ApiType>, [PolymeshPrimitivesAssetAssetId, PolymeshPrimitivesComplianceManagerComplianceRequirement]>;
      /**
       * See [`Pallet::pause_asset_compliance`].
       **/
      pauseAssetCompliance: AugmentedSubmittable<(assetId: PolymeshPrimitivesAssetAssetId | string | Uint8Array) => SubmittableExtrinsic<ApiType>, [PolymeshPrimitivesAssetAssetId]>;
      /**
       * See [`Pallet::remove_compliance_requirement`].
       **/
      removeComplianceRequirement: AugmentedSubmittable<(assetId: PolymeshPrimitivesAssetAssetId | string | Uint8Array, id: u32 | AnyNumber | Uint8Array) => SubmittableExtrinsic<ApiType>, [PolymeshPrimitivesAssetAssetId, u32]>;
      /**
       * See [`Pallet::remove_default_trusted_claim_issuer`].
       **/
      removeDefaultTrustedClaimIssuer: AugmentedSubmittable<(assetId: PolymeshPrimitivesAssetAssetId | string | Uint8Array, issuer: PolymeshPrimitivesIdentityId | string | Uint8Array) => SubmittableExtrinsic<ApiType>, [PolymeshPrimitivesAssetAssetId, PolymeshPrimitivesIdentityId]>;
      /**
       * See [`Pallet::replace_asset_compliance`].
       **/
      replaceAssetCompliance: AugmentedSubmittable<(assetId: PolymeshPrimitivesAssetAssetId | string | Uint8Array, assetCompliance: Vec<PolymeshPrimitivesComplianceManagerComplianceRequirement> | (PolymeshPrimitivesComplianceManagerComplianceRequirement | { senderConditions?: any; receiverConditions?: any; id?: any } | string | Uint8Array)[]) => SubmittableExtrinsic<ApiType>, [PolymeshPrimitivesAssetAssetId, Vec<PolymeshPrimitivesComplianceManagerComplianceRequirement>]>;
      /**
       * See [`Pallet::reset_asset_compliance`].
       **/
      resetAssetCompliance: AugmentedSubmittable<(assetId: PolymeshPrimitivesAssetAssetId | string | Uint8Array) => SubmittableExtrinsic<ApiType>, [PolymeshPrimitivesAssetAssetId]>;
      /**
       * See [`Pallet::resume_asset_compliance`].
       **/
      resumeAssetCompliance: AugmentedSubmittable<(assetId: PolymeshPrimitivesAssetAssetId | string | Uint8Array) => SubmittableExtrinsic<ApiType>, [PolymeshPrimitivesAssetAssetId]>;
      /**
       * Generic tx
       **/
      [key: string]: SubmittableExtrinsicFunction<ApiType>;
    };
    contracts: {
      /**
       * See [`Pallet::call`].
       **/
      call: AugmentedSubmittable<(dest: MultiAddress | { Id: any } | { Index: any } | { Raw: any } | { Address32: any } | { Address20: any } | string | Uint8Array, value: Compact<u128> | AnyNumber | Uint8Array, gasLimit: SpWeightsWeightV2Weight | { refTime?: any; proofSize?: any } | string | Uint8Array, storageDepositLimit: Option<Compact<u128>> | null | Uint8Array | Compact<u128> | AnyNumber, data: Bytes | string | Uint8Array) => SubmittableExtrinsic<ApiType>, [MultiAddress, Compact<u128>, SpWeightsWeightV2Weight, Option<Compact<u128>>, Bytes]>;
      /**
       * See [`Pallet::call_old_weight`].
       **/
      callOldWeight: AugmentedSubmittable<(dest: MultiAddress | { Id: any } | { Index: any } | { Raw: any } | { Address32: any } | { Address20: any } | string | Uint8Array, value: Compact<u128> | AnyNumber | Uint8Array, gasLimit: Compact<u64> | AnyNumber | Uint8Array, storageDepositLimit: Option<Compact<u128>> | null | Uint8Array | Compact<u128> | AnyNumber, data: Bytes | string | Uint8Array) => SubmittableExtrinsic<ApiType>, [MultiAddress, Compact<u128>, Compact<u64>, Option<Compact<u128>>, Bytes]>;
      /**
       * See [`Pallet::instantiate`].
       **/
      instantiate: AugmentedSubmittable<(value: Compact<u128> | AnyNumber | Uint8Array, gasLimit: SpWeightsWeightV2Weight | { refTime?: any; proofSize?: any } | string | Uint8Array, storageDepositLimit: Option<Compact<u128>> | null | Uint8Array | Compact<u128> | AnyNumber, codeHash: H256 | string | Uint8Array, data: Bytes | string | Uint8Array, salt: Bytes | string | Uint8Array) => SubmittableExtrinsic<ApiType>, [Compact<u128>, SpWeightsWeightV2Weight, Option<Compact<u128>>, H256, Bytes, Bytes]>;
      /**
       * See [`Pallet::instantiate_old_weight`].
       **/
      instantiateOldWeight: AugmentedSubmittable<(value: Compact<u128> | AnyNumber | Uint8Array, gasLimit: Compact<u64> | AnyNumber | Uint8Array, storageDepositLimit: Option<Compact<u128>> | null | Uint8Array | Compact<u128> | AnyNumber, codeHash: H256 | string | Uint8Array, data: Bytes | string | Uint8Array, salt: Bytes | string | Uint8Array) => SubmittableExtrinsic<ApiType>, [Compact<u128>, Compact<u64>, Option<Compact<u128>>, H256, Bytes, Bytes]>;
      /**
       * See [`Pallet::instantiate_with_code`].
       **/
      instantiateWithCode: AugmentedSubmittable<(value: Compact<u128> | AnyNumber | Uint8Array, gasLimit: SpWeightsWeightV2Weight | { refTime?: any; proofSize?: any } | string | Uint8Array, storageDepositLimit: Option<Compact<u128>> | null | Uint8Array | Compact<u128> | AnyNumber, code: Bytes | string | Uint8Array, data: Bytes | string | Uint8Array, salt: Bytes | string | Uint8Array) => SubmittableExtrinsic<ApiType>, [Compact<u128>, SpWeightsWeightV2Weight, Option<Compact<u128>>, Bytes, Bytes, Bytes]>;
      /**
       * See [`Pallet::instantiate_with_code_old_weight`].
       **/
      instantiateWithCodeOldWeight: AugmentedSubmittable<(value: Compact<u128> | AnyNumber | Uint8Array, gasLimit: Compact<u64> | AnyNumber | Uint8Array, storageDepositLimit: Option<Compact<u128>> | null | Uint8Array | Compact<u128> | AnyNumber, code: Bytes | string | Uint8Array, data: Bytes | string | Uint8Array, salt: Bytes | string | Uint8Array) => SubmittableExtrinsic<ApiType>, [Compact<u128>, Compact<u64>, Option<Compact<u128>>, Bytes, Bytes, Bytes]>;
      /**
       * See [`Pallet::migrate`].
       **/
      migrate: AugmentedSubmittable<(weightLimit: SpWeightsWeightV2Weight | { refTime?: any; proofSize?: any } | string | Uint8Array) => SubmittableExtrinsic<ApiType>, [SpWeightsWeightV2Weight]>;
      /**
       * See [`Pallet::remove_code`].
       **/
      removeCode: AugmentedSubmittable<(codeHash: H256 | string | Uint8Array) => SubmittableExtrinsic<ApiType>, [H256]>;
      /**
       * See [`Pallet::set_code`].
       **/
      setCode: AugmentedSubmittable<(dest: MultiAddress | { Id: any } | { Index: any } | { Raw: any } | { Address32: any } | { Address20: any } | string | Uint8Array, codeHash: H256 | string | Uint8Array) => SubmittableExtrinsic<ApiType>, [MultiAddress, H256]>;
      /**
       * See [`Pallet::upload_code`].
       **/
      uploadCode: AugmentedSubmittable<(code: Bytes | string | Uint8Array, storageDepositLimit: Option<Compact<u128>> | null | Uint8Array | Compact<u128> | AnyNumber, determinism: PalletContractsWasmDeterminism | 'Enforced' | 'Relaxed' | number | Uint8Array) => SubmittableExtrinsic<ApiType>, [Bytes, Option<Compact<u128>>, PalletContractsWasmDeterminism]>;
      /**
       * Generic tx
       **/
      [key: string]: SubmittableExtrinsicFunction<ApiType>;
    };
    corporateAction: {
      /**
       * See [`Pallet::change_record_date`].
       **/
      changeRecordDate: AugmentedSubmittable<(caId: PalletCorporateActionsCaId | { assetId?: any; localId?: any } | string | Uint8Array, recordDate: Option<PalletCorporateActionsRecordDateSpec> | null | Uint8Array | PalletCorporateActionsRecordDateSpec | { Scheduled: any } | { ExistingSchedule: any } | { Existing: any } | string) => SubmittableExtrinsic<ApiType>, [PalletCorporateActionsCaId, Option<PalletCorporateActionsRecordDateSpec>]>;
      /**
       * See [`Pallet::initiate_corporate_action`].
       **/
      initiateCorporateAction: AugmentedSubmittable<(assetId: PolymeshPrimitivesAssetAssetId | string | Uint8Array, kind: PalletCorporateActionsCaKind | 'PredictableBenefit' | 'UnpredictableBenefit' | 'IssuerNotice' | 'Reorganization' | 'Other' | number | Uint8Array, declDate: u64 | AnyNumber | Uint8Array, recordDate: Option<PalletCorporateActionsRecordDateSpec> | null | Uint8Array | PalletCorporateActionsRecordDateSpec | { Scheduled: any } | { ExistingSchedule: any } | { Existing: any } | string, details: Bytes | string | Uint8Array, targets: Option<PalletCorporateActionsTargetIdentities> | null | Uint8Array | PalletCorporateActionsTargetIdentities | { identities?: any; treatment?: any } | string, defaultWithholdingTax: Option<Permill> | null | Uint8Array | Permill | AnyNumber, withholdingTax: Option<Vec<ITuple<[PolymeshPrimitivesIdentityId, Permill]>>> | null | Uint8Array | Vec<ITuple<[PolymeshPrimitivesIdentityId, Permill]>> | ([PolymeshPrimitivesIdentityId | string | Uint8Array, Permill | AnyNumber | Uint8Array])[]) => SubmittableExtrinsic<ApiType>, [PolymeshPrimitivesAssetAssetId, PalletCorporateActionsCaKind, u64, Option<PalletCorporateActionsRecordDateSpec>, Bytes, Option<PalletCorporateActionsTargetIdentities>, Option<Permill>, Option<Vec<ITuple<[PolymeshPrimitivesIdentityId, Permill]>>>]>;
      /**
       * See [`Pallet::initiate_corporate_action_and_ballot`].
       **/
      initiateCorporateActionAndBallot: AugmentedSubmittable<(caArgs: PalletCorporateActionsInitiateCorporateActionArgs | { assetId?: any; kind?: any; declDate?: any; recordDate?: any; details?: any; targets?: any; defaultWithholdingTax?: any; withholdingTax?: any } | string | Uint8Array, ballotTimeRange: PalletCorporateActionsBallotBallotTimeRange | { start?: any; end?: any } | string | Uint8Array, ballotMeta: PalletCorporateActionsBallotBallotMeta | { title?: any; motions?: any } | string | Uint8Array, rcv: bool | boolean | Uint8Array) => SubmittableExtrinsic<ApiType>, [PalletCorporateActionsInitiateCorporateActionArgs, PalletCorporateActionsBallotBallotTimeRange, PalletCorporateActionsBallotBallotMeta, bool]>;
      /**
       * See [`Pallet::initiate_corporate_action_and_distribute`].
       **/
      initiateCorporateActionAndDistribute: AugmentedSubmittable<(caArgs: PalletCorporateActionsInitiateCorporateActionArgs | { assetId?: any; kind?: any; declDate?: any; recordDate?: any; details?: any; targets?: any; defaultWithholdingTax?: any; withholdingTax?: any } | string | Uint8Array, portfolio: Option<u64> | null | Uint8Array | u64 | AnyNumber, currency: PolymeshPrimitivesAssetAssetId | string | Uint8Array, perShare: u128 | AnyNumber | Uint8Array, amount: u128 | AnyNumber | Uint8Array, paymentAt: u64 | AnyNumber | Uint8Array, expiresAt: Option<u64> | null | Uint8Array | u64 | AnyNumber) => SubmittableExtrinsic<ApiType>, [PalletCorporateActionsInitiateCorporateActionArgs, Option<u64>, PolymeshPrimitivesAssetAssetId, u128, u128, u64, Option<u64>]>;
      /**
       * See [`Pallet::link_ca_doc`].
       **/
      linkCaDoc: AugmentedSubmittable<(id: PalletCorporateActionsCaId | { assetId?: any; localId?: any } | string | Uint8Array, docs: Vec<u32> | (u32 | AnyNumber | Uint8Array)[]) => SubmittableExtrinsic<ApiType>, [PalletCorporateActionsCaId, Vec<u32>]>;
      /**
       * See [`Pallet::remove_ca`].
       **/
      removeCa: AugmentedSubmittable<(caId: PalletCorporateActionsCaId | { assetId?: any; localId?: any } | string | Uint8Array) => SubmittableExtrinsic<ApiType>, [PalletCorporateActionsCaId]>;
      /**
       * See [`Pallet::set_default_targets`].
       **/
      setDefaultTargets: AugmentedSubmittable<(assetId: PolymeshPrimitivesAssetAssetId | string | Uint8Array, targets: PalletCorporateActionsTargetIdentities | { identities?: any; treatment?: any } | string | Uint8Array) => SubmittableExtrinsic<ApiType>, [PolymeshPrimitivesAssetAssetId, PalletCorporateActionsTargetIdentities]>;
      /**
       * See [`Pallet::set_default_withholding_tax`].
       **/
      setDefaultWithholdingTax: AugmentedSubmittable<(assetId: PolymeshPrimitivesAssetAssetId | string | Uint8Array, tax: Permill | AnyNumber | Uint8Array) => SubmittableExtrinsic<ApiType>, [PolymeshPrimitivesAssetAssetId, Permill]>;
      /**
       * See [`Pallet::set_did_withholding_tax`].
       **/
      setDidWithholdingTax: AugmentedSubmittable<(assetId: PolymeshPrimitivesAssetAssetId | string | Uint8Array, taxedDid: PolymeshPrimitivesIdentityId | string | Uint8Array, tax: Option<Permill> | null | Uint8Array | Permill | AnyNumber) => SubmittableExtrinsic<ApiType>, [PolymeshPrimitivesAssetAssetId, PolymeshPrimitivesIdentityId, Option<Permill>]>;
      /**
       * See [`Pallet::set_max_details_length`].
       **/
      setMaxDetailsLength: AugmentedSubmittable<(length: u32 | AnyNumber | Uint8Array) => SubmittableExtrinsic<ApiType>, [u32]>;
      /**
       * Generic tx
       **/
      [key: string]: SubmittableExtrinsicFunction<ApiType>;
    };
    corporateBallot: {
      /**
       * See [`Pallet::attach_ballot`].
       **/
      attachBallot: AugmentedSubmittable<(caId: PalletCorporateActionsCaId | { assetId?: any; localId?: any } | string | Uint8Array, range: PalletCorporateActionsBallotBallotTimeRange | { start?: any; end?: any } | string | Uint8Array, meta: PalletCorporateActionsBallotBallotMeta | { title?: any; motions?: any } | string | Uint8Array, rcv: bool | boolean | Uint8Array) => SubmittableExtrinsic<ApiType>, [PalletCorporateActionsCaId, PalletCorporateActionsBallotBallotTimeRange, PalletCorporateActionsBallotBallotMeta, bool]>;
      /**
       * See [`Pallet::change_end`].
       **/
      changeEnd: AugmentedSubmittable<(caId: PalletCorporateActionsCaId | { assetId?: any; localId?: any } | string | Uint8Array, end: u64 | AnyNumber | Uint8Array) => SubmittableExtrinsic<ApiType>, [PalletCorporateActionsCaId, u64]>;
      /**
       * See [`Pallet::change_meta`].
       **/
      changeMeta: AugmentedSubmittable<(caId: PalletCorporateActionsCaId | { assetId?: any; localId?: any } | string | Uint8Array, meta: PalletCorporateActionsBallotBallotMeta | { title?: any; motions?: any } | string | Uint8Array) => SubmittableExtrinsic<ApiType>, [PalletCorporateActionsCaId, PalletCorporateActionsBallotBallotMeta]>;
      /**
       * See [`Pallet::change_rcv`].
       **/
      changeRcv: AugmentedSubmittable<(caId: PalletCorporateActionsCaId | { assetId?: any; localId?: any } | string | Uint8Array, rcv: bool | boolean | Uint8Array) => SubmittableExtrinsic<ApiType>, [PalletCorporateActionsCaId, bool]>;
      /**
       * See [`Pallet::remove_ballot`].
       **/
      removeBallot: AugmentedSubmittable<(caId: PalletCorporateActionsCaId | { assetId?: any; localId?: any } | string | Uint8Array) => SubmittableExtrinsic<ApiType>, [PalletCorporateActionsCaId]>;
      /**
       * See [`Pallet::vote`].
       **/
      vote: AugmentedSubmittable<(caId: PalletCorporateActionsCaId | { assetId?: any; localId?: any } | string | Uint8Array, votes: Vec<PalletCorporateActionsBallotBallotVote> | (PalletCorporateActionsBallotBallotVote | { power?: any; fallback?: any } | string | Uint8Array)[]) => SubmittableExtrinsic<ApiType>, [PalletCorporateActionsCaId, Vec<PalletCorporateActionsBallotBallotVote>]>;
      /**
       * Generic tx
       **/
      [key: string]: SubmittableExtrinsicFunction<ApiType>;
    };
    electionProviderMultiPhase: {
      /**
       * See [`Pallet::governance_fallback`].
       **/
      governanceFallback: AugmentedSubmittable<(maybeMaxVoters: Option<u32> | null | Uint8Array | u32 | AnyNumber, maybeMaxTargets: Option<u32> | null | Uint8Array | u32 | AnyNumber) => SubmittableExtrinsic<ApiType>, [Option<u32>, Option<u32>]>;
      /**
       * See [`Pallet::set_emergency_election_result`].
       **/
      setEmergencyElectionResult: AugmentedSubmittable<(supports: Vec<ITuple<[AccountId32, SpNposElectionsSupport]>> | ([AccountId32 | string | Uint8Array, SpNposElectionsSupport | { total?: any; voters?: any } | string | Uint8Array])[]) => SubmittableExtrinsic<ApiType>, [Vec<ITuple<[AccountId32, SpNposElectionsSupport]>>]>;
      /**
       * See [`Pallet::set_minimum_untrusted_score`].
       **/
      setMinimumUntrustedScore: AugmentedSubmittable<(maybeNextScore: Option<SpNposElectionsElectionScore> | null | Uint8Array | SpNposElectionsElectionScore | { minimalStake?: any; sumStake?: any; sumStakeSquared?: any } | string) => SubmittableExtrinsic<ApiType>, [Option<SpNposElectionsElectionScore>]>;
      /**
       * See [`Pallet::submit`].
       **/
      submit: AugmentedSubmittable<(rawSolution: PalletElectionProviderMultiPhaseRawSolution | { solution?: any; score?: any; round?: any } | string | Uint8Array) => SubmittableExtrinsic<ApiType>, [PalletElectionProviderMultiPhaseRawSolution]>;
      /**
       * See [`Pallet::submit_unsigned`].
       **/
      submitUnsigned: AugmentedSubmittable<(rawSolution: PalletElectionProviderMultiPhaseRawSolution | { solution?: any; score?: any; round?: any } | string | Uint8Array, witness: PalletElectionProviderMultiPhaseSolutionOrSnapshotSize | { voters?: any; targets?: any } | string | Uint8Array) => SubmittableExtrinsic<ApiType>, [PalletElectionProviderMultiPhaseRawSolution, PalletElectionProviderMultiPhaseSolutionOrSnapshotSize]>;
      /**
       * Generic tx
       **/
      [key: string]: SubmittableExtrinsicFunction<ApiType>;
    };
    externalAgents: {
      /**
       * See [`Pallet::abdicate`].
       **/
      abdicate: AugmentedSubmittable<(assetId: PolymeshPrimitivesAssetAssetId | string | Uint8Array) => SubmittableExtrinsic<ApiType>, [PolymeshPrimitivesAssetAssetId]>;
      /**
       * See [`Pallet::accept_become_agent`].
       **/
      acceptBecomeAgent: AugmentedSubmittable<(authId: u64 | AnyNumber | Uint8Array) => SubmittableExtrinsic<ApiType>, [u64]>;
      /**
       * See [`Pallet::change_group`].
       **/
      changeGroup: AugmentedSubmittable<(assetId: PolymeshPrimitivesAssetAssetId | string | Uint8Array, agent: PolymeshPrimitivesIdentityId | string | Uint8Array, group: PolymeshPrimitivesAgentAgentGroup | { Full: any } | { Custom: any } | { ExceptMeta: any } | { PolymeshV1CAA: any } | { PolymeshV1PIA: any } | string | Uint8Array) => SubmittableExtrinsic<ApiType>, [PolymeshPrimitivesAssetAssetId, PolymeshPrimitivesIdentityId, PolymeshPrimitivesAgentAgentGroup]>;
      /**
       * See [`Pallet::create_and_change_custom_group`].
       **/
      createAndChangeCustomGroup: AugmentedSubmittable<(assetId: PolymeshPrimitivesAssetAssetId | string | Uint8Array, perms: PolymeshPrimitivesSecondaryKeyExtrinsicPermissions | { Whole: any } | { These: any } | { Except: any } | string | Uint8Array, agent: PolymeshPrimitivesIdentityId | string | Uint8Array) => SubmittableExtrinsic<ApiType>, [PolymeshPrimitivesAssetAssetId, PolymeshPrimitivesSecondaryKeyExtrinsicPermissions, PolymeshPrimitivesIdentityId]>;
      /**
       * See [`Pallet::create_group`].
       **/
      createGroup: AugmentedSubmittable<(assetId: PolymeshPrimitivesAssetAssetId | string | Uint8Array, perms: PolymeshPrimitivesSecondaryKeyExtrinsicPermissions | { Whole: any } | { These: any } | { Except: any } | string | Uint8Array) => SubmittableExtrinsic<ApiType>, [PolymeshPrimitivesAssetAssetId, PolymeshPrimitivesSecondaryKeyExtrinsicPermissions]>;
      /**
       * See [`Pallet::create_group_and_add_auth`].
       **/
      createGroupAndAddAuth: AugmentedSubmittable<(assetId: PolymeshPrimitivesAssetAssetId | string | Uint8Array, perms: PolymeshPrimitivesSecondaryKeyExtrinsicPermissions | { Whole: any } | { These: any } | { Except: any } | string | Uint8Array, target: PolymeshPrimitivesIdentityId | string | Uint8Array, expiry: Option<u64> | null | Uint8Array | u64 | AnyNumber) => SubmittableExtrinsic<ApiType>, [PolymeshPrimitivesAssetAssetId, PolymeshPrimitivesSecondaryKeyExtrinsicPermissions, PolymeshPrimitivesIdentityId, Option<u64>]>;
      /**
       * See [`Pallet::remove_agent`].
       **/
      removeAgent: AugmentedSubmittable<(assetId: PolymeshPrimitivesAssetAssetId | string | Uint8Array, agent: PolymeshPrimitivesIdentityId | string | Uint8Array) => SubmittableExtrinsic<ApiType>, [PolymeshPrimitivesAssetAssetId, PolymeshPrimitivesIdentityId]>;
      /**
       * See [`Pallet::set_group_permissions`].
       **/
      setGroupPermissions: AugmentedSubmittable<(assetId: PolymeshPrimitivesAssetAssetId | string | Uint8Array, id: u32 | AnyNumber | Uint8Array, perms: PolymeshPrimitivesSecondaryKeyExtrinsicPermissions | { Whole: any } | { These: any } | { Except: any } | string | Uint8Array) => SubmittableExtrinsic<ApiType>, [PolymeshPrimitivesAssetAssetId, u32, PolymeshPrimitivesSecondaryKeyExtrinsicPermissions]>;
      /**
       * Generic tx
       **/
      [key: string]: SubmittableExtrinsicFunction<ApiType>;
    };
    grandpa: {
      /**
       * See [`Pallet::note_stalled`].
       **/
      noteStalled: AugmentedSubmittable<(delay: u32 | AnyNumber | Uint8Array, bestFinalizedBlockNumber: u32 | AnyNumber | Uint8Array) => SubmittableExtrinsic<ApiType>, [u32, u32]>;
      /**
       * See [`Pallet::report_equivocation`].
       **/
      reportEquivocation: AugmentedSubmittable<(equivocationProof: SpConsensusGrandpaEquivocationProof | { setId?: any; equivocation?: any } | string | Uint8Array, keyOwnerProof: SpSessionMembershipProof | { session?: any; trieNodes?: any; validatorCount?: any } | string | Uint8Array) => SubmittableExtrinsic<ApiType>, [SpConsensusGrandpaEquivocationProof, SpSessionMembershipProof]>;
      /**
       * See [`Pallet::report_equivocation_unsigned`].
       **/
      reportEquivocationUnsigned: AugmentedSubmittable<(equivocationProof: SpConsensusGrandpaEquivocationProof | { setId?: any; equivocation?: any } | string | Uint8Array, keyOwnerProof: SpSessionMembershipProof | { session?: any; trieNodes?: any; validatorCount?: any } | string | Uint8Array) => SubmittableExtrinsic<ApiType>, [SpConsensusGrandpaEquivocationProof, SpSessionMembershipProof]>;
      /**
       * Generic tx
       **/
      [key: string]: SubmittableExtrinsicFunction<ApiType>;
    };
    identity: {
      /**
       * See [`Pallet::accept_primary_key`].
       **/
      acceptPrimaryKey: AugmentedSubmittable<(rotationAuthId: u64 | AnyNumber | Uint8Array, optionalCddAuthId: Option<u64> | null | Uint8Array | u64 | AnyNumber) => SubmittableExtrinsic<ApiType>, [u64, Option<u64>]>;
      /**
       * See [`Pallet::add_authorization`].
       **/
      addAuthorization: AugmentedSubmittable<(target: PolymeshPrimitivesSecondaryKeySignatory | { Identity: any } | { Account: any } | string | Uint8Array, data: PolymeshPrimitivesAuthorizationAuthorizationData | { AttestPrimaryKeyRotation: any } | { RotatePrimaryKey: any } | { TransferTicker: any } | { AddMultiSigSigner: any } | { TransferAssetOwnership: any } | { JoinIdentity: any } | { PortfolioCustody: any } | { BecomeAgent: any } | { AddRelayerPayingKey: any } | { RotatePrimaryKeyToSecondary: any } | string | Uint8Array, expiry: Option<u64> | null | Uint8Array | u64 | AnyNumber) => SubmittableExtrinsic<ApiType>, [PolymeshPrimitivesSecondaryKeySignatory, PolymeshPrimitivesAuthorizationAuthorizationData, Option<u64>]>;
      /**
       * See [`Pallet::add_claim`].
       **/
      addClaim: AugmentedSubmittable<(target: PolymeshPrimitivesIdentityId | string | Uint8Array, claim: PolymeshPrimitivesIdentityClaimClaim | { Accredited: any } | { Affiliate: any } | { BuyLockup: any } | { SellLockup: any } | { CustomerDueDiligence: any } | { KnowYourCustomer: any } | { Jurisdiction: any } | { Exempted: any } | { Blocked: any } | { Custom: any } | string | Uint8Array, expiry: Option<u64> | null | Uint8Array | u64 | AnyNumber) => SubmittableExtrinsic<ApiType>, [PolymeshPrimitivesIdentityId, PolymeshPrimitivesIdentityClaimClaim, Option<u64>]>;
      /**
       * See [`Pallet::add_secondary_keys_with_authorization`].
       **/
      addSecondaryKeysWithAuthorization: AugmentedSubmittable<(additionalKeys: Vec<PolymeshCommonUtilitiesIdentitySecondaryKeyWithAuth> | (PolymeshCommonUtilitiesIdentitySecondaryKeyWithAuth | { secondaryKey?: any; authSignature?: any } | string | Uint8Array)[], expiresAt: u64 | AnyNumber | Uint8Array) => SubmittableExtrinsic<ApiType>, [Vec<PolymeshCommonUtilitiesIdentitySecondaryKeyWithAuth>, u64]>;
      /**
       * See [`Pallet::cdd_register_did`].
       **/
      cddRegisterDid: AugmentedSubmittable<(targetAccount: AccountId32 | string | Uint8Array, secondaryKeys: Vec<PolymeshPrimitivesSecondaryKey> | (PolymeshPrimitivesSecondaryKey | { key?: any; permissions?: any } | string | Uint8Array)[]) => SubmittableExtrinsic<ApiType>, [AccountId32, Vec<PolymeshPrimitivesSecondaryKey>]>;
      /**
       * See [`Pallet::cdd_register_did_with_cdd`].
       **/
      cddRegisterDidWithCdd: AugmentedSubmittable<(targetAccount: AccountId32 | string | Uint8Array, secondaryKeys: Vec<PolymeshPrimitivesSecondaryKey> | (PolymeshPrimitivesSecondaryKey | { key?: any; permissions?: any } | string | Uint8Array)[], expiry: Option<u64> | null | Uint8Array | u64 | AnyNumber) => SubmittableExtrinsic<ApiType>, [AccountId32, Vec<PolymeshPrimitivesSecondaryKey>, Option<u64>]>;
      /**
       * See [`Pallet::change_cdd_requirement_for_mk_rotation`].
       **/
      changeCddRequirementForMkRotation: AugmentedSubmittable<(authRequired: bool | boolean | Uint8Array) => SubmittableExtrinsic<ApiType>, [bool]>;
      /**
       * See [`Pallet::create_child_identities`].
       **/
      createChildIdentities: AugmentedSubmittable<(childKeys: Vec<PolymeshCommonUtilitiesIdentityCreateChildIdentityWithAuth> | (PolymeshCommonUtilitiesIdentityCreateChildIdentityWithAuth | { key?: any; authSignature?: any } | string | Uint8Array)[], expiresAt: u64 | AnyNumber | Uint8Array) => SubmittableExtrinsic<ApiType>, [Vec<PolymeshCommonUtilitiesIdentityCreateChildIdentityWithAuth>, u64]>;
      /**
       * See [`Pallet::create_child_identity`].
       **/
      createChildIdentity: AugmentedSubmittable<(secondaryKey: AccountId32 | string | Uint8Array) => SubmittableExtrinsic<ApiType>, [AccountId32]>;
      /**
       * See [`Pallet::freeze_secondary_keys`].
       **/
      freezeSecondaryKeys: AugmentedSubmittable<() => SubmittableExtrinsic<ApiType>, []>;
      /**
       * See [`Pallet::gc_add_cdd_claim`].
       **/
      gcAddCddClaim: AugmentedSubmittable<(target: PolymeshPrimitivesIdentityId | string | Uint8Array) => SubmittableExtrinsic<ApiType>, [PolymeshPrimitivesIdentityId]>;
      /**
       * See [`Pallet::gc_revoke_cdd_claim`].
       **/
      gcRevokeCddClaim: AugmentedSubmittable<(target: PolymeshPrimitivesIdentityId | string | Uint8Array) => SubmittableExtrinsic<ApiType>, [PolymeshPrimitivesIdentityId]>;
      /**
       * See [`Pallet::invalidate_cdd_claims`].
       **/
      invalidateCddClaims: AugmentedSubmittable<(cdd: PolymeshPrimitivesIdentityId | string | Uint8Array, disableFrom: u64 | AnyNumber | Uint8Array, expiry: Option<u64> | null | Uint8Array | u64 | AnyNumber) => SubmittableExtrinsic<ApiType>, [PolymeshPrimitivesIdentityId, u64, Option<u64>]>;
      /**
       * See [`Pallet::join_identity_as_key`].
       **/
      joinIdentityAsKey: AugmentedSubmittable<(authId: u64 | AnyNumber | Uint8Array) => SubmittableExtrinsic<ApiType>, [u64]>;
      /**
       * See [`Pallet::leave_identity_as_key`].
       **/
      leaveIdentityAsKey: AugmentedSubmittable<() => SubmittableExtrinsic<ApiType>, []>;
      /**
       * See [`Pallet::register_custom_claim_type`].
       **/
      registerCustomClaimType: AugmentedSubmittable<(ty: Bytes | string | Uint8Array) => SubmittableExtrinsic<ApiType>, [Bytes]>;
      /**
       * See [`Pallet::remove_authorization`].
       **/
      removeAuthorization: AugmentedSubmittable<(target: PolymeshPrimitivesSecondaryKeySignatory | { Identity: any } | { Account: any } | string | Uint8Array, authId: u64 | AnyNumber | Uint8Array, authIssuerPays: bool | boolean | Uint8Array) => SubmittableExtrinsic<ApiType>, [PolymeshPrimitivesSecondaryKeySignatory, u64, bool]>;
      /**
       * See [`Pallet::remove_secondary_keys`].
       **/
      removeSecondaryKeys: AugmentedSubmittable<(keysToRemove: Vec<AccountId32> | (AccountId32 | string | Uint8Array)[]) => SubmittableExtrinsic<ApiType>, [Vec<AccountId32>]>;
      /**
       * See [`Pallet::revoke_claim`].
       **/
      revokeClaim: AugmentedSubmittable<(target: PolymeshPrimitivesIdentityId | string | Uint8Array, claim: PolymeshPrimitivesIdentityClaimClaim | { Accredited: any } | { Affiliate: any } | { BuyLockup: any } | { SellLockup: any } | { CustomerDueDiligence: any } | { KnowYourCustomer: any } | { Jurisdiction: any } | { Exempted: any } | { Blocked: any } | { Custom: any } | string | Uint8Array) => SubmittableExtrinsic<ApiType>, [PolymeshPrimitivesIdentityId, PolymeshPrimitivesIdentityClaimClaim]>;
      /**
       * See [`Pallet::revoke_claim_by_index`].
       **/
      revokeClaimByIndex: AugmentedSubmittable<(target: PolymeshPrimitivesIdentityId | string | Uint8Array, claimType: PolymeshPrimitivesIdentityClaimClaimType | { Accredited: any } | { Affiliate: any } | { BuyLockup: any } | { SellLockup: any } | { CustomerDueDiligence: any } | { KnowYourCustomer: any } | { Jurisdiction: any } | { Exempted: any } | { Blocked: any } | { Custom: any } | string | Uint8Array, scope: Option<PolymeshPrimitivesIdentityClaimScope> | null | Uint8Array | PolymeshPrimitivesIdentityClaimScope | { Identity: any } | { Asset: any } | { Custom: any } | string) => SubmittableExtrinsic<ApiType>, [PolymeshPrimitivesIdentityId, PolymeshPrimitivesIdentityClaimClaimType, Option<PolymeshPrimitivesIdentityClaimScope>]>;
      /**
       * See [`Pallet::rotate_primary_key_to_secondary`].
       **/
      rotatePrimaryKeyToSecondary: AugmentedSubmittable<(authId: u64 | AnyNumber | Uint8Array, optionalCddAuthId: Option<u64> | null | Uint8Array | u64 | AnyNumber) => SubmittableExtrinsic<ApiType>, [u64, Option<u64>]>;
      /**
       * See [`Pallet::set_secondary_key_permissions`].
       **/
      setSecondaryKeyPermissions: AugmentedSubmittable<(key: AccountId32 | string | Uint8Array, perms: PolymeshPrimitivesSecondaryKeyPermissions | { asset?: any; extrinsic?: any; portfolio?: any } | string | Uint8Array) => SubmittableExtrinsic<ApiType>, [AccountId32, PolymeshPrimitivesSecondaryKeyPermissions]>;
      /**
       * See [`Pallet::unfreeze_secondary_keys`].
       **/
      unfreezeSecondaryKeys: AugmentedSubmittable<() => SubmittableExtrinsic<ApiType>, []>;
      /**
       * See [`Pallet::unlink_child_identity`].
       **/
      unlinkChildIdentity: AugmentedSubmittable<(childDid: PolymeshPrimitivesIdentityId | string | Uint8Array) => SubmittableExtrinsic<ApiType>, [PolymeshPrimitivesIdentityId]>;
      /**
       * Generic tx
       **/
      [key: string]: SubmittableExtrinsicFunction<ApiType>;
    };
    imOnline: {
      /**
       * See [`Pallet::heartbeat`].
       **/
      heartbeat: AugmentedSubmittable<(heartbeat: PalletImOnlineHeartbeat | { blockNumber?: any; sessionIndex?: any; authorityIndex?: any; validatorsLen?: any } | string | Uint8Array, signature: PalletImOnlineSr25519AppSr25519Signature | string | Uint8Array) => SubmittableExtrinsic<ApiType>, [PalletImOnlineHeartbeat, PalletImOnlineSr25519AppSr25519Signature]>;
      /**
       * Generic tx
       **/
      [key: string]: SubmittableExtrinsicFunction<ApiType>;
    };
    indices: {
      /**
       * See [`Pallet::claim`].
       **/
      claim: AugmentedSubmittable<(index: u32 | AnyNumber | Uint8Array) => SubmittableExtrinsic<ApiType>, [u32]>;
      /**
       * See [`Pallet::force_transfer`].
       **/
      forceTransfer: AugmentedSubmittable<(updated: MultiAddress | { Id: any } | { Index: any } | { Raw: any } | { Address32: any } | { Address20: any } | string | Uint8Array, index: u32 | AnyNumber | Uint8Array, freeze: bool | boolean | Uint8Array) => SubmittableExtrinsic<ApiType>, [MultiAddress, u32, bool]>;
      /**
       * See [`Pallet::free`].
       **/
      free: AugmentedSubmittable<(index: u32 | AnyNumber | Uint8Array) => SubmittableExtrinsic<ApiType>, [u32]>;
      /**
       * See [`Pallet::freeze`].
       **/
      freeze: AugmentedSubmittable<(index: u32 | AnyNumber | Uint8Array) => SubmittableExtrinsic<ApiType>, [u32]>;
      /**
       * See [`Pallet::transfer`].
       **/
      transfer: AugmentedSubmittable<(updated: MultiAddress | { Id: any } | { Index: any } | { Raw: any } | { Address32: any } | { Address20: any } | string | Uint8Array, index: u32 | AnyNumber | Uint8Array) => SubmittableExtrinsic<ApiType>, [MultiAddress, u32]>;
      /**
       * Generic tx
       **/
      [key: string]: SubmittableExtrinsicFunction<ApiType>;
    };
    multiSig: {
      /**
       * See [`Pallet::accept_multisig_signer`].
       **/
      acceptMultisigSigner: AugmentedSubmittable<(authId: u64 | AnyNumber | Uint8Array) => SubmittableExtrinsic<ApiType>, [u64]>;
      /**
       * See [`Pallet::add_admin`].
       **/
      addAdmin: AugmentedSubmittable<(adminDid: PolymeshPrimitivesIdentityId | string | Uint8Array) => SubmittableExtrinsic<ApiType>, [PolymeshPrimitivesIdentityId]>;
      /**
       * See [`Pallet::add_multisig_signers`].
       **/
      addMultisigSigners: AugmentedSubmittable<(signers: Vec<AccountId32> | (AccountId32 | string | Uint8Array)[]) => SubmittableExtrinsic<ApiType>, [Vec<AccountId32>]>;
      /**
       * See [`Pallet::add_multisig_signers_via_admin`].
       **/
      addMultisigSignersViaAdmin: AugmentedSubmittable<(multisig: AccountId32 | string | Uint8Array, signers: Vec<AccountId32> | (AccountId32 | string | Uint8Array)[]) => SubmittableExtrinsic<ApiType>, [AccountId32, Vec<AccountId32>]>;
      /**
       * See [`Pallet::approve`].
       **/
      approve: AugmentedSubmittable<(multisig: AccountId32 | string | Uint8Array, proposalId: u64 | AnyNumber | Uint8Array, maxWeight: Option<SpWeightsWeightV2Weight> | null | Uint8Array | SpWeightsWeightV2Weight | { refTime?: any; proofSize?: any } | string) => SubmittableExtrinsic<ApiType>, [AccountId32, u64, Option<SpWeightsWeightV2Weight>]>;
      /**
       * See [`Pallet::approve_join_identity`].
       **/
      approveJoinIdentity: AugmentedSubmittable<(multisig: AccountId32 | string | Uint8Array, authId: u64 | AnyNumber | Uint8Array) => SubmittableExtrinsic<ApiType>, [AccountId32, u64]>;
      /**
       * See [`Pallet::change_sigs_required`].
       **/
      changeSigsRequired: AugmentedSubmittable<(sigsRequired: u64 | AnyNumber | Uint8Array) => SubmittableExtrinsic<ApiType>, [u64]>;
      /**
       * See [`Pallet::change_sigs_required_via_admin`].
       **/
      changeSigsRequiredViaAdmin: AugmentedSubmittable<(multisig: AccountId32 | string | Uint8Array, signaturesRequired: u64 | AnyNumber | Uint8Array) => SubmittableExtrinsic<ApiType>, [AccountId32, u64]>;
      /**
       * See [`Pallet::create_multisig`].
       **/
      createMultisig: AugmentedSubmittable<(signers: Vec<AccountId32> | (AccountId32 | string | Uint8Array)[], sigsRequired: u64 | AnyNumber | Uint8Array, permissions: Option<PolymeshPrimitivesSecondaryKeyPermissions> | null | Uint8Array | PolymeshPrimitivesSecondaryKeyPermissions | { asset?: any; extrinsic?: any; portfolio?: any } | string) => SubmittableExtrinsic<ApiType>, [Vec<AccountId32>, u64, Option<PolymeshPrimitivesSecondaryKeyPermissions>]>;
      /**
       * See [`Pallet::create_proposal`].
       **/
      createProposal: AugmentedSubmittable<(multisig: AccountId32 | string | Uint8Array, proposal: Call | IMethod | string | Uint8Array, expiry: Option<u64> | null | Uint8Array | u64 | AnyNumber) => SubmittableExtrinsic<ApiType>, [AccountId32, Call, Option<u64>]>;
      /**
       * See [`Pallet::join_identity`].
       **/
      joinIdentity: AugmentedSubmittable<(authId: u64 | AnyNumber | Uint8Array) => SubmittableExtrinsic<ApiType>, [u64]>;
      /**
       * See [`Pallet::reject`].
       **/
      reject: AugmentedSubmittable<(multisig: AccountId32 | string | Uint8Array, proposalId: u64 | AnyNumber | Uint8Array) => SubmittableExtrinsic<ApiType>, [AccountId32, u64]>;
      /**
       * See [`Pallet::remove_admin`].
       **/
      removeAdmin: AugmentedSubmittable<() => SubmittableExtrinsic<ApiType>, []>;
      /**
       * See [`Pallet::remove_admin_via_admin`].
       **/
      removeAdminViaAdmin: AugmentedSubmittable<(multisig: AccountId32 | string | Uint8Array) => SubmittableExtrinsic<ApiType>, [AccountId32]>;
      /**
       * See [`Pallet::remove_multisig_signers`].
       **/
      removeMultisigSigners: AugmentedSubmittable<(signers: Vec<AccountId32> | (AccountId32 | string | Uint8Array)[]) => SubmittableExtrinsic<ApiType>, [Vec<AccountId32>]>;
      /**
       * See [`Pallet::remove_multisig_signers_via_admin`].
       **/
      removeMultisigSignersViaAdmin: AugmentedSubmittable<(multisig: AccountId32 | string | Uint8Array, signers: Vec<AccountId32> | (AccountId32 | string | Uint8Array)[]) => SubmittableExtrinsic<ApiType>, [AccountId32, Vec<AccountId32>]>;
      /**
       * See [`Pallet::remove_payer`].
       **/
      removePayer: AugmentedSubmittable<() => SubmittableExtrinsic<ApiType>, []>;
      /**
       * See [`Pallet::remove_payer_via_payer`].
       **/
      removePayerViaPayer: AugmentedSubmittable<(multisig: AccountId32 | string | Uint8Array) => SubmittableExtrinsic<ApiType>, [AccountId32]>;
      /**
       * Generic tx
       **/
      [key: string]: SubmittableExtrinsicFunction<ApiType>;
    };
    nft: {
      /**
       * See [`Pallet::controller_transfer`].
       **/
      controllerTransfer: AugmentedSubmittable<(nfts: PolymeshPrimitivesNftNfTs | { assetId?: any; ids?: any } | string | Uint8Array, sourcePortfolio: PolymeshPrimitivesIdentityIdPortfolioId | { did?: any; kind?: any } | string | Uint8Array, callersPortfolioKind: PolymeshPrimitivesIdentityIdPortfolioKind | { Default: any } | { User: any } | string | Uint8Array) => SubmittableExtrinsic<ApiType>, [PolymeshPrimitivesNftNfTs, PolymeshPrimitivesIdentityIdPortfolioId, PolymeshPrimitivesIdentityIdPortfolioKind]>;
      /**
       * See [`Pallet::create_nft_collection`].
       **/
      createNftCollection: AugmentedSubmittable<(assetId: Option<PolymeshPrimitivesAssetAssetId> | null | Uint8Array | PolymeshPrimitivesAssetAssetId | string, nftType: Option<PolymeshPrimitivesAssetNonFungibleType> | null | Uint8Array | PolymeshPrimitivesAssetNonFungibleType | { Derivative: any } | { FixedIncome: any } | { Invoice: any } | { Custom: any } | string, collectionKeys: PolymeshPrimitivesNftNftCollectionKeys) => SubmittableExtrinsic<ApiType>, [Option<PolymeshPrimitivesAssetAssetId>, Option<PolymeshPrimitivesAssetNonFungibleType>, PolymeshPrimitivesNftNftCollectionKeys]>;
      /**
       * See [`Pallet::issue_nft`].
       **/
      issueNft: AugmentedSubmittable<(assetId: PolymeshPrimitivesAssetAssetId | string | Uint8Array, nftMetadataAttributes: Vec<PolymeshPrimitivesNftNftMetadataAttribute> | (PolymeshPrimitivesNftNftMetadataAttribute | { key?: any; value?: any } | string | Uint8Array)[], portfolioKind: PolymeshPrimitivesIdentityIdPortfolioKind | { Default: any } | { User: any } | string | Uint8Array) => SubmittableExtrinsic<ApiType>, [PolymeshPrimitivesAssetAssetId, Vec<PolymeshPrimitivesNftNftMetadataAttribute>, PolymeshPrimitivesIdentityIdPortfolioKind]>;
      /**
       * See [`Pallet::redeem_nft`].
       **/
      redeemNft: AugmentedSubmittable<(assetId: PolymeshPrimitivesAssetAssetId | string | Uint8Array, nftId: u64 | AnyNumber | Uint8Array, portfolioKind: PolymeshPrimitivesIdentityIdPortfolioKind | { Default: any } | { User: any } | string | Uint8Array, numberOfKeys: Option<u8> | null | Uint8Array | u8 | AnyNumber) => SubmittableExtrinsic<ApiType>, [PolymeshPrimitivesAssetAssetId, u64, PolymeshPrimitivesIdentityIdPortfolioKind, Option<u8>]>;
      /**
       * Generic tx
       **/
      [key: string]: SubmittableExtrinsicFunction<ApiType>;
    };
    pips: {
      /**
       * See [`Pallet::approve_committee_proposal`].
       **/
      approveCommitteeProposal: AugmentedSubmittable<(id: u32 | AnyNumber | Uint8Array) => SubmittableExtrinsic<ApiType>, [u32]>;
      /**
       * See [`Pallet::clear_snapshot`].
       **/
      clearSnapshot: AugmentedSubmittable<() => SubmittableExtrinsic<ApiType>, []>;
      /**
       * See [`Pallet::enact_snapshot_results`].
       **/
      enactSnapshotResults: AugmentedSubmittable<(results: Vec<ITuple<[u32, PalletPipsSnapshotResult]>> | ([u32 | AnyNumber | Uint8Array, PalletPipsSnapshotResult | 'Approve' | 'Reject' | 'Skip' | number | Uint8Array])[]) => SubmittableExtrinsic<ApiType>, [Vec<ITuple<[u32, PalletPipsSnapshotResult]>>]>;
      /**
       * See [`Pallet::execute_scheduled_pip`].
       **/
      executeScheduledPip: AugmentedSubmittable<(id: u32 | AnyNumber | Uint8Array) => SubmittableExtrinsic<ApiType>, [u32]>;
      /**
       * See [`Pallet::expire_scheduled_pip`].
       **/
      expireScheduledPip: AugmentedSubmittable<(did: PolymeshPrimitivesIdentityId | string | Uint8Array, id: u32 | AnyNumber | Uint8Array) => SubmittableExtrinsic<ApiType>, [PolymeshPrimitivesIdentityId, u32]>;
      /**
       * See [`Pallet::propose`].
       **/
      propose: AugmentedSubmittable<(proposal: Call | IMethod | string | Uint8Array, deposit: u128 | AnyNumber | Uint8Array, url: Option<Bytes> | null | Uint8Array | Bytes | string, description: Option<Bytes> | null | Uint8Array | Bytes | string) => SubmittableExtrinsic<ApiType>, [Call, u128, Option<Bytes>, Option<Bytes>]>;
      /**
       * See [`Pallet::prune_proposal`].
       **/
      pruneProposal: AugmentedSubmittable<(id: u32 | AnyNumber | Uint8Array) => SubmittableExtrinsic<ApiType>, [u32]>;
      /**
       * See [`Pallet::reject_proposal`].
       **/
      rejectProposal: AugmentedSubmittable<(id: u32 | AnyNumber | Uint8Array) => SubmittableExtrinsic<ApiType>, [u32]>;
      /**
       * See [`Pallet::reschedule_execution`].
       **/
      rescheduleExecution: AugmentedSubmittable<(id: u32 | AnyNumber | Uint8Array, until: Option<u32> | null | Uint8Array | u32 | AnyNumber) => SubmittableExtrinsic<ApiType>, [u32, Option<u32>]>;
      /**
       * See [`Pallet::set_active_pip_limit`].
       **/
      setActivePipLimit: AugmentedSubmittable<(limit: u32 | AnyNumber | Uint8Array) => SubmittableExtrinsic<ApiType>, [u32]>;
      /**
       * See [`Pallet::set_default_enactment_period`].
       **/
      setDefaultEnactmentPeriod: AugmentedSubmittable<(duration: u32 | AnyNumber | Uint8Array) => SubmittableExtrinsic<ApiType>, [u32]>;
      /**
       * See [`Pallet::set_max_pip_skip_count`].
       **/
      setMaxPipSkipCount: AugmentedSubmittable<(max: u8 | AnyNumber | Uint8Array) => SubmittableExtrinsic<ApiType>, [u8]>;
      /**
       * See [`Pallet::set_min_proposal_deposit`].
       **/
      setMinProposalDeposit: AugmentedSubmittable<(deposit: u128 | AnyNumber | Uint8Array) => SubmittableExtrinsic<ApiType>, [u128]>;
      /**
       * See [`Pallet::set_pending_pip_expiry`].
       **/
      setPendingPipExpiry: AugmentedSubmittable<(expiry: PolymeshPrimitivesMaybeBlock | { Some: any } | { None: any } | string | Uint8Array) => SubmittableExtrinsic<ApiType>, [PolymeshPrimitivesMaybeBlock]>;
      /**
       * See [`Pallet::set_prune_historical_pips`].
       **/
      setPruneHistoricalPips: AugmentedSubmittable<(prune: bool | boolean | Uint8Array) => SubmittableExtrinsic<ApiType>, [bool]>;
      /**
       * See [`Pallet::snapshot`].
       **/
      snapshot: AugmentedSubmittable<() => SubmittableExtrinsic<ApiType>, []>;
      /**
       * See [`Pallet::vote`].
       **/
      vote: AugmentedSubmittable<(id: u32 | AnyNumber | Uint8Array, ayeOrNay: bool | boolean | Uint8Array, deposit: u128 | AnyNumber | Uint8Array) => SubmittableExtrinsic<ApiType>, [u32, bool, u128]>;
      /**
       * Generic tx
       **/
      [key: string]: SubmittableExtrinsicFunction<ApiType>;
    };
    polymeshCommittee: {
      /**
       * See [`Pallet::set_expires_after`].
       **/
      setExpiresAfter: AugmentedSubmittable<(expiry: PolymeshPrimitivesMaybeBlock | { Some: any } | { None: any } | string | Uint8Array) => SubmittableExtrinsic<ApiType>, [PolymeshPrimitivesMaybeBlock]>;
      /**
       * See [`Pallet::set_release_coordinator`].
       **/
      setReleaseCoordinator: AugmentedSubmittable<(id: PolymeshPrimitivesIdentityId | string | Uint8Array) => SubmittableExtrinsic<ApiType>, [PolymeshPrimitivesIdentityId]>;
      /**
       * See [`Pallet::set_vote_threshold`].
       **/
      setVoteThreshold: AugmentedSubmittable<(n: u32 | AnyNumber | Uint8Array, d: u32 | AnyNumber | Uint8Array) => SubmittableExtrinsic<ApiType>, [u32, u32]>;
      /**
       * See [`Pallet::vote`].
       **/
      vote: AugmentedSubmittable<(proposal: H256 | string | Uint8Array, index: u32 | AnyNumber | Uint8Array, approve: bool | boolean | Uint8Array) => SubmittableExtrinsic<ApiType>, [H256, u32, bool]>;
      /**
       * See [`Pallet::vote_or_propose`].
       **/
      voteOrPropose: AugmentedSubmittable<(approve: bool | boolean | Uint8Array, call: Call | IMethod | string | Uint8Array) => SubmittableExtrinsic<ApiType>, [bool, Call]>;
      /**
       * Generic tx
       **/
      [key: string]: SubmittableExtrinsicFunction<ApiType>;
    };
    polymeshContracts: {
      /**
       * See [`Pallet::instantiate_with_code_as_primary_key`].
       **/
      instantiateWithCodeAsPrimaryKey: AugmentedSubmittable<(endowment: u128 | AnyNumber | Uint8Array, gasLimit: SpWeightsWeightV2Weight | { refTime?: any; proofSize?: any } | string | Uint8Array, storageDepositLimit: Option<u128> | null | Uint8Array | u128 | AnyNumber, code: Bytes | string | Uint8Array, data: Bytes | string | Uint8Array, salt: Bytes | string | Uint8Array) => SubmittableExtrinsic<ApiType>, [u128, SpWeightsWeightV2Weight, Option<u128>, Bytes, Bytes, Bytes]>;
      /**
       * See [`Pallet::instantiate_with_code_perms`].
       **/
      instantiateWithCodePerms: AugmentedSubmittable<(endowment: u128 | AnyNumber | Uint8Array, gasLimit: SpWeightsWeightV2Weight | { refTime?: any; proofSize?: any } | string | Uint8Array, storageDepositLimit: Option<u128> | null | Uint8Array | u128 | AnyNumber, code: Bytes | string | Uint8Array, data: Bytes | string | Uint8Array, salt: Bytes | string | Uint8Array, perms: PolymeshPrimitivesSecondaryKeyPermissions | { asset?: any; extrinsic?: any; portfolio?: any } | string | Uint8Array) => SubmittableExtrinsic<ApiType>, [u128, SpWeightsWeightV2Weight, Option<u128>, Bytes, Bytes, Bytes, PolymeshPrimitivesSecondaryKeyPermissions]>;
      /**
       * See [`Pallet::instantiate_with_hash_as_primary_key`].
       **/
      instantiateWithHashAsPrimaryKey: AugmentedSubmittable<(endowment: u128 | AnyNumber | Uint8Array, gasLimit: SpWeightsWeightV2Weight | { refTime?: any; proofSize?: any } | string | Uint8Array, storageDepositLimit: Option<u128> | null | Uint8Array | u128 | AnyNumber, codeHash: H256 | string | Uint8Array, data: Bytes | string | Uint8Array, salt: Bytes | string | Uint8Array) => SubmittableExtrinsic<ApiType>, [u128, SpWeightsWeightV2Weight, Option<u128>, H256, Bytes, Bytes]>;
      /**
       * See [`Pallet::instantiate_with_hash_perms`].
       **/
      instantiateWithHashPerms: AugmentedSubmittable<(endowment: u128 | AnyNumber | Uint8Array, gasLimit: SpWeightsWeightV2Weight | { refTime?: any; proofSize?: any } | string | Uint8Array, storageDepositLimit: Option<u128> | null | Uint8Array | u128 | AnyNumber, codeHash: H256 | string | Uint8Array, data: Bytes | string | Uint8Array, salt: Bytes | string | Uint8Array, perms: PolymeshPrimitivesSecondaryKeyPermissions | { asset?: any; extrinsic?: any; portfolio?: any } | string | Uint8Array) => SubmittableExtrinsic<ApiType>, [u128, SpWeightsWeightV2Weight, Option<u128>, H256, Bytes, Bytes, PolymeshPrimitivesSecondaryKeyPermissions]>;
      /**
       * See [`Pallet::update_call_runtime_whitelist`].
       **/
      updateCallRuntimeWhitelist: AugmentedSubmittable<(updates: Vec<ITuple<[PolymeshContractsChainExtensionExtrinsicId, bool]>> | ([PolymeshContractsChainExtensionExtrinsicId, bool | boolean | Uint8Array])[]) => SubmittableExtrinsic<ApiType>, [Vec<ITuple<[PolymeshContractsChainExtensionExtrinsicId, bool]>>]>;
      /**
       * See [`Pallet::upgrade_api`].
       **/
      upgradeApi: AugmentedSubmittable<(api: PolymeshContractsApi | { desc?: any; major?: any } | string | Uint8Array, nextUpgrade: PolymeshContractsNextUpgrade | { chainVersion?: any; apiHash?: any } | string | Uint8Array) => SubmittableExtrinsic<ApiType>, [PolymeshContractsApi, PolymeshContractsNextUpgrade]>;
      /**
       * Generic tx
       **/
      [key: string]: SubmittableExtrinsicFunction<ApiType>;
    };
    portfolio: {
      /**
       * See [`Pallet::accept_portfolio_custody`].
       **/
      acceptPortfolioCustody: AugmentedSubmittable<(authId: u64 | AnyNumber | Uint8Array) => SubmittableExtrinsic<ApiType>, [u64]>;
      /**
       * See [`Pallet::allow_identity_to_create_portfolios`].
       **/
      allowIdentityToCreatePortfolios: AugmentedSubmittable<(trustedIdentity: PolymeshPrimitivesIdentityId | string | Uint8Array) => SubmittableExtrinsic<ApiType>, [PolymeshPrimitivesIdentityId]>;
      /**
       * See [`Pallet::create_custody_portfolio`].
       **/
      createCustodyPortfolio: AugmentedSubmittable<(portfolioOwnerId: PolymeshPrimitivesIdentityId | string | Uint8Array, portfolioName: Bytes | string | Uint8Array) => SubmittableExtrinsic<ApiType>, [PolymeshPrimitivesIdentityId, Bytes]>;
      /**
       * See [`Pallet::create_portfolio`].
       **/
      createPortfolio: AugmentedSubmittable<(name: Bytes | string | Uint8Array) => SubmittableExtrinsic<ApiType>, [Bytes]>;
      /**
       * See [`Pallet::delete_portfolio`].
       **/
      deletePortfolio: AugmentedSubmittable<(num: u64 | AnyNumber | Uint8Array) => SubmittableExtrinsic<ApiType>, [u64]>;
      /**
       * See [`Pallet::move_portfolio_funds`].
       **/
      movePortfolioFunds: AugmentedSubmittable<(from: PolymeshPrimitivesIdentityIdPortfolioId | { did?: any; kind?: any } | string | Uint8Array, to: PolymeshPrimitivesIdentityIdPortfolioId | { did?: any; kind?: any } | string | Uint8Array, funds: Vec<PolymeshPrimitivesPortfolioFund> | (PolymeshPrimitivesPortfolioFund | { description?: any; memo?: any } | string | Uint8Array)[]) => SubmittableExtrinsic<ApiType>, [PolymeshPrimitivesIdentityIdPortfolioId, PolymeshPrimitivesIdentityIdPortfolioId, Vec<PolymeshPrimitivesPortfolioFund>]>;
      /**
       * See [`Pallet::pre_approve_portfolio`].
       **/
      preApprovePortfolio: AugmentedSubmittable<(assetId: PolymeshPrimitivesAssetAssetId | string | Uint8Array, portfolioId: PolymeshPrimitivesIdentityIdPortfolioId | { did?: any; kind?: any } | string | Uint8Array) => SubmittableExtrinsic<ApiType>, [PolymeshPrimitivesAssetAssetId, PolymeshPrimitivesIdentityIdPortfolioId]>;
      /**
       * See [`Pallet::quit_portfolio_custody`].
       **/
      quitPortfolioCustody: AugmentedSubmittable<(pid: PolymeshPrimitivesIdentityIdPortfolioId | { did?: any; kind?: any } | string | Uint8Array) => SubmittableExtrinsic<ApiType>, [PolymeshPrimitivesIdentityIdPortfolioId]>;
      /**
       * See [`Pallet::remove_portfolio_pre_approval`].
       **/
      removePortfolioPreApproval: AugmentedSubmittable<(assetId: PolymeshPrimitivesAssetAssetId | string | Uint8Array, portfolioId: PolymeshPrimitivesIdentityIdPortfolioId | { did?: any; kind?: any } | string | Uint8Array) => SubmittableExtrinsic<ApiType>, [PolymeshPrimitivesAssetAssetId, PolymeshPrimitivesIdentityIdPortfolioId]>;
      /**
       * See [`Pallet::rename_portfolio`].
       **/
      renamePortfolio: AugmentedSubmittable<(num: u64 | AnyNumber | Uint8Array, toName: Bytes | string | Uint8Array) => SubmittableExtrinsic<ApiType>, [u64, Bytes]>;
      /**
       * See [`Pallet::revoke_create_portfolios_permission`].
       **/
      revokeCreatePortfoliosPermission: AugmentedSubmittable<(identity: PolymeshPrimitivesIdentityId | string | Uint8Array) => SubmittableExtrinsic<ApiType>, [PolymeshPrimitivesIdentityId]>;
      /**
       * Generic tx
       **/
      [key: string]: SubmittableExtrinsicFunction<ApiType>;
    };
    preimage: {
      /**
       * See [`Pallet::note_preimage`].
       **/
      notePreimage: AugmentedSubmittable<(bytes: Bytes | string | Uint8Array) => SubmittableExtrinsic<ApiType>, [Bytes]>;
      /**
       * See [`Pallet::request_preimage`].
       **/
      requestPreimage: AugmentedSubmittable<(hash: H256 | string | Uint8Array) => SubmittableExtrinsic<ApiType>, [H256]>;
      /**
       * See [`Pallet::unnote_preimage`].
       **/
      unnotePreimage: AugmentedSubmittable<(hash: H256 | string | Uint8Array) => SubmittableExtrinsic<ApiType>, [H256]>;
      /**
       * See [`Pallet::unrequest_preimage`].
       **/
      unrequestPreimage: AugmentedSubmittable<(hash: H256 | string | Uint8Array) => SubmittableExtrinsic<ApiType>, [H256]>;
      /**
       * Generic tx
       **/
      [key: string]: SubmittableExtrinsicFunction<ApiType>;
    };
    protocolFee: {
      /**
       * See [`Pallet::change_base_fee`].
       **/
      changeBaseFee: AugmentedSubmittable<(op: PolymeshCommonUtilitiesProtocolFeeProtocolOp | 'AssetRegisterTicker' | 'AssetIssue' | 'AssetAddDocuments' | 'AssetCreateAsset' | 'CheckpointCreateSchedule' | 'ComplianceManagerAddComplianceRequirement' | 'IdentityCddRegisterDid' | 'IdentityAddClaim' | 'IdentityAddSecondaryKeysWithAuthorization' | 'PipsPropose' | 'ContractsPutCode' | 'CorporateBallotAttachBallot' | 'CapitalDistributionDistribute' | 'NFTCreateCollection' | 'NFTMint' | 'IdentityCreateChildIdentity' | number | Uint8Array, baseFee: u128 | AnyNumber | Uint8Array) => SubmittableExtrinsic<ApiType>, [PolymeshCommonUtilitiesProtocolFeeProtocolOp, u128]>;
      /**
       * See [`Pallet::change_coefficient`].
       **/
      changeCoefficient: AugmentedSubmittable<(coefficient: PolymeshPrimitivesPosRatio) => SubmittableExtrinsic<ApiType>, [PolymeshPrimitivesPosRatio]>;
      /**
       * Generic tx
       **/
      [key: string]: SubmittableExtrinsicFunction<ApiType>;
    };
    relayer: {
      /**
       * See [`Pallet::accept_paying_key`].
       **/
      acceptPayingKey: AugmentedSubmittable<(authId: u64 | AnyNumber | Uint8Array) => SubmittableExtrinsic<ApiType>, [u64]>;
      /**
       * See [`Pallet::decrease_polyx_limit`].
       **/
      decreasePolyxLimit: AugmentedSubmittable<(userKey: AccountId32 | string | Uint8Array, amount: u128 | AnyNumber | Uint8Array) => SubmittableExtrinsic<ApiType>, [AccountId32, u128]>;
      /**
       * See [`Pallet::increase_polyx_limit`].
       **/
      increasePolyxLimit: AugmentedSubmittable<(userKey: AccountId32 | string | Uint8Array, amount: u128 | AnyNumber | Uint8Array) => SubmittableExtrinsic<ApiType>, [AccountId32, u128]>;
      /**
       * See [`Pallet::remove_paying_key`].
       **/
      removePayingKey: AugmentedSubmittable<(userKey: AccountId32 | string | Uint8Array, payingKey: AccountId32 | string | Uint8Array) => SubmittableExtrinsic<ApiType>, [AccountId32, AccountId32]>;
      /**
       * See [`Pallet::set_paying_key`].
       **/
      setPayingKey: AugmentedSubmittable<(userKey: AccountId32 | string | Uint8Array, polyxLimit: u128 | AnyNumber | Uint8Array) => SubmittableExtrinsic<ApiType>, [AccountId32, u128]>;
      /**
       * See [`Pallet::update_polyx_limit`].
       **/
      updatePolyxLimit: AugmentedSubmittable<(userKey: AccountId32 | string | Uint8Array, polyxLimit: u128 | AnyNumber | Uint8Array) => SubmittableExtrinsic<ApiType>, [AccountId32, u128]>;
      /**
       * Generic tx
       **/
      [key: string]: SubmittableExtrinsicFunction<ApiType>;
    };
    scheduler: {
      /**
       * See [`Pallet::cancel`].
       **/
      cancel: AugmentedSubmittable<(when: u32 | AnyNumber | Uint8Array, index: u32 | AnyNumber | Uint8Array) => SubmittableExtrinsic<ApiType>, [u32, u32]>;
      /**
       * See [`Pallet::cancel_named`].
       **/
      cancelNamed: AugmentedSubmittable<(id: U8aFixed | string | Uint8Array) => SubmittableExtrinsic<ApiType>, [U8aFixed]>;
      /**
       * See [`Pallet::schedule`].
       **/
      schedule: AugmentedSubmittable<(when: u32 | AnyNumber | Uint8Array, maybePeriodic: Option<ITuple<[u32, u32]>> | null | Uint8Array | ITuple<[u32, u32]> | [u32 | AnyNumber | Uint8Array, u32 | AnyNumber | Uint8Array], priority: u8 | AnyNumber | Uint8Array, call: Call | IMethod | string | Uint8Array) => SubmittableExtrinsic<ApiType>, [u32, Option<ITuple<[u32, u32]>>, u8, Call]>;
      /**
       * See [`Pallet::schedule_after`].
       **/
      scheduleAfter: AugmentedSubmittable<(after: u32 | AnyNumber | Uint8Array, maybePeriodic: Option<ITuple<[u32, u32]>> | null | Uint8Array | ITuple<[u32, u32]> | [u32 | AnyNumber | Uint8Array, u32 | AnyNumber | Uint8Array], priority: u8 | AnyNumber | Uint8Array, call: Call | IMethod | string | Uint8Array) => SubmittableExtrinsic<ApiType>, [u32, Option<ITuple<[u32, u32]>>, u8, Call]>;
      /**
       * See [`Pallet::schedule_named`].
       **/
      scheduleNamed: AugmentedSubmittable<(id: U8aFixed | string | Uint8Array, when: u32 | AnyNumber | Uint8Array, maybePeriodic: Option<ITuple<[u32, u32]>> | null | Uint8Array | ITuple<[u32, u32]> | [u32 | AnyNumber | Uint8Array, u32 | AnyNumber | Uint8Array], priority: u8 | AnyNumber | Uint8Array, call: Call | IMethod | string | Uint8Array) => SubmittableExtrinsic<ApiType>, [U8aFixed, u32, Option<ITuple<[u32, u32]>>, u8, Call]>;
      /**
       * See [`Pallet::schedule_named_after`].
       **/
      scheduleNamedAfter: AugmentedSubmittable<(id: U8aFixed | string | Uint8Array, after: u32 | AnyNumber | Uint8Array, maybePeriodic: Option<ITuple<[u32, u32]>> | null | Uint8Array | ITuple<[u32, u32]> | [u32 | AnyNumber | Uint8Array, u32 | AnyNumber | Uint8Array], priority: u8 | AnyNumber | Uint8Array, call: Call | IMethod | string | Uint8Array) => SubmittableExtrinsic<ApiType>, [U8aFixed, u32, Option<ITuple<[u32, u32]>>, u8, Call]>;
      /**
       * Generic tx
       **/
      [key: string]: SubmittableExtrinsicFunction<ApiType>;
    };
    session: {
      /**
       * See [`Pallet::purge_keys`].
       **/
      purgeKeys: AugmentedSubmittable<() => SubmittableExtrinsic<ApiType>, []>;
      /**
       * See [`Pallet::set_keys`].
       **/
      setKeys: AugmentedSubmittable<(keys: PolymeshRuntimeDevelopRuntimeSessionKeys | { grandpa?: any; babe?: any; imOnline?: any; authorityDiscovery?: any } | string | Uint8Array, proof: Bytes | string | Uint8Array) => SubmittableExtrinsic<ApiType>, [PolymeshRuntimeDevelopRuntimeSessionKeys, Bytes]>;
      /**
       * Generic tx
       **/
      [key: string]: SubmittableExtrinsicFunction<ApiType>;
    };
    settlement: {
      /**
       * See [`Pallet::add_and_affirm_instruction`].
       **/
      addAndAffirmInstruction: AugmentedSubmittable<(venueId: Option<u64> | null | Uint8Array | u64 | AnyNumber, settlementType: PolymeshPrimitivesSettlementSettlementType | { SettleOnAffirmation: any } | { SettleOnBlock: any } | { SettleManual: any } | { SettleAfterLock: any } | string | Uint8Array, tradeDate: Option<u64> | null | Uint8Array | u64 | AnyNumber, valueDate: Option<u64> | null | Uint8Array | u64 | AnyNumber, legs: Vec<PolymeshPrimitivesSettlementLeg> | (PolymeshPrimitivesSettlementLeg | { Fungible: any } | { NonFungible: any } | { OffChain: any } | string | Uint8Array)[], portfolios: BTreeSet<PolymeshPrimitivesIdentityIdPortfolioId>, instructionMemo: Option<PolymeshPrimitivesMemo> | null | Uint8Array | PolymeshPrimitivesMemo | string) => SubmittableExtrinsic<ApiType>, [Option<u64>, PolymeshPrimitivesSettlementSettlementType, Option<u64>, Option<u64>, Vec<PolymeshPrimitivesSettlementLeg>, BTreeSet<PolymeshPrimitivesIdentityIdPortfolioId>, Option<PolymeshPrimitivesMemo>]>;
      /**
       * See [`Pallet::add_and_affirm_with_mediators`].
       **/
      addAndAffirmWithMediators: AugmentedSubmittable<(venueId: Option<u64> | null | Uint8Array | u64 | AnyNumber, settlementType: PolymeshPrimitivesSettlementSettlementType | { SettleOnAffirmation: any } | { SettleOnBlock: any } | { SettleManual: any } | { SettleAfterLock: any } | string | Uint8Array, tradeDate: Option<u64> | null | Uint8Array | u64 | AnyNumber, valueDate: Option<u64> | null | Uint8Array | u64 | AnyNumber, legs: Vec<PolymeshPrimitivesSettlementLeg> | (PolymeshPrimitivesSettlementLeg | { Fungible: any } | { NonFungible: any } | { OffChain: any } | string | Uint8Array)[], portfolios: BTreeSet<PolymeshPrimitivesIdentityIdPortfolioId>, instructionMemo: Option<PolymeshPrimitivesMemo> | null | Uint8Array | PolymeshPrimitivesMemo | string, mediators: BTreeSet<PolymeshPrimitivesIdentityId>) => SubmittableExtrinsic<ApiType>, [Option<u64>, PolymeshPrimitivesSettlementSettlementType, Option<u64>, Option<u64>, Vec<PolymeshPrimitivesSettlementLeg>, BTreeSet<PolymeshPrimitivesIdentityIdPortfolioId>, Option<PolymeshPrimitivesMemo>, BTreeSet<PolymeshPrimitivesIdentityId>]>;
      /**
       * See [`Pallet::add_instruction`].
       **/
      addInstruction: AugmentedSubmittable<(venueId: Option<u64> | null | Uint8Array | u64 | AnyNumber, settlementType: PolymeshPrimitivesSettlementSettlementType | { SettleOnAffirmation: any } | { SettleOnBlock: any } | { SettleManual: any } | { SettleAfterLock: any } | string | Uint8Array, tradeDate: Option<u64> | null | Uint8Array | u64 | AnyNumber, valueDate: Option<u64> | null | Uint8Array | u64 | AnyNumber, legs: Vec<PolymeshPrimitivesSettlementLeg> | (PolymeshPrimitivesSettlementLeg | { Fungible: any } | { NonFungible: any } | { OffChain: any } | string | Uint8Array)[], instructionMemo: Option<PolymeshPrimitivesMemo> | null | Uint8Array | PolymeshPrimitivesMemo | string) => SubmittableExtrinsic<ApiType>, [Option<u64>, PolymeshPrimitivesSettlementSettlementType, Option<u64>, Option<u64>, Vec<PolymeshPrimitivesSettlementLeg>, Option<PolymeshPrimitivesMemo>]>;
      /**
       * See [`Pallet::add_instruction_with_mediators`].
       **/
      addInstructionWithMediators: AugmentedSubmittable<(venueId: Option<u64> | null | Uint8Array | u64 | AnyNumber, settlementType: PolymeshPrimitivesSettlementSettlementType | { SettleOnAffirmation: any } | { SettleOnBlock: any } | { SettleManual: any } | { SettleAfterLock: any } | string | Uint8Array, tradeDate: Option<u64> | null | Uint8Array | u64 | AnyNumber, valueDate: Option<u64> | null | Uint8Array | u64 | AnyNumber, legs: Vec<PolymeshPrimitivesSettlementLeg> | (PolymeshPrimitivesSettlementLeg | { Fungible: any } | { NonFungible: any } | { OffChain: any } | string | Uint8Array)[], instructionMemo: Option<PolymeshPrimitivesMemo> | null | Uint8Array | PolymeshPrimitivesMemo | string, mediators: BTreeSet<PolymeshPrimitivesIdentityId>) => SubmittableExtrinsic<ApiType>, [Option<u64>, PolymeshPrimitivesSettlementSettlementType, Option<u64>, Option<u64>, Vec<PolymeshPrimitivesSettlementLeg>, Option<PolymeshPrimitivesMemo>, BTreeSet<PolymeshPrimitivesIdentityId>]>;
      /**
       * See [`Pallet::affirm_instruction`].
       **/
      affirmInstruction: AugmentedSubmittable<(id: u64 | AnyNumber | Uint8Array, portfolios: BTreeSet<PolymeshPrimitivesIdentityIdPortfolioId>) => SubmittableExtrinsic<ApiType>, [u64, BTreeSet<PolymeshPrimitivesIdentityIdPortfolioId>]>;
      /**
       * See [`Pallet::affirm_instruction_as_mediator`].
       **/
      affirmInstructionAsMediator: AugmentedSubmittable<(instructionId: u64 | AnyNumber | Uint8Array, expiry: Option<u64> | null | Uint8Array | u64 | AnyNumber) => SubmittableExtrinsic<ApiType>, [u64, Option<u64>]>;
      /**
       * See [`Pallet::affirm_instruction_with_count`].
       **/
      affirmInstructionWithCount: AugmentedSubmittable<(id: u64 | AnyNumber | Uint8Array, portfolios: BTreeSet<PolymeshPrimitivesIdentityIdPortfolioId>, numberOfAssets: Option<PolymeshPrimitivesSettlementAffirmationCount> | null | Uint8Array | PolymeshPrimitivesSettlementAffirmationCount | { senderAssetCount?: any; receiverAssetCount?: any; offchainCount?: any } | string) => SubmittableExtrinsic<ApiType>, [u64, BTreeSet<PolymeshPrimitivesIdentityIdPortfolioId>, Option<PolymeshPrimitivesSettlementAffirmationCount>]>;
      /**
       * See [`Pallet::affirm_with_receipts`].
       **/
      affirmWithReceipts: AugmentedSubmittable<(id: u64 | AnyNumber | Uint8Array, receiptDetails: Vec<PolymeshPrimitivesSettlementReceiptDetails> | (PolymeshPrimitivesSettlementReceiptDetails | { uid?: any; instructionId?: any; legId?: any; signer?: any; signature?: any; metadata?: any } | string | Uint8Array)[], portfolios: BTreeSet<PolymeshPrimitivesIdentityIdPortfolioId>) => SubmittableExtrinsic<ApiType>, [u64, Vec<PolymeshPrimitivesSettlementReceiptDetails>, BTreeSet<PolymeshPrimitivesIdentityIdPortfolioId>]>;
      /**
       * See [`Pallet::affirm_with_receipts_with_count`].
       **/
      affirmWithReceiptsWithCount: AugmentedSubmittable<(id: u64 | AnyNumber | Uint8Array, receiptDetails: Vec<PolymeshPrimitivesSettlementReceiptDetails> | (PolymeshPrimitivesSettlementReceiptDetails | { uid?: any; instructionId?: any; legId?: any; signer?: any; signature?: any; metadata?: any } | string | Uint8Array)[], portfolios: BTreeSet<PolymeshPrimitivesIdentityIdPortfolioId>, numberOfAssets: Option<PolymeshPrimitivesSettlementAffirmationCount> | null | Uint8Array | PolymeshPrimitivesSettlementAffirmationCount | { senderAssetCount?: any; receiverAssetCount?: any; offchainCount?: any } | string) => SubmittableExtrinsic<ApiType>, [u64, Vec<PolymeshPrimitivesSettlementReceiptDetails>, BTreeSet<PolymeshPrimitivesIdentityIdPortfolioId>, Option<PolymeshPrimitivesSettlementAffirmationCount>]>;
      /**
       * See [`Pallet::allow_venues`].
       **/
      allowVenues: AugmentedSubmittable<(assetId: PolymeshPrimitivesAssetAssetId | string | Uint8Array, venues: Vec<u64> | (u64 | AnyNumber | Uint8Array)[]) => SubmittableExtrinsic<ApiType>, [PolymeshPrimitivesAssetAssetId, Vec<u64>]>;
      /**
       * See [`Pallet::create_venue`].
       **/
      createVenue: AugmentedSubmittable<(details: Bytes | string | Uint8Array, signers: Vec<AccountId32> | (AccountId32 | string | Uint8Array)[], typ: PolymeshPrimitivesSettlementVenueType | 'Other' | 'Distribution' | 'Sto' | 'Exchange' | number | Uint8Array) => SubmittableExtrinsic<ApiType>, [Bytes, Vec<AccountId32>, PolymeshPrimitivesSettlementVenueType]>;
      /**
       * See [`Pallet::disallow_venues`].
       **/
      disallowVenues: AugmentedSubmittable<(assetId: PolymeshPrimitivesAssetAssetId | string | Uint8Array, venues: Vec<u64> | (u64 | AnyNumber | Uint8Array)[]) => SubmittableExtrinsic<ApiType>, [PolymeshPrimitivesAssetAssetId, Vec<u64>]>;
      /**
       * See [`Pallet::execute_manual_instruction`].
       **/
      executeManualInstruction: AugmentedSubmittable<(id: u64 | AnyNumber | Uint8Array, portfolio: Option<PolymeshPrimitivesIdentityIdPortfolioId> | null | Uint8Array | PolymeshPrimitivesIdentityIdPortfolioId | { did?: any; kind?: any } | string, fungibleTransfers: u32 | AnyNumber | Uint8Array, nftsTransfers: u32 | AnyNumber | Uint8Array, offchainTransfers: u32 | AnyNumber | Uint8Array, weightLimit: Option<SpWeightsWeightV2Weight> | null | Uint8Array | SpWeightsWeightV2Weight | { refTime?: any; proofSize?: any } | string) => SubmittableExtrinsic<ApiType>, [u64, Option<PolymeshPrimitivesIdentityIdPortfolioId>, u32, u32, u32, Option<SpWeightsWeightV2Weight>]>;
      /**
       * See [`Pallet::execute_scheduled_instruction`].
       **/
      executeScheduledInstruction: AugmentedSubmittable<(id: u64 | AnyNumber | Uint8Array, weightLimit: SpWeightsWeightV2Weight | { refTime?: any; proofSize?: any } | string | Uint8Array) => SubmittableExtrinsic<ApiType>, [u64, SpWeightsWeightV2Weight]>;
      /**
       * See [`Pallet::lock_instruction`].
       **/
      lockInstruction: AugmentedSubmittable<(instId: u64 | AnyNumber | Uint8Array, weightLimit: SpWeightsWeightV2Weight | { refTime?: any; proofSize?: any } | string | Uint8Array) => SubmittableExtrinsic<ApiType>, [u64, SpWeightsWeightV2Weight]>;
      /**
       * See [`Pallet::reject_instruction`].
       **/
      rejectInstruction: AugmentedSubmittable<(id: u64 | AnyNumber | Uint8Array, portfolio: PolymeshPrimitivesIdentityIdPortfolioId | { did?: any; kind?: any } | string | Uint8Array) => SubmittableExtrinsic<ApiType>, [u64, PolymeshPrimitivesIdentityIdPortfolioId]>;
      /**
       * See [`Pallet::reject_instruction_as_mediator`].
       **/
      rejectInstructionAsMediator: AugmentedSubmittable<(instructionId: u64 | AnyNumber | Uint8Array, numberOfAssets: Option<PolymeshPrimitivesSettlementAssetCount> | null | Uint8Array | PolymeshPrimitivesSettlementAssetCount | { fungible?: any; nonFungible?: any; offChain?: any } | string) => SubmittableExtrinsic<ApiType>, [u64, Option<PolymeshPrimitivesSettlementAssetCount>]>;
      /**
       * See [`Pallet::reject_instruction_with_count`].
       **/
      rejectInstructionWithCount: AugmentedSubmittable<(id: u64 | AnyNumber | Uint8Array, portfolio: PolymeshPrimitivesIdentityIdPortfolioId | { did?: any; kind?: any } | string | Uint8Array, numberOfAssets: Option<PolymeshPrimitivesSettlementAssetCount> | null | Uint8Array | PolymeshPrimitivesSettlementAssetCount | { fungible?: any; nonFungible?: any; offChain?: any } | string) => SubmittableExtrinsic<ApiType>, [u64, PolymeshPrimitivesIdentityIdPortfolioId, Option<PolymeshPrimitivesSettlementAssetCount>]>;
      /**
       * See [`Pallet::set_venue_filtering`].
       **/
      setVenueFiltering: AugmentedSubmittable<(assetId: PolymeshPrimitivesAssetAssetId | string | Uint8Array, enabled: bool | boolean | Uint8Array) => SubmittableExtrinsic<ApiType>, [PolymeshPrimitivesAssetAssetId, bool]>;
      /**
       * See [`Pallet::update_venue_details`].
       **/
      updateVenueDetails: AugmentedSubmittable<(id: u64 | AnyNumber | Uint8Array, details: Bytes | string | Uint8Array) => SubmittableExtrinsic<ApiType>, [u64, Bytes]>;
      /**
       * See [`Pallet::update_venue_signers`].
       **/
      updateVenueSigners: AugmentedSubmittable<(id: u64 | AnyNumber | Uint8Array, signers: Vec<AccountId32> | (AccountId32 | string | Uint8Array)[], addSigners: bool | boolean | Uint8Array) => SubmittableExtrinsic<ApiType>, [u64, Vec<AccountId32>, bool]>;
      /**
       * See [`Pallet::update_venue_type`].
       **/
      updateVenueType: AugmentedSubmittable<(id: u64 | AnyNumber | Uint8Array, typ: PolymeshPrimitivesSettlementVenueType | 'Other' | 'Distribution' | 'Sto' | 'Exchange' | number | Uint8Array) => SubmittableExtrinsic<ApiType>, [u64, PolymeshPrimitivesSettlementVenueType]>;
      /**
       * See [`Pallet::withdraw_affirmation`].
       **/
      withdrawAffirmation: AugmentedSubmittable<(id: u64 | AnyNumber | Uint8Array, portfolios: BTreeSet<PolymeshPrimitivesIdentityIdPortfolioId>) => SubmittableExtrinsic<ApiType>, [u64, BTreeSet<PolymeshPrimitivesIdentityIdPortfolioId>]>;
      /**
       * See [`Pallet::withdraw_affirmation_as_mediator`].
       **/
      withdrawAffirmationAsMediator: AugmentedSubmittable<(instructionId: u64 | AnyNumber | Uint8Array) => SubmittableExtrinsic<ApiType>, [u64]>;
      /**
       * See [`Pallet::withdraw_affirmation_with_count`].
       **/
      withdrawAffirmationWithCount: AugmentedSubmittable<(id: u64 | AnyNumber | Uint8Array, portfolios: BTreeSet<PolymeshPrimitivesIdentityIdPortfolioId>, numberOfAssets: Option<PolymeshPrimitivesSettlementAffirmationCount> | null | Uint8Array | PolymeshPrimitivesSettlementAffirmationCount | { senderAssetCount?: any; receiverAssetCount?: any; offchainCount?: any } | string) => SubmittableExtrinsic<ApiType>, [u64, BTreeSet<PolymeshPrimitivesIdentityIdPortfolioId>, Option<PolymeshPrimitivesSettlementAffirmationCount>]>;
      /**
       * Generic tx
       **/
      [key: string]: SubmittableExtrinsicFunction<ApiType>;
    };
    staking: {
      /**
       * See [`Pallet::bond`].
       **/
      bond: AugmentedSubmittable<(value: Compact<u128> | AnyNumber | Uint8Array, payee: PalletStakingRewardDestination | { Staked: any } | { Stash: any } | { Controller: any } | { Account: any } | { None: any } | string | Uint8Array) => SubmittableExtrinsic<ApiType>, [Compact<u128>, PalletStakingRewardDestination]>;
      /**
       * See [`Pallet::bond_extra`].
       **/
      bondExtra: AugmentedSubmittable<(maxAdditional: Compact<u128> | AnyNumber | Uint8Array) => SubmittableExtrinsic<ApiType>, [Compact<u128>]>;
      /**
       * See [`Pallet::cancel_deferred_slash`].
       **/
      cancelDeferredSlash: AugmentedSubmittable<(era: u32 | AnyNumber | Uint8Array, slashIndices: Vec<u32> | (u32 | AnyNumber | Uint8Array)[]) => SubmittableExtrinsic<ApiType>, [u32, Vec<u32>]>;
      /**
       * See [`Pallet::chill`].
       **/
      chill: AugmentedSubmittable<() => SubmittableExtrinsic<ApiType>, []>;
      /**
       * See [`Pallet::chill_other`].
       **/
      chillOther: AugmentedSubmittable<(controller: AccountId32 | string | Uint8Array) => SubmittableExtrinsic<ApiType>, [AccountId32]>;
      /**
       * See [`Pallet::force_apply_min_commission`].
       **/
      forceApplyMinCommission: AugmentedSubmittable<(validatorStash: AccountId32 | string | Uint8Array) => SubmittableExtrinsic<ApiType>, [AccountId32]>;
      /**
       * See [`Pallet::force_new_era`].
       **/
      forceNewEra: AugmentedSubmittable<() => SubmittableExtrinsic<ApiType>, []>;
      /**
       * See [`Pallet::force_new_era_always`].
       **/
      forceNewEraAlways: AugmentedSubmittable<() => SubmittableExtrinsic<ApiType>, []>;
      /**
       * See [`Pallet::force_no_eras`].
       **/
      forceNoEras: AugmentedSubmittable<() => SubmittableExtrinsic<ApiType>, []>;
      /**
       * See [`Pallet::force_unstake`].
       **/
      forceUnstake: AugmentedSubmittable<(stash: AccountId32 | string | Uint8Array, numSlashingSpans: u32 | AnyNumber | Uint8Array) => SubmittableExtrinsic<ApiType>, [AccountId32, u32]>;
      /**
       * See [`Pallet::increase_validator_count`].
       **/
      increaseValidatorCount: AugmentedSubmittable<(additional: Compact<u32> | AnyNumber | Uint8Array) => SubmittableExtrinsic<ApiType>, [Compact<u32>]>;
      /**
       * See [`Pallet::kick`].
       **/
      kick: AugmentedSubmittable<(who: Vec<MultiAddress> | (MultiAddress | { Id: any } | { Index: any } | { Raw: any } | { Address32: any } | { Address20: any } | string | Uint8Array)[]) => SubmittableExtrinsic<ApiType>, [Vec<MultiAddress>]>;
      /**
       * See [`Pallet::nominate`].
       **/
      nominate: AugmentedSubmittable<(targets: Vec<MultiAddress> | (MultiAddress | { Id: any } | { Index: any } | { Raw: any } | { Address32: any } | { Address20: any } | string | Uint8Array)[]) => SubmittableExtrinsic<ApiType>, [Vec<MultiAddress>]>;
      /**
       * See [`Pallet::payout_stakers`].
       **/
      payoutStakers: AugmentedSubmittable<(validatorStash: AccountId32 | string | Uint8Array, era: u32 | AnyNumber | Uint8Array) => SubmittableExtrinsic<ApiType>, [AccountId32, u32]>;
      /**
       * See [`Pallet::reap_stash`].
       **/
      reapStash: AugmentedSubmittable<(stash: AccountId32 | string | Uint8Array, numSlashingSpans: u32 | AnyNumber | Uint8Array) => SubmittableExtrinsic<ApiType>, [AccountId32, u32]>;
      /**
       * See [`Pallet::rebond`].
       **/
      rebond: AugmentedSubmittable<(value: Compact<u128> | AnyNumber | Uint8Array) => SubmittableExtrinsic<ApiType>, [Compact<u128>]>;
      /**
       * See [`Pallet::scale_validator_count`].
       **/
      scaleValidatorCount: AugmentedSubmittable<(factor: Percent | AnyNumber | Uint8Array) => SubmittableExtrinsic<ApiType>, [Percent]>;
      /**
       * See [`Pallet::set_controller`].
       **/
      setController: AugmentedSubmittable<() => SubmittableExtrinsic<ApiType>, []>;
      /**
       * See [`Pallet::set_invulnerables`].
       **/
      setInvulnerables: AugmentedSubmittable<(invulnerables: Vec<AccountId32> | (AccountId32 | string | Uint8Array)[]) => SubmittableExtrinsic<ApiType>, [Vec<AccountId32>]>;
      /**
       * See [`Pallet::set_min_commission`].
       **/
      setMinCommission: AugmentedSubmittable<(updated: Perbill | AnyNumber | Uint8Array) => SubmittableExtrinsic<ApiType>, [Perbill]>;
      /**
       * See [`Pallet::set_payee`].
       **/
      setPayee: AugmentedSubmittable<(payee: PalletStakingRewardDestination | { Staked: any } | { Stash: any } | { Controller: any } | { Account: any } | { None: any } | string | Uint8Array) => SubmittableExtrinsic<ApiType>, [PalletStakingRewardDestination]>;
      /**
       * See [`Pallet::set_staking_configs`].
       **/
      setStakingConfigs: AugmentedSubmittable<(minNominatorBond: PalletStakingPalletConfigOpU128 | { Noop: any } | { Set: any } | { Remove: any } | string | Uint8Array, minValidatorBond: PalletStakingPalletConfigOpU128 | { Noop: any } | { Set: any } | { Remove: any } | string | Uint8Array, maxNominatorCount: PalletStakingPalletConfigOpU32 | { Noop: any } | { Set: any } | { Remove: any } | string | Uint8Array, maxValidatorCount: PalletStakingPalletConfigOpU32 | { Noop: any } | { Set: any } | { Remove: any } | string | Uint8Array, chillThreshold: PalletStakingPalletConfigOpPercent | { Noop: any } | { Set: any } | { Remove: any } | string | Uint8Array, minCommission: PalletStakingPalletConfigOpPerbill | { Noop: any } | { Set: any } | { Remove: any } | string | Uint8Array) => SubmittableExtrinsic<ApiType>, [PalletStakingPalletConfigOpU128, PalletStakingPalletConfigOpU128, PalletStakingPalletConfigOpU32, PalletStakingPalletConfigOpU32, PalletStakingPalletConfigOpPercent, PalletStakingPalletConfigOpPerbill]>;
      /**
       * See [`Pallet::set_validator_count`].
       **/
      setValidatorCount: AugmentedSubmittable<(updated: Compact<u32> | AnyNumber | Uint8Array) => SubmittableExtrinsic<ApiType>, [Compact<u32>]>;
      /**
       * See [`Pallet::unbond`].
       **/
      unbond: AugmentedSubmittable<(value: Compact<u128> | AnyNumber | Uint8Array) => SubmittableExtrinsic<ApiType>, [Compact<u128>]>;
      /**
       * See [`Pallet::validate`].
       **/
      validate: AugmentedSubmittable<(prefs: PalletStakingValidatorPrefs | { commission?: any; blocked?: any } | string | Uint8Array) => SubmittableExtrinsic<ApiType>, [PalletStakingValidatorPrefs]>;
      /**
       * See [`Pallet::withdraw_unbonded`].
       **/
      withdrawUnbonded: AugmentedSubmittable<(numSlashingSpans: u32 | AnyNumber | Uint8Array) => SubmittableExtrinsic<ApiType>, [u32]>;
      /**
       * Generic tx
       **/
      [key: string]: SubmittableExtrinsicFunction<ApiType>;
    };
    statistics: {
      /**
       * See [`Pallet::batch_update_asset_stats`].
       **/
      batchUpdateAssetStats: AugmentedSubmittable<(assetId: PolymeshPrimitivesAssetAssetId | string | Uint8Array, statType: PolymeshPrimitivesStatisticsStatType | { operationType?: any; claimIssuer?: any } | string | Uint8Array, values: BTreeSet<PolymeshPrimitivesStatisticsStatUpdate>) => SubmittableExtrinsic<ApiType>, [PolymeshPrimitivesAssetAssetId, PolymeshPrimitivesStatisticsStatType, BTreeSet<PolymeshPrimitivesStatisticsStatUpdate>]>;
      /**
       * See [`Pallet::set_active_asset_stats`].
       **/
      setActiveAssetStats: AugmentedSubmittable<(assetId: PolymeshPrimitivesAssetAssetId | string | Uint8Array, statTypes: BTreeSet<PolymeshPrimitivesStatisticsStatType>) => SubmittableExtrinsic<ApiType>, [PolymeshPrimitivesAssetAssetId, BTreeSet<PolymeshPrimitivesStatisticsStatType>]>;
      /**
       * See [`Pallet::set_asset_transfer_compliance`].
       **/
      setAssetTransferCompliance: AugmentedSubmittable<(assetId: PolymeshPrimitivesAssetAssetId | string | Uint8Array, transferConditions: BTreeSet<PolymeshPrimitivesTransferComplianceTransferCondition>) => SubmittableExtrinsic<ApiType>, [PolymeshPrimitivesAssetAssetId, BTreeSet<PolymeshPrimitivesTransferComplianceTransferCondition>]>;
      /**
       * See [`Pallet::set_entities_exempt`].
       **/
      setEntitiesExempt: AugmentedSubmittable<(isExempt: bool | boolean | Uint8Array, exemptKey: PolymeshPrimitivesTransferComplianceTransferConditionExemptKey | { assetId?: any; op?: any; claimType?: any } | string | Uint8Array, entities: BTreeSet<PolymeshPrimitivesIdentityId>) => SubmittableExtrinsic<ApiType>, [bool, PolymeshPrimitivesTransferComplianceTransferConditionExemptKey, BTreeSet<PolymeshPrimitivesIdentityId>]>;
      /**
       * Generic tx
       **/
      [key: string]: SubmittableExtrinsicFunction<ApiType>;
    };
    sto: {
      /**
       * See [`Pallet::create_fundraiser`].
       **/
      createFundraiser: AugmentedSubmittable<(offeringPortfolio: PolymeshPrimitivesIdentityIdPortfolioId | { did?: any; kind?: any } | string | Uint8Array, offeringAsset: PolymeshPrimitivesAssetAssetId | string | Uint8Array, raisingPortfolio: PolymeshPrimitivesIdentityIdPortfolioId | { did?: any; kind?: any } | string | Uint8Array, raisingAsset: PolymeshPrimitivesAssetAssetId | string | Uint8Array, tiers: Vec<PalletStoPriceTier> | (PalletStoPriceTier | { total?: any; price?: any } | string | Uint8Array)[], venueId: u64 | AnyNumber | Uint8Array, start: Option<u64> | null | Uint8Array | u64 | AnyNumber, end: Option<u64> | null | Uint8Array | u64 | AnyNumber, minimumInvestment: u128 | AnyNumber | Uint8Array, fundraiserName: Bytes | string | Uint8Array) => SubmittableExtrinsic<ApiType>, [PolymeshPrimitivesIdentityIdPortfolioId, PolymeshPrimitivesAssetAssetId, PolymeshPrimitivesIdentityIdPortfolioId, PolymeshPrimitivesAssetAssetId, Vec<PalletStoPriceTier>, u64, Option<u64>, Option<u64>, u128, Bytes]>;
      /**
       * See [`Pallet::enable_offchain_funding`].
       **/
      enableOffchainFunding: AugmentedSubmittable<(offeringAsset: PolymeshPrimitivesAssetAssetId | string | Uint8Array, fundraiserId: u64 | AnyNumber | Uint8Array, ticker: PolymeshPrimitivesTicker | string | Uint8Array) => SubmittableExtrinsic<ApiType>, [PolymeshPrimitivesAssetAssetId, u64, PolymeshPrimitivesTicker]>;
      /**
       * See [`Pallet::freeze_fundraiser`].
       **/
      freezeFundraiser: AugmentedSubmittable<(offeringAsset: PolymeshPrimitivesAssetAssetId | string | Uint8Array, fundraiserId: u64 | AnyNumber | Uint8Array) => SubmittableExtrinsic<ApiType>, [PolymeshPrimitivesAssetAssetId, u64]>;
      /**
       * See [`Pallet::invest`].
       **/
      invest: AugmentedSubmittable<(offeringAsset: PolymeshPrimitivesAssetAssetId | string | Uint8Array, fundraiserId: u64 | AnyNumber | Uint8Array, investmentPortfolio: PolymeshPrimitivesIdentityIdPortfolioId | { did?: any; kind?: any } | string | Uint8Array, funding: PalletStoFundingMethod | { OnChain: any } | { OffChain: any } | string | Uint8Array, purchaseAmount: u128 | AnyNumber | Uint8Array, maxPrice: Option<u128> | null | Uint8Array | u128 | AnyNumber) => SubmittableExtrinsic<ApiType>, [PolymeshPrimitivesAssetAssetId, u64, PolymeshPrimitivesIdentityIdPortfolioId, PalletStoFundingMethod, u128, Option<u128>]>;
      /**
       * See [`Pallet::modify_fundraiser_window`].
       **/
      modifyFundraiserWindow: AugmentedSubmittable<(offeringAsset: PolymeshPrimitivesAssetAssetId | string | Uint8Array, fundraiserId: u64 | AnyNumber | Uint8Array, start: u64 | AnyNumber | Uint8Array, end: Option<u64> | null | Uint8Array | u64 | AnyNumber) => SubmittableExtrinsic<ApiType>, [PolymeshPrimitivesAssetAssetId, u64, u64, Option<u64>]>;
      /**
       * See [`Pallet::stop`].
       **/
      stop: AugmentedSubmittable<(offeringAsset: PolymeshPrimitivesAssetAssetId | string | Uint8Array, fundraiserId: u64 | AnyNumber | Uint8Array) => SubmittableExtrinsic<ApiType>, [PolymeshPrimitivesAssetAssetId, u64]>;
      /**
       * See [`Pallet::unfreeze_fundraiser`].
       **/
      unfreezeFundraiser: AugmentedSubmittable<(offeringAsset: PolymeshPrimitivesAssetAssetId | string | Uint8Array, fundraiserId: u64 | AnyNumber | Uint8Array) => SubmittableExtrinsic<ApiType>, [PolymeshPrimitivesAssetAssetId, u64]>;
      /**
       * Generic tx
       **/
      [key: string]: SubmittableExtrinsicFunction<ApiType>;
    };
    sudo: {
      /**
       * See [`Pallet::set_key`].
       **/
      setKey: AugmentedSubmittable<(updated: MultiAddress | { Id: any } | { Index: any } | { Raw: any } | { Address32: any } | { Address20: any } | string | Uint8Array) => SubmittableExtrinsic<ApiType>, [MultiAddress]>;
      /**
       * See [`Pallet::sudo`].
       **/
      sudo: AugmentedSubmittable<(call: Call | IMethod | string | Uint8Array) => SubmittableExtrinsic<ApiType>, [Call]>;
      /**
       * See [`Pallet::sudo_as`].
       **/
      sudoAs: AugmentedSubmittable<(who: MultiAddress | { Id: any } | { Index: any } | { Raw: any } | { Address32: any } | { Address20: any } | string | Uint8Array, call: Call | IMethod | string | Uint8Array) => SubmittableExtrinsic<ApiType>, [MultiAddress, Call]>;
      /**
       * See [`Pallet::sudo_unchecked_weight`].
       **/
      sudoUncheckedWeight: AugmentedSubmittable<(call: Call | IMethod | string | Uint8Array, weight: SpWeightsWeightV2Weight | { refTime?: any; proofSize?: any } | string | Uint8Array) => SubmittableExtrinsic<ApiType>, [Call, SpWeightsWeightV2Weight]>;
      /**
       * Generic tx
       **/
      [key: string]: SubmittableExtrinsicFunction<ApiType>;
    };
    system: {
      /**
       * See [`Pallet::kill_prefix`].
       **/
      killPrefix: AugmentedSubmittable<(prefix: Bytes | string | Uint8Array, subkeys: u32 | AnyNumber | Uint8Array) => SubmittableExtrinsic<ApiType>, [Bytes, u32]>;
      /**
       * See [`Pallet::kill_storage`].
       **/
      killStorage: AugmentedSubmittable<(keys: Vec<Bytes> | (Bytes | string | Uint8Array)[]) => SubmittableExtrinsic<ApiType>, [Vec<Bytes>]>;
      /**
       * See [`Pallet::remark`].
       **/
      remark: AugmentedSubmittable<(remark: Bytes | string | Uint8Array) => SubmittableExtrinsic<ApiType>, [Bytes]>;
      /**
       * See [`Pallet::remark_with_event`].
       **/
      remarkWithEvent: AugmentedSubmittable<(remark: Bytes | string | Uint8Array) => SubmittableExtrinsic<ApiType>, [Bytes]>;
      /**
       * See [`Pallet::set_code`].
       **/
      setCode: AugmentedSubmittable<(code: Bytes | string | Uint8Array) => SubmittableExtrinsic<ApiType>, [Bytes]>;
      /**
       * See [`Pallet::set_code_without_checks`].
       **/
      setCodeWithoutChecks: AugmentedSubmittable<(code: Bytes | string | Uint8Array) => SubmittableExtrinsic<ApiType>, [Bytes]>;
      /**
       * See [`Pallet::set_heap_pages`].
       **/
      setHeapPages: AugmentedSubmittable<(pages: u64 | AnyNumber | Uint8Array) => SubmittableExtrinsic<ApiType>, [u64]>;
      /**
       * See [`Pallet::set_storage`].
       **/
      setStorage: AugmentedSubmittable<(items: Vec<ITuple<[Bytes, Bytes]>> | ([Bytes | string | Uint8Array, Bytes | string | Uint8Array])[]) => SubmittableExtrinsic<ApiType>, [Vec<ITuple<[Bytes, Bytes]>>]>;
      /**
       * Generic tx
       **/
      [key: string]: SubmittableExtrinsicFunction<ApiType>;
    };
    technicalCommittee: {
      /**
       * See [`Pallet::set_expires_after`].
       **/
      setExpiresAfter: AugmentedSubmittable<(expiry: PolymeshPrimitivesMaybeBlock | { Some: any } | { None: any } | string | Uint8Array) => SubmittableExtrinsic<ApiType>, [PolymeshPrimitivesMaybeBlock]>;
      /**
       * See [`Pallet::set_release_coordinator`].
       **/
      setReleaseCoordinator: AugmentedSubmittable<(id: PolymeshPrimitivesIdentityId | string | Uint8Array) => SubmittableExtrinsic<ApiType>, [PolymeshPrimitivesIdentityId]>;
      /**
       * See [`Pallet::set_vote_threshold`].
       **/
      setVoteThreshold: AugmentedSubmittable<(n: u32 | AnyNumber | Uint8Array, d: u32 | AnyNumber | Uint8Array) => SubmittableExtrinsic<ApiType>, [u32, u32]>;
      /**
       * See [`Pallet::vote`].
       **/
      vote: AugmentedSubmittable<(proposal: H256 | string | Uint8Array, index: u32 | AnyNumber | Uint8Array, approve: bool | boolean | Uint8Array) => SubmittableExtrinsic<ApiType>, [H256, u32, bool]>;
      /**
       * See [`Pallet::vote_or_propose`].
       **/
      voteOrPropose: AugmentedSubmittable<(approve: bool | boolean | Uint8Array, call: Call | IMethod | string | Uint8Array) => SubmittableExtrinsic<ApiType>, [bool, Call]>;
      /**
       * Generic tx
       **/
      [key: string]: SubmittableExtrinsicFunction<ApiType>;
    };
    technicalCommitteeMembership: {
      /**
       * See [`Pallet::abdicate_membership`].
       **/
      abdicateMembership: AugmentedSubmittable<() => SubmittableExtrinsic<ApiType>, []>;
      /**
       * See [`Pallet::add_member`].
       **/
      addMember: AugmentedSubmittable<(who: PolymeshPrimitivesIdentityId | string | Uint8Array) => SubmittableExtrinsic<ApiType>, [PolymeshPrimitivesIdentityId]>;
      /**
       * See [`Pallet::disable_member`].
       **/
      disableMember: AugmentedSubmittable<(who: PolymeshPrimitivesIdentityId | string | Uint8Array, expiry: Option<u64> | null | Uint8Array | u64 | AnyNumber, at: Option<u64> | null | Uint8Array | u64 | AnyNumber) => SubmittableExtrinsic<ApiType>, [PolymeshPrimitivesIdentityId, Option<u64>, Option<u64>]>;
      /**
       * See [`Pallet::remove_member`].
       **/
      removeMember: AugmentedSubmittable<(who: PolymeshPrimitivesIdentityId | string | Uint8Array) => SubmittableExtrinsic<ApiType>, [PolymeshPrimitivesIdentityId]>;
      /**
       * See [`Pallet::reset_members`].
       **/
      resetMembers: AugmentedSubmittable<(members: Vec<PolymeshPrimitivesIdentityId> | (PolymeshPrimitivesIdentityId | string | Uint8Array)[]) => SubmittableExtrinsic<ApiType>, [Vec<PolymeshPrimitivesIdentityId>]>;
      /**
       * See [`Pallet::set_active_members_limit`].
       **/
      setActiveMembersLimit: AugmentedSubmittable<(limit: u32 | AnyNumber | Uint8Array) => SubmittableExtrinsic<ApiType>, [u32]>;
      /**
       * See [`Pallet::swap_member`].
       **/
      swapMember: AugmentedSubmittable<(remove: PolymeshPrimitivesIdentityId | string | Uint8Array, add: PolymeshPrimitivesIdentityId | string | Uint8Array) => SubmittableExtrinsic<ApiType>, [PolymeshPrimitivesIdentityId, PolymeshPrimitivesIdentityId]>;
      /**
       * Generic tx
       **/
      [key: string]: SubmittableExtrinsicFunction<ApiType>;
    };
    timestamp: {
      /**
       * See [`Pallet::set`].
       **/
      set: AugmentedSubmittable<(now: Compact<u64> | AnyNumber | Uint8Array) => SubmittableExtrinsic<ApiType>, [Compact<u64>]>;
      /**
       * Generic tx
       **/
      [key: string]: SubmittableExtrinsicFunction<ApiType>;
    };
    transactionPayment: {
      /**
       * See [`Pallet::set_disable_fees`].
       **/
      setDisableFees: AugmentedSubmittable<(value: bool | boolean | Uint8Array) => SubmittableExtrinsic<ApiType>, [bool]>;
      /**
       * Generic tx
       **/
      [key: string]: SubmittableExtrinsicFunction<ApiType>;
    };
    treasury: {
      /**
       * See [`Pallet::disbursement`].
       **/
      disbursement: AugmentedSubmittable<(beneficiaries: Vec<PolymeshPrimitivesBeneficiary> | (PolymeshPrimitivesBeneficiary | { id?: any; amount?: any } | string | Uint8Array)[]) => SubmittableExtrinsic<ApiType>, [Vec<PolymeshPrimitivesBeneficiary>]>;
      /**
       * See [`Pallet::reimbursement`].
       **/
      reimbursement: AugmentedSubmittable<(amount: u128 | AnyNumber | Uint8Array) => SubmittableExtrinsic<ApiType>, [u128]>;
      /**
       * Generic tx
       **/
      [key: string]: SubmittableExtrinsicFunction<ApiType>;
    };
    upgradeCommittee: {
      /**
       * See [`Pallet::set_expires_after`].
       **/
      setExpiresAfter: AugmentedSubmittable<(expiry: PolymeshPrimitivesMaybeBlock | { Some: any } | { None: any } | string | Uint8Array) => SubmittableExtrinsic<ApiType>, [PolymeshPrimitivesMaybeBlock]>;
      /**
       * See [`Pallet::set_release_coordinator`].
       **/
      setReleaseCoordinator: AugmentedSubmittable<(id: PolymeshPrimitivesIdentityId | string | Uint8Array) => SubmittableExtrinsic<ApiType>, [PolymeshPrimitivesIdentityId]>;
      /**
       * See [`Pallet::set_vote_threshold`].
       **/
      setVoteThreshold: AugmentedSubmittable<(n: u32 | AnyNumber | Uint8Array, d: u32 | AnyNumber | Uint8Array) => SubmittableExtrinsic<ApiType>, [u32, u32]>;
      /**
       * See [`Pallet::vote`].
       **/
      vote: AugmentedSubmittable<(proposal: H256 | string | Uint8Array, index: u32 | AnyNumber | Uint8Array, approve: bool | boolean | Uint8Array) => SubmittableExtrinsic<ApiType>, [H256, u32, bool]>;
      /**
       * See [`Pallet::vote_or_propose`].
       **/
      voteOrPropose: AugmentedSubmittable<(approve: bool | boolean | Uint8Array, call: Call | IMethod | string | Uint8Array) => SubmittableExtrinsic<ApiType>, [bool, Call]>;
      /**
       * Generic tx
       **/
      [key: string]: SubmittableExtrinsicFunction<ApiType>;
    };
    upgradeCommitteeMembership: {
      /**
       * See [`Pallet::abdicate_membership`].
       **/
      abdicateMembership: AugmentedSubmittable<() => SubmittableExtrinsic<ApiType>, []>;
      /**
       * See [`Pallet::add_member`].
       **/
      addMember: AugmentedSubmittable<(who: PolymeshPrimitivesIdentityId | string | Uint8Array) => SubmittableExtrinsic<ApiType>, [PolymeshPrimitivesIdentityId]>;
      /**
       * See [`Pallet::disable_member`].
       **/
      disableMember: AugmentedSubmittable<(who: PolymeshPrimitivesIdentityId | string | Uint8Array, expiry: Option<u64> | null | Uint8Array | u64 | AnyNumber, at: Option<u64> | null | Uint8Array | u64 | AnyNumber) => SubmittableExtrinsic<ApiType>, [PolymeshPrimitivesIdentityId, Option<u64>, Option<u64>]>;
      /**
       * See [`Pallet::remove_member`].
       **/
      removeMember: AugmentedSubmittable<(who: PolymeshPrimitivesIdentityId | string | Uint8Array) => SubmittableExtrinsic<ApiType>, [PolymeshPrimitivesIdentityId]>;
      /**
       * See [`Pallet::reset_members`].
       **/
      resetMembers: AugmentedSubmittable<(members: Vec<PolymeshPrimitivesIdentityId> | (PolymeshPrimitivesIdentityId | string | Uint8Array)[]) => SubmittableExtrinsic<ApiType>, [Vec<PolymeshPrimitivesIdentityId>]>;
      /**
       * See [`Pallet::set_active_members_limit`].
       **/
      setActiveMembersLimit: AugmentedSubmittable<(limit: u32 | AnyNumber | Uint8Array) => SubmittableExtrinsic<ApiType>, [u32]>;
      /**
       * See [`Pallet::swap_member`].
       **/
      swapMember: AugmentedSubmittable<(remove: PolymeshPrimitivesIdentityId | string | Uint8Array, add: PolymeshPrimitivesIdentityId | string | Uint8Array) => SubmittableExtrinsic<ApiType>, [PolymeshPrimitivesIdentityId, PolymeshPrimitivesIdentityId]>;
      /**
       * Generic tx
       **/
      [key: string]: SubmittableExtrinsicFunction<ApiType>;
    };
    utility: {
      /**
       * See [`Pallet::as_derivative`].
       **/
      asDerivative: AugmentedSubmittable<(index: u16 | AnyNumber | Uint8Array, call: Call | IMethod | string | Uint8Array) => SubmittableExtrinsic<ApiType>, [u16, Call]>;
      /**
       * See [`Pallet::batch`].
       **/
      batch: AugmentedSubmittable<(calls: Vec<Call> | (Call | IMethod | string | Uint8Array)[]) => SubmittableExtrinsic<ApiType>, [Vec<Call>]>;
      /**
       * See [`Pallet::batch_all`].
       **/
      batchAll: AugmentedSubmittable<(calls: Vec<Call> | (Call | IMethod | string | Uint8Array)[]) => SubmittableExtrinsic<ApiType>, [Vec<Call>]>;
      /**
       * See [`Pallet::dispatch_as`].
       **/
      dispatchAs: AugmentedSubmittable<(asOrigin: PolymeshRuntimeDevelopRuntimeOriginCaller | { system: any } | { Void: any } | { PolymeshCommittee: any } | { TechnicalCommittee: any } | { UpgradeCommittee: any } | string | Uint8Array, call: Call | IMethod | string | Uint8Array) => SubmittableExtrinsic<ApiType>, [PolymeshRuntimeDevelopRuntimeOriginCaller, Call]>;
      /**
       * See [`Pallet::force_batch`].
       **/
      forceBatch: AugmentedSubmittable<(calls: Vec<Call> | (Call | IMethod | string | Uint8Array)[]) => SubmittableExtrinsic<ApiType>, [Vec<Call>]>;
      /**
       * See [`Pallet::relay_tx`].
       **/
      relayTx: AugmentedSubmittable<(target: AccountId32 | string | Uint8Array, signature: SpRuntimeMultiSignature | { Ed25519: any } | { Sr25519: any } | { Ecdsa: any } | string | Uint8Array, call: PalletUtilityUniqueCall | { nonce?: any; call?: any } | string | Uint8Array) => SubmittableExtrinsic<ApiType>, [AccountId32, SpRuntimeMultiSignature, PalletUtilityUniqueCall]>;
      /**
       * See [`Pallet::with_weight`].
       **/
      withWeight: AugmentedSubmittable<(call: Call | IMethod | string | Uint8Array, weight: SpWeightsWeightV2Weight | { refTime?: any; proofSize?: any } | string | Uint8Array) => SubmittableExtrinsic<ApiType>, [Call, SpWeightsWeightV2Weight]>;
      /**
       * Generic tx
       **/
      [key: string]: SubmittableExtrinsicFunction<ApiType>;
    };
    validators: {
      /**
       * See [`Pallet::add_permissioned_validator`].
       **/
      addPermissionedValidator: AugmentedSubmittable<(identity: PolymeshPrimitivesIdentityId | string | Uint8Array, intendedCount: Option<u32> | null | Uint8Array | u32 | AnyNumber) => SubmittableExtrinsic<ApiType>, [PolymeshPrimitivesIdentityId, Option<u32>]>;
      /**
       * See [`Pallet::change_slashing_allowed_for`].
       **/
      changeSlashingAllowedFor: AugmentedSubmittable<(slashingSwitch: PalletValidatorsSlashingSwitch | 'Validator' | 'ValidatorAndNominator' | 'None' | number | Uint8Array) => SubmittableExtrinsic<ApiType>, [PalletValidatorsSlashingSwitch]>;
      /**
       * See [`Pallet::chill_from_governance`].
       **/
      chillFromGovernance: AugmentedSubmittable<(identity: PolymeshPrimitivesIdentityId | string | Uint8Array, stashKeys: Vec<AccountId32> | (AccountId32 | string | Uint8Array)[]) => SubmittableExtrinsic<ApiType>, [PolymeshPrimitivesIdentityId, Vec<AccountId32>]>;
      /**
       * See [`Pallet::payout_stakers_by_system`].
       **/
      payoutStakersBySystem: AugmentedSubmittable<(validatorStash: AccountId32 | string | Uint8Array, era: u32 | AnyNumber | Uint8Array) => SubmittableExtrinsic<ApiType>, [AccountId32, u32]>;
      /**
       * See [`Pallet::remove_permissioned_validator`].
       **/
      removePermissionedValidator: AugmentedSubmittable<(identity: PolymeshPrimitivesIdentityId | string | Uint8Array) => SubmittableExtrinsic<ApiType>, [PolymeshPrimitivesIdentityId]>;
      /**
       * See [`Pallet::set_commission_cap`].
       **/
      setCommissionCap: AugmentedSubmittable<(newCap: Perbill | AnyNumber | Uint8Array) => SubmittableExtrinsic<ApiType>, [Perbill]>;
      /**
       * See [`Pallet::update_permissioned_validator_intended_count`].
       **/
      updatePermissionedValidatorIntendedCount: AugmentedSubmittable<(identity: PolymeshPrimitivesIdentityId | string | Uint8Array, newIntendedCount: u32 | AnyNumber | Uint8Array) => SubmittableExtrinsic<ApiType>, [PolymeshPrimitivesIdentityId, u32]>;
      /**
       * Generic tx
       **/
      [key: string]: SubmittableExtrinsicFunction<ApiType>;
    };
  } // AugmentedSubmittables
} // declare module
