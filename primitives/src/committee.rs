/// The maximum number of members in a committee defined for the sake of weight computation.  This
/// is not defined as a trait parameter but rather as a plain constant because this value has to be
/// the same for all instances.
pub const COMMITTEE_MEMBERS_MAX: u32 = 500;