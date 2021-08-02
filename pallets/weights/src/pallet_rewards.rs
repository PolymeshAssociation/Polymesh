use polymesh_runtime_common::Weight;

pub struct WeightInfo;
impl pallet_rewards::WeightInfo for WeightInfo {
    fn claim_itn_reward() -> Weight {
        0u64
    }
    fn set_itn_reward_status() -> Weight {
        0u64
    }
}
