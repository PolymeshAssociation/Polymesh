#![cfg_attr(not(feature = "std"), no_std)]

use frame_support::dispatch::{DispatchError, DispatchResult};
use frame_support::traits::Get;
use frame_support::{decl_error, decl_event, decl_module, ensure};

pub trait Trait: frame_system::Trait {
    /// The overarching event type.
    type Event: From<Event> + Into<<Self as frame_system::Trait>::Event>;

    /// The maximum length governing `TooLong`.
    ///
    /// How lengths are computed to compare against this value is situation based.
    /// For example, you could halve it, double it, compute a sum for some tree of strings, etc.
    type MaxLen: Get<u32>;
}

decl_module! {
    pub struct Module<T: Trait> for enum Call where origin: T::Origin {
        type Error = Error<T>;
        fn deposit_event() = default;
    }
}

decl_event! {
    pub enum Event {
        /// An unexpected error happened that should be investigated.
        UnexpectedError(Option<DispatchError>),
    }
}

/// Emit an unexpected error event that should be investigated manually
pub fn emit_unexpected_error<T: Trait>(error: Option<DispatchError>) {
    Module::<T>::deposit_event(Event::UnexpectedError(error));
}

decl_error! {
    pub enum Error for Module<T: Trait> {
        /// Exceeded a generic length limit.
        /// The limit could be for any sort of lists of things, including a string.
        TooLong,
    }
}

/// Ensure that the `len` provided is within the generic length limit.
pub fn ensure_length_ok<T: Trait>(len: u32) -> DispatchResult {
    ensure!(len <= T::MaxLen::get(), Error::<T>::TooLong);
    Ok(())
}

/// Ensure that string's `.len()` is within the generic length limit.
pub fn ensure_string_limited<T: Trait>(s: &[u8]) -> DispatchResult {
    ensure_length_ok::<T>(s.len() as u32)
}
