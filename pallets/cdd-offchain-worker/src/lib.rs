use frame_support::{
    debug, decl_error, decl_event, decl_module, decl_storage, dispatch::DispatchResult, ensure,
    traits:: {
        Get
    },
};
use frame_system::{self as system, offchain};
use sp_runtime::{
    transaction_validity::{
        InvalidTransaction, TransactionLongevity, TransactionValidity, ValidTransaction,
    },
    KeyTypeId,
};
use polymesh_runtime_common::identity::Trait as IdentityTrait;

// The key type ID can be any 4-character string
pub const KEY_TYPE: KeyTypeId = KeyTypeId(*b"cddw");

pub mod crypto {
    pub use super::KEY_TYPE;
    use sp_runtime::app_crypto::{app_crypto, sr25519};
    app_crypto!(sr25519, KEY_TYPE);
}

pub trait Trait: pallet_timestamp::Trait + frame_system::Trait + IdentityTrait {
    /// The overarching event type.
    type Event: From<Event<Self>> + Into<<Self as system::Trait>::Event>;
    type Call: From<Call<Self>>;
    // No. of blocks delayed to execute the offchain worker
    type BlockDelays: Get<u64>;
    type SubmitSignedTransaction: offchain::SubmitSignedTransaction<Self, <Self as Trait>::Call>;
    type SubmitUnsignedTransaction: offchain::SubmitUnsignedTransaction<Self, <Self as Trait>::Call>;
}

decl_storage! {
    trait Store for Module<T: Trait> as CddOffchainWorker {

    }
}

decl_module! {
    pub struct Module<T: Trait> for enum Call where origin: T::Origin {
        type Error = Error<T>;

        const BlockDelays: u64 = T:BlockDelays::get();

        /// initialize the default event for this module
        fn deposit_event() = default;

        fn fetch_identities_with_invalid_cdd_claim() {
            
        }

        fn offchain_worker(block: T::BlockNumber) {
            debug::info!("Hello World.");
        }
    }
}

decl_event! {
    pub enum Event<T>
        where
        Moment = <T as pallet_timestamp::Trait>::Moment,
        AccountId = <T as frame_system::Trait>::AccountId,
    {
        LogA(Moment, AccountId),
    }
}

decl_error! {
    pub enum Error for Module<T: Trait> {
    }
}
