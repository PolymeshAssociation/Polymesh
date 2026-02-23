// Auto-generated via `yarn polkadot-types-from-chain`, do not edit
/* eslint-disable */

// import type lookup before we augment - in some environments
// this is required to allow for ambient/previous definitions
import '@polkadot/api-base/types/consts';

import type { ApiTypes, AugmentedConst } from '@polkadot/api-base/types';
import type { bool, u128, u16, u32, u64, u8 } from '@polkadot/types-codec';
import type { Codec } from '@polkadot/types-codec/types';
import type { Perbill, Permill } from '@polkadot/types/interfaces/runtime';
import type { FrameSystemLimitsBlockLength, FrameSystemLimitsBlockWeights, PalletContractsEnvironment, PalletContractsSchedule, SpVersionRuntimeVersion, SpWeightsRuntimeDbWeight, SpWeightsWeightV2Weight } from '@polkadot/types/lookup';

export type __AugmentedConst<ApiType extends ApiTypes> = AugmentedConst<ApiType>;

declare module '@polkadot/api-base/types/consts' {
  interface AugmentedConsts<ApiType extends ApiTypes> {
    asset: {
      /**
       * Max length for the Asset Metadata type name.
       **/
      assetMetadataNameMaxLength: u32 & AugmentedConst<ApiType>;
      /**
       * Max length for the Asset Metadata type definition.
       **/
      assetMetadataTypeDefMaxLength: u32 & AugmentedConst<ApiType>;
      /**
       * Max length for the Asset Metadata value.
       **/
      assetMetadataValueMaxLength: u32 & AugmentedConst<ApiType>;
      /**
       * Max length for the name of an asset.
       **/
      assetNameMaxLength: u32 & AugmentedConst<ApiType>;
      /**
       * Max length of the funding round name.
       **/
      fundingRoundNameMaxLength: u32 & AugmentedConst<ApiType>;
      /**
       * Maximum number of mediators for an asset.
       **/
      maxAssetMediators: u32 & AugmentedConst<ApiType>;
      /**
       * Generic const
       **/
      [key: string]: Codec;
    };
    babe: {
      /**
       * The amount of time, in slots, that each epoch should last.
       * NOTE: Currently it is not possible to change the epoch duration after
       * the chain has started. Attempting to do so will brick block production.
       **/
      epochDuration: u64 & AugmentedConst<ApiType>;
      /**
       * The expected average block time at which BABE should be creating
       * blocks. Since BABE is probabilistic it is not trivial to figure out
       * what the expected average block time should be based on the slot
       * duration and the security parameter `c` (where `1 - c` represents
       * the probability of a slot being empty).
       **/
      expectedBlockTime: u64 & AugmentedConst<ApiType>;
      /**
       * Max number of authorities allowed
       **/
      maxAuthorities: u32 & AugmentedConst<ApiType>;
      /**
       * The maximum number of nominators for each validator.
       **/
      maxNominators: u32 & AugmentedConst<ApiType>;
      /**
       * Generic const
       **/
      [key: string]: Codec;
    };
    balances: {
      /**
       * The minimum amount required to keep an account open. MUST BE GREATER THAN ZERO!
       * 
       * If you *really* need it to be zero, you can enable the feature `insecure_zero_ed` for
       * this pallet. However, you do so at your own risk: this will open up a major DoS vector.
       * In case you have multiple sources of provider references, you may also get unexpected
       * behaviour if you set this to zero.
       * 
       * Bottom line: Do yourself a favour and make it at least one!
       **/
      existentialDeposit: u128 & AugmentedConst<ApiType>;
      /**
       * The maximum number of individual freeze locks that can exist on an account at any time.
       **/
      maxFreezes: u32 & AugmentedConst<ApiType>;
      /**
       * The maximum number of locks that should exist on an account.
       * Not strictly enforced, but used for weight estimation.
       * 
       * Use of locks is deprecated in favour of freezes. See `https://github.com/paritytech/substrate/pull/12951/`
       **/
      maxLocks: u32 & AugmentedConst<ApiType>;
      /**
       * The maximum number of named reserves that can exist on an account.
       * 
       * Use of reserves is deprecated in favour of holds. See `https://github.com/paritytech/substrate/pull/12951/`
       **/
      maxReserves: u32 & AugmentedConst<ApiType>;
      /**
       * Generic const
       **/
      [key: string]: Codec;
    };
    base: {
      /**
       * The maximum length governing `TooLong`.
       * 
       * How lengths are computed to compare against this value is situation based.
       * For example, you could halve it, double it, compute a sum for some tree of strings, etc.
       **/
      maxLen: u32 & AugmentedConst<ApiType>;
      /**
       * Generic const
       **/
      [key: string]: Codec;
    };
    beefy: {
      /**
       * The maximum number of authorities that can be added.
       **/
      maxAuthorities: u32 & AugmentedConst<ApiType>;
      /**
       * The maximum number of nominators for each validator.
       **/
      maxNominators: u32 & AugmentedConst<ApiType>;
      /**
       * The maximum number of entries to keep in the set id to session index mapping.
       * 
       * Since the `SetIdSession` map is only used for validating equivocations this
       * value should relate to the bonding duration of whatever staking system is
       * being used (if any). If equivocation handling is not enabled then this value
       * can be zero.
       **/
      maxSetIdSessionEntries: u64 & AugmentedConst<ApiType>;
      /**
       * Generic const
       **/
      [key: string]: Codec;
    };
    complianceManager: {
      /**
       * The maximum claim reads that are allowed to happen in worst case of a condition resolution
       **/
      maxConditionComplexity: u32 & AugmentedConst<ApiType>;
      /**
       * Generic const
       **/
      [key: string]: Codec;
    };
    contracts: {
      /**
       * The version of the HostFn APIs that are available in the runtime.
       * 
       * Only valid value is `()`.
       **/
      apiVersion: u16 & AugmentedConst<ApiType>;
      /**
       * The percentage of the storage deposit that should be held for using a code hash.
       * Instantiating a contract, or calling [`chain_extension::Ext::lock_delegate_dependency`]
       * protects the code from being removed. In order to prevent abuse these actions are
       * protected with a percentage of the code deposit.
       **/
      codeHashLockupDepositPercent: Perbill & AugmentedConst<ApiType>;
      /**
       * Fallback value to limit the storage deposit if it's not being set by the caller.
       **/
      defaultDepositLimit: u128 & AugmentedConst<ApiType>;
      /**
       * The amount of balance a caller has to pay for each byte of storage.
       * 
       * # Note
       * 
       * Changing this value for an existing chain might need a storage migration.
       **/
      depositPerByte: u128 & AugmentedConst<ApiType>;
      /**
       * The amount of balance a caller has to pay for each storage item.
       * 
       * # Note
       * 
       * Changing this value for an existing chain might need a storage migration.
       **/
      depositPerItem: u128 & AugmentedConst<ApiType>;
      /**
       * Type that bundles together all the runtime configurable interface types.
       * 
       * This is not a real config. We just mention the type here as constant so that
       * its type appears in the metadata. Only valid value is `()`.
       **/
      environment: PalletContractsEnvironment & AugmentedConst<ApiType>;
      /**
       * The maximum length of a contract code in bytes.
       * 
       * The value should be chosen carefully taking into the account the overall memory limit
       * your runtime has, as well as the [maximum allowed callstack
       * depth](#associatedtype.CallStack). Look into the `integrity_test()` for some insights.
       **/
      maxCodeLen: u32 & AugmentedConst<ApiType>;
      /**
       * The maximum length of the debug buffer in bytes.
       **/
      maxDebugBufferLen: u32 & AugmentedConst<ApiType>;
      /**
       * The maximum number of delegate_dependencies that a contract can lock with
       * [`chain_extension::Ext::lock_delegate_dependency`].
       **/
      maxDelegateDependencies: u32 & AugmentedConst<ApiType>;
      /**
       * The maximum allowable length in bytes for storage keys.
       **/
      maxStorageKeyLen: u32 & AugmentedConst<ApiType>;
      /**
       * The maximum size of the transient storage in bytes.
       * This includes keys, values, and previous entries used for storage rollback.
       **/
      maxTransientStorageSize: u32 & AugmentedConst<ApiType>;
      /**
       * Cost schedule and limits.
       **/
      schedule: PalletContractsSchedule & AugmentedConst<ApiType>;
      /**
       * Make contract callable functions marked as `#[unstable]` available.
       * 
       * Contracts that use `#[unstable]` functions won't be able to be uploaded unless
       * this is set to `true`. This is only meant for testnets and dev nodes in order to
       * experiment with new features.
       * 
       * # Warning
       * 
       * Do **not** set to `true` on productions chains.
       **/
      unsafeUnstableInterface: bool & AugmentedConst<ApiType>;
      /**
       * Generic const
       **/
      [key: string]: Codec;
    };
    corporateAction: {
      /**
       * Max number of per-DID withholding tax overrides.
       **/
      maxDidWhts: u32 & AugmentedConst<ApiType>;
      /**
       * Max number of DID specified in `TargetIdentities`.
       **/
      maxTargetIds: u32 & AugmentedConst<ApiType>;
      /**
       * Generic const
       **/
      [key: string]: Codec;
    };
    electionProviderMultiPhase: {
      /**
       * The minimum amount of improvement to the solution score that defines a solution as
       * "better" in the Signed phase.
       **/
      betterSignedThreshold: Perbill & AugmentedConst<ApiType>;
      /**
       * Maximum number of voters that can support a winner in an election solution.
       * 
       * This is needed to ensure election computation is bounded.
       **/
      maxBackersPerWinner: u32 & AugmentedConst<ApiType>;
      /**
       * Maximum number of winners that an election supports.
       * 
       * Note: This must always be greater or equal to `T::DataProvider::desired_targets()`.
       **/
      maxWinners: u32 & AugmentedConst<ApiType>;
      minerMaxLength: u32 & AugmentedConst<ApiType>;
      minerMaxVotesPerVoter: u32 & AugmentedConst<ApiType>;
      minerMaxWeight: SpWeightsWeightV2Weight & AugmentedConst<ApiType>;
      minerMaxWinners: u32 & AugmentedConst<ApiType>;
      /**
       * The priority of the unsigned transaction submitted in the unsigned-phase
       **/
      minerTxPriority: u64 & AugmentedConst<ApiType>;
      /**
       * The repeat threshold of the offchain worker.
       * 
       * For example, if it is 5, that means that at least 5 blocks will elapse between attempts
       * to submit the worker's solution.
       **/
      offchainRepeat: u32 & AugmentedConst<ApiType>;
      /**
       * Per-byte deposit for a signed solution.
       **/
      signedDepositByte: u128 & AugmentedConst<ApiType>;
      /**
       * Per-weight deposit for a signed solution.
       **/
      signedDepositWeight: u128 & AugmentedConst<ApiType>;
      /**
       * The maximum amount of unchecked solutions to refund the call fee for.
       **/
      signedMaxRefunds: u32 & AugmentedConst<ApiType>;
      /**
       * Maximum number of signed submissions that can be queued.
       * 
       * It is best to avoid adjusting this during an election, as it impacts downstream data
       * structures. In particular, `SignedSubmissionIndices<T>` is bounded on this value. If you
       * update this value during an election, you _must_ ensure that
       * `SignedSubmissionIndices.len()` is less than or equal to the new value. Otherwise,
       * attempts to submit new solutions may cause a runtime panic.
       **/
      signedMaxSubmissions: u32 & AugmentedConst<ApiType>;
      /**
       * Maximum weight of a signed solution.
       * 
       * If [`Config::MinerConfig`] is being implemented to submit signed solutions (outside of
       * this pallet), then [`MinerConfig::solution_weight`] is used to compare against
       * this value.
       **/
      signedMaxWeight: SpWeightsWeightV2Weight & AugmentedConst<ApiType>;
      /**
       * Base reward for a signed solution
       **/
      signedRewardBase: u128 & AugmentedConst<ApiType>;
      /**
       * Generic const
       **/
      [key: string]: Codec;
    };
    grandpa: {
      /**
       * Max Authorities in use
       **/
      maxAuthorities: u32 & AugmentedConst<ApiType>;
      /**
       * The maximum number of nominators for each validator.
       **/
      maxNominators: u32 & AugmentedConst<ApiType>;
      /**
       * The maximum number of entries to keep in the set id to session index mapping.
       * 
       * Since the `SetIdSession` map is only used for validating equivocations this
       * value should relate to the bonding duration of whatever staking system is
       * being used (if any). If equivocation handling is not enabled then this value
       * can be zero.
       **/
      maxSetIdSessionEntries: u64 & AugmentedConst<ApiType>;
      /**
       * Generic const
       **/
      [key: string]: Codec;
    };
    identity: {
      /**
       * POLYX given to primary keys of all new Identities
       **/
      initialPOLYX: u128 & AugmentedConst<ApiType>;
      /**
       * Maximum number of retry attempts allowed for an authorization
       * before it is considered unusable.
       **/
      maxAuthRetries: u8 & AugmentedConst<ApiType>;
      /**
       * Maximum number of authorizations an identity can give.
       **/
      maxGivenAuths: u32 & AugmentedConst<ApiType>;
      /**
       * Generic const
       **/
      [key: string]: Codec;
    };
    imOnline: {
      /**
       * A configuration for base priority of unsigned transactions.
       * 
       * This is exposed so that it can be tuned for particular runtime, when
       * multiple pallets send unsigned transactions.
       **/
      unsignedPriority: u64 & AugmentedConst<ApiType>;
      /**
       * Generic const
       **/
      [key: string]: Codec;
    };
    indices: {
      /**
       * The deposit needed for reserving an index.
       **/
      deposit: u128 & AugmentedConst<ApiType>;
      /**
       * Generic const
       **/
      [key: string]: Codec;
    };
    multiSig: {
      /**
       * Maximum number of signers that can be added/removed in one call.
       **/
      maxSigners: u32 & AugmentedConst<ApiType>;
      /**
       * Generic const
       **/
      [key: string]: Codec;
    };
    nft: {
      maxNumberOfCollectionKeys: u8 & AugmentedConst<ApiType>;
      maxNumberOfNFTsCount: u32 & AugmentedConst<ApiType>;
      /**
       * Generic const
       **/
      [key: string]: Codec;
    };
    pips: {
      /**
       * The maximum number of votes that can be pruned at once.
       **/
      maxRefundsAndVotesPruned: u32 & AugmentedConst<ApiType>;
      /**
       * Generic const
       **/
      [key: string]: Codec;
    };
    portfolio: {
      /**
       * Maximum number of fungible assets that can be moved in a single transfer call.
       **/
      maxNumberOfFungibleMoves: u32 & AugmentedConst<ApiType>;
      /**
       * Maximum number of NFTs that can be moved in a single transfer call.
       **/
      maxNumberOfNFTsMoves: u32 & AugmentedConst<ApiType>;
      /**
       * Generic const
       **/
      [key: string]: Codec;
    };
    scheduler: {
      /**
       * The maximum weight that may be scheduled per block for any dispatchables.
       **/
      maximumWeight: SpWeightsWeightV2Weight & AugmentedConst<ApiType>;
      /**
       * The maximum number of scheduled calls in the queue for a single block.
       * 
       * NOTE:
       * + Dependent pallets' benchmarks might require a higher limit for the setting. Set a
       * higher limit under `runtime-benchmarks` feature.
       **/
      maxScheduledPerBlock: u32 & AugmentedConst<ApiType>;
      /**
       * Generic const
       **/
      [key: string]: Codec;
    };
    session: {
      /**
       * The amount to be held when setting keys.
       **/
      keyDeposit: u128 & AugmentedConst<ApiType>;
      /**
       * Generic const
       **/
      [key: string]: Codec;
    };
    settlement: {
      /**
       * The maximum time period that an instruction can be held in the `LockedForExecution` status.
       **/
      maximumLockPeriod: u64 & AugmentedConst<ApiType>;
      /**
       * Maximum number mediators in the instruction level (this does not include asset mediators).
       **/
      maxInstructionMediators: u32 & AugmentedConst<ApiType>;
      /**
       * Maximum number of fungible assets that can be in a single instruction.
       **/
      maxNumberOfFungibleAssets: u32 & AugmentedConst<ApiType>;
      /**
       * Maximum number of NFTs that can be transferred in a instruction.
       **/
      maxNumberOfNFTs: u32 & AugmentedConst<ApiType>;
      /**
       * Maximum number of NFTs that can be transferred in a leg.
       **/
      maxNumberOfNFTsPerLeg: u32 & AugmentedConst<ApiType>;
      /**
       * Maximum number of off-chain assets that can be transferred in a instruction.
       **/
      maxNumberOfOffChainAssets: u32 & AugmentedConst<ApiType>;
      /**
       * Maximum number of portfolios.
       **/
      maxNumberOfPortfolios: u32 & AugmentedConst<ApiType>;
      /**
       * Maximum number of venue signers.
       **/
      maxNumberOfVenueSigners: u32 & AugmentedConst<ApiType>;
      /**
       * Generic const
       **/
      [key: string]: Codec;
    };
    staking: {
      /**
       * Number of eras that staked funds must remain bonded for.
       **/
      bondingDuration: u32 & AugmentedConst<ApiType>;
      /**
       * Number of eras to keep in history.
       * 
       * Following information is kept for eras in `[current_era -
       * HistoryDepth, current_era]`: `ErasStakers`, `ErasStakersClipped`,
       * `ErasValidatorPrefs`, `ErasValidatorReward`, `ErasRewardPoints`,
       * `ErasTotalStake`, `ErasStartSessionIndex`, `ClaimedRewards`, `ErasStakersPaged`,
       * `ErasStakersOverview`.
       * 
       * Must be more than the number of eras delayed by session.
       * I.e. active era must always be in history. I.e. `active_era >
       * current_era - history_depth` must be guaranteed.
       * 
       * If migrating an existing pallet from storage value to config value,
       * this should be set to same value or greater as in storage.
       * 
       * Note: `HistoryDepth` is used as the upper bound for the `BoundedVec`
       * item `StakingLedger.legacy_claimed_rewards`. Setting this value lower than
       * the existing value can lead to inconsistencies in the
       * `StakingLedger` and will need to be handled properly in a migration.
       * The test `reducing_history_depth_abrupt` shows this effect.
       **/
      historyDepth: u32 & AugmentedConst<ApiType>;
      /**
       * The maximum size of each `T::ExposurePage`.
       * 
       * An `ExposurePage` is weakly bounded to a maximum of `MaxExposurePageSize`
       * nominators.
       * 
       * For older non-paged exposure, a reward payout was restricted to the top
       * `MaxExposurePageSize` nominators. This is to limit the i/o cost for the
       * nominator payout.
       * 
       * Note: `MaxExposurePageSize` is used to bound `ClaimedRewards` and is unsafe to reduce
       * without handling it in a migration.
       **/
      maxExposurePageSize: u32 & AugmentedConst<ApiType>;
      /**
       * The maximum number of `unlocking` chunks a [`StakingLedger`] can
       * have. Effectively determines how many unique eras a staker may be
       * unbonding in.
       * 
       * Note: `MaxUnlockingChunks` is used as the upper bound for the
       * `BoundedVec` item `StakingLedger.unlocking`. Setting this value
       * lower than the existing value can lead to inconsistencies in the
       * `StakingLedger` and will need to be handled properly in a runtime
       * migration. The test `reducing_max_unlocking_chunks_abrupt` shows
       * this effect.
       **/
      maxUnlockingChunks: u32 & AugmentedConst<ApiType>;
      /**
       * The absolute maximum of winner validators this pallet should return.
       **/
      maxValidatorSet: u32 & AugmentedConst<ApiType>;
      /**
       * Number of sessions per era.
       **/
      sessionsPerEra: u32 & AugmentedConst<ApiType>;
      /**
       * Number of eras that slashes are deferred by, after computation.
       * 
       * This should be less than the bonding duration. Set to 0 if slashes
       * should be applied immediately, without opportunity for intervention.
       **/
      slashDeferDuration: u32 & AugmentedConst<ApiType>;
      /**
       * Generic const
       **/
      [key: string]: Codec;
    };
    statistics: {
      /**
       * Maximum stats that can be enabled for an Asset.
       **/
      maxStatsPerAsset: u32 & AugmentedConst<ApiType>;
      /**
       * Maximum transfer conditions that can be enabled for an Asset.
       **/
      maxTransferConditionsPerAsset: u32 & AugmentedConst<ApiType>;
      /**
       * Generic const
       **/
      [key: string]: Codec;
    };
    system: {
      /**
       * Maximum number of block number to block hash mappings to keep (oldest pruned first).
       **/
      blockHashCount: u32 & AugmentedConst<ApiType>;
      /**
       * The maximum length of a block (in bytes).
       **/
      blockLength: FrameSystemLimitsBlockLength & AugmentedConst<ApiType>;
      /**
       * Block & extrinsics weights: base values and limits.
       **/
      blockWeights: FrameSystemLimitsBlockWeights & AugmentedConst<ApiType>;
      /**
       * The weight of runtime database operations the runtime can invoke.
       **/
      dbWeight: SpWeightsRuntimeDbWeight & AugmentedConst<ApiType>;
      /**
       * The designated SS58 prefix of this chain.
       * 
       * This replaces the "ss58Format" property declared in the chain spec. Reason is
       * that the runtime should know about the prefix in order to make use of it as
       * an identifier of the chain.
       **/
      ss58Prefix: u16 & AugmentedConst<ApiType>;
      /**
       * Get the chain's in-code version.
       **/
      version: SpVersionRuntimeVersion & AugmentedConst<ApiType>;
      /**
       * Generic const
       **/
      [key: string]: Codec;
    };
    timestamp: {
      /**
       * The minimum period between blocks.
       * 
       * Be aware that this is different to the *expected* period that the block production
       * apparatus provides. Your chosen consensus system will generally work with this to
       * determine a sensible block time. For example, in the Aura pallet it will be double this
       * period on default settings.
       **/
      minimumPeriod: u64 & AugmentedConst<ApiType>;
      /**
       * Generic const
       **/
      [key: string]: Codec;
    };
    transactionPayment: {
      /**
       * A fee multiplier for `Operational` extrinsics to compute "virtual tip" to boost their
       * `priority`
       * 
       * This value is multiplied by the `final_fee` to obtain a "virtual tip" that is later
       * added to a tip component in regular `priority` calculations.
       * It means that a `Normal` transaction can front-run a similarly-sized `Operational`
       * extrinsic (with no tip), by including a tip value greater than the virtual tip.
       * 
       * ```rust,ignore
       * // For `Normal`
       * let priority = priority_calc(tip);
       * 
       * // For `Operational`
       * let virtual_tip = (inclusion_fee + tip) * OperationalFeeMultiplier;
       * let priority = priority_calc(tip + virtual_tip);
       * ```
       * 
       * Note that since we use `final_fee` the multiplier applies also to the regular `tip`
       * sent with the transaction. So, not only does the transaction get a priority bump based
       * on the `inclusion_fee`, but we also amplify the impact of tips applied to `Operational`
       * transactions.
       **/
      operationalFeeMultiplier: u8 & AugmentedConst<ApiType>;
      /**
       * Generic const
       **/
      [key: string]: Codec;
    };
    utility: {
      /**
       * The limit on the number of batched calls.
       **/
      batchedCallsLimit: u32 & AugmentedConst<ApiType>;
      /**
       * Generic const
       **/
      [key: string]: Codec;
    };
    validators: {
      /**
       * Yearly total reward amount that gets distributed when fixed rewards kicks in.
       **/
      fixedYearlyReward: u128 & AugmentedConst<ApiType>;
      /**
       * Maximum amount of validators that can run by an identity.
       * It will be MaxValidatorPerIdentity * Self::validator_count().
       **/
      maxValidatorPerIdentity: Permill & AugmentedConst<ApiType>;
      /**
       * Maximum amount of total issuance after which fixed rewards kicks in.
       **/
      maxVariableInflationTotalIssuance: u128 & AugmentedConst<ApiType>;
      /**
       * Generic const
       **/
      [key: string]: Codec;
    };
  } // AugmentedConsts
} // declare module
