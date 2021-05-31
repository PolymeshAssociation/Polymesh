// Auto-generated via `yarn polkadot-types-from-chain`, do not edit
/* eslint-disable */

import type { Vec, u32, u64 } from '@polkadot/types';
import type { Codec } from '@polkadot/types/types';
import type { Balance, BalanceOf, BlockNumber, Moment, Perbill, Permill, RuntimeDbWeight, Weight } from '@polkadot/types/interfaces/runtime';
import type { SessionIndex } from '@polkadot/types/interfaces/session';
import type { EraIndex } from '@polkadot/types/interfaces/staking';
import type { WeightToFeeCoefficient } from '@polkadot/types/interfaces/support';
import type { ApiTypes } from '@polkadot/api/types';

declare module '@polkadot/api/types/consts' {
  export interface AugmentedConsts<ApiType> {
    asset: {
      [key: string]: Codec;
      allowedGasLimit: u64 & AugmentedConst<ApiType>;
    };
    babe: {
      [key: string]: Codec;
      /**
       * The number of **slots** that an epoch takes. We couple sessions to
       * epochs, i.e. we start a new session once the new epoch begins.
       **/
      epochDuration: u64 & AugmentedConst<ApiType>;
      /**
       * The expected average block time at which BABE should be creating
       * blocks. Since BABE is probabilistic it is not trivial to figure out
       * what the expected average block time should be based on the slot
       * duration and the security parameter `c` (where `1 - c` represents
       * the probability of a slot being empty).
       **/
      expectedBlockTime: Moment & AugmentedConst<ApiType>;
    };
    balances: {
      [key: string]: Codec;
      /**
       * This is no longer needed but kept for compatibility reasons
       * The minimum amount required to keep an account open.
       **/
      existentialDeposit: Balance & AugmentedConst<ApiType>;
    };
    baseContracts: {
      [key: string]: Codec;
      /**
       * The maximum nesting level of a call/instantiate stack. A reasonable default
       * value is 100.
       **/
      maxDepth: u32 & AugmentedConst<ApiType>;
      /**
       * The maximum size of a storage value in bytes. A reasonable default is 16 KiB.
       **/
      maxValueSize: u32 & AugmentedConst<ApiType>;
      /**
       * Price of a byte of storage per one block interval. Should be greater than 0.
       **/
      rentByteFee: BalanceOf & AugmentedConst<ApiType>;
      /**
       * The amount of funds a contract should deposit in order to offset
       * the cost of one byte.
       * 
       * Let's suppose the deposit is 1,000 BU (balance units)/byte and the rent is 1 BU/byte/day,
       * then a contract with 1,000,000 BU that uses 1,000 bytes of storage would pay no rent.
       * But if the balance reduced to 500,000 BU and the storage stayed the same at 1,000,
       * then it would pay 500 BU/day.
       **/
      rentDepositOffset: BalanceOf & AugmentedConst<ApiType>;
      /**
       * Number of block delay an extrinsic claim surcharge has.
       * 
       * When claim surcharge is called by an extrinsic the rent is checked
       * for current_block - delay
       **/
      signedClaimHandicap: BlockNumber & AugmentedConst<ApiType>;
      /**
       * A size offset for an contract. A just created account with untouched storage will have that
       * much of storage from the perspective of the state rent.
       * 
       * This is a simple way to ensure that contracts with empty storage eventually get deleted
       * by making them pay rent. This creates an incentive to remove them early in order to save
       * rent.
       **/
      storageSizeOffset: u32 & AugmentedConst<ApiType>;
      /**
       * Reward that is received by the party whose touch has led
       * to removal of a contract.
       **/
      surchargeReward: BalanceOf & AugmentedConst<ApiType>;
      /**
       * The minimum amount required to generate a tombstone.
       **/
      tombstoneDeposit: BalanceOf & AugmentedConst<ApiType>;
    };
    contracts: {
      [key: string]: Codec;
      /**
       * The minimum amount required to generate a tombstone.
       **/
      networkShareInInstantiationFee: Perbill & AugmentedConst<ApiType>;
    };
    finalityTracker: {
      [key: string]: Codec;
      /**
       * The delay after which point things become suspicious. Default is 1000.
       **/
      reportLatency: BlockNumber & AugmentedConst<ApiType>;
      /**
       * The number of recent samples to keep from this chain. Default is 101.
       **/
      windowSize: BlockNumber & AugmentedConst<ApiType>;
    };
    indices: {
      [key: string]: Codec;
      /**
       * The deposit needed for reserving an index.
       **/
      deposit: BalanceOf & AugmentedConst<ApiType>;
    };
    staking: {
      [key: string]: Codec;
      /**
       * Number of eras that staked funds must remain bonded for.
       **/
      bondingDuration: EraIndex & AugmentedConst<ApiType>;
      /**
       * The number of blocks before the end of the era from which election submissions are allowed.
       * 
       * Setting this to zero will disable the offchain compute and only on-chain seq-phragmen will
       * be used.
       * 
       * This is bounded by being within the last session. Hence, setting it to a value more than the
       * length of a session will be pointless.
       **/
      electionLookahead: BlockNumber & AugmentedConst<ApiType>;
      /**
       * Total year rewards that gets paid during fixed reward schedule.
       **/
      fixedYearlyReward: BalanceOf & AugmentedConst<ApiType>;
      /**
       * Maximum number of balancing iterations to run in the offchain submission.
       * 
       * If set to 0, balance_solution will not be executed at all.
       **/
      maxIterations: u32 & AugmentedConst<ApiType>;
      /**
       * The maximum number of nominators rewarded for each validator.
       * 
       * For each validator only the `$MaxNominatorRewardedPerValidator` biggest stakers can claim
       * their reward. This used to limit the i/o cost for the nominator payout.
       **/
      maxNominatorRewardedPerValidator: u32 & AugmentedConst<ApiType>;
      /**
       * Maximum number of validators for each permissioned identity.
       * 
       * Max number of validators count = `MaxValidatorPerIdentity * Self::validator_count()`.
       **/
      maxValidatorPerIdentity: Permill & AugmentedConst<ApiType>;
      /**
       * Maximum amount of `T::currency::total_issuance()` after that non-inflated rewards get paid.
       **/
      maxVariableInflationTotalIssuance: BalanceOf & AugmentedConst<ApiType>;
      /**
       * The threshold of improvement that should be provided for a new solution to be accepted.
       **/
      minSolutionScoreBump: Perbill & AugmentedConst<ApiType>;
      /**
       * Number of sessions per era.
       **/
      sessionsPerEra: SessionIndex & AugmentedConst<ApiType>;
      /**
       * Number of eras that slashes are deferred by, after computation.
       * 
       * This should be less than the bonding duration.
       * Set to 0 if slashes should be applied immediately, without opportunity for
       * intervention.
       **/
      slashDeferDuration: EraIndex & AugmentedConst<ApiType>;
    };
    system: {
      [key: string]: Codec;
      /**
       * The base weight of executing a block, independent of the transactions in the block.
       **/
      blockExecutionWeight: Weight & AugmentedConst<ApiType>;
      /**
       * The maximum number of blocks to allow in mortal eras.
       **/
      blockHashCount: BlockNumber & AugmentedConst<ApiType>;
      /**
       * The weight of runtime database operations the runtime can invoke.
       **/
      dbWeight: RuntimeDbWeight & AugmentedConst<ApiType>;
      /**
       * The base weight of an Extrinsic in the block, independent of the of extrinsic being executed.
       **/
      extrinsicBaseWeight: Weight & AugmentedConst<ApiType>;
      /**
       * The maximum length of a block (in bytes).
       **/
      maximumBlockLength: u32 & AugmentedConst<ApiType>;
      /**
       * The maximum weight of a block.
       **/
      maximumBlockWeight: Weight & AugmentedConst<ApiType>;
    };
    timestamp: {
      [key: string]: Codec;
      /**
       * The minimum period between blocks. Beware that this is different to the *expected* period
       * that the block production apparatus provides. Your chosen consensus system will generally
       * work with this to determine a sensible block time. e.g. For Aura, it will be double this
       * period on default settings.
       **/
      minimumPeriod: Moment & AugmentedConst<ApiType>;
    };
    transactionPayment: {
      [key: string]: Codec;
      /**
       * The fee to be paid for making a transaction; the per-byte portion.
       **/
      transactionByteFee: BalanceOf & AugmentedConst<ApiType>;
      /**
       * The polynomial that is applied in order to derive fee from weight.
       **/
      weightToFee: Vec<WeightToFeeCoefficient> & AugmentedConst<ApiType>;
    };
  }

  export interface QueryableConsts<ApiType extends ApiTypes> extends AugmentedConsts<ApiType> {
    [key: string]: QueryableModuleConsts;
  }
}
