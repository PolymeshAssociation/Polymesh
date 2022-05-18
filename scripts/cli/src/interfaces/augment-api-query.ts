// Auto-generated via `yarn polkadot-types-from-chain`, do not edit
/* eslint-disable */

import type { ApiTypes } from '@polkadot/api-base/types';
import type { Bytes, Null, Option, U8aFixed, Vec, WrapperOpaque, bool, u128, u16, u32, u64, u8 } from '@polkadot/types-codec';
import type { AnyNumber, ITuple } from '@polkadot/types-codec/types';
import type { AccountId32, Call, H256, Perbill, Permill } from '@polkadot/types/interfaces/runtime';
import type { BTreeSetStatType, FrameSupportWeightsPerDispatchClassU64, FrameSystemAccountInfo, FrameSystemEventRecord, FrameSystemLastRuntimeUpgradeInfo, FrameSystemPhase, PalletAssetAssetOwnershipRelation, PalletAssetClassicTickerRegistration, PalletAssetSecurityToken, PalletAssetTickerRegistration, PalletAssetTickerRegistrationConfig, PalletAuthorshipUncleEntryItem, PalletBalancesBalanceLock, PalletBridgeBridgeTxDetail, PalletCommitteePolymeshVotes, PalletContractsStorageDeletedContract, PalletContractsStorageRawContractInfo, PalletContractsWasmOwnerInfo, PalletContractsWasmPrefabWasmModule, PalletCorporateActionsBallotBallotMeta, PalletCorporateActionsBallotBallotTimeRange, PalletCorporateActionsBallotBallotVote, PalletCorporateActionsCaId, PalletCorporateActionsCorporateAction, PalletCorporateActionsDistribution, PalletCorporateActionsTargetIdentities, PalletGrandpaStoredPendingChange, PalletGrandpaStoredState, PalletIdentityClaim1stKey, PalletIdentityClaim2ndKey, PalletImOnlineBoundedOpaqueNetworkState, PalletImOnlineSr25519AppSr25519Public, PalletMultisigProposalDetails, PalletPipsDepositInfo, PalletPipsPip, PalletPipsPipsMetadata, PalletPipsSnapshotMetadata, PalletPipsSnapshottedPip, PalletPipsVote, PalletPipsVotingResult, PalletPreimageRequestStatus, PalletRelayerSubsidy, PalletRewardsItnRewardStatus, PalletSchedulerScheduledV3, PalletSettlementAffirmationStatus, PalletSettlementInstruction, PalletSettlementLeg, PalletSettlementLegStatus, PalletSettlementVenue, PalletStakingActiveEraInfo, PalletStakingElectionResult, PalletStakingElectionStatus, PalletStakingEraRewardPoints, PalletStakingExposure, PalletStakingForcing, PalletStakingNominations, PalletStakingPermissionedIdentityPrefs, PalletStakingReleases, PalletStakingRewardDestination, PalletStakingSlashingSlashingSpans, PalletStakingSlashingSpanRecord, PalletStakingSlashingSwitch, PalletStakingStakingLedger, PalletStakingUnappliedSlash, PalletStakingValidatorPrefs, PalletStoFundraiser, PalletTransactionPaymentReleases, PolymeshCommonUtilitiesCheckpointStoredSchedule, PolymeshCommonUtilitiesGroupInactiveMember, PolymeshCommonUtilitiesMaybeBlock, PolymeshCommonUtilitiesProtocolFeeProtocolOp, PolymeshPrimitivesAgentAgentGroup, PolymeshPrimitivesAssetIdentifier, PolymeshPrimitivesAssetMetadataAssetMetadataKey, PolymeshPrimitivesAssetMetadataAssetMetadataSpec, PolymeshPrimitivesAssetMetadataAssetMetadataValueDetail, PolymeshPrimitivesAuthorization, PolymeshPrimitivesComplianceManagerAssetCompliance, PolymeshPrimitivesConditionTrustedIssuer, PolymeshPrimitivesDocument, PolymeshPrimitivesIdentityClaim, PolymeshPrimitivesIdentityDidRecord, PolymeshPrimitivesIdentityId, PolymeshPrimitivesIdentityIdPortfolioId, PolymeshPrimitivesPosRatio, PolymeshPrimitivesSecondaryKeyKeyRecord, PolymeshPrimitivesSecondaryKeySignatory, PolymeshPrimitivesStatisticsAssetScope, PolymeshPrimitivesStatisticsStat1stKey, PolymeshPrimitivesStatisticsStat2ndKey, PolymeshPrimitivesStatisticsStatType, PolymeshPrimitivesSubsetSubsetRestrictionPalletPermissions, PolymeshPrimitivesTicker, PolymeshPrimitivesTransferComplianceAssetTransferCompliance, PolymeshPrimitivesTransferComplianceTransferConditionExemptKey, PolymeshRuntimeDevelopRuntimeSessionKeys, SpConsensusBabeAppPublic, SpConsensusBabeBabeEpochConfiguration, SpConsensusBabeDigestsNextConfigDescriptor, SpCoreCryptoKeyTypeId, SpNposElectionsElectionScore, SpRuntimeDigest, SpStakingOffenceOffenceDetails } from '@polkadot/types/lookup';
import type { Observable } from '@polkadot/types/types';

declare module '@polkadot/api-base/types/storage' {
  export interface AugmentedQueries<ApiType extends ApiTypes> {
    asset: {
      /**
       * Store aggregate balance of those identities that has the same `ScopeId`.
       * (Ticker, ScopeId) => Balance.
       **/
      aggregateBalance: AugmentedQuery<ApiType, (arg1: PolymeshPrimitivesTicker | string | Uint8Array, arg2: PolymeshPrimitivesIdentityId | string | Uint8Array) => Observable<u128>, [PolymeshPrimitivesTicker, PolymeshPrimitivesIdentityId]> & QueryableStorageEntry<ApiType, [PolymeshPrimitivesTicker, PolymeshPrimitivesIdentityId]>;
      /**
       * Documents attached to an Asset
       * (ticker, doc_id) -> document
       **/
      assetDocuments: AugmentedQuery<ApiType, (arg1: PolymeshPrimitivesTicker | string | Uint8Array, arg2: u32 | AnyNumber | Uint8Array) => Observable<PolymeshPrimitivesDocument>, [PolymeshPrimitivesTicker, u32]> & QueryableStorageEntry<ApiType, [PolymeshPrimitivesTicker, u32]>;
      /**
       * Per-ticker document ID counter.
       * (ticker) -> doc_id
       **/
      assetDocumentsIdSequence: AugmentedQuery<ApiType, (arg: PolymeshPrimitivesTicker | string | Uint8Array) => Observable<u32>, [PolymeshPrimitivesTicker]> & QueryableStorageEntry<ApiType, [PolymeshPrimitivesTicker]>;
      /**
       * Asset Metadata Global Key -> Name.
       **/
      assetMetadataGlobalKeyToName: AugmentedQuery<ApiType, (arg: u64 | AnyNumber | Uint8Array) => Observable<Option<Bytes>>, [u64]> & QueryableStorageEntry<ApiType, [u64]>;
      /**
       * Asset Metadata Global Name -> Key.
       **/
      assetMetadataGlobalNameToKey: AugmentedQuery<ApiType, (arg: Bytes | string | Uint8Array) => Observable<Option<u64>>, [Bytes]> & QueryableStorageEntry<ApiType, [Bytes]>;
      /**
       * Asset Metadata Global Key specs.
       **/
      assetMetadataGlobalSpecs: AugmentedQuery<ApiType, (arg: u64 | AnyNumber | Uint8Array) => Observable<Option<PolymeshPrimitivesAssetMetadataAssetMetadataSpec>>, [u64]> & QueryableStorageEntry<ApiType, [u64]>;
      /**
       * Asset Metadata Local Key -> Name.
       **/
      assetMetadataLocalKeyToName: AugmentedQuery<ApiType, (arg1: PolymeshPrimitivesTicker | string | Uint8Array, arg2: u64 | AnyNumber | Uint8Array) => Observable<Option<Bytes>>, [PolymeshPrimitivesTicker, u64]> & QueryableStorageEntry<ApiType, [PolymeshPrimitivesTicker, u64]>;
      /**
       * Asset Metadata Local Name -> Key.
       **/
      assetMetadataLocalNameToKey: AugmentedQuery<ApiType, (arg1: PolymeshPrimitivesTicker | string | Uint8Array, arg2: Bytes | string | Uint8Array) => Observable<Option<u64>>, [PolymeshPrimitivesTicker, Bytes]> & QueryableStorageEntry<ApiType, [PolymeshPrimitivesTicker, Bytes]>;
      /**
       * Asset Metadata Local Key specs.
       **/
      assetMetadataLocalSpecs: AugmentedQuery<ApiType, (arg1: PolymeshPrimitivesTicker | string | Uint8Array, arg2: u64 | AnyNumber | Uint8Array) => Observable<Option<PolymeshPrimitivesAssetMetadataAssetMetadataSpec>>, [PolymeshPrimitivesTicker, u64]> & QueryableStorageEntry<ApiType, [PolymeshPrimitivesTicker, u64]>;
      /**
       * Next Asset Metadata Global Key.
       **/
      assetMetadataNextGlobalKey: AugmentedQuery<ApiType, () => Observable<u64>, []> & QueryableStorageEntry<ApiType, []>;
      /**
       * Next Asset Metadata Local Key.
       **/
      assetMetadataNextLocalKey: AugmentedQuery<ApiType, (arg: PolymeshPrimitivesTicker | string | Uint8Array) => Observable<u64>, [PolymeshPrimitivesTicker]> & QueryableStorageEntry<ApiType, [PolymeshPrimitivesTicker]>;
      /**
       * Details for an asset's Metadata values.
       **/
      assetMetadataValueDetails: AugmentedQuery<ApiType, (arg1: PolymeshPrimitivesTicker | string | Uint8Array, arg2: PolymeshPrimitivesAssetMetadataAssetMetadataKey | { Global: any } | { Local: any } | string | Uint8Array) => Observable<Option<PolymeshPrimitivesAssetMetadataAssetMetadataValueDetail>>, [PolymeshPrimitivesTicker, PolymeshPrimitivesAssetMetadataAssetMetadataKey]> & QueryableStorageEntry<ApiType, [PolymeshPrimitivesTicker, PolymeshPrimitivesAssetMetadataAssetMetadataKey]>;
      /**
       * Metatdata values for an asset.
       **/
      assetMetadataValues: AugmentedQuery<ApiType, (arg1: PolymeshPrimitivesTicker | string | Uint8Array, arg2: PolymeshPrimitivesAssetMetadataAssetMetadataKey | { Global: any } | { Local: any } | string | Uint8Array) => Observable<Option<Bytes>>, [PolymeshPrimitivesTicker, PolymeshPrimitivesAssetMetadataAssetMetadataKey]> & QueryableStorageEntry<ApiType, [PolymeshPrimitivesTicker, PolymeshPrimitivesAssetMetadataAssetMetadataKey]>;
      /**
       * Asset name of the token corresponding to the token ticker.
       * (ticker) -> `AssetName`
       **/
      assetNames: AugmentedQuery<ApiType, (arg: PolymeshPrimitivesTicker | string | Uint8Array) => Observable<Bytes>, [PolymeshPrimitivesTicker]> & QueryableStorageEntry<ApiType, [PolymeshPrimitivesTicker]>;
      /**
       * Tickers and token owned by a user
       * (user, ticker) -> AssetOwnership
       **/
      assetOwnershipRelations: AugmentedQuery<ApiType, (arg1: PolymeshPrimitivesIdentityId | string | Uint8Array, arg2: PolymeshPrimitivesTicker | string | Uint8Array) => Observable<PalletAssetAssetOwnershipRelation>, [PolymeshPrimitivesIdentityId, PolymeshPrimitivesTicker]> & QueryableStorageEntry<ApiType, [PolymeshPrimitivesIdentityId, PolymeshPrimitivesTicker]>;
      /**
       * The total asset ticker balance per identity.
       * (ticker, DID) -> Balance
       **/
      balanceOf: AugmentedQuery<ApiType, (arg1: PolymeshPrimitivesTicker | string | Uint8Array, arg2: PolymeshPrimitivesIdentityId | string | Uint8Array) => Observable<u128>, [PolymeshPrimitivesTicker, PolymeshPrimitivesIdentityId]> & QueryableStorageEntry<ApiType, [PolymeshPrimitivesTicker, PolymeshPrimitivesIdentityId]>;
      /**
       * Balances get stored on the basis of the `ScopeId`.
       * Right now it is only helpful for the UI purposes but in future it can be used to do miracles on-chain.
       * (ScopeId, IdentityId) => Balance.
       **/
      balanceOfAtScope: AugmentedQuery<ApiType, (arg1: PolymeshPrimitivesIdentityId | string | Uint8Array, arg2: PolymeshPrimitivesIdentityId | string | Uint8Array) => Observable<u128>, [PolymeshPrimitivesIdentityId, PolymeshPrimitivesIdentityId]> & QueryableStorageEntry<ApiType, [PolymeshPrimitivesIdentityId, PolymeshPrimitivesIdentityId]>;
      /**
       * Ticker registration details on Polymath Classic / Ethereum.
       **/
      classicTickers: AugmentedQuery<ApiType, (arg: PolymeshPrimitivesTicker | string | Uint8Array) => Observable<Option<PalletAssetClassicTickerRegistration>>, [PolymeshPrimitivesTicker]> & QueryableStorageEntry<ApiType, [PolymeshPrimitivesTicker]>;
      /**
       * The next `AssetType::Custom` ID in the sequence.
       * 
       * Numbers in the sequence start from 1 rather than 0.
       **/
      customTypeIdSequence: AugmentedQuery<ApiType, () => Observable<u32>, []> & QueryableStorageEntry<ApiType, []>;
      /**
       * Maps custom asset type ids to the registered string contents.
       **/
      customTypes: AugmentedQuery<ApiType, (arg: u32 | AnyNumber | Uint8Array) => Observable<Bytes>, [u32]> & QueryableStorageEntry<ApiType, [u32]>;
      /**
       * Inverse map of `CustomTypes`, from registered string contents to custom asset type ids.
       **/
      customTypesInverse: AugmentedQuery<ApiType, (arg: Bytes | string | Uint8Array) => Observable<u32>, [Bytes]> & QueryableStorageEntry<ApiType, [Bytes]>;
      /**
       * Decides whether investor uniqueness requirement is enforced for this asset.
       * `false` means that it is enforced.
       * 
       * Ticker => bool.
       **/
      disableInvestorUniqueness: AugmentedQuery<ApiType, (arg: PolymeshPrimitivesTicker | string | Uint8Array) => Observable<bool>, [PolymeshPrimitivesTicker]> & QueryableStorageEntry<ApiType, [PolymeshPrimitivesTicker]>;
      /**
       * The set of frozen assets implemented as a membership map.
       * ticker -> bool
       **/
      frozen: AugmentedQuery<ApiType, (arg: PolymeshPrimitivesTicker | string | Uint8Array) => Observable<bool>, [PolymeshPrimitivesTicker]> & QueryableStorageEntry<ApiType, [PolymeshPrimitivesTicker]>;
      /**
       * The name of the current funding round.
       * ticker -> funding round
       **/
      fundingRound: AugmentedQuery<ApiType, (arg: PolymeshPrimitivesTicker | string | Uint8Array) => Observable<Bytes>, [PolymeshPrimitivesTicker]> & QueryableStorageEntry<ApiType, [PolymeshPrimitivesTicker]>;
      /**
       * A map of a ticker name and asset identifiers.
       **/
      identifiers: AugmentedQuery<ApiType, (arg: PolymeshPrimitivesTicker | string | Uint8Array) => Observable<Vec<PolymeshPrimitivesAssetIdentifier>>, [PolymeshPrimitivesTicker]> & QueryableStorageEntry<ApiType, [PolymeshPrimitivesTicker]>;
      /**
       * The total balances of tokens issued in all recorded funding rounds.
       * (ticker, funding round) -> balance
       **/
      issuedInFundingRound: AugmentedQuery<ApiType, (arg: ITuple<[PolymeshPrimitivesTicker, Bytes]> | [PolymeshPrimitivesTicker | string | Uint8Array, Bytes | string | Uint8Array]) => Observable<u128>, [ITuple<[PolymeshPrimitivesTicker, Bytes]>]> & QueryableStorageEntry<ApiType, [ITuple<[PolymeshPrimitivesTicker, Bytes]>]>;
      /**
       * Tracks the ScopeId of the identity for a given ticker.
       * (Ticker, IdentityId) => ScopeId.
       **/
      scopeIdOf: AugmentedQuery<ApiType, (arg1: PolymeshPrimitivesTicker | string | Uint8Array, arg2: PolymeshPrimitivesIdentityId | string | Uint8Array) => Observable<U8aFixed>, [PolymeshPrimitivesTicker, PolymeshPrimitivesIdentityId]> & QueryableStorageEntry<ApiType, [PolymeshPrimitivesTicker, PolymeshPrimitivesIdentityId]>;
      /**
       * Storage version.
       **/
      storageVersion: AugmentedQuery<ApiType, () => Observable<u8>, []> & QueryableStorageEntry<ApiType, []>;
      /**
       * Ticker registration config.
       * (ticker) -> TickerRegistrationConfig
       **/
      tickerConfig: AugmentedQuery<ApiType, () => Observable<PalletAssetTickerRegistrationConfig>, []> & QueryableStorageEntry<ApiType, []>;
      /**
       * Ticker registration details.
       * (ticker) -> TickerRegistration
       **/
      tickers: AugmentedQuery<ApiType, (arg: PolymeshPrimitivesTicker | string | Uint8Array) => Observable<PalletAssetTickerRegistration>, [PolymeshPrimitivesTicker]> & QueryableStorageEntry<ApiType, [PolymeshPrimitivesTicker]>;
      /**
       * Details of the token corresponding to the token ticker.
       * (ticker) -> SecurityToken details [returns SecurityToken struct]
       **/
      tokens: AugmentedQuery<ApiType, (arg: PolymeshPrimitivesTicker | string | Uint8Array) => Observable<PalletAssetSecurityToken>, [PolymeshPrimitivesTicker]> & QueryableStorageEntry<ApiType, [PolymeshPrimitivesTicker]>;
      /**
       * Generic query
       **/
      [key: string]: QueryableStorageEntry<ApiType>;
    };
    authorship: {
      /**
       * Author of current block.
       **/
      author: AugmentedQuery<ApiType, () => Observable<Option<AccountId32>>, []> & QueryableStorageEntry<ApiType, []>;
      /**
       * Whether uncles were already set in this block.
       **/
      didSetUncles: AugmentedQuery<ApiType, () => Observable<bool>, []> & QueryableStorageEntry<ApiType, []>;
      /**
       * Uncles
       **/
      uncles: AugmentedQuery<ApiType, () => Observable<Vec<PalletAuthorshipUncleEntryItem>>, []> & QueryableStorageEntry<ApiType, []>;
      /**
       * Generic query
       **/
      [key: string]: QueryableStorageEntry<ApiType>;
    };
    babe: {
      /**
       * Current epoch authorities.
       **/
      authorities: AugmentedQuery<ApiType, () => Observable<Vec<ITuple<[SpConsensusBabeAppPublic, u64]>>>, []> & QueryableStorageEntry<ApiType, []>;
      /**
       * This field should always be populated during block processing unless
       * secondary plain slots are enabled (which don't contain a VRF output).
       * 
       * It is set in `on_initialize`, before it will contain the value from the last block.
       **/
      authorVrfRandomness: AugmentedQuery<ApiType, () => Observable<Option<U8aFixed>>, []> & QueryableStorageEntry<ApiType, []>;
      /**
       * Current slot number.
       **/
      currentSlot: AugmentedQuery<ApiType, () => Observable<u64>, []> & QueryableStorageEntry<ApiType, []>;
      /**
       * The configuration for the current epoch. Should never be `None` as it is initialized in
       * genesis.
       **/
      epochConfig: AugmentedQuery<ApiType, () => Observable<Option<SpConsensusBabeBabeEpochConfiguration>>, []> & QueryableStorageEntry<ApiType, []>;
      /**
       * Current epoch index.
       **/
      epochIndex: AugmentedQuery<ApiType, () => Observable<u64>, []> & QueryableStorageEntry<ApiType, []>;
      /**
       * The block numbers when the last and current epoch have started, respectively `N-1` and
       * `N`.
       * NOTE: We track this is in order to annotate the block number when a given pool of
       * entropy was fixed (i.e. it was known to chain observers). Since epochs are defined in
       * slots, which may be skipped, the block numbers may not line up with the slot numbers.
       **/
      epochStart: AugmentedQuery<ApiType, () => Observable<ITuple<[u32, u32]>>, []> & QueryableStorageEntry<ApiType, []>;
      /**
       * The slot at which the first epoch actually started. This is 0
       * until the first block of the chain.
       **/
      genesisSlot: AugmentedQuery<ApiType, () => Observable<u64>, []> & QueryableStorageEntry<ApiType, []>;
      /**
       * Temporary value (cleared at block finalization) which is `Some`
       * if per-block initialization has already been called for current block.
       **/
      initialized: AugmentedQuery<ApiType, () => Observable<Option<Option<U8aFixed>>>, []> & QueryableStorageEntry<ApiType, []>;
      /**
       * How late the current block is compared to its parent.
       * 
       * This entry is populated as part of block execution and is cleaned up
       * on block finalization. Querying this storage entry outside of block
       * execution context should always yield zero.
       **/
      lateness: AugmentedQuery<ApiType, () => Observable<u32>, []> & QueryableStorageEntry<ApiType, []>;
      /**
       * Next epoch authorities.
       **/
      nextAuthorities: AugmentedQuery<ApiType, () => Observable<Vec<ITuple<[SpConsensusBabeAppPublic, u64]>>>, []> & QueryableStorageEntry<ApiType, []>;
      /**
       * The configuration for the next epoch, `None` if the config will not change
       * (you can fallback to `EpochConfig` instead in that case).
       **/
      nextEpochConfig: AugmentedQuery<ApiType, () => Observable<Option<SpConsensusBabeBabeEpochConfiguration>>, []> & QueryableStorageEntry<ApiType, []>;
      /**
       * Next epoch randomness.
       **/
      nextRandomness: AugmentedQuery<ApiType, () => Observable<U8aFixed>, []> & QueryableStorageEntry<ApiType, []>;
      /**
       * Pending epoch configuration change that will be applied when the next epoch is enacted.
       **/
      pendingEpochConfigChange: AugmentedQuery<ApiType, () => Observable<Option<SpConsensusBabeDigestsNextConfigDescriptor>>, []> & QueryableStorageEntry<ApiType, []>;
      /**
       * The epoch randomness for the *current* epoch.
       * 
       * # Security
       * 
       * This MUST NOT be used for gambling, as it can be influenced by a
       * malicious validator in the short term. It MAY be used in many
       * cryptographic protocols, however, so long as one remembers that this
       * (like everything else on-chain) it is public. For example, it can be
       * used where a number is needed that cannot have been chosen by an
       * adversary, for purposes such as public-coin zero-knowledge proofs.
       **/
      randomness: AugmentedQuery<ApiType, () => Observable<U8aFixed>, []> & QueryableStorageEntry<ApiType, []>;
      /**
       * Randomness under construction.
       * 
       * We make a trade-off between storage accesses and list length.
       * We store the under-construction randomness in segments of up to
       * `UNDER_CONSTRUCTION_SEGMENT_LENGTH`.
       * 
       * Once a segment reaches this length, we begin the next one.
       * We reset all segments and return to `0` at the beginning of every
       * epoch.
       **/
      segmentIndex: AugmentedQuery<ApiType, () => Observable<u32>, []> & QueryableStorageEntry<ApiType, []>;
      /**
       * TWOX-NOTE: `SegmentIndex` is an increasing integer, so this is okay.
       **/
      underConstruction: AugmentedQuery<ApiType, (arg: u32 | AnyNumber | Uint8Array) => Observable<Vec<U8aFixed>>, [u32]> & QueryableStorageEntry<ApiType, [u32]>;
      /**
       * Generic query
       **/
      [key: string]: QueryableStorageEntry<ApiType>;
    };
    balances: {
      /**
       * Any liquidity locks on some account balances.
       * NOTE: Should only be accessed when setting, changing and freeing a lock.
       **/
      locks: AugmentedQuery<ApiType, (arg: AccountId32 | string | Uint8Array) => Observable<Vec<PalletBalancesBalanceLock>>, [AccountId32]> & QueryableStorageEntry<ApiType, [AccountId32]>;
      /**
       * The total units issued in the system.
       **/
      totalIssuance: AugmentedQuery<ApiType, () => Observable<u128>, []> & QueryableStorageEntry<ApiType, []>;
      /**
       * Generic query
       **/
      [key: string]: QueryableStorageEntry<ApiType>;
    };
    bridge: {
      /**
       * The admin key.
       **/
      admin: AugmentedQuery<ApiType, () => Observable<AccountId32>, []> & QueryableStorageEntry<ApiType, []>;
      /**
       * The maximum number of bridged POLYX per identity within a set interval of
       * blocks. Fields: POLYX amount and the block interval duration.
       **/
      bridgeLimit: AugmentedQuery<ApiType, () => Observable<ITuple<[u128, u32]>>, []> & QueryableStorageEntry<ApiType, []>;
      /**
       * Identities not constrained by the bridge limit.
       **/
      bridgeLimitExempted: AugmentedQuery<ApiType, (arg: PolymeshPrimitivesIdentityId | string | Uint8Array) => Observable<bool>, [PolymeshPrimitivesIdentityId]> & QueryableStorageEntry<ApiType, [PolymeshPrimitivesIdentityId]>;
      /**
       * Details of bridge transactions identified with pairs of the recipient account and the
       * bridge transaction nonce.
       **/
      bridgeTxDetails: AugmentedQuery<ApiType, (arg1: AccountId32 | string | Uint8Array, arg2: u32 | AnyNumber | Uint8Array) => Observable<PalletBridgeBridgeTxDetail>, [AccountId32, u32]> & QueryableStorageEntry<ApiType, [AccountId32, u32]>;
      /**
       * The multisig account of the bridge controller. The genesis signers accept their
       * authorizations and are able to get their proposals delivered. The bridge creator
       * transfers some POLY to their identity.
       **/
      controller: AugmentedQuery<ApiType, () => Observable<AccountId32>, []> & QueryableStorageEntry<ApiType, []>;
      /**
       * Freeze bridge admins.  These accounts can only freeze the bridge.
       **/
      freezeAdmins: AugmentedQuery<ApiType, (arg: AccountId32 | string | Uint8Array) => Observable<bool>, [AccountId32]> & QueryableStorageEntry<ApiType, [AccountId32]>;
      /**
       * Whether or not the bridge operation is frozen.
       **/
      frozen: AugmentedQuery<ApiType, () => Observable<bool>, []> & QueryableStorageEntry<ApiType, []>;
      /**
       * Amount of POLYX bridged by the identity in last block interval. Fields: the bridged
       * amount and the last interval number.
       **/
      polyxBridged: AugmentedQuery<ApiType, (arg: PolymeshPrimitivesIdentityId | string | Uint8Array) => Observable<ITuple<[u128, u32]>>, [PolymeshPrimitivesIdentityId]> & QueryableStorageEntry<ApiType, [PolymeshPrimitivesIdentityId]>;
      /**
       * Storage version.
       **/
      storageVersion: AugmentedQuery<ApiType, () => Observable<u8>, []> & QueryableStorageEntry<ApiType, []>;
      /**
       * The bridge transaction timelock period, in blocks, since the acceptance of the
       * transaction proposal during which the admin key can freeze the transaction.
       **/
      timelock: AugmentedQuery<ApiType, () => Observable<u32>, []> & QueryableStorageEntry<ApiType, []>;
      /**
       * Generic query
       **/
      [key: string]: QueryableStorageEntry<ApiType>;
    };
    capitalDistribution: {
      /**
       * All capital distributions, tied to their respective corporate actions (CAs).
       * 
       * (CAId) => Distribution
       **/
      distributions: AugmentedQuery<ApiType, (arg: PalletCorporateActionsCaId | { ticker?: any; localId?: any } | string | Uint8Array) => Observable<Option<PalletCorporateActionsDistribution>>, [PalletCorporateActionsCaId]> & QueryableStorageEntry<ApiType, [PalletCorporateActionsCaId]>;
      /**
       * Has an asset holder been paid yet?
       * 
       * (CAId, DID) -> Was DID paid in the CAId?
       **/
      holderPaid: AugmentedQuery<ApiType, (arg: ITuple<[PalletCorporateActionsCaId, PolymeshPrimitivesIdentityId]> | [PalletCorporateActionsCaId | { ticker?: any; localId?: any } | string | Uint8Array, PolymeshPrimitivesIdentityId | string | Uint8Array]) => Observable<bool>, [ITuple<[PalletCorporateActionsCaId, PolymeshPrimitivesIdentityId]>]> & QueryableStorageEntry<ApiType, [ITuple<[PalletCorporateActionsCaId, PolymeshPrimitivesIdentityId]>]>;
      /**
       * Storage version.
       **/
      storageVersion: AugmentedQuery<ApiType, () => Observable<u8>, []> & QueryableStorageEntry<ApiType, []>;
      /**
       * Generic query
       **/
      [key: string]: QueryableStorageEntry<ApiType>;
    };
    cddServiceProviders: {
      /**
       * The current "active" membership, stored as an ordered Vec.
       **/
      activeMembers: AugmentedQuery<ApiType, () => Observable<Vec<PolymeshPrimitivesIdentityId>>, []> & QueryableStorageEntry<ApiType, []>;
      /**
       * Limit of how many "active" members there can be.
       **/
      activeMembersLimit: AugmentedQuery<ApiType, () => Observable<u32>, []> & QueryableStorageEntry<ApiType, []>;
      /**
       * The current "inactive" membership, stored as an ordered Vec.
       **/
      inactiveMembers: AugmentedQuery<ApiType, () => Observable<Vec<PolymeshCommonUtilitiesGroupInactiveMember>>, []> & QueryableStorageEntry<ApiType, []>;
      /**
       * Generic query
       **/
      [key: string]: QueryableStorageEntry<ApiType>;
    };
    checkpoint: {
      /**
       * Balance of a DID at a checkpoint.
       * 
       * (ticker, did, checkpoint ID) -> Balance of a DID at a checkpoint
       **/
      balance: AugmentedQuery<ApiType, (arg1: ITuple<[PolymeshPrimitivesTicker, u64]> | [PolymeshPrimitivesTicker | string | Uint8Array, u64 | AnyNumber | Uint8Array], arg2: PolymeshPrimitivesIdentityId | string | Uint8Array) => Observable<u128>, [ITuple<[PolymeshPrimitivesTicker, u64]>, PolymeshPrimitivesIdentityId]> & QueryableStorageEntry<ApiType, [ITuple<[PolymeshPrimitivesTicker, u64]>, PolymeshPrimitivesIdentityId]>;
      /**
       * Checkpoints where a DID's balance was updated.
       * (ticker, did) -> [checkpoint ID where user balance changed]
       **/
      balanceUpdates: AugmentedQuery<ApiType, (arg1: PolymeshPrimitivesTicker | string | Uint8Array, arg2: PolymeshPrimitivesIdentityId | string | Uint8Array) => Observable<Vec<u64>>, [PolymeshPrimitivesTicker, PolymeshPrimitivesIdentityId]> & QueryableStorageEntry<ApiType, [PolymeshPrimitivesTicker, PolymeshPrimitivesIdentityId]>;
      /**
       * Checkpoints ID generator sequence.
       * ID of first checkpoint is 1 instead of 0.
       * 
       * (ticker) -> no. of checkpoints
       **/
      checkpointIdSequence: AugmentedQuery<ApiType, (arg: PolymeshPrimitivesTicker | string | Uint8Array) => Observable<u64>, [PolymeshPrimitivesTicker]> & QueryableStorageEntry<ApiType, [PolymeshPrimitivesTicker]>;
      /**
       * Checkpoint schedule ID sequence for tickers.
       * 
       * (ticker) -> schedule ID
       **/
      scheduleIdSequence: AugmentedQuery<ApiType, (arg: PolymeshPrimitivesTicker | string | Uint8Array) => Observable<u64>, [PolymeshPrimitivesTicker]> & QueryableStorageEntry<ApiType, [PolymeshPrimitivesTicker]>;
      /**
       * All the checkpoints a given schedule originated.
       * 
       * (ticker, schedule ID) -> [checkpoint ID]
       **/
      schedulePoints: AugmentedQuery<ApiType, (arg1: PolymeshPrimitivesTicker | string | Uint8Array, arg2: u64 | AnyNumber | Uint8Array) => Observable<Vec<u64>>, [PolymeshPrimitivesTicker, u64]> & QueryableStorageEntry<ApiType, [PolymeshPrimitivesTicker, u64]>;
      /**
       * How many "strong" references are there to a given `ScheduleId`?
       * 
       * The presence of a "strong" reference, in the sense of `Rc<T>`,
       * entails that the referenced schedule cannot be removed.
       * Thus, as long as `strong_ref_count(schedule_id) > 0`,
       * `remove_schedule(schedule_id)` will error.
       * 
       * (ticker, schedule ID) -> strong ref count
       **/
      scheduleRefCount: AugmentedQuery<ApiType, (arg1: PolymeshPrimitivesTicker | string | Uint8Array, arg2: u64 | AnyNumber | Uint8Array) => Observable<u32>, [PolymeshPrimitivesTicker, u64]> & QueryableStorageEntry<ApiType, [PolymeshPrimitivesTicker, u64]>;
      /**
       * Checkpoint schedules for tickers.
       * 
       * (ticker) -> [schedule]
       **/
      schedules: AugmentedQuery<ApiType, (arg: PolymeshPrimitivesTicker | string | Uint8Array) => Observable<Vec<PolymeshCommonUtilitiesCheckpointStoredSchedule>>, [PolymeshPrimitivesTicker]> & QueryableStorageEntry<ApiType, [PolymeshPrimitivesTicker]>;
      /**
       * The maximum complexity allowed for an arbitrary ticker's schedule set
       * (i.e. `Schedules` storage item below).
       **/
      schedulesMaxComplexity: AugmentedQuery<ApiType, () => Observable<u64>, []> & QueryableStorageEntry<ApiType, []>;
      /**
       * Storage version.
       **/
      storageVersion: AugmentedQuery<ApiType, () => Observable<u8>, []> & QueryableStorageEntry<ApiType, []>;
      /**
       * Checkpoint timestamps.
       * 
       * Every schedule-originated checkpoint maps its ID to its due time.
       * Every checkpoint manually created maps its ID to the time of recording.
       * 
       * (ticker) -> (checkpoint ID) -> checkpoint timestamp
       **/
      timestamps: AugmentedQuery<ApiType, (arg1: PolymeshPrimitivesTicker | string | Uint8Array, arg2: u64 | AnyNumber | Uint8Array) => Observable<u64>, [PolymeshPrimitivesTicker, u64]> & QueryableStorageEntry<ApiType, [PolymeshPrimitivesTicker, u64]>;
      /**
       * Total supply of the token at the checkpoint.
       * 
       * (ticker, checkpointId) -> total supply at given checkpoint
       **/
      totalSupply: AugmentedQuery<ApiType, (arg1: PolymeshPrimitivesTicker | string | Uint8Array, arg2: u64 | AnyNumber | Uint8Array) => Observable<u128>, [PolymeshPrimitivesTicker, u64]> & QueryableStorageEntry<ApiType, [PolymeshPrimitivesTicker, u64]>;
      /**
       * Generic query
       **/
      [key: string]: QueryableStorageEntry<ApiType>;
    };
    committeeMembership: {
      /**
       * The current "active" membership, stored as an ordered Vec.
       **/
      activeMembers: AugmentedQuery<ApiType, () => Observable<Vec<PolymeshPrimitivesIdentityId>>, []> & QueryableStorageEntry<ApiType, []>;
      /**
       * Limit of how many "active" members there can be.
       **/
      activeMembersLimit: AugmentedQuery<ApiType, () => Observable<u32>, []> & QueryableStorageEntry<ApiType, []>;
      /**
       * The current "inactive" membership, stored as an ordered Vec.
       **/
      inactiveMembers: AugmentedQuery<ApiType, () => Observable<Vec<PolymeshCommonUtilitiesGroupInactiveMember>>, []> & QueryableStorageEntry<ApiType, []>;
      /**
       * Generic query
       **/
      [key: string]: QueryableStorageEntry<ApiType>;
    };
    complianceManager: {
      /**
       * Asset compliance for a ticker (Ticker -> AssetCompliance)
       **/
      assetCompliances: AugmentedQuery<ApiType, (arg: PolymeshPrimitivesTicker | string | Uint8Array) => Observable<PolymeshPrimitivesComplianceManagerAssetCompliance>, [PolymeshPrimitivesTicker]> & QueryableStorageEntry<ApiType, [PolymeshPrimitivesTicker]>;
      /**
       * Storage version.
       **/
      storageVersion: AugmentedQuery<ApiType, () => Observable<u8>, []> & QueryableStorageEntry<ApiType, []>;
      /**
       * List of trusted claim issuer Ticker -> Issuer Identity
       **/
      trustedClaimIssuer: AugmentedQuery<ApiType, (arg: PolymeshPrimitivesTicker | string | Uint8Array) => Observable<Vec<PolymeshPrimitivesConditionTrustedIssuer>>, [PolymeshPrimitivesTicker]> & QueryableStorageEntry<ApiType, [PolymeshPrimitivesTicker]>;
      /**
       * Generic query
       **/
      [key: string]: QueryableStorageEntry<ApiType>;
    };
    contracts: {
      /**
       * A mapping between an original code hash and instrumented wasm code, ready for execution.
       **/
      codeStorage: AugmentedQuery<ApiType, (arg: H256 | string | Uint8Array) => Observable<Option<PalletContractsWasmPrefabWasmModule>>, [H256]> & QueryableStorageEntry<ApiType, [H256]>;
      /**
       * The code associated with a given account.
       * 
       * TWOX-NOTE: SAFE since `AccountId` is a secure hash.
       **/
      contractInfoOf: AugmentedQuery<ApiType, (arg: AccountId32 | string | Uint8Array) => Observable<Option<PalletContractsStorageRawContractInfo>>, [AccountId32]> & QueryableStorageEntry<ApiType, [AccountId32]>;
      /**
       * Evicted contracts that await child trie deletion.
       * 
       * Child trie deletion is a heavy operation depending on the amount of storage items
       * stored in said trie. Therefore this operation is performed lazily in `on_initialize`.
       **/
      deletionQueue: AugmentedQuery<ApiType, () => Observable<Vec<PalletContractsStorageDeletedContract>>, []> & QueryableStorageEntry<ApiType, []>;
      /**
       * This is a **monotonic** counter incremented on contract instantiation.
       * 
       * This is used in order to generate unique trie ids for contracts.
       * The trie id of a new contract is calculated from hash(account_id, nonce).
       * The nonce is required because otherwise the following sequence would lead to
       * a possible collision of storage:
       * 
       * 1. Create a new contract.
       * 2. Terminate the contract.
       * 3. Immediately recreate the contract with the same account_id.
       * 
       * This is bad because the contents of a trie are deleted lazily and there might be
       * storage of the old instantiation still in it when the new contract is created. Please
       * note that we can't replace the counter by the block number because the sequence above
       * can happen in the same block. We also can't keep the account counter in memory only
       * because storage is the only way to communicate across different extrinsics in the
       * same block.
       * 
       * # Note
       * 
       * Do not use it to determine the number of contracts. It won't be decremented if
       * a contract is destroyed.
       **/
      nonce: AugmentedQuery<ApiType, () => Observable<u64>, []> & QueryableStorageEntry<ApiType, []>;
      /**
       * A mapping between an original code hash and its owner information.
       **/
      ownerInfoOf: AugmentedQuery<ApiType, (arg: H256 | string | Uint8Array) => Observable<Option<PalletContractsWasmOwnerInfo>>, [H256]> & QueryableStorageEntry<ApiType, [H256]>;
      /**
       * A mapping from an original code hash to the original code, untouched by instrumentation.
       **/
      pristineCode: AugmentedQuery<ApiType, (arg: H256 | string | Uint8Array) => Observable<Option<Bytes>>, [H256]> & QueryableStorageEntry<ApiType, [H256]>;
      /**
       * Generic query
       **/
      [key: string]: QueryableStorageEntry<ApiType>;
    };
    corporateAction: {
      /**
       * Associations from CAs to `Document`s via their IDs.
       * (CAId => [DocumentId])
       * 
       * The `CorporateActions` map stores `Ticker => LocalId => The CA`,
       * so we can infer `Ticker => CAId`. Therefore, we don't need a double map.
       **/
      caDocLink: AugmentedQuery<ApiType, (arg: PalletCorporateActionsCaId | { ticker?: any; localId?: any } | string | Uint8Array) => Observable<Vec<u32>>, [PalletCorporateActionsCaId]> & QueryableStorageEntry<ApiType, [PalletCorporateActionsCaId]>;
      /**
       * The next per-`Ticker` CA ID in the sequence.
       * The full ID is defined as a combination of `Ticker` and a number in this sequence.
       **/
      caIdSequence: AugmentedQuery<ApiType, (arg: PolymeshPrimitivesTicker | string | Uint8Array) => Observable<u32>, [PolymeshPrimitivesTicker]> & QueryableStorageEntry<ApiType, [PolymeshPrimitivesTicker]>;
      /**
       * All recorded CAs thus far.
       * Only generic information is stored here.
       * Specific `CAKind`s, e.g., benefits and corporate ballots, may use additional on-chain storage.
       * 
       * (ticker => local ID => the corporate action)
       **/
      corporateActions: AugmentedQuery<ApiType, (arg1: PolymeshPrimitivesTicker | string | Uint8Array, arg2: u32 | AnyNumber | Uint8Array) => Observable<Option<PalletCorporateActionsCorporateAction>>, [PolymeshPrimitivesTicker, u32]> & QueryableStorageEntry<ApiType, [PolymeshPrimitivesTicker, u32]>;
      /**
       * The identities targeted by default for CAs for this ticker,
       * either to be excluded or included.
       * 
       * (ticker => target identities)
       **/
      defaultTargetIdentities: AugmentedQuery<ApiType, (arg: PolymeshPrimitivesTicker | string | Uint8Array) => Observable<PalletCorporateActionsTargetIdentities>, [PolymeshPrimitivesTicker]> & QueryableStorageEntry<ApiType, [PolymeshPrimitivesTicker]>;
      /**
       * The default amount of tax to withhold ("withholding tax", WT) for this ticker when distributing dividends.
       * 
       * To understand withholding tax, e.g., let's assume that you hold ACME shares.
       * ACME now decides to distribute 100 SEK to Alice.
       * Alice lives in Sweden, so Skatteverket (the Swedish tax authority) wants 30% of that.
       * Then those 100 * 30% are withheld from Alice, and ACME will send them to Skatteverket.
       * 
       * (ticker => % to withhold)
       **/
      defaultWithholdingTax: AugmentedQuery<ApiType, (arg: PolymeshPrimitivesTicker | string | Uint8Array) => Observable<Permill>, [PolymeshPrimitivesTicker]> & QueryableStorageEntry<ApiType, [PolymeshPrimitivesTicker]>;
      /**
       * Associates details in free-form text with a CA by its ID.
       * (CAId => CADetails)
       **/
      details: AugmentedQuery<ApiType, (arg: PalletCorporateActionsCaId | { ticker?: any; localId?: any } | string | Uint8Array) => Observable<Bytes>, [PalletCorporateActionsCaId]> & QueryableStorageEntry<ApiType, [PalletCorporateActionsCaId]>;
      /**
       * The amount of tax to withhold ("withholding tax", WT) for a certain ticker x DID.
       * If an entry exists for a certain DID, it overrides the default in `DefaultWithholdingTax`.
       * 
       * (ticker => [(did, % to withhold)]
       **/
      didWithholdingTax: AugmentedQuery<ApiType, (arg: PolymeshPrimitivesTicker | string | Uint8Array) => Observable<Vec<ITuple<[PolymeshPrimitivesIdentityId, Permill]>>>, [PolymeshPrimitivesTicker]> & QueryableStorageEntry<ApiType, [PolymeshPrimitivesTicker]>;
      /**
       * Determines the maximum number of bytes that the free-form `details` of a CA can store.
       * 
       * Note that this is not the number of `char`s or the number of [graphemes].
       * While this may be unnatural in terms of human understanding of a text's length,
       * it more closely reflects actual storage costs (`'a'` is cheaper to store than an emoji).
       * 
       * [graphemes]: https://en.wikipedia.org/wiki/Grapheme
       **/
      maxDetailsLength: AugmentedQuery<ApiType, () => Observable<u32>, []> & QueryableStorageEntry<ApiType, []>;
      /**
       * Storage version.
       **/
      storageVersion: AugmentedQuery<ApiType, () => Observable<u8>, []> & QueryableStorageEntry<ApiType, []>;
      /**
       * Generic query
       **/
      [key: string]: QueryableStorageEntry<ApiType>;
    };
    corporateBallot: {
      /**
       * Metadata of a corporate ballot.
       * 
       * (CAId) => BallotMeta
       **/
      metas: AugmentedQuery<ApiType, (arg: PalletCorporateActionsCaId | { ticker?: any; localId?: any } | string | Uint8Array) => Observable<Option<PalletCorporateActionsBallotBallotMeta>>, [PalletCorporateActionsCaId]> & QueryableStorageEntry<ApiType, [PalletCorporateActionsCaId]>;
      /**
       * Stores how many choices there are in each motion.
       * 
       * At all times, the invariant holds that `motion_choices[idx]` is equal to
       * `metas.unwrap().motions[idx].choices.len()`. That is, this is just a cache,
       * used to avoid fetching all the motions with their associated texts.
       * 
       * `u16` choices should be more than enough to fit real use cases.
       * 
       * (CAId) => Number of choices in each motion.
       **/
      motionNumChoices: AugmentedQuery<ApiType, (arg: PalletCorporateActionsCaId | { ticker?: any; localId?: any } | string | Uint8Array) => Observable<Vec<u16>>, [PalletCorporateActionsCaId]> & QueryableStorageEntry<ApiType, [PalletCorporateActionsCaId]>;
      /**
       * Is ranked choice voting (RCV) enabled for this ballot?
       * For an understanding of how RCV is handled, see note on `BallotVote`'s `fallback` field.
       * 
       * (CAId) => bool
       **/
      rcv: AugmentedQuery<ApiType, (arg: PalletCorporateActionsCaId | { ticker?: any; localId?: any } | string | Uint8Array) => Observable<bool>, [PalletCorporateActionsCaId]> & QueryableStorageEntry<ApiType, [PalletCorporateActionsCaId]>;
      /**
       * Stores the total vote tally on each choice.
       * 
       * RCV is not accounted for,
       * as there are too many wants to interpret the graph,
       * and because it would not be efficient.
       * 
       * (CAId) => [current vote weights]
       **/
      results: AugmentedQuery<ApiType, (arg: PalletCorporateActionsCaId | { ticker?: any; localId?: any } | string | Uint8Array) => Observable<Vec<u128>>, [PalletCorporateActionsCaId]> & QueryableStorageEntry<ApiType, [PalletCorporateActionsCaId]>;
      /**
       * Time details of a corporate ballot associated with a CA.
       * The timestamps denote when voting starts and stops.
       * 
       * (CAId) => BallotTimeRange
       **/
      timeRanges: AugmentedQuery<ApiType, (arg: PalletCorporateActionsCaId | { ticker?: any; localId?: any } | string | Uint8Array) => Observable<Option<PalletCorporateActionsBallotBallotTimeRange>>, [PalletCorporateActionsCaId]> & QueryableStorageEntry<ApiType, [PalletCorporateActionsCaId]>;
      /**
       * Stores each DID's votes in a given ballot.
       * See the documentation of `BallotVote` for notes on semantics.
       * 
       * (CAId) => (DID) => [vote weight]
       * 
       * User must enter 0 vote weight if they don't want to vote for a choice.
       **/
      votes: AugmentedQuery<ApiType, (arg1: PalletCorporateActionsCaId | { ticker?: any; localId?: any } | string | Uint8Array, arg2: PolymeshPrimitivesIdentityId | string | Uint8Array) => Observable<Vec<PalletCorporateActionsBallotBallotVote>>, [PalletCorporateActionsCaId, PolymeshPrimitivesIdentityId]> & QueryableStorageEntry<ApiType, [PalletCorporateActionsCaId, PolymeshPrimitivesIdentityId]>;
      /**
       * Generic query
       **/
      [key: string]: QueryableStorageEntry<ApiType>;
    };
    externalAgents: {
      /**
       * Maps an agent (`IdentityId`) to all all `Ticker`s they belong to, if any.
       **/
      agentOf: AugmentedQuery<ApiType, (arg1: PolymeshPrimitivesIdentityId | string | Uint8Array, arg2: PolymeshPrimitivesTicker | string | Uint8Array) => Observable<Null>, [PolymeshPrimitivesIdentityId, PolymeshPrimitivesTicker]> & QueryableStorageEntry<ApiType, [PolymeshPrimitivesIdentityId, PolymeshPrimitivesTicker]>;
      /**
       * The next per-`Ticker` AG ID in the sequence.
       * 
       * The full ID is defined as a combination of `Ticker` and a number in this sequence,
       * which starts from 1, rather than 0.
       **/
      agIdSequence: AugmentedQuery<ApiType, (arg: PolymeshPrimitivesTicker | string | Uint8Array) => Observable<u32>, [PolymeshPrimitivesTicker]> & QueryableStorageEntry<ApiType, [PolymeshPrimitivesTicker]>;
      /**
       * Maps agents (`IdentityId`) for a `Ticker` to what AG they belong to, if any.
       **/
      groupOfAgent: AugmentedQuery<ApiType, (arg1: PolymeshPrimitivesTicker | string | Uint8Array, arg2: PolymeshPrimitivesIdentityId | string | Uint8Array) => Observable<Option<PolymeshPrimitivesAgentAgentGroup>>, [PolymeshPrimitivesTicker, PolymeshPrimitivesIdentityId]> & QueryableStorageEntry<ApiType, [PolymeshPrimitivesTicker, PolymeshPrimitivesIdentityId]>;
      /**
       * For custom AGs of a `Ticker`, maps to what permissions an agent in that AG would have.
       **/
      groupPermissions: AugmentedQuery<ApiType, (arg1: PolymeshPrimitivesTicker | string | Uint8Array, arg2: u32 | AnyNumber | Uint8Array) => Observable<Option<PolymeshPrimitivesSubsetSubsetRestrictionPalletPermissions>>, [PolymeshPrimitivesTicker, u32]> & QueryableStorageEntry<ApiType, [PolymeshPrimitivesTicker, u32]>;
      /**
       * Maps a `Ticker` to the number of `Full` agents for it.
       **/
      numFullAgents: AugmentedQuery<ApiType, (arg: PolymeshPrimitivesTicker | string | Uint8Array) => Observable<u32>, [PolymeshPrimitivesTicker]> & QueryableStorageEntry<ApiType, [PolymeshPrimitivesTicker]>;
      /**
       * Generic query
       **/
      [key: string]: QueryableStorageEntry<ApiType>;
    };
    grandpa: {
      /**
       * The number of changes (both in terms of keys and underlying economic responsibilities)
       * in the "set" of Grandpa validators from genesis.
       **/
      currentSetId: AugmentedQuery<ApiType, () => Observable<u64>, []> & QueryableStorageEntry<ApiType, []>;
      /**
       * next block number where we can force a change.
       **/
      nextForced: AugmentedQuery<ApiType, () => Observable<Option<u32>>, []> & QueryableStorageEntry<ApiType, []>;
      /**
       * Pending change: (signaled at, scheduled change).
       **/
      pendingChange: AugmentedQuery<ApiType, () => Observable<Option<PalletGrandpaStoredPendingChange>>, []> & QueryableStorageEntry<ApiType, []>;
      /**
       * A mapping from grandpa set ID to the index of the *most recent* session for which its
       * members were responsible.
       * 
       * TWOX-NOTE: `SetId` is not under user control.
       **/
      setIdSession: AugmentedQuery<ApiType, (arg: u64 | AnyNumber | Uint8Array) => Observable<Option<u32>>, [u64]> & QueryableStorageEntry<ApiType, [u64]>;
      /**
       * `true` if we are currently stalled.
       **/
      stalled: AugmentedQuery<ApiType, () => Observable<Option<ITuple<[u32, u32]>>>, []> & QueryableStorageEntry<ApiType, []>;
      /**
       * State of the current authority set.
       **/
      state: AugmentedQuery<ApiType, () => Observable<PalletGrandpaStoredState>, []> & QueryableStorageEntry<ApiType, []>;
      /**
       * Generic query
       **/
      [key: string]: QueryableStorageEntry<ApiType>;
    };
    identity: {
      /**
       * How many "strong" references to the account key.
       * 
       * Strong references will block a key from leaving it's identity.
       * 
       * Pallets using "strong" references to account keys:
       * * Relayer: For `user_key` and `paying_key`
       * 
       **/
      accountKeyRefCount: AugmentedQuery<ApiType, (arg: AccountId32 | string | Uint8Array) => Observable<u64>, [AccountId32]> & QueryableStorageEntry<ApiType, [AccountId32]>;
      /**
       * All authorizations that an identity/key has
       **/
      authorizations: AugmentedQuery<ApiType, (arg1: PolymeshPrimitivesSecondaryKeySignatory | { Identity: any } | { Account: any } | string | Uint8Array, arg2: u64 | AnyNumber | Uint8Array) => Observable<Option<PolymeshPrimitivesAuthorization>>, [PolymeshPrimitivesSecondaryKeySignatory, u64]> & QueryableStorageEntry<ApiType, [PolymeshPrimitivesSecondaryKeySignatory, u64]>;
      /**
       * All authorizations that an identity has given. (Authorizer, auth_id -> authorized)
       **/
      authorizationsGiven: AugmentedQuery<ApiType, (arg1: PolymeshPrimitivesIdentityId | string | Uint8Array, arg2: u64 | AnyNumber | Uint8Array) => Observable<PolymeshPrimitivesSecondaryKeySignatory>, [PolymeshPrimitivesIdentityId, u64]> & QueryableStorageEntry<ApiType, [PolymeshPrimitivesIdentityId, u64]>;
      /**
       * Obsoleted storage variable superceded by `CddAuthForPrimaryKeyRotation`. It is kept here
       * for the purpose of storage migration.
       **/
      cddAuthForMasterKeyRotation: AugmentedQuery<ApiType, () => Observable<bool>, []> & QueryableStorageEntry<ApiType, []>;
      /**
       * A config flag that, if set, instructs an authorization from a CDD provider in order to
       * change the primary key of an identity.
       **/
      cddAuthForPrimaryKeyRotation: AugmentedQuery<ApiType, () => Observable<bool>, []> & QueryableStorageEntry<ApiType, []>;
      /**
       * (Target ID, claim type) (issuer,scope) -> Associated claims
       **/
      claims: AugmentedQuery<ApiType, (arg1: PalletIdentityClaim1stKey | { target?: any; claimType?: any } | string | Uint8Array, arg2: PalletIdentityClaim2ndKey | { issuer?: any; scope?: any } | string | Uint8Array) => Observable<PolymeshPrimitivesIdentityClaim>, [PalletIdentityClaim1stKey, PalletIdentityClaim2ndKey]> & QueryableStorageEntry<ApiType, [PalletIdentityClaim1stKey, PalletIdentityClaim2ndKey]>;
      /**
       * It stores the current identity for current transaction.
       **/
      currentDid: AugmentedQuery<ApiType, () => Observable<Option<U8aFixed>>, []> & QueryableStorageEntry<ApiType, []>;
      /**
       * It stores the current gas fee payer for the current transaction
       **/
      currentPayer: AugmentedQuery<ApiType, () => Observable<Option<AccountId32>>, []> & QueryableStorageEntry<ApiType, []>;
      /**
       * A reverse double map to allow finding all keys for an identity.
       **/
      didKeys: AugmentedQuery<ApiType, (arg1: PolymeshPrimitivesIdentityId | string | Uint8Array, arg2: AccountId32 | string | Uint8Array) => Observable<bool>, [PolymeshPrimitivesIdentityId, AccountId32]> & QueryableStorageEntry<ApiType, [PolymeshPrimitivesIdentityId, AccountId32]>;
      /**
       * DID -> identity info
       **/
      didRecords: AugmentedQuery<ApiType, (arg: PolymeshPrimitivesIdentityId | string | Uint8Array) => Observable<Option<PolymeshPrimitivesIdentityDidRecord>>, [PolymeshPrimitivesIdentityId]> & QueryableStorageEntry<ApiType, [PolymeshPrimitivesIdentityId]>;
      /**
       * DID -> bool that indicates if secondary keys are frozen.
       **/
      isDidFrozen: AugmentedQuery<ApiType, (arg: PolymeshPrimitivesIdentityId | string | Uint8Array) => Observable<bool>, [PolymeshPrimitivesIdentityId]> & QueryableStorageEntry<ApiType, [PolymeshPrimitivesIdentityId]>;
      /**
       * Map from AccountId to `KeyRecord` that holds the key's identity and permissions.
       **/
      keyRecords: AugmentedQuery<ApiType, (arg: AccountId32 | string | Uint8Array) => Observable<Option<PolymeshPrimitivesSecondaryKeyKeyRecord>>, [AccountId32]> & QueryableStorageEntry<ApiType, [AccountId32]>;
      /**
       * Nonce to ensure unique actions. starts from 1.
       **/
      multiPurposeNonce: AugmentedQuery<ApiType, () => Observable<u64>, []> & QueryableStorageEntry<ApiType, []>;
      /**
       * Authorization nonce per Identity. Initially is 0.
       **/
      offChainAuthorizationNonce: AugmentedQuery<ApiType, (arg: PolymeshPrimitivesIdentityId | string | Uint8Array) => Observable<u64>, [PolymeshPrimitivesIdentityId]> & QueryableStorageEntry<ApiType, [PolymeshPrimitivesIdentityId]>;
      /**
       * Storage version.
       **/
      storageVersion: AugmentedQuery<ApiType, () => Observable<u8>, []> & QueryableStorageEntry<ApiType, []>;
      /**
       * Generic query
       **/
      [key: string]: QueryableStorageEntry<ApiType>;
    };
    imOnline: {
      /**
       * For each session index, we keep a mapping of `ValidatorId<T>` to the
       * number of blocks authored by the given authority.
       **/
      authoredBlocks: AugmentedQuery<ApiType, (arg1: u32 | AnyNumber | Uint8Array, arg2: AccountId32 | string | Uint8Array) => Observable<u32>, [u32, AccountId32]> & QueryableStorageEntry<ApiType, [u32, AccountId32]>;
      /**
       * The block number after which it's ok to send heartbeats in the current
       * session.
       * 
       * At the beginning of each session we set this to a value that should fall
       * roughly in the middle of the session duration. The idea is to first wait for
       * the validators to produce a block in the current session, so that the
       * heartbeat later on will not be necessary.
       * 
       * This value will only be used as a fallback if we fail to get a proper session
       * progress estimate from `NextSessionRotation`, as those estimates should be
       * more accurate then the value we calculate for `HeartbeatAfter`.
       **/
      heartbeatAfter: AugmentedQuery<ApiType, () => Observable<u32>, []> & QueryableStorageEntry<ApiType, []>;
      /**
       * The current set of keys that may issue a heartbeat.
       **/
      keys: AugmentedQuery<ApiType, () => Observable<Vec<PalletImOnlineSr25519AppSr25519Public>>, []> & QueryableStorageEntry<ApiType, []>;
      /**
       * For each session index, we keep a mapping of `SessionIndex` and `AuthIndex` to
       * `WrapperOpaque<BoundedOpaqueNetworkState>`.
       **/
      receivedHeartbeats: AugmentedQuery<ApiType, (arg1: u32 | AnyNumber | Uint8Array, arg2: u32 | AnyNumber | Uint8Array) => Observable<Option<WrapperOpaque<PalletImOnlineBoundedOpaqueNetworkState>>>, [u32, u32]> & QueryableStorageEntry<ApiType, [u32, u32]>;
      /**
       * Generic query
       **/
      [key: string]: QueryableStorageEntry<ApiType>;
    };
    indices: {
      /**
       * The lookup from index to account.
       **/
      accounts: AugmentedQuery<ApiType, (arg: u32 | AnyNumber | Uint8Array) => Observable<Option<ITuple<[AccountId32, u128, bool]>>>, [u32]> & QueryableStorageEntry<ApiType, [u32]>;
      /**
       * Generic query
       **/
      [key: string]: QueryableStorageEntry<ApiType>;
    };
    multiSig: {
      /**
       * Nonce to ensure unique MultiSig addresses are generated; starts from 1.
       **/
      multiSigNonce: AugmentedQuery<ApiType, () => Observable<u64>, []> & QueryableStorageEntry<ApiType, []>;
      /**
       * Signers of a multisig. (multisig, signer) => signer.
       **/
      multiSigSigners: AugmentedQuery<ApiType, (arg1: AccountId32 | string | Uint8Array, arg2: PolymeshPrimitivesSecondaryKeySignatory | { Identity: any } | { Account: any } | string | Uint8Array) => Observable<PolymeshPrimitivesSecondaryKeySignatory>, [AccountId32, PolymeshPrimitivesSecondaryKeySignatory]> & QueryableStorageEntry<ApiType, [AccountId32, PolymeshPrimitivesSecondaryKeySignatory]>;
      /**
       * Confirmations required before processing a multisig tx.
       **/
      multiSigSignsRequired: AugmentedQuery<ApiType, (arg: AccountId32 | string | Uint8Array) => Observable<u64>, [AccountId32]> & QueryableStorageEntry<ApiType, [AccountId32]>;
      /**
       * Maps a multisig account to its identity.
       **/
      multiSigToIdentity: AugmentedQuery<ApiType, (arg: AccountId32 | string | Uint8Array) => Observable<U8aFixed>, [AccountId32]> & QueryableStorageEntry<ApiType, [AccountId32]>;
      /**
       * Number of transactions proposed in a multisig. Used as tx id; starts from 0.
       **/
      multiSigTxDone: AugmentedQuery<ApiType, (arg: AccountId32 | string | Uint8Array) => Observable<u64>, [AccountId32]> & QueryableStorageEntry<ApiType, [AccountId32]>;
      /**
       * Number of approved/accepted signers of a multisig.
       **/
      numberOfSigners: AugmentedQuery<ApiType, (arg: AccountId32 | string | Uint8Array) => Observable<u64>, [AccountId32]> & QueryableStorageEntry<ApiType, [AccountId32]>;
      /**
       * Details of a multisig proposal
       **/
      proposalDetail: AugmentedQuery<ApiType, (arg: ITuple<[AccountId32, u64]> | [AccountId32 | string | Uint8Array, u64 | AnyNumber | Uint8Array]) => Observable<PalletMultisigProposalDetails>, [ITuple<[AccountId32, u64]>]> & QueryableStorageEntry<ApiType, [ITuple<[AccountId32, u64]>]>;
      /**
       * A mapping of proposals to their IDs.
       **/
      proposalIds: AugmentedQuery<ApiType, (arg1: AccountId32 | string | Uint8Array, arg2: Call | { callIndex?: any; args?: any } | string | Uint8Array) => Observable<Option<u64>>, [AccountId32, Call]> & QueryableStorageEntry<ApiType, [AccountId32, Call]>;
      /**
       * Proposals presented for voting to a multisig (multisig, proposal id) => Option<T::Proposal>.
       **/
      proposals: AugmentedQuery<ApiType, (arg: ITuple<[AccountId32, u64]> | [AccountId32 | string | Uint8Array, u64 | AnyNumber | Uint8Array]) => Observable<Option<Call>>, [ITuple<[AccountId32, u64]>]> & QueryableStorageEntry<ApiType, [ITuple<[AccountId32, u64]>]>;
      /**
       * Storage version.
       **/
      storageVersion: AugmentedQuery<ApiType, () => Observable<u8>, []> & QueryableStorageEntry<ApiType, []>;
      /**
       * The last transaction version, used for `on_runtime_upgrade`.
       **/
      transactionVersion: AugmentedQuery<ApiType, () => Observable<u32>, []> & QueryableStorageEntry<ApiType, []>;
      /**
       * Individual multisig signer votes. (multi sig, signer, proposal) => vote.
       **/
      votes: AugmentedQuery<ApiType, (arg: ITuple<[AccountId32, PolymeshPrimitivesSecondaryKeySignatory, u64]> | [AccountId32 | string | Uint8Array, PolymeshPrimitivesSecondaryKeySignatory | { Identity: any } | { Account: any } | string | Uint8Array, u64 | AnyNumber | Uint8Array]) => Observable<bool>, [ITuple<[AccountId32, PolymeshPrimitivesSecondaryKeySignatory, u64]>]> & QueryableStorageEntry<ApiType, [ITuple<[AccountId32, PolymeshPrimitivesSecondaryKeySignatory, u64]>]>;
      /**
       * Generic query
       **/
      [key: string]: QueryableStorageEntry<ApiType>;
    };
    offences: {
      /**
       * A vector of reports of the same kind that happened at the same time slot.
       **/
      concurrentReportsIndex: AugmentedQuery<ApiType, (arg1: U8aFixed | string | Uint8Array, arg2: Bytes | string | Uint8Array) => Observable<Vec<H256>>, [U8aFixed, Bytes]> & QueryableStorageEntry<ApiType, [U8aFixed, Bytes]>;
      /**
       * The primary structure that holds all offence records keyed by report identifiers.
       **/
      reports: AugmentedQuery<ApiType, (arg: H256 | string | Uint8Array) => Observable<Option<SpStakingOffenceOffenceDetails>>, [H256]> & QueryableStorageEntry<ApiType, [H256]>;
      /**
       * Enumerates all reports of a kind along with the time they happened.
       * 
       * All reports are sorted by the time of offence.
       * 
       * Note that the actual type of this mapping is `Vec<u8>`, this is because values of
       * different types are not supported at the moment so we are doing the manual serialization.
       **/
      reportsByKindIndex: AugmentedQuery<ApiType, (arg: U8aFixed | string | Uint8Array) => Observable<Bytes>, [U8aFixed]> & QueryableStorageEntry<ApiType, [U8aFixed]>;
      /**
       * Generic query
       **/
      [key: string]: QueryableStorageEntry<ApiType>;
    };
    pips: {
      /**
       * Total count of current pending or scheduled PIPs.
       **/
      activePipCount: AugmentedQuery<ApiType, () => Observable<u32>, []> & QueryableStorageEntry<ApiType, []>;
      /**
       * The maximum allowed number for `ActivePipCount`.
       * Once reached, new PIPs cannot be proposed by community members.
       **/
      activePipLimit: AugmentedQuery<ApiType, () => Observable<u32>, []> & QueryableStorageEntry<ApiType, []>;
      /**
       * All existing PIPs where the proposer is a committee.
       * This list is a cache of all ids in `Proposals` with `Proposer::Committee(_)`.
       **/
      committeePips: AugmentedQuery<ApiType, () => Observable<Vec<u32>>, []> & QueryableStorageEntry<ApiType, []>;
      /**
       * Default enactment period that will be use after a proposal is accepted by GC.
       **/
      defaultEnactmentPeriod: AugmentedQuery<ApiType, () => Observable<u32>, []> & QueryableStorageEntry<ApiType, []>;
      /**
       * Those who have locked a deposit.
       * proposal (id, proposer) -> deposit
       **/
      deposits: AugmentedQuery<ApiType, (arg1: u32 | AnyNumber | Uint8Array, arg2: AccountId32 | string | Uint8Array) => Observable<PalletPipsDepositInfo>, [u32, AccountId32]> & QueryableStorageEntry<ApiType, [u32, AccountId32]>;
      /**
       * A live priority queue (lowest priority at index 0)
       * of pending PIPs up to the active limit.
       * Priority is defined by the `weight` in the `SnapshottedPip`.
       * 
       * Unlike `SnapshotQueue`, this queue is live, getting updated with each vote cast.
       * The snapshot is therefore essentially a point-in-time clone of this queue.
       **/
      liveQueue: AugmentedQuery<ApiType, () => Observable<Vec<PalletPipsSnapshottedPip>>, []> & QueryableStorageEntry<ApiType, []>;
      /**
       * Maximum times a PIP can be skipped before triggering `CannotSkipPip` in `enact_snapshot_results`.
       **/
      maxPipSkipCount: AugmentedQuery<ApiType, () => Observable<u8>, []> & QueryableStorageEntry<ApiType, []>;
      /**
       * The minimum amount to be used as a deposit for community PIP creation.
       **/
      minimumProposalDeposit: AugmentedQuery<ApiType, () => Observable<u128>, []> & QueryableStorageEntry<ApiType, []>;
      /**
       * How many blocks will it take, after a `Pending` PIP expires,
       * assuming it has not transitioned to another `ProposalState`?
       **/
      pendingPipExpiry: AugmentedQuery<ApiType, () => Observable<PolymeshCommonUtilitiesMaybeBlock>, []> & QueryableStorageEntry<ApiType, []>;
      /**
       * Proposals so far. id can be used to keep track of PIPs off-chain.
       **/
      pipIdSequence: AugmentedQuery<ApiType, () => Observable<u32>, []> & QueryableStorageEntry<ApiType, []>;
      /**
       * The number of times a certain PIP has been skipped.
       * Once a (configurable) threshhold is exceeded, a PIP cannot be skipped again.
       **/
      pipSkipCount: AugmentedQuery<ApiType, (arg: u32 | AnyNumber | Uint8Array) => Observable<u8>, [u32]> & QueryableStorageEntry<ApiType, [u32]>;
      /**
       * Maps PIPs to the block at which they will be executed, if any.
       **/
      pipToSchedule: AugmentedQuery<ApiType, (arg: u32 | AnyNumber | Uint8Array) => Observable<Option<u32>>, [u32]> & QueryableStorageEntry<ApiType, [u32]>;
      /**
       * The metadata of the active proposals.
       **/
      proposalMetadata: AugmentedQuery<ApiType, (arg: u32 | AnyNumber | Uint8Array) => Observable<Option<PalletPipsPipsMetadata>>, [u32]> & QueryableStorageEntry<ApiType, [u32]>;
      /**
       * PolymeshVotes on a given proposal, if it is ongoing.
       * proposal id -> vote count
       **/
      proposalResult: AugmentedQuery<ApiType, (arg: u32 | AnyNumber | Uint8Array) => Observable<PalletPipsVotingResult>, [u32]> & QueryableStorageEntry<ApiType, [u32]>;
      /**
       * Actual proposal for a given id, if it's current.
       * proposal id -> proposal
       **/
      proposals: AugmentedQuery<ApiType, (arg: u32 | AnyNumber | Uint8Array) => Observable<Option<PalletPipsPip>>, [u32]> & QueryableStorageEntry<ApiType, [u32]>;
      /**
       * Votes per Proposal and account. Used to avoid double vote issue.
       * (proposal id, account) -> Vote
       **/
      proposalVotes: AugmentedQuery<ApiType, (arg1: u32 | AnyNumber | Uint8Array, arg2: AccountId32 | string | Uint8Array) => Observable<Option<ITuple<[bool, u128]>>>, [u32, AccountId32]> & QueryableStorageEntry<ApiType, [u32, AccountId32]>;
      /**
       * Determines whether historical PIP data is persisted or removed
       **/
      pruneHistoricalPips: AugmentedQuery<ApiType, () => Observable<bool>, []> & QueryableStorageEntry<ApiType, []>;
      /**
       * Snapshots so far. id can be used to keep track of snapshots off-chain.
       **/
      snapshotIdSequence: AugmentedQuery<ApiType, () => Observable<u32>, []> & QueryableStorageEntry<ApiType, []>;
      /**
       * The metadata of the snapshot, if there is one.
       **/
      snapshotMeta: AugmentedQuery<ApiType, () => Observable<Option<PalletPipsSnapshotMetadata>>, []> & QueryableStorageEntry<ApiType, []>;
      /**
       * The priority queue (lowest priority at index 0) of PIPs at the point of snapshotting.
       * Priority is defined by the `weight` in the `SnapshottedPip`.
       * 
       * A queued PIP can be skipped. Doing so bumps the `pip_skip_count`.
       * Once a (configurable) threshhold is exceeded, a PIP cannot be skipped again.
       **/
      snapshotQueue: AugmentedQuery<ApiType, () => Observable<Vec<PalletPipsSnapshottedPip>>, []> & QueryableStorageEntry<ApiType, []>;
      storageVersion: AugmentedQuery<ApiType, () => Observable<u8>, []> & QueryableStorageEntry<ApiType, []>;
      /**
       * Generic query
       **/
      [key: string]: QueryableStorageEntry<ApiType>;
    };
    polymeshCommittee: {
      /**
       * Time after which a proposal will expire.
       **/
      expiresAfter: AugmentedQuery<ApiType, () => Observable<PolymeshCommonUtilitiesMaybeBlock>, []> & QueryableStorageEntry<ApiType, []>;
      /**
       * The current members of the committee.
       **/
      members: AugmentedQuery<ApiType, () => Observable<Vec<PolymeshPrimitivesIdentityId>>, []> & QueryableStorageEntry<ApiType, []>;
      /**
       * Proposals so far.
       **/
      proposalCount: AugmentedQuery<ApiType, () => Observable<u32>, []> & QueryableStorageEntry<ApiType, []>;
      /**
       * Actual proposal for a given hash.
       **/
      proposalOf: AugmentedQuery<ApiType, (arg: H256 | string | Uint8Array) => Observable<Option<Call>>, [H256]> & QueryableStorageEntry<ApiType, [H256]>;
      /**
       * The hashes of the active proposals.
       **/
      proposals: AugmentedQuery<ApiType, () => Observable<Vec<H256>>, []> & QueryableStorageEntry<ApiType, []>;
      /**
       * Release coordinator.
       **/
      releaseCoordinator: AugmentedQuery<ApiType, () => Observable<Option<U8aFixed>>, []> & QueryableStorageEntry<ApiType, []>;
      /**
       * Storage version.
       **/
      storageVersion: AugmentedQuery<ApiType, () => Observable<u8>, []> & QueryableStorageEntry<ApiType, []>;
      /**
       * Vote threshold for an approval.
       **/
      voteThreshold: AugmentedQuery<ApiType, () => Observable<ITuple<[u32, u32]>>, []> & QueryableStorageEntry<ApiType, []>;
      /**
       * PolymeshVotes on a given proposal, if it is ongoing.
       **/
      voting: AugmentedQuery<ApiType, (arg: H256 | string | Uint8Array) => Observable<Option<PalletCommitteePolymeshVotes>>, [H256]> & QueryableStorageEntry<ApiType, [H256]>;
      /**
       * Generic query
       **/
      [key: string]: QueryableStorageEntry<ApiType>;
    };
    polymeshContracts: {
      /**
       * Generic query
       **/
      [key: string]: QueryableStorageEntry<ApiType>;
    };
    portfolio: {
      /**
       * Inverse map of `Portfolios` used to ensure bijectivitiy,
       * and uniqueness of names in `Portfolios`.
       **/
      nameToNumber: AugmentedQuery<ApiType, (arg1: PolymeshPrimitivesIdentityId | string | Uint8Array, arg2: Bytes | string | Uint8Array) => Observable<u64>, [PolymeshPrimitivesIdentityId, Bytes]> & QueryableStorageEntry<ApiType, [PolymeshPrimitivesIdentityId, Bytes]>;
      /**
       * The next portfolio sequence number of an identity.
       **/
      nextPortfolioNumber: AugmentedQuery<ApiType, (arg: PolymeshPrimitivesIdentityId | string | Uint8Array) => Observable<u64>, [PolymeshPrimitivesIdentityId]> & QueryableStorageEntry<ApiType, [PolymeshPrimitivesIdentityId]>;
      /**
       * The asset balances of portfolios.
       **/
      portfolioAssetBalances: AugmentedQuery<ApiType, (arg1: PolymeshPrimitivesIdentityIdPortfolioId | { did?: any; kind?: any } | string | Uint8Array, arg2: PolymeshPrimitivesTicker | string | Uint8Array) => Observable<u128>, [PolymeshPrimitivesIdentityIdPortfolioId, PolymeshPrimitivesTicker]> & QueryableStorageEntry<ApiType, [PolymeshPrimitivesIdentityIdPortfolioId, PolymeshPrimitivesTicker]>;
      /**
       * How many assets with non-zero balance this portfolio contains.
       **/
      portfolioAssetCount: AugmentedQuery<ApiType, (arg: PolymeshPrimitivesIdentityIdPortfolioId | { did?: any; kind?: any } | string | Uint8Array) => Observable<u64>, [PolymeshPrimitivesIdentityIdPortfolioId]> & QueryableStorageEntry<ApiType, [PolymeshPrimitivesIdentityIdPortfolioId]>;
      /**
       * The custodian of a particular portfolio. None implies that the identity owner is the custodian.
       **/
      portfolioCustodian: AugmentedQuery<ApiType, (arg: PolymeshPrimitivesIdentityIdPortfolioId | { did?: any; kind?: any } | string | Uint8Array) => Observable<Option<U8aFixed>>, [PolymeshPrimitivesIdentityIdPortfolioId]> & QueryableStorageEntry<ApiType, [PolymeshPrimitivesIdentityIdPortfolioId]>;
      /**
       * Amount of assets locked in a portfolio.
       * These assets show up in portfolio balance but can not be transferred away.
       **/
      portfolioLockedAssets: AugmentedQuery<ApiType, (arg1: PolymeshPrimitivesIdentityIdPortfolioId | { did?: any; kind?: any } | string | Uint8Array, arg2: PolymeshPrimitivesTicker | string | Uint8Array) => Observable<u128>, [PolymeshPrimitivesIdentityIdPortfolioId, PolymeshPrimitivesTicker]> & QueryableStorageEntry<ApiType, [PolymeshPrimitivesIdentityIdPortfolioId, PolymeshPrimitivesTicker]>;
      /**
       * The set of existing portfolios with their names. If a certain pair of a DID and
       * portfolio number maps to `None` then such a portfolio doesn't exist. Conversely, if a
       * pair maps to `Some(name)` then such a portfolio exists and is called `name`.
       **/
      portfolios: AugmentedQuery<ApiType, (arg1: PolymeshPrimitivesIdentityId | string | Uint8Array, arg2: u64 | AnyNumber | Uint8Array) => Observable<Bytes>, [PolymeshPrimitivesIdentityId, u64]> & QueryableStorageEntry<ApiType, [PolymeshPrimitivesIdentityId, u64]>;
      /**
       * Tracks all the portfolios in custody of a particular identity. Only used by the UIs.
       * When `true` is stored as the value for a given `(did, pid)`, it means that `pid` is in custody of `did`.
       * `false` values are never explicitly stored in the map, and are instead inferred by the absence of a key.
       **/
      portfoliosInCustody: AugmentedQuery<ApiType, (arg1: PolymeshPrimitivesIdentityId | string | Uint8Array, arg2: PolymeshPrimitivesIdentityIdPortfolioId | { did?: any; kind?: any } | string | Uint8Array) => Observable<bool>, [PolymeshPrimitivesIdentityId, PolymeshPrimitivesIdentityIdPortfolioId]> & QueryableStorageEntry<ApiType, [PolymeshPrimitivesIdentityId, PolymeshPrimitivesIdentityIdPortfolioId]>;
      /**
       * Storage version.
       **/
      storageVersion: AugmentedQuery<ApiType, () => Observable<u8>, []> & QueryableStorageEntry<ApiType, []>;
      /**
       * Generic query
       **/
      [key: string]: QueryableStorageEntry<ApiType>;
    };
    preimage: {
      /**
       * The preimages stored by this pallet.
       **/
      preimageFor: AugmentedQuery<ApiType, (arg: H256 | string | Uint8Array) => Observable<Option<Bytes>>, [H256]> & QueryableStorageEntry<ApiType, [H256]>;
      /**
       * The request status of a given hash.
       **/
      statusFor: AugmentedQuery<ApiType, (arg: H256 | string | Uint8Array) => Observable<Option<PalletPreimageRequestStatus>>, [H256]> & QueryableStorageEntry<ApiType, [H256]>;
      /**
       * Generic query
       **/
      [key: string]: QueryableStorageEntry<ApiType>;
    };
    protocolFee: {
      /**
       * The mapping of operation names to the base fees of those operations.
       **/
      baseFees: AugmentedQuery<ApiType, (arg: PolymeshCommonUtilitiesProtocolFeeProtocolOp | 'AssetRegisterTicker' | 'AssetIssue' | 'AssetAddDocuments' | 'AssetCreateAsset' | 'CheckpointCreateSchedule' | 'ComplianceManagerAddComplianceRequirement' | 'IdentityCddRegisterDid' | 'IdentityAddClaim' | 'IdentityAddSecondaryKeysWithAuthorization' | 'PipsPropose' | 'ContractsPutCode' | 'CorporateBallotAttachBallot' | 'CapitalDistributionDistribute' | number | Uint8Array) => Observable<u128>, [PolymeshCommonUtilitiesProtocolFeeProtocolOp]> & QueryableStorageEntry<ApiType, [PolymeshCommonUtilitiesProtocolFeeProtocolOp]>;
      /**
       * The fee coefficient as a positive rational (numerator, denominator).
       **/
      coefficient: AugmentedQuery<ApiType, () => Observable<ITuple<[u32, u32]>>, []> & QueryableStorageEntry<ApiType, []>;
      /**
       * Generic query
       **/
      [key: string]: QueryableStorageEntry<ApiType>;
    };
    randomnessCollectiveFlip: {
      /**
       * Series of block headers from the last 81 blocks that acts as random seed material. This
       * is arranged as a ring buffer with `block_number % 81` being the index into the `Vec` of
       * the oldest hash.
       **/
      randomMaterial: AugmentedQuery<ApiType, () => Observable<Vec<H256>>, []> & QueryableStorageEntry<ApiType, []>;
      /**
       * Generic query
       **/
      [key: string]: QueryableStorageEntry<ApiType>;
    };
    relayer: {
      /**
       * The subsidy for a `user_key` if they are being subsidised,
       * as a map `user_key` => `Subsidy`.
       * 
       * A key can only have one subsidy at a time.  To change subsidisers
       * a key needs to call `remove_paying_key` to remove the current subsidy,
       * before they can accept a new subsidiser.
       **/
      subsidies: AugmentedQuery<ApiType, (arg: AccountId32 | string | Uint8Array) => Observable<Option<PalletRelayerSubsidy>>, [AccountId32]> & QueryableStorageEntry<ApiType, [AccountId32]>;
      /**
       * Generic query
       **/
      [key: string]: QueryableStorageEntry<ApiType>;
    };
    rewards: {
      /**
       * Map of (Itn Address `AccountId`) -> (Reward `ItnRewardStatus`).
       **/
      itnRewards: AugmentedQuery<ApiType, (arg: AccountId32 | string | Uint8Array) => Observable<Option<PalletRewardsItnRewardStatus>>, [AccountId32]> & QueryableStorageEntry<ApiType, [AccountId32]>;
      /**
       * Generic query
       **/
      [key: string]: QueryableStorageEntry<ApiType>;
    };
    scheduler: {
      /**
       * Items to be executed, indexed by the block number that they should be executed on.
       **/
      agenda: AugmentedQuery<ApiType, (arg: u32 | AnyNumber | Uint8Array) => Observable<Vec<Option<PalletSchedulerScheduledV3>>>, [u32]> & QueryableStorageEntry<ApiType, [u32]>;
      /**
       * Lookup from identity to the block number and index of the task.
       **/
      lookup: AugmentedQuery<ApiType, (arg: Bytes | string | Uint8Array) => Observable<Option<ITuple<[u32, u32]>>>, [Bytes]> & QueryableStorageEntry<ApiType, [Bytes]>;
      /**
       * Generic query
       **/
      [key: string]: QueryableStorageEntry<ApiType>;
    };
    session: {
      /**
       * Current index of the session.
       **/
      currentIndex: AugmentedQuery<ApiType, () => Observable<u32>, []> & QueryableStorageEntry<ApiType, []>;
      /**
       * Indices of disabled validators.
       * 
       * The vec is always kept sorted so that we can find whether a given validator is
       * disabled using binary search. It gets cleared when `on_session_ending` returns
       * a new set of identities.
       **/
      disabledValidators: AugmentedQuery<ApiType, () => Observable<Vec<u32>>, []> & QueryableStorageEntry<ApiType, []>;
      /**
       * The owner of a key. The key is the `KeyTypeId` + the encoded key.
       **/
      keyOwner: AugmentedQuery<ApiType, (arg: ITuple<[SpCoreCryptoKeyTypeId, Bytes]> | [SpCoreCryptoKeyTypeId | string | Uint8Array, Bytes | string | Uint8Array]) => Observable<Option<AccountId32>>, [ITuple<[SpCoreCryptoKeyTypeId, Bytes]>]> & QueryableStorageEntry<ApiType, [ITuple<[SpCoreCryptoKeyTypeId, Bytes]>]>;
      /**
       * The next session keys for a validator.
       **/
      nextKeys: AugmentedQuery<ApiType, (arg: AccountId32 | string | Uint8Array) => Observable<Option<PolymeshRuntimeDevelopRuntimeSessionKeys>>, [AccountId32]> & QueryableStorageEntry<ApiType, [AccountId32]>;
      /**
       * True if the underlying economic identities or weighting behind the validators
       * has changed in the queued validator set.
       **/
      queuedChanged: AugmentedQuery<ApiType, () => Observable<bool>, []> & QueryableStorageEntry<ApiType, []>;
      /**
       * The queued keys for the next session. When the next session begins, these keys
       * will be used to determine the validator's session keys.
       **/
      queuedKeys: AugmentedQuery<ApiType, () => Observable<Vec<ITuple<[AccountId32, PolymeshRuntimeDevelopRuntimeSessionKeys]>>>, []> & QueryableStorageEntry<ApiType, []>;
      /**
       * The current set of validators.
       **/
      validators: AugmentedQuery<ApiType, () => Observable<Vec<AccountId32>>, []> & QueryableStorageEntry<ApiType, []>;
      /**
       * Generic query
       **/
      [key: string]: QueryableStorageEntry<ApiType>;
    };
    settlement: {
      /**
       * Tracks affirmations received for an instruction. (instruction_id, counter_party) -> AffirmationStatus
       **/
      affirmsReceived: AugmentedQuery<ApiType, (arg1: u64 | AnyNumber | Uint8Array, arg2: PolymeshPrimitivesIdentityIdPortfolioId | { did?: any; kind?: any } | string | Uint8Array) => Observable<PalletSettlementAffirmationStatus>, [u64, PolymeshPrimitivesIdentityIdPortfolioId]> & QueryableStorageEntry<ApiType, [u64, PolymeshPrimitivesIdentityIdPortfolioId]>;
      /**
       * Free-form text about a venue. venue_id -> `VenueDetails`
       * Only needed for the UI.
       **/
      details: AugmentedQuery<ApiType, (arg: u64 | AnyNumber | Uint8Array) => Observable<Bytes>, [u64]> & QueryableStorageEntry<ApiType, [u64]>;
      /**
       * Number of affirmations pending before instruction is executed. instruction_id -> affirm_pending
       **/
      instructionAffirmsPending: AugmentedQuery<ApiType, (arg: u64 | AnyNumber | Uint8Array) => Observable<u64>, [u64]> & QueryableStorageEntry<ApiType, [u64]>;
      /**
       * Number of instructions in the system (It's one more than the actual number)
       **/
      instructionCounter: AugmentedQuery<ApiType, () => Observable<u64>, []> & QueryableStorageEntry<ApiType, []>;
      /**
       * Details about an instruction. instruction_id -> instruction_details
       **/
      instructionDetails: AugmentedQuery<ApiType, (arg: u64 | AnyNumber | Uint8Array) => Observable<PalletSettlementInstruction>, [u64]> & QueryableStorageEntry<ApiType, [u64]>;
      /**
       * Legs under an instruction. (instruction_id, leg_id) -> Leg
       **/
      instructionLegs: AugmentedQuery<ApiType, (arg1: u64 | AnyNumber | Uint8Array, arg2: u64 | AnyNumber | Uint8Array) => Observable<PalletSettlementLeg>, [u64, u64]> & QueryableStorageEntry<ApiType, [u64, u64]>;
      /**
       * Status of a leg under an instruction. (instruction_id, leg_id) -> LegStatus
       **/
      instructionLegStatus: AugmentedQuery<ApiType, (arg1: u64 | AnyNumber | Uint8Array, arg2: u64 | AnyNumber | Uint8Array) => Observable<PalletSettlementLegStatus>, [u64, u64]> & QueryableStorageEntry<ApiType, [u64, u64]>;
      /**
       * Tracks redemption of receipts. (signer, receipt_uid) -> receipt_used
       **/
      receiptsUsed: AugmentedQuery<ApiType, (arg1: AccountId32 | string | Uint8Array, arg2: u64 | AnyNumber | Uint8Array) => Observable<bool>, [AccountId32, u64]> & QueryableStorageEntry<ApiType, [AccountId32, u64]>;
      /**
       * Storage version.
       **/
      storageVersion: AugmentedQuery<ApiType, () => Observable<u8>, []> & QueryableStorageEntry<ApiType, []>;
      /**
       * Helps a user track their pending instructions and affirmations (only needed for UI).
       * (counter_party, instruction_id) -> AffirmationStatus
       **/
      userAffirmations: AugmentedQuery<ApiType, (arg1: PolymeshPrimitivesIdentityIdPortfolioId | { did?: any; kind?: any } | string | Uint8Array, arg2: u64 | AnyNumber | Uint8Array) => Observable<PalletSettlementAffirmationStatus>, [PolymeshPrimitivesIdentityIdPortfolioId, u64]> & QueryableStorageEntry<ApiType, [PolymeshPrimitivesIdentityIdPortfolioId, u64]>;
      /**
       * Array of venues created by an identity. Only needed for the UI. IdentityId -> Vec<venue_id>
       **/
      userVenues: AugmentedQuery<ApiType, (arg: PolymeshPrimitivesIdentityId | string | Uint8Array) => Observable<Vec<u64>>, [PolymeshPrimitivesIdentityId]> & QueryableStorageEntry<ApiType, [PolymeshPrimitivesIdentityId]>;
      /**
       * Venues that are allowed to create instructions involving a particular ticker. Only used if filtering is enabled.
       * (ticker, venue_id) -> allowed
       **/
      venueAllowList: AugmentedQuery<ApiType, (arg1: PolymeshPrimitivesTicker | string | Uint8Array, arg2: u64 | AnyNumber | Uint8Array) => Observable<bool>, [PolymeshPrimitivesTicker, u64]> & QueryableStorageEntry<ApiType, [PolymeshPrimitivesTicker, u64]>;
      /**
       * Number of venues in the system (It's one more than the actual number)
       **/
      venueCounter: AugmentedQuery<ApiType, () => Observable<u64>, []> & QueryableStorageEntry<ApiType, []>;
      /**
       * Tracks if a token has enabled filtering venues that can create instructions involving their token. Ticker -> filtering_enabled
       **/
      venueFiltering: AugmentedQuery<ApiType, (arg: PolymeshPrimitivesTicker | string | Uint8Array) => Observable<bool>, [PolymeshPrimitivesTicker]> & QueryableStorageEntry<ApiType, [PolymeshPrimitivesTicker]>;
      /**
       * Info about a venue. venue_id -> venue
       **/
      venueInfo: AugmentedQuery<ApiType, (arg: u64 | AnyNumber | Uint8Array) => Observable<Option<PalletSettlementVenue>>, [u64]> & QueryableStorageEntry<ApiType, [u64]>;
      /**
       * Instructions under a venue.
       * Only needed for the UI.
       * 
       * venue_id -> instruction_id -> ()
       **/
      venueInstructions: AugmentedQuery<ApiType, (arg1: u64 | AnyNumber | Uint8Array, arg2: u64 | AnyNumber | Uint8Array) => Observable<Null>, [u64, u64]> & QueryableStorageEntry<ApiType, [u64, u64]>;
      /**
       * Signers allowed by the venue. (venue_id, signer) -> bool
       **/
      venueSigners: AugmentedQuery<ApiType, (arg1: u64 | AnyNumber | Uint8Array, arg2: AccountId32 | string | Uint8Array) => Observable<bool>, [u64, AccountId32]> & QueryableStorageEntry<ApiType, [u64, AccountId32]>;
      /**
       * Generic query
       **/
      [key: string]: QueryableStorageEntry<ApiType>;
    };
    staking: {
      /**
       * The active era information, it holds index and start.
       * 
       * The active era is the era being currently rewarded. Validator set of this era must be
       * equal to [`SessionInterface::validators`].
       **/
      activeEra: AugmentedQuery<ApiType, () => Observable<Option<PalletStakingActiveEraInfo>>, []> & QueryableStorageEntry<ApiType, []>;
      /**
       * Map from all locked "stash" accounts to the controller account.
       **/
      bonded: AugmentedQuery<ApiType, (arg: AccountId32 | string | Uint8Array) => Observable<Option<AccountId32>>, [AccountId32]> & QueryableStorageEntry<ApiType, [AccountId32]>;
      /**
       * A mapping from still-bonded eras to the first session index of that era.
       * 
       * Must contains information for eras for the range:
       * `[active_era - bounding_duration; active_era]`
       **/
      bondedEras: AugmentedQuery<ApiType, () => Observable<Vec<ITuple<[u32, u32]>>>, []> & QueryableStorageEntry<ApiType, []>;
      /**
       * The amount of currency given to reporters of a slash event which was
       * canceled by extraordinary circumstances (e.g. governance).
       **/
      canceledSlashPayout: AugmentedQuery<ApiType, () => Observable<u128>, []> & QueryableStorageEntry<ApiType, []>;
      /**
       * The current era index.
       * 
       * This is the latest planned era, depending on how the Session pallet queues the validator
       * set, it might be active or not.
       **/
      currentEra: AugmentedQuery<ApiType, () => Observable<Option<u32>>, []> & QueryableStorageEntry<ApiType, []>;
      /**
       * The earliest era for which we have a pending, unapplied slash.
       **/
      earliestUnappliedSlash: AugmentedQuery<ApiType, () => Observable<Option<u32>>, []> & QueryableStorageEntry<ApiType, []>;
      /**
       * Flag to control the execution of the offchain election. When `Open(_)`, we accept
       * solutions to be submitted.
       **/
      eraElectionStatus: AugmentedQuery<ApiType, () => Observable<PalletStakingElectionStatus>, []> & QueryableStorageEntry<ApiType, []>;
      /**
       * Rewards for the last `HISTORY_DEPTH` eras.
       * If reward hasn't been set or has been removed then 0 reward is returned.
       **/
      erasRewardPoints: AugmentedQuery<ApiType, (arg: u32 | AnyNumber | Uint8Array) => Observable<PalletStakingEraRewardPoints>, [u32]> & QueryableStorageEntry<ApiType, [u32]>;
      /**
       * Exposure of validator at era.
       * 
       * This is keyed first by the era index to allow bulk deletion and then the stash account.
       * 
       * Is it removed after `HISTORY_DEPTH` eras.
       * If stakers hasn't been set or has been removed then empty exposure is returned.
       **/
      erasStakers: AugmentedQuery<ApiType, (arg1: u32 | AnyNumber | Uint8Array, arg2: AccountId32 | string | Uint8Array) => Observable<PalletStakingExposure>, [u32, AccountId32]> & QueryableStorageEntry<ApiType, [u32, AccountId32]>;
      /**
       * Clipped Exposure of validator at era.
       * 
       * This is similar to [`ErasStakers`] but number of nominators exposed is reduced to the
       * `T::MaxNominatorRewardedPerValidator` biggest stakers.
       * (Note: the field `total` and `own` of the exposure remains unchanged).
       * This is used to limit the i/o cost for the nominator payout.
       * 
       * This is keyed fist by the era index to allow bulk deletion and then the stash account.
       * 
       * Is it removed after `HISTORY_DEPTH` eras.
       * If stakers hasn't been set or has been removed then empty exposure is returned.
       **/
      erasStakersClipped: AugmentedQuery<ApiType, (arg1: u32 | AnyNumber | Uint8Array, arg2: AccountId32 | string | Uint8Array) => Observable<PalletStakingExposure>, [u32, AccountId32]> & QueryableStorageEntry<ApiType, [u32, AccountId32]>;
      /**
       * The session index at which the era start for the last `HISTORY_DEPTH` eras.
       * 
       * Note: This tracks the starting session (i.e. session index when era start being active)
       * for the eras in `[CurrentEra - HISTORY_DEPTH, CurrentEra]`.
       **/
      erasStartSessionIndex: AugmentedQuery<ApiType, (arg: u32 | AnyNumber | Uint8Array) => Observable<Option<u32>>, [u32]> & QueryableStorageEntry<ApiType, [u32]>;
      /**
       * The total amount staked for the last `HISTORY_DEPTH` eras.
       * If total hasn't been set or has been removed then 0 stake is returned.
       **/
      erasTotalStake: AugmentedQuery<ApiType, (arg: u32 | AnyNumber | Uint8Array) => Observable<u128>, [u32]> & QueryableStorageEntry<ApiType, [u32]>;
      /**
       * Similar to `ErasStakers`, this holds the preferences of validators.
       * 
       * This is keyed first by the era index to allow bulk deletion and then the stash account.
       * 
       * Is it removed after `HISTORY_DEPTH` eras.
       **/
      erasValidatorPrefs: AugmentedQuery<ApiType, (arg1: u32 | AnyNumber | Uint8Array, arg2: AccountId32 | string | Uint8Array) => Observable<PalletStakingValidatorPrefs>, [u32, AccountId32]> & QueryableStorageEntry<ApiType, [u32, AccountId32]>;
      /**
       * The total validator era payout for the last `HISTORY_DEPTH` eras.
       * 
       * Eras that haven't finished yet or has been removed doesn't have reward.
       **/
      erasValidatorReward: AugmentedQuery<ApiType, (arg: u32 | AnyNumber | Uint8Array) => Observable<Option<u128>>, [u32]> & QueryableStorageEntry<ApiType, [u32]>;
      /**
       * Mode of era forcing.
       **/
      forceEra: AugmentedQuery<ApiType, () => Observable<PalletStakingForcing>, []> & QueryableStorageEntry<ApiType, []>;
      /**
       * Number of eras to keep in history.
       * 
       * Information is kept for eras in `[current_era - history_depth; current_era]`.
       * 
       * Must be more than the number of eras delayed by session otherwise. I.e. active era must
       * always be in history. I.e. `active_era > current_era - history_depth` must be
       * guaranteed.
       **/
      historyDepth: AugmentedQuery<ApiType, () => Observable<u32>, []> & QueryableStorageEntry<ApiType, []>;
      /**
       * Any validators that may never be slashed or forcibly kicked. It's a Vec since they're
       * easy to initialize and the performance hit is minimal (we expect no more than four
       * invulnerables) and restricted to testnets.
       **/
      invulnerables: AugmentedQuery<ApiType, () => Observable<Vec<AccountId32>>, []> & QueryableStorageEntry<ApiType, []>;
      /**
       * True if the current **planned** session is final. Note that this does not take era
       * forcing into account.
       **/
      isCurrentSessionFinal: AugmentedQuery<ApiType, () => Observable<bool>, []> & QueryableStorageEntry<ApiType, []>;
      /**
       * Map from all (unlocked) "controller" accounts to the info regarding the staking.
       **/
      ledger: AugmentedQuery<ApiType, (arg: AccountId32 | string | Uint8Array) => Observable<Option<PalletStakingStakingLedger>>, [AccountId32]> & QueryableStorageEntry<ApiType, [AccountId32]>;
      /**
       * The minimum amount with which a validator can bond.
       **/
      minimumBondThreshold: AugmentedQuery<ApiType, () => Observable<u128>, []> & QueryableStorageEntry<ApiType, []>;
      /**
       * Minimum number of staking participants before emergency conditions are imposed.
       **/
      minimumValidatorCount: AugmentedQuery<ApiType, () => Observable<u32>, []> & QueryableStorageEntry<ApiType, []>;
      /**
       * The map from nominator stash key to the set of stash keys of all validators to nominate.
       **/
      nominators: AugmentedQuery<ApiType, (arg: AccountId32 | string | Uint8Array) => Observable<Option<PalletStakingNominations>>, [AccountId32]> & QueryableStorageEntry<ApiType, [AccountId32]>;
      /**
       * All slashing events on nominators, mapped by era to the highest slash value of the era.
       **/
      nominatorSlashInEra: AugmentedQuery<ApiType, (arg1: u32 | AnyNumber | Uint8Array, arg2: AccountId32 | string | Uint8Array) => Observable<Option<u128>>, [u32, AccountId32]> & QueryableStorageEntry<ApiType, [u32, AccountId32]>;
      /**
       * Where the reward payment should be made. Keyed by stash.
       **/
      payee: AugmentedQuery<ApiType, (arg: AccountId32 | string | Uint8Array) => Observable<PalletStakingRewardDestination>, [AccountId32]> & QueryableStorageEntry<ApiType, [AccountId32]>;
      /**
       * Entities that are allowed to run operator/validator nodes.
       **/
      permissionedIdentity: AugmentedQuery<ApiType, (arg: PolymeshPrimitivesIdentityId | string | Uint8Array) => Observable<Option<PalletStakingPermissionedIdentityPrefs>>, [PolymeshPrimitivesIdentityId]> & QueryableStorageEntry<ApiType, [PolymeshPrimitivesIdentityId]>;
      /**
       * The next validator set. At the end of an era, if this is available (potentially from the
       * result of an offchain worker), it is immediately used. Otherwise, the on-chain election
       * is executed.
       **/
      queuedElected: AugmentedQuery<ApiType, () => Observable<Option<PalletStakingElectionResult>>, []> & QueryableStorageEntry<ApiType, []>;
      /**
       * The score of the current [`QueuedElected`].
       **/
      queuedScore: AugmentedQuery<ApiType, () => Observable<Option<SpNposElectionsElectionScore>>, []> & QueryableStorageEntry<ApiType, []>;
      slashingAllowedFor: AugmentedQuery<ApiType, () => Observable<PalletStakingSlashingSwitch>, []> & QueryableStorageEntry<ApiType, []>;
      /**
       * Slashing spans for stash accounts.
       **/
      slashingSpans: AugmentedQuery<ApiType, (arg: AccountId32 | string | Uint8Array) => Observable<Option<PalletStakingSlashingSlashingSpans>>, [AccountId32]> & QueryableStorageEntry<ApiType, [AccountId32]>;
      /**
       * The percentage of the slash that is distributed to reporters.
       * 
       * The rest of the slashed value is handled by the `Slash`.
       **/
      slashRewardFraction: AugmentedQuery<ApiType, () => Observable<Perbill>, []> & QueryableStorageEntry<ApiType, []>;
      /**
       * Snapshot of nominators at the beginning of the current election window. This should only
       * have a value when [`EraElectionStatus`] == `ElectionStatus::Open(_)`.
       **/
      snapshotNominators: AugmentedQuery<ApiType, () => Observable<Option<Vec<AccountId32>>>, []> & QueryableStorageEntry<ApiType, []>;
      /**
       * Snapshot of validators at the beginning of the current election window. This should only
       * have a value when [`EraElectionStatus`] == `ElectionStatus::Open(_)`.
       **/
      snapshotValidators: AugmentedQuery<ApiType, () => Observable<Option<Vec<AccountId32>>>, []> & QueryableStorageEntry<ApiType, []>;
      /**
       * Records information about the maximum slash of a stash within a slashing span,
       * as well as how much reward has been paid out.
       **/
      spanSlash: AugmentedQuery<ApiType, (arg: ITuple<[AccountId32, u32]> | [AccountId32 | string | Uint8Array, u32 | AnyNumber | Uint8Array]) => Observable<PalletStakingSlashingSpanRecord>, [ITuple<[AccountId32, u32]>]> & QueryableStorageEntry<ApiType, [ITuple<[AccountId32, u32]>]>;
      /**
       * True if network has been upgraded to this version.
       * Storage version of the pallet.
       * 
       * This is set to v6.0.1 for new networks.
       **/
      storageVersion: AugmentedQuery<ApiType, () => Observable<PalletStakingReleases>, []> & QueryableStorageEntry<ApiType, []>;
      /**
       * All unapplied slashes that are queued for later.
       **/
      unappliedSlashes: AugmentedQuery<ApiType, (arg: u32 | AnyNumber | Uint8Array) => Observable<Vec<PalletStakingUnappliedSlash>>, [u32]> & QueryableStorageEntry<ApiType, [u32]>;
      /**
       * Every validator has commission that should be in the range [0, Cap].
       **/
      validatorCommissionCap: AugmentedQuery<ApiType, () => Observable<Perbill>, []> & QueryableStorageEntry<ApiType, []>;
      /**
       * The ideal number of staking participants.
       **/
      validatorCount: AugmentedQuery<ApiType, () => Observable<u32>, []> & QueryableStorageEntry<ApiType, []>;
      /**
       * The map from (wannabe) validator stash key to the preferences of that validator.
       **/
      validators: AugmentedQuery<ApiType, (arg: AccountId32 | string | Uint8Array) => Observable<PalletStakingValidatorPrefs>, [AccountId32]> & QueryableStorageEntry<ApiType, [AccountId32]>;
      /**
       * All slashing events on validators, mapped by era to the highest slash proportion
       * and slash value of the era.
       **/
      validatorSlashInEra: AugmentedQuery<ApiType, (arg1: u32 | AnyNumber | Uint8Array, arg2: AccountId32 | string | Uint8Array) => Observable<Option<ITuple<[Perbill, u128]>>>, [u32, AccountId32]> & QueryableStorageEntry<ApiType, [u32, AccountId32]>;
      /**
       * Generic query
       **/
      [key: string]: QueryableStorageEntry<ApiType>;
    };
    statistics: {
      /**
       * Active stats for a ticker/company.  There should be a max limit on the number of active stats for a ticker/company.
       **/
      activeAssetStats: AugmentedQuery<ApiType, (arg: PolymeshPrimitivesStatisticsAssetScope | { Ticker: any } | string | Uint8Array) => Observable<Vec<PolymeshPrimitivesStatisticsStatType>>, [PolymeshPrimitivesStatisticsAssetScope]> & QueryableStorageEntry<ApiType, [PolymeshPrimitivesStatisticsAssetScope]>;
      /**
       * Asset stats.
       **/
      assetStats: AugmentedQuery<ApiType, (arg1: PolymeshPrimitivesStatisticsStat1stKey | { asset?: any; statType?: any } | string | Uint8Array, arg2: PolymeshPrimitivesStatisticsStat2ndKey | { NoClaimStat: any } | { Claim: any } | string | Uint8Array) => Observable<u128>, [PolymeshPrimitivesStatisticsStat1stKey, PolymeshPrimitivesStatisticsStat2ndKey]> & QueryableStorageEntry<ApiType, [PolymeshPrimitivesStatisticsStat1stKey, PolymeshPrimitivesStatisticsStat2ndKey]>;
      /**
       * Asset transfer compliance for a ticker (AssetScope -> AssetTransferCompliance)
       **/
      assetTransferCompliances: AugmentedQuery<ApiType, (arg: PolymeshPrimitivesStatisticsAssetScope | { Ticker: any } | string | Uint8Array) => Observable<PolymeshPrimitivesTransferComplianceAssetTransferCompliance>, [PolymeshPrimitivesStatisticsAssetScope]> & QueryableStorageEntry<ApiType, [PolymeshPrimitivesStatisticsAssetScope]>;
      /**
       * Storage migration version.
       **/
      storageVersion: AugmentedQuery<ApiType, () => Observable<u8>, []> & QueryableStorageEntry<ApiType, []>;
      /**
       * Entities exempt from a Transfer Compliance rule.
       **/
      transferConditionExemptEntities: AugmentedQuery<ApiType, (arg1: PolymeshPrimitivesTransferComplianceTransferConditionExemptKey | { asset?: any; op?: any; claimType?: any } | string | Uint8Array, arg2: PolymeshPrimitivesIdentityId | string | Uint8Array) => Observable<bool>, [PolymeshPrimitivesTransferComplianceTransferConditionExemptKey, PolymeshPrimitivesIdentityId]> & QueryableStorageEntry<ApiType, [PolymeshPrimitivesTransferComplianceTransferConditionExemptKey, PolymeshPrimitivesIdentityId]>;
      /**
       * Generic query
       **/
      [key: string]: QueryableStorageEntry<ApiType>;
    };
    sto: {
      /**
       * Total fundraisers created for a token.
       **/
      fundraiserCount: AugmentedQuery<ApiType, (arg: PolymeshPrimitivesTicker | string | Uint8Array) => Observable<u64>, [PolymeshPrimitivesTicker]> & QueryableStorageEntry<ApiType, [PolymeshPrimitivesTicker]>;
      /**
       * Name for the Fundraiser. Only used offchain.
       * (ticker, fundraiser_id) -> Fundraiser name
       **/
      fundraiserNames: AugmentedQuery<ApiType, (arg1: PolymeshPrimitivesTicker | string | Uint8Array, arg2: u64 | AnyNumber | Uint8Array) => Observable<Bytes>, [PolymeshPrimitivesTicker, u64]> & QueryableStorageEntry<ApiType, [PolymeshPrimitivesTicker, u64]>;
      /**
       * All fundraisers that are currently running.
       * (ticker, fundraiser_id) -> Fundraiser
       **/
      fundraisers: AugmentedQuery<ApiType, (arg1: PolymeshPrimitivesTicker | string | Uint8Array, arg2: u64 | AnyNumber | Uint8Array) => Observable<Option<PalletStoFundraiser>>, [PolymeshPrimitivesTicker, u64]> & QueryableStorageEntry<ApiType, [PolymeshPrimitivesTicker, u64]>;
      /**
       * Generic query
       **/
      [key: string]: QueryableStorageEntry<ApiType>;
    };
    sudo: {
      /**
       * The `AccountId` of the sudo key.
       **/
      key: AugmentedQuery<ApiType, () => Observable<AccountId32>, []> & QueryableStorageEntry<ApiType, []>;
      /**
       * Generic query
       **/
      [key: string]: QueryableStorageEntry<ApiType>;
    };
    system: {
      /**
       * The full account information for a particular account ID.
       **/
      account: AugmentedQuery<ApiType, (arg: AccountId32 | string | Uint8Array) => Observable<FrameSystemAccountInfo>, [AccountId32]> & QueryableStorageEntry<ApiType, [AccountId32]>;
      /**
       * Total length (in bytes) for all extrinsics put together, for the current block.
       **/
      allExtrinsicsLen: AugmentedQuery<ApiType, () => Observable<Option<u32>>, []> & QueryableStorageEntry<ApiType, []>;
      /**
       * Map of block numbers to block hashes.
       **/
      blockHash: AugmentedQuery<ApiType, (arg: u32 | AnyNumber | Uint8Array) => Observable<H256>, [u32]> & QueryableStorageEntry<ApiType, [u32]>;
      /**
       * The current weight for the block.
       **/
      blockWeight: AugmentedQuery<ApiType, () => Observable<FrameSupportWeightsPerDispatchClassU64>, []> & QueryableStorageEntry<ApiType, []>;
      /**
       * Digest of the current block, also part of the block header.
       **/
      digest: AugmentedQuery<ApiType, () => Observable<SpRuntimeDigest>, []> & QueryableStorageEntry<ApiType, []>;
      /**
       * The number of events in the `Events<T>` list.
       **/
      eventCount: AugmentedQuery<ApiType, () => Observable<u32>, []> & QueryableStorageEntry<ApiType, []>;
      /**
       * Events deposited for the current block.
       * 
       * NOTE: The item is unbound and should therefore never be read on chain.
       * It could otherwise inflate the PoV size of a block.
       * 
       * Events have a large in-memory size. Box the events to not go out-of-memory
       * just in case someone still reads them from within the runtime.
       **/
      events: AugmentedQuery<ApiType, () => Observable<Vec<FrameSystemEventRecord>>, []> & QueryableStorageEntry<ApiType, []>;
      /**
       * Mapping between a topic (represented by T::Hash) and a vector of indexes
       * of events in the `<Events<T>>` list.
       * 
       * All topic vectors have deterministic storage locations depending on the topic. This
       * allows light-clients to leverage the changes trie storage tracking mechanism and
       * in case of changes fetch the list of events of interest.
       * 
       * The value has the type `(T::BlockNumber, EventIndex)` because if we used only just
       * the `EventIndex` then in case if the topic has the same contents on the next block
       * no notification will be triggered thus the event might be lost.
       **/
      eventTopics: AugmentedQuery<ApiType, (arg: H256 | string | Uint8Array) => Observable<Vec<ITuple<[u32, u32]>>>, [H256]> & QueryableStorageEntry<ApiType, [H256]>;
      /**
       * The execution phase of the block.
       **/
      executionPhase: AugmentedQuery<ApiType, () => Observable<Option<FrameSystemPhase>>, []> & QueryableStorageEntry<ApiType, []>;
      /**
       * Total extrinsics count for the current block.
       **/
      extrinsicCount: AugmentedQuery<ApiType, () => Observable<Option<u32>>, []> & QueryableStorageEntry<ApiType, []>;
      /**
       * Extrinsics data for the current block (maps an extrinsic's index to its data).
       **/
      extrinsicData: AugmentedQuery<ApiType, (arg: u32 | AnyNumber | Uint8Array) => Observable<Bytes>, [u32]> & QueryableStorageEntry<ApiType, [u32]>;
      /**
       * Stores the `spec_version` and `spec_name` of when the last runtime upgrade happened.
       **/
      lastRuntimeUpgrade: AugmentedQuery<ApiType, () => Observable<Option<FrameSystemLastRuntimeUpgradeInfo>>, []> & QueryableStorageEntry<ApiType, []>;
      /**
       * The current block number being processed. Set by `execute_block`.
       **/
      number: AugmentedQuery<ApiType, () => Observable<u32>, []> & QueryableStorageEntry<ApiType, []>;
      /**
       * Hash of the previous block.
       **/
      parentHash: AugmentedQuery<ApiType, () => Observable<H256>, []> & QueryableStorageEntry<ApiType, []>;
      /**
       * True if we have upgraded so that AccountInfo contains three types of `RefCount`. False
       * (default) if not.
       **/
      upgradedToTripleRefCount: AugmentedQuery<ApiType, () => Observable<bool>, []> & QueryableStorageEntry<ApiType, []>;
      /**
       * True if we have upgraded so that `type RefCount` is `u32`. False (default) if not.
       **/
      upgradedToU32RefCount: AugmentedQuery<ApiType, () => Observable<bool>, []> & QueryableStorageEntry<ApiType, []>;
      /**
       * Generic query
       **/
      [key: string]: QueryableStorageEntry<ApiType>;
    };
    technicalCommittee: {
      /**
       * Time after which a proposal will expire.
       **/
      expiresAfter: AugmentedQuery<ApiType, () => Observable<PolymeshCommonUtilitiesMaybeBlock>, []> & QueryableStorageEntry<ApiType, []>;
      /**
       * The current members of the committee.
       **/
      members: AugmentedQuery<ApiType, () => Observable<Vec<PolymeshPrimitivesIdentityId>>, []> & QueryableStorageEntry<ApiType, []>;
      /**
       * Proposals so far.
       **/
      proposalCount: AugmentedQuery<ApiType, () => Observable<u32>, []> & QueryableStorageEntry<ApiType, []>;
      /**
       * Actual proposal for a given hash.
       **/
      proposalOf: AugmentedQuery<ApiType, (arg: H256 | string | Uint8Array) => Observable<Option<Call>>, [H256]> & QueryableStorageEntry<ApiType, [H256]>;
      /**
       * The hashes of the active proposals.
       **/
      proposals: AugmentedQuery<ApiType, () => Observable<Vec<H256>>, []> & QueryableStorageEntry<ApiType, []>;
      /**
       * Release coordinator.
       **/
      releaseCoordinator: AugmentedQuery<ApiType, () => Observable<Option<U8aFixed>>, []> & QueryableStorageEntry<ApiType, []>;
      /**
       * Storage version.
       **/
      storageVersion: AugmentedQuery<ApiType, () => Observable<u8>, []> & QueryableStorageEntry<ApiType, []>;
      /**
       * Vote threshold for an approval.
       **/
      voteThreshold: AugmentedQuery<ApiType, () => Observable<ITuple<[u32, u32]>>, []> & QueryableStorageEntry<ApiType, []>;
      /**
       * PolymeshVotes on a given proposal, if it is ongoing.
       **/
      voting: AugmentedQuery<ApiType, (arg: H256 | string | Uint8Array) => Observable<Option<PalletCommitteePolymeshVotes>>, [H256]> & QueryableStorageEntry<ApiType, [H256]>;
      /**
       * Generic query
       **/
      [key: string]: QueryableStorageEntry<ApiType>;
    };
    technicalCommitteeMembership: {
      /**
       * The current "active" membership, stored as an ordered Vec.
       **/
      activeMembers: AugmentedQuery<ApiType, () => Observable<Vec<PolymeshPrimitivesIdentityId>>, []> & QueryableStorageEntry<ApiType, []>;
      /**
       * Limit of how many "active" members there can be.
       **/
      activeMembersLimit: AugmentedQuery<ApiType, () => Observable<u32>, []> & QueryableStorageEntry<ApiType, []>;
      /**
       * The current "inactive" membership, stored as an ordered Vec.
       **/
      inactiveMembers: AugmentedQuery<ApiType, () => Observable<Vec<PolymeshCommonUtilitiesGroupInactiveMember>>, []> & QueryableStorageEntry<ApiType, []>;
      /**
       * Generic query
       **/
      [key: string]: QueryableStorageEntry<ApiType>;
    };
    testUtils: {
      /**
       * Generic query
       **/
      [key: string]: QueryableStorageEntry<ApiType>;
    };
    timestamp: {
      /**
       * Did the timestamp get updated in this block?
       **/
      didUpdate: AugmentedQuery<ApiType, () => Observable<bool>, []> & QueryableStorageEntry<ApiType, []>;
      /**
       * Current time for the current block.
       **/
      now: AugmentedQuery<ApiType, () => Observable<u64>, []> & QueryableStorageEntry<ApiType, []>;
      /**
       * Generic query
       **/
      [key: string]: QueryableStorageEntry<ApiType>;
    };
    transactionPayment: {
      nextFeeMultiplier: AugmentedQuery<ApiType, () => Observable<u128>, []> & QueryableStorageEntry<ApiType, []>;
      storageVersion: AugmentedQuery<ApiType, () => Observable<PalletTransactionPaymentReleases>, []> & QueryableStorageEntry<ApiType, []>;
      /**
       * Generic query
       **/
      [key: string]: QueryableStorageEntry<ApiType>;
    };
    upgradeCommittee: {
      /**
       * Time after which a proposal will expire.
       **/
      expiresAfter: AugmentedQuery<ApiType, () => Observable<PolymeshCommonUtilitiesMaybeBlock>, []> & QueryableStorageEntry<ApiType, []>;
      /**
       * The current members of the committee.
       **/
      members: AugmentedQuery<ApiType, () => Observable<Vec<PolymeshPrimitivesIdentityId>>, []> & QueryableStorageEntry<ApiType, []>;
      /**
       * Proposals so far.
       **/
      proposalCount: AugmentedQuery<ApiType, () => Observable<u32>, []> & QueryableStorageEntry<ApiType, []>;
      /**
       * Actual proposal for a given hash.
       **/
      proposalOf: AugmentedQuery<ApiType, (arg: H256 | string | Uint8Array) => Observable<Option<Call>>, [H256]> & QueryableStorageEntry<ApiType, [H256]>;
      /**
       * The hashes of the active proposals.
       **/
      proposals: AugmentedQuery<ApiType, () => Observable<Vec<H256>>, []> & QueryableStorageEntry<ApiType, []>;
      /**
       * Release coordinator.
       **/
      releaseCoordinator: AugmentedQuery<ApiType, () => Observable<Option<U8aFixed>>, []> & QueryableStorageEntry<ApiType, []>;
      /**
       * Storage version.
       **/
      storageVersion: AugmentedQuery<ApiType, () => Observable<u8>, []> & QueryableStorageEntry<ApiType, []>;
      /**
       * Vote threshold for an approval.
       **/
      voteThreshold: AugmentedQuery<ApiType, () => Observable<ITuple<[u32, u32]>>, []> & QueryableStorageEntry<ApiType, []>;
      /**
       * PolymeshVotes on a given proposal, if it is ongoing.
       **/
      voting: AugmentedQuery<ApiType, (arg: H256 | string | Uint8Array) => Observable<Option<PalletCommitteePolymeshVotes>>, [H256]> & QueryableStorageEntry<ApiType, [H256]>;
      /**
       * Generic query
       **/
      [key: string]: QueryableStorageEntry<ApiType>;
    };
    upgradeCommitteeMembership: {
      /**
       * The current "active" membership, stored as an ordered Vec.
       **/
      activeMembers: AugmentedQuery<ApiType, () => Observable<Vec<PolymeshPrimitivesIdentityId>>, []> & QueryableStorageEntry<ApiType, []>;
      /**
       * Limit of how many "active" members there can be.
       **/
      activeMembersLimit: AugmentedQuery<ApiType, () => Observable<u32>, []> & QueryableStorageEntry<ApiType, []>;
      /**
       * The current "inactive" membership, stored as an ordered Vec.
       **/
      inactiveMembers: AugmentedQuery<ApiType, () => Observable<Vec<PolymeshCommonUtilitiesGroupInactiveMember>>, []> & QueryableStorageEntry<ApiType, []>;
      /**
       * Generic query
       **/
      [key: string]: QueryableStorageEntry<ApiType>;
    };
    utility: {
      nonces: AugmentedQuery<ApiType, (arg: AccountId32 | string | Uint8Array) => Observable<u64>, [AccountId32]> & QueryableStorageEntry<ApiType, [AccountId32]>;
      /**
       * Generic query
       **/
      [key: string]: QueryableStorageEntry<ApiType>;
    };
  } // AugmentedQueries
} // declare module
