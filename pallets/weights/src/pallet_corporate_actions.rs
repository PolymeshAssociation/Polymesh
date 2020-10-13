use frame_support::weights::Weight;
use pallet_corporate_actions::TargetIdentities;

pub struct WeightInfo;
impl pallet_corporate_actions::WeightInfo for WeightInfo {
    fn set_default_targets(_: &TargetIdentities) -> Weight {
        999_999_999_999
    }
    fn set_default_withholding_tax() -> Weight {
        999_999_999_999
    }
    fn set_did_withholding_tax() -> Weight {
        999_999_999_999
    }
}
