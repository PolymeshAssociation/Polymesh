use crate::*;

#[cfg(not(feature = "std"))]
use alloc::string::String;
use frame_benchmarking::benchmarks;
use polymesh_primitives::{ExtrinsicName, PalletName};
use sp_std::{iter, prelude::*};

const MAX_PALLET_NAME_LENGTH: u32 = 512;
const MAX_DISPATCHABLE_NAME_LENGTH: u32 = 1024;

fn make_name(m: u32) -> String {
    iter::repeat('x').take(m as usize).collect()
}

benchmarks! {
    set_call_metadata {
        let pallet_name: PalletName = make_name(MAX_PALLET_NAME_LENGTH).into();
        let pallet_name_exp = pallet_name.clone();
        let extrinsic_name: ExtrinsicName = make_name(MAX_DISPATCHABLE_NAME_LENGTH).into();
        let extrinsic_name_exp = extrinsic_name.clone();

    }: {
        StoreCallMetadata::<T>::set_call_metadata(pallet_name, extrinsic_name);
    }
    verify {
        assert_eq!(CurrentPalletName::get(), pallet_name_exp, "Unexpected pallet name");
        assert_eq!(CurrentDispatchableName::get(), extrinsic_name_exp, "Unexpected extrinsic name");
    }

    clear_call_metadata {
        let pallet_name: PalletName = make_name(MAX_PALLET_NAME_LENGTH).into();
        let extrinsic_name: ExtrinsicName = make_name(MAX_DISPATCHABLE_NAME_LENGTH).into();
        StoreCallMetadata::<T>::set_call_metadata(pallet_name, extrinsic_name);
    }: {
        StoreCallMetadata::<T>::clear_call_metadata();
    }
    verify {
        assert!(!CurrentPalletName::exists(), "Pallet name should not be exist");
        assert!(!CurrentDispatchableName::exists(), "Dispatchable name should not be exist");
    }
}
