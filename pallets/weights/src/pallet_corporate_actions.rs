use frame_support::weights::Weight;

pub struct WeightInfo;
impl pallet_corporate_actions::WeightInfo for WeightInfo {
    fn reset_caa() -> Weight {
        999_999_999_999
    }
    fn set_default_targets(_: u32) -> Weight {
        999_999_999_999
    }
    fn set_default_withholding_tax() -> Weight {
        999_999_999_999
    }
    fn set_did_withholding_tax() -> Weight {
        999_999_999_999
    }
}
