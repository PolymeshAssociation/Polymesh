use crate::types::{Claim1stKey, Claim2ndKey};

use polymesh_primitives::{CddId, Claim, ClaimType, IdentityClaim};
use sp_io::hashing::blake2_128;

/// Migrate claim
pub fn migrate_claim(
    k1: Claim1stKey,
    k2: Claim2ndKey,
    id_claim: IdentityClaim,
) -> Option<IdentityClaim> {
    match &k1.claim_type {
        ClaimType::CustomerDueDiligence => migrate_cdd_claim(&k1, &k2, id_claim),
        _ => Some(id_claim),
    }
}

fn migrate_cdd_claim(
    k1: &Claim1stKey,
    k2: &Claim2ndKey,
    id_claim: IdentityClaim,
) -> Option<IdentityClaim> {
    let uid = blake2_128(k1.target.as_bytes()).into();
    let cdd_id = CddId::new(k1.target, uid);
    let new_claim = Claim::CustomerDueDiligence(cdd_id);

    let mut id_claim = id_claim;
    id_claim.claim = new_claim;

    Some(id_claim)
}
