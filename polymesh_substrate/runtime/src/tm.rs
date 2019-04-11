use parity_codec::Codec;
use crate::asset;

use support::{dispatch::Result, Parameter, StorageMap, StorageValue, decl_storage, decl_module, decl_event, ensure};
use runtime_primitives::traits::{CheckedSub, CheckedAdd, Member, SimpleArithmetic, As};
use system::{self, ensure_signed};

/// The module's configuration trait.
pub trait Trait: system::Trait {
    //fn verify_restriction(token_id: u32, from: Self::AccountId, to: Self::AccountId, value: Self::TokenBalance) -> Result;
    type Asset: asset::Trait;
}

decl_storage! {
	trait Store for Module<T: Trait> as Tm {
		
	}
}

decl_module! {
	/// The module declaration.
	pub struct Module<T: Trait> for enum Call where origin: T::Origin {
		
	}
}
