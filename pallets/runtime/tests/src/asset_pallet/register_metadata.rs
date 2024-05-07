use frame_support::assert_ok;
use frame_support::storage::{StorageDoubleMap, StorageMap, StorageValue};
use sp_keyring::AccountKeyring;

use pallet_asset::{
    AssetMetadataGlobalKeyToName, AssetMetadataGlobalNameToKey, AssetMetadataGlobalSpecs,
    AssetMetadataLocalKeyToName, AssetMetadataLocalNameToKey, AssetMetadataLocalSpecs,
    AssetMetadataNextGlobalKey, AssetMetadataNextLocalKey, CurrentAssetMetadataGlobalKey,
    CurrentAssetMetadataLocalKey,
};
use polymesh_primitives::asset::AssetType;
use polymesh_primitives::asset_metadata::{
    AssetMetadataGlobalKey, AssetMetadataLocalKey, AssetMetadataName, AssetMetadataSpec,
};
use polymesh_primitives::Ticker;

use crate::storage::{root, User};
use crate::{ExtBuilder, TestStorage};

type Asset = pallet_asset::Module<TestStorage>;

#[test]
fn register_multiple_global_metadata() {
    ExtBuilder::default().build().execute_with(|| {
        assert_eq!(CurrentAssetMetadataGlobalKey::get(), None);
        assert_eq!(
            AssetMetadataNextGlobalKey::get(),
            AssetMetadataGlobalKey::default()
        );

        let asset_metadata_name = AssetMetadataName(b"MyGlobalKey".to_vec());
        let asset_metadata_spec = AssetMetadataSpec::default();
        assert_ok!(Asset::register_asset_metadata_global_type(
            root(),
            asset_metadata_name.clone(),
            asset_metadata_spec.clone()
        ));

        assert_eq!(
            CurrentAssetMetadataGlobalKey::get(),
            Some(AssetMetadataGlobalKey(1))
        );
        assert_eq!(AssetMetadataNextGlobalKey::get(), AssetMetadataGlobalKey(1));
        assert_eq!(
            AssetMetadataGlobalNameToKey::get(asset_metadata_name.clone()),
            Some(AssetMetadataGlobalKey(1))
        );
        assert_eq!(
            AssetMetadataGlobalKeyToName::get(AssetMetadataGlobalKey(1)),
            Some(asset_metadata_name)
        );
        assert_eq!(
            AssetMetadataGlobalSpecs::get(AssetMetadataGlobalKey(1)),
            Some(asset_metadata_spec)
        );

        let asset_metadata_name2 = AssetMetadataName(b"MyGlobalKey2".to_vec());
        let asset_metadata_spec2 = AssetMetadataSpec::default();
        assert_ok!(Asset::register_asset_metadata_global_type(
            root(),
            asset_metadata_name2.clone(),
            asset_metadata_spec2.clone()
        ));
        assert_eq!(
            CurrentAssetMetadataGlobalKey::get(),
            Some(AssetMetadataGlobalKey(2))
        );
        assert_eq!(AssetMetadataNextGlobalKey::get(), AssetMetadataGlobalKey(2));
    })
}

#[test]
fn register_multiple_local_metadata() {
    ExtBuilder::default().build().execute_with(|| {
        let alice = User::new(AccountKeyring::Alice);
        let ticker = Ticker::from_slice_truncated(b"TICKER");
        assert_eq!(CurrentAssetMetadataLocalKey::get(ticker), None);
        assert_eq!(
            AssetMetadataNextLocalKey::get(ticker),
            AssetMetadataLocalKey::default()
        );

        assert_ok!(Asset::create_asset(
            alice.origin(),
            ticker.as_ref().into(),
            ticker,
            true,
            AssetType::Derivative,
            Vec::new(),
            None,
        ));

        let asset_metadata_name = AssetMetadataName(b"MyLocalKey".to_vec());
        let asset_metadata_spec = AssetMetadataSpec::default();
        assert_ok!(Asset::register_asset_metadata_local_type(
            alice.origin(),
            ticker,
            asset_metadata_name.clone(),
            asset_metadata_spec.clone()
        ));

        assert_eq!(
            CurrentAssetMetadataLocalKey::get(ticker),
            Some(AssetMetadataLocalKey(1))
        );
        assert_eq!(
            AssetMetadataNextLocalKey::get(ticker),
            AssetMetadataLocalKey(1)
        );
        assert_eq!(
            AssetMetadataLocalNameToKey::get(ticker, asset_metadata_name.clone()),
            Some(AssetMetadataLocalKey(1))
        );
        assert_eq!(
            AssetMetadataLocalKeyToName::get(ticker, AssetMetadataLocalKey(1)),
            Some(asset_metadata_name)
        );
        assert_eq!(
            AssetMetadataLocalSpecs::get(ticker, AssetMetadataLocalKey(1)),
            Some(asset_metadata_spec)
        );

        let asset_metadata_name2 = AssetMetadataName(b"MyGlobalKey2".to_vec());
        let asset_metadata_spec2 = AssetMetadataSpec::default();
        assert_ok!(Asset::register_asset_metadata_local_type(
            alice.origin(),
            ticker,
            asset_metadata_name2.clone(),
            asset_metadata_spec2.clone()
        ));
        assert_eq!(
            CurrentAssetMetadataLocalKey::get(ticker),
            Some(AssetMetadataLocalKey(2))
        );
        assert_eq!(
            AssetMetadataNextLocalKey::get(ticker),
            AssetMetadataLocalKey(2)
        );
    })
}
