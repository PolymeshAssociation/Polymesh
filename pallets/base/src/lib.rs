#![cfg_attr(not(feature = "std"), no_std)]

use frame_support::dispatch::{DispatchError, DispatchResult};
use frame_support::traits::Get;
use frame_support::{decl_error, decl_module, ensure};

pub use polymesh_common_utilities::traits::base::{Event, Trait};

decl_module! {
    pub struct Module<T: Trait> for enum Call where origin: T::Origin {
        type Error = Error<T>;
        fn deposit_event() = default;
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

/// Ensure that `s.len()` is within the generic length limit.
pub fn ensure_string_limited<T: Trait>(s: &[u8]) -> DispatchResult {
    ensure_length_ok::<T>(s.len() as u32)
}

/// Ensure that given `Some(s)`, `s.len()` is within the generic length limit.
pub fn ensure_opt_string_limited<T: Trait>(s: Option<&[u8]>) -> DispatchResult {
    match s {
        None => Ok(()),
        Some(s) => ensure_string_limited::<T>(s),
    }
}
