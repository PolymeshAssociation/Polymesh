use frame_support::dispatch::DispatchResult;
use sp_std::vec::Vec;
/// Trait used by the off-chain worker to interact with staking pallet.
pub trait OffchainInterface<AccountId> {
    /// Returns a list of nominator's identities whom CDD claim get expired.
    ///
    /// # Argument
    /// * buffer - Leeway of no. of seconds.
    fn fetch_invalid_cdd_nominators(buffer: u64) -> Vec<AccountId>;

    /// Remove the list of nominators whom CDD claim has expired.
    ///
    /// # Argument
    /// * targets - List of nominator stash key.
    fn unchecked_remove_expired_cdd_nominators(targets: &Vec<AccountId>) -> DispatchResult;
}
