use sp_runtime::transaction_validity::InvalidTransaction;

// Polymesh note: This was specifically added for Polymesh
pub trait CddAndFeeDetails<AccountId, Call> {
    fn get_valid_payer(
        call: &Call,
        caller: &AccountId,
    ) -> Result<Option<AccountId>, InvalidTransaction>;
    fn clear_context();
    fn set_payer_context(payer: Option<AccountId>);
    fn get_payer_from_context() -> Option<AccountId>;
}
