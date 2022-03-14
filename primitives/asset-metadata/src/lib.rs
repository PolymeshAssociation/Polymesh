use codec::{Decode, Encode};
use scale_info::TypeInfo;

/// ISO 8601 time duration with nanosecond precision. This also allows for the negative duration; see individual methods for details.
#[derive(Encode, Decode, TypeInfo)]
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Duration {
    pub secs: i64,
    pub nanos: i32,
}

/// Simple Date.
#[derive(Encode, Decode, TypeInfo)]
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Date {
    pub year: u16,
    pub month: u8,
    pub day: u8,
}

/// Simple Time.
#[derive(Encode, Decode, TypeInfo)]
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Time {
    pub hour: u8,
    pub minute: u8,
    pub second: u8,
}

/// Simple DateTime.
#[derive(Encode, Decode, TypeInfo)]
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct DateTime {
    pub date: Date,
    pub time: Time,
}

#[cfg(test)]
mod tests {
    use super::*;
    use polymesh_primitives::asset_metadata::{AssetMetadataSpec, AssetMetadataTypeDef};

    /// Asset Metadata Test Type.
    ///
    /// Wrap a few `TypeInfo` types for testing.
    #[derive(Encode, Decode, TypeInfo)]
    #[derive(Clone, Debug, Default, PartialEq, Eq)]
    struct MetadataTestType1 {
        duration: Duration,
        date: Date,
        time: Time,
        date_time: DateTime,
    }

    #[test]
    fn encode_decode_metadata_test() {
        let type_def = AssetMetadataTypeDef::new_from_type::<MetadataTestType1>();
        let mut spec = AssetMetadataSpec::default();
        // Encode type definition.
        spec.set_type_def(type_def.clone());
        println!("Type definition length: {}", spec.type_def_len());
        // Decode type definition.
        let type_def2 = spec
            .decode_type_def()
            .expect("Failed to decode type def.")
            .expect("Missing type def.");
        assert_eq!(type_def, type_def2);
    }
}
