// Auto-generated via `yarn polkadot-types-from-chain`, do not edit
/* eslint-disable */

// import type lookup before we augment - in some environments
// this is required to allow for ambient/previous definitions
import '@polkadot/api-base/types/events';

import type { ApiTypes, AugmentedEvent } from '@polkadot/api-base/types';
import type { BTreeSet, Bytes, Null, Option, Result, U8aFixed, Vec, bool, u128, u32, u64, u8 } from '@polkadot/types-codec';
import type { ITuple } from '@polkadot/types-codec/types';
import type { AccountId32, H256, Perbill, Permill } from '@polkadot/types/interfaces/runtime';
import type { FrameSupportTokensMiscBalanceStatus, FrameSystemDispatchEventInfo, PalletBalancesUnexpectedKind, PalletContractsOrigin, PalletCorporateActionsBallotBallotMeta, PalletCorporateActionsBallotBallotTimeRange, PalletCorporateActionsBallotBallotVote, PalletCorporateActionsCaId, PalletCorporateActionsCorporateAction, PalletCorporateActionsDistribution, PalletCorporateActionsTargetIdentities, PalletElectionProviderMultiPhaseElectionCompute, PalletElectionProviderMultiPhasePhase, PalletImOnlineSr25519AppSr25519Public, PalletPipsProposalData, PalletPipsProposalState, PalletPipsProposer, PalletPipsSnapshottedPip, PalletStakingForcing, PalletStakingRewardDestination, PalletStakingValidatorPrefs, PalletStoFundingAsset, PalletStoFundraiser, PalletValidatorsSlashingSwitch, PolymeshContractsApi, PolymeshContractsChainExtensionExtrinsicId, PolymeshContractsChainVersion, PolymeshPrimitivesAgentAgentGroup, PolymeshPrimitivesAssetAssetHolder, PolymeshPrimitivesAssetAssetId, PolymeshPrimitivesAssetAssetType, PolymeshPrimitivesAssetHoldingsUpdateReason, PolymeshPrimitivesAssetIdentifier, PolymeshPrimitivesAssetMetadataAssetMetadataKey, PolymeshPrimitivesAssetMetadataAssetMetadataSpec, PolymeshPrimitivesAssetMetadataAssetMetadataValueDetail, PolymeshPrimitivesAuthorizationAuthorizationData, PolymeshPrimitivesCheckpointScheduleCheckpoints, PolymeshPrimitivesComplianceManagerComplianceRequirement, PolymeshPrimitivesConditionTrustedIssuer, PolymeshPrimitivesDocument, PolymeshPrimitivesEventOnly, PolymeshPrimitivesIdentityClaim, PolymeshPrimitivesIdentityId, PolymeshPrimitivesIdentityIdPortfolioId, PolymeshPrimitivesMaybeBlock, PolymeshPrimitivesMemo, PolymeshPrimitivesNftNfTs, PolymeshPrimitivesPortfolioFundDescription, PolymeshPrimitivesPosRatio, PolymeshPrimitivesSecondaryKey, PolymeshPrimitivesSecondaryKeyExtrinsicPermissions, PolymeshPrimitivesSecondaryKeyPermissions, PolymeshPrimitivesSettlementLeg, PolymeshPrimitivesSettlementReceiptMetadata, PolymeshPrimitivesSettlementSettlementType, PolymeshPrimitivesSettlementVenueType, PolymeshPrimitivesStatisticsStatType, PolymeshPrimitivesStatisticsStatUpdate, PolymeshPrimitivesTicker, PolymeshPrimitivesTransferComplianceTransferCondition, PolymeshPrimitivesTransferComplianceTransferConditionExemptKey, PolymeshRuntimeDevelopRuntimeRuntimeHoldReason, SpConsensusGrandpaAppPublic, SpNposElectionsElectionScore, SpRuntimeDispatchError, SpStakingExposure } from '@polkadot/types/lookup';

export type __AugmentedEvent<ApiType extends ApiTypes> = AugmentedEvent<ApiType>;

declare module '@polkadot/api-base/types/events' {
  interface AugmentedEvents<ApiType extends ApiTypes> {
    asset: {
      /**
       * An asset has been added to the list of pre aprroved receivement (valid for all identities).
       * Parameters: [`AssetId`] of the pre approved asset.
       **/
      AssetAffirmationExemption: AugmentedEvent<ApiType, [PolymeshPrimitivesAssetAssetId]>;
      /**
       * Emitted when Tokens were issued, redeemed or transferred.
       * Contains the [`IdentityId`] of the receiver/issuer/redeemer, the [`AssetId`] for the token, the balance that was issued/transferred/redeemed,
       * the [`AssetHolder`] of the source, the [`AssetHolder`] of the destination and the [`HoldingsUpdateReason`].
       **/
      AssetBalanceUpdated: AugmentedEvent<ApiType, [PolymeshPrimitivesIdentityId, PolymeshPrimitivesAssetAssetId, u128, Option<PolymeshPrimitivesAssetAssetHolder>, Option<PolymeshPrimitivesAssetAssetHolder>, PolymeshPrimitivesAssetHoldingsUpdateReason]>;
      /**
       * Event for creation of the asset.
       * caller DID/ owner DID, AssetId, divisibility, asset type, beneficiary DID, asset name, identifiers, funding round
       **/
      AssetCreated: AugmentedEvent<ApiType, [PolymeshPrimitivesIdentityId, PolymeshPrimitivesAssetAssetId, bool, PolymeshPrimitivesAssetAssetType, PolymeshPrimitivesIdentityId, Bytes, Vec<PolymeshPrimitivesAssetIdentifier>, Option<Bytes>]>;
      /**
       * An event emitted when an asset is frozen.
       * Parameter: caller DID, AssetId.
       **/
      AssetFrozen: AugmentedEvent<ApiType, [PolymeshPrimitivesIdentityId, PolymeshPrimitivesAssetAssetId]>;
      /**
       * An identity has added mandatory mediators to an asset.
       * Parameters: [`IdentityId`] of caller, [`AssetId`] of the asset, the identity of all mediators added.
       **/
      AssetMediatorsAdded: AugmentedEvent<ApiType, [PolymeshPrimitivesIdentityId, PolymeshPrimitivesAssetAssetId, BTreeSet<PolymeshPrimitivesIdentityId>]>;
      /**
       * An identity has removed mediators from an asset.
       * Parameters: [`IdentityId`] of caller, [`AssetId`] of the asset, the identity of all mediators removed.
       **/
      AssetMediatorsRemoved: AugmentedEvent<ApiType, [PolymeshPrimitivesIdentityId, PolymeshPrimitivesAssetAssetId, BTreeSet<PolymeshPrimitivesIdentityId>]>;
      /**
       * Emit when token ownership is transferred.
       * caller DID / token ownership transferred to DID, AssetId, from
       **/
      AssetOwnershipTransferred: AugmentedEvent<ApiType, [PolymeshPrimitivesIdentityId, PolymeshPrimitivesAssetAssetId, PolymeshPrimitivesIdentityId]>;
      /**
       * An event emitted when a token is renamed.
       * Parameters: caller DID, AssetId, new token name.
       **/
      AssetRenamed: AugmentedEvent<ApiType, [PolymeshPrimitivesIdentityId, PolymeshPrimitivesAssetAssetId, Bytes]>;
      /**
       * An event emitted when the type of an asset changed.
       * Parameters: caller DID, AssetId, new token type.
       **/
      AssetTypeChanged: AugmentedEvent<ApiType, [PolymeshPrimitivesIdentityId, PolymeshPrimitivesAssetAssetId, PolymeshPrimitivesAssetAssetType]>;
      /**
       * An event emitted when an asset is unfrozen.
       * Parameter: caller DID, AssetId.
       **/
      AssetUnfrozen: AugmentedEvent<ApiType, [PolymeshPrimitivesIdentityId, PolymeshPrimitivesAssetAssetId]>;
      /**
       * Event for when a forced transfer takes place.
       * caller DID/ controller DID, ExtensionRemoved, Portfolio of token holder, value.
       **/
      ControllerTransfer: AugmentedEvent<ApiType, [PolymeshPrimitivesIdentityId, PolymeshPrimitivesAssetAssetId, PolymeshPrimitivesAssetAssetHolder, u128]>;
      /**
       * An asset transfer has been created.
       **/
      CreatedAssetTransfer: AugmentedEvent<ApiType, [assetId: PolymeshPrimitivesAssetAssetId, from: AccountId32, to: AccountId32, amount: u128, memo: Option<PolymeshPrimitivesMemo>, pendingTransferId: Option<u64>], { assetId: PolymeshPrimitivesAssetAssetId, from: AccountId32, to: AccountId32, amount: u128, memo: Option<PolymeshPrimitivesMemo>, pendingTransferId: Option<u64> }>;
      /**
       * A custom asset type already exists on-chain.
       * caller DID, the ID of the custom asset type, the string contents registered.
       **/
      CustomAssetTypeExists: AugmentedEvent<ApiType, [PolymeshPrimitivesIdentityId, u32, Bytes]>;
      /**
       * A custom asset type was registered on-chain.
       * caller DID, the ID of the custom asset type, the string contents registered.
       **/
      CustomAssetTypeRegistered: AugmentedEvent<ApiType, [PolymeshPrimitivesIdentityId, u32, Bytes]>;
      /**
       * Event for change in divisibility.
       * caller DID, AssetId, divisibility
       **/
      DivisibilityChanged: AugmentedEvent<ApiType, [PolymeshPrimitivesIdentityId, PolymeshPrimitivesAssetAssetId, bool]>;
      /**
       * A new document attached to an asset
       **/
      DocumentAdded: AugmentedEvent<ApiType, [PolymeshPrimitivesIdentityId, PolymeshPrimitivesAssetAssetId, u32, PolymeshPrimitivesDocument]>;
      /**
       * A document removed from an asset
       **/
      DocumentRemoved: AugmentedEvent<ApiType, [PolymeshPrimitivesIdentityId, PolymeshPrimitivesAssetAssetId, u32]>;
      /**
       * An event carrying the name of the current funding round of an asset.
       * Parameters: caller DID, AssetId, funding round name.
       **/
      FundingRoundSet: AugmentedEvent<ApiType, [PolymeshPrimitivesIdentityId, PolymeshPrimitivesAssetAssetId, Bytes]>;
      /**
       * Asset Global Metadata Spec has been Updated.
       * Parameters: [`AssetMetadataName`] of the metadata, [`AssetMetadataSpec`] of the metadata.
       **/
      GlobalMetadataSpecUpdated: AugmentedEvent<ApiType, [Bytes, PolymeshPrimitivesAssetMetadataAssetMetadataSpec]>;
      /**
       * Event emitted when any token identifiers are updated.
       * caller DID, AssetId, a vector of (identifier type, identifier value)
       **/
      IdentifiersUpdated: AugmentedEvent<ApiType, [PolymeshPrimitivesIdentityId, PolymeshPrimitivesAssetAssetId, Vec<PolymeshPrimitivesAssetIdentifier>]>;
      /**
       * An event emitted when a local metadata key has been removed.
       * Parameters: caller AssetId, Local type name
       **/
      LocalMetadataKeyDeleted: AugmentedEvent<ApiType, [PolymeshPrimitivesIdentityId, PolymeshPrimitivesAssetAssetId, u64]>;
      /**
       * An identity has set or cleared mandatory receiver affirmation for incoming transfers.
       * Parameters: [`IdentityId`] of caller, whether affirmation is now required.
       **/
      MandatoryReceiverAffirmationSet: AugmentedEvent<ApiType, [PolymeshPrimitivesIdentityId, bool]>;
      /**
       * An event emitted when a local metadata value has been removed.
       * Parameters: caller AssetId, Local type name
       **/
      MetadataValueDeleted: AugmentedEvent<ApiType, [PolymeshPrimitivesIdentityId, PolymeshPrimitivesAssetAssetId, PolymeshPrimitivesAssetMetadataAssetMetadataKey]>;
      /**
       * An identity has added an asset to the list of pre aprroved receivement.
       * Parameters: [`IdentityId`] of caller, [`AssetId`] of the pre approved asset.
       **/
      PreApprovedAsset: AugmentedEvent<ApiType, [PolymeshPrimitivesIdentityId, PolymeshPrimitivesAssetAssetId]>;
      /**
       * Register asset metadata global type.
       * (Global type name, Global type key, type specs)
       **/
      RegisterAssetMetadataGlobalType: AugmentedEvent<ApiType, [Bytes, u64, PolymeshPrimitivesAssetMetadataAssetMetadataSpec]>;
      /**
       * Register asset metadata local type.
       * (Caller DID, AssetId, Local type name, Local type key, type specs)
       **/
      RegisterAssetMetadataLocalType: AugmentedEvent<ApiType, [PolymeshPrimitivesIdentityId, PolymeshPrimitivesAssetAssetId, Bytes, u64, PolymeshPrimitivesAssetMetadataAssetMetadataSpec]>;
      /**
       * An asset has been removed from the list of pre aprroved receivement (valid for all identities).
       * Parameters: [`AssetId`] of the asset.
       **/
      RemoveAssetAffirmationExemption: AugmentedEvent<ApiType, [PolymeshPrimitivesAssetAssetId]>;
      /**
       * An identity has removed an asset to the list of pre aprroved receivement.
       * Parameters: [`IdentityId`] of caller, [`AssetId`] of the asset.
       **/
      RemovePreApprovedAsset: AugmentedEvent<ApiType, [PolymeshPrimitivesIdentityId, PolymeshPrimitivesAssetAssetId]>;
      /**
       * Set asset metadata value.
       * (Caller DID, AssetId, metadata value, optional value details)
       **/
      SetAssetMetadataValue: AugmentedEvent<ApiType, [PolymeshPrimitivesIdentityId, PolymeshPrimitivesAssetAssetId, Bytes, Option<PolymeshPrimitivesAssetMetadataAssetMetadataValueDetail>]>;
      /**
       * Set asset metadata value details (expire, lock status).
       * (Caller DID, AssetId, value details)
       **/
      SetAssetMetadataValueDetails: AugmentedEvent<ApiType, [PolymeshPrimitivesIdentityId, PolymeshPrimitivesAssetAssetId, PolymeshPrimitivesAssetMetadataAssetMetadataValueDetail]>;
      /**
       * An identity has linked a ticker to an asset.
       * Parameters: [`IdentityId`] of caller, [`Ticker`] of the asset, the asset identifier [`AssetId`].
       **/
      TickerLinkedToAsset: AugmentedEvent<ApiType, [PolymeshPrimitivesIdentityId, PolymeshPrimitivesTicker, PolymeshPrimitivesAssetAssetId]>;
      /**
       * Emit when ticker is registered.
       * caller DID / ticker owner did, ticker, ticker owner, expiry
       **/
      TickerRegistered: AugmentedEvent<ApiType, [PolymeshPrimitivesIdentityId, PolymeshPrimitivesTicker, Option<u64>]>;
      /**
       * Emit when ticker is transferred.
       * caller DID / ticker transferred to DID, ticker, from
       **/
      TickerTransferred: AugmentedEvent<ApiType, [PolymeshPrimitivesIdentityId, PolymeshPrimitivesTicker, PolymeshPrimitivesIdentityId]>;
      /**
       * An identity has unlinked a ticker from an asset.
       * Parameters: [`IdentityId`] of caller, unlinked [`Ticker`], the asset identifier [`AssetId`].
       **/
      TickerUnlinkedFromAsset: AugmentedEvent<ApiType, [PolymeshPrimitivesIdentityId, PolymeshPrimitivesTicker, PolymeshPrimitivesAssetAssetId]>;
      /**
       * Generic event
       **/
      [key: string]: AugmentedEvent<ApiType>;
    };
    balances: {
      /**
       * A balance was set by root.
       **/
      BalanceSet: AugmentedEvent<ApiType, [who: AccountId32, free: u128], { who: AccountId32, free: u128 }>;
      /**
       * Some amount was burned from an account.
       **/
      Burned: AugmentedEvent<ApiType, [who: AccountId32, amount: u128], { who: AccountId32, amount: u128 }>;
      /**
       * Some debt has been dropped from the Total Issuance.
       **/
      BurnedDebt: AugmentedEvent<ApiType, [amount: u128], { amount: u128 }>;
      /**
       * Held balance was burned from an account.
       **/
      BurnedHeld: AugmentedEvent<ApiType, [reason: PolymeshRuntimeDevelopRuntimeRuntimeHoldReason, who: AccountId32, amount: u128], { reason: PolymeshRuntimeDevelopRuntimeRuntimeHoldReason, who: AccountId32, amount: u128 }>;
      /**
       * Some amount was deposited (e.g. for transaction fees).
       **/
      Deposit: AugmentedEvent<ApiType, [who: AccountId32, amount: u128], { who: AccountId32, amount: u128 }>;
      /**
       * An account was removed whose balance was non-zero but below ExistentialDeposit,
       * resulting in an outright loss.
       **/
      DustLost: AugmentedEvent<ApiType, [account: AccountId32, amount: u128], { account: AccountId32, amount: u128 }>;
      /**
       * An account was created with some free balance.
       **/
      Endowed: AugmentedEvent<ApiType, [account: AccountId32, freeBalance: u128], { account: AccountId32, freeBalance: u128 }>;
      /**
       * Some balance was frozen.
       **/
      Frozen: AugmentedEvent<ApiType, [who: AccountId32, amount: u128], { who: AccountId32, amount: u128 }>;
      /**
       * Some balance was placed on hold.
       **/
      Held: AugmentedEvent<ApiType, [reason: PolymeshRuntimeDevelopRuntimeRuntimeHoldReason, who: AccountId32, amount: u128], { reason: PolymeshRuntimeDevelopRuntimeRuntimeHoldReason, who: AccountId32, amount: u128 }>;
      /**
       * Total issuance was increased by `amount`, creating a credit to be balanced.
       **/
      Issued: AugmentedEvent<ApiType, [amount: u128], { amount: u128 }>;
      /**
       * Some balance was locked.
       **/
      Locked: AugmentedEvent<ApiType, [who: AccountId32, amount: u128], { who: AccountId32, amount: u128 }>;
      /**
       * Some amount was minted into an account.
       **/
      Minted: AugmentedEvent<ApiType, [who: AccountId32, amount: u128], { who: AccountId32, amount: u128 }>;
      /**
       * Some credit was balanced and added to the TotalIssuance.
       **/
      MintedCredit: AugmentedEvent<ApiType, [amount: u128], { amount: u128 }>;
      /**
       * Some balance was released from hold.
       **/
      Released: AugmentedEvent<ApiType, [reason: PolymeshRuntimeDevelopRuntimeRuntimeHoldReason, who: AccountId32, amount: u128], { reason: PolymeshRuntimeDevelopRuntimeRuntimeHoldReason, who: AccountId32, amount: u128 }>;
      /**
       * Total issuance was decreased by `amount`, creating a debt to be balanced.
       **/
      Rescinded: AugmentedEvent<ApiType, [amount: u128], { amount: u128 }>;
      /**
       * Some balance was reserved (moved from free to reserved).
       **/
      Reserved: AugmentedEvent<ApiType, [who: AccountId32, amount: u128], { who: AccountId32, amount: u128 }>;
      /**
       * Some balance was moved from the reserve of the first account to the second account.
       * Final argument indicates the destination balance type.
       **/
      ReserveRepatriated: AugmentedEvent<ApiType, [from: AccountId32, to: AccountId32, amount: u128, destinationStatus: FrameSupportTokensMiscBalanceStatus], { from: AccountId32, to: AccountId32, amount: u128, destinationStatus: FrameSupportTokensMiscBalanceStatus }>;
      /**
       * Some amount was restored into an account.
       **/
      Restored: AugmentedEvent<ApiType, [who: AccountId32, amount: u128], { who: AccountId32, amount: u128 }>;
      /**
       * Some amount was removed from the account (e.g. for misbehavior).
       **/
      Slashed: AugmentedEvent<ApiType, [who: AccountId32, amount: u128], { who: AccountId32, amount: u128 }>;
      /**
       * Some amount was suspended from an account (it can be restored later).
       **/
      Suspended: AugmentedEvent<ApiType, [who: AccountId32, amount: u128], { who: AccountId32, amount: u128 }>;
      /**
       * Some balance was thawed.
       **/
      Thawed: AugmentedEvent<ApiType, [who: AccountId32, amount: u128], { who: AccountId32, amount: u128 }>;
      /**
       * The `TotalIssuance` was forcefully changed.
       **/
      TotalIssuanceForced: AugmentedEvent<ApiType, [old: u128, new_: u128], { old: u128, new_: u128 }>;
      /**
       * Transfer succeeded.
       **/
      Transfer: AugmentedEvent<ApiType, [from: AccountId32, to: AccountId32, amount: u128], { from: AccountId32, to: AccountId32, amount: u128 }>;
      /**
       * The `transferred` balance is placed on hold at the `dest` account.
       **/
      TransferAndHold: AugmentedEvent<ApiType, [reason: PolymeshRuntimeDevelopRuntimeRuntimeHoldReason, source: AccountId32, dest: AccountId32, transferred: u128], { reason: PolymeshRuntimeDevelopRuntimeRuntimeHoldReason, source: AccountId32, dest: AccountId32, transferred: u128 }>;
      /**
       * A transfer of `amount` on hold from `source` to `dest` was initiated.
       **/
      TransferOnHold: AugmentedEvent<ApiType, [reason: PolymeshRuntimeDevelopRuntimeRuntimeHoldReason, source: AccountId32, dest: AccountId32, amount: u128], { reason: PolymeshRuntimeDevelopRuntimeRuntimeHoldReason, source: AccountId32, dest: AccountId32, amount: u128 }>;
      /**
       * Transfer with memo succeeded.
       **/
      TransferWithMemo: AugmentedEvent<ApiType, [from: AccountId32, to: AccountId32, amount: u128, memo: Option<PolymeshPrimitivesMemo>], { from: AccountId32, to: AccountId32, amount: u128, memo: Option<PolymeshPrimitivesMemo> }>;
      /**
       * An unexpected/defensive event was triggered.
       **/
      Unexpected: AugmentedEvent<ApiType, [PalletBalancesUnexpectedKind]>;
      /**
       * Some balance was unlocked.
       **/
      Unlocked: AugmentedEvent<ApiType, [who: AccountId32, amount: u128], { who: AccountId32, amount: u128 }>;
      /**
       * Some balance was unreserved (moved from reserved to free).
       **/
      Unreserved: AugmentedEvent<ApiType, [who: AccountId32, amount: u128], { who: AccountId32, amount: u128 }>;
      /**
       * An account was upgraded.
       **/
      Upgraded: AugmentedEvent<ApiType, [who: AccountId32], { who: AccountId32 }>;
      /**
       * Some amount was withdrawn from the account (e.g. for transaction fees).
       **/
      Withdraw: AugmentedEvent<ApiType, [who: AccountId32, amount: u128], { who: AccountId32, amount: u128 }>;
      /**
       * Generic event
       **/
      [key: string]: AugmentedEvent<ApiType>;
    };
    capitalDistribution: {
      /**
       * A token holder's benefit of a capital distribution for the given `CAId` was claimed.
       * 
       * (Caller DID, Holder/Claimant DID, CA's ID, updated distribution details, DID's benefit, DID's tax %)
       **/
      BenefitClaimed: AugmentedEvent<ApiType, [PolymeshPrimitivesEventOnly, PolymeshPrimitivesEventOnly, PalletCorporateActionsCaId, PalletCorporateActionsDistribution, u128, Permill]>;
      /**
       * A capital distribution, with details included,
       * was created by the DID (permissioned agent) for the CA identified by `CAId`.
       * 
       * (Agent DID, CA's ID, distribution details)
       **/
      Created: AugmentedEvent<ApiType, [PolymeshPrimitivesEventOnly, PalletCorporateActionsCaId, PalletCorporateActionsDistribution]>;
      /**
       * Stats from `push_benefit` was emitted.
       * 
       * (Agent DID, CA's ID, max requested DIDs, processed DIDs, failed DIDs)
       **/
      Reclaimed: AugmentedEvent<ApiType, [PolymeshPrimitivesEventOnly, PalletCorporateActionsCaId, u128]>;
      /**
       * A capital distribution was removed.
       * 
       * (Agent DID, CA's ID)
       **/
      Removed: AugmentedEvent<ApiType, [PolymeshPrimitivesEventOnly, PalletCorporateActionsCaId]>;
      /**
       * Generic event
       **/
      [key: string]: AugmentedEvent<ApiType>;
    };
    checkpoint: {
      /**
       * A checkpoint was created.
       * 
       * (caller DID, AssetId, checkpoint ID, total supply, checkpoint timestamp)
       **/
      CheckpointCreated: AugmentedEvent<ApiType, [Option<PolymeshPrimitivesIdentityId>, PolymeshPrimitivesAssetAssetId, u64, u128, u64]>;
      /**
       * The maximum complexity for an arbitrary asset's schedule set was changed.
       * 
       * (GC DID, the new maximum)
       **/
      MaximumSchedulesComplexityChanged: AugmentedEvent<ApiType, [PolymeshPrimitivesIdentityId, u64]>;
      /**
       * A checkpoint schedule was created.
       * 
       * (caller DID, AssetId, schedule id, schedule)
       **/
      ScheduleCreated: AugmentedEvent<ApiType, [PolymeshPrimitivesIdentityId, PolymeshPrimitivesAssetAssetId, u64, PolymeshPrimitivesCheckpointScheduleCheckpoints]>;
      /**
       * A checkpoint schedule was removed.
       * 
       * (caller DID, AssetId, schedule id, schedule)
       **/
      ScheduleRemoved: AugmentedEvent<ApiType, [PolymeshPrimitivesIdentityId, PolymeshPrimitivesAssetAssetId, u64, PolymeshPrimitivesCheckpointScheduleCheckpoints]>;
      /**
       * Generic event
       **/
      [key: string]: AugmentedEvent<ApiType>;
    };
    committeeMembership: {
      /**
       * The limit of how many active members there can be concurrently was changed.
       **/
      ActiveLimitChanged: AugmentedEvent<ApiType, [PolymeshPrimitivesIdentityId, u32, u32]>;
      /**
       * The given member was added; see the transaction for who.
       * caller DID, New member DID.
       **/
      MemberAdded: AugmentedEvent<ApiType, [PolymeshPrimitivesIdentityId, PolymeshPrimitivesIdentityId]>;
      /**
       * The given member was removed; see the transaction for who.
       * caller DID, member DID that get removed.
       **/
      MemberRemoved: AugmentedEvent<ApiType, [PolymeshPrimitivesIdentityId, PolymeshPrimitivesIdentityId]>;
      /**
       * The given member has been revoked at specific time-stamp.
       * caller DID, member DID that get revoked.
       **/
      MemberRevoked: AugmentedEvent<ApiType, [PolymeshPrimitivesIdentityId, PolymeshPrimitivesIdentityId]>;
      /**
       * The membership was reset; see the transaction for who the new set is.
       * caller DID, List of new members.
       **/
      MembersReset: AugmentedEvent<ApiType, [PolymeshPrimitivesIdentityId, Vec<PolymeshPrimitivesIdentityId>]>;
      /**
       * Two members were swapped; see the transaction for who.
       * caller DID, Removed DID, New add DID.
       **/
      MembersSwapped: AugmentedEvent<ApiType, [PolymeshPrimitivesIdentityId, PolymeshPrimitivesIdentityId, PolymeshPrimitivesIdentityId]>;
      /**
       * Generic event
       **/
      [key: string]: AugmentedEvent<ApiType>;
    };
    complianceManager: {
      /**
       * Emitted when an asset compliance for a given asset_id gets paused.
       * (caller DID, AssetId).
       **/
      AssetCompliancePaused: AugmentedEvent<ApiType, [PolymeshPrimitivesIdentityId, PolymeshPrimitivesAssetAssetId]>;
      /**
       * Emitted when an asset compliance is replaced.
       * Parameters: caller DID, AssetId, new asset compliance.
       **/
      AssetComplianceReplaced: AugmentedEvent<ApiType, [PolymeshPrimitivesIdentityId, PolymeshPrimitivesAssetAssetId, Vec<PolymeshPrimitivesComplianceManagerComplianceRequirement>]>;
      /**
       * Emitted when an asset compliance of a asset_id is reset.
       * (caller DID, AssetId).
       **/
      AssetComplianceReset: AugmentedEvent<ApiType, [PolymeshPrimitivesIdentityId, PolymeshPrimitivesAssetAssetId]>;
      /**
       * Emitted when an asset compliance for a given asset_id gets resume.
       * (caller DID, AssetId).
       **/
      AssetComplianceResumed: AugmentedEvent<ApiType, [PolymeshPrimitivesIdentityId, PolymeshPrimitivesAssetAssetId]>;
      /**
       * Emitted when compliance requirement get modified/change.
       * (caller DID, AssetId, ComplianceRequirement).
       **/
      ComplianceRequirementChanged: AugmentedEvent<ApiType, [PolymeshPrimitivesIdentityId, PolymeshPrimitivesAssetAssetId, PolymeshPrimitivesComplianceManagerComplianceRequirement]>;
      /**
       * Emitted when new compliance requirement is created.
       * (caller DID, AssetId, ComplianceRequirement).
       **/
      ComplianceRequirementCreated: AugmentedEvent<ApiType, [PolymeshPrimitivesIdentityId, PolymeshPrimitivesAssetAssetId, PolymeshPrimitivesComplianceManagerComplianceRequirement]>;
      /**
       * Emitted when a compliance requirement is removed.
       * (caller DID, AssetId, requirement_id).
       **/
      ComplianceRequirementRemoved: AugmentedEvent<ApiType, [PolymeshPrimitivesIdentityId, PolymeshPrimitivesAssetAssetId, u32]>;
      /**
       * Emitted when default claim issuer list for a given asset_id gets added.
       * (caller DID, AssetId, Added TrustedIssuer).
       **/
      TrustedDefaultClaimIssuerAdded: AugmentedEvent<ApiType, [PolymeshPrimitivesIdentityId, PolymeshPrimitivesAssetAssetId, PolymeshPrimitivesConditionTrustedIssuer]>;
      /**
       * Emitted when default claim issuer list for a given asset_id get removed.
       * (caller DID, AssetId, Removed TrustedIssuer).
       **/
      TrustedDefaultClaimIssuerRemoved: AugmentedEvent<ApiType, [PolymeshPrimitivesIdentityId, PolymeshPrimitivesAssetAssetId, PolymeshPrimitivesIdentityId]>;
      /**
       * Generic event
       **/
      [key: string]: AugmentedEvent<ApiType>;
    };
    contracts: {
      /**
       * A contract was called either by a plain account or another contract.
       * 
       * # Note
       * 
       * Please keep in mind that like all events this is only emitted for successful
       * calls. This is because on failure all storage changes including events are
       * rolled back.
       **/
      Called: AugmentedEvent<ApiType, [caller: PalletContractsOrigin, contract: AccountId32], { caller: PalletContractsOrigin, contract: AccountId32 }>;
      /**
       * A code with the specified hash was removed.
       **/
      CodeRemoved: AugmentedEvent<ApiType, [codeHash: H256, depositReleased: u128, remover: AccountId32], { codeHash: H256, depositReleased: u128, remover: AccountId32 }>;
      /**
       * Code with the specified hash has been stored.
       **/
      CodeStored: AugmentedEvent<ApiType, [codeHash: H256, depositHeld: u128, uploader: AccountId32], { codeHash: H256, depositHeld: u128, uploader: AccountId32 }>;
      /**
       * A contract's code was updated.
       **/
      ContractCodeUpdated: AugmentedEvent<ApiType, [contract: AccountId32, newCodeHash: H256, oldCodeHash: H256], { contract: AccountId32, newCodeHash: H256, oldCodeHash: H256 }>;
      /**
       * A custom event emitted by the contract.
       **/
      ContractEmitted: AugmentedEvent<ApiType, [contract: AccountId32, data: Bytes], { contract: AccountId32, data: Bytes }>;
      /**
       * A contract delegate called a code hash.
       * 
       * # Note
       * 
       * Please keep in mind that like all events this is only emitted for successful
       * calls. This is because on failure all storage changes including events are
       * rolled back.
       **/
      DelegateCalled: AugmentedEvent<ApiType, [contract: AccountId32, codeHash: H256], { contract: AccountId32, codeHash: H256 }>;
      /**
       * Contract deployed by address at the specified address.
       **/
      Instantiated: AugmentedEvent<ApiType, [deployer: AccountId32, contract: AccountId32], { deployer: AccountId32, contract: AccountId32 }>;
      /**
       * Some funds have been transferred and held as storage deposit.
       **/
      StorageDepositTransferredAndHeld: AugmentedEvent<ApiType, [from: AccountId32, to: AccountId32, amount: u128], { from: AccountId32, to: AccountId32, amount: u128 }>;
      /**
       * Some storage deposit funds have been transferred and released.
       **/
      StorageDepositTransferredAndReleased: AugmentedEvent<ApiType, [from: AccountId32, to: AccountId32, amount: u128], { from: AccountId32, to: AccountId32, amount: u128 }>;
      /**
       * Contract has been removed.
       * 
       * # Note
       * 
       * The only way for a contract to be removed and emitting this event is by calling
       * `seal_terminate`.
       **/
      Terminated: AugmentedEvent<ApiType, [contract: AccountId32, beneficiary: AccountId32], { contract: AccountId32, beneficiary: AccountId32 }>;
      /**
       * Generic event
       **/
      [key: string]: AugmentedEvent<ApiType>;
    };
    corporateAction: {
      /**
       * A CA was initiated.
       * (Agent DID, CA id, the CA, the CA details)
       **/
      CAInitiated: AugmentedEvent<ApiType, [PolymeshPrimitivesEventOnly, PalletCorporateActionsCaId, PalletCorporateActionsCorporateAction, Bytes]>;
      /**
       * A CA was linked to a set of docs.
       * (Agent DID, CA Id, List of doc identifiers)
       **/
      CALinkedToDoc: AugmentedEvent<ApiType, [PolymeshPrimitivesIdentityId, PalletCorporateActionsCaId, Vec<u32>]>;
      /**
       * A CA was removed.
       * (Agent DID, CA Id)
       **/
      CARemoved: AugmentedEvent<ApiType, [PolymeshPrimitivesEventOnly, PalletCorporateActionsCaId]>;
      /**
       * The set of default `TargetIdentities` for the asset changed.
       * (Agent DID, AssetId, New TargetIdentities)
       **/
      DefaultTargetIdentitiesChanged: AugmentedEvent<ApiType, [PolymeshPrimitivesIdentityId, PolymeshPrimitivesAssetAssetId, PalletCorporateActionsTargetIdentities]>;
      /**
       * The default withholding tax for the asset changed.
       * (Agent DID, AssetId, New Tax).
       **/
      DefaultWithholdingTaxChanged: AugmentedEvent<ApiType, [PolymeshPrimitivesIdentityId, PolymeshPrimitivesAssetAssetId, Permill]>;
      /**
       * The withholding tax specific to a DID for the asset changed.
       * (Agent DID, AssetId, Taxed DID, New Tax).
       **/
      DidWithholdingTaxChanged: AugmentedEvent<ApiType, [PolymeshPrimitivesIdentityId, PolymeshPrimitivesAssetAssetId, PolymeshPrimitivesIdentityId, Option<Permill>]>;
      /**
       * The maximum length of `details` in bytes was changed.
       * (GC DID, new length)
       **/
      MaxDetailsLengthChanged: AugmentedEvent<ApiType, [PolymeshPrimitivesIdentityId, u32]>;
      /**
       * A CA's record date changed.
       **/
      RecordDateChanged: AugmentedEvent<ApiType, [PolymeshPrimitivesEventOnly, PalletCorporateActionsCaId, PalletCorporateActionsCorporateAction]>;
      /**
       * Generic event
       **/
      [key: string]: AugmentedEvent<ApiType>;
    };
    corporateBallot: {
      /**
       * A corporate ballot was created.
       * 
       * (Agent DID, CA's ID, Voting start/end, Ballot metadata, RCV enabled?)
       **/
      Created: AugmentedEvent<ApiType, [PolymeshPrimitivesIdentityId, PalletCorporateActionsCaId, PalletCorporateActionsBallotBallotTimeRange, PalletCorporateActionsBallotBallotMeta, bool]>;
      /**
       * A corporate ballot changed its metadata.
       * 
       * (Agent DID, CA's ID, New metadata)
       **/
      MetaChanged: AugmentedEvent<ApiType, [PolymeshPrimitivesIdentityId, PalletCorporateActionsCaId, PalletCorporateActionsBallotBallotMeta]>;
      /**
       * A corporate ballot changed its start/end date range.
       * 
       * (Agent DID, CA's ID, Voting start/end)
       **/
      RangeChanged: AugmentedEvent<ApiType, [PolymeshPrimitivesIdentityId, PalletCorporateActionsCaId, PalletCorporateActionsBallotBallotTimeRange]>;
      /**
       * A corporate ballot changed its RCV support.
       * 
       * (Agent DID, CA's ID, New support)
       **/
      RCVChanged: AugmentedEvent<ApiType, [PolymeshPrimitivesIdentityId, PalletCorporateActionsCaId, bool]>;
      /**
       * A corporate ballot was removed.
       * 
       * (Agent DID, CA's ID)
       **/
      Removed: AugmentedEvent<ApiType, [PolymeshPrimitivesEventOnly, PalletCorporateActionsCaId]>;
      /**
       * A vote was cast in a corporate ballot.
       * 
       * (voter DID, CAId, Votes)
       **/
      VoteCast: AugmentedEvent<ApiType, [PolymeshPrimitivesIdentityId, PalletCorporateActionsCaId, Vec<PalletCorporateActionsBallotBallotVote>]>;
      /**
       * Generic event
       **/
      [key: string]: AugmentedEvent<ApiType>;
    };
    didRegistrars: {
      /**
       * The limit of how many active members there can be concurrently was changed.
       **/
      ActiveLimitChanged: AugmentedEvent<ApiType, [PolymeshPrimitivesIdentityId, u32, u32]>;
      /**
       * The given member was added; see the transaction for who.
       * caller DID, New member DID.
       **/
      MemberAdded: AugmentedEvent<ApiType, [PolymeshPrimitivesIdentityId, PolymeshPrimitivesIdentityId]>;
      /**
       * The given member was removed; see the transaction for who.
       * caller DID, member DID that get removed.
       **/
      MemberRemoved: AugmentedEvent<ApiType, [PolymeshPrimitivesIdentityId, PolymeshPrimitivesIdentityId]>;
      /**
       * The given member has been revoked at specific time-stamp.
       * caller DID, member DID that get revoked.
       **/
      MemberRevoked: AugmentedEvent<ApiType, [PolymeshPrimitivesIdentityId, PolymeshPrimitivesIdentityId]>;
      /**
       * The membership was reset; see the transaction for who the new set is.
       * caller DID, List of new members.
       **/
      MembersReset: AugmentedEvent<ApiType, [PolymeshPrimitivesIdentityId, Vec<PolymeshPrimitivesIdentityId>]>;
      /**
       * Two members were swapped; see the transaction for who.
       * caller DID, Removed DID, New add DID.
       **/
      MembersSwapped: AugmentedEvent<ApiType, [PolymeshPrimitivesIdentityId, PolymeshPrimitivesIdentityId, PolymeshPrimitivesIdentityId]>;
      /**
       * Generic event
       **/
      [key: string]: AugmentedEvent<ApiType>;
    };
    electionProviderMultiPhase: {
      /**
       * An election failed.
       * 
       * Not much can be said about which computes failed in the process.
       **/
      ElectionFailed: AugmentedEvent<ApiType, []>;
      /**
       * The election has been finalized, with the given computation and score.
       **/
      ElectionFinalized: AugmentedEvent<ApiType, [compute: PalletElectionProviderMultiPhaseElectionCompute, score: SpNposElectionsElectionScore], { compute: PalletElectionProviderMultiPhaseElectionCompute, score: SpNposElectionsElectionScore }>;
      /**
       * There was a phase transition in a given round.
       **/
      PhaseTransitioned: AugmentedEvent<ApiType, [from: PalletElectionProviderMultiPhasePhase, to: PalletElectionProviderMultiPhasePhase, round: u32], { from: PalletElectionProviderMultiPhasePhase, to: PalletElectionProviderMultiPhasePhase, round: u32 }>;
      /**
       * An account has been rewarded for their signed submission being finalized.
       **/
      Rewarded: AugmentedEvent<ApiType, [account: AccountId32, value: u128], { account: AccountId32, value: u128 }>;
      /**
       * An account has been slashed for submitting an invalid signed submission.
       **/
      Slashed: AugmentedEvent<ApiType, [account: AccountId32, value: u128], { account: AccountId32, value: u128 }>;
      /**
       * A solution was stored with the given compute.
       * 
       * The `origin` indicates the origin of the solution. If `origin` is `Some(AccountId)`,
       * the stored solution was submitted in the signed phase by a miner with the `AccountId`.
       * Otherwise, the solution was stored either during the unsigned phase or by
       * `T::ForceOrigin`. The `bool` is `true` when a previous solution was ejected to make
       * room for this one.
       **/
      SolutionStored: AugmentedEvent<ApiType, [compute: PalletElectionProviderMultiPhaseElectionCompute, origin: Option<AccountId32>, prevEjected: bool], { compute: PalletElectionProviderMultiPhaseElectionCompute, origin: Option<AccountId32>, prevEjected: bool }>;
      /**
       * Generic event
       **/
      [key: string]: AugmentedEvent<ApiType>;
    };
    externalAgents: {
      /**
       * An agent was added.
       * 
       * (Caller/Agent DID, Agent's AssetId, Agent's group)
       **/
      AgentAdded: AugmentedEvent<ApiType, [PolymeshPrimitivesEventOnly, PolymeshPrimitivesAssetAssetId, PolymeshPrimitivesAgentAgentGroup]>;
      /**
       * An agent was removed.
       * 
       * (Caller DID, Agent's AssetId, Agent's DID)
       **/
      AgentRemoved: AugmentedEvent<ApiType, [PolymeshPrimitivesEventOnly, PolymeshPrimitivesAssetAssetId, PolymeshPrimitivesIdentityId]>;
      /**
       * An agent's group was changed.
       * 
       * (Caller DID, Agent's AssetId, Agent's DID, The new group of the agent)
       **/
      GroupChanged: AugmentedEvent<ApiType, [PolymeshPrimitivesEventOnly, PolymeshPrimitivesAssetAssetId, PolymeshPrimitivesIdentityId, PolymeshPrimitivesAgentAgentGroup]>;
      /**
       * An Agent Group was created.
       * 
       * (Caller DID, AG's AssetId, AG's ID, AG's permissions)
       **/
      GroupCreated: AugmentedEvent<ApiType, [PolymeshPrimitivesEventOnly, PolymeshPrimitivesAssetAssetId, u32, PolymeshPrimitivesSecondaryKeyExtrinsicPermissions]>;
      /**
       * An Agent Group's permissions was updated.
       * 
       * (Caller DID, AG's AssetId, AG's ID, AG's new permissions)
       **/
      GroupPermissionsUpdated: AugmentedEvent<ApiType, [PolymeshPrimitivesEventOnly, PolymeshPrimitivesAssetAssetId, u32, PolymeshPrimitivesSecondaryKeyExtrinsicPermissions]>;
      /**
       * Generic event
       **/
      [key: string]: AugmentedEvent<ApiType>;
    };
    grandpa: {
      /**
       * New authority set has been applied.
       **/
      NewAuthorities: AugmentedEvent<ApiType, [authoritySet: Vec<ITuple<[SpConsensusGrandpaAppPublic, u64]>>], { authoritySet: Vec<ITuple<[SpConsensusGrandpaAppPublic, u64]>> }>;
      /**
       * Current authority set has been paused.
       **/
      Paused: AugmentedEvent<ApiType, []>;
      /**
       * Current authority set has been resumed.
       **/
      Resumed: AugmentedEvent<ApiType, []>;
      /**
       * Generic event
       **/
      [key: string]: AugmentedEvent<ApiType>;
    };
    historical: {
      /**
       * The merkle roots of up to this session index were pruned
       **/
      RootsPruned: AugmentedEvent<ApiType, [upTo: u32], { upTo: u32 }>;
      /**
       * The merkle root of the validators of the said session were stored
       **/
      RootStored: AugmentedEvent<ApiType, [index: u32], { index: u32 }>;
      /**
       * Generic event
       **/
      [key: string]: AugmentedEvent<ApiType>;
    };
    identity: {
      /**
       * New authorization added.
       * 
       * (authorised_by, target_did, target_key, auth_id, authorization_data, expiry)
       **/
      AuthorizationAdded: AugmentedEvent<ApiType, [PolymeshPrimitivesIdentityId, Option<PolymeshPrimitivesIdentityId>, Option<AccountId32>, u64, PolymeshPrimitivesAuthorizationAuthorizationData, Option<u64>]>;
      /**
       * Authorization consumed.
       * 
       * (authorized_identity, authorized_key, auth_id)
       **/
      AuthorizationConsumed: AugmentedEvent<ApiType, [Option<PolymeshPrimitivesIdentityId>, Option<AccountId32>, u64]>;
      /**
       * Authorization rejected by the user who was authorized.
       * 
       * (authorized_identity, authorized_key, auth_id)
       **/
      AuthorizationRejected: AugmentedEvent<ApiType, [Option<PolymeshPrimitivesIdentityId>, Option<AccountId32>, u64]>;
      /**
       * Accepting Authorization retry limit reached.
       * 
       * (authorized_identity, authorized_key, auth_id)
       **/
      AuthorizationRetryLimitReached: AugmentedEvent<ApiType, [Option<PolymeshPrimitivesIdentityId>, Option<AccountId32>, u64]>;
      /**
       * Authorization revoked by the authorizer.
       * 
       * (authorized_identity, authorized_key, auth_id)
       **/
      AuthorizationRevoked: AugmentedEvent<ApiType, [Option<PolymeshPrimitivesIdentityId>, Option<AccountId32>, u64]>;
      /**
       * Child identity created.
       * 
       * (Parent DID, Child DID, primary key)
       **/
      ChildDidCreated: AugmentedEvent<ApiType, [PolymeshPrimitivesIdentityId, PolymeshPrimitivesIdentityId, AccountId32]>;
      /**
       * Child identity unlinked from parent identity.
       * 
       * (Caller DID, Parent DID, Child DID)
       **/
      ChildDidUnlinked: AugmentedEvent<ApiType, [PolymeshPrimitivesIdentityId, PolymeshPrimitivesIdentityId, PolymeshPrimitivesIdentityId]>;
      /**
       * Claim added to identity.
       * 
       * (DID, claim)
       **/
      ClaimAdded: AugmentedEvent<ApiType, [PolymeshPrimitivesIdentityId, PolymeshPrimitivesIdentityClaim]>;
      /**
       * Claim revoked from identity.
       * 
       * (DID, claim)
       **/
      ClaimRevoked: AugmentedEvent<ApiType, [PolymeshPrimitivesIdentityId, PolymeshPrimitivesIdentityClaim]>;
      /**
       * A new CustomClaimType was added.
       * 
       * (DID, id, Type)
       **/
      CustomClaimTypeAdded: AugmentedEvent<ApiType, [PolymeshPrimitivesIdentityId, u32, Bytes]>;
      /**
       * Identity created.
       * 
       * (DID, primary key, secondary keys)
       **/
      DidCreated: AugmentedEvent<ApiType, [PolymeshPrimitivesIdentityId, AccountId32, Vec<PolymeshPrimitivesSecondaryKey>]>;
      /**
       * Primary key of identity changed.
       * 
       * (DID, old primary key account ID, new ID)
       **/
      PrimaryKeyUpdated: AugmentedEvent<ApiType, [PolymeshPrimitivesIdentityId, AccountId32, AccountId32]>;
      /**
       * A secondary key left their identity.
       * 
       * (DID, secondary key)
       **/
      SecondaryKeyLeftIdentity: AugmentedEvent<ApiType, [PolymeshPrimitivesIdentityId, AccountId32]>;
      /**
       * Secondary key permissions updated.
       * 
       * (DID, updated secondary key, previous permissions, new permissions)
       **/
      SecondaryKeyPermissionsUpdated: AugmentedEvent<ApiType, [PolymeshPrimitivesIdentityId, AccountId32, PolymeshPrimitivesSecondaryKeyPermissions, PolymeshPrimitivesSecondaryKeyPermissions]>;
      /**
       * Secondary keys added to identity.
       * 
       * (DID, new keys)
       **/
      SecondaryKeysAdded: AugmentedEvent<ApiType, [PolymeshPrimitivesIdentityId, Vec<PolymeshPrimitivesSecondaryKey>]>;
      /**
       * All Secondary keys of the identity ID are frozen.
       * 
       * (DID)
       **/
      SecondaryKeysFrozen: AugmentedEvent<ApiType, [PolymeshPrimitivesIdentityId]>;
      /**
       * Secondary keys removed from identity.
       * 
       * (DID, the keys that got removed)
       **/
      SecondaryKeysRemoved: AugmentedEvent<ApiType, [PolymeshPrimitivesIdentityId, Vec<AccountId32>]>;
      /**
       * All Secondary keys of the identity ID are unfrozen.
       * 
       * (DID)
       **/
      SecondaryKeysUnfrozen: AugmentedEvent<ApiType, [PolymeshPrimitivesIdentityId]>;
      /**
       * Generic event
       **/
      [key: string]: AugmentedEvent<ApiType>;
    };
    imOnline: {
      /**
       * At the end of the session, no offence was committed.
       **/
      AllGood: AugmentedEvent<ApiType, []>;
      /**
       * A new heartbeat was received from `AuthorityId`.
       **/
      HeartbeatReceived: AugmentedEvent<ApiType, [authorityId: PalletImOnlineSr25519AppSr25519Public], { authorityId: PalletImOnlineSr25519AppSr25519Public }>;
      /**
       * At the end of the session, at least one validator was found to be offline.
       **/
      SomeOffline: AugmentedEvent<ApiType, [offline: Vec<ITuple<[AccountId32, SpStakingExposure]>>], { offline: Vec<ITuple<[AccountId32, SpStakingExposure]>> }>;
      /**
       * Generic event
       **/
      [key: string]: AugmentedEvent<ApiType>;
    };
    indices: {
      /**
       * A deposit to reserve an index has been poked/reconsidered.
       **/
      DepositPoked: AugmentedEvent<ApiType, [who: AccountId32, index: u32, oldDeposit: u128, newDeposit: u128], { who: AccountId32, index: u32, oldDeposit: u128, newDeposit: u128 }>;
      /**
       * A account index was assigned.
       **/
      IndexAssigned: AugmentedEvent<ApiType, [who: AccountId32, index: u32], { who: AccountId32, index: u32 }>;
      /**
       * A account index has been freed up (unassigned).
       **/
      IndexFreed: AugmentedEvent<ApiType, [index: u32], { index: u32 }>;
      /**
       * A account index has been frozen to its current account ID.
       **/
      IndexFrozen: AugmentedEvent<ApiType, [index: u32, who: AccountId32], { index: u32, who: AccountId32 }>;
      /**
       * Generic event
       **/
      [key: string]: AugmentedEvent<ApiType>;
    };
    multiSig: {
      /**
       * A Multisig has added an admin DID.
       **/
      MultiSigAddedAdmin: AugmentedEvent<ApiType, [callerDid: PolymeshPrimitivesIdentityId, multisig: AccountId32, adminDid: PolymeshPrimitivesIdentityId], { callerDid: PolymeshPrimitivesIdentityId, multisig: AccountId32, adminDid: PolymeshPrimitivesIdentityId }>;
      /**
       * A Multisig has been created.
       **/
      MultiSigCreated: AugmentedEvent<ApiType, [callerDid: PolymeshPrimitivesIdentityId, multisig: AccountId32, caller: AccountId32, signers: Vec<AccountId32>, sigsRequired: u64], { callerDid: PolymeshPrimitivesIdentityId, multisig: AccountId32, caller: AccountId32, signers: Vec<AccountId32>, sigsRequired: u64 }>;
      /**
       * A Multisig has removed it's admin DID.
       **/
      MultiSigRemovedAdmin: AugmentedEvent<ApiType, [callerDid: PolymeshPrimitivesIdentityId, multisig: AccountId32, adminDid: PolymeshPrimitivesIdentityId], { callerDid: PolymeshPrimitivesIdentityId, multisig: AccountId32, adminDid: PolymeshPrimitivesIdentityId }>;
      /**
       * A Multisig has removed it's paying DID.
       **/
      MultiSigRemovedPayingDid: AugmentedEvent<ApiType, [callerDid: PolymeshPrimitivesIdentityId, multisig: AccountId32, payingDid: PolymeshPrimitivesIdentityId], { callerDid: PolymeshPrimitivesIdentityId, multisig: AccountId32, payingDid: PolymeshPrimitivesIdentityId }>;
      /**
       * A new signer has been added to a Multisig.
       **/
      MultiSigSignerAdded: AugmentedEvent<ApiType, [callerDid: PolymeshPrimitivesIdentityId, multisig: AccountId32, signer: AccountId32], { callerDid: PolymeshPrimitivesIdentityId, multisig: AccountId32, signer: AccountId32 }>;
      /**
       * New keys have been authorized to be signers on a Multisig.
       **/
      MultiSigSignersAuthorized: AugmentedEvent<ApiType, [callerDid: PolymeshPrimitivesIdentityId, multisig: AccountId32, signers: Vec<AccountId32>], { callerDid: PolymeshPrimitivesIdentityId, multisig: AccountId32, signers: Vec<AccountId32> }>;
      /**
       * Signers have been removed from a Multisig.
       **/
      MultiSigSignersRemoved: AugmentedEvent<ApiType, [callerDid: PolymeshPrimitivesIdentityId, multisig: AccountId32, signers: Vec<AccountId32>], { callerDid: PolymeshPrimitivesIdentityId, multisig: AccountId32, signers: Vec<AccountId32> }>;
      /**
       * A Multisig has changed its required number of approvals.
       **/
      MultiSigSignersRequiredChanged: AugmentedEvent<ApiType, [callerDid: Option<PolymeshPrimitivesIdentityId>, multisig: AccountId32, sigsRequired: u64], { callerDid: Option<PolymeshPrimitivesIdentityId>, multisig: AccountId32, sigsRequired: u64 }>;
      /**
       * A Multisig proposal has been created.
       **/
      ProposalAdded: AugmentedEvent<ApiType, [callerDid: Option<PolymeshPrimitivesIdentityId>, multisig: AccountId32, proposalId: u64], { callerDid: Option<PolymeshPrimitivesIdentityId>, multisig: AccountId32, proposalId: u64 }>;
      /**
       * A signer has voted to approve a Multisig proposal.
       **/
      ProposalApprovalVote: AugmentedEvent<ApiType, [callerDid: Option<PolymeshPrimitivesIdentityId>, multisig: AccountId32, signer: AccountId32, proposalId: u64], { callerDid: Option<PolymeshPrimitivesIdentityId>, multisig: AccountId32, signer: AccountId32, proposalId: u64 }>;
      /**
       * A Multisig proposal has been approved.
       **/
      ProposalApproved: AugmentedEvent<ApiType, [callerDid: Option<PolymeshPrimitivesIdentityId>, multisig: AccountId32, proposalId: u64], { callerDid: Option<PolymeshPrimitivesIdentityId>, multisig: AccountId32, proposalId: u64 }>;
      /**
       * A Multisig proposal has been executed.
       **/
      ProposalExecuted: AugmentedEvent<ApiType, [callerDid: Option<PolymeshPrimitivesIdentityId>, multisig: AccountId32, proposalId: u64, result: Result<Null, SpRuntimeDispatchError>], { callerDid: Option<PolymeshPrimitivesIdentityId>, multisig: AccountId32, proposalId: u64, result: Result<Null, SpRuntimeDispatchError> }>;
      /**
       * A Multisig proposal has been rejected.
       **/
      ProposalRejected: AugmentedEvent<ApiType, [callerDid: Option<PolymeshPrimitivesIdentityId>, multisig: AccountId32, proposalId: u64], { callerDid: Option<PolymeshPrimitivesIdentityId>, multisig: AccountId32, proposalId: u64 }>;
      /**
       * A signer has voted to reject a Multisig proposal.
       **/
      ProposalRejectionVote: AugmentedEvent<ApiType, [callerDid: Option<PolymeshPrimitivesIdentityId>, multisig: AccountId32, signer: AccountId32, proposalId: u64], { callerDid: Option<PolymeshPrimitivesIdentityId>, multisig: AccountId32, signer: AccountId32, proposalId: u64 }>;
      /**
       * Generic event
       **/
      [key: string]: AugmentedEvent<ApiType>;
    };
    nft: {
      /**
       * Emitted when a new nft collection is created.
       **/
      NftCollectionCreated: AugmentedEvent<ApiType, [PolymeshPrimitivesIdentityId, PolymeshPrimitivesAssetAssetId, u64]>;
      /**
       * Emitted when NFTs were issued, redeemed or transferred.
       * Contains the [`IdentityId`] of the receiver/issuer/redeemer, the [`NFTs`], the [`AssetHolder`] of the source, the [`AssetHolder`]
       * of the destination and the [`HoldingsUpdateReason`].
       **/
      NFTHoldingsUpdated: AugmentedEvent<ApiType, [PolymeshPrimitivesIdentityId, PolymeshPrimitivesNftNfTs, Option<PolymeshPrimitivesAssetAssetHolder>, Option<PolymeshPrimitivesAssetAssetHolder>, PolymeshPrimitivesAssetHoldingsUpdateReason]>;
      /**
       * Generic event
       **/
      [key: string]: AugmentedEvent<ApiType>;
    };
    offences: {
      /**
       * There is an offence reported of the given `kind` happened at the `session_index` and
       * (kind-specific) time slot. This event is not deposited for duplicate slashes.
       * \[kind, timeslot\].
       **/
      Offence: AugmentedEvent<ApiType, [kind: U8aFixed, timeslot: Bytes], { kind: U8aFixed, timeslot: Bytes }>;
      /**
       * Generic event
       **/
      [key: string]: AugmentedEvent<ApiType>;
    };
    pips: {
      /**
       * The maximum number of active PIPs was changed.
       * 
       * Parameters:
       * - `IdentityId`: The DID of the caller.
       * - `u32`: The old active PIP limit.
       * - `u32`: The new active PIP limit.
       **/
      ActivePipLimitChanged: AugmentedEvent<ApiType, [PolymeshPrimitivesIdentityId, u32, u32]>;
      /**
       * The default enactment period was changed.
       * 
       * Parameters:
       * - `IdentityId`: The DID of the caller.
       * - `BlockNumber`: The old enactment period.
       * - `BlockNumber`: The new enactment period.
       **/
      DefaultEnactmentPeriodChanged: AugmentedEvent<ApiType, [PolymeshPrimitivesIdentityId, u32, u32]>;
      /**
       * Cancelling the PIP execution failed in the scheduler pallet.
       * 
       * Parameters:
       * - `PipId`: The ID of the PIP.
       **/
      ExecutionCancellingFailed: AugmentedEvent<ApiType, [u32]>;
      /**
       * The execution of a PIP was scheduled.
       * 
       * Parameters:
       * - `IdentityId`: The DID of the caller.
       * - `PipId`: The ID of the PIP.
       * - `BlockNumber`: The block number at which the PIP is scheduled for execution.
       **/
      ExecutionScheduled: AugmentedEvent<ApiType, [PolymeshPrimitivesIdentityId, u32, u32]>;
      /**
       * Scheduling of the PIP for execution failed in the scheduler pallet.
       * 
       * Parameters:
       * - `IdentityId`: The DID of the caller.
       * - `PipId`: The ID of the PIP.
       * - `BlockNumber`: The block number at which the PIP was scheduled for execution.
       **/
      ExecutionSchedulingFailed: AugmentedEvent<ApiType, [PolymeshPrimitivesIdentityId, u32, u32]>;
      /**
       * The PIP has been scheduled for expiry.
       * 
       * Parameters:
       * - `IdentityId`: The DID of the caller.
       * - `PipId`: The ID of the PIP.
       * - `BlockNumber`: The block number at which the PIP is scheduled for expiry.
       **/
      ExpiryScheduled: AugmentedEvent<ApiType, [PolymeshPrimitivesIdentityId, u32, u32]>;
      /**
       * Scheduling of the PIP for expiry failed in the scheduler pallet.
       * 
       * Parameters:
       * - `IdentityId`: The DID of the caller.
       * - `PipId`: The ID of the PIP.
       * - `BlockNumber`: The block number at which the PIP was scheduled for expiry.
       **/
      ExpirySchedulingFailed: AugmentedEvent<ApiType, [PolymeshPrimitivesIdentityId, u32, u32]>;
      /**
       * Historical PIPs Pruning has been set.
       * 
       * Parameters:
       * - `IdentityId`: The DID of the caller.
       * - `bool`: The old value of the pruning setting.
       * - `bool`: The new value of the pruning setting.
       **/
      HistoricalPipsPruned: AugmentedEvent<ApiType, [PolymeshPrimitivesIdentityId, bool, bool]>;
      /**
       * The maximum number of times a PIP can be skipped was changed.
       * 
       * Parameters:
       * - `IdentityId`: The DID of the caller.
       * - `SkippedCount`: The old skip count.
       * - `SkippedCount`: The new skip count.
       **/
      MaxPipSkipCountChanged: AugmentedEvent<ApiType, [PolymeshPrimitivesIdentityId, u8, u8]>;
      /**
       * The minimum deposit amount for proposals was changed.
       * 
       * Parameters:
       * - `IdentityId`: The DID of the caller.
       * - `Balance`: The old deposit amount.
       * - `Balance`: The new deposit amount.
       **/
      MinimumProposalDepositChanged: AugmentedEvent<ApiType, [PolymeshPrimitivesIdentityId, u128, u128]>;
      /**
       * The expiry time for pending PIPs was changed.
       * 
       * Parameters:
       * - `IdentityId`: The DID of the caller.
       * - `MaybeBlock<T::BlockNumber>`: The old expiry time.
       * - `MaybeBlock<T::BlockNumber>`: The new expiry time.
       **/
      PendingPipExpiryChanged: AugmentedEvent<ApiType, [PolymeshPrimitivesIdentityId, PolymeshPrimitivesMaybeBlock, PolymeshPrimitivesMaybeBlock]>;
      /**
       * A PIP was closed.
       * 
       * Parameters:
       * - `IdentityId`: The DID of the caller.
       * - `PipId`: The ID of the PIP.
       * - `bool`: Indicates whether the data was pruned.
       **/
      PipClosed: AugmentedEvent<ApiType, [PolymeshPrimitivesIdentityId, u32, bool]>;
      /**
       * A PIP in the snapshot queue was skipped.
       * 
       * Parameters:
       * - `IdentityId`: The DID of the caller.
       * - `PipId`: The ID of the PIP.
       * - `SkippedCount`: The new skip count.
       **/
      PipSkipped: AugmentedEvent<ApiType, [PolymeshPrimitivesIdentityId, u32, u8]>;
      /**
       * A PIP was created with a specified `Balance` stake.
       * 
       * Parameters:
       * - `IdentityId`: The DID of the caller.
       * - `Proposer<T::AccountId>`: The proposer of the PIP.
       * - `PipId`: The ID of the PIP.
       * - `Balance`: The deposit amount.
       * - `Option<Url>`: The URL for proposal discussion.
       * - `Option<PipDescription>`: The description of the proposal.
       * - `MaybeBlock<T::BlockNumber>`: The expiry time of the proposal.
       * - `ProposalData`: The data of the proposal.
       **/
      ProposalCreated: AugmentedEvent<ApiType, [PolymeshPrimitivesIdentityId, PalletPipsProposer, u32, u128, Option<Bytes>, Option<Bytes>, PolymeshPrimitivesMaybeBlock, PalletPipsProposalData]>;
      /**
       * A proposal was refunded.
       * 
       * Parameters:
       * - `IdentityId`: The DID of the caller.
       * - `PipId`: The ID of the PIP.
       * - `Balance`: The total amount refunded.
       **/
      ProposalRefund: AugmentedEvent<ApiType, [PolymeshPrimitivesIdentityId, u32, u128]>;
      /**
       * The state of a proposal was updated.
       * 
       * Parameters:
       * - `IdentityId`: The DID of the caller.
       * - `PipId`: The ID of the PIP.
       * - `ProposalState`: The new state of the proposal.
       **/
      ProposalStateUpdated: AugmentedEvent<ApiType, [PolymeshPrimitivesIdentityId, u32, PalletPipsProposalState]>;
      /**
       * The snapshot was cleared.
       * 
       * Parameters:
       * - `IdentityId`: The DID of the caller.
       * - `SnapshotId`: The ID of the snapshot.
       **/
      SnapshotCleared: AugmentedEvent<ApiType, [PolymeshPrimitivesIdentityId, u32]>;
      /**
       * Results were enacted for some PIPs in the snapshot queue.
       * 
       * Parameters:
       * - `IdentityId`: The DID of the caller.
       * - `Option<SnapshotId>`: The ID of the snapshot, if any.
       * - `Vec<(PipId, SkippedCount)>`: The list of skipped PIPs with their new skip counts.
       * - `Vec<PipId>`: The list of rejected PIPs.
       * - `Vec<PipId>`: The list of approved PIPs.
       **/
      SnapshotResultsEnacted: AugmentedEvent<ApiType, [PolymeshPrimitivesIdentityId, Option<u32>, Vec<ITuple<[u32, u8]>>, Vec<u32>, Vec<u32>]>;
      /**
       * A new snapshot was taken.
       * 
       * Parameters:
       * - `IdentityId`: The DID of the caller.
       * - `SnapshotId`: The ID of the snapshot.
       * - `Vec<SnapshottedPip>`: The list of PIPs in the snapshot.
       **/
      SnapshotTaken: AugmentedEvent<ApiType, [PolymeshPrimitivesIdentityId, u32, Vec<PalletPipsSnapshottedPip>]>;
      /**
       * An account voted on a proposal.
       * 
       * Parameters:
       * - `IdentityId`: The DID of the caller.
       * - `T::AccountId`: The account that voted.
       * - `PipId`: The ID of the PIP.
       * - `bool`: The vote (true for aye, false for nay).
       * - `Balance`: The deposit amount of the vote.
       **/
      Voted: AugmentedEvent<ApiType, [PolymeshPrimitivesIdentityId, AccountId32, u32, bool, u128]>;
      /**
       * Generic event
       **/
      [key: string]: AugmentedEvent<ApiType>;
    };
    polymeshCommittee: {
      /**
       * A motion was approved by the required threshold with the following
       * tally (yes votes, no votes and total seats given respectively as `MemberCount`).
       * Parameters: caller DID, proposal hash, yay vote count, nay vote count, total seats.
       **/
      Approved: AugmentedEvent<ApiType, [Option<PolymeshPrimitivesIdentityId>, H256, u32, u32, u32]>;
      /**
       * A motion was executed; `DispatchResult` is `Ok(())` if returned without error.
       * Parameters: caller DID, proposal hash, result of proposal dispatch.
       **/
      Executed: AugmentedEvent<ApiType, [Option<PolymeshPrimitivesIdentityId>, H256, Result<Null, SpRuntimeDispatchError>]>;
      /**
       * Proposal expiry time has been updated.
       * Parameters: caller DID, new expiry time (if any).
       **/
      ExpiresAfterUpdated: AugmentedEvent<ApiType, [PolymeshPrimitivesIdentityId, PolymeshPrimitivesMaybeBlock]>;
      /**
       * Final votes on a motion (given hash)
       * caller DID, ProposalIndex, Proposal hash, yes voters, no voter
       **/
      FinalVotes: AugmentedEvent<ApiType, [Option<PolymeshPrimitivesIdentityId>, u32, H256, Vec<PolymeshPrimitivesIdentityId>, Vec<PolymeshPrimitivesIdentityId>]>;
      /**
       * A motion (given hash) has been proposed (by given account) with a threshold (given `MemberCount`).
       * Parameters: caller DID, proposal index, proposal hash.
       **/
      Proposed: AugmentedEvent<ApiType, [PolymeshPrimitivesIdentityId, u32, H256]>;
      /**
       * A motion was rejected by the required threshold with the following
       * tally (yes votes, no votes and total seats given respectively as `MemberCount`).
       * Parameters: caller DID, proposal hash, yay vote count, nay vote count, total seats.
       **/
      Rejected: AugmentedEvent<ApiType, [Option<PolymeshPrimitivesIdentityId>, H256, u32, u32, u32]>;
      /**
       * Release coordinator has been updated.
       * Parameters: DID of the release coordinator.
       **/
      ReleaseCoordinatorUpdated: AugmentedEvent<ApiType, [Option<PolymeshPrimitivesIdentityId>]>;
      /**
       * A motion (given hash) has been voted on by given account, leaving
       * a tally (yes votes, no votes and total seats given respectively as `MemberCount`).
       * caller DID, Proposal index, Proposal hash, current vote, yay vote count, nay vote count, total seats.
       **/
      Voted: AugmentedEvent<ApiType, [PolymeshPrimitivesIdentityId, u32, H256, bool, u32, u32, u32]>;
      /**
       * A vote on a motion (given hash) has been retracted.
       * caller DID, ProposalIndex, Proposal hash, vote that was retracted
       **/
      VoteRetracted: AugmentedEvent<ApiType, [PolymeshPrimitivesIdentityId, u32, H256, bool]>;
      /**
       * Voting threshold has been updated
       * Parameters: caller DID, numerator, denominator
       **/
      VoteThresholdUpdated: AugmentedEvent<ApiType, [PolymeshPrimitivesIdentityId, u32, u32]>;
      /**
       * Generic event
       **/
      [key: string]: AugmentedEvent<ApiType>;
    };
    polymeshContracts: {
      /**
       * Emitted when a contract starts supporting a new API upgrade.
       * Contains the [`Api`], [`ChainVersion`], and the bytes for the code hash.
       **/
      ApiHashUpdated: AugmentedEvent<ApiType, [PolymeshContractsApi, PolymeshContractsChainVersion, H256]>;
      /**
       * Emitted when a contract calls into the runtime.
       * Contains the account id set by the contract owner and the [`ExtrinsicId`].
       **/
      SCRuntimeCall: AugmentedEvent<ApiType, [AccountId32, PolymeshContractsChainExtensionExtrinsicId]>;
      /**
       * Generic event
       **/
      [key: string]: AugmentedEvent<ApiType>;
    };
    portfolio: {
      /**
       * Allow another identity to create portfolios.
       * 
       * # Parameters
       * * [`IdentityId`] of the caller.
       * * [`IdentityId`] allowed to create portfolios.
       **/
      AllowIdentityToCreatePortfolios: AugmentedEvent<ApiType, [PolymeshPrimitivesIdentityId, PolymeshPrimitivesIdentityId]>;
      /**
       * Funds have moved between portfolios
       * 
       * # Parameters
       * * Origin DID.
       * * Source portfolio.
       * * Destination portfolio.
       * * The type of fund that was moved.
       * * Optional memo for the move.
       **/
      FundsMovedBetweenPortfolios: AugmentedEvent<ApiType, [PolymeshPrimitivesIdentityId, PolymeshPrimitivesIdentityIdPortfolioId, PolymeshPrimitivesIdentityIdPortfolioId, PolymeshPrimitivesPortfolioFundDescription, Option<PolymeshPrimitivesMemo>]>;
      /**
       * The portfolio has been successfully created.
       * 
       * # Parameters
       * * origin DID
       * * portfolio number
       * * portfolio name
       **/
      PortfolioCreated: AugmentedEvent<ApiType, [PolymeshPrimitivesIdentityId, u64, Bytes]>;
      /**
       * Custody of a portfolio has been given to a different identity
       * 
       * # Parameters
       * * origin DID
       * * portfolio id
       * * portfolio custodian did
       **/
      PortfolioCustodianChanged: AugmentedEvent<ApiType, [PolymeshPrimitivesIdentityId, PolymeshPrimitivesIdentityIdPortfolioId, PolymeshPrimitivesIdentityId]>;
      /**
       * The portfolio has been successfully removed.
       * 
       * # Parameters
       * * origin DID
       * * portfolio number
       **/
      PortfolioDeleted: AugmentedEvent<ApiType, [PolymeshPrimitivesIdentityId, u64]>;
      /**
       * The portfolio identified with `num` has been renamed to `name`.
       * 
       * # Parameters
       * * origin DID
       * * portfolio number
       * * portfolio name
       **/
      PortfolioRenamed: AugmentedEvent<ApiType, [PolymeshPrimitivesIdentityId, u64, Bytes]>;
      /**
       * A portfolio has pre approved the receivement of an asset.
       * 
       * # Parameters
       * * [`IdentityId`] of the caller.
       * * [`PortfolioId`] that will receive assets without explicit affirmation.
       * * [`AssetId`] of the asset that has been exempt from explicit affirmation.
       **/
      PreApprovedPortfolio: AugmentedEvent<ApiType, [PolymeshPrimitivesIdentityId, PolymeshPrimitivesIdentityIdPortfolioId, PolymeshPrimitivesAssetAssetId]>;
      /**
       * Revoke another identities permission to create portfolios.
       * 
       * # Parameters
       * * [`IdentityId`] of the caller.
       * * [`IdentityId`] permissions to create portfolios is revoked.
       **/
      RevokeCreatePortfoliosPermission: AugmentedEvent<ApiType, [PolymeshPrimitivesIdentityId, PolymeshPrimitivesIdentityId]>;
      /**
       * A portfolio has removed the approval of an asset.
       * 
       * # Parameters
       * * [`IdentityId`] of the caller.
       * * [`PortfolioId`] that had its pre approval revoked.
       * * [`AssetId`] of the asset that had its pre approval revoked.
       **/
      RevokePreApprovedPortfolio: AugmentedEvent<ApiType, [PolymeshPrimitivesIdentityId, PolymeshPrimitivesIdentityIdPortfolioId, PolymeshPrimitivesAssetAssetId]>;
      /**
       * All non-default portfolio numbers and names of a DID.
       * 
       * # Parameters
       * * origin DID
       * * vector of number-name pairs
       **/
      UserPortfolios: AugmentedEvent<ApiType, [PolymeshPrimitivesIdentityId, Vec<ITuple<[u64, Bytes]>>]>;
      /**
       * Generic event
       **/
      [key: string]: AugmentedEvent<ApiType>;
    };
    preimage: {
      /**
       * A preimage has ben cleared.
       **/
      Cleared: AugmentedEvent<ApiType, [hash_: H256], { hash_: H256 }>;
      /**
       * A preimage has been noted.
       **/
      Noted: AugmentedEvent<ApiType, [hash_: H256], { hash_: H256 }>;
      /**
       * A preimage has been requested.
       **/
      Requested: AugmentedEvent<ApiType, [hash_: H256], { hash_: H256 }>;
      /**
       * Generic event
       **/
      [key: string]: AugmentedEvent<ApiType>;
    };
    protocolFee: {
      /**
       * The fee coefficient.
       **/
      CoefficientSet: AugmentedEvent<ApiType, [PolymeshPrimitivesIdentityId, PolymeshPrimitivesPosRatio]>;
      /**
       * Fee charged.
       **/
      FeeCharged: AugmentedEvent<ApiType, [AccountId32, u128]>;
      /**
       * The protocol fee of an operation.
       **/
      FeeSet: AugmentedEvent<ApiType, [PolymeshPrimitivesIdentityId, u128]>;
      /**
       * Generic event
       **/
      [key: string]: AugmentedEvent<ApiType>;
    };
    relayer: {
      /**
       * Accepted subsidy.
       **/
      AcceptedSubsidy: AugmentedEvent<ApiType, [userKey: AccountId32, payingKey: AccountId32, initialPolyxLimit: u128], { userKey: AccountId32, payingKey: AccountId32, initialPolyxLimit: u128 }>;
      /**
       * A `paying_key` has approved subsidy for a `user_key`.
       **/
      ApprovedSubsidy: AugmentedEvent<ApiType, [userKey: AccountId32, payingKey: AccountId32, initialPolyxLimit: u128], { userKey: AccountId32, payingKey: AccountId32, initialPolyxLimit: u128 }>;
      /**
       * Relayed transaction.
       **/
      RelayedTx: AugmentedEvent<ApiType, [caller: AccountId32, target: AccountId32, result: Result<Null, SpRuntimeDispatchError>], { caller: AccountId32, target: AccountId32, result: Result<Null, SpRuntimeDispatchError> }>;
      /**
       * Removed pending subsidy.
       **/
      RemovedPendingSubsidy: AugmentedEvent<ApiType, [userKey: AccountId32, payingKey: AccountId32, initialPolyxLimit: u128], { userKey: AccountId32, payingKey: AccountId32, initialPolyxLimit: u128 }>;
      /**
       * Removed subsidy.
       **/
      RemovedSubsidy: AugmentedEvent<ApiType, [userKey: AccountId32, payingKey: AccountId32, remaining: u128], { userKey: AccountId32, payingKey: AccountId32, remaining: u128 }>;
      /**
       * The subsidy fee has been used to pay for a transaction or protocol fee.
       **/
      SubsidyDebited: AugmentedEvent<ApiType, [userKey: AccountId32, payingKey: AccountId32, amount: u128], { userKey: AccountId32, payingKey: AccountId32, amount: u128 }>;
      /**
       * Updated polyx limit.
       **/
      UpdatedPolyxLimit: AugmentedEvent<ApiType, [userKey: AccountId32, payingKey: AccountId32, remaining: u128, oldRemaining: u128], { userKey: AccountId32, payingKey: AccountId32, remaining: u128, oldRemaining: u128 }>;
      /**
       * Generic event
       **/
      [key: string]: AugmentedEvent<ApiType>;
    };
    scheduler: {
      /**
       * Agenda is incomplete from `when`.
       **/
      AgendaIncomplete: AugmentedEvent<ApiType, [when: u32], { when: u32 }>;
      /**
       * The call for the provided hash was not found so the task has been aborted.
       **/
      CallUnavailable: AugmentedEvent<ApiType, [task: ITuple<[u32, u32]>, id: Option<U8aFixed>], { task: ITuple<[u32, u32]>, id: Option<U8aFixed> }>;
      /**
       * Canceled some task.
       **/
      Canceled: AugmentedEvent<ApiType, [when: u32, index: u32], { when: u32, index: u32 }>;
      /**
       * Dispatched some task.
       **/
      Dispatched: AugmentedEvent<ApiType, [task: ITuple<[u32, u32]>, id: Option<U8aFixed>, result: Result<Null, SpRuntimeDispatchError>], { task: ITuple<[u32, u32]>, id: Option<U8aFixed>, result: Result<Null, SpRuntimeDispatchError> }>;
      /**
       * The given task was unable to be renewed since the agenda is full at that block.
       **/
      PeriodicFailed: AugmentedEvent<ApiType, [task: ITuple<[u32, u32]>, id: Option<U8aFixed>], { task: ITuple<[u32, u32]>, id: Option<U8aFixed> }>;
      /**
       * The given task can never be executed since it is overweight.
       **/
      PermanentlyOverweight: AugmentedEvent<ApiType, [task: ITuple<[u32, u32]>, id: Option<U8aFixed>], { task: ITuple<[u32, u32]>, id: Option<U8aFixed> }>;
      /**
       * Cancel a retry configuration for some task.
       **/
      RetryCancelled: AugmentedEvent<ApiType, [task: ITuple<[u32, u32]>, id: Option<U8aFixed>], { task: ITuple<[u32, u32]>, id: Option<U8aFixed> }>;
      /**
       * The given task was unable to be retried since the agenda is full at that block or there
       * was not enough weight to reschedule it.
       **/
      RetryFailed: AugmentedEvent<ApiType, [task: ITuple<[u32, u32]>, id: Option<U8aFixed>], { task: ITuple<[u32, u32]>, id: Option<U8aFixed> }>;
      /**
       * Set a retry configuration for some task.
       **/
      RetrySet: AugmentedEvent<ApiType, [task: ITuple<[u32, u32]>, id: Option<U8aFixed>, period: u32, retries: u8], { task: ITuple<[u32, u32]>, id: Option<U8aFixed>, period: u32, retries: u8 }>;
      /**
       * Scheduled some task.
       **/
      Scheduled: AugmentedEvent<ApiType, [when: u32, index: u32], { when: u32, index: u32 }>;
      /**
       * Generic event
       **/
      [key: string]: AugmentedEvent<ApiType>;
    };
    session: {
      /**
       * The `NewSession` event in the current block also implies a new validator set to be
       * queued.
       **/
      NewQueued: AugmentedEvent<ApiType, []>;
      /**
       * New session has happened. Note that the argument is the session index, not the
       * block number as the type might suggest.
       **/
      NewSession: AugmentedEvent<ApiType, [sessionIndex: u32], { sessionIndex: u32 }>;
      /**
       * Validator has been disabled.
       **/
      ValidatorDisabled: AugmentedEvent<ApiType, [validator: AccountId32], { validator: AccountId32 }>;
      /**
       * Validator has been re-enabled.
       **/
      ValidatorReenabled: AugmentedEvent<ApiType, [validator: AccountId32], { validator: AccountId32 }>;
      /**
       * Generic event
       **/
      [key: string]: AugmentedEvent<ApiType>;
    };
    settlement: {
      /**
       * An affirmation has been withdrawn (did, asset_holder, instruction_id)
       **/
      AffirmationWithdrawn: AugmentedEvent<ApiType, [PolymeshPrimitivesIdentityId, PolymeshPrimitivesAssetAssetHolder, u64]>;
      /**
       * Failed to execute instruction.
       **/
      FailedToExecuteInstruction: AugmentedEvent<ApiType, [u64, SpRuntimeDispatchError]>;
      /**
       * An instruction has been affirmed (did, asset_holder, instruction_id)
       **/
      InstructionAffirmed: AugmentedEvent<ApiType, [PolymeshPrimitivesIdentityId, PolymeshPrimitivesAssetAssetHolder, u64]>;
      /**
       * An instruction has been automatically affirmed.
       * Parameters: [`IdentityId`] of the caller, [`AssetHolder`] of the receiver, and [`InstructionId`] of the instruction.
       **/
      InstructionAutomaticallyAffirmed: AugmentedEvent<ApiType, [PolymeshPrimitivesIdentityId, PolymeshPrimitivesAssetAssetHolder, u64]>;
      /**
       * A new instruction has been created
       * (did, venue_id, instruction_id, settlement_type, trade_date, value_date, legs, memo)
       **/
      InstructionCreated: AugmentedEvent<ApiType, [PolymeshPrimitivesIdentityId, Option<u64>, u64, PolymeshPrimitivesSettlementSettlementType, Option<u64>, Option<u64>, Vec<PolymeshPrimitivesSettlementLeg>, Option<PolymeshPrimitivesMemo>]>;
      /**
       * Instruction executed successfully(did, instruction_id)
       **/
      InstructionExecuted: AugmentedEvent<ApiType, [PolymeshPrimitivesIdentityId, u64]>;
      /**
       * An instruction has been sucessfully locked for execution
       * 
       * Parameters:
       * - `IdentityId`: The [`IdentityId`] of the caller.
       * - `InstructionId`: The [`InstructionId`] of the instruction.
       **/
      InstructionLocked: AugmentedEvent<ApiType, [PolymeshPrimitivesIdentityId, u64]>;
      /**
       * An instruction with mediators has been created.
       * Parameters: [`InstructionId`] of the instruction and the [`IdentityId`] of all mediators.
       **/
      InstructionMediators: AugmentedEvent<ApiType, [u64, BTreeSet<PolymeshPrimitivesIdentityId>]>;
      /**
       * An instruction has been rejected (did, instruction_id)
       **/
      InstructionRejected: AugmentedEvent<ApiType, [PolymeshPrimitivesIdentityId, u64]>;
      /**
       * Instruction is rescheduled.
       * (caller DID, instruction_id)
       **/
      InstructionRescheduled: AugmentedEvent<ApiType, [PolymeshPrimitivesIdentityId, u64]>;
      /**
       * Execution of a leg failed (did, instruction_id, leg_id)
       **/
      LegFailedExecution: AugmentedEvent<ApiType, [PolymeshPrimitivesIdentityId, u64, u64]>;
      /**
       * An instruction has affirmed by a mediator.
       * Parameters: [`IdentityId`] of the mediator and [`InstructionId`] of the instruction.
       **/
      MediatorAffirmationReceived: AugmentedEvent<ApiType, [PolymeshPrimitivesIdentityId, u64, Option<u64>]>;
      /**
       * An instruction affirmation has been withdrawn by a mediator.
       * Parameters: [`IdentityId`] of the mediator and [`InstructionId`] of the instruction.
       **/
      MediatorAffirmationWithdrawn: AugmentedEvent<ApiType, [PolymeshPrimitivesIdentityId, u64]>;
      /**
       * A receipt has been claimed (did, instruction_id, leg_id, receipt_uid, signer, receipt metadata)
       **/
      ReceiptClaimed: AugmentedEvent<ApiType, [PolymeshPrimitivesIdentityId, u64, u64, u64, AccountId32, Option<PolymeshPrimitivesSettlementReceiptMetadata>]>;
      /**
       * Scheduling of instruction fails.
       **/
      SchedulingFailed: AugmentedEvent<ApiType, [u64, SpRuntimeDispatchError]>;
      /**
       * Settlement manually executed (did, id)
       **/
      SettlementManuallyExecuted: AugmentedEvent<ApiType, [PolymeshPrimitivesIdentityId, u64]>;
      /**
       * A new venue has been created (did, venue_id, details, type)
       **/
      VenueCreated: AugmentedEvent<ApiType, [PolymeshPrimitivesIdentityId, u64, Bytes, PolymeshPrimitivesSettlementVenueType]>;
      /**
       * An existing venue's details has been updated (did, venue_id, details)
       **/
      VenueDetailsUpdated: AugmentedEvent<ApiType, [PolymeshPrimitivesIdentityId, u64, Bytes]>;
      /**
       * Venue filtering has been enabled or disabled for an asset (did, AssetId, filtering_enabled)
       **/
      VenueFiltering: AugmentedEvent<ApiType, [PolymeshPrimitivesIdentityId, PolymeshPrimitivesAssetAssetId, bool]>;
      /**
       * Venues added to allow list (did, AssetId, vec<venue_id>)
       **/
      VenuesAllowed: AugmentedEvent<ApiType, [PolymeshPrimitivesIdentityId, PolymeshPrimitivesAssetAssetId, Vec<u64>]>;
      /**
       * Venues added to block list (did, AssetId, vec<venue_id>)
       **/
      VenuesBlocked: AugmentedEvent<ApiType, [PolymeshPrimitivesIdentityId, PolymeshPrimitivesAssetAssetId, Vec<u64>]>;
      /**
       * An existing venue's signers has been updated (did, venue_id, signers, update_type)
       **/
      VenueSignersUpdated: AugmentedEvent<ApiType, [PolymeshPrimitivesIdentityId, u64, Vec<AccountId32>, bool]>;
      /**
       * An existing venue's type has been updated (did, venue_id, type)
       **/
      VenueTypeUpdated: AugmentedEvent<ApiType, [PolymeshPrimitivesIdentityId, u64, PolymeshPrimitivesSettlementVenueType]>;
      /**
       * Venue not part of the token's allow list (did, AssetId, venue_id)
       **/
      VenueUnauthorized: AugmentedEvent<ApiType, [PolymeshPrimitivesIdentityId, PolymeshPrimitivesAssetAssetId, u64]>;
      /**
       * Generic event
       **/
      [key: string]: AugmentedEvent<ApiType>;
    };
    staking: {
      /**
       * An account has bonded this amount. \[stash, amount\]
       * 
       * NOTE: This event is only emitted when funds are bonded via a dispatchable. Notably,
       * it will not be emitted for staking rewards when they are added to stake.
       **/
      Bonded: AugmentedEvent<ApiType, [stash: AccountId32, amount: u128], { stash: AccountId32, amount: u128 }>;
      /**
       * An account has stopped participating as either a validator or nominator.
       **/
      Chilled: AugmentedEvent<ApiType, [stash: AccountId32], { stash: AccountId32 }>;
      /**
       * Report of a controller batch deprecation.
       **/
      ControllerBatchDeprecated: AugmentedEvent<ApiType, [failures: u32], { failures: u32 }>;
      /**
       * Staking balance migrated from locks to holds, with any balance that could not be held
       * is force withdrawn.
       **/
      CurrencyMigrated: AugmentedEvent<ApiType, [stash: AccountId32, forceWithdraw: u128], { stash: AccountId32, forceWithdraw: u128 }>;
      /**
       * The era payout has been set; the first balance is the validator-payout; the second is
       * the remainder from the maximum amount of reward.
       **/
      EraPaid: AugmentedEvent<ApiType, [eraIndex: u32, validatorPayout: u128, remainder: u128], { eraIndex: u32, validatorPayout: u128, remainder: u128 }>;
      /**
       * A new force era mode was set.
       **/
      ForceEra: AugmentedEvent<ApiType, [mode: PalletStakingForcing], { mode: PalletStakingForcing }>;
      /**
       * A nominator has been kicked from a validator.
       **/
      Kicked: AugmentedEvent<ApiType, [nominator: AccountId32, stash: AccountId32], { nominator: AccountId32, stash: AccountId32 }>;
      /**
       * An old slashing report from a prior era was discarded because it could
       * not be processed.
       **/
      OldSlashingReportDiscarded: AugmentedEvent<ApiType, [sessionIndex: u32], { sessionIndex: u32 }>;
      /**
       * A Page of stakers rewards are getting paid. `next` is `None` if all pages are claimed.
       **/
      PayoutStarted: AugmentedEvent<ApiType, [eraIndex: u32, validatorStash: AccountId32, page: u32, next: Option<u32>], { eraIndex: u32, validatorStash: AccountId32, page: u32, next: Option<u32> }>;
      /**
       * The nominator has been rewarded by this amount to this destination.
       **/
      Rewarded: AugmentedEvent<ApiType, [stash: AccountId32, dest: PalletStakingRewardDestination, amount: u128], { stash: AccountId32, dest: PalletStakingRewardDestination, amount: u128 }>;
      /**
       * A staker (validator or nominator) has been slashed by the given amount.
       **/
      Slashed: AugmentedEvent<ApiType, [staker: AccountId32, amount: u128], { staker: AccountId32, amount: u128 }>;
      /**
       * A slash for the given validator, for the given percentage of their stake, at the given
       * era as been reported.
       **/
      SlashReported: AugmentedEvent<ApiType, [validator: AccountId32, fraction: Perbill, slashEra: u32], { validator: AccountId32, fraction: Perbill, slashEra: u32 }>;
      /**
       * Targets size limit reached.
       **/
      SnapshotTargetsSizeExceeded: AugmentedEvent<ApiType, [size_: u32], { size_: u32 }>;
      /**
       * Voters size limit reached.
       **/
      SnapshotVotersSizeExceeded: AugmentedEvent<ApiType, [size_: u32], { size_: u32 }>;
      /**
       * A new set of stakers was elected.
       **/
      StakersElected: AugmentedEvent<ApiType, []>;
      /**
       * The election failed. No new era is planned.
       **/
      StakingElectionFailed: AugmentedEvent<ApiType, []>;
      /**
       * An account has unbonded this amount.
       **/
      Unbonded: AugmentedEvent<ApiType, [stash: AccountId32, amount: u128], { stash: AccountId32, amount: u128 }>;
      /**
       * A validator has set their preferences.
       **/
      ValidatorPrefsSet: AugmentedEvent<ApiType, [stash: AccountId32, prefs: PalletStakingValidatorPrefs], { stash: AccountId32, prefs: PalletStakingValidatorPrefs }>;
      /**
       * An account has called `withdraw_unbonded` and removed unbonding chunks worth `Balance`
       * from the unlocking queue.
       **/
      Withdrawn: AugmentedEvent<ApiType, [stash: AccountId32, amount: u128], { stash: AccountId32, amount: u128 }>;
      /**
       * Generic event
       **/
      [key: string]: AugmentedEvent<ApiType>;
    };
    statistics: {
      /**
       * Asset stats updated.
       * 
       * (Caller DID, AssetId, Stat type, Updates)
       **/
      AssetStatsUpdated: AugmentedEvent<ApiType, [PolymeshPrimitivesIdentityId, PolymeshPrimitivesAssetAssetId, PolymeshPrimitivesStatisticsStatType, Vec<PolymeshPrimitivesStatisticsStatUpdate>]>;
      /**
       * Set Transfer compliance rules for asset.
       * 
       * (Caller DID, AssetId, Transfer conditions)
       **/
      SetAssetTransferCompliance: AugmentedEvent<ApiType, [PolymeshPrimitivesIdentityId, PolymeshPrimitivesAssetAssetId, Vec<PolymeshPrimitivesTransferComplianceTransferCondition>]>;
      /**
       * Stat types added to asset.
       * 
       * (Caller DID, AssetId, Stat types)
       **/
      StatTypesAdded: AugmentedEvent<ApiType, [PolymeshPrimitivesIdentityId, PolymeshPrimitivesAssetAssetId, Vec<PolymeshPrimitivesStatisticsStatType>]>;
      /**
       * Stat types removed from asset.
       * 
       * (Caller DID, AssetId, Stat types)
       **/
      StatTypesRemoved: AugmentedEvent<ApiType, [PolymeshPrimitivesIdentityId, PolymeshPrimitivesAssetAssetId, Vec<PolymeshPrimitivesStatisticsStatType>]>;
      /**
       * Add `IdentityId`s exempt for transfer conditions matching exempt key.
       * 
       * (Caller DID, Exempt key, Entities)
       **/
      TransferConditionExemptionsAdded: AugmentedEvent<ApiType, [PolymeshPrimitivesIdentityId, PolymeshPrimitivesTransferComplianceTransferConditionExemptKey, Vec<PolymeshPrimitivesIdentityId>]>;
      /**
       * Remove `IdentityId`s exempt for transfer conditions matching exempt key.
       * 
       * (Caller DID, Exempt key, Entities)
       **/
      TransferConditionExemptionsRemoved: AugmentedEvent<ApiType, [PolymeshPrimitivesIdentityId, PolymeshPrimitivesTransferComplianceTransferConditionExemptKey, Vec<PolymeshPrimitivesIdentityId>]>;
      /**
       * Generic event
       **/
      [key: string]: AugmentedEvent<ApiType>;
    };
    sto: {
      /**
       * A fundraiser has been permanently closed.
       * 
       * [agent_did, offering_asset, fundraiser_id]
       **/
      FundraiserClosed: AugmentedEvent<ApiType, [agentDid: PolymeshPrimitivesIdentityId, offeringAsset: PolymeshPrimitivesAssetAssetId, fundraiserId: u64], { agentDid: PolymeshPrimitivesIdentityId, offeringAsset: PolymeshPrimitivesAssetAssetId, fundraiserId: u64 }>;
      /**
       * A new fundraiser has been created.
       * 
       * [agent_did, offering_asset, raising_asset, fundraiser_id, fundraiser_name, fundraiser]
       **/
      FundraiserCreated: AugmentedEvent<ApiType, [agentDid: PolymeshPrimitivesIdentityId, offeringAsset: PolymeshPrimitivesAssetAssetId, raisingAsset: PolymeshPrimitivesAssetAssetId, fundraiserId: u64, fundraiserName: Bytes, fundraiser: PalletStoFundraiser], { agentDid: PolymeshPrimitivesIdentityId, offeringAsset: PolymeshPrimitivesAssetAssetId, raisingAsset: PolymeshPrimitivesAssetAssetId, fundraiserId: u64, fundraiserName: Bytes, fundraiser: PalletStoFundraiser }>;
      /**
       * A fundraiser has been frozen, preventing new investments.
       * 
       * [agent_did, offering_asset, fundraiser_id]
       **/
      FundraiserFrozen: AugmentedEvent<ApiType, [agentDid: PolymeshPrimitivesIdentityId, offeringAsset: PolymeshPrimitivesAssetAssetId, fundraiserId: u64], { agentDid: PolymeshPrimitivesIdentityId, offeringAsset: PolymeshPrimitivesAssetAssetId, fundraiserId: u64 }>;
      /**
       * Off-chain funding has been enabled for a fundraiser.
       * 
       * [agent_did, offering_asset, fundraiser_id, ticker]
       **/
      FundraiserOffchainFundingEnabled: AugmentedEvent<ApiType, [agentDid: PolymeshPrimitivesIdentityId, offeringAsset: PolymeshPrimitivesAssetAssetId, fundraiserId: u64, ticker: PolymeshPrimitivesTicker], { agentDid: PolymeshPrimitivesIdentityId, offeringAsset: PolymeshPrimitivesAssetAssetId, fundraiserId: u64, ticker: PolymeshPrimitivesTicker }>;
      /**
       * A fundraiser has been unfrozen, allowing new investments.
       * 
       * [agent_did, offering_asset, fundraiser_id]
       **/
      FundraiserUnfrozen: AugmentedEvent<ApiType, [agentDid: PolymeshPrimitivesIdentityId, offeringAsset: PolymeshPrimitivesAssetAssetId, fundraiserId: u64], { agentDid: PolymeshPrimitivesIdentityId, offeringAsset: PolymeshPrimitivesAssetAssetId, fundraiserId: u64 }>;
      /**
       * A fundraiser's time window has been modified.
       * 
       * [agent_did, offering_asset, fundraiser_id, old_start, old_end, new_start, new_end]
       **/
      FundraiserWindowModified: AugmentedEvent<ApiType, [agentDid: PolymeshPrimitivesEventOnly, offeringAsset: PolymeshPrimitivesAssetAssetId, fundraiserId: u64, oldStart: u64, oldEnd: Option<u64>, newStart: u64, newEnd: Option<u64>], { agentDid: PolymeshPrimitivesEventOnly, offeringAsset: PolymeshPrimitivesAssetAssetId, fundraiserId: u64, oldStart: u64, oldEnd: Option<u64>, newStart: u64, newEnd: Option<u64> }>;
      /**
       * An investor successfully invested in the fundraiser.
       * 
       * [investor_did, offering_asset, fundraiser_id, funding_asset, offering_amount, raise_amount]
       **/
      Invested: AugmentedEvent<ApiType, [investorDid: PolymeshPrimitivesIdentityId, offeringAsset: PolymeshPrimitivesAssetAssetId, fundraiserId: u64, fundingAsset: PalletStoFundingAsset, offeringAmount: u128, raiseAmount: u128], { investorDid: PolymeshPrimitivesIdentityId, offeringAsset: PolymeshPrimitivesAssetAssetId, fundraiserId: u64, fundingAsset: PalletStoFundingAsset, offeringAmount: u128, raiseAmount: u128 }>;
      /**
       * Generic event
       **/
      [key: string]: AugmentedEvent<ApiType>;
    };
    sudo: {
      /**
       * The sudo key has been updated.
       **/
      KeyChanged: AugmentedEvent<ApiType, [old: Option<AccountId32>, new_: AccountId32], { old: Option<AccountId32>, new_: AccountId32 }>;
      /**
       * The key was permanently removed.
       **/
      KeyRemoved: AugmentedEvent<ApiType, []>;
      /**
       * A sudo call just took place.
       **/
      Sudid: AugmentedEvent<ApiType, [sudoResult: Result<Null, SpRuntimeDispatchError>], { sudoResult: Result<Null, SpRuntimeDispatchError> }>;
      /**
       * A [sudo_as](Pallet::sudo_as) call just took place.
       **/
      SudoAsDone: AugmentedEvent<ApiType, [sudoResult: Result<Null, SpRuntimeDispatchError>], { sudoResult: Result<Null, SpRuntimeDispatchError> }>;
      /**
       * Generic event
       **/
      [key: string]: AugmentedEvent<ApiType>;
    };
    system: {
      /**
       * `:code` was updated.
       **/
      CodeUpdated: AugmentedEvent<ApiType, []>;
      /**
       * An extrinsic failed.
       **/
      ExtrinsicFailed: AugmentedEvent<ApiType, [dispatchError: SpRuntimeDispatchError, dispatchInfo: FrameSystemDispatchEventInfo], { dispatchError: SpRuntimeDispatchError, dispatchInfo: FrameSystemDispatchEventInfo }>;
      /**
       * An extrinsic completed successfully.
       **/
      ExtrinsicSuccess: AugmentedEvent<ApiType, [dispatchInfo: FrameSystemDispatchEventInfo], { dispatchInfo: FrameSystemDispatchEventInfo }>;
      /**
       * An account was reaped.
       **/
      KilledAccount: AugmentedEvent<ApiType, [account: AccountId32], { account: AccountId32 }>;
      /**
       * A new account was created.
       **/
      NewAccount: AugmentedEvent<ApiType, [account: AccountId32], { account: AccountId32 }>;
      /**
       * An invalid authorized upgrade was rejected while trying to apply it.
       **/
      RejectedInvalidAuthorizedUpgrade: AugmentedEvent<ApiType, [codeHash: H256, error: SpRuntimeDispatchError], { codeHash: H256, error: SpRuntimeDispatchError }>;
      /**
       * On on-chain remark happened.
       **/
      Remarked: AugmentedEvent<ApiType, [sender: AccountId32, hash_: H256], { sender: AccountId32, hash_: H256 }>;
      /**
       * An upgrade was authorized.
       **/
      UpgradeAuthorized: AugmentedEvent<ApiType, [codeHash: H256, checkVersion: bool], { codeHash: H256, checkVersion: bool }>;
      /**
       * Generic event
       **/
      [key: string]: AugmentedEvent<ApiType>;
    };
    technicalCommittee: {
      /**
       * A motion was approved by the required threshold with the following
       * tally (yes votes, no votes and total seats given respectively as `MemberCount`).
       * Parameters: caller DID, proposal hash, yay vote count, nay vote count, total seats.
       **/
      Approved: AugmentedEvent<ApiType, [Option<PolymeshPrimitivesIdentityId>, H256, u32, u32, u32]>;
      /**
       * A motion was executed; `DispatchResult` is `Ok(())` if returned without error.
       * Parameters: caller DID, proposal hash, result of proposal dispatch.
       **/
      Executed: AugmentedEvent<ApiType, [Option<PolymeshPrimitivesIdentityId>, H256, Result<Null, SpRuntimeDispatchError>]>;
      /**
       * Proposal expiry time has been updated.
       * Parameters: caller DID, new expiry time (if any).
       **/
      ExpiresAfterUpdated: AugmentedEvent<ApiType, [PolymeshPrimitivesIdentityId, PolymeshPrimitivesMaybeBlock]>;
      /**
       * Final votes on a motion (given hash)
       * caller DID, ProposalIndex, Proposal hash, yes voters, no voter
       **/
      FinalVotes: AugmentedEvent<ApiType, [Option<PolymeshPrimitivesIdentityId>, u32, H256, Vec<PolymeshPrimitivesIdentityId>, Vec<PolymeshPrimitivesIdentityId>]>;
      /**
       * A motion (given hash) has been proposed (by given account) with a threshold (given `MemberCount`).
       * Parameters: caller DID, proposal index, proposal hash.
       **/
      Proposed: AugmentedEvent<ApiType, [PolymeshPrimitivesIdentityId, u32, H256]>;
      /**
       * A motion was rejected by the required threshold with the following
       * tally (yes votes, no votes and total seats given respectively as `MemberCount`).
       * Parameters: caller DID, proposal hash, yay vote count, nay vote count, total seats.
       **/
      Rejected: AugmentedEvent<ApiType, [Option<PolymeshPrimitivesIdentityId>, H256, u32, u32, u32]>;
      /**
       * Release coordinator has been updated.
       * Parameters: DID of the release coordinator.
       **/
      ReleaseCoordinatorUpdated: AugmentedEvent<ApiType, [Option<PolymeshPrimitivesIdentityId>]>;
      /**
       * A motion (given hash) has been voted on by given account, leaving
       * a tally (yes votes, no votes and total seats given respectively as `MemberCount`).
       * caller DID, Proposal index, Proposal hash, current vote, yay vote count, nay vote count, total seats.
       **/
      Voted: AugmentedEvent<ApiType, [PolymeshPrimitivesIdentityId, u32, H256, bool, u32, u32, u32]>;
      /**
       * A vote on a motion (given hash) has been retracted.
       * caller DID, ProposalIndex, Proposal hash, vote that was retracted
       **/
      VoteRetracted: AugmentedEvent<ApiType, [PolymeshPrimitivesIdentityId, u32, H256, bool]>;
      /**
       * Voting threshold has been updated
       * Parameters: caller DID, numerator, denominator
       **/
      VoteThresholdUpdated: AugmentedEvent<ApiType, [PolymeshPrimitivesIdentityId, u32, u32]>;
      /**
       * Generic event
       **/
      [key: string]: AugmentedEvent<ApiType>;
    };
    technicalCommitteeMembership: {
      /**
       * The limit of how many active members there can be concurrently was changed.
       **/
      ActiveLimitChanged: AugmentedEvent<ApiType, [PolymeshPrimitivesIdentityId, u32, u32]>;
      /**
       * The given member was added; see the transaction for who.
       * caller DID, New member DID.
       **/
      MemberAdded: AugmentedEvent<ApiType, [PolymeshPrimitivesIdentityId, PolymeshPrimitivesIdentityId]>;
      /**
       * The given member was removed; see the transaction for who.
       * caller DID, member DID that get removed.
       **/
      MemberRemoved: AugmentedEvent<ApiType, [PolymeshPrimitivesIdentityId, PolymeshPrimitivesIdentityId]>;
      /**
       * The given member has been revoked at specific time-stamp.
       * caller DID, member DID that get revoked.
       **/
      MemberRevoked: AugmentedEvent<ApiType, [PolymeshPrimitivesIdentityId, PolymeshPrimitivesIdentityId]>;
      /**
       * The membership was reset; see the transaction for who the new set is.
       * caller DID, List of new members.
       **/
      MembersReset: AugmentedEvent<ApiType, [PolymeshPrimitivesIdentityId, Vec<PolymeshPrimitivesIdentityId>]>;
      /**
       * Two members were swapped; see the transaction for who.
       * caller DID, Removed DID, New add DID.
       **/
      MembersSwapped: AugmentedEvent<ApiType, [PolymeshPrimitivesIdentityId, PolymeshPrimitivesIdentityId, PolymeshPrimitivesIdentityId]>;
      /**
       * Generic event
       **/
      [key: string]: AugmentedEvent<ApiType>;
    };
    transactionPayment: {
      /**
       * A transaction fee `actual_fee`, of which `tip` was added to the minimum inclusion fee,
       * has been paid by `who`.
       **/
      TransactionFeePaid: AugmentedEvent<ApiType, [who: AccountId32, actualFee: u128, tip: u128], { who: AccountId32, actualFee: u128, tip: u128 }>;
      /**
       * Generic event
       **/
      [key: string]: AugmentedEvent<ApiType>;
    };
    treasury: {
      /**
       * Disbursement to a target Identity.
       * 
       * (treasury identity, target identity, target primary key, amount)
       **/
      TreasuryDisbursement: AugmentedEvent<ApiType, [PolymeshPrimitivesIdentityId, PolymeshPrimitivesIdentityId, AccountId32, u128]>;
      /**
       * Disbursement to a target Identity failed.
       * 
       * (treasury identity, target identity, target primary key, amount)
       **/
      TreasuryDisbursementFailed: AugmentedEvent<ApiType, [PolymeshPrimitivesIdentityId, PolymeshPrimitivesIdentityId, AccountId32, u128]>;
      /**
       * Treasury reimbursement.
       * 
       * (source identity, amount)
       **/
      TreasuryReimbursement: AugmentedEvent<ApiType, [PolymeshPrimitivesIdentityId, u128]>;
      /**
       * Generic event
       **/
      [key: string]: AugmentedEvent<ApiType>;
    };
    upgradeCommittee: {
      /**
       * A motion was approved by the required threshold with the following
       * tally (yes votes, no votes and total seats given respectively as `MemberCount`).
       * Parameters: caller DID, proposal hash, yay vote count, nay vote count, total seats.
       **/
      Approved: AugmentedEvent<ApiType, [Option<PolymeshPrimitivesIdentityId>, H256, u32, u32, u32]>;
      /**
       * A motion was executed; `DispatchResult` is `Ok(())` if returned without error.
       * Parameters: caller DID, proposal hash, result of proposal dispatch.
       **/
      Executed: AugmentedEvent<ApiType, [Option<PolymeshPrimitivesIdentityId>, H256, Result<Null, SpRuntimeDispatchError>]>;
      /**
       * Proposal expiry time has been updated.
       * Parameters: caller DID, new expiry time (if any).
       **/
      ExpiresAfterUpdated: AugmentedEvent<ApiType, [PolymeshPrimitivesIdentityId, PolymeshPrimitivesMaybeBlock]>;
      /**
       * Final votes on a motion (given hash)
       * caller DID, ProposalIndex, Proposal hash, yes voters, no voter
       **/
      FinalVotes: AugmentedEvent<ApiType, [Option<PolymeshPrimitivesIdentityId>, u32, H256, Vec<PolymeshPrimitivesIdentityId>, Vec<PolymeshPrimitivesIdentityId>]>;
      /**
       * A motion (given hash) has been proposed (by given account) with a threshold (given `MemberCount`).
       * Parameters: caller DID, proposal index, proposal hash.
       **/
      Proposed: AugmentedEvent<ApiType, [PolymeshPrimitivesIdentityId, u32, H256]>;
      /**
       * A motion was rejected by the required threshold with the following
       * tally (yes votes, no votes and total seats given respectively as `MemberCount`).
       * Parameters: caller DID, proposal hash, yay vote count, nay vote count, total seats.
       **/
      Rejected: AugmentedEvent<ApiType, [Option<PolymeshPrimitivesIdentityId>, H256, u32, u32, u32]>;
      /**
       * Release coordinator has been updated.
       * Parameters: DID of the release coordinator.
       **/
      ReleaseCoordinatorUpdated: AugmentedEvent<ApiType, [Option<PolymeshPrimitivesIdentityId>]>;
      /**
       * A motion (given hash) has been voted on by given account, leaving
       * a tally (yes votes, no votes and total seats given respectively as `MemberCount`).
       * caller DID, Proposal index, Proposal hash, current vote, yay vote count, nay vote count, total seats.
       **/
      Voted: AugmentedEvent<ApiType, [PolymeshPrimitivesIdentityId, u32, H256, bool, u32, u32, u32]>;
      /**
       * A vote on a motion (given hash) has been retracted.
       * caller DID, ProposalIndex, Proposal hash, vote that was retracted
       **/
      VoteRetracted: AugmentedEvent<ApiType, [PolymeshPrimitivesIdentityId, u32, H256, bool]>;
      /**
       * Voting threshold has been updated
       * Parameters: caller DID, numerator, denominator
       **/
      VoteThresholdUpdated: AugmentedEvent<ApiType, [PolymeshPrimitivesIdentityId, u32, u32]>;
      /**
       * Generic event
       **/
      [key: string]: AugmentedEvent<ApiType>;
    };
    upgradeCommitteeMembership: {
      /**
       * The limit of how many active members there can be concurrently was changed.
       **/
      ActiveLimitChanged: AugmentedEvent<ApiType, [PolymeshPrimitivesIdentityId, u32, u32]>;
      /**
       * The given member was added; see the transaction for who.
       * caller DID, New member DID.
       **/
      MemberAdded: AugmentedEvent<ApiType, [PolymeshPrimitivesIdentityId, PolymeshPrimitivesIdentityId]>;
      /**
       * The given member was removed; see the transaction for who.
       * caller DID, member DID that get removed.
       **/
      MemberRemoved: AugmentedEvent<ApiType, [PolymeshPrimitivesIdentityId, PolymeshPrimitivesIdentityId]>;
      /**
       * The given member has been revoked at specific time-stamp.
       * caller DID, member DID that get revoked.
       **/
      MemberRevoked: AugmentedEvent<ApiType, [PolymeshPrimitivesIdentityId, PolymeshPrimitivesIdentityId]>;
      /**
       * The membership was reset; see the transaction for who the new set is.
       * caller DID, List of new members.
       **/
      MembersReset: AugmentedEvent<ApiType, [PolymeshPrimitivesIdentityId, Vec<PolymeshPrimitivesIdentityId>]>;
      /**
       * Two members were swapped; see the transaction for who.
       * caller DID, Removed DID, New add DID.
       **/
      MembersSwapped: AugmentedEvent<ApiType, [PolymeshPrimitivesIdentityId, PolymeshPrimitivesIdentityId, PolymeshPrimitivesIdentityId]>;
      /**
       * Generic event
       **/
      [key: string]: AugmentedEvent<ApiType>;
    };
    utility: {
      /**
       * Batch of dispatches completed fully with no error.
       **/
      BatchCompleted: AugmentedEvent<ApiType, []>;
      /**
       * Batch of dispatches completed but has errors.
       **/
      BatchCompletedWithErrors: AugmentedEvent<ApiType, []>;
      /**
       * Batch of dispatches did not complete fully. Index of first failing dispatch given, as
       * well as the error.
       **/
      BatchInterrupted: AugmentedEvent<ApiType, [index: u32, error: SpRuntimeDispatchError], { index: u32, error: SpRuntimeDispatchError }>;
      /**
       * A call was dispatched.
       **/
      DispatchedAs: AugmentedEvent<ApiType, [result: Result<Null, SpRuntimeDispatchError>], { result: Result<Null, SpRuntimeDispatchError> }>;
      /**
       * A single item within a Batch of dispatches has completed with no error.
       **/
      ItemCompleted: AugmentedEvent<ApiType, []>;
      /**
       * A single item within a Batch of dispatches has completed with error.
       **/
      ItemFailed: AugmentedEvent<ApiType, [error: SpRuntimeDispatchError], { error: SpRuntimeDispatchError }>;
      /**
       * Generic event
       **/
      [key: string]: AugmentedEvent<ApiType>;
    };
    validators: {
      /**
       * Commission cap has been updated.
       **/
      CommissionCapUpdated: AugmentedEvent<ApiType, [governanceCouncillDid: PolymeshPrimitivesIdentityId, oldCommissionCap: Perbill, newCommissionCap: Perbill], { governanceCouncillDid: PolymeshPrimitivesIdentityId, oldCommissionCap: Perbill, newCommissionCap: Perbill }>;
      /**
       * Remove the nominators from the valid nominators when there CDD expired.
       **/
      InvalidatedNominators: AugmentedEvent<ApiType, [governanceCouncillDid: PolymeshPrimitivesIdentityId, governanceCouncillAccount: PolymeshPrimitivesIdentityId, expiredNominators: Vec<AccountId32>], { governanceCouncillDid: PolymeshPrimitivesIdentityId, governanceCouncillAccount: PolymeshPrimitivesIdentityId, expiredNominators: Vec<AccountId32> }>;
      /**
       * User has updated their nominations.
       **/
      Nominated: AugmentedEvent<ApiType, [nominatorIdentity: PolymeshPrimitivesIdentityId, stash: AccountId32, targets: Vec<AccountId32>], { nominatorIdentity: PolymeshPrimitivesIdentityId, stash: AccountId32, targets: Vec<AccountId32> }>;
      /**
       * An identity has issued a candidacy for becoming a validator.
       **/
      PermissionedIdentityAdded: AugmentedEvent<ApiType, [governanceCouncillDid: PolymeshPrimitivesIdentityId, validatorsIdentity: PolymeshPrimitivesIdentityId], { governanceCouncillDid: PolymeshPrimitivesIdentityId, validatorsIdentity: PolymeshPrimitivesIdentityId }>;
      /**
       * An identity has been removed from the permissioned identities pool.
       **/
      PermissionedIdentityRemoved: AugmentedEvent<ApiType, [governanceCouncillDid: PolymeshPrimitivesIdentityId, validatorsIdentity: PolymeshPrimitivesIdentityId], { governanceCouncillDid: PolymeshPrimitivesIdentityId, validatorsIdentity: PolymeshPrimitivesIdentityId }>;
      /**
       * Reward scheduling interrupted.
       **/
      RewardPaymentSchedulingInterrupted: AugmentedEvent<ApiType, [accountId: AccountId32, era: u32, error: SpRuntimeDispatchError], { accountId: AccountId32, era: u32, error: SpRuntimeDispatchError }>;
      /**
       * Slashing allowed has been updated.
       **/
      SlashingAllowedForChanged: AugmentedEvent<ApiType, [slashingSwitch: PalletValidatorsSlashingSwitch], { slashingSwitch: PalletValidatorsSlashingSwitch }>;
      /**
       * Generic event
       **/
      [key: string]: AugmentedEvent<ApiType>;
    };
  } // AugmentedEvents
} // declare module
