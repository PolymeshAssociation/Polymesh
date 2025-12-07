use codec::{Decode, DecodeWithMemTracking, Encode};
use polymesh_primitives_derive::SliceU8StrongTyped;
use polymesh_primitives_derive::{DeserializeU8StrongTyped, SerializeU8StrongTyped};
use scale_info::TypeInfo;

/// A CDD ID only has meaning to the CDD provider that issues a CDD claim.
#[derive(Encode, Decode, DecodeWithMemTracking, TypeInfo, SliceU8StrongTyped)]
#[derive(Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[derive(SerializeU8StrongTyped, DeserializeU8StrongTyped)]
pub struct CddId([u8; 32]);

impl CddId {
    /// Check if the CddId is the default value (all zeros).
    pub fn is_default_cdd(&self) -> bool {
        *self == Self::default()
    }
}

impl From<[u8; 32]> for CddId {
    #[inline]
    fn from(data: [u8; 32]) -> Self {
        Self(data)
    }
}

#[cfg(test)]
mod tests {
    use crate::CddId;

    #[test]
    fn cdd_id_is_default() {
        let cdd_id = CddId::default();
        assert!(cdd_id.is_default_cdd());

        let cdd_id = CddId::from([0u8; 32]);
        assert!(cdd_id.is_default_cdd());
    }
}
