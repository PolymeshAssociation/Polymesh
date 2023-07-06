// Auto-generated via `yarn polkadot-types-from-chain`, do not edit
/* eslint-disable */

import type { ApiTypes } from '@polkadot/api-base/types';

declare module '@polkadot/api-base/types/errors' {
  export interface AugmentedErrors<ApiType extends ApiTypes> {
    asset: {
      /**
       * The token is already frozen.
       **/
      AlreadyFrozen: AugmentedError<ApiType>;
      /**
       * The token has already been created.
       **/
      AssetAlreadyCreated: AugmentedError<ApiType>;
      /**
       * The token is already divisible.
       **/
      AssetAlreadyDivisible: AugmentedError<ApiType>;
      /**
       * Asset Metadata Global type already exists.
       **/
      AssetMetadataGlobalKeyAlreadyExists: AugmentedError<ApiType>;
      /**
       * Attempt to delete a key that is needed for an NFT collection.
       **/
      AssetMetadataKeyBelongsToNFTCollection: AugmentedError<ApiType>;
      /**
       * Asset Metadata key is missing.
       **/
      AssetMetadataKeyIsMissing: AugmentedError<ApiType>;
      /**
       * Asset Metadata Local type already exists for asset.
       **/
      AssetMetadataLocalKeyAlreadyExists: AugmentedError<ApiType>;
      /**
       * Maximum length of the asset metadata type name has been exceeded.
       **/
      AssetMetadataNameMaxLengthExceeded: AugmentedError<ApiType>;
      /**
       * Maximum length of the asset metadata type definition has been exceeded.
       **/
      AssetMetadataTypeDefMaxLengthExceeded: AugmentedError<ApiType>;
      /**
       * Attempt to lock a metadata value that is empty.
       **/
      AssetMetadataValueIsEmpty: AugmentedError<ApiType>;
      /**
       * Asset Metadata value is locked.
       **/
      AssetMetadataValueIsLocked: AugmentedError<ApiType>;
      /**
       * Maximum length of the asset metadata value has been exceeded.
       **/
      AssetMetadataValueMaxLengthExceeded: AugmentedError<ApiType>;
      /**
       * An overflow while calculating the balance.
       **/
      BalanceOverflow: AugmentedError<ApiType>;
      /**
       * Maximum length of the funding round name has been exceeded.
       **/
      FundingRoundNameMaxLengthExceeded: AugmentedError<ApiType>;
      /**
       * Attempt to update the type of a non fungible token to a fungible token or the other way around.
       **/
      IncompatibleAssetTypeUpdate: AugmentedError<ApiType>;
      /**
       * The sender balance is not sufficient.
       **/
      InsufficientBalance: AugmentedError<ApiType>;
      /**
       * Some `AssetIdentifier` was invalid.
       **/
      InvalidAssetIdentifier: AugmentedError<ApiType>;
      /**
       * Invalid `CustomAssetTypeId`.
       **/
      InvalidCustomAssetTypeId: AugmentedError<ApiType>;
      /**
       * An invalid Ethereum `EcdsaSignature`.
       **/
      InvalidEthereumSignature: AugmentedError<ApiType>;
      /**
       * An invalid granularity.
       **/
      InvalidGranularity: AugmentedError<ApiType>;
      /**
       * Transfer validation check failed.
       **/
      InvalidTransfer: AugmentedError<ApiType>;
      /**
       * Investor Uniqueness claims are not allowed for this asset.
       **/
      InvestorUniquenessClaimNotAllowed: AugmentedError<ApiType>;
      /**
       * Maximum length of asset name has been exceeded.
       **/
      MaxLengthOfAssetNameExceeded: AugmentedError<ApiType>;
      /**
       * No such token.
       **/
      NoSuchAsset: AugmentedError<ApiType>;
      /**
       * The given Document does not exist.
       **/
      NoSuchDoc: AugmentedError<ApiType>;
      /**
       * Not an owner of the token on Ethereum.
       **/
      NotAnOwner: AugmentedError<ApiType>;
      /**
       * The asset must be frozen.
       **/
      NotFrozen: AugmentedError<ApiType>;
      /**
       * Transfers to self are not allowed
       **/
      SenderSameAsReceiver: AugmentedError<ApiType>;
      /**
       * The ticker is already registered to someone else.
       **/
      TickerAlreadyRegistered: AugmentedError<ApiType>;
      /**
       * Tickers should start with at least one valid byte.
       **/
      TickerFirstByteNotValid: AugmentedError<ApiType>;
      /**
       * The ticker has non-alphanumeric parts.
       **/
      TickerNotAlphanumeric: AugmentedError<ApiType>;
      /**
       * Registration of ticker has expired.
       **/
      TickerRegistrationExpired: AugmentedError<ApiType>;
      /**
       * The ticker length is over the limit.
       **/
      TickerTooLong: AugmentedError<ApiType>;
      /**
       * The total supply is above the limit.
       **/
      TotalSupplyAboveLimit: AugmentedError<ApiType>;
      /**
       * An overflow while calculating the total supply.
       **/
      TotalSupplyOverflow: AugmentedError<ApiType>;
      /**
       * The user is not authorized.
       **/
      Unauthorized: AugmentedError<ApiType>;
      /**
       * Attempt to call an extrinsic that is only permitted for fungible tokens.
       **/
      UnexpectedNonFungibleToken: AugmentedError<ApiType>;
      /**
       * Generic error
       **/
      [key: string]: AugmentedError<ApiType>;
    };
    babe: {
      /**
       * A given equivocation report is valid but already previously reported.
       **/
      DuplicateOffenceReport: AugmentedError<ApiType>;
      /**
       * Submitted configuration is invalid.
       **/
      InvalidConfiguration: AugmentedError<ApiType>;
      /**
       * An equivocation proof provided as part of an equivocation report is invalid.
       **/
      InvalidEquivocationProof: AugmentedError<ApiType>;
      /**
       * A key ownership proof provided as part of an equivocation report is invalid.
       **/
      InvalidKeyOwnershipProof: AugmentedError<ApiType>;
      /**
       * Generic error
       **/
      [key: string]: AugmentedError<ApiType>;
    };
    balances: {
      /**
       * Value too low to create account due to existential deposit
       **/
      ExistentialDeposit: AugmentedError<ApiType>;
      /**
       * Balance too low to send value
       **/
      InsufficientBalance: AugmentedError<ApiType>;
      /**
       * Account liquidity restrictions prevent withdrawal
       **/
      LiquidityRestrictions: AugmentedError<ApiType>;
      /**
       * Got an overflow after adding
       **/
      Overflow: AugmentedError<ApiType>;
      /**
       * Receiver does not have a valid CDD
       **/
      ReceiverCddMissing: AugmentedError<ApiType>;
      /**
       * Generic error
       **/
      [key: string]: AugmentedError<ApiType>;
    };
    base: {
      /**
       * The sequence counter for something overflowed.
       * 
       * When this happens depends on e.g., the capacity of the identifier type.
       * For example, we might have `pub struct PipId(u32);`, with `u32::MAX` capacity.
       * In practice, these errors will never happen but no code path should result in a panic,
       * so these corner cases need to be covered with an error variant.
       **/
      CounterOverflow: AugmentedError<ApiType>;
      /**
       * Exceeded a generic length limit.
       * The limit could be for any sort of lists of things, including a string.
       **/
      TooLong: AugmentedError<ApiType>;
      /**
       * Generic error
       **/
      [key: string]: AugmentedError<ApiType>;
    };
    bridge: {
      /**
       * The origin is not the admin address.
       **/
      BadAdmin: AugmentedError<ApiType>;
      /**
       * The origin is not the controller or the admin address.
       **/
      BadCaller: AugmentedError<ApiType>;
      /**
       * The identity's minted total has reached the bridge limit.
       **/
      BridgeLimitReached: AugmentedError<ApiType>;
      /**
       * The bridge controller address is not set.
       **/
      ControllerNotSet: AugmentedError<ApiType>;
      /**
       * The block interval duration is zero. Cannot divide.
       **/
      DivisionByZero: AugmentedError<ApiType>;
      /**
       * The bridge is already frozen.
       **/
      Frozen: AugmentedError<ApiType>;
      /**
       * The transaction is frozen.
       **/
      FrozenTx: AugmentedError<ApiType>;
      /**
       * The bridge is not frozen.
       **/
      NotFrozen: AugmentedError<ApiType>;
      /**
       * The recipient DID has no valid CDD.
       **/
      NoValidCdd: AugmentedError<ApiType>;
      /**
       * The identity's minted total has overflowed.
       **/
      Overflow: AugmentedError<ApiType>;
      /**
       * The bridge transaction proposal has already been handled and the funds minted.
       **/
      ProposalAlreadyHandled: AugmentedError<ApiType>;
      /**
       * The transaction is timelocked.
       **/
      TimelockedTx: AugmentedError<ApiType>;
      /**
       * Unauthorized to perform an operation.
       **/
      Unauthorized: AugmentedError<ApiType>;
      /**
       * Generic error
       **/
      [key: string]: AugmentedError<ApiType>;
    };
    capitalDistribution: {
      /**
       * A distribution already exists for this CA.
       **/
      AlreadyExists: AugmentedError<ApiType>;
      /**
       * DID who created the distribution already did reclaim.
       **/
      AlreadyReclaimed: AugmentedError<ApiType>;
      /**
       * Multiplication of the balance with the per share payout amount overflowed.
       **/
      BalancePerShareProductOverflowed: AugmentedError<ApiType>;
      /**
       * Distribution's expiry has passed. DID cannot claim anymore and has forfeited the benefits.
       **/
      CannotClaimAfterExpiry: AugmentedError<ApiType>;
      /**
       * Distribution allotment cannot be claimed as the current time is before start-of-payment.
       **/
      CannotClaimBeforeStart: AugmentedError<ApiType>;
      /**
       * A capital distribution was made for a non-benefit CA.
       **/
      CANotBenefit: AugmentedError<ApiType>;
      /**
       * Distribution `amount` cannot be zero.
       **/
      DistributionAmountIsZero: AugmentedError<ApiType>;
      /**
       * Distribution `per_share` cannot be zero.
       **/
      DistributionPerShareIsZero: AugmentedError<ApiType>;
      /**
       * A distribution has been activated, as `payment_at <= now` holds.
       **/
      DistributionStarted: AugmentedError<ApiType>;
      /**
       * A distributions provided expiry date was strictly before its payment date.
       * In other words, everything to distribute would immediately be forfeited.
       **/
      ExpiryBeforePayment: AugmentedError<ApiType>;
      /**
       * The token holder has already been paid their benefit.
       **/
      HolderAlreadyPaid: AugmentedError<ApiType>;
      /**
       * A distribution has insufficient remaining amount of currency to distribute.
       **/
      InsufficientRemainingAmount: AugmentedError<ApiType>;
      /**
       * A capital distribution doesn't exist for this CA.
       **/
      NoSuchDistribution: AugmentedError<ApiType>;
      /**
       * DID is not the one who created the distribution.
       **/
      NotDistributionCreator: AugmentedError<ApiType>;
      /**
       * Distribution had not expired yet, or there's no expiry date.
       **/
      NotExpired: AugmentedError<ApiType>;
      /**
       * Generic error
       **/
      [key: string]: AugmentedError<ApiType>;
    };
    cddServiceProviders: {
      /**
       * The limit for the number of concurrent active members for this group has been exceeded.
       **/
      ActiveMembersLimitExceeded: AugmentedError<ApiType>;
      /**
       * Active member limit was greater than maximum committee members limit.
       **/
      ActiveMembersLimitOverflow: AugmentedError<ApiType>;
      /**
       * Group member was added already.
       **/
      DuplicateMember: AugmentedError<ApiType>;
      /**
       * Last member of the committee can not quit.
       **/
      LastMemberCannotQuit: AugmentedError<ApiType>;
      /**
       * Missing current DID
       **/
      MissingCurrentIdentity: AugmentedError<ApiType>;
      /**
       * Can't remove a member that doesn't exist.
       **/
      NoSuchMember: AugmentedError<ApiType>;
      /**
       * Only primary key of the identity is allowed.
       **/
      OnlyPrimaryKeyAllowed: AugmentedError<ApiType>;
      /**
       * Generic error
       **/
      [key: string]: AugmentedError<ApiType>;
    };
    checkpoint: {
      /**
       * A checkpoint schedule does not exist for the asset.
       **/
      NoSuchSchedule: AugmentedError<ApiType>;
      /**
       * The schedule has no more checkpoints.
       **/
      ScheduleFinished: AugmentedError<ApiType>;
      /**
       * The schedule has expired checkpoints.
       **/
      ScheduleHasExpiredCheckpoints: AugmentedError<ApiType>;
      /**
       * Can't create an empty schedule.
       **/
      ScheduleIsEmpty: AugmentedError<ApiType>;
      /**
       * A checkpoint schedule is not removable as `ref_count(schedule_id) > 0`.
       **/
      ScheduleNotRemovable: AugmentedError<ApiType>;
      /**
       * The new schedule would put the ticker over the maximum complexity allowed.
       **/
      SchedulesOverMaxComplexity: AugmentedError<ApiType>;
      /**
       * Generic error
       **/
      [key: string]: AugmentedError<ApiType>;
    };
    committeeMembership: {
      /**
       * The limit for the number of concurrent active members for this group has been exceeded.
       **/
      ActiveMembersLimitExceeded: AugmentedError<ApiType>;
      /**
       * Active member limit was greater than maximum committee members limit.
       **/
      ActiveMembersLimitOverflow: AugmentedError<ApiType>;
      /**
       * Group member was added already.
       **/
      DuplicateMember: AugmentedError<ApiType>;
      /**
       * Last member of the committee can not quit.
       **/
      LastMemberCannotQuit: AugmentedError<ApiType>;
      /**
       * Missing current DID
       **/
      MissingCurrentIdentity: AugmentedError<ApiType>;
      /**
       * Can't remove a member that doesn't exist.
       **/
      NoSuchMember: AugmentedError<ApiType>;
      /**
       * Only primary key of the identity is allowed.
       **/
      OnlyPrimaryKeyAllowed: AugmentedError<ApiType>;
      /**
       * Generic error
       **/
      [key: string]: AugmentedError<ApiType>;
    };
    complianceManager: {
      /**
       * The worst case scenario of the compliance requirement is too complex.
       **/
      ComplianceRequirementTooComplex: AugmentedError<ApiType>;
      /**
       * Did not exist.
       **/
      DidNotExist: AugmentedError<ApiType>;
      /**
       * There are duplicate compliance requirements.
       **/
      DuplicateComplianceRequirements: AugmentedError<ApiType>;
      /**
       * Issuer exist but trying to add it again.
       **/
      IncorrectOperationOnTrustedIssuer: AugmentedError<ApiType>;
      /**
       * Compliance requirement id doesn't exist.
       **/
      InvalidComplianceRequirementId: AugmentedError<ApiType>;
      /**
       * User is not authorized.
       **/
      Unauthorized: AugmentedError<ApiType>;
      /**
       * The maximum weight limit for executing the function was exceeded.
       **/
      WeightLimitExceeded: AugmentedError<ApiType>;
      /**
       * Generic error
       **/
      [key: string]: AugmentedError<ApiType>;
    };
    contracts: {
      /**
       * Code removal was denied because the code is still in use by at least one contract.
       **/
      CodeInUse: AugmentedError<ApiType>;
      /**
       * No code could be found at the supplied code hash.
       **/
      CodeNotFound: AugmentedError<ApiType>;
      /**
       * The contract's code was found to be invalid during validation or instrumentation.
       * 
       * The most likely cause of this is that an API was used which is not supported by the
       * node. This hapens if an older node is used with a new version of ink!. Try updating
       * your node to the newest available version.
       * 
       * A more detailed error can be found on the node console if debug messages are enabled
       * by supplying `-lruntime::contracts=debug`.
       **/
      CodeRejected: AugmentedError<ApiType>;
      /**
       * The code supplied to `instantiate_with_code` exceeds the limit specified in the
       * current schedule.
       **/
      CodeTooLarge: AugmentedError<ApiType>;
      /**
       * No contract was found at the specified address.
       **/
      ContractNotFound: AugmentedError<ApiType>;
      /**
       * The contract ran to completion but decided to revert its storage changes.
       * Please note that this error is only returned from extrinsics. When called directly
       * or via RPC an `Ok` will be returned. In this case the caller needs to inspect the flags
       * to determine whether a reversion has taken place.
       **/
      ContractReverted: AugmentedError<ApiType>;
      /**
       * Contract trapped during execution.
       **/
      ContractTrapped: AugmentedError<ApiType>;
      /**
       * Input passed to a contract API function failed to decode as expected type.
       **/
      DecodingFailed: AugmentedError<ApiType>;
      /**
       * Removal of a contract failed because the deletion queue is full.
       * 
       * This can happen when calling `seal_terminate`.
       * The queue is filled by deleting contracts and emptied by a fixed amount each block.
       * Trying again during another block is the only way to resolve this issue.
       **/
      DeletionQueueFull: AugmentedError<ApiType>;
      /**
       * A contract with the same AccountId already exists.
       **/
      DuplicateContract: AugmentedError<ApiType>;
      /**
       * An indetermistic code was used in a context where this is not permitted.
       **/
      Indeterministic: AugmentedError<ApiType>;
      /**
       * `seal_call` forwarded this contracts input. It therefore is no longer available.
       **/
      InputForwarded: AugmentedError<ApiType>;
      /**
       * Invalid combination of flags supplied to `seal_call` or `seal_delegate_call`.
       **/
      InvalidCallFlags: AugmentedError<ApiType>;
      /**
       * A new schedule must have a greater version than the current one.
       **/
      InvalidScheduleVersion: AugmentedError<ApiType>;
      /**
       * Performing a call was denied because the calling depth reached the limit
       * of what is specified in the schedule.
       **/
      MaxCallDepthReached: AugmentedError<ApiType>;
      /**
       * The chain does not provide a chain extension. Calling the chain extension results
       * in this error. Note that this usually  shouldn't happen as deploying such contracts
       * is rejected.
       **/
      NoChainExtension: AugmentedError<ApiType>;
      /**
       * A buffer outside of sandbox memory was passed to a contract API function.
       **/
      OutOfBounds: AugmentedError<ApiType>;
      /**
       * The executed contract exhausted its gas limit.
       **/
      OutOfGas: AugmentedError<ApiType>;
      /**
       * The output buffer supplied to a contract API call was too small.
       **/
      OutputBufferTooSmall: AugmentedError<ApiType>;
      /**
       * The subject passed to `seal_random` exceeds the limit.
       **/
      RandomSubjectTooLong: AugmentedError<ApiType>;
      /**
       * A call tried to invoke a contract that is flagged as non-reentrant.
       **/
      ReentranceDenied: AugmentedError<ApiType>;
      /**
       * More storage was created than allowed by the storage deposit limit.
       **/
      StorageDepositLimitExhausted: AugmentedError<ApiType>;
      /**
       * Origin doesn't have enough balance to pay the required storage deposits.
       **/
      StorageDepositNotEnoughFunds: AugmentedError<ApiType>;
      /**
       * A contract self destructed in its constructor.
       * 
       * This can be triggered by a call to `seal_terminate`.
       **/
      TerminatedInConstructor: AugmentedError<ApiType>;
      /**
       * Termination of a contract is not allowed while the contract is already
       * on the call stack. Can be triggered by `seal_terminate`.
       **/
      TerminatedWhileReentrant: AugmentedError<ApiType>;
      /**
       * The amount of topics passed to `seal_deposit_events` exceeds the limit.
       **/
      TooManyTopics: AugmentedError<ApiType>;
      /**
       * Performing the requested transfer failed. Probably because there isn't enough
       * free balance in the sender's account.
       **/
      TransferFailed: AugmentedError<ApiType>;
      /**
       * The size defined in `T::MaxValueSize` was exceeded.
       **/
      ValueTooLarge: AugmentedError<ApiType>;
      /**
       * Generic error
       **/
      [key: string]: AugmentedError<ApiType>;
    };
    corporateAction: {
      /**
       * A CA's declaration date was strictly after its record date.
       **/
      DeclDateAfterRecordDate: AugmentedError<ApiType>;
      /**
       * A CA's declaration date occurs in the future.
       **/
      DeclDateInFuture: AugmentedError<ApiType>;
      /**
       * The `details` of a CA exceeded the max allowed length.
       **/
      DetailsTooLong: AugmentedError<ApiType>;
      /**
       * A withholding tax override for a given DID was specified more than once.
       * The chain refused to make a choice, and hence there was an error.
       **/
      DuplicateDidTax: AugmentedError<ApiType>;
      /**
       * The CA did not have a record date.
       **/
      NoRecordDate: AugmentedError<ApiType>;
      /**
       * A CA with the given `CAId` did not exist.
       **/
      NoSuchCA: AugmentedError<ApiType>;
      /**
       * On CA creation, a checkpoint ID was provided which doesn't exist.
       **/
      NoSuchCheckpointId: AugmentedError<ApiType>;
      /**
       * CA does not target the DID.
       **/
      NotTargetedByCA: AugmentedError<ApiType>;
      /**
       * A CA's record date was strictly after the "start" time,
       * where "start" is context dependent.
       * For example, it could be the start of a ballot, or the start-of-payment in capital distribution.
       **/
      RecordDateAfterStart: AugmentedError<ApiType>;
      /**
       * Too many withholding tax overrides were specified.
       **/
      TooManyDidTaxes: AugmentedError<ApiType>;
      /**
       * Too many identities in `TargetIdentities` were specified.
       **/
      TooManyTargetIds: AugmentedError<ApiType>;
      /**
       * Generic error
       **/
      [key: string]: AugmentedError<ApiType>;
    };
    corporateBallot: {
      /**
       * A corporate ballot already exists for this CA.
       **/
      AlreadyExists: AugmentedError<ApiType>;
      /**
       * A corporate ballot was made for a non `IssuerNotice` CA.
       **/
      CANotNotice: AugmentedError<ApiType>;
      /**
       * Voting power used by a DID on a motion exceeds that which is available to them.
       **/
      InsufficientVotes: AugmentedError<ApiType>;
      /**
       * A corporate ballot doesn't exist for this CA.
       **/
      NoSuchBallot: AugmentedError<ApiType>;
      /**
       * The RCV fallback of some choice does not exist.
       **/
      NoSuchRCVFallback: AugmentedError<ApiType>;
      /**
       * A corporate ballot's end time was strictly before the current time.
       **/
      NowAfterEnd: AugmentedError<ApiType>;
      /**
       * If some motion in a corporate ballot has more choices than would fit in `u16`.
       **/
      NumberOfChoicesOverflow: AugmentedError<ApiType>;
      /**
       * RCV is not allowed for this ballot.
       **/
      RCVNotAllowed: AugmentedError<ApiType>;
      /**
       * The RCV fallback points to the origin choice.
       **/
      RCVSelfCycle: AugmentedError<ApiType>;
      /**
       * A corporate ballot's start time was strictly after the ballot's end.
       **/
      StartAfterEnd: AugmentedError<ApiType>;
      /**
       * Voting ended already.
       **/
      VotingAlreadyEnded: AugmentedError<ApiType>;
      /**
       * Voting started already. Amending a ballot is no longer possible.
       **/
      VotingAlreadyStarted: AugmentedError<ApiType>;
      /**
       * Voting hasn't started yet.
       **/
      VotingNotStarted: AugmentedError<ApiType>;
      /**
       * Provided list of balances does not match the total number of choices.
       **/
      WrongVoteCount: AugmentedError<ApiType>;
      /**
       * Generic error
       **/
      [key: string]: AugmentedError<ApiType>;
    };
    externalAgents: {
      /**
       * The provided `agent` is already an agent for the `Ticker`.
       **/
      AlreadyAnAgent: AugmentedError<ApiType>;
      /**
       * An AG with the given `AGId` did not exist for the `Ticker`.
       **/
      NoSuchAG: AugmentedError<ApiType>;
      /**
       * The provided `agent` is not an agent for the `Ticker`.
       **/
      NotAnAgent: AugmentedError<ApiType>;
      /**
       * This agent is the last full one, and it's being removed,
       * making the asset orphaned.
       **/
      RemovingLastFullAgent: AugmentedError<ApiType>;
      /**
       * The caller's secondary key does not have the required asset permission.
       **/
      SecondaryKeyNotAuthorizedForAsset: AugmentedError<ApiType>;
      /**
       * The agent is not authorized to call the current extrinsic.
       **/
      UnauthorizedAgent: AugmentedError<ApiType>;
      /**
       * Generic error
       **/
      [key: string]: AugmentedError<ApiType>;
    };
    grandpa: {
      /**
       * Attempt to signal GRANDPA change with one already pending.
       **/
      ChangePending: AugmentedError<ApiType>;
      /**
       * A given equivocation report is valid but already previously reported.
       **/
      DuplicateOffenceReport: AugmentedError<ApiType>;
      /**
       * An equivocation proof provided as part of an equivocation report is invalid.
       **/
      InvalidEquivocationProof: AugmentedError<ApiType>;
      /**
       * A key ownership proof provided as part of an equivocation report is invalid.
       **/
      InvalidKeyOwnershipProof: AugmentedError<ApiType>;
      /**
       * Attempt to signal GRANDPA pause when the authority set isn't live
       * (either paused or already pending pause).
       **/
      PauseFailed: AugmentedError<ApiType>;
      /**
       * Attempt to signal GRANDPA resume when the authority set isn't paused
       * (either live or already pending resume).
       **/
      ResumeFailed: AugmentedError<ApiType>;
      /**
       * Cannot signal forced change so soon after last.
       **/
      TooSoon: AugmentedError<ApiType>;
      /**
       * Generic error
       **/
      [key: string]: AugmentedError<ApiType>;
    };
    identity: {
      /**
       * The account key is being used, it can't be unlinked.
       **/
      AccountKeyIsBeingUsed: AugmentedError<ApiType>;
      /**
       * One secondary or primary key can only belong to one DID
       **/
      AlreadyLinked: AugmentedError<ApiType>;
      /**
       * The offchain authorization has expired.
       **/
      AuthorizationExpired: AugmentedError<ApiType>;
      /**
       * Authorization has been explicitly revoked.
       **/
      AuthorizationHasBeenRevoked: AugmentedError<ApiType>;
      /**
       * Authorizations are not for the same DID.
       **/
      AuthorizationsNotForSameDids: AugmentedError<ApiType>;
      /**
       * Cannot convert a `T::AccountId` to `AnySignature::Signer::AccountId`.
       **/
      CannotDecodeSignerAccountId: AugmentedError<ApiType>;
      /**
       * CDDId should be unique & same within all cdd claims possessed by a DID.
       **/
      CDDIdNotUniqueForIdentity: AugmentedError<ApiType>;
      /**
       * Claim and Proof versions are different.
       **/
      ClaimAndProofVersionsDoNotMatch: AugmentedError<ApiType>;
      /**
       * Claim does not exist.
       **/
      ClaimDoesNotExist: AugmentedError<ApiType>;
      /**
       * Try to add a claim variant using un-designated extrinsic.
       **/
      ClaimVariantNotAllowed: AugmentedError<ApiType>;
      /**
       * Current identity cannot be forwarded, it is not a secondary key of target identity.
       **/
      CurrentIdentityCannotBeForwarded: AugmentedError<ApiType>;
      /**
       * The custom claim type trying to be registered already exists.
       **/
      CustomClaimTypeAlreadyExists: AugmentedError<ApiType>;
      /**
       * The custom claim type does not exist.
       **/
      CustomClaimTypeDoesNotExist: AugmentedError<ApiType>;
      /**
       * A custom scope is too long.
       * It can at most be `32` characters long.
       **/
      CustomScopeTooLong: AugmentedError<ApiType>;
      /**
       * The DID already exists.
       **/
      DidAlreadyExists: AugmentedError<ApiType>;
      /**
       * The DID does not exist.
       **/
      DidDoesNotExist: AugmentedError<ApiType>;
      /**
       * The DID must already exist.
       **/
      DidMustAlreadyExist: AugmentedError<ApiType>;
      /**
       * The same key was included multiple times.
       **/
      DuplicateKey: AugmentedError<ApiType>;
      /**
       * Cannot use Except when specifying extrinsic permissions.
       **/
      ExceptNotAllowedForExtrinsics: AugmentedError<ApiType>;
      /**
       * Couldn't charge fee for the transaction.
       **/
      FailedToChargeFee: AugmentedError<ApiType>;
      /**
       * Account Id cannot be extracted from signer
       **/
      InvalidAccountKey: AugmentedError<ApiType>;
      /**
       * An invalid authorization from the CDD provider.
       **/
      InvalidAuthorizationFromCddProvider: AugmentedError<ApiType>;
      /**
       * An invalid authorization from the owner.
       **/
      InvalidAuthorizationFromOwner: AugmentedError<ApiType>;
      /**
       * An invalid authorization signature.
       **/
      InvalidAuthorizationSignature: AugmentedError<ApiType>;
      /**
       * Non systematic CDD providers can not create default cdd_id claims.
       **/
      InvalidCDDId: AugmentedError<ApiType>;
      /**
       * Identity is already a child of an other identity, can't create grand-child identity.
       **/
      IsChildIdentity: AugmentedError<ApiType>;
      /**
       * This key is not allowed to execute a given operation.
       **/
      KeyNotAllowed: AugmentedError<ApiType>;
      /**
       * Missing current identity on the transaction
       **/
      MissingCurrentIdentity: AugmentedError<ApiType>;
      /**
       * Multisig can not be unlinked from an identity while it still holds POLYX
       **/
      MultiSigHasBalance: AugmentedError<ApiType>;
      /**
       * The Identity doesn't have a parent identity.
       **/
      NoParentIdentity: AugmentedError<ApiType>;
      /**
       * Signer is not a secondary key of the provided identity
       **/
      NotASigner: AugmentedError<ApiType>;
      /**
       * Attestation was not by a CDD service provider.
       **/
      NotCddProviderAttestation: AugmentedError<ApiType>;
      /**
       * The caller is not the parent or child identity.
       **/
      NotParentOrChildIdentity: AugmentedError<ApiType>;
      /**
       * Only the primary key is allowed to revoke an Identity Signatory off-chain authorization.
       **/
      NotPrimaryKey: AugmentedError<ApiType>;
      /**
       * The secondary keys contain the primary key.
       **/
      SecondaryKeysContainPrimaryKey: AugmentedError<ApiType>;
      /**
       * The target DID has no valid CDD.
       **/
      TargetHasNoCdd: AugmentedError<ApiType>;
      /**
       * Try to delete the IU claim even when the user has non zero balance at given scopeId.
       **/
      TargetHasNonZeroBalanceAtScopeId: AugmentedError<ApiType>;
      /**
       * Signatory is not pre authorized by the identity
       **/
      Unauthorized: AugmentedError<ApiType>;
      /**
       * Only CDD service providers are allowed.
       **/
      UnAuthorizedCddProvider: AugmentedError<ApiType>;
      /**
       * Generic error
       **/
      [key: string]: AugmentedError<ApiType>;
    };
    imOnline: {
      /**
       * Duplicated heartbeat.
       **/
      DuplicatedHeartbeat: AugmentedError<ApiType>;
      /**
       * Non existent public key.
       **/
      InvalidKey: AugmentedError<ApiType>;
      /**
       * Generic error
       **/
      [key: string]: AugmentedError<ApiType>;
    };
    indices: {
      /**
       * The index was not available.
       **/
      InUse: AugmentedError<ApiType>;
      /**
       * The index was not already assigned.
       **/
      NotAssigned: AugmentedError<ApiType>;
      /**
       * The index is assigned to another account.
       **/
      NotOwner: AugmentedError<ApiType>;
      /**
       * The source and destination accounts are identical.
       **/
      NotTransfer: AugmentedError<ApiType>;
      /**
       * The index is permanent and may not be freed/changed.
       **/
      Permanent: AugmentedError<ApiType>;
      /**
       * Generic error
       **/
      [key: string]: AugmentedError<ApiType>;
    };
    multiSig: {
      /**
       * Already a signer.
       **/
      AlreadyASigner: AugmentedError<ApiType>;
      /**
       * Already voted.
       **/
      AlreadyVoted: AugmentedError<ApiType>;
      /**
       * The multisig is not attached to a CDD'd identity.
       **/
      CddMissing: AugmentedError<ApiType>;
      /**
       * Changing multisig parameters not allowed since multisig is a primary key.
       **/
      ChangeNotAllowed: AugmentedError<ApiType>;
      /**
       * The creator is no longer allowed to call via creator extrinsics.
       **/
      CreatorControlsHaveBeenRemoved: AugmentedError<ApiType>;
      /**
       * Multisig address.
       **/
      DecodingError: AugmentedError<ApiType>;
      /**
       * Couldn't charge fee for the transaction.
       **/
      FailedToChargeFee: AugmentedError<ApiType>;
      /**
       * Scheduling of a proposal fails
       **/
      FailedToSchedule: AugmentedError<ApiType>;
      /**
       * Identity provided is not the multisig's creator.
       **/
      IdentityNotCreator: AugmentedError<ApiType>;
      /**
       * Current DID is missing
       **/
      MissingCurrentIdentity: AugmentedError<ApiType>;
      /**
       * Multisig is not attached to an identity
       **/
      MultisigMissingIdentity: AugmentedError<ApiType>;
      /**
       * Multisig not allowed to add itself as a signer.
       **/
      MultisigNotAllowedToLinkToItself: AugmentedError<ApiType>;
      /**
       * A nonce overflow.
       **/
      NonceOverflow: AugmentedError<ApiType>;
      /**
       * No signers.
       **/
      NoSigners: AugmentedError<ApiType>;
      /**
       * No such multisig.
       **/
      NoSuchMultisig: AugmentedError<ApiType>;
      /**
       * Not a signer.
       **/
      NotASigner: AugmentedError<ApiType>;
      /**
       * Not enough signers.
       **/
      NotEnoughSigners: AugmentedError<ApiType>;
      /**
       * The function can only be called by the primary key of the did
       **/
      NotPrimaryKey: AugmentedError<ApiType>;
      /**
       * Proposal was executed earlier
       **/
      ProposalAlreadyExecuted: AugmentedError<ApiType>;
      /**
       * Proposal was rejected earlier
       **/
      ProposalAlreadyRejected: AugmentedError<ApiType>;
      /**
       * Proposal has expired
       **/
      ProposalExpired: AugmentedError<ApiType>;
      /**
       * The proposal does not exist.
       **/
      ProposalMissing: AugmentedError<ApiType>;
      /**
       * Too few or too many required signatures.
       **/
      RequiredSignaturesOutOfBounds: AugmentedError<ApiType>;
      /**
       * Signer is an account key that is already associated with an identity.
       **/
      SignerAlreadyLinkedToIdentity: AugmentedError<ApiType>;
      /**
       * Signer is an account key that is already associated with a multisig.
       **/
      SignerAlreadyLinkedToMultisig: AugmentedError<ApiType>;
      /**
       * More signers than required.
       **/
      TooManySigners: AugmentedError<ApiType>;
      /**
       * Generic error
       **/
      [key: string]: AugmentedError<ApiType>;
    };
    nft: {
      /**
       * An overflow while calculating the balance.
       **/
      BalanceOverflow: AugmentedError<ApiType>;
      /**
       * An underflow while calculating the balance.
       **/
      BalanceUnderflow: AugmentedError<ApiType>;
      /**
       * The ticker is already associated to an NFT collection.
       **/
      CollectionAlredyRegistered: AugmentedError<ApiType>;
      /**
       * The NFT collection does not exist.
       **/
      CollectionNotFound: AugmentedError<ApiType>;
      /**
       * Duplicate ids are not allowed.
       **/
      DuplicatedNFTId: AugmentedError<ApiType>;
      /**
       * A duplicate metadata key has been passed as parameter.
       **/
      DuplicateMetadataKey: AugmentedError<ApiType>;
      /**
       * The asset must be of type non-fungible.
       **/
      InvalidAssetType: AugmentedError<ApiType>;
      /**
       * Either the number of keys or the key identifier does not match the keys defined for the collection.
       **/
      InvalidMetadataAttribute: AugmentedError<ApiType>;
      /**
       * Failed to transfer an NFT - NFT collection not found.
       **/
      InvalidNFTTransferCollectionNotFound: AugmentedError<ApiType>;
      /**
       * Failed to transfer an NFT - compliance failed.
       **/
      InvalidNFTTransferComplianceFailure: AugmentedError<ApiType>;
      /**
       * Failed to transfer an NFT - identity count would overflow.
       **/
      InvalidNFTTransferCountOverflow: AugmentedError<ApiType>;
      /**
       * Failed to transfer an NFT - asset is frozen.
       **/
      InvalidNFTTransferFrozenAsset: AugmentedError<ApiType>;
      /**
       * Failed to transfer an NFT - the number of nfts in the identity is insufficient.
       **/
      InvalidNFTTransferInsufficientCount: AugmentedError<ApiType>;
      /**
       * Failed to transfer an NFT - NFT not found in portfolio.
       **/
      InvalidNFTTransferNFTNotOwned: AugmentedError<ApiType>;
      /**
       * Failed to transfer an NFT - attempt to move to the same portfolio.
       **/
      InvalidNFTTransferSamePortfolio: AugmentedError<ApiType>;
      /**
       * The maximum number of metadata keys was exceeded.
       **/
      MaxNumberOfKeysExceeded: AugmentedError<ApiType>;
      /**
       * The maximum number of nfts being transferred in one leg was exceeded.
       **/
      MaxNumberOfNFTsPerLegExceeded: AugmentedError<ApiType>;
      /**
       * The NFT does not exist.
       **/
      NFTNotFound: AugmentedError<ApiType>;
      /**
       * At least one of the metadata keys has not been registered.
       **/
      UnregisteredMetadataKey: AugmentedError<ApiType>;
      /**
       * It is not possible to transferr zero nft.
       **/
      ZeroCount: AugmentedError<ApiType>;
      /**
       * Generic error
       **/
      [key: string]: AugmentedError<ApiType>;
    };
    permissions: {
      /**
       * The caller is not authorized to call the current extrinsic.
       **/
      UnauthorizedCaller: AugmentedError<ApiType>;
      /**
       * Generic error
       **/
      [key: string]: AugmentedError<ApiType>;
    };
    pips: {
      /**
       * When enacting snapshot results, an unskippable PIP was skipped.
       **/
      CannotSkipPip: AugmentedError<ApiType>;
      /**
       * Proposer specifies an incorrect deposit
       **/
      IncorrectDeposit: AugmentedError<ApiType>;
      /**
       * Proposal is not in the correct state
       **/
      IncorrectProposalState: AugmentedError<ApiType>;
      /**
       * Proposer can't afford to lock minimum deposit
       **/
      InsufficientDeposit: AugmentedError<ApiType>;
      /**
       * When a block number is less than current block number.
       **/
      InvalidFutureBlockNumber: AugmentedError<ApiType>;
      /**
       * Missing current DID
       **/
      MissingCurrentIdentity: AugmentedError<ApiType>;
      /**
       * The proposal does not exist.
       **/
      NoSuchProposal: AugmentedError<ApiType>;
      /**
       * Not part of governance committee.
       **/
      NotACommitteeMember: AugmentedError<ApiType>;
      /**
       * The given dispatchable call is not valid for this proposal.
       * The proposal must be by community, but isn't.
       **/
      NotByCommittee: AugmentedError<ApiType>;
      /**
       * The given dispatchable call is not valid for this proposal.
       * The proposal must be from the community, but isn't.
       **/
      NotFromCommunity: AugmentedError<ApiType>;
      /**
       * When number of votes overflows.
       **/
      NumberOfVotesExceeded: AugmentedError<ApiType>;
      /**
       * A proposal that is not in a scheduled state cannot be executed.
       **/
      ProposalNotInScheduledState: AugmentedError<ApiType>;
      /**
       * Only the GC release coordinator is allowed to reschedule proposal execution.
       **/
      RescheduleNotByReleaseCoordinator: AugmentedError<ApiType>;
      /**
       * Execution of a scheduled proposal failed because it is missing.
       **/
      ScheduledProposalDoesntExist: AugmentedError<ApiType>;
      /**
       * Tried to enact result for PIP with id different from that at the position in the queue.
       **/
      SnapshotIdMismatch: AugmentedError<ApiType>;
      /**
       * Tried to enact results for the snapshot queue overflowing its length.
       **/
      SnapshotResultTooLarge: AugmentedError<ApiType>;
      /**
       * When stake amount of a vote overflows.
       **/
      StakeAmountOfVotesExceeded: AugmentedError<ApiType>;
      /**
       * The current number of active (pending | scheduled) PIPs exceed the maximum
       * and the proposal is not by a committee.
       **/
      TooManyActivePips: AugmentedError<ApiType>;
      /**
       * Generic error
       **/
      [key: string]: AugmentedError<ApiType>;
    };
    polymeshCommittee: {
      /**
       * Duplicate proposal.
       **/
      DuplicateProposal: AugmentedError<ApiType>;
      /**
       * Duplicate votes are not allowed.
       **/
      DuplicateVote: AugmentedError<ApiType>;
      /**
       * First vote on a proposal creates it, so it must be an approval.
       * All proposals are motions to execute something as "GC majority".
       * To reject e.g., a PIP, a motion to reject should be *approved*.
       **/
      FirstVoteReject: AugmentedError<ApiType>;
      /**
       * Proportion must be a rational number.
       **/
      InvalidProportion: AugmentedError<ApiType>;
      /**
       * Mismatched voting index.
       **/
      MismatchedVotingIndex: AugmentedError<ApiType>;
      /**
       * No such proposal.
       **/
      NoSuchProposal: AugmentedError<ApiType>;
      /**
       * A DID isn't part of the committee.
       * The DID may either be a caller or some other context.
       **/
      NotAMember: AugmentedError<ApiType>;
      /**
       * Proposal exists, but it has expired.
       **/
      ProposalExpired: AugmentedError<ApiType>;
      /**
       * Maximum number of proposals has been reached.
       **/
      ProposalsLimitReached: AugmentedError<ApiType>;
      /**
       * Generic error
       **/
      [key: string]: AugmentedError<ApiType>;
    };
    polymeshContracts: {
      /**
       * Data left in input when decoding arguments of a call.
       **/
      DataLeftAfterDecoding: AugmentedError<ApiType>;
      /**
       * Input data that a contract passed when using the ChainExtension was too large.
       **/
      InLenTooLarge: AugmentedError<ApiType>;
      /**
       * A contract was attempted to be instantiated,
       * but no identity was given to associate the new contract's key with.
       **/
      InstantiatorWithNoIdentity: AugmentedError<ApiType>;
      /**
       * Invalid `func_id` provided from contract.
       **/
      InvalidFuncId: AugmentedError<ApiType>;
      /**
       * Failed to decode a valid `RuntimeCall`.
       **/
      InvalidRuntimeCall: AugmentedError<ApiType>;
      /**
       * Output data returned from the ChainExtension was too large.
       **/
      OutLenTooLarge: AugmentedError<ApiType>;
      /**
       * `ReadStorage` failed to write value into the contract's buffer.
       **/
      ReadStorageFailed: AugmentedError<ApiType>;
      /**
       * Extrinsic is not allowed to be called by contracts.
       **/
      RuntimeCallDenied: AugmentedError<ApiType>;
      /**
       * Generic error
       **/
      [key: string]: AugmentedError<ApiType>;
    };
    portfolio: {
      /**
       * The source and destination portfolios should be different.
       **/
      DestinationIsSamePortfolio: AugmentedError<ApiType>;
      /**
       * The portfolios belong to different identities
       **/
      DifferentIdentityPortfolios: AugmentedError<ApiType>;
      /**
       * Trying to move an amount of zero assets.
       **/
      EmptyTransfer: AugmentedError<ApiType>;
      /**
       * Insufficient balance for a transaction.
       **/
      InsufficientPortfolioBalance: AugmentedError<ApiType>;
      /**
       * Can not unlock more tokens than what are locked
       **/
      InsufficientTokensLocked: AugmentedError<ApiType>;
      /**
       * Locked NFTs can not be moved between portfolios.
       **/
      InvalidTransferNFTIsLocked: AugmentedError<ApiType>;
      /**
       * Only owned NFTs can be moved between portfolios.
       **/
      InvalidTransferNFTNotOwned: AugmentedError<ApiType>;
      /**
       * The NFT is already locked.
       **/
      NFTAlreadyLocked: AugmentedError<ApiType>;
      /**
       * The NFT does not exist in the portfolio.
       **/
      NFTNotFoundInPortfolio: AugmentedError<ApiType>;
      /**
       * The NFT has never been locked.
       **/
      NFTNotLocked: AugmentedError<ApiType>;
      /**
       * Duplicate asset among the items.
       **/
      NoDuplicateAssetsAllowed: AugmentedError<ApiType>;
      /**
       * The portfolio doesn't exist.
       **/
      PortfolioDoesNotExist: AugmentedError<ApiType>;
      /**
       * The portfolio couldn't be renamed because the chosen name is already in use.
       **/
      PortfolioNameAlreadyInUse: AugmentedError<ApiType>;
      /**
       * The portfolio still has some asset balance left
       **/
      PortfolioNotEmpty: AugmentedError<ApiType>;
      /**
       * The secondary key is not authorized to access the portfolio(s).
       **/
      SecondaryKeyNotAuthorizedForPortfolio: AugmentedError<ApiType>;
      /**
       * The porfolio's custody is with someone other than the caller.
       **/
      UnauthorizedCustodian: AugmentedError<ApiType>;
      /**
       * Generic error
       **/
      [key: string]: AugmentedError<ApiType>;
    };
    preimage: {
      /**
       * Preimage has already been noted on-chain.
       **/
      AlreadyNoted: AugmentedError<ApiType>;
      /**
       * The user is not authorized to perform this action.
       **/
      NotAuthorized: AugmentedError<ApiType>;
      /**
       * The preimage cannot be removed since it has not yet been noted.
       **/
      NotNoted: AugmentedError<ApiType>;
      /**
       * The preimage request cannot be removed since no outstanding requests exist.
       **/
      NotRequested: AugmentedError<ApiType>;
      /**
       * A preimage may not be removed when there are outstanding requests.
       **/
      Requested: AugmentedError<ApiType>;
      /**
       * Preimage is too large to store on-chain.
       **/
      TooBig: AugmentedError<ApiType>;
      /**
       * Generic error
       **/
      [key: string]: AugmentedError<ApiType>;
    };
    protocolFee: {
      /**
       * Insufficient account balance to pay the fee.
       **/
      InsufficientAccountBalance: AugmentedError<ApiType>;
      /**
       * Insufficient subsidy balance to pay the fee.
       **/
      InsufficientSubsidyBalance: AugmentedError<ApiType>;
      /**
       * Not able to handled the imbalances
       **/
      UnHandledImbalances: AugmentedError<ApiType>;
      /**
       * Generic error
       **/
      [key: string]: AugmentedError<ApiType>;
    };
    relayer: {
      /**
       * The `user_key` doesn't have a `paying_key`.
       **/
      NoPayingKey: AugmentedError<ApiType>;
      /**
       * The signer is not authorized for `paying_key`.
       **/
      NotAuthorizedForPayingKey: AugmentedError<ApiType>;
      /**
       * The signer is not authorized for `user_key`.
       **/
      NotAuthorizedForUserKey: AugmentedError<ApiType>;
      /**
       * The `user_key` has a different `paying_key`.
       **/
      NotPayingKey: AugmentedError<ApiType>;
      /**
       * The remaining POLYX for `user_key` overflowed.
       **/
      Overflow: AugmentedError<ApiType>;
      /**
       * The `user_key` is not attached to a CDD'd identity.
       **/
      PayingKeyCddMissing: AugmentedError<ApiType>;
      /**
       * The `user_key` is not attached to a CDD'd identity.
       **/
      UserKeyCddMissing: AugmentedError<ApiType>;
      /**
       * Generic error
       **/
      [key: string]: AugmentedError<ApiType>;
    };
    rewards: {
      /**
       * Provided signature was invalid.
       **/
      InvalidSignature: AugmentedError<ApiType>;
      /**
       * Itn reward was already claimed.
       **/
      ItnRewardAlreadyClaimed: AugmentedError<ApiType>;
      /**
       * Balance can not be converted to a primitive.
       **/
      UnableToCovertBalance: AugmentedError<ApiType>;
      /**
       * Address was not found in the list of Itn addresses.
       **/
      UnknownItnAddress: AugmentedError<ApiType>;
      /**
       * Generic error
       **/
      [key: string]: AugmentedError<ApiType>;
    };
    scheduler: {
      /**
       * Failed to schedule a call
       **/
      FailedToSchedule: AugmentedError<ApiType>;
      /**
       * Attempt to use a non-named function on a named task.
       **/
      Named: AugmentedError<ApiType>;
      /**
       * Cannot find the scheduled call.
       **/
      NotFound: AugmentedError<ApiType>;
      /**
       * Reschedule failed because it does not change scheduled time.
       **/
      RescheduleNoChange: AugmentedError<ApiType>;
      /**
       * Given target block number is in the past.
       **/
      TargetBlockNumberInPast: AugmentedError<ApiType>;
      /**
       * Generic error
       **/
      [key: string]: AugmentedError<ApiType>;
    };
    session: {
      /**
       * Registered duplicate key.
       **/
      DuplicatedKey: AugmentedError<ApiType>;
      /**
       * Invalid ownership proof.
       **/
      InvalidProof: AugmentedError<ApiType>;
      /**
       * Key setting account is not live, so it's impossible to associate keys.
       **/
      NoAccount: AugmentedError<ApiType>;
      /**
       * No associated validator ID for account.
       **/
      NoAssociatedValidatorId: AugmentedError<ApiType>;
      /**
       * No keys are associated with this account.
       **/
      NoKeys: AugmentedError<ApiType>;
      /**
       * Generic error
       **/
      [key: string]: AugmentedError<ApiType>;
    };
    settlement: {
      /**
       * The caller is not a party of this instruction.
       **/
      CallerIsNotAParty: AugmentedError<ApiType>;
      /**
       * No duplicate uid are allowed for different receipts.
       **/
      DuplicateReceiptUid: AugmentedError<ApiType>;
      /**
       * The instruction failed to release asset locks or transfer the assets.
       **/
      FailedToReleaseLockOrTransferAssets: AugmentedError<ApiType>;
      /**
       * Scheduling of an instruction fails.
       **/
      FailedToSchedule: AugmentedError<ApiType>;
      /**
       * The input weight is less than the minimum required.
       **/
      InputWeightIsLessThanMinimum: AugmentedError<ApiType>;
      /**
       * Instruction has invalid dates
       **/
      InstructionDatesInvalid: AugmentedError<ApiType>;
      /**
       * Instruction has not been affirmed.
       **/
      InstructionNotAffirmed: AugmentedError<ApiType>;
      /**
       * Instruction settlement block has not yet been reached.
       **/
      InstructionSettleBlockNotReached: AugmentedError<ApiType>;
      /**
       * Instruction's target settle block reached.
       **/
      InstructionSettleBlockPassed: AugmentedError<ApiType>;
      /**
       * Only [`InstructionStatus::Pending`] or [`InstructionStatus::Failed`] instructions can be executed.
       **/
      InvalidInstructionStatusForExecution: AugmentedError<ApiType>;
      /**
       * Offchain signature is invalid.
       **/
      InvalidSignature: AugmentedError<ApiType>;
      /**
       * Venue does not exist.
       **/
      InvalidVenue: AugmentedError<ApiType>;
      /**
       * No leg with the given id was found
       **/
      LegNotFound: AugmentedError<ApiType>;
      /**
       * The maximum number of fungible assets was exceeded.
       **/
      MaxNumberOfFungibleAssetsExceeded: AugmentedError<ApiType>;
      /**
       * The number of nfts being transferred in the instruction was exceeded.
       **/
      MaxNumberOfNFTsExceeded: AugmentedError<ApiType>;
      /**
       * The maximum number of off-chain assets was exceeded.
       **/
      MaxNumberOfOffChainAssetsExceeded: AugmentedError<ApiType>;
      /**
       * The maximum number of receipts was exceeded.
       **/
      MaxNumberOfReceiptsExceeded: AugmentedError<ApiType>;
      /**
       * Multiple receipts for the same leg are not allowed.
       **/
      MultipleReceiptsForOneLeg: AugmentedError<ApiType>;
      /**
       * There are parties who have not affirmed the instruction.
       **/
      NotAllAffirmationsHaveBeenReceived: AugmentedError<ApiType>;
      /**
       * The given number of fungible transfers was underestimated.
       **/
      NumberOfFungibleTransfersUnderestimated: AugmentedError<ApiType>;
      /**
       * The given number of off-chain transfers was underestimated.
       **/
      NumberOfOffChainTransfersUnderestimated: AugmentedError<ApiType>;
      /**
       * The given number of nfts being transferred was underestimated.
       **/
      NumberOfTransferredNFTsUnderestimated: AugmentedError<ApiType>;
      /**
       * Off-Chain assets cannot be locked.
       **/
      OffChainAssetCantBeLocked: AugmentedError<ApiType>;
      /**
       * Receipt already used.
       **/
      ReceiptAlreadyClaimed: AugmentedError<ApiType>;
      /**
       * Off-chain receipts can only be used for off-chain leg type.
       **/
      ReceiptForInvalidLegType: AugmentedError<ApiType>;
      /**
       * The instruction id in all receipts must match the extrinsic parameter.
       **/
      ReceiptInstructionIdMissmatch: AugmentedError<ApiType>;
      /**
       * Sender and receiver are the same.
       **/
      SameSenderReceiver: AugmentedError<ApiType>;
      /**
       * The provided settlement block number is in the past and cannot be used by the scheduler.
       **/
      SettleOnPastBlock: AugmentedError<ApiType>;
      /**
       * Signer is already added to venue.
       **/
      SignerAlreadyExists: AugmentedError<ApiType>;
      /**
       * Signer is not added to venue.
       **/
      SignerDoesNotExist: AugmentedError<ApiType>;
      /**
       * Sender does not have required permissions.
       **/
      Unauthorized: AugmentedError<ApiType>;
      /**
       * Signer is not authorized by the venue.
       **/
      UnauthorizedSigner: AugmentedError<ApiType>;
      /**
       * Venue does not have required permissions.
       **/
      UnauthorizedVenue: AugmentedError<ApiType>;
      /**
       * The current instruction affirmation status does not support the requested action.
       **/
      UnexpectedAffirmationStatus: AugmentedError<ApiType>;
      /**
       * An invalid has been reached.
       **/
      UnexpectedLegStatus: AugmentedError<ApiType>;
      /**
       * Ticker could not be found on chain.
       **/
      UnexpectedOFFChainAsset: AugmentedError<ApiType>;
      /**
       * Instruction status is unknown
       **/
      UnknownInstruction: AugmentedError<ApiType>;
      /**
       * The maximum weight limit for executing the function was exceeded.
       **/
      WeightLimitExceeded: AugmentedError<ApiType>;
      /**
       * Instruction leg amount can't be zero.
       **/
      ZeroAmount: AugmentedError<ApiType>;
      /**
       * Generic error
       **/
      [key: string]: AugmentedError<ApiType>;
    };
    staking: {
      /**
       * Stash is already bonded.
       **/
      AlreadyBonded: AugmentedError<ApiType>;
      /**
       * Rewards for this era have already been claimed for this validator.
       **/
      AlreadyClaimed: AugmentedError<ApiType>;
      /**
       * Permissioned validator already exists.
       **/
      AlreadyExists: AugmentedError<ApiType>;
      /**
       * Controller is already paired.
       **/
      AlreadyPaired: AugmentedError<ApiType>;
      /**
       * Internal state has become somehow corrupted and the operation cannot continue.
       **/
      BadState: AugmentedError<ApiType>;
      /**
       * A nomination target was supplied that was blocked or otherwise not a validator.
       **/
      BadTarget: AugmentedError<ApiType>;
      /**
       * When the amount to be bonded is less than `MinimumBond`
       **/
      BondTooSmall: AugmentedError<ApiType>;
      /**
       * The call is not allowed at the given time due to restrictions of election period.
       **/
      CallNotAllowed: AugmentedError<ApiType>;
      /**
       * Targets cannot be empty.
       **/
      EmptyTargets: AugmentedError<ApiType>;
      /**
       * Attempting to target a stash that still has funds.
       **/
      FundedTarget: AugmentedError<ApiType>;
      /**
       * Running validator count hit the intended count.
       **/
      HitIntendedValidatorCount: AugmentedError<ApiType>;
      /**
       * Incorrect number of slashing spans provided.
       **/
      IncorrectSlashingSpans: AugmentedError<ApiType>;
      /**
       * Can not bond with value less than minimum balance.
       **/
      InsufficientValue: AugmentedError<ApiType>;
      /**
       * When the intended number of validators to run is >= 2/3 of `validator_count`.
       **/
      IntendedCountIsExceedingConsensusLimit: AugmentedError<ApiType>;
      /**
       * Invalid era to reward.
       **/
      InvalidEraToReward: AugmentedError<ApiType>;
      /**
       * Slash record index out of bounds.
       **/
      InvalidSlashIndex: AugmentedError<ApiType>;
      /**
       * Validator prefs are not in valid range.
       **/
      InvalidValidatorCommission: AugmentedError<ApiType>;
      /**
       * Given potential validator identity is invalid.
       **/
      InvalidValidatorIdentity: AugmentedError<ApiType>;
      /**
       * Validator should have minimum 50k POLYX bonded.
       **/
      InvalidValidatorUnbondAmount: AugmentedError<ApiType>;
      /**
       * Updates with same value.
       **/
      NoChange: AugmentedError<ApiType>;
      /**
       * Can not schedule more unlock chunks.
       **/
      NoMoreChunks: AugmentedError<ApiType>;
      /**
       * Not a controller account.
       **/
      NotController: AugmentedError<ApiType>;
      /**
       * Permissioned validator not exists.
       **/
      NotExists: AugmentedError<ApiType>;
      /**
       * Items are not sorted and unique.
       **/
      NotSortedAndUnique: AugmentedError<ApiType>;
      /**
       * Not a stash account.
       **/
      NotStash: AugmentedError<ApiType>;
      /**
       * Can not rebond without unlocking chunks.
       **/
      NoUnlockChunk: AugmentedError<ApiType>;
      /**
       * Error while building the assignment type from the compact. This can happen if an index
       * is invalid, or if the weights _overflow_.
       **/
      OffchainElectionBogusCompact: AugmentedError<ApiType>;
      /**
       * The submitted result has unknown edges that are not among the presented winners.
       **/
      OffchainElectionBogusEdge: AugmentedError<ApiType>;
      /**
       * The election size is invalid.
       **/
      OffchainElectionBogusElectionSize: AugmentedError<ApiType>;
      /**
       * One of the submitted nominators has an edge to which they have not voted on chain.
       **/
      OffchainElectionBogusNomination: AugmentedError<ApiType>;
      /**
       * One of the submitted nominators is not an active nominator on chain.
       **/
      OffchainElectionBogusNominator: AugmentedError<ApiType>;
      /**
       * The claimed score does not match with the one computed from the data.
       **/
      OffchainElectionBogusScore: AugmentedError<ApiType>;
      /**
       * A self vote must only be originated from a validator to ONLY themselves.
       **/
      OffchainElectionBogusSelfVote: AugmentedError<ApiType>;
      /**
       * One of the submitted winners is not an active candidate on chain (index is out of range
       * in snapshot).
       **/
      OffchainElectionBogusWinner: AugmentedError<ApiType>;
      /**
       * Incorrect number of winners were presented.
       **/
      OffchainElectionBogusWinnerCount: AugmentedError<ApiType>;
      /**
       * The submitted result is received out of the open window.
       **/
      OffchainElectionEarlySubmission: AugmentedError<ApiType>;
      /**
       * One of the submitted nominators has an edge which is submitted before the last non-zero
       * slash of the target.
       **/
      OffchainElectionSlashedNomination: AugmentedError<ApiType>;
      /**
       * The submitted result is not as good as the one stored on chain.
       **/
      OffchainElectionWeakSubmission: AugmentedError<ApiType>;
      /**
       * The snapshot data of the current window is missing.
       **/
      SnapshotUnavailable: AugmentedError<ApiType>;
      /**
       * Validator or nominator stash identity does not exist.
       **/
      StashIdentityDoesNotExist: AugmentedError<ApiType>;
      /**
       * Nominator stash was not CDDed.
       **/
      StashIdentityNotCDDed: AugmentedError<ApiType>;
      /**
       * Validator stash identity was not permissioned.
       **/
      StashIdentityNotPermissioned: AugmentedError<ApiType>;
      /**
       * Too many nomination targets supplied.
       **/
      TooManyTargets: AugmentedError<ApiType>;
      /**
       * Generic error
       **/
      [key: string]: AugmentedError<ApiType>;
    };
    statistics: {
      /**
       * A Stattype is in use and can't be removed.
       **/
      CannotRemoveStatTypeInUse: AugmentedError<ApiType>;
      /**
       * Transfer not allowed.
       **/
      InvalidTransfer: AugmentedError<ApiType>;
      /**
       * The limit of StatTypes allowed for an asset has been reached.
       **/
      StatTypeLimitReached: AugmentedError<ApiType>;
      /**
       * StatType is not enabled.
       **/
      StatTypeMissing: AugmentedError<ApiType>;
      /**
       * StatType is needed by TransferCondition.
       **/
      StatTypeNeededByTransferCondition: AugmentedError<ApiType>;
      /**
       * The limit of TransferConditions allowed for an asset has been reached.
       **/
      TransferConditionLimitReached: AugmentedError<ApiType>;
      /**
       * The maximum weight limit for executing the function was exceeded.
       **/
      WeightLimitExceeded: AugmentedError<ApiType>;
      /**
       * Generic error
       **/
      [key: string]: AugmentedError<ApiType>;
    };
    sto: {
      /**
       * Fundraiser has been closed/stopped already.
       **/
      FundraiserClosed: AugmentedError<ApiType>;
      /**
       * Interacting with a fundraiser past the end `Moment`.
       **/
      FundraiserExpired: AugmentedError<ApiType>;
      /**
       * Fundraiser not found.
       **/
      FundraiserNotFound: AugmentedError<ApiType>;
      /**
       * Fundraiser is either frozen or stopped.
       **/
      FundraiserNotLive: AugmentedError<ApiType>;
      /**
       * Not enough tokens left for sale.
       **/
      InsufficientTokensRemaining: AugmentedError<ApiType>;
      /**
       * Window (start time, end time) has invalid parameters, e.g start time is after end time.
       **/
      InvalidOfferingWindow: AugmentedError<ApiType>;
      /**
       * An individual price tier was invalid or a set of price tiers was invalid.
       **/
      InvalidPriceTiers: AugmentedError<ApiType>;
      /**
       * An invalid venue provided.
       **/
      InvalidVenue: AugmentedError<ApiType>;
      /**
       * Investment amount is lower than minimum investment amount.
       **/
      InvestmentAmountTooLow: AugmentedError<ApiType>;
      /**
       * Price of the investment exceeded the max price.
       **/
      MaxPriceExceeded: AugmentedError<ApiType>;
      /**
       * An arithmetic operation overflowed.
       **/
      Overflow: AugmentedError<ApiType>;
      /**
       * Sender does not have required permissions.
       **/
      Unauthorized: AugmentedError<ApiType>;
      /**
       * Generic error
       **/
      [key: string]: AugmentedError<ApiType>;
    };
    sudo: {
      /**
       * Sender must be the Sudo account
       **/
      RequireSudo: AugmentedError<ApiType>;
      /**
       * Generic error
       **/
      [key: string]: AugmentedError<ApiType>;
    };
    system: {
      /**
       * The origin filter prevent the call to be dispatched.
       **/
      CallFiltered: AugmentedError<ApiType>;
      /**
       * Failed to extract the runtime version from the new runtime.
       * 
       * Either calling `Core_version` or decoding `RuntimeVersion` failed.
       **/
      FailedToExtractRuntimeVersion: AugmentedError<ApiType>;
      /**
       * The name of specification does not match between the current runtime
       * and the new runtime.
       **/
      InvalidSpecName: AugmentedError<ApiType>;
      /**
       * Suicide called when the account has non-default composite data.
       **/
      NonDefaultComposite: AugmentedError<ApiType>;
      /**
       * There is a non-zero reference count preventing the account from being purged.
       **/
      NonZeroRefCount: AugmentedError<ApiType>;
      /**
       * The specification version is not allowed to decrease between the current runtime
       * and the new runtime.
       **/
      SpecVersionNeedsToIncrease: AugmentedError<ApiType>;
      /**
       * Generic error
       **/
      [key: string]: AugmentedError<ApiType>;
    };
    technicalCommittee: {
      /**
       * Duplicate proposal.
       **/
      DuplicateProposal: AugmentedError<ApiType>;
      /**
       * Duplicate votes are not allowed.
       **/
      DuplicateVote: AugmentedError<ApiType>;
      /**
       * First vote on a proposal creates it, so it must be an approval.
       * All proposals are motions to execute something as "GC majority".
       * To reject e.g., a PIP, a motion to reject should be *approved*.
       **/
      FirstVoteReject: AugmentedError<ApiType>;
      /**
       * Proportion must be a rational number.
       **/
      InvalidProportion: AugmentedError<ApiType>;
      /**
       * Mismatched voting index.
       **/
      MismatchedVotingIndex: AugmentedError<ApiType>;
      /**
       * No such proposal.
       **/
      NoSuchProposal: AugmentedError<ApiType>;
      /**
       * A DID isn't part of the committee.
       * The DID may either be a caller or some other context.
       **/
      NotAMember: AugmentedError<ApiType>;
      /**
       * Proposal exists, but it has expired.
       **/
      ProposalExpired: AugmentedError<ApiType>;
      /**
       * Maximum number of proposals has been reached.
       **/
      ProposalsLimitReached: AugmentedError<ApiType>;
      /**
       * Generic error
       **/
      [key: string]: AugmentedError<ApiType>;
    };
    technicalCommitteeMembership: {
      /**
       * The limit for the number of concurrent active members for this group has been exceeded.
       **/
      ActiveMembersLimitExceeded: AugmentedError<ApiType>;
      /**
       * Active member limit was greater than maximum committee members limit.
       **/
      ActiveMembersLimitOverflow: AugmentedError<ApiType>;
      /**
       * Group member was added already.
       **/
      DuplicateMember: AugmentedError<ApiType>;
      /**
       * Last member of the committee can not quit.
       **/
      LastMemberCannotQuit: AugmentedError<ApiType>;
      /**
       * Missing current DID
       **/
      MissingCurrentIdentity: AugmentedError<ApiType>;
      /**
       * Can't remove a member that doesn't exist.
       **/
      NoSuchMember: AugmentedError<ApiType>;
      /**
       * Only primary key of the identity is allowed.
       **/
      OnlyPrimaryKeyAllowed: AugmentedError<ApiType>;
      /**
       * Generic error
       **/
      [key: string]: AugmentedError<ApiType>;
    };
    testUtils: {
      /**
       * Generic error
       **/
      [key: string]: AugmentedError<ApiType>;
    };
    treasury: {
      /**
       * Proposer's balance is too low.
       **/
      InsufficientBalance: AugmentedError<ApiType>;
      /**
       * Invalid identity for disbursement.
       **/
      InvalidIdentity: AugmentedError<ApiType>;
      /**
       * Generic error
       **/
      [key: string]: AugmentedError<ApiType>;
    };
    upgradeCommittee: {
      /**
       * Duplicate proposal.
       **/
      DuplicateProposal: AugmentedError<ApiType>;
      /**
       * Duplicate votes are not allowed.
       **/
      DuplicateVote: AugmentedError<ApiType>;
      /**
       * First vote on a proposal creates it, so it must be an approval.
       * All proposals are motions to execute something as "GC majority".
       * To reject e.g., a PIP, a motion to reject should be *approved*.
       **/
      FirstVoteReject: AugmentedError<ApiType>;
      /**
       * Proportion must be a rational number.
       **/
      InvalidProportion: AugmentedError<ApiType>;
      /**
       * Mismatched voting index.
       **/
      MismatchedVotingIndex: AugmentedError<ApiType>;
      /**
       * No such proposal.
       **/
      NoSuchProposal: AugmentedError<ApiType>;
      /**
       * A DID isn't part of the committee.
       * The DID may either be a caller or some other context.
       **/
      NotAMember: AugmentedError<ApiType>;
      /**
       * Proposal exists, but it has expired.
       **/
      ProposalExpired: AugmentedError<ApiType>;
      /**
       * Maximum number of proposals has been reached.
       **/
      ProposalsLimitReached: AugmentedError<ApiType>;
      /**
       * Generic error
       **/
      [key: string]: AugmentedError<ApiType>;
    };
    upgradeCommitteeMembership: {
      /**
       * The limit for the number of concurrent active members for this group has been exceeded.
       **/
      ActiveMembersLimitExceeded: AugmentedError<ApiType>;
      /**
       * Active member limit was greater than maximum committee members limit.
       **/
      ActiveMembersLimitOverflow: AugmentedError<ApiType>;
      /**
       * Group member was added already.
       **/
      DuplicateMember: AugmentedError<ApiType>;
      /**
       * Last member of the committee can not quit.
       **/
      LastMemberCannotQuit: AugmentedError<ApiType>;
      /**
       * Missing current DID
       **/
      MissingCurrentIdentity: AugmentedError<ApiType>;
      /**
       * Can't remove a member that doesn't exist.
       **/
      NoSuchMember: AugmentedError<ApiType>;
      /**
       * Only primary key of the identity is allowed.
       **/
      OnlyPrimaryKeyAllowed: AugmentedError<ApiType>;
      /**
       * Generic error
       **/
      [key: string]: AugmentedError<ApiType>;
    };
    utility: {
      /**
       * Provided nonce was invalid
       * If the provided nonce < current nonce, the call was already executed
       * If the provided nonce > current nonce, the call(s) before the current failed to execute
       * POLYMESH error
       **/
      InvalidNonce: AugmentedError<ApiType>;
      /**
       * Offchain signature is invalid
       * POLYMESH error
       **/
      InvalidSignature: AugmentedError<ApiType>;
      /**
       * Target does not have a valid CDD
       * POLYMESH error
       **/
      TargetCddMissing: AugmentedError<ApiType>;
      /**
       * Too many calls batched.
       **/
      TooManyCalls: AugmentedError<ApiType>;
      /**
       * Generic error
       **/
      [key: string]: AugmentedError<ApiType>;
    };
  } // AugmentedErrors
} // declare module
