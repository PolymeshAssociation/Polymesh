use crate::Signatory;

#[derive(Clone, Debug, PartialEq, Eq)]
/// Stores the information needed for the payment of a call.
pub struct CallPaymentInfo<AccountId> {
    /// The account that will pay for the call.
    paying_account: AccountId,
    /// The authorization ID.
    auth_id: Option<u64>,
    /// The multisig signatory.
    ms_signatory: Option<Signatory<AccountId>>,
}

impl<AccountId> CallPaymentInfo<AccountId> {
    /// Creates a new [`CallPaymentInfo`] instance.
    pub fn new(
        paying_account: AccountId,
        auth_id: Option<u64>,
        ms_signatory: Option<Signatory<AccountId>>,
    ) -> Self {
        Self {
            paying_account,
            auth_id,
            ms_signatory,
        }
    }

    /// Returns the account that will pay for the call.
    pub fn paying_account(&self) -> &AccountId {
        &self.paying_account
    }

    /// Returns the authorization ID.
    pub fn auth_id(&self) -> Option<u64> {
        self.auth_id
    }

    /// Returns the multisig signatory.
    pub fn ms_signatory(&self) -> &Option<Signatory<AccountId>> {
        &self.ms_signatory
    }
}
