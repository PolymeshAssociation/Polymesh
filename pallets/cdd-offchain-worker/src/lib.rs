use frame_support::{
    debug, decl_error, decl_event, decl_module, decl_storage, dispatch::DispatchResult, ensure,
    traits::Get,
};
use frame_system::{self as system, offchain};
use polymesh_runtime_common::identity::Trait as IdentityTrait;
use sp_runtime::{
    transaction_validity::{
        InvalidTransaction, TransactionLongevity, TransactionValidity, ValidTransaction,
    },
    KeyTypeId,
};

// The key type ID can be any 4-character string
pub const KEY_TYPE: KeyTypeId = KeyTypeId(*b"cddw");

pub mod crypto {
    pub use super::KEY_TYPE;
    use sp_runtime::app_crypto::{app_crypto, sr25519};
    app_crypto!(sr25519, KEY_TYPE);
}

pub trait Trait: pallet_timestamp::Trait + frame_system::Trait + staking::Trait {
    /// The overarching event type.
    type Event: From<Event<Self>> + Into<<Self as system::Trait>::Event>;
    type Call: From<Call<Self>>;
    /// No. of blocks delayed to execute the offchain worker
    type BlockDelays: Get<Self::BlockNumber>;
    /// Buffer given to check the validity of the cdd claim. It is in block numbers.
    type BufferTime: Get<Self::BlockNumber>;
    type SubmitSignedTransaction: offchain::SubmitSignedTransaction<Self, <Self as Trait>::Call>;
    type SubmitUnsignedTransaction: offchain::SubmitUnsignedTransaction<Self, <Self as Trait>::Call>;
}

decl_storage! {
    trait Store for Module<T: Trait> as CddOffchainWorker {
        // Last block no. at which offchain_worker executed.
        pub LastCheckedBlock get(fn last_checked_block): T::BlockNumber;
    }
}

decl_module! {
    pub struct Module<T: Trait> for enum Call where origin: T::Origin {
        type Error = Error<T>;

        /// initialize the default event for this module
        fn deposit_event() = default;

        // pub fn onchain_fallback(origin, _block: T::BlockNumber, invalid_nominators: T::AccountId) -> DispatchResult {

        // }

        fn offchain_worker(block: T::BlockNumber) {
            if self::last_checked_block() + T::BlockDelays::get() >= block && T::BlockDelays::get() > 0.into() {
                let invalid_nominators = <staking::Module<T>>::fetch_invalid_cdd_nominators(0_u64);
                <LastCheckedBlock<T>>::insert(block);
                if invalid_nominators.len() > 0 {
                    // Here we specify the function to be called back on-chain in next block import.
                    let call = Call::Staking<T>::validate_kyc_expiry_nominators(invalid_nominators.clone());
                    T::SubmitSignedTransaction::submit_signed(call);
                }
            }
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
