use pallet_permissions::StoreCallMetadata;
use pallet_transaction_payment::{ChargeTransactionPayment, Trait as TxPaymentTrait};
use polymesh_common_utilities::traits::permissions::PermissionChecker;

use codec::{Decode, Encode};
use frame_support::{
    dispatch::DispatchResult,
    traits::{Currency, GetCallMetadata},
    weights::{DispatchInfo, PostDispatchInfo},
};
use frame_system::{
    CheckEra, CheckGenesis, CheckNonce, CheckSpecVersion, CheckTxVersion, CheckWeight,
    Trait as SystemTrait,
};
use sp_runtime::{
    generic::Era,
    traits::{DispatchInfoOf, Dispatchable, PostDispatchInfoOf, SignedExtension},
    transaction_validity::{TransactionValidity, TransactionValidityError},
    FixedPointOperand,
};
use sp_std::fmt::Debug;

pub type BalanceOf<T> =
    <<T as TxPaymentTrait>::Currency as Currency<<T as SystemTrait>::AccountId>>::Balance;

#[derive(Debug, Eq, PartialEq, Clone, Encode, Decode)]
pub struct SignedExtra<T>
where
    T: SystemTrait + TxPaymentTrait + PermissionChecker + Send + Sync + Debug,
{
    check_spec_version: CheckSpecVersion<T>,
    check_tx_version: CheckTxVersion<T>,
    check_genesis: CheckGenesis<T>,
    check_era: CheckEra<T>,
    check_nonce: CheckNonce<T>,
    check_weight: CheckWeight<T>,

    charge_tx_payment: ChargeTransactionPayment<T>,
    store_call_metadata: StoreCallMetadata<T>,
}

impl<T> SignedExtra<T>
where
    T: SystemTrait + TxPaymentTrait + PermissionChecker + Send + Sync + Debug,
    <T as SystemTrait>::Call:
        Dispatchable<Info = DispatchInfo, PostInfo = PostDispatchInfo> + GetCallMetadata + Clone,
    BalanceOf<T>: Send + Sync + FixedPointOperand,
{
    pub fn new(curr_block: u64, period: u64, nonce: T::Index, tip: BalanceOf<T>) -> Self {
        Self {
            check_spec_version: CheckSpecVersion::new(),
            check_tx_version: CheckTxVersion::new(),
            check_genesis: CheckGenesis::new(),
            check_era: CheckEra::from(Era::mortal(period, curr_block)),
            check_nonce: CheckNonce::from(nonce),
            check_weight: CheckWeight::new(),
            charge_tx_payment: ChargeTransactionPayment::from(tip),
            store_call_metadata: StoreCallMetadata::new(),
        }
    }
}

impl<T> SignedExtension for SignedExtra<T>
where
    T: SystemTrait + TxPaymentTrait + PermissionChecker + Send + Sync + Debug,
    <T as SystemTrait>::Call:
        Dispatchable<Info = DispatchInfo, PostInfo = PostDispatchInfo> + GetCallMetadata + Clone,
    BalanceOf<T>: Send + Sync + FixedPointOperand + From<u64>,
{
    const IDENTIFIER: &'static str = "RuntimeSignedExtra";
    type AccountId = T::AccountId;
    type Call = <T as SystemTrait>::Call;

    type AdditionalSigned = (
        <CheckSpecVersion<T> as SignedExtension>::AdditionalSigned,
        <CheckTxVersion<T> as SignedExtension>::AdditionalSigned,
        <CheckGenesis<T> as SignedExtension>::AdditionalSigned,
        <CheckEra<T> as SignedExtension>::AdditionalSigned,
        <CheckNonce<T> as SignedExtension>::AdditionalSigned,
        <CheckWeight<T> as SignedExtension>::AdditionalSigned,
        <ChargeTransactionPayment<T> as SignedExtension>::AdditionalSigned,
        <StoreCallMetadata<T> as SignedExtension>::AdditionalSigned,
    );
    type Pre = (
        <CheckSpecVersion<T> as SignedExtension>::Pre,
        <CheckTxVersion<T> as SignedExtension>::Pre,
        <CheckGenesis<T> as SignedExtension>::Pre,
        <CheckEra<T> as SignedExtension>::Pre,
        <CheckNonce<T> as SignedExtension>::Pre,
        <CheckWeight<T> as SignedExtension>::Pre,
        <ChargeTransactionPayment<T> as SignedExtension>::Pre,
        <StoreCallMetadata<T> as SignedExtension>::Pre,
    );

    fn additional_signed(&self) -> Result<Self::AdditionalSigned, TransactionValidityError> {
        let additional = (
            self.check_spec_version.additional_signed()?,
            self.check_tx_version.additional_signed()?,
            self.check_genesis.additional_signed()?,
            self.check_era.additional_signed()?,
            self.check_nonce.additional_signed()?,
            self.check_weight.additional_signed()?,
            self.charge_tx_payment.additional_signed()?,
            self.store_call_metadata.additional_signed()?,
        );
        Ok(additional)
    }

    /// It combines the `TransactionValidity` for each internal signed extension, and overwrites
    /// the priority with the output from `ChargeTransactionPayment`.
    /// That overwritten is needed because the combination of `priorities`, defined inside
    /// `combine_with`, is addiction.
    fn validate(
        &self,
        who: &Self::AccountId,
        call: &Self::Call,
        info: &DispatchInfoOf<Self::Call>,
        len: usize,
    ) -> TransactionValidity {
        let charge_tx_payment_valid = self.charge_tx_payment.validate(who, call, info, len)?;
        let priority = charge_tx_payment_valid.priority;

        let mut valid = self
            .check_spec_version
            .validate(who, call, info, len)?
            .combine_with(self.check_tx_version.validate(who, call, info, len)?)
            .combine_with(self.check_genesis.validate(who, call, info, len)?)
            .combine_with(self.check_era.validate(who, call, info, len)?)
            .combine_with(self.check_nonce.validate(who, call, info, len)?)
            .combine_with(self.check_weight.validate(who, call, info, len)?)
            .combine_with(charge_tx_payment_valid);

        // Overwrite priority from `ChargeTransactionPayment`.
        valid.priority = priority;

        Ok(valid)
    }

    fn pre_dispatch(
        self,
        who: &Self::AccountId,
        call: &Self::Call,
        info: &DispatchInfoOf<Self::Call>,
        len: usize,
    ) -> Result<Self::Pre, TransactionValidityError> {
        let pre = (
            self.check_spec_version.pre_dispatch(who, call, info, len)?,
            self.check_tx_version.pre_dispatch(who, call, info, len)?,
            self.check_genesis.pre_dispatch(who, call, info, len)?,
            self.check_era.pre_dispatch(who, call, info, len)?,
            self.check_nonce.pre_dispatch(who, call, info, len)?,
            self.check_weight.pre_dispatch(who, call, info, len)?,
            self.charge_tx_payment.pre_dispatch(who, call, info, len)?,
            self.store_call_metadata
                .pre_dispatch(who, call, info, len)?,
        );

        Ok(pre)
    }

    fn validate_unsigned(
        call: &Self::Call,
        info: &DispatchInfoOf<Self::Call>,
        len: usize,
    ) -> TransactionValidity {
        let charge_tx_payment_valid =
            ChargeTransactionPayment::<T>::validate_unsigned(call, info, len)?;
        let priority = charge_tx_payment_valid.priority;

        let mut valid = CheckSpecVersion::<T>::validate_unsigned(call, info, len)?
            .combine_with(CheckTxVersion::<T>::validate_unsigned(call, info, len)?)
            .combine_with(CheckGenesis::<T>::validate_unsigned(call, info, len)?)
            .combine_with(CheckEra::<T>::validate_unsigned(call, info, len)?)
            .combine_with(CheckNonce::<T>::validate_unsigned(call, info, len)?)
            .combine_with(CheckWeight::<T>::validate_unsigned(call, info, len)?)
            .combine_with(charge_tx_payment_valid)
            .combine_with(StoreCallMetadata::<T>::validate_unsigned(call, info, len)?);

        // Overwrite priority from `ChargeTransactionPayment`.
        valid.priority = priority;

        Ok(valid)
    }

    fn pre_dispatch_unsigned(
        call: &Self::Call,
        info: &DispatchInfoOf<Self::Call>,
        len: usize,
    ) -> Result<Self::Pre, TransactionValidityError> {
        let pre_unsigned = (
            CheckSpecVersion::<T>::pre_dispatch_unsigned(call, info, len)?,
            CheckTxVersion::<T>::pre_dispatch_unsigned(call, info, len)?,
            CheckGenesis::<T>::pre_dispatch_unsigned(call, info, len)?,
            CheckEra::<T>::pre_dispatch_unsigned(call, info, len)?,
            CheckNonce::<T>::pre_dispatch_unsigned(call, info, len)?,
            CheckWeight::<T>::pre_dispatch_unsigned(call, info, len)?,
            ChargeTransactionPayment::<T>::pre_dispatch_unsigned(call, info, len)?,
            StoreCallMetadata::<T>::pre_dispatch_unsigned(call, info, len)?,
        );

        Ok(pre_unsigned)
    }

    fn post_dispatch(
        pre: Self::Pre,
        info: &DispatchInfoOf<Self::Call>,
        post_info: &PostDispatchInfoOf<Self::Call>,
        len: usize,
        result: &DispatchResult,
    ) -> Result<(), TransactionValidityError> {
        CheckSpecVersion::<T>::post_dispatch(pre.0, info, post_info, len, result)?;
        CheckTxVersion::<T>::post_dispatch(pre.1, info, post_info, len, result)?;
        CheckGenesis::<T>::post_dispatch(pre.2, info, post_info, len, result)?;
        CheckEra::<T>::post_dispatch(pre.3, info, post_info, len, result)?;
        CheckNonce::<T>::post_dispatch(pre.4, info, post_info, len, result)?;
        CheckWeight::<T>::post_dispatch(pre.5, info, post_info, len, result)?;
        ChargeTransactionPayment::<T>::post_dispatch(pre.6, info, post_info, len, result)?;
        StoreCallMetadata::<T>::post_dispatch(pre.7, info, post_info, len, result)?;

        Ok(())
    }
}
