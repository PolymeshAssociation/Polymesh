use crate::*;
use core::convert::TryFrom;
use frame_benchmarking::benchmarks;
use polymesh_common_utilities::{
    asset::AssetType,
    benchs::{User, UserBuilder},
};
use sp_std::prelude::*;

/// Create a new token with name `name` on behalf of `owner`.
/// The new token is a _divisible_ one with 1_000_000 units.
pub fn make_token<T: Trait>(owner: &User<T>, name: Vec<u8>) -> Ticker {
    let ticker = Ticker::try_from(name.as_slice()).unwrap();
    T::Asset::create_asset(
        owner.origin.clone().into(),
        name.into(),
        ticker.clone(),
        1_000_000.into(),
        true,
        AssetType::default(),
        Vec::new(),
        None,
    )
    .expect("Cannot create an asset");

    ticker
}

benchmarks! {
    _ {}

    add_transfer_manager {
        let owner = UserBuilder::<T>::default().generate_did().build("OWNER");
        let ticker = make_token::<T>(&owner, b"1".to_vec());
        let mut tms = Vec::new();
        for i in 1..T::MaxTransferManagersPerAsset::get() {
            tms.push(TransferManager::CountTransferManager(i.into()));
        }
        ActiveTransferManagers::insert(ticker, tms.clone());
        tms.push(TransferManager::CountTransferManager(420));
    }: _(owner.origin, ticker, tms.last().unwrap().clone())
    verify {
        assert!(Module::<T>::transfer_managers(ticker) == tms);
    }

    remove_transfer_manager {
        let owner = UserBuilder::<T>::default().generate_did().build("OWNER");
        let ticker = make_token::<T>(&owner, b"1".to_vec());
        let mut tms = Vec::new();
        for i in 0..T::MaxTransferManagersPerAsset::get() {
            tms.push(TransferManager::CountTransferManager(i.into()));
        }
        ActiveTransferManagers::insert(ticker, tms.clone());
    }: _(owner.origin, ticker, tms.pop().unwrap().clone())
    verify {
        assert!(Module::<T>::transfer_managers(ticker) == tms);
    }

    add_exempted_entities {
        // Length of the vector of Exempted identities being added.
        let i in 0 .. 1000;

        let owner = UserBuilder::<T>::default().generate_did().build("OWNER");
        let ticker = make_token::<T>(&owner, b"1".to_vec());
        let mut scope_ids = Vec::new();
        for x in 0..i {
            scope_ids.push(IdentityId::from(x as u128));
        }
        let tm = TransferManager::CountTransferManager(420);
    }: _(owner.origin, ticker, tm.clone(), scope_ids)
    verify {
        if i > 0 {
            assert!(Module::<T>::entity_exempt((ticker, tm), IdentityId::from(0u128)));
        }
    }

    remove_exempted_entities {
        // Length of the vector of Exempted identities being removed.
        let i in 0 .. 1000;

        let owner = UserBuilder::<T>::default().generate_did().build("OWNER");
        let ticker = make_token::<T>(&owner, b"1".to_vec());
        let tm = TransferManager::CountTransferManager(420);
        let mut scope_ids = Vec::new();
        for x in 0..i {
            let scope_id = IdentityId::from(x as u128);
            scope_ids.push(scope_id.clone());
            ExemptEntities::insert((ticker, tm.clone()), scope_id, true);
        }
    }: _(owner.origin, ticker, tm.clone(), scope_ids)
    verify {
        if i > 0 {
            assert!(!Module::<T>::entity_exempt((ticker, tm), IdentityId::from(0u128)));
        }
    }

    #[extra]
    verify_tm_restrictions {
        let owner = UserBuilder::<T>::default().generate_did().build("OWNER");
        let ticker = make_token::<T>(&owner, b"1".to_vec());

        let t in 0 .. T::MaxTransferManagersPerAsset::get();
        let mut tms = Vec::new();
        for i in 0..t {
            let tm = TransferManager::CountTransferManager(i.into());
            tms.push(tm.clone());
            ExemptEntities::insert((ticker, tm), owner.did.unwrap(), true);
        }
        ActiveTransferManagers::insert(ticker, tms.clone());
        InvestorCountPerAsset::insert(ticker, 1337);
    }: {
        // This will trigger the worse case (exemption)
        Module::<T>::verify_tm_restrictions(
            &ticker,
            owner.did.unwrap(),
            owner.did.unwrap(),
            100.into(),
            200.into(),
            0.into(),
            500.into(),
        )?;
    }
}
