use polymesh_primitives::traits::IdentityFnTrait;
use sp_std::marker::PhantomData;

/// Helper class to access to some context information.
/// Currently it allows to access to
///     - `current_payer throught an `IdentityFnTrait`, because it is stored using extrinsics.
#[derive(Default)]
pub struct Context<AccountId> {
    _marker: PhantomData<AccountId>,
}

impl<AccountId> Context<AccountId> {
    #[inline]
    pub fn current_payer<I: IdentityFnTrait<AccountId>>() -> Option<AccountId> {
        I::current_payer()
    }

    #[inline]
    pub fn set_current_payer<I: IdentityFnTrait<AccountId>>(payer: Option<AccountId>) {
        I::set_current_payer(payer)
    }
}
