// Auto-generated via `yarn polkadot-types-from-chain`, do not edit
/* eslint-disable */

import type { ApiTypes } from '@polkadot/api-base/types';
import type { Bytes, Null, Option, Result, U8aFixed, Vec, bool, u128, u32, u64, u8 } from '@polkadot/types-codec';
import type { ITuple } from '@polkadot/types-codec/types';
import type { AccountId32, H256, Perbill, Permill } from '@polkadot/types/interfaces/runtime';
import type { FrameSupportDispatchDispatchInfo, FrameSupportTokensMiscBalanceStatus, PalletBridgeBridgeTx, PalletBridgeHandledTxStatus, PalletCorporateActionsBallotBallotMeta, PalletCorporateActionsBallotBallotTimeRange, PalletCorporateActionsBallotBallotVote, PalletCorporateActionsCaId, PalletCorporateActionsCorporateAction, PalletCorporateActionsDistribution, PalletCorporateActionsTargetIdentities, PalletImOnlineSr25519AppSr25519Public, PalletPipsProposalData, PalletPipsProposalState, PalletPipsProposer, PalletPipsSnapshottedPip, PalletStakingElectionCompute, PalletStakingExposure, PalletStakingSlashingSwitch, PalletStoFundraiser, PolymeshCommonUtilitiesCheckpointScheduleCheckpoints, PolymeshCommonUtilitiesMaybeBlock, PolymeshPrimitivesAgentAgentGroup, PolymeshPrimitivesAssetAssetType, PolymeshPrimitivesAssetIdentifier, PolymeshPrimitivesAssetMetadataAssetMetadataKey, PolymeshPrimitivesAssetMetadataAssetMetadataSpec, PolymeshPrimitivesAssetMetadataAssetMetadataValueDetail, PolymeshPrimitivesAuthorizationAuthorizationData, PolymeshPrimitivesComplianceManagerComplianceRequirement, PolymeshPrimitivesConditionTrustedIssuer, PolymeshPrimitivesDocument, PolymeshPrimitivesEventOnly, PolymeshPrimitivesIdentityClaim, PolymeshPrimitivesIdentityId, PolymeshPrimitivesIdentityIdPortfolioId, PolymeshPrimitivesMemo, PolymeshPrimitivesNftNfTs, PolymeshPrimitivesPortfolioFundDescription, PolymeshPrimitivesPortfolioPortfolioUpdateReason, PolymeshPrimitivesPosRatio, PolymeshPrimitivesSecondaryKey, PolymeshPrimitivesSecondaryKeyPermissions, PolymeshPrimitivesSecondaryKeySignatory, PolymeshPrimitivesSettlementLeg, PolymeshPrimitivesSettlementReceiptMetadata, PolymeshPrimitivesSettlementSettlementType, PolymeshPrimitivesSettlementVenueType, PolymeshPrimitivesStatisticsAssetScope, PolymeshPrimitivesStatisticsStatType, PolymeshPrimitivesStatisticsStatUpdate, PolymeshPrimitivesSubsetSubsetRestrictionPalletPermissions, PolymeshPrimitivesTicker, PolymeshPrimitivesTransferComplianceTransferCondition, PolymeshPrimitivesTransferComplianceTransferConditionExemptKey, SpConsensusGrandpaAppPublic, SpRuntimeDispatchError } from '@polkadot/types/lookup';

declare module '@polkadot/api-base/types/events' {
  export interface AugmentedEvents<ApiType extends ApiTypes> {
    asset: {
      /**
       * Emitted when Tokens were issued, redeemed or transferred.
       * Contains the [`IdentityId`] of the receiver/issuer/redeemer, the [`Ticker`] for the token, the balance that was issued/transferred/redeemed,
       * the [`PortfolioId`] of the source, the [`PortfolioId`] of the destination and the [`PortfolioUpdateReason`].
       **/
      AssetBalanceUpdated: AugmentedEvent<ApiType, [PolymeshPrimitivesIdentityId, PolymeshPrimitivesTicker, u128, Option<PolymeshPrimitivesIdentityIdPortfolioId>, Option<PolymeshPrimitivesIdentityIdPortfolioId>, PolymeshPrimitivesPortfolioPortfolioUpdateReason]>;
      /**
       * Event for creation of the asset.
       * caller DID/ owner DID, ticker, divisibility, asset type, beneficiary DID, asset name, identifiers, funding round
       **/
      AssetCreated: AugmentedEvent<ApiType, [PolymeshPrimitivesIdentityId, PolymeshPrimitivesTicker, bool, PolymeshPrimitivesAssetAssetType, PolymeshPrimitivesIdentityId, Bytes, Vec<PolymeshPrimitivesAssetIdentifier>, Option<Bytes>]>;
      /**
       * An event emitted when an asset is frozen.
       * Parameter: caller DID, ticker.
       **/
      AssetFrozen: AugmentedEvent<ApiType, [PolymeshPrimitivesIdentityId, PolymeshPrimitivesTicker]>;
      /**
       * Emit when token ownership is transferred.
       * caller DID / token ownership transferred to DID, ticker, from
       **/
      AssetOwnershipTransferred: AugmentedEvent<ApiType, [PolymeshPrimitivesIdentityId, PolymeshPrimitivesTicker, PolymeshPrimitivesIdentityId]>;
      /**
       * An event emitted when a token is renamed.
       * Parameters: caller DID, ticker, new token name.
       **/
      AssetRenamed: AugmentedEvent<ApiType, [PolymeshPrimitivesIdentityId, PolymeshPrimitivesTicker, Bytes]>;
      /**
       * An event emitted when the type of an asset changed.
       * Parameters: caller DID, ticker, new token type.
       **/
      AssetTypeChanged: AugmentedEvent<ApiType, [PolymeshPrimitivesIdentityId, PolymeshPrimitivesTicker, PolymeshPrimitivesAssetAssetType]>;
      /**
       * An event emitted when an asset is unfrozen.
       * Parameter: caller DID, ticker.
       **/
      AssetUnfrozen: AugmentedEvent<ApiType, [PolymeshPrimitivesIdentityId, PolymeshPrimitivesTicker]>;
      /**
       * Event for when a forced transfer takes place.
       * caller DID/ controller DID, ticker, Portfolio of token holder, value.
       **/
      ControllerTransfer: AugmentedEvent<ApiType, [PolymeshPrimitivesIdentityId, PolymeshPrimitivesTicker, PolymeshPrimitivesIdentityIdPortfolioId, u128]>;
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
       * caller DID, ticker, divisibility
       **/
      DivisibilityChanged: AugmentedEvent<ApiType, [PolymeshPrimitivesIdentityId, PolymeshPrimitivesTicker, bool]>;
      /**
       * A new document attached to an asset
       **/
      DocumentAdded: AugmentedEvent<ApiType, [PolymeshPrimitivesIdentityId, PolymeshPrimitivesTicker, u32, PolymeshPrimitivesDocument]>;
      /**
       * A document removed from an asset
       **/
      DocumentRemoved: AugmentedEvent<ApiType, [PolymeshPrimitivesIdentityId, PolymeshPrimitivesTicker, u32]>;
      /**
       * A extension got removed.
       * caller DID, ticker, AccountId
       **/
      ExtensionRemoved: AugmentedEvent<ApiType, [PolymeshPrimitivesIdentityId, PolymeshPrimitivesTicker, AccountId32]>;
      /**
       * An event carrying the name of the current funding round of a ticker.
       * Parameters: caller DID, ticker, funding round name.
       **/
      FundingRoundSet: AugmentedEvent<ApiType, [PolymeshPrimitivesIdentityId, PolymeshPrimitivesTicker, Bytes]>;
      /**
       * Event emitted when any token identifiers are updated.
       * caller DID, ticker, a vector of (identifier type, identifier value)
       **/
      IdentifiersUpdated: AugmentedEvent<ApiType, [PolymeshPrimitivesIdentityId, PolymeshPrimitivesTicker, Vec<PolymeshPrimitivesAssetIdentifier>]>;
      /**
       * is_issuable() output
       * ticker, return value (true if issuable)
       **/
      IsIssuable: AugmentedEvent<ApiType, [PolymeshPrimitivesTicker, bool]>;
      /**
       * An event emitted when a local metadata key has been removed.
       * Parameters: caller ticker, Local type name
       **/
      LocalMetadataKeyDeleted: AugmentedEvent<ApiType, [PolymeshPrimitivesIdentityId, PolymeshPrimitivesTicker, u64]>;
      /**
       * An event emitted when a local metadata value has been removed.
       * Parameters: caller ticker, Local type name
       **/
      MetadataValueDeleted: AugmentedEvent<ApiType, [PolymeshPrimitivesIdentityId, PolymeshPrimitivesTicker, PolymeshPrimitivesAssetMetadataAssetMetadataKey]>;
      /**
       * Register asset metadata global type.
       * (Global type name, Global type key, type specs)
       **/
      RegisterAssetMetadataGlobalType: AugmentedEvent<ApiType, [Bytes, u64, PolymeshPrimitivesAssetMetadataAssetMetadataSpec]>;
      /**
       * Register asset metadata local type.
       * (Caller DID, ticker, Local type name, Local type key, type specs)
       **/
      RegisterAssetMetadataLocalType: AugmentedEvent<ApiType, [PolymeshPrimitivesIdentityId, PolymeshPrimitivesTicker, Bytes, u64, PolymeshPrimitivesAssetMetadataAssetMetadataSpec]>;
      /**
       * Set asset metadata value.
       * (Caller DID, ticker, metadata value, optional value details)
       **/
      SetAssetMetadataValue: AugmentedEvent<ApiType, [PolymeshPrimitivesIdentityId, PolymeshPrimitivesTicker, Bytes, Option<PolymeshPrimitivesAssetMetadataAssetMetadataValueDetail>]>;
      /**
       * Set asset metadata value details (expire, lock status).
       * (Caller DID, ticker, value details)
       **/
      SetAssetMetadataValueDetails: AugmentedEvent<ApiType, [PolymeshPrimitivesIdentityId, PolymeshPrimitivesTicker, PolymeshPrimitivesAssetMetadataAssetMetadataValueDetail]>;
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
       * An additional event to Transfer; emitted when `transfer_with_data` is called.
       * caller DID , ticker, from DID, to DID, value, data
       **/
      TransferWithData: AugmentedEvent<ApiType, [PolymeshPrimitivesIdentityId, PolymeshPrimitivesTicker, PolymeshPrimitivesIdentityId, PolymeshPrimitivesIdentityId, u128, Bytes]>;
      /**
       * Generic event
       **/
      [key: string]: AugmentedEvent<ApiType>;
    };
    balances: {
      /**
       * The account and the amount of unlocked balance of that account that was burned.
       * (caller Id, caller account, amount)
       **/
      AccountBalanceBurned: AugmentedEvent<ApiType, [PolymeshPrimitivesIdentityId, AccountId32, u128]>;
      /**
       * A balance was set by root (did, who, free, reserved).
       **/
      BalanceSet: AugmentedEvent<ApiType, [PolymeshPrimitivesIdentityId, AccountId32, u128, u128]>;
      /**
       * An account was created with some free balance. \[did, account, free_balance]
       **/
      Endowed: AugmentedEvent<ApiType, [Option<PolymeshPrimitivesIdentityId>, AccountId32, u128]>;
      /**
       * Some balance was reserved (moved from free to reserved). \[who, value]
       **/
      Reserved: AugmentedEvent<ApiType, [AccountId32, u128]>;
      /**
       * Some balance was moved from the reserve of the first account to the second account.
       * Final argument indicates the destination balance type.
       * \[from, to, balance, destination_status]
       **/
      ReserveRepatriated: AugmentedEvent<ApiType, [AccountId32, AccountId32, u128, FrameSupportTokensMiscBalanceStatus]>;
      /**
       * Transfer succeeded (from_did, from, to_did, to, value, memo).
       **/
      Transfer: AugmentedEvent<ApiType, [Option<PolymeshPrimitivesIdentityId>, AccountId32, Option<PolymeshPrimitivesIdentityId>, AccountId32, u128, Option<PolymeshPrimitivesMemo>]>;
      /**
       * Some balance was unreserved (moved from reserved to free). \[who, value]
       **/
      Unreserved: AugmentedEvent<ApiType, [AccountId32, u128]>;
      /**
       * Generic event
       **/
      [key: string]: AugmentedEvent<ApiType>;
    };
    base: {
      /**
       * An unexpected error happened that should be investigated.
       **/
      UnexpectedError: AugmentedEvent<ApiType, [Option<SpRuntimeDispatchError>]>;
      /**
       * Generic event
       **/
      [key: string]: AugmentedEvent<ApiType>;
    };
    bridge: {
      /**
       * Confirmation of Admin change.
       **/
      AdminChanged: AugmentedEvent<ApiType, [PolymeshPrimitivesIdentityId, AccountId32]>;
      /**
       * Confirmation of POLYX upgrade on Polymesh from POLY tokens on Ethereum.
       **/
      Bridged: AugmentedEvent<ApiType, [PolymeshPrimitivesIdentityId, PalletBridgeBridgeTx]>;
      /**
       * Bridge limit has been updated.
       **/
      BridgeLimitUpdated: AugmentedEvent<ApiType, [PolymeshPrimitivesIdentityId, u128, u32]>;
      /**
       * Bridge Tx failed.  Recipient missing CDD or limit reached.
       **/
      BridgeTxFailed: AugmentedEvent<ApiType, [PolymeshPrimitivesIdentityId, PalletBridgeBridgeTx, SpRuntimeDispatchError]>;
      /**
       * Bridge Tx Scheduled.
       **/
      BridgeTxScheduled: AugmentedEvent<ApiType, [PolymeshPrimitivesIdentityId, PalletBridgeBridgeTx, u32]>;
      /**
       * Failed to schedule Bridge Tx.
       **/
      BridgeTxScheduleFailed: AugmentedEvent<ApiType, [PolymeshPrimitivesIdentityId, PalletBridgeBridgeTx, Bytes]>;
      /**
       * Confirmation of a signer set change.
       **/
      ControllerChanged: AugmentedEvent<ApiType, [PolymeshPrimitivesIdentityId, AccountId32]>;
      /**
       * Exemption status of an identity has been updated.
       **/
      ExemptedUpdated: AugmentedEvent<ApiType, [PolymeshPrimitivesIdentityId, PolymeshPrimitivesIdentityId, bool]>;
      /**
       * A new freeze admin has been added.
       **/
      FreezeAdminAdded: AugmentedEvent<ApiType, [PolymeshPrimitivesIdentityId, AccountId32]>;
      /**
       * A freeze admin has been removed.
       **/
      FreezeAdminRemoved: AugmentedEvent<ApiType, [PolymeshPrimitivesIdentityId, AccountId32]>;
      /**
       * Notification of freezing the bridge.
       **/
      Frozen: AugmentedEvent<ApiType, [PolymeshPrimitivesIdentityId]>;
      /**
       * Notification of freezing a transaction.
       **/
      FrozenTx: AugmentedEvent<ApiType, [PolymeshPrimitivesIdentityId, PalletBridgeBridgeTx]>;
      /**
       * Confirmation of default timelock change.
       **/
      TimelockChanged: AugmentedEvent<ApiType, [PolymeshPrimitivesIdentityId, u32]>;
      /**
       * Notification of removing a transaction.
       **/
      TxRemoved: AugmentedEvent<ApiType, [PolymeshPrimitivesIdentityId, PalletBridgeBridgeTx]>;
      /**
       * An event emitted after a vector of transactions is handled. The parameter is a vector of
       * tuples of recipient account, its nonce, and the status of the processed transaction.
       **/
      TxsHandled: AugmentedEvent<ApiType, [Vec<ITuple<[AccountId32, u32, PalletBridgeHandledTxStatus]>>]>;
      /**
       * Notification of unfreezing the bridge.
       **/
      Unfrozen: AugmentedEvent<ApiType, [PolymeshPrimitivesIdentityId]>;
      /**
       * Notification of unfreezing a transaction.
       **/
      UnfrozenTx: AugmentedEvent<ApiType, [PolymeshPrimitivesIdentityId, PalletBridgeBridgeTx]>;
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
    cddServiceProviders: {
      /**
       * The limit of how many active members there can be concurrently was changed.
       **/
      ActiveLimitChanged: AugmentedEvent<ApiType, [PolymeshPrimitivesIdentityId, u32, u32]>;
      /**
       * Phantom member, never used.
       **/
      Dummy: AugmentedEvent<ApiType, []>;
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
    checkpoint: {
      /**
       * A checkpoint was created.
       * 
       * (caller DID, ticker, checkpoint ID, total supply, checkpoint timestamp)
       **/
      CheckpointCreated: AugmentedEvent<ApiType, [Option<PolymeshPrimitivesIdentityId>, PolymeshPrimitivesTicker, u64, u128, u64]>;
      /**
       * The maximum complexity for an arbitrary ticker's schedule set was changed.
       * 
       * (GC DID, the new maximum)
       **/
      MaximumSchedulesComplexityChanged: AugmentedEvent<ApiType, [PolymeshPrimitivesIdentityId, u64]>;
      /**
       * A checkpoint schedule was created.
       * 
       * (caller DID, ticker, schedule id, schedule)
       **/
      ScheduleCreated: AugmentedEvent<ApiType, [PolymeshPrimitivesIdentityId, PolymeshPrimitivesTicker, u64, PolymeshCommonUtilitiesCheckpointScheduleCheckpoints]>;
      /**
       * A checkpoint schedule was removed.
       * 
       * (caller DID, ticker, schedule id, schedule)
       **/
      ScheduleRemoved: AugmentedEvent<ApiType, [PolymeshPrimitivesIdentityId, PolymeshPrimitivesTicker, u64, PolymeshCommonUtilitiesCheckpointScheduleCheckpoints]>;
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
       * Phantom member, never used.
       **/
      Dummy: AugmentedEvent<ApiType, []>;
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
       * Emitted when an asset compliance for a given ticker gets paused.
       * (caller DID, Ticker).
       **/
      AssetCompliancePaused: AugmentedEvent<ApiType, [PolymeshPrimitivesIdentityId, PolymeshPrimitivesTicker]>;
      /**
       * Emitted when an asset compliance is replaced.
       * Parameters: caller DID, ticker, new asset compliance.
       **/
      AssetComplianceReplaced: AugmentedEvent<ApiType, [PolymeshPrimitivesIdentityId, PolymeshPrimitivesTicker, Vec<PolymeshPrimitivesComplianceManagerComplianceRequirement>]>;
      /**
       * Emitted when an asset compliance of a ticker is reset.
       * (caller DID, Ticker).
       **/
      AssetComplianceReset: AugmentedEvent<ApiType, [PolymeshPrimitivesIdentityId, PolymeshPrimitivesTicker]>;
      /**
       * Emitted when an asset compliance for a given ticker gets resume.
       * (caller DID, Ticker).
       **/
      AssetComplianceResumed: AugmentedEvent<ApiType, [PolymeshPrimitivesIdentityId, PolymeshPrimitivesTicker]>;
      /**
       * Emitted when compliance requirement get modified/change.
       * (caller DID, Ticker, ComplianceRequirement).
       **/
      ComplianceRequirementChanged: AugmentedEvent<ApiType, [PolymeshPrimitivesIdentityId, PolymeshPrimitivesTicker, PolymeshPrimitivesComplianceManagerComplianceRequirement]>;
      /**
       * Emitted when new compliance requirement is created.
       * (caller DID, Ticker, ComplianceRequirement).
       **/
      ComplianceRequirementCreated: AugmentedEvent<ApiType, [PolymeshPrimitivesIdentityId, PolymeshPrimitivesTicker, PolymeshPrimitivesComplianceManagerComplianceRequirement]>;
      /**
       * Emitted when a compliance requirement is removed.
       * (caller DID, Ticker, requirement_id).
       **/
      ComplianceRequirementRemoved: AugmentedEvent<ApiType, [PolymeshPrimitivesIdentityId, PolymeshPrimitivesTicker, u32]>;
      /**
       * Emitted when default claim issuer list for a given ticker gets added.
       * (caller DID, Ticker, Added TrustedIssuer).
       **/
      TrustedDefaultClaimIssuerAdded: AugmentedEvent<ApiType, [PolymeshPrimitivesIdentityId, PolymeshPrimitivesTicker, PolymeshPrimitivesConditionTrustedIssuer]>;
      /**
       * Emitted when default claim issuer list for a given ticker get removed.
       * (caller DID, Ticker, Removed TrustedIssuer).
       **/
      TrustedDefaultClaimIssuerRemoved: AugmentedEvent<ApiType, [PolymeshPrimitivesIdentityId, PolymeshPrimitivesTicker, PolymeshPrimitivesIdentityId]>;
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
      Called: AugmentedEvent<ApiType, [AccountId32, AccountId32]>;
      /**
       * A code with the specified hash was removed.
       **/
      CodeRemoved: AugmentedEvent<ApiType, [H256]>;
      /**
       * Code with the specified hash has been stored.
       **/
      CodeStored: AugmentedEvent<ApiType, [H256]>;
      /**
       * A contract's code was updated.
       **/
      ContractCodeUpdated: AugmentedEvent<ApiType, [AccountId32, H256, H256]>;
      /**
       * A custom event emitted by the contract.
       **/
      ContractEmitted: AugmentedEvent<ApiType, [AccountId32, Bytes]>;
      /**
       * A contract delegate called a code hash.
       * 
       * # Note
       * 
       * Please keep in mind that like all events this is only emitted for successful
       * calls. This is because on failure all storage changes including events are
       * rolled back.
       **/
      DelegateCalled: AugmentedEvent<ApiType, [AccountId32, H256]>;
      /**
       * Contract deployed by address at the specified address.
       **/
      Instantiated: AugmentedEvent<ApiType, [AccountId32, AccountId32]>;
      /**
       * Contract has been removed.
       * 
       * # Note
       * 
       * The only way for a contract to be removed and emitting this event is by calling
       * `seal_terminate`.
       **/
      Terminated: AugmentedEvent<ApiType, [AccountId32, AccountId32]>;
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
       * The set of default `TargetIdentities` for a ticker changed.
       * (Agent DID, Ticker, New TargetIdentities)
       **/
      DefaultTargetIdentitiesChanged: AugmentedEvent<ApiType, [PolymeshPrimitivesIdentityId, PolymeshPrimitivesTicker, PalletCorporateActionsTargetIdentities]>;
      /**
       * The default withholding tax for a ticker changed.
       * (Agent DID, Ticker, New Tax).
       **/
      DefaultWithholdingTaxChanged: AugmentedEvent<ApiType, [PolymeshPrimitivesIdentityId, PolymeshPrimitivesTicker, Permill]>;
      /**
       * The withholding tax specific to a DID for a ticker changed.
       * (Agent DID, Ticker, Taxed DID, New Tax).
       **/
      DidWithholdingTaxChanged: AugmentedEvent<ApiType, [PolymeshPrimitivesIdentityId, PolymeshPrimitivesTicker, PolymeshPrimitivesIdentityId, Option<Permill>]>;
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
    externalAgents: {
      /**
       * An agent was added.
       * 
       * (Caller/Agent DID, Agent's ticker, Agent's group)
       **/
      AgentAdded: AugmentedEvent<ApiType, [PolymeshPrimitivesEventOnly, PolymeshPrimitivesTicker, PolymeshPrimitivesAgentAgentGroup]>;
      /**
       * An agent was removed.
       * 
       * (Caller DID, Agent's ticker, Agent's DID)
       **/
      AgentRemoved: AugmentedEvent<ApiType, [PolymeshPrimitivesEventOnly, PolymeshPrimitivesTicker, PolymeshPrimitivesIdentityId]>;
      /**
       * An agent's group was changed.
       * 
       * (Caller DID, Agent's ticker, Agent's DID, The new group of the agent)
       **/
      GroupChanged: AugmentedEvent<ApiType, [PolymeshPrimitivesEventOnly, PolymeshPrimitivesTicker, PolymeshPrimitivesIdentityId, PolymeshPrimitivesAgentAgentGroup]>;
      /**
       * An Agent Group was created.
       * 
       * (Caller DID, AG's ticker, AG's ID, AG's permissions)
       **/
      GroupCreated: AugmentedEvent<ApiType, [PolymeshPrimitivesEventOnly, PolymeshPrimitivesTicker, u32, PolymeshPrimitivesSubsetSubsetRestrictionPalletPermissions]>;
      /**
       * An Agent Group's permissions was updated.
       * 
       * (Caller DID, AG's ticker, AG's ID, AG's new permissions)
       **/
      GroupPermissionsUpdated: AugmentedEvent<ApiType, [PolymeshPrimitivesEventOnly, PolymeshPrimitivesTicker, u32, PolymeshPrimitivesSubsetSubsetRestrictionPalletPermissions]>;
      /**
       * Generic event
       **/
      [key: string]: AugmentedEvent<ApiType>;
    };
    grandpa: {
      /**
       * New authority set has been applied.
       **/
      NewAuthorities: AugmentedEvent<ApiType, [Vec<ITuple<[SpConsensusGrandpaAppPublic, u64]>>]>;
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
    identity: {
      /**
       * Asset's identity registered.
       * 
       * (Asset DID, ticker)
       **/
      AssetDidRegistered: AugmentedEvent<ApiType, [PolymeshPrimitivesIdentityId, PolymeshPrimitivesTicker]>;
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
       * CDD claims generated by `IdentityId` (a CDD Provider) have been invalidated from
       * `Moment`.
       * 
       * (CDD provider DID, disable from date)
       **/
      CddClaimsInvalidated: AugmentedEvent<ApiType, [PolymeshPrimitivesIdentityId, u64]>;
      /**
       * CDD requirement for updating primary key changed.
       * 
       * (new_requirement)
       **/
      CddRequirementForPrimaryKeyUpdated: AugmentedEvent<ApiType, [bool]>;
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
      HeartbeatReceived: AugmentedEvent<ApiType, [PalletImOnlineSr25519AppSr25519Public]>;
      /**
       * At the end of the session, at least one validator was found to be offline.
       **/
      SomeOffline: AugmentedEvent<ApiType, [Vec<ITuple<[AccountId32, PalletStakingExposure]>>]>;
      /**
       * Generic event
       **/
      [key: string]: AugmentedEvent<ApiType>;
    };
    indices: {
      /**
       * A account index was assigned.
       **/
      IndexAssigned: AugmentedEvent<ApiType, [AccountId32, u32]>;
      /**
       * A account index has been freed up (unassigned).
       **/
      IndexFreed: AugmentedEvent<ApiType, [u32]>;
      /**
       * A account index has been frozen to its current account ID.
       **/
      IndexFrozen: AugmentedEvent<ApiType, [u32, AccountId32]>;
      /**
       * Generic event
       **/
      [key: string]: AugmentedEvent<ApiType>;
    };
    multiSig: {
      /**
       * Event emitted after creation of a multisig.
       * Arguments: caller DID, multisig address, signers (pending approval), signatures required.
       **/
      MultiSigCreated: AugmentedEvent<ApiType, [PolymeshPrimitivesIdentityId, AccountId32, AccountId32, Vec<PolymeshPrimitivesSecondaryKeySignatory>, u64]>;
      /**
       * Event emitted when the number of required signatures is changed.
       * Arguments: caller DID, multisig, new required signatures.
       **/
      MultiSigSignaturesRequiredChanged: AugmentedEvent<ApiType, [PolymeshPrimitivesIdentityId, AccountId32, u64]>;
      /**
       * Event emitted when a signatory is added.
       * Arguments: caller DID, multisig, added signer.
       **/
      MultiSigSignerAdded: AugmentedEvent<ApiType, [PolymeshPrimitivesIdentityId, AccountId32, PolymeshPrimitivesSecondaryKeySignatory]>;
      /**
       * Event emitted when a multisig signatory is authorized to be added.
       * Arguments: caller DID, multisig, authorized signer.
       **/
      MultiSigSignerAuthorized: AugmentedEvent<ApiType, [PolymeshPrimitivesIdentityId, AccountId32, PolymeshPrimitivesSecondaryKeySignatory]>;
      /**
       * Event emitted when a multisig signatory is removed.
       * Arguments: caller DID, multisig, removed signer.
       **/
      MultiSigSignerRemoved: AugmentedEvent<ApiType, [PolymeshPrimitivesIdentityId, AccountId32, PolymeshPrimitivesSecondaryKeySignatory]>;
      /**
       * Event emitted after adding a proposal.
       * Arguments: caller DID, multisig, proposal ID.
       **/
      ProposalAdded: AugmentedEvent<ApiType, [PolymeshPrimitivesIdentityId, AccountId32, u64]>;
      /**
       * Event emitted when the proposal get approved.
       * Arguments: caller DID, multisig, authorized signer, proposal id.
       **/
      ProposalApproved: AugmentedEvent<ApiType, [PolymeshPrimitivesIdentityId, AccountId32, PolymeshPrimitivesSecondaryKeySignatory, u64]>;
      /**
       * Event emitted when a proposal is executed.
       * Arguments: caller DID, multisig, proposal ID, result.
       **/
      ProposalExecuted: AugmentedEvent<ApiType, [PolymeshPrimitivesIdentityId, AccountId32, u64, bool]>;
      /**
       * Event emitted when there's an error in proposal execution
       **/
      ProposalExecutionFailed: AugmentedEvent<ApiType, [SpRuntimeDispatchError]>;
      /**
       * Event emitted when a proposal is rejected.
       * Arguments: caller DID, multisig, proposal ID.
       **/
      ProposalRejected: AugmentedEvent<ApiType, [PolymeshPrimitivesIdentityId, AccountId32, u64]>;
      /**
       * Event emitted when a vote is cast in favor of rejecting a proposal.
       * Arguments: caller DID, multisig, authorized signer, proposal id.
       **/
      ProposalRejectionVote: AugmentedEvent<ApiType, [PolymeshPrimitivesIdentityId, AccountId32, PolymeshPrimitivesSecondaryKeySignatory, u64]>;
      /**
       * Scheduling of proposal fails.
       **/
      SchedulingFailed: AugmentedEvent<ApiType, [SpRuntimeDispatchError]>;
      /**
       * Generic event
       **/
      [key: string]: AugmentedEvent<ApiType>;
    };
    nft: {
      /**
       * Emitted when a new nft collection is created.
       **/
      NftCollectionCreated: AugmentedEvent<ApiType, [PolymeshPrimitivesIdentityId, PolymeshPrimitivesTicker, u64]>;
      /**
       * Emitted when NFTs were issued, redeemed or transferred.
       * Contains the [`IdentityId`] of the receiver/issuer/redeemer, the [`NFTs`], the [`PortfolioId`] of the source, the [`PortfolioId`]
       * of the destination and the [`PortfolioUpdateReason`].
       **/
      NFTPortfolioUpdated: AugmentedEvent<ApiType, [PolymeshPrimitivesIdentityId, PolymeshPrimitivesNftNfTs, Option<PolymeshPrimitivesIdentityIdPortfolioId>, Option<PolymeshPrimitivesIdentityIdPortfolioId>, PolymeshPrimitivesPortfolioPortfolioUpdateReason]>;
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
      Offence: AugmentedEvent<ApiType, [U8aFixed, Bytes]>;
      /**
       * Generic event
       **/
      [key: string]: AugmentedEvent<ApiType>;
    };
    pips: {
      /**
       * The maximum number of active PIPs was changed.
       * (caller DID, old value, new value)
       **/
      ActivePipLimitChanged: AugmentedEvent<ApiType, [PolymeshPrimitivesIdentityId, u32, u32]>;
      /**
       * Default enactment period (in blocks) has been changed.
       * (caller DID, old period, new period)
       **/
      DefaultEnactmentPeriodChanged: AugmentedEvent<ApiType, [PolymeshPrimitivesIdentityId, u32, u32]>;
      /**
       * Cancelling the PIP execution failed in the scheduler pallet.
       **/
      ExecutionCancellingFailed: AugmentedEvent<ApiType, [u32]>;
      /**
       * Execution of a PIP has been scheduled at specific block.
       **/
      ExecutionScheduled: AugmentedEvent<ApiType, [PolymeshPrimitivesIdentityId, u32, u32]>;
      /**
       * Scheduling of the PIP for execution failed in the scheduler pallet.
       **/
      ExecutionSchedulingFailed: AugmentedEvent<ApiType, [PolymeshPrimitivesIdentityId, u32, u32]>;
      /**
       * The PIP has been scheduled for expiry.
       **/
      ExpiryScheduled: AugmentedEvent<ApiType, [PolymeshPrimitivesIdentityId, u32, u32]>;
      /**
       * Scheduling of the PIP for expiry failed in the scheduler pallet.
       **/
      ExpirySchedulingFailed: AugmentedEvent<ApiType, [PolymeshPrimitivesIdentityId, u32, u32]>;
      /**
       * Pruning Historical PIPs is enabled or disabled (caller DID, old value, new value)
       **/
      HistoricalPipsPruned: AugmentedEvent<ApiType, [PolymeshPrimitivesIdentityId, bool, bool]>;
      /**
       * The maximum times a PIP can be skipped was changed.
       * (caller DID, old value, new value)
       **/
      MaxPipSkipCountChanged: AugmentedEvent<ApiType, [PolymeshPrimitivesIdentityId, u8, u8]>;
      /**
       * Minimum deposit amount modified
       * (caller DID, old amount, new amount)
       **/
      MinimumProposalDepositChanged: AugmentedEvent<ApiType, [PolymeshPrimitivesIdentityId, u128, u128]>;
      /**
       * Amount of blocks after which a pending PIP expires.
       * (caller DID, old expiry, new expiry)
       **/
      PendingPipExpiryChanged: AugmentedEvent<ApiType, [PolymeshPrimitivesIdentityId, PolymeshCommonUtilitiesMaybeBlock, PolymeshCommonUtilitiesMaybeBlock]>;
      /**
       * Pip has been closed, bool indicates whether data is pruned
       **/
      PipClosed: AugmentedEvent<ApiType, [PolymeshPrimitivesIdentityId, u32, bool]>;
      /**
       * A PIP in the snapshot queue was skipped.
       * (gc_did, pip_id, new_skip_count)
       **/
      PipSkipped: AugmentedEvent<ApiType, [PolymeshPrimitivesIdentityId, u32, u8]>;
      /**
       * A PIP was made with a `Balance` stake.
       * 
       * # Parameters:
       * 
       * Caller DID, Proposer, PIP ID, deposit, URL, description, expiry time, proposal data.
       **/
      ProposalCreated: AugmentedEvent<ApiType, [PolymeshPrimitivesIdentityId, PalletPipsProposer, u32, u128, Option<Bytes>, Option<Bytes>, PolymeshCommonUtilitiesMaybeBlock, PalletPipsProposalData]>;
      /**
       * Refund proposal
       * (id, total amount)
       **/
      ProposalRefund: AugmentedEvent<ApiType, [PolymeshPrimitivesIdentityId, u32, u128]>;
      /**
       * Triggered each time the state of a proposal is amended
       **/
      ProposalStateUpdated: AugmentedEvent<ApiType, [PolymeshPrimitivesIdentityId, u32, PalletPipsProposalState]>;
      /**
       * The snapshot was cleared.
       **/
      SnapshotCleared: AugmentedEvent<ApiType, [PolymeshPrimitivesIdentityId, u32]>;
      /**
       * Results (e.g., approved, rejected, and skipped), were enacted for some PIPs.
       * (gc_did, snapshot_id_opt, skipped_pips_with_new_count, rejected_pips, approved_pips)
       **/
      SnapshotResultsEnacted: AugmentedEvent<ApiType, [PolymeshPrimitivesIdentityId, Option<u32>, Vec<ITuple<[u32, u8]>>, Vec<u32>, Vec<u32>]>;
      /**
       * A new snapshot was taken.
       **/
      SnapshotTaken: AugmentedEvent<ApiType, [PolymeshPrimitivesIdentityId, u32, Vec<PalletPipsSnapshottedPip>]>;
      /**
       * `AccountId` voted `bool` on the proposal referenced by `PipId`
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
      Approved: AugmentedEvent<ApiType, [PolymeshPrimitivesIdentityId, H256, u32, u32, u32]>;
      /**
       * A motion was executed; `DispatchResult` is `Ok(())` if returned without error.
       * Parameters: caller DID, proposal hash, result of proposal dispatch.
       **/
      Executed: AugmentedEvent<ApiType, [PolymeshPrimitivesIdentityId, H256, Result<Null, SpRuntimeDispatchError>]>;
      /**
       * Proposal expiry time has been updated.
       * Parameters: caller DID, new expiry time (if any).
       **/
      ExpiresAfterUpdated: AugmentedEvent<ApiType, [PolymeshPrimitivesIdentityId, PolymeshCommonUtilitiesMaybeBlock]>;
      /**
       * Final votes on a motion (given hash)
       * caller DID, ProposalIndex, Proposal hash, yes voters, no voter
       **/
      FinalVotes: AugmentedEvent<ApiType, [PolymeshPrimitivesIdentityId, u32, H256, Vec<PolymeshPrimitivesIdentityId>, Vec<PolymeshPrimitivesIdentityId>]>;
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
      Rejected: AugmentedEvent<ApiType, [PolymeshPrimitivesIdentityId, H256, u32, u32, u32]>;
      /**
       * Release coordinator has been updated.
       * Parameters: caller DID, DID of the release coordinator.
       **/
      ReleaseCoordinatorUpdated: AugmentedEvent<ApiType, [PolymeshPrimitivesIdentityId, Option<PolymeshPrimitivesIdentityId>]>;
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
       * Generic event
       **/
      [key: string]: AugmentedEvent<ApiType>;
    };
    portfolio: {
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
      Cleared: AugmentedEvent<ApiType, [H256]>;
      /**
       * A preimage has been noted.
       **/
      Noted: AugmentedEvent<ApiType, [H256]>;
      /**
       * A preimage has been requested.
       **/
      Requested: AugmentedEvent<ApiType, [H256]>;
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
       * Accepted paying key.
       * 
       * (Caller DID, User Key, Paying Key)
       **/
      AcceptedPayingKey: AugmentedEvent<ApiType, [PolymeshPrimitivesEventOnly, AccountId32, AccountId32]>;
      /**
       * Authorization given for `paying_key` to `user_key`.
       * 
       * (Caller DID, User Key, Paying Key, Initial POLYX limit, Auth ID)
       **/
      AuthorizedPayingKey: AugmentedEvent<ApiType, [PolymeshPrimitivesEventOnly, AccountId32, AccountId32, u128, u64]>;
      /**
       * Removed paying key.
       * 
       * (Caller DID, User Key, Paying Key)
       **/
      RemovedPayingKey: AugmentedEvent<ApiType, [PolymeshPrimitivesEventOnly, AccountId32, AccountId32]>;
      /**
       * Updated polyx limit.
       * 
       * (Caller DID, User Key, Paying Key, POLYX limit, old remaining POLYX)
       **/
      UpdatedPolyxLimit: AugmentedEvent<ApiType, [PolymeshPrimitivesEventOnly, AccountId32, AccountId32, u128, u128]>;
      /**
       * Generic event
       **/
      [key: string]: AugmentedEvent<ApiType>;
    };
    rewards: {
      /**
       * Itn reward was claimed.
       **/
      ItnRewardClaimed: AugmentedEvent<ApiType, [AccountId32, u128]>;
      /**
       * Generic event
       **/
      [key: string]: AugmentedEvent<ApiType>;
    };
    scheduler: {
      /**
       * The call for the provided hash was not found so the task has been aborted.
       **/
      CallUnavailable: AugmentedEvent<ApiType, [ITuple<[u32, u32]>, Option<U8aFixed>]>;
      /**
       * Canceled some task.
       **/
      Canceled: AugmentedEvent<ApiType, [u32, u32]>;
      /**
       * Dispatched some task.
       **/
      Dispatched: AugmentedEvent<ApiType, [ITuple<[u32, u32]>, Option<U8aFixed>, Result<Null, SpRuntimeDispatchError>]>;
      /**
       * The given task was unable to be renewed since the agenda is full at that block.
       **/
      PeriodicFailed: AugmentedEvent<ApiType, [ITuple<[u32, u32]>, Option<U8aFixed>]>;
      /**
       * The given task can never be executed since it is overweight.
       **/
      PermanentlyOverweight: AugmentedEvent<ApiType, [ITuple<[u32, u32]>, Option<U8aFixed>]>;
      /**
       * Scheduled some task.
       **/
      Scheduled: AugmentedEvent<ApiType, [u32, u32]>;
      /**
       * Generic event
       **/
      [key: string]: AugmentedEvent<ApiType>;
    };
    session: {
      /**
       * New session has happened. Note that the argument is the session index, not the
       * block number as the type might suggest.
       **/
      NewSession: AugmentedEvent<ApiType, [u32]>;
      /**
       * Generic event
       **/
      [key: string]: AugmentedEvent<ApiType>;
    };
    settlement: {
      /**
       * An affirmation has been withdrawn (did, portfolio, instruction_id)
       **/
      AffirmationWithdrawn: AugmentedEvent<ApiType, [PolymeshPrimitivesIdentityId, PolymeshPrimitivesIdentityIdPortfolioId, u64]>;
      /**
       * Failed to execute instruction.
       **/
      FailedToExecuteInstruction: AugmentedEvent<ApiType, [u64, SpRuntimeDispatchError]>;
      /**
       * An instruction has been affirmed (did, portfolio, instruction_id)
       **/
      InstructionAffirmed: AugmentedEvent<ApiType, [PolymeshPrimitivesIdentityId, PolymeshPrimitivesIdentityIdPortfolioId, u64]>;
      /**
       * A new instruction has been created
       * (did, venue_id, instruction_id, settlement_type, trade_date, value_date, legs, memo)
       **/
      InstructionCreated: AugmentedEvent<ApiType, [PolymeshPrimitivesIdentityId, u64, u64, PolymeshPrimitivesSettlementSettlementType, Option<u64>, Option<u64>, Vec<PolymeshPrimitivesSettlementLeg>, Option<PolymeshPrimitivesMemo>]>;
      /**
       * Instruction executed successfully(did, instruction_id)
       **/
      InstructionExecuted: AugmentedEvent<ApiType, [PolymeshPrimitivesIdentityId, u64]>;
      /**
       * Instruction failed execution (did, instruction_id)
       **/
      InstructionFailed: AugmentedEvent<ApiType, [PolymeshPrimitivesIdentityId, u64]>;
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
       * A receipt has been claimed (did, instruction_id, leg_id, receipt_uid, signer, receipt metadata)
       **/
      ReceiptClaimed: AugmentedEvent<ApiType, [PolymeshPrimitivesIdentityId, u64, u64, u64, AccountId32, Option<PolymeshPrimitivesSettlementReceiptMetadata>]>;
      /**
       * Scheduling of instruction fails.
       **/
      SchedulingFailed: AugmentedEvent<ApiType, [SpRuntimeDispatchError]>;
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
       * Venue filtering has been enabled or disabled for a ticker (did, ticker, filtering_enabled)
       **/
      VenueFiltering: AugmentedEvent<ApiType, [PolymeshPrimitivesIdentityId, PolymeshPrimitivesTicker, bool]>;
      /**
       * Venues added to allow list (did, ticker, vec<venue_id>)
       **/
      VenuesAllowed: AugmentedEvent<ApiType, [PolymeshPrimitivesIdentityId, PolymeshPrimitivesTicker, Vec<u64>]>;
      /**
       * Venues added to block list (did, ticker, vec<venue_id>)
       **/
      VenuesBlocked: AugmentedEvent<ApiType, [PolymeshPrimitivesIdentityId, PolymeshPrimitivesTicker, Vec<u64>]>;
      /**
       * An existing venue's signers has been updated (did, venue_id, signers, update_type)
       **/
      VenueSignersUpdated: AugmentedEvent<ApiType, [PolymeshPrimitivesIdentityId, u64, Vec<AccountId32>, bool]>;
      /**
       * An existing venue's type has been updated (did, venue_id, type)
       **/
      VenueTypeUpdated: AugmentedEvent<ApiType, [PolymeshPrimitivesIdentityId, u64, PolymeshPrimitivesSettlementVenueType]>;
      /**
       * Venue not part of the token's allow list (did, Ticker, venue_id)
       **/
      VenueUnauthorized: AugmentedEvent<ApiType, [PolymeshPrimitivesIdentityId, PolymeshPrimitivesTicker, u64]>;
      /**
       * Generic event
       **/
      [key: string]: AugmentedEvent<ApiType>;
    };
    staking: {
      /**
       * An account has bonded this amount. \[did, stash, amount\]
       * 
       * NOTE: This event is only emitted when funds are bonded via a dispatchable. Notably,
       * it will not be emitted for staking rewards when they are added to stake.
       **/
      Bonded: AugmentedEvent<ApiType, [PolymeshPrimitivesIdentityId, AccountId32, u128]>;
      /**
       * When commission cap get updated.
       * (old value, new value)
       **/
      CommissionCapUpdated: AugmentedEvent<ApiType, [PolymeshPrimitivesIdentityId, Perbill, Perbill]>;
      /**
       * The era payout has been set; the first balance is the validator-payout; the second is
       * the remainder from the maximum amount of reward.
       * \[era_index, validator_payout, remainder\]
       **/
      EraPayout: AugmentedEvent<ApiType, [u32, u128, u128]>;
      /**
       * Remove the nominators from the valid nominators when there CDD expired.
       * Caller, Stash accountId of nominators
       **/
      InvalidatedNominators: AugmentedEvent<ApiType, [PolymeshPrimitivesIdentityId, AccountId32, Vec<AccountId32>]>;
      /**
       * Min bond threshold was updated (new value).
       **/
      MinimumBondThresholdUpdated: AugmentedEvent<ApiType, [Option<PolymeshPrimitivesIdentityId>, u128]>;
      /**
       * User has updated their nominations
       **/
      Nominated: AugmentedEvent<ApiType, [PolymeshPrimitivesIdentityId, AccountId32, Vec<AccountId32>]>;
      /**
       * An old slashing report from a prior era was discarded because it could
       * not be processed. \[session_index\]
       **/
      OldSlashingReportDiscarded: AugmentedEvent<ApiType, [u32]>;
      /**
       * An DID has issued a candidacy. See the transaction for who.
       * GC identity , Validator's identity.
       **/
      PermissionedIdentityAdded: AugmentedEvent<ApiType, [PolymeshPrimitivesIdentityId, PolymeshPrimitivesIdentityId]>;
      /**
       * The given member was removed. See the transaction for who.
       * GC identity , Validator's identity.
       **/
      PermissionedIdentityRemoved: AugmentedEvent<ApiType, [PolymeshPrimitivesIdentityId, PolymeshPrimitivesIdentityId]>;
      /**
       * The staker has been rewarded by this amount. \[stash_identity, stash, amount\]
       **/
      Reward: AugmentedEvent<ApiType, [PolymeshPrimitivesIdentityId, AccountId32, u128]>;
      /**
       * When scheduling of reward payments get interrupted.
       **/
      RewardPaymentSchedulingInterrupted: AugmentedEvent<ApiType, [AccountId32, u32, SpRuntimeDispatchError]>;
      /**
       * One validator (and its nominators) has been slashed by the given amount.
       * \[validator, amount\]
       **/
      Slash: AugmentedEvent<ApiType, [AccountId32, u128]>;
      /**
       * Update for whom balance get slashed.
       **/
      SlashingAllowedForChanged: AugmentedEvent<ApiType, [PalletStakingSlashingSwitch]>;
      /**
       * A new solution for the upcoming election has been stored. \[compute\]
       **/
      SolutionStored: AugmentedEvent<ApiType, [PalletStakingElectionCompute]>;
      /**
       * A new set of stakers was elected with the given \[compute\].
       **/
      StakingElection: AugmentedEvent<ApiType, [PalletStakingElectionCompute]>;
      /**
       * An account has unbonded this amount. \[did, stash, amount\]
       **/
      Unbonded: AugmentedEvent<ApiType, [PolymeshPrimitivesIdentityId, AccountId32, u128]>;
      /**
       * An account has called `withdraw_unbonded` and removed unbonding chunks worth `Balance`
       * from the unlocking queue. \[stash, amount\]
       **/
      Withdrawn: AugmentedEvent<ApiType, [AccountId32, u128]>;
      /**
       * Generic event
       **/
      [key: string]: AugmentedEvent<ApiType>;
    };
    statistics: {
      /**
       * Asset stats updated.
       * 
       * (Caller DID, Asset, Stat type, Updates)
       **/
      AssetStatsUpdated: AugmentedEvent<ApiType, [PolymeshPrimitivesIdentityId, PolymeshPrimitivesStatisticsAssetScope, PolymeshPrimitivesStatisticsStatType, Vec<PolymeshPrimitivesStatisticsStatUpdate>]>;
      /**
       * Set Transfer compliance rules for asset.
       * 
       * (Caller DID, Asset, Transfer conditions)
       **/
      SetAssetTransferCompliance: AugmentedEvent<ApiType, [PolymeshPrimitivesIdentityId, PolymeshPrimitivesStatisticsAssetScope, Vec<PolymeshPrimitivesTransferComplianceTransferCondition>]>;
      /**
       * Stat types added to asset.
       * 
       * (Caller DID, Asset, Stat types)
       **/
      StatTypesAdded: AugmentedEvent<ApiType, [PolymeshPrimitivesIdentityId, PolymeshPrimitivesStatisticsAssetScope, Vec<PolymeshPrimitivesStatisticsStatType>]>;
      /**
       * Stat types removed from asset.
       * 
       * (Caller DID, Asset, Stat types)
       **/
      StatTypesRemoved: AugmentedEvent<ApiType, [PolymeshPrimitivesIdentityId, PolymeshPrimitivesStatisticsAssetScope, Vec<PolymeshPrimitivesStatisticsStatType>]>;
      /**
       * Add `ScopeId`s exempt for transfer conditions matching exempt key.
       * 
       * (Caller DID, Exempt key, Entities)
       **/
      TransferConditionExemptionsAdded: AugmentedEvent<ApiType, [PolymeshPrimitivesIdentityId, PolymeshPrimitivesTransferComplianceTransferConditionExemptKey, Vec<PolymeshPrimitivesIdentityId>]>;
      /**
       * Remove `ScopeId`s exempt for transfer conditions matching exempt key.
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
       * A fundraiser has been stopped.
       * (Agent DID, fundraiser id)
       **/
      FundraiserClosed: AugmentedEvent<ApiType, [PolymeshPrimitivesIdentityId, u64]>;
      /**
       * A new fundraiser has been created.
       * (Agent DID, fundraiser id, fundraiser name, fundraiser details)
       **/
      FundraiserCreated: AugmentedEvent<ApiType, [PolymeshPrimitivesIdentityId, u64, Bytes, PalletStoFundraiser]>;
      /**
       * A fundraiser has been frozen.
       * (Agent DID, fundraiser id)
       **/
      FundraiserFrozen: AugmentedEvent<ApiType, [PolymeshPrimitivesIdentityId, u64]>;
      /**
       * A fundraiser has been unfrozen.
       * (Agent DID, fundraiser id)
       **/
      FundraiserUnfrozen: AugmentedEvent<ApiType, [PolymeshPrimitivesIdentityId, u64]>;
      /**
       * A fundraiser window has been modified.
       * (Agent DID, fundraiser id, old_start, old_end, new_start, new_end)
       **/
      FundraiserWindowModified: AugmentedEvent<ApiType, [PolymeshPrimitivesEventOnly, u64, u64, Option<u64>, u64, Option<u64>]>;
      /**
       * An investor invested in the fundraiser.
       * (Investor, fundraiser_id, offering token, raise token, offering_token_amount, raise_token_amount)
       **/
      Invested: AugmentedEvent<ApiType, [PolymeshPrimitivesIdentityId, u64, PolymeshPrimitivesTicker, PolymeshPrimitivesTicker, u128, u128]>;
      /**
       * Generic event
       **/
      [key: string]: AugmentedEvent<ApiType>;
    };
    sudo: {
      /**
       * The \[sudoer\] just switched identity; the old key is supplied.
       **/
      KeyChanged: AugmentedEvent<ApiType, [Option<AccountId32>]>;
      /**
       * A sudo just took place. \[result\]
       **/
      Sudid: AugmentedEvent<ApiType, [Result<Null, SpRuntimeDispatchError>]>;
      /**
       * A sudo just took place. \[result\]
       **/
      SudoAsDone: AugmentedEvent<ApiType, [Result<Null, SpRuntimeDispatchError>]>;
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
      ExtrinsicFailed: AugmentedEvent<ApiType, [SpRuntimeDispatchError, FrameSupportDispatchDispatchInfo]>;
      /**
       * An extrinsic completed successfully.
       **/
      ExtrinsicSuccess: AugmentedEvent<ApiType, [FrameSupportDispatchDispatchInfo]>;
      /**
       * An account was reaped.
       **/
      KilledAccount: AugmentedEvent<ApiType, [AccountId32]>;
      /**
       * A new account was created.
       **/
      NewAccount: AugmentedEvent<ApiType, [AccountId32]>;
      /**
       * On on-chain remark happened.
       **/
      Remarked: AugmentedEvent<ApiType, [AccountId32, H256]>;
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
      Approved: AugmentedEvent<ApiType, [PolymeshPrimitivesIdentityId, H256, u32, u32, u32]>;
      /**
       * A motion was executed; `DispatchResult` is `Ok(())` if returned without error.
       * Parameters: caller DID, proposal hash, result of proposal dispatch.
       **/
      Executed: AugmentedEvent<ApiType, [PolymeshPrimitivesIdentityId, H256, Result<Null, SpRuntimeDispatchError>]>;
      /**
       * Proposal expiry time has been updated.
       * Parameters: caller DID, new expiry time (if any).
       **/
      ExpiresAfterUpdated: AugmentedEvent<ApiType, [PolymeshPrimitivesIdentityId, PolymeshCommonUtilitiesMaybeBlock]>;
      /**
       * Final votes on a motion (given hash)
       * caller DID, ProposalIndex, Proposal hash, yes voters, no voter
       **/
      FinalVotes: AugmentedEvent<ApiType, [PolymeshPrimitivesIdentityId, u32, H256, Vec<PolymeshPrimitivesIdentityId>, Vec<PolymeshPrimitivesIdentityId>]>;
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
      Rejected: AugmentedEvent<ApiType, [PolymeshPrimitivesIdentityId, H256, u32, u32, u32]>;
      /**
       * Release coordinator has been updated.
       * Parameters: caller DID, DID of the release coordinator.
       **/
      ReleaseCoordinatorUpdated: AugmentedEvent<ApiType, [PolymeshPrimitivesIdentityId, Option<PolymeshPrimitivesIdentityId>]>;
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
       * Phantom member, never used.
       **/
      Dummy: AugmentedEvent<ApiType, []>;
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
    testUtils: {
      /**
       * Shows the `DID` associated to the `AccountId`, and a flag indicates if that DID has a
       * valid CDD claim.
       * (Target DID, Target Account, a valid CDD claim exists)
       **/
      CddStatus: AugmentedEvent<ApiType, [Option<PolymeshPrimitivesIdentityId>, AccountId32, bool]>;
      /**
       * Emits the `IdentityId` and the `AccountId` of the caller.
       * (Caller DID, Caller account)
       **/
      DidStatus: AugmentedEvent<ApiType, [PolymeshPrimitivesIdentityId, AccountId32]>;
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
      TransactionFeePaid: AugmentedEvent<ApiType, [AccountId32, u128, u128]>;
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
      Approved: AugmentedEvent<ApiType, [PolymeshPrimitivesIdentityId, H256, u32, u32, u32]>;
      /**
       * A motion was executed; `DispatchResult` is `Ok(())` if returned without error.
       * Parameters: caller DID, proposal hash, result of proposal dispatch.
       **/
      Executed: AugmentedEvent<ApiType, [PolymeshPrimitivesIdentityId, H256, Result<Null, SpRuntimeDispatchError>]>;
      /**
       * Proposal expiry time has been updated.
       * Parameters: caller DID, new expiry time (if any).
       **/
      ExpiresAfterUpdated: AugmentedEvent<ApiType, [PolymeshPrimitivesIdentityId, PolymeshCommonUtilitiesMaybeBlock]>;
      /**
       * Final votes on a motion (given hash)
       * caller DID, ProposalIndex, Proposal hash, yes voters, no voter
       **/
      FinalVotes: AugmentedEvent<ApiType, [PolymeshPrimitivesIdentityId, u32, H256, Vec<PolymeshPrimitivesIdentityId>, Vec<PolymeshPrimitivesIdentityId>]>;
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
      Rejected: AugmentedEvent<ApiType, [PolymeshPrimitivesIdentityId, H256, u32, u32, u32]>;
      /**
       * Release coordinator has been updated.
       * Parameters: caller DID, DID of the release coordinator.
       **/
      ReleaseCoordinatorUpdated: AugmentedEvent<ApiType, [PolymeshPrimitivesIdentityId, Option<PolymeshPrimitivesIdentityId>]>;
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
       * Phantom member, never used.
       **/
      Dummy: AugmentedEvent<ApiType, []>;
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
       * Batch of dispatches completed fully with no error.
       * Includes a vector of event counts for each dispatch.
       * POLYMESH: event deprecated.
       **/
      BatchCompletedOld: AugmentedEvent<ApiType, [Vec<u32>]>;
      /**
       * Batch of dispatches completed but has errors.
       **/
      BatchCompletedWithErrors: AugmentedEvent<ApiType, []>;
      /**
       * Batch of dispatches did not complete fully. Index of first failing dispatch given, as
       * well as the error.
       **/
      BatchInterrupted: AugmentedEvent<ApiType, [u32, SpRuntimeDispatchError]>;
      /**
       * Batch of dispatches did not complete fully.
       * Includes a vector of event counts for each dispatch and
       * the index of the first failing dispatch as well as the error.
       * POLYMESH: event deprecated.
       **/
      BatchInterruptedOld: AugmentedEvent<ApiType, [Vec<u32>, ITuple<[u32, SpRuntimeDispatchError]>]>;
      /**
       * Batch of dispatches did not complete fully.
       * Includes a vector of event counts for each call and
       * a vector of any failed dispatches with their indices and associated error.
       * POLYMESH: event deprecated.
       **/
      BatchOptimisticFailed: AugmentedEvent<ApiType, [Vec<u32>, Vec<ITuple<[u32, SpRuntimeDispatchError]>>]>;
      /**
       * A call was dispatched.
       **/
      DispatchedAs: AugmentedEvent<ApiType, [Result<Null, SpRuntimeDispatchError>]>;
      /**
       * A single item within a Batch of dispatches has completed with no error.
       **/
      ItemCompleted: AugmentedEvent<ApiType, []>;
      /**
       * A single item within a Batch of dispatches has completed with error.
       **/
      ItemFailed: AugmentedEvent<ApiType, [SpRuntimeDispatchError]>;
      /**
       * Relayed transaction.
       * POLYMESH: event.
       **/
      RelayedTx: AugmentedEvent<ApiType, [PolymeshPrimitivesIdentityId, AccountId32, Result<Null, SpRuntimeDispatchError>]>;
      /**
       * Generic event
       **/
      [key: string]: AugmentedEvent<ApiType>;
    };
  } // AugmentedEvents
} // declare module
