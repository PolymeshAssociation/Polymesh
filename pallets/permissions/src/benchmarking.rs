use crate::*;

use polymesh_primitives::{DispatchableName, PalletName};

use frame_benchmarking::benchmarks;
use sp_std::{iter, prelude::*};

const MAX_PALLET_NAME: u32 = 512;
const MAX_DISPATCHABLE_NAME: u32 = 1024;

fn make_name(m: u32) -> Vec<u8> {
    iter::repeat(b'x').take(m as usize).collect::<Vec<_>>()
}

benchmarks! {
    _ {}

    set_call_metadata {
        let p in 1..MAX_PALLET_NAME;
        let d in 1..MAX_DISPATCHABLE_NAME;

        let pallet_name :PalletName = make_name(p).into();
        let pallet_name_exp = pallet_name.clone();
        let dispatchable_name :DispatchableName = make_name(p).into();
        let dispatchable_name_exp = dispatchable_name.clone();

    }: {
        StoreCallMetadata::<T>::set_call_metadata(pallet_name, dispatchable_name);
    }
    verify {
        assert_eq!( StoreCallMetadata::<T>::current_pallet_name(), pallet_name_exp);
        assert_eq!( StoreCallMetadata::<T>::current_dispatchable_name(), dispatchable_name_exp);
    }

    clear_call_metadata {
        let pallet_name :PalletName = make_name(MAX_PALLET_NAME).into();
        let dispatchable_name :DispatchableName = make_name(MAX_DISPATCHABLE_NAME).into();
        StoreCallMetadata::<T>::set_call_metadata(pallet_name, dispatchable_name);
    }: {
        StoreCallMetadata::<T>::clear_call_metadata();
    }
    verify {
        assert_eq!( StoreCallMetadata::<T>::current_pallet_name(), PalletName::default());
        assert_eq!( StoreCallMetadata::<T>::current_dispatchable_name(), DispatchableName::default());
     }
}
