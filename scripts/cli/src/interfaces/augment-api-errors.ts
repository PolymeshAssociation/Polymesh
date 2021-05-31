// Auto-generated via `yarn polkadot-types-from-chain`, do not edit
/* eslint-disable */

import type { ApiTypes } from '@polkadot/api/types';

declare module '@polkadot/api/types/errors' {
  export interface AugmentedErrors<ApiType> {
    asset: {
      [key: string]: AugmentedError<ApiType>;
      /**
       * When extension already archived.
       **/
      AlreadyArchived: AugmentedError<ApiType>;
      /**
       * The token is already frozen.
       **/
      AlreadyFrozen: AugmentedError<ApiType>;
      /**
       * When extension already un-archived.
       **/
      AlreadyUnArchived: AugmentedError<ApiType>;
      /**
       * The token has already been created.
       **/
      AssetAlreadyCreated: AugmentedError<ApiType>;
      /**
       * The token is already divisible.
       **/
      AssetAlreadyDivisible: AugmentedError<ApiType>;
      /**
       * An overflow while calculating the balance.
       **/
      BalanceOverflow: AugmentedError<ApiType>;
      /**
       * When extension is already added.
       **/
      ExtensionAlreadyPresent: AugmentedError<ApiType>;
      /**
       * Maximum length of the funding round name has been exceeded.
       **/
      FundingRoundNameMaxLengthExceeded: AugmentedError<ApiType>;
      /**
       * Given smart extension is not compatible with the asset.
       **/
      IncompatibleExtensionVersion: AugmentedError<ApiType>;
      /**
       * The sender balance is not sufficient.
       **/
      InsufficientBalance: AugmentedError<ApiType>;
      /**
       * An invalid Ethereum `EcdsaSignature`.
       **/
      InvalidEthereumSignature: AugmentedError<ApiType>;
      /**
       * An invalid granularity.
       **/
      InvalidGranularity: AugmentedError<ApiType>;
      /**
       * An invalid total supply.
       **/
      InvalidTotalSupply: AugmentedError<ApiType>;
      /**
       * Transfer validation check failed.
       **/
      InvalidTransfer: AugmentedError<ApiType>;
      /**
       * Number of Transfer Manager extensions attached to an asset is equal to MaxNumberOfTMExtensionForAsset.
       **/
      MaximumTMExtensionLimitReached: AugmentedError<ApiType>;
      /**
       * Maximum length of asset name has been exceeded.
       **/
      MaxLengthOfAssetNameExceeded: AugmentedError<ApiType>;
      /**
       * Not a primary issuance agent transfer auth.
       **/
      NoPrimaryIssuanceAgentTransferAuth: AugmentedError<ApiType>;
      /**
       * No such token.
       **/
      NoSuchAsset: AugmentedError<ApiType>;
      /**
       * The given ticker is not a classic one.
       **/
      NoSuchClassicTicker: AugmentedError<ApiType>;
      /**
       * The given Document does not exist.
       **/
      NoSuchDoc: AugmentedError<ApiType>;
      /**
       * No such smart extension.
       **/
      NoSuchSmartExtension: AugmentedError<ApiType>;
      /**
       * Not an owner of the token.
       **/
      NotAnOwner: AugmentedError<ApiType>;
      /**
       * The asset must be frozen.
       **/
      NotFrozen: AugmentedError<ApiType>;
      /**
       * Not a ticker transfer auth.
       **/
      NoTickerTransferAuth: AugmentedError<ApiType>;
      /**
       * Not a token ownership transfer auth.
       **/
      NotTickerOwnershipTransferAuth: AugmentedError<ApiType>;
      /**
       * The secondary key does not have the required Asset permission
       **/
      SecondaryKeyNotAuthorizedForAsset: AugmentedError<ApiType>;
      /**
       * Transfers to self are not allowed
       **/
      SenderSameAsReceiver: AugmentedError<ApiType>;
      /**
       * The ticker is already registered to someone else.
       **/
      TickerAlreadyRegistered: AugmentedError<ApiType>;
      /**
       * The ticker has non-ascii-encoded parts.
       **/
      TickerNotAscii: AugmentedError<ApiType>;
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
    };
    authorship: {
      [key: string]: AugmentedError<ApiType>;
      /**
       * The uncle is genesis.
       **/
      GenesisUncle: AugmentedError<ApiType>;
      /**
       * The uncle parent not in the chain.
       **/
      InvalidUncleParent: AugmentedError<ApiType>;
      /**
       * The uncle isn't recent enough to be included.
       **/
      OldUncle: AugmentedError<ApiType>;
      /**
       * The uncle is too high in chain.
       **/
      TooHighUncle: AugmentedError<ApiType>;
      /**
       * Too many uncles.
       **/
      TooManyUncles: AugmentedError<ApiType>;
      /**
       * The uncle is already included.
       **/
      UncleAlreadyIncluded: AugmentedError<ApiType>;
      /**
       * Uncles already set in the block.
       **/
      UnclesAlreadySet: AugmentedError<ApiType>;
    };
    balances: {
      [key: string]: AugmentedError<ApiType>;
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
    };
    baseContracts: {
      [key: string]: AugmentedError<ApiType>;
      /**
       * Performing the requested transfer would have brought the contract below
       * the subsistence threshold. No transfer is allowed to do this in order to allow
       * for a tombstone to be created. Use `seal_terminate` to remove a contract without
       * leaving a tombstone behind.
       **/
      BelowSubsistenceThreshold: AugmentedError<ApiType>;
      /**
       * No code could be found at the supplied code hash.
       **/
      CodeNotFound: AugmentedError<ApiType>;
      /**
       * The code supplied to `put_code` exceeds the limit specified in the current schedule.
       **/
      CodeTooLarge: AugmentedError<ApiType>;
      /**
       * Contract trapped during execution.
       **/
      ContractTrapped: AugmentedError<ApiType>;
      /**
       * Input passed to a contract API function failed to decode as expected type.
       **/
      DecodingFailed: AugmentedError<ApiType>;
      /**
       * An origin TrieId written in the current block.
       **/
      InvalidContractOrigin: AugmentedError<ApiType>;
      /**
       * Cannot restore to nonexisting or alive contract.
       **/
      InvalidDestinationContract: AugmentedError<ApiType>;
      /**
       * A new schedule must have a greater version than the current one.
       **/
      InvalidScheduleVersion: AugmentedError<ApiType>;
      /**
       * Cannot restore from nonexisting or tombstone contract.
       **/
      InvalidSourceContract: AugmentedError<ApiType>;
      /**
       * An origin must be signed or inherent and auxiliary sender only provided on inherent.
       **/
      InvalidSurchargeClaim: AugmentedError<ApiType>;
      /**
       * Tombstones don't match.
       **/
      InvalidTombstone: AugmentedError<ApiType>;
      /**
       * Performing a call was denied because the calling depth reached the limit
       * of what is specified in the schedule.
       **/
      MaxCallDepthReached: AugmentedError<ApiType>;
      /**
       * The newly created contract is below the subsistence threshold after executing
       * its contructor. No contracts are allowed to exist below that threshold.
       **/
      NewContractNotFunded: AugmentedError<ApiType>;
      /**
       * The contract that was called is either no contract at all (a plain account)
       * or is a tombstone.
       **/
      NotCallable: AugmentedError<ApiType>;
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
       * Performing the requested transfer failed for a reason originating in the
       * chosen currency implementation of the runtime. Most probably the balance is
       * too low or locks are placed on it.
       **/
      TransferFailed: AugmentedError<ApiType>;
    };
    bridge: {
      [key: string]: AugmentedError<ApiType>;
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
    };
    capitalDistribution: {
      [key: string]: AugmentedError<ApiType>;
      /**
       * A distribution already exists for this CA.
       **/
      AlreadyExists: AugmentedError<ApiType>;
      /**
       * DID who created the distribution already did reclaim.
       **/
      AlreadyReclaimed: AugmentedError<ApiType>;
      /**
       * Multiplication of the balance with the total payout amount overflowed.
       **/
      BalanceAmountProductOverflowed: AugmentedError<ApiType>;
      /**
       * A failed division of the balance amount product by the total supply.
       **/
      BalanceAmountProductSupplyDivisionFailed: AugmentedError<ApiType>;
      /**
       * Distribution's expiry has passed. DID cannot claim anymore and has forfeited the benefits.
       **/
      CannotClaimAfterExpiry: AugmentedError<ApiType>;
      /**
       * Distribution allotment cannot be claimed as the current time is before start-of-payment.
       **/
      CannotClaimBeforeStart: AugmentedError<ApiType>;
      /**
       * A corporate ballot was made for a non-benefit CA.
       **/
      CANotBenefit: AugmentedError<ApiType>;
      /**
       * Currency that is distributed is the same as the CA's ticker.
       * CAA is attempting a form of stock split, which is not what the extrinsic is for.
       **/
      DistributingAsset: AugmentedError<ApiType>;
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
    };
    cddServiceProviders: {
      [key: string]: AugmentedError<ApiType>;
      /**
       * The limit for the number of concurrent active members for this group has been exceeded.
       **/
      ActiveMembersLimitExceeded: AugmentedError<ApiType>;
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
    };
    checkpoint: {
      [key: string]: AugmentedError<ApiType>;
      /**
       * An overflow while calculating the checkpoint ID.
       **/
      CheckpointOverflow: AugmentedError<ApiType>;
      /**
       * Failed to compute the next checkpoint.
       * The schedule does not have any upcoming checkpoints.
       **/
      FailedToComputeNextCheckpoint: AugmentedError<ApiType>;
      /**
       * A checkpoint schedule does not exist for the asset.
       **/
      NoSuchSchedule: AugmentedError<ApiType>;
      /**
       * The duration of a schedule period is too short.
       **/
      ScheduleDurationTooShort: AugmentedError<ApiType>;
      /**
       * A checkpoint schedule is not removable as `ref_count(schedule_id) > 0`.
       **/
      ScheduleNotRemovable: AugmentedError<ApiType>;
      /**
       * An overflow while calculating the checkpoint schedule ID.
       **/
      ScheduleOverflow: AugmentedError<ApiType>;
      /**
       * The set of schedules taken together are too complex.
       * For example, they are too many, or they occurs too frequently.
       **/
      SchedulesTooComplex: AugmentedError<ApiType>;
    };
    committeeMembership: {
      [key: string]: AugmentedError<ApiType>;
      /**
       * The limit for the number of concurrent active members for this group has been exceeded.
       **/
      ActiveMembersLimitExceeded: AugmentedError<ApiType>;
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
    };
    complianceManager: {
      [key: string]: AugmentedError<ApiType>;
      /**
       * The worst case scenario of the compliance requirement is too complex
       **/
      ComplianceRequirementTooComplex: AugmentedError<ApiType>;
      /**
       * Did not exist
       **/
      DidNotExist: AugmentedError<ApiType>;
      /**
       * There are duplicate compliance requirements.
       **/
      DuplicateComplianceRequirements: AugmentedError<ApiType>;
      /**
       * Issuer exist but trying to add it again
       **/
      IncorrectOperationOnTrustedIssuer: AugmentedError<ApiType>;
      /**
       * Compliance requirement id doesn't exist
       **/
      InvalidComplianceRequirementId: AugmentedError<ApiType>;
      /**
       * User is not authorized.
       **/
      Unauthorized: AugmentedError<ApiType>;
    };
    contracts: {
      [key: string]: AugmentedError<ApiType>;
      /**
       * When instantiation of the template is already frozen.
       **/
      InstantiationAlreadyFrozen: AugmentedError<ApiType>;
      /**
       * When instantiation of the template is already un-frozen.
       **/
      InstantiationAlreadyUnFrozen: AugmentedError<ApiType>;
      /**
       * Instantiation is not allowed.
       **/
      InstantiationIsNotAllowed: AugmentedError<ApiType>;
      /**
       * Insufficient max_fee provided by the user to instantiate the SE.
       **/
      InsufficientMaxFee: AugmentedError<ApiType>;
      /**
       * Given identityId is not CDD.
       **/
      NewOwnerIsNotCDD: AugmentedError<ApiType>;
      /**
       * `put_code` extrinsic is disabled. See `set_put_code_flag` extrinsic.
       **/
      PutCodeIsNotAllowed: AugmentedError<ApiType>;
      /**
       * Smart extension template not exist in the storage.
       **/
      TemplateNotExists: AugmentedError<ApiType>;
      /**
       * When un-authorized personnel try to access the un-authorized extrinsic.
       **/
      UnAuthorizedOrigin: AugmentedError<ApiType>;
    };
    corporateAction: {
      [key: string]: AugmentedError<ApiType>;
      /**
       * The authorization type is not to transfer the CAA to another DID.
       **/
      AuthNotCAATransfer: AugmentedError<ApiType>;
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
       * There have been too many CAs for this ticker and the ID would overflow.
       * This won't occur in practice.
       **/
      LocalCAIdOverflow: AugmentedError<ApiType>;
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
       * The signer is not authorized to act as a CAA for this asset.
       **/
      UnauthorizedAsAgent: AugmentedError<ApiType>;
    };
    corporateBallot: {
      [key: string]: AugmentedError<ApiType>;
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
    };
    finalityTracker: {
      [key: string]: AugmentedError<ApiType>;
      /**
       * Final hint must be updated only once in the block
       **/
      AlreadyUpdated: AugmentedError<ApiType>;
      /**
       * Finalized height above block number
       **/
      BadHint: AugmentedError<ApiType>;
    };
    grandpa: {
      [key: string]: AugmentedError<ApiType>;
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
    };
    identity: {
      [key: string]: AugmentedError<ApiType>;
      /**
       * One secondary key can only belong to one DID
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
       * Try to add a claim variant using un-designated extrinsic.
       **/
      ClaimVariantNotAllowed: AugmentedError<ApiType>;
      /**
       * Confidential Scope claims can be added by an Identity to it-self.
       **/
      ConfidentialScopeClaimNotAllowed: AugmentedError<ApiType>;
      /**
       * Current identity cannot be forwarded, it is not a secondary key of target identity.
       **/
      CurrentIdentityCannotBeForwarded: AugmentedError<ApiType>;
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
       * Addition of a new scope claim gets invalidated.
       **/
      InvalidScopeClaim: AugmentedError<ApiType>;
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
       * Signer is not a secondary key of the provided identity
       **/
      NotASigner: AugmentedError<ApiType>;
      /**
       * Attestation was not by a CDD service provider.
       **/
      NotCddProviderAttestation: AugmentedError<ApiType>;
      /**
       * Only the primary key is allowed to revoke an Identity Signatory off-chain authorization.
       **/
      NotPrimaryKey: AugmentedError<ApiType>;
      /**
       * Do not allow forwarded call to be called recursively
       **/
      RecursionNotAllowed: AugmentedError<ApiType>;
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
       * Given authorization is not pre-known
       **/
      UnknownAuthorization: AugmentedError<ApiType>;
    };
    imOnline: {
      [key: string]: AugmentedError<ApiType>;
      /**
       * Duplicated heartbeat.
       **/
      DuplicatedHeartbeat: AugmentedError<ApiType>;
      /**
       * Non existent public key.
       **/
      InvalidKey: AugmentedError<ApiType>;
    };
    multiSig: {
      [key: string]: AugmentedError<ApiType>;
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
       * Not a multisig authorization.
       **/
      NotAMultisigAuth: AugmentedError<ApiType>;
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
       * Signer is an account key that is already associated with a multisig.
       **/
      SignerAlreadyLinked: AugmentedError<ApiType>;
      /**
       * More signers than required.
       **/
      TooManySigners: AugmentedError<ApiType>;
    };
    permissions: {
      [key: string]: AugmentedError<ApiType>;
      RecursionNotAllowed: AugmentedError<ApiType>;
      /**
       * The caller is not authorized to call the current extrinsic.
       **/
      UnauthorizedCaller: AugmentedError<ApiType>;
    };
    pips: {
      [key: string]: AugmentedError<ApiType>;
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
    };
    polymeshCommittee: {
      [key: string]: AugmentedError<ApiType>;
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
    };
    portfolio: {
      [key: string]: AugmentedError<ApiType>;
      /**
       * The source and destination portfolios should be different.
       **/
      DestinationIsSamePortfolio: AugmentedError<ApiType>;
      /**
       * The portfolios belong to different identities
       **/
      DifferentIdentityPortfolios: AugmentedError<ApiType>;
      /**
       * Insufficient balance for a transaction.
       **/
      InsufficientPortfolioBalance: AugmentedError<ApiType>;
      /**
       * Can not unlock more tokens than what are locked
       **/
      InsufficientTokensLocked: AugmentedError<ApiType>;
      /**
       * The authorization is for something other than portfolio custody
       **/
      IrrelevantAuthorization: AugmentedError<ApiType>;
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
    };
    protocolFee: {
      [key: string]: AugmentedError<ApiType>;
      /**
       * Insufficient account balance to pay the fee.
       **/
      InsufficientAccountBalance: AugmentedError<ApiType>;
      /**
       * Not able to handled the imbalances
       **/
      UnHandledImbalances: AugmentedError<ApiType>;
    };
    scheduler: {
      [key: string]: AugmentedError<ApiType>;
      /**
       * Failed to cancel a scheduled call
       **/
      FailedToCancel: AugmentedError<ApiType>;
      /**
       * Failed to schedule a call
       **/
      FailedToSchedule: AugmentedError<ApiType>;
      /**
       * Given target block number is in the past.
       **/
      TargetBlockNumberInPast: AugmentedError<ApiType>;
    };
    session: {
      [key: string]: AugmentedError<ApiType>;
      /**
       * Registered duplicate key.
       **/
      DuplicatedKey: AugmentedError<ApiType>;
      /**
       * Invalid ownership proof.
       **/
      InvalidProof: AugmentedError<ApiType>;
      /**
       * No associated validator ID for account.
       **/
      NoAssociatedValidatorId: AugmentedError<ApiType>;
      /**
       * No keys are associated with this account.
       **/
      NoKeys: AugmentedError<ApiType>;
    };
    settlement: {
      [key: string]: AugmentedError<ApiType>;
      /**
       * While affirming the transfer, system failed to lock the assets involved.
       **/
      FailedToLockTokens: AugmentedError<ApiType>;
      /**
       * Scheduling of an instruction fails.
       **/
      FailedToSchedule: AugmentedError<ApiType>;
      /**
       * Instruction has invalid dates
       **/
      InstructionDatesInvalid: AugmentedError<ApiType>;
      /**
       * Instruction failed to execute.
       **/
      InstructionFailed: AugmentedError<ApiType>;
      /**
       * Instruction has not been affirmed.
       **/
      InstructionNotAffirmed: AugmentedError<ApiType>;
      /**
       * Provided instruction is not pending execution.
       **/
      InstructionNotPending: AugmentedError<ApiType>;
      /**
       * Instruction's target settle block reached.
       **/
      InstructionSettleBlockPassed: AugmentedError<ApiType>;
      /**
       * Offchain signature is invalid.
       **/
      InvalidSignature: AugmentedError<ApiType>;
      /**
       * Venue does not exist.
       **/
      InvalidVenue: AugmentedError<ApiType>;
      /**
       * Legs count should matches with the total number of legs in which given portfolio act as `from_portfolio`.
       **/
      LegCountTooSmall: AugmentedError<ApiType>;
      /**
       * Provided leg is not pending execution.
       **/
      LegNotPending: AugmentedError<ApiType>;
      /**
       * No pending affirmation for the provided instruction.
       **/
      NoPendingAffirm: AugmentedError<ApiType>;
      /**
       * Portfolio based actions require at least one portfolio to be provided as input.
       **/
      NoPortfolioProvided: AugmentedError<ApiType>;
      /**
       * Portfolio in receipt does not match with portfolios provided by the user.
       **/
      PortfolioMismatch: AugmentedError<ApiType>;
      /**
       * Receipt already used.
       **/
      ReceiptAlreadyClaimed: AugmentedError<ApiType>;
      /**
       * Receipt not used yet.
       **/
      ReceiptNotClaimed: AugmentedError<ApiType>;
      /**
       * Sender and receiver are the same.
       **/
      SameSenderReceiver: AugmentedError<ApiType>;
      /**
       * The provided settlement block number is in the past and cannot be used by the scheduler.
       **/
      SettleOnPastBlock: AugmentedError<ApiType>;
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
    };
    staking: {
      [key: string]: AugmentedError<ApiType>;
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
       * Validator stash identity was not permissioned.
       **/
      StashIdentityNotPermissioned: AugmentedError<ApiType>;
    };
    statistics: {
      [key: string]: AugmentedError<ApiType>;
      /**
       * The transfer manager already exists
       **/
      DuplicateTransferManager: AugmentedError<ApiType>;
      /**
       * Transfer not allowed
       **/
      InvalidTransfer: AugmentedError<ApiType>;
      /**
       * Transfer manager is not enabled
       **/
      TransferManagerMissing: AugmentedError<ApiType>;
      /**
       * The limit of transfer managers allowed for an asset has been reached
       **/
      TransferManagersLimitReached: AugmentedError<ApiType>;
    };
    sto: {
      [key: string]: AugmentedError<ApiType>;
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
    };
    sudo: {
      [key: string]: AugmentedError<ApiType>;
      /**
       * Sender must be the Sudo account
       **/
      RequireSudo: AugmentedError<ApiType>;
    };
    system: {
      [key: string]: AugmentedError<ApiType>;
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
    };
    technicalCommittee: {
      [key: string]: AugmentedError<ApiType>;
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
    };
    technicalCommitteeMembership: {
      [key: string]: AugmentedError<ApiType>;
      /**
       * The limit for the number of concurrent active members for this group has been exceeded.
       **/
      ActiveMembersLimitExceeded: AugmentedError<ApiType>;
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
    };
    treasury: {
      [key: string]: AugmentedError<ApiType>;
      /**
       * Proposer's balance is too low.
       **/
      InsufficientBalance: AugmentedError<ApiType>;
    };
    upgradeCommittee: {
      [key: string]: AugmentedError<ApiType>;
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
    };
    upgradeCommitteeMembership: {
      [key: string]: AugmentedError<ApiType>;
      /**
       * The limit for the number of concurrent active members for this group has been exceeded.
       **/
      ActiveMembersLimitExceeded: AugmentedError<ApiType>;
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
    };
    utility: {
      [key: string]: AugmentedError<ApiType>;
      /**
       * Provided nonce was invalid
       * If the provided nonce < current nonce, the call was already executed
       * If the provided nonce > current nonce, the call(s) before the current failed to execute
       **/
      InvalidNonce: AugmentedError<ApiType>;
      /**
       * Offchain signature is invalid
       **/
      InvalidSignature: AugmentedError<ApiType>;
      /**
       * Target does not have a valid CDD
       **/
      TargetCddMissing: AugmentedError<ApiType>;
    };
  }

  export interface DecoratedErrors<ApiType extends ApiTypes> extends AugmentedErrors<ApiType> {
    [key: string]: ModuleErrors<ApiType>;
  }
}
