use crate::IdentityId;
use cryptography::claim_proofs::PedersenGenerators;
use curve25519_dalek::scalar::Scalar;
use polymesh_primitives_derive::SliceU8StrongTyped;

use blake2::{Blake2b, Digest};
use codec::{Decode, Encode};

use sp_std::convert::TryInto;

#[cfg(feature = "std")]
use polymesh_primitives_derive::{DeserializeU8StrongTyped, SerializeU8StrongTyped};

/// The investor UID identifies the legal entity of an investor.
/// It should be keep it encrypted in order to have the investor's portfolio hidden between several
/// Identities Id.
///
/// That UID is generated by any trusted CDD provided, based on the investor's Personal
/// Identifiable Information (PII). That process is driven by the specification of the Polymath
/// Unique Identity System (PUIS).
#[derive(Default, Clone, Copy, Encode, Decode, SliceU8StrongTyped)]
#[cfg_attr(
    feature = "std",
    derive(SerializeU8StrongTyped, DeserializeU8StrongTyped)
)]
pub struct InvestorUID([u8; 32]);

/// It links the investor UID with an specific Identity DID in a way that no one can extract that
/// investor UID from this CDD Id, and the investor can create a Zero Knowledge Proof to prove that
/// an specific DID belows to him.
/// The main purpose of this claim is to keep the privacy of the investor using several identities
/// to handle his portfolio.
#[derive(
    Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Encode, Decode, SliceU8StrongTyped,
)]
#[cfg_attr(
    feature = "std",
    derive(SerializeU8StrongTyped, DeserializeU8StrongTyped)
)]
pub struct CddId([u8; 32]);

impl CddId {
    /// Create a new CDD Id given the `did` and the `investor_uid`.
    /// The blind factor is generated as a `Blake2b` hash of the concatenation of the given `did`
    /// and `investor_uid`.
    pub fn new(did: IdentityId, investor_uid: InvestorUID) -> Self {
        let blind = Blake2b::default()
            .chain(did.as_bytes())
            .chain(investor_uid.as_slice())
            .finalize();
        let blind_plain: [u8; 32] = blind.as_ref().try_into().unwrap_or_else(|_| [0u8; 32]);

        Self::with_blind(did, investor_uid, blind_plain)
    }

    /// Create a new CDD id using an specific blind factor.
    /// # TODO
    ///  - Limit IdentityId & InvestorUID to fit into Scalar (see Scalar::from_bits)
    pub fn with_blind(did: IdentityId, investor_uid: InvestorUID, blind: [u8; 32]) -> Self {
        let uid: [u8; 32] = investor_uid
            .as_slice()
            .try_into()
            .unwrap_or_else(|_| [0u8; 32]);

        let a0 = Scalar::from_bits(did.as_fixed_bytes().clone());
        let a1 = Scalar::from_bits(uid);
        let a2 = Scalar::from_bits(blind);

        let pg = PedersenGenerators::default();
        let values = [a0, a1, a2];
        let commitment = pg.commit(&values);
        let commitment_compressed = commitment.compress().as_bytes().clone();

        CddId(commitment_compressed)
    }

    /// Only the zero-filled `CddId` is considered as invalid.
    pub fn is_valid(&self) -> bool {
        self.0 != [0u8; 32]
    }
}

#[cfg(test)]
mod tests {
    use crate::{CddId, IdentityId, InvestorUID};

    #[test]
    fn cdd_id_generation() {
        let alice_id_1 = IdentityId::from(1);
        let alice_id_2 = IdentityId::from(2);
        let alice_uid = InvestorUID::from(b"alice_uid");

        let alice_cdd_id_1 = CddId::new(alice_id_1, alice_uid);
        let alice_cdd_id_2 = CddId::new(alice_id_2, alice_uid);

        assert!(alice_id_1 != alice_id_2);
        assert!(alice_cdd_id_1 != alice_cdd_id_2);
    }
}
