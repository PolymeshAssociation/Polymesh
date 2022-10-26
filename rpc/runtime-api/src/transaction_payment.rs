use codec::Codec;
pub use pallet_transaction_payment::{FeeDetails, InclusionFee, RuntimeDispatchInfo};
use polymesh_primitives::Balance;
use sp_std::vec::Vec;

sp_api::decl_runtime_apis! {
    #[api_version(2)]
    pub trait TransactionPaymentApi<Extrinsic> where
        Extrinsic: Codec,
    {
        fn query_info(encoded_xt: Vec<u8>) -> Option<RuntimeDispatchInfo<Balance>>;
        #[changed_in(2)]
        fn query_info(uxt: Extrinsic, len: u32) -> RuntimeDispatchInfo<Balance>;

            fn query_fee_details(encoded_xt: Vec<u8>) -> Option<FeeDetails<Balance>>;
    }
}
