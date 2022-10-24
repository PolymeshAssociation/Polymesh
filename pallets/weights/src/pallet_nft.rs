use polymesh_runtime_common::Weight;

pub struct WeightInfo;

impl pallet_nft::WeightInfo for WeightInfo {
    fn create_nft_collection() -> Weight {
        100_490_000 as Weight
    }

    fn mint_nft() -> Weight {
        100_490_000 as Weight
    }
}
