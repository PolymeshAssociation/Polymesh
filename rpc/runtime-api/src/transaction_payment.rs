use codec::Codec;

pub use pallet_transaction_payment::{FeeDetails, InclusionFee, RuntimeDispatchInfo};
use polymesh_primitives::Balance;

sp_api::decl_runtime_apis! {
    pub trait TransactionPaymentApi
    {
        fn query_info(uxt: Block::Extrinsic, len: u32) -> RuntimeDispatchInfo<Balance>;
        fn query_fee_details(uxt: Block::Extrinsic, len: u32) -> FeeDetails<Balance>;
    }

    pub trait TransactionPaymentCallApi<Call>
    where
        Call: Codec,
    {
        /// Query information of a dispatch class, weight, and fee of a given encoded `Call`.
        fn query_call_info(call: Call, len: u32) -> RuntimeDispatchInfo<Balance>;

        /// Query fee details of a given encoded `Call`.
        fn query_call_fee_details(call: Call, len: u32) -> FeeDetails<Balance>;
    }
}
