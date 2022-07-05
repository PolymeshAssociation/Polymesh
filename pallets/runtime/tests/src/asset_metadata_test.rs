use super::{
    asset_test::create_token,
    exec_noop, exec_ok,
    storage::{TestStorage, User},
    ExtBuilder,
};
use frame_support::{assert_noop, assert_ok, dispatch::DispatchError};
use polymesh_primitives::{
    asset_metadata::{
        AssetMetadataKey, AssetMetadataLockStatus, AssetMetadataName, AssetMetadataSpec,
        AssetMetadataValue, AssetMetadataValueDetail,
    },
    Ticker,
};
use test_client::AccountKeyring;

type Origin = <TestStorage as frame_system::Config>::Origin;
type Moment = <TestStorage as pallet_timestamp::Config>::Moment;
type Timestamp = pallet_timestamp::Pallet<TestStorage>;

type MaxLen = <TestStorage as pallet_base::Config>::MaxLen;
type AssetMetadataNameMaxLength = <TestStorage as pallet_asset::Config>::AssetMetadataNameMaxLength;
type AssetMetadataValueMaxLength =
    <TestStorage as pallet_asset::Config>::AssetMetadataValueMaxLength;
type AssetMetadataTypeDefMaxLength =
    <TestStorage as pallet_asset::Config>::AssetMetadataTypeDefMaxLength;

type Asset = pallet_asset::Module<TestStorage>;

type BaseError = pallet_base::Error<TestStorage>;
type EAError = pallet_external_agents::Error<TestStorage>;
type AssetError = pallet_asset::Error<TestStorage>;

/// Make metadata name & specs using the requested lengths.
fn make_metadata_type_sizes(
    name_len: u32,
    url_len: u32,
    desc_len: u32,
    type_def_len: u32,
) -> (AssetMetadataName, AssetMetadataSpec) {
    let spec = AssetMetadataSpec {
        url: Some(vec![b'u'; url_len as usize].into()),
        description: Some(vec![b'd'; desc_len as usize].into()),
        type_def: Some(vec![b't'; type_def_len as usize]),
    };
    let name = AssetMetadataName(vec![b'n'; name_len as usize]);

    (name, spec)
}

/// Helper for creating metadata value details.
fn make_metadata_value_details(
    expire: Option<Moment>,
    locked: bool,
) -> AssetMetadataValueDetail<Moment> {
    AssetMetadataValueDetail {
        expire,
        lock_status: if locked {
            AssetMetadataLockStatus::Locked
        } else {
            AssetMetadataLockStatus::Unlocked
        },
    }
}

/// Make metadata name & spec for the given name.
fn make_metadata_type(name: &str) -> (AssetMetadataName, AssetMetadataSpec) {
    let spec = AssetMetadataSpec {
        url: Some(b"http://example.com/test_specs".into()),
        description: Some(format!("{} metadata type", name).as_bytes().into()),
        type_def: Some(vec![]),
    };
    let name: AssetMetadataName = name.as_bytes().into();

    (name, spec)
}

/// Helper to register metadata type with the give name.
fn register_metadata_type(owner: User, ticker: Option<Ticker>, name: &str) -> AssetMetadataKey {
    let (name, spec) = make_metadata_type(name);

    if let Some(ticker) = ticker {
        // Register local metadata type with asset owner.
        exec_ok!(Asset::register_asset_metadata_local_type(
            owner.origin(),
            ticker,
            name.clone(),
            spec,
        ));

        Asset::asset_metadata_local_name_to_key(ticker, name)
            .map(AssetMetadataKey::from)
            .expect("Failed to register metadata")
    } else {
        let root = frame_system::RawOrigin::Root;
        // Register global metadata type with root.
        assert_ok!(Asset::register_asset_metadata_global_type(
            root.into(),
            name.clone(),
            spec,
        ));

        Asset::asset_metadata_global_name_to_key(name)
            .map(AssetMetadataKey::from)
            .expect("Failed to register metadata")
    }
}

#[test]
fn set_asset_metadata_local_type() {
    ExtBuilder::default().build().execute_with(|| {
        let owner = User::new(AccountKeyring::Dave);
        let other = User::new(AccountKeyring::Alice);

        // Create asset.
        let (ticker, _) = create_token(owner);

        let global_key = register_metadata_type(owner, None, "TEST");
        let local_key = register_metadata_type(owner, Some(ticker), "TEST");

        let value = AssetMetadataValue("cow".as_bytes().into());
        let details = Some(make_metadata_value_details(None, false));

        // Try adding global metadata value with user that doesn't have permissions for this asset.
        exec_noop!(
            Asset::set_asset_metadata(
                other.origin(),
                ticker,
                global_key,
                value.clone(),
                details.clone()
            ),
            EAError::UnauthorizedAgent
        );

        // Try adding local metadata value with user that doesn't have permissions for this asset.
        exec_noop!(
            Asset::set_asset_metadata(
                other.origin(),
                ticker,
                local_key,
                value.clone(),
                details.clone()
            ),
            EAError::UnauthorizedAgent
        );

        // Make value that exceeds the maximum limit.
        let value_len = AssetMetadataValueMaxLength::get() + 1;
        let over_sized_value = AssetMetadataValue(vec![b'v'; value_len as usize]);

        // Try to set a global key with a value that exceeds the maximum limit.
        exec_noop!(
            Asset::set_asset_metadata(
                owner.origin(),
                ticker,
                global_key,
                over_sized_value.clone(),
                details.clone()
            ),
            AssetError::AssetMetadataValueMaxLengthExceeded
        );

        // Try to set a local key with a value that exceeds the maximum limit.
        exec_noop!(
            Asset::set_asset_metadata(
                owner.origin(),
                ticker,
                local_key,
                over_sized_value.clone(),
                details.clone()
            ),
            AssetError::AssetMetadataValueMaxLengthExceeded
        );

        // Set metadata value for global key.
        exec_ok!(Asset::set_asset_metadata(
            owner.origin(),
            ticker,
            global_key,
            value.clone(),
            details.clone()
        ));

        // Set metadata value for local key.
        exec_ok!(Asset::set_asset_metadata(
            owner.origin(),
            ticker,
            local_key,
            value.clone(),
            details.clone()
        ));

        let value2 = AssetMetadataValue("beef".as_bytes().into());
        let details2 = Some(make_metadata_value_details(Some(1), false));

        // Try updating global metadata value with user that doesn't have permissions for this asset.
        exec_noop!(
            Asset::set_asset_metadata(
                other.origin(),
                ticker,
                global_key,
                value2.clone(),
                details2.clone()
            ),
            EAError::UnauthorizedAgent
        );

        // Try updating local metadata value with user that doesn't have permissions for this asset.
        exec_noop!(
            Asset::set_asset_metadata(
                other.origin(),
                ticker,
                local_key,
                value2.clone(),
                details2.clone()
            ),
            EAError::UnauthorizedAgent
        );

        // Update metadata value for global key.
        exec_ok!(Asset::set_asset_metadata(
            owner.origin(),
            ticker,
            global_key,
            value2.clone(),
            details2.clone(),
        ));

        // Update metadata value for local key.
        exec_ok!(Asset::set_asset_metadata(
            owner.origin(),
            ticker,
            local_key,
            value2.clone(),
            details2.clone(),
        ));

        let details_locked = make_metadata_value_details(None, true);
        // Try locking global metadata value with user that doesn't have permissions for this asset.
        exec_noop!(
            Asset::set_asset_metadata_details(
                other.origin(),
                ticker,
                global_key,
                details_locked.clone()
            ),
            EAError::UnauthorizedAgent
        );

        // Try locking local metadata value with user that doesn't have permissions for this asset.
        exec_noop!(
            Asset::set_asset_metadata_details(
                other.origin(),
                ticker,
                local_key,
                details_locked.clone()
            ),
            EAError::UnauthorizedAgent
        );

        // Lock metadata value for global key.
        exec_ok!(Asset::set_asset_metadata_details(
            owner.origin(),
            ticker,
            global_key,
            details_locked.clone(),
        ));

        // Lock metadata value for local key.
        exec_ok!(Asset::set_asset_metadata_details(
            owner.origin(),
            ticker,
            local_key,
            details_locked.clone(),
        ));

        let value3 = AssetMetadataValue("deadbeef".as_bytes().into());
        let details3 = Some(make_metadata_value_details(Some(3), false));

        // Try updating locked metadata value for global key.
        exec_noop!(
            Asset::set_asset_metadata(
                owner.origin(),
                ticker,
                global_key,
                value3.clone(),
                details3.clone(),
            ),
            AssetError::AssetMetadataValueIsLocked
        );

        // Try updating locked metadata value for local key.
        exec_noop!(
            Asset::set_asset_metadata(
                owner.origin(),
                ticker,
                local_key,
                value3.clone(),
                details3.clone(),
            ),
            AssetError::AssetMetadataValueIsLocked
        );
    });
}

#[test]
fn register_and_set_local_asset_metadata() {
    ExtBuilder::default().build().execute_with(|| {
        let owner = User::new(AccountKeyring::Dave);
        let other = User::new(AccountKeyring::Alice);

        // Create asset.
        let (ticker, _) = create_token(owner);

        let (name, spec) = make_metadata_type("TEST");
        let value = AssetMetadataValue("cow".as_bytes().into());
        let details = Some(make_metadata_value_details(None, false));

        // Try registering local metadata type with user that doesn't have permissions for this asset.
        exec_noop!(
            Asset::register_and_set_local_asset_metadata(
                other.origin(),
                ticker,
                name.clone(),
                spec.clone(),
                value.clone(),
                details.clone(),
            ),
            EAError::UnauthorizedAgent
        );

        // Register and set local metadata type with asset owner.
        exec_ok!(Asset::register_and_set_local_asset_metadata(
            owner.origin(),
            ticker,
            name.clone(),
            spec.clone(),
            value.clone(),
            details.clone(),
        ));

        // Try registering and setting metadata with the same name.
        exec_noop!(
            Asset::register_and_set_local_asset_metadata(
                owner.origin(),
                ticker,
                name,
                spec,
                value,
                details
            ),
            AssetError::AssetMetadataLocalKeyAlreadyExists
        );
    });
}

#[test]
fn register_asset_metadata_local_type() {
    ExtBuilder::default().build().execute_with(|| {
        let owner = User::new(AccountKeyring::Dave);
        let other = User::new(AccountKeyring::Alice);

        // Create asset.
        let (ticker, _) = create_token(owner);

        let (name, spec) = make_metadata_type("TEST");

        // Try registering local metadata type with user that doesn't have permissions for this asset.
        exec_noop!(
            Asset::register_asset_metadata_local_type(
                other.origin(),
                ticker,
                name.clone(),
                spec.clone(),
            ),
            EAError::UnauthorizedAgent
        );

        // Register local metadata type with asset owner.
        exec_ok!(Asset::register_asset_metadata_local_type(
            owner.origin(),
            ticker,
            name.clone(),
            spec.clone(),
        ));

        // Try registering metadata with the same name.
        exec_noop!(
            Asset::register_asset_metadata_local_type(owner.origin(), ticker, name, spec,),
            AssetError::AssetMetadataLocalKeyAlreadyExists
        );
    });
}

#[test]
fn register_asset_metadata_global_type() {
    ExtBuilder::default().build().execute_with(|| {
        let alice = User::new(AccountKeyring::Alice);
        let root = Origin::from(frame_system::RawOrigin::Root);

        let (name, spec) = make_metadata_type("TEST");

        // Try registering global metadata type with non-root.
        exec_noop!(
            Asset::register_asset_metadata_global_type(alice.origin(), name.clone(), spec.clone(),),
            DispatchError::BadOrigin
        );

        // Register global metadata type with root.
        assert_ok!(Asset::register_asset_metadata_global_type(
            root.clone(),
            name.clone(),
            spec.clone(),
        ));

        // Try registering metadata with the same name.
        assert_noop!(
            Asset::register_asset_metadata_global_type(root, name, spec,),
            AssetError::AssetMetadataGlobalKeyAlreadyExists
        );
    });
}

#[test]
fn register_asset_metadata_local_type_limits() {
    ExtBuilder::default().build().execute_with(|| {
        let owner = User::new(AccountKeyring::Dave);

        // Create asset.
        let (ticker, _) = create_token(owner);

        // Try registering metadata with over-sized values.
        let register_type = |name, url, desc, type_def, err: DispatchError| {
            let (name, spec) = make_metadata_type_sizes(name, url, desc, type_def);
            exec_noop!(
                Asset::register_asset_metadata_local_type(owner.origin(), ticker, name, spec,),
                err
            );
        };

        // Oversized metadata name.
        register_type(
            AssetMetadataNameMaxLength::get() + 1,
            10,
            10,
            10,
            AssetError::AssetMetadataNameMaxLengthExceeded.into(),
        );

        // Oversized metadata url.
        register_type(10, MaxLen::get() + 1, 10, 10, BaseError::TooLong.into());

        // Oversized metadata description.
        register_type(10, 10, MaxLen::get() + 1, 10, BaseError::TooLong.into());

        // Oversized metadata type definition.
        register_type(
            10,
            10,
            10,
            AssetMetadataTypeDefMaxLength::get() + 1,
            AssetError::AssetMetadataTypeDefMaxLengthExceeded.into(),
        );
    });
}

#[test]
fn register_asset_metadata_global_type_limits() {
    ExtBuilder::default().build().execute_with(|| {
        let root = Origin::from(frame_system::RawOrigin::Root);

        // Try registering metadata with over-sized values.
        let register_type = |name, url, desc, type_def, err: DispatchError| {
            let (name, spec) = make_metadata_type_sizes(name, url, desc, type_def);
            assert_noop!(
                Asset::register_asset_metadata_global_type(root.clone(), name, spec,),
                err
            );
        };

        // Oversized metadata name.
        register_type(
            AssetMetadataNameMaxLength::get() + 1,
            10,
            10,
            10,
            AssetError::AssetMetadataNameMaxLengthExceeded.into(),
        );

        // Oversized metadata url.
        register_type(10, MaxLen::get() + 1, 10, 10, BaseError::TooLong.into());

        // Oversized metadata description.
        register_type(10, 10, MaxLen::get() + 1, 10, BaseError::TooLong.into());

        // Oversized metadata type definition.
        register_type(
            10,
            10,
            10,
            AssetMetadataTypeDefMaxLength::get() + 1,
            AssetError::AssetMetadataTypeDefMaxLengthExceeded.into(),
        );
    });
}

#[test]
fn check_locked_until() {
    ExtBuilder::default().build().execute_with(|| {
        let owner = User::new(AccountKeyring::Dave);

        // Create asset.
        let (ticker, _) = create_token(owner);

        let global_key = register_metadata_type(owner, None, "TEST");
        let local_key = register_metadata_type(owner, Some(ticker), "TEST");

        let value = AssetMetadataValue("cow".as_bytes().into());
        let details = Some(make_metadata_value_details(None, false));

        // Set metadata value for global key.
        exec_ok!(Asset::set_asset_metadata(
            owner.origin(),
            ticker,
            global_key,
            value.clone(),
            details.clone()
        ));

        // Set metadata value for local key.
        exec_ok!(Asset::set_asset_metadata(
            owner.origin(),
            ticker,
            local_key,
            value.clone(),
            details.clone()
        ));

        let unlock_timestamp = Timestamp::now() + 1_000_000_000;
        let details_locked_until = AssetMetadataValueDetail {
            expire: None,
            lock_status: AssetMetadataLockStatus::LockedUntil(unlock_timestamp),
        };

        // Lock metadata value for global key until `unlock_timestamp`.
        exec_ok!(Asset::set_asset_metadata_details(
            owner.origin(),
            ticker,
            global_key,
            details_locked_until.clone(),
        ));

        // Lock metadata value for local key until `unlock_timestamp`.
        exec_ok!(Asset::set_asset_metadata_details(
            owner.origin(),
            ticker,
            local_key,
            details_locked_until.clone(),
        ));

        let value2 = AssetMetadataValue("deadbeef".as_bytes().into());
        let details2 = Some(make_metadata_value_details(None, false));

        // Try updating locked metadata value for global key.
        exec_noop!(
            Asset::set_asset_metadata(
                owner.origin(),
                ticker,
                global_key,
                value2.clone(),
                details2.clone(),
            ),
            AssetError::AssetMetadataValueIsLocked
        );

        // Try updating locked metadata value for local key.
        exec_noop!(
            Asset::set_asset_metadata(
                owner.origin(),
                ticker,
                local_key,
                value2.clone(),
                details2.clone(),
            ),
            AssetError::AssetMetadataValueIsLocked
        );

        // Move time forward until after `unlock_timestamp`.
        Timestamp::set_timestamp(unlock_timestamp + 1_000);

        // Updated unlocked metadata value for global key.
        exec_ok!(Asset::set_asset_metadata(
            owner.origin(),
            ticker,
            global_key,
            value2.clone(),
            details2.clone()
        ));

        // Updated unlocked metadata value for local key.
        exec_ok!(Asset::set_asset_metadata(
            owner.origin(),
            ticker,
            local_key,
            value2.clone(),
            details2.clone()
        ));
    });
}
