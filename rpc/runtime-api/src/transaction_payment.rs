use codec::Codec;
pub use pallet_transaction_payment::{FeeDetails, InclusionFee, RuntimeDispatchInfo};
use polymesh_primitives::Balance;

sp_api::decl_runtime_apis! {
    pub trait TransactionPaymentApi<Extrinsic> where
        Extrinsic: Codec,
    {
        fn query_info(uxt: Extrinsic, len: u32) -> RuntimeDispatchInfo<Balance>;
        fn query_fee_details(uxt: Extrinsic, len: u32) -> FeeDetails<Balance>;
    }
}
