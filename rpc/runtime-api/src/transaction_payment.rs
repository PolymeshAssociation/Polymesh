use codec::{Codec, Encode, Output};
pub use pallet_transaction_payment::{FeeDetails, InclusionFee, RuntimeDispatchInfo};
use polymesh_primitives::Balance;
use sp_std::vec::Vec;

/// `Encoded` is used to pass the raw transaction to the runtime.
/// This type is only used to support runtimes with the old v1 `query_info` interface.
#[derive(Clone, Debug)]
pub struct Encoded(pub Vec<u8>);

impl Encode for Encoded {
    fn size_hint(&self) -> usize {
        self.0.len()
    }
    fn encode_to<T: Output + ?Sized>(&self, dest: &mut T) {
        dest.write(&self.0);
    }
}

sp_api::decl_runtime_apis! {
    #[api_version(2)]
    pub trait TransactionPaymentApi<Extrinsic> where
        Extrinsic: Codec,
    {
        fn query_info(encoded_xt: Vec<u8>) -> Option<RuntimeDispatchInfo<Balance>>;
        #[changed_in(2)]
        fn query_info(uxt: Encoded, len: u32) -> RuntimeDispatchInfo<Balance>;

        fn query_fee_details(encoded_xt: Vec<u8>) -> Option<FeeDetails<Balance>>;
    }
}
