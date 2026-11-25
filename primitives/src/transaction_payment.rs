use codec::{Decode, Encode, MaxEncodedLen};
use scale_info::TypeInfo;

use crate::Signatory;

#[derive(Clone, Debug, PartialEq, Eq, Encode, Decode, MaxEncodedLen, TypeInfo)]
/// Stores the information needed for the payment of a call.
pub struct CallPaymentInfo<AccountId> {
    /// The account that will pay for the call.
    paying_account: AccountId,
    /// The authorization ID.
    auth_id: Option<u64>,
    /// The multisig signatory.
    ms_signatory: Option<Signatory<AccountId>>,
    /// `true` if the call was wrapped in `Revive::eth_substrate_call`.
    eth_wrapped: bool,
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
            eth_wrapped: false,
        }
    }

    /// Marks the call as wrapped in `Revive::eth_substrate_call`.
    pub fn with_eth_wrapped(mut self, eth_wrapped: bool) -> Self {
        self.eth_wrapped = eth_wrapped;
        self
    }

    /// Returns `true` if the call was wrapped in `Revive::eth_substrate_call`.
    pub fn is_eth_wrapped(&self) -> bool {
        self.eth_wrapped
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
