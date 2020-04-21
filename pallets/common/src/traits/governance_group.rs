use crate::traits::group::GroupTrait;

use polymesh_primitives::IdentityId;

pub trait GovernanceGroupTrait<Moment: PartialOrd + Copy>: GroupTrait<Moment> {
    fn release_coordinator() -> Option<IdentityId>;
}
