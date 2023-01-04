pub use pallet_transaction_payment::{FeeDetails, InclusionFee, RuntimeDispatchInfo};
use polymesh_primitives::Balance;

sp_api::decl_runtime_apis! {
    pub trait TransactionPaymentApi
    {
        fn query_info(uxt: Block::Extrinsic, len: u32) -> RuntimeDispatchInfo<Balance>;
        fn query_fee_details(uxt: Block::Extrinsic, len: u32) -> FeeDetails<Balance>;
    }
}
