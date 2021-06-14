use crate::*;
use frame_benchmarking::benchmarks;
use polymesh_common_utilities::{
    benchs::{make_asset, AccountIdOf, User, UserBuilder},
    traits::{asset::Config as Asset, TestUtilsFn},
};
use polymesh_primitives::IdentityId;
use sp_std::prelude::*;

fn init_ticker<T: Asset + TestUtilsFn<AccountIdOf<T>>>() -> (User<T>, Ticker) {
    let owner = UserBuilder::<T>::default().generate_did().build("OWNER");
    let ticker = make_asset::<T>(&owner, Some(b"1"));
    (owner, ticker)
}

fn init_ctm<T: Config + Asset + TestUtilsFn<AccountIdOf<T>>>(
    max_transfer_manager_per_asset: u32,
) -> (User<T>, Ticker, Vec<TransferManager>) {
    let (owner, ticker) = init_ticker::<T>();
    let tms = (0..max_transfer_manager_per_asset)
        .map(|x| TransferManager::CountTransferManager(x.into()))
        .collect::<Vec<_>>();
    ActiveTransferManagers::insert(ticker, tms.clone());
    (owner, ticker, tms)
}

#[cfg(feature = "running-ci")]
mod limits {
    pub const MAX_EXEMPTED_IDENTITIES: u32 = 10;
}

#[cfg(not(feature = "running-ci"))]
mod limits {
    pub const MAX_EXEMPTED_IDENTITIES: u32 = 1000;
}

benchmarks! {
    where_clause { where T: Asset, T: TestUtilsFn<AccountIdOf<T>> }

    add_transfer_manager {
        let max_tm = T::MaxTransferManagersPerAsset::get().saturating_sub(1);
        let (owner, ticker, mut tms) = init_ctm::<T>(max_tm);

        let last_tm = TransferManager::CountTransferManager(420);
        tms.push(last_tm.clone());
    }: _(owner.origin, ticker, last_tm)
    verify {
        assert_eq!(Module::<T>::transfer_managers(ticker), tms);
    }

    remove_transfer_manager {
        let (owner, ticker, mut tms) = init_ctm::<T>(T::MaxTransferManagersPerAsset::get());
        let last_tm = tms.pop().expect("MaxTransferManagersPerAsset should be greater than zero");
    }: _(owner.origin, ticker, last_tm)
    verify {
        assert_eq!(Module::<T>::transfer_managers(ticker), tms);
    }

    add_exempted_entities {
        // Length of the vector of Exempted identities being added.
        let i in 0 .. limits::MAX_EXEMPTED_IDENTITIES;

        let (owner, ticker) = init_ticker::<T>();
        let scope_ids = (0..i as u128).map(IdentityId::from).collect::<Vec<_>>();
        let tm = TransferManager::CountTransferManager(420);
        let ephemeral_tm = tm.clone();
    }: _(owner.origin, ticker, ephemeral_tm, scope_ids)
    verify {
        assert!(Module::<T>::entity_exempt((ticker, tm), IdentityId::from(0u128)) == (i != 0));
    }

    remove_exempted_entities {
        // Length of the vector of Exempted identities being removed.
        let i in 0 .. limits::MAX_EXEMPTED_IDENTITIES;

        let (owner, ticker) = init_ticker::<T>();
        let tm = TransferManager::CountTransferManager(420);
        let scope_ids = (0..i).map(|x| {
            let scope_id = IdentityId::from(x as u128);
            ExemptEntities::insert((ticker, tm.clone()), scope_id.clone(), true);
            scope_id
        }).collect::<Vec<_>>();
        let ephemeral_tm = tm.clone();
    }: _(owner.origin, ticker, ephemeral_tm, scope_ids)
    verify {
        assert!(!Module::<T>::entity_exempt((ticker, tm), IdentityId::from(0u128)));
    }

    #[extra]
    verify_tm_restrictions {
        let t in 0 .. T::MaxTransferManagersPerAsset::get();

        let (owner, ticker) = init_ticker::<T>();
        let owner_did = owner.did.unwrap();
        let tms = (0..t).map(|x| {
            let tm = TransferManager::CountTransferManager(x.into());
            ExemptEntities::insert((ticker, tm.clone()), owner_did, true);
            tm
        }).collect::<Vec<_>>();
        ActiveTransferManagers::insert(ticker, tms.clone());
        InvestorCountPerAsset::insert(ticker, 1337);
    }: {
        // This will trigger the worse case (exemption)
        Module::<T>::verify_tm_restrictions(
            &ticker,
            owner_did,
            owner_did,
            100u32.into(),
            200u32.into(),
            0u32.into(),
            500u32.into(),
        ).unwrap();
    }
}
