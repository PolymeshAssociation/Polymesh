#![cfg_attr(not(feature = "std"), no_std)]

use codec::{Decode, Encode};
use frame_support::dispatch::DispatchResult;
use frame_support::{decl_error, decl_module, decl_storage};
pub use polymesh_common_utilities::traits::nft::{Config, Event, WeightInfo};
use polymesh_primitives::Ticker;
use scale_info::TypeInfo;

#[derive(Clone, Debug, Decode, Default, Encode, PartialEq, TypeInfo)]
pub struct NFTCollection {
    ticker: Ticker,
}

decl_storage!(
    trait Store for Module<T: Config> as NFT {
        /// Details of the nft collection corresponding to the given ticker.
        /// (ticker) -> NftCollection [returns NFTCollection struct]
        pub NFtCollection get(fn nft_collection): map hasher(blake2_128_concat) Ticker => NFTCollection;
    }

    add_extra_genesis {
        build(|_config: &GenesisConfig| {});
    }
);

decl_module! {
    pub struct Module<T: Config> for enum Call where origin: T::Origin {

        type Error = Error<T>;

        /// Creates a new nft collection.
        ///
        /// # Arguments
        /// * `origin` It contains the secondary key of the caller (i.e. who signed the transaction to execute this function).
        /// * `ticker` ticker associated to the collection.
        ///
        /// # Permissions
        /// * Asset
        #[weight = <T as Config>::WeightInfo::create_nft_collection()]
        pub fn create_nft_collection(_origin, _ticker: Ticker) -> DispatchResult {
            Ok(())
        }
    }
}

decl_error! {
    pub enum Error for Module<T: Config> {
        Unauthorized,
    }
}
