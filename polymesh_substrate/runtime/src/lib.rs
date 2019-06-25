//! The Substrate Node Template runtime. This can be compiled with `#[no_std]`, ready for Wasm.

#![cfg_attr(not(feature = "std"), no_std)]
//#![cfg_attr(not(feature = "std"), feature(alloc))]
// `construct_runtime!` does a lot of recursion and requires us to increase the limit to 256.
#![recursion_limit = "256"]

use client::{
    block_builder::api::{self as block_builder_api, CheckInherentsResult, InherentData},
    impl_runtime_apis, runtime_api as client_api,
};
#[cfg(feature = "std")]
use council::seats as council_seats;
pub use council::{motions as council_motions, voting as council_voting};
pub use grandpa::fg_primitives::{self, ScheduledChange};
use parity_codec::{Decode, Encode};
#[cfg(feature = "std")]
use primitives::bytes;
use primitives::u32_trait::{_2, _4};
use primitives::{ed25519, sr25519, OpaqueMetadata};
use rstd::prelude::*;
use runtime_primitives::{
    create_runtime_str, generic,
    traits::{
        self, AuthorityIdFor, BlakeTwo256, Block as BlockT, Convert, DigestFor, NumberFor,
        StaticLookup, Verify,
    },
    transaction_validity::TransactionValidity,
    ApplyResult,
};
#[cfg(feature = "std")]
use serde_derive::{Deserialize, Serialize};
#[cfg(feature = "std")]
use version::NativeVersion;
use version::RuntimeVersion;
// A few exports that help ease life for downstream crates.
pub use balances::Call as BalancesCall;
pub use consensus::Call as ConsensusCall;
#[cfg(any(feature = "std", test))]
pub use runtime_primitives::BuildStorage;
pub use runtime_primitives::{Perbill, Permill};
pub use staking::StakerStatus;
pub use support::{construct_runtime, StorageValue};
pub use timestamp::BlockPeriod;
pub use timestamp::Call as TimestampCall;

/// The type that is used for identifying authorities.
pub type AuthorityId = <AuthoritySignature as Verify>::Signer;

/// The type used by authorities to prove their ID.
pub type AuthoritySignature = ed25519::Signature;

/// Alias to pubkey that identifies an account on the chain.
pub type AccountId = <AccountSignature as Verify>::Signer;

/// The type used by authorities to prove their ID.
pub type AccountSignature = sr25519::Signature;

/// A hash of some data used by the chain.
pub type Hash = primitives::H256;

/// Index of a block number in the chain.
pub type BlockNumber = u64;

/// Index of an account's extrinsic in the chain.
pub type Nonce = u64;

mod asset;
mod dividend;
mod erc20;
mod exemption;
mod general_tm;
mod identity;
mod percentage_tm;
mod sto_capped;
mod template;
mod utils;

pub struct CurrencyToVoteHandler;

impl CurrencyToVoteHandler {
    fn factor() -> u128 {
        (Balances::total_issuance() / u64::max_value() as u128).max(1)
    }
}

impl Convert<u128, u64> for CurrencyToVoteHandler {
    fn convert(x: u128) -> u64 {
        (x / Self::factor()) as u64
    }
}

impl Convert<u128, u128> for CurrencyToVoteHandler {
    fn convert(x: u128) -> u128 {
        x * Self::factor()
    }
}

pub struct CurrencyToBalanceHandler;

impl Convert<u128, u128> for CurrencyToBalanceHandler {
    fn convert(x: u128) -> u128 {
        x
    }
}

/// Opaque types. These are used by the CLI to instantiate machinery that don't need to know
/// the specifics of the runtime. They can then be made to be agnostic over specific formats
/// of data like extrinsics, allowing for them to continue syncing the network through upgrades
/// to even the core datastructures.
pub mod opaque {
    use super::*;

    /// Opaque, encoded, unchecked extrinsic.
    #[derive(PartialEq, Eq, Clone, Default, Encode, Decode)]
    #[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
    pub struct UncheckedExtrinsic(#[cfg_attr(feature = "std", serde(with = "bytes"))] pub Vec<u8>);
    #[cfg(feature = "std")]
    impl std::fmt::Debug for UncheckedExtrinsic {
        fn fmt(&self, fmt: &mut std::fmt::Formatter) -> std::fmt::Result {
            write!(fmt, "{}", primitives::hexdisplay::HexDisplay::from(&self.0))
        }
    }
    impl traits::Extrinsic for UncheckedExtrinsic {
        fn is_signed(&self) -> Option<bool> {
            None
        }
    }
    /// Opaque block header type.
    pub type Header = generic::Header<
        BlockNumber,
        BlakeTwo256,
        generic::DigestItem<Hash, AuthorityId, AuthoritySignature>,
    >;
    /// Opaque block type.
    pub type Block = generic::Block<Header, UncheckedExtrinsic>;
    /// Opaque block identifier type.
    pub type BlockId = generic::BlockId<Block>;
    /// Opaque session key type.
    pub type SessionKey = AuthorityId;
}

/// This runtime version.
pub const VERSION: RuntimeVersion = RuntimeVersion {
    spec_name: create_runtime_str!("polymesh-substrate"),
    impl_name: create_runtime_str!("polymesh-substrate"),
    authoring_version: 3,
    spec_version: 3,
    impl_version: 0,
    apis: RUNTIME_API_VERSIONS,
};

/// The version infromation used to identify this runtime when compiled natively.
#[cfg(feature = "std")]
pub fn native_version() -> NativeVersion {
    NativeVersion {
        runtime_version: VERSION,
        can_author_with: Default::default(),
    }
}

impl system::Trait for Runtime {
    /// The identifier used to distinguish between accounts.
    type AccountId = AccountId;
    /// The lookup mechanism to get account ID from whatever is passed in dispatchers.
    type Lookup = Indices;
    /// The index type for storing how many extrinsics an account has signed.
    type Index = Nonce;
    /// The index type for blocks.
    type BlockNumber = BlockNumber;
    /// The type for hashing blocks and tries.
    type Hash = Hash;
    /// The hashing algorithm used.
    type Hashing = BlakeTwo256;
    /// The header digest type.
    type Digest = generic::Digest<Log>;
    /// The header type.
    type Header = generic::Header<BlockNumber, BlakeTwo256, Log>;
    /// The ubiquitous event type.
    type Event = Event;
    /// The ubiquitous log type.
    type Log = Log;
    /// The ubiquitous origin type.
    type Origin = Origin;
}

impl aura::Trait for Runtime {
    type HandleReport = aura::StakingSlasher<Runtime>;
}

impl consensus::Trait for Runtime {
    /// The identifier we use to refer to authorities.
    type SessionKey = AuthorityId;
    // The aura module handles offline-reports internally
    // rather than using an explicit report system.
    type InherentOfflineReport = ();
    /// The ubiquitous log type.
    type Log = Log;
}

impl treasury::Trait for Runtime {
    type Currency = Balances;
    type ApproveOrigin = council_motions::EnsureMembers<_4>;
    type RejectOrigin = council_motions::EnsureMembers<_2>;
    type Event = Event;
    type MintedForSpending = ();
    type ProposalRejection = ();
}

impl session::Trait for Runtime {
    type ConvertAccountIdToSessionKey = ();
    type OnSessionChange = (Staking, grandpa::SyncedAuthorities<Runtime>);
    type Event = Event;
}

impl staking::Trait for Runtime {
    type Currency = Balances;
    type CurrencyToVote = CurrencyToVoteHandler;
    type OnRewardMinted = Treasury;
    type Event = Event;
    type Slash = ();
    type Reward = ();
}

impl democracy::Trait for Runtime {
    type Currency = Balances;
    type Proposal = Call;
    type Event = Event;
}

impl council::Trait for Runtime {
    type Event = Event;
    type BadPresentation = ();
    type BadReaper = ();
}

impl council::voting::Trait for Runtime {
    type Event = Event;
}

impl council::motions::Trait for Runtime {
    type Origin = Origin;
    type Proposal = Call;
    type Event = Event;
}

impl grandpa::Trait for Runtime {
    type SessionKey = AuthorityId;
    type Log = Log;
    type Event = Event;
}

impl finality_tracker::Trait for Runtime {
    type OnFinalizationStalled = grandpa::SyncedAuthorities<Runtime>;
}

impl indices::Trait for Runtime {
    /// The type for recording indexing into the account enumeration. If this ever overflows, there
    /// will be problems!
    type AccountIndex = u32;
    /// Use the standard means of resolving an index hint from an id.
    type ResolveHint = indices::SimpleResolveHint<Self::AccountId, Self::AccountIndex>;
    /// Determine whether an account is dead.
    type IsDeadAccount = Balances;
    /// The uniquitous event type.
    type Event = Event;
}

impl timestamp::Trait for Runtime {
    /// A timestamp: seconds since the unix epoch.
    type Moment = u64;
    type OnTimestampSet = Aura;
}

impl balances::Trait for Runtime {
    /// The type for recording an account's balance.
    type Balance = u128;
    /// What to do if an account's free balance gets zeroed.
    type OnFreeBalanceZero = ();
    /// What to do if a new account is created.
    type OnNewAccount = Indices;
    /// The uniquitous event type.
    type Event = Event;

    type TransactionPayment = ();
    type DustRemoval = ();
    type TransferPayment = ();
}

impl sudo::Trait for Runtime {
    /// The uniquitous event type.
    type Event = Event;
    type Proposal = Call;
}

/// Used for the module template in `./template.rs`
impl template::Trait for Runtime {
    type Event = Event;
}

impl asset::Trait for Runtime {
    type Event = Event;
    //type TokenBalance = u128;
    type Currency = Balances;
    type CurrencyToBalance = CurrencyToBalanceHandler;
}

impl utils::Trait for Runtime {
    type TokenBalance = u128;
    fn as_u128(v: Self::TokenBalance) -> u128 {
        v
    }
    fn as_tb(v: u128) -> Self::TokenBalance {
        v
    }
}

impl erc20::Trait for Runtime {
    type Currency = Balances;
    type Event = Event;
}

impl general_tm::Trait for Runtime {
    type Event = Event;
    type Asset = Asset;
}

impl sto_capped::Trait for Runtime {
    type Event = Event;
    type ERC20Trait = ERC20;
}

impl percentage_tm::Trait for Runtime {
    type Event = Event;
}

impl identity::Trait for Runtime {
    type Event = Event;
}

impl exemption::Trait for Runtime {
    type Event = Event;
    type Asset = Asset;
}

impl dividend::Trait for Runtime {
    type Event = Event;
}

construct_runtime!(
    pub enum Runtime with Log(InternalLog: DigestItem<Hash, AuthorityId, AuthoritySignature>) where
        Block = Block,
        NodeBlock = opaque::Block,
        UncheckedExtrinsic = UncheckedExtrinsic
    {
        System: system::{default, Log(ChangesTrieRoot)},
        Timestamp: timestamp::{Module, Call, Storage, Config<T>, Inherent},
        Consensus: consensus::{Module, Call, Storage, Config<T>, Log(AuthoritiesChange), Inherent},
        Aura: aura::{Module, Inherent(Timestamp)},
        Indices: indices,
        Balances: balances,
        Sudo: sudo,
        // Used for the module template in `./template.rs`
        TemplateModule: template::{Module, Call, Storage, Event<T>},
        Asset: asset::{Module, Call, Storage, Config<T>, Event<T>},
        Utils: utils::{Module, Call, Storage},
        Dividend: dividend::{Module, Call, Storage, Event<T>},
        Identity: identity::{Module, Call, Storage, Event<T>, Config<T>},
        GeneralTM: general_tm::{Module, Call, Storage, Event<T>},
        STOCapped: sto_capped::{Module, Call, Storage, Event<T>},
        PercentageTM: percentage_tm::{Module, Call, Storage, Event<T>},
        Exemption: exemption::{Module, Call, Storage, Event<T>},
        Session: session,
        Staking: staking::{default, OfflineWorker},
        Democracy: democracy,
        Council: council::{Module, Call, Storage, Event<T>},
        CouncilVoting: council_voting,
        CouncilMotions: council_motions::{Module, Call, Storage, Event<T>, Origin},
        CouncilSeats: council_seats::{Config<T>},
        FinalityTracker: finality_tracker::{Module, Call, Inherent},
        Grandpa: grandpa::{Module, Call, Storage, Config<T>, Log(), Event<T>},
        Treasury: treasury,
        ERC20: erc20::{Module, Call, Storage, Event<T>, Config<T>},
    }
);

/// The type used as a helper for interpreting the sender of transactions.
type Context = system::ChainContext<Runtime>;
/// The address format for describing accounts.
type Address = <Indices as StaticLookup>::Source;
/// Block header type as expected by this runtime.
pub type Header = generic::Header<BlockNumber, BlakeTwo256, Log>;
/// Block type as expected by this runtime.
pub type Block = generic::Block<Header, UncheckedExtrinsic>;
/// BlockId type as expected by this runtime.
pub type BlockId = generic::BlockId<Block>;
/// Unchecked extrinsic type as expected by this runtime.
pub type UncheckedExtrinsic =
    generic::UncheckedMortalCompactExtrinsic<Address, Nonce, Call, AccountSignature>;
/// Extrinsic type that has already been checked.
pub type CheckedExtrinsic = generic::CheckedExtrinsic<AccountId, Nonce, Call>;
/// Executive: handles dispatch to the various modules.
pub type Executive = executive::Executive<Runtime, Block, Context, Balances, AllModules>;

// Implement our runtime API endpoints. This is just a bunch of proxying.
impl_runtime_apis! {
    impl client_api::Core<Block> for Runtime {
        fn version() -> RuntimeVersion {
            VERSION
        }

        fn execute_block(block: Block) {
            Executive::execute_block(block)
        }

        fn initialize_block(header: &<Block as BlockT>::Header) {
            Executive::initialize_block(header)
        }

        fn authorities() -> Vec<AuthorityIdFor<Block>> {
            panic!("Deprecated, please use `AuthoritiesApi`.")
        }
    }

    impl client_api::Metadata<Block> for Runtime {
        fn metadata() -> OpaqueMetadata {
            Runtime::metadata().into()
        }
    }

    impl block_builder_api::BlockBuilder<Block> for Runtime {
        fn apply_extrinsic(extrinsic: <Block as BlockT>::Extrinsic) -> ApplyResult {
            Executive::apply_extrinsic(extrinsic)
        }

        fn finalize_block() -> <Block as BlockT>::Header {
            Executive::finalize_block()
        }

        fn inherent_extrinsics(data: InherentData) -> Vec<<Block as BlockT>::Extrinsic> {
            data.create_extrinsics()
        }

        fn check_inherents(block: Block, data: InherentData) -> CheckInherentsResult {
            data.check_extrinsics(&block)
        }

        fn random_seed() -> <Block as BlockT>::Hash {
            System::random_seed()
        }
    }

    impl client_api::TaggedTransactionQueue<Block> for Runtime {
        fn validate_transaction(tx: <Block as BlockT>::Extrinsic) -> TransactionValidity {
            Executive::validate_transaction(tx)
        }
    }

    impl offchain_primitives::OffchainWorkerApi<Block> for Runtime {
        fn offchain_worker(number: NumberFor<Block>) {
            Executive::offchain_worker(number)
        }
    }

    impl fg_primitives::GrandpaApi<Block> for Runtime {
        fn grandpa_pending_change(digest: &DigestFor<Block>)
            -> Option<ScheduledChange<NumberFor<Block>>>
        {
            for log in digest.logs.iter().filter_map(|l| match l {
                Log(InternalLog::grandpa(grandpa_signal)) => Some(grandpa_signal),
                _ => None
            }) {
                if let Some(change) = Grandpa::scrape_digest_change(log) {
                    return Some(change);
                }
            }
            None
        }

        fn grandpa_forced_change(digest: &DigestFor<Block>)
            -> Option<(NumberFor<Block>, ScheduledChange<NumberFor<Block>>)>
        {
            for log in digest.logs.iter().filter_map(|l| match l {
                Log(InternalLog::grandpa(grandpa_signal)) => Some(grandpa_signal),
                _ => None
            }) {
                if let Some(change) = Grandpa::scrape_digest_forced_change(log) {
                    return Some(change);
                }
            }
            None
        }

        fn grandpa_authorities() -> Vec<(AuthorityId, u64)> {
            Grandpa::grandpa_authorities()
        }
    }

    impl consensus_aura::AuraApi<Block> for Runtime {
        fn slot_duration() -> u64 {
            Aura::slot_duration()
        }
    }

    impl consensus_authorities::AuthoritiesApi<Block> for Runtime {
        fn authorities() -> Vec<AuthorityIdFor<Block>> {
            Consensus::authorities()
        }
    }
}
