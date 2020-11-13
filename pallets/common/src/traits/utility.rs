use frame_support::weights::{GetDispatchInfo, Weight};

pub trait WeightInfo {
    fn batch(calls: &[impl GetDispatchInfo]) -> Weight;
    fn batch_atomic(calls: &[impl GetDispatchInfo]) -> Weight;
    fn batch_optimistic(calls: &[impl GetDispatchInfo]) -> Weight;
    fn relay_tx(call: &impl GetDispatchInfo) -> Weight;
}
