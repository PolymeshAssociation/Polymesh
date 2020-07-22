use codec::Codec;
use pallet_transaction_payment::RuntimeDispatchInfo;

sp_api::decl_runtime_apis! {
    pub trait TransactionPaymentApi<Balance, Extrinsic> where
        Balance: Codec,
        Extrinsic: Codec,
    {
        fn query_info(uxt: Extrinsic, len: u32) -> RuntimeDispatchInfo<Balance>;
    }
}
