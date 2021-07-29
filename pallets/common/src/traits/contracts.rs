use crate::traits::{base, identity};
use frame_support::decl_event;
use frame_support::traits::Get;
use frame_support::weights::Weight;
use pallet_contracts::BalanceOf;
use polymesh_primitives::{IdentityId, MetaUrl};
use sp_runtime::Perbill;

pub trait WeightInfo {
    fn put_code(l: u32, u: u32, d: u32) -> Weight;
    fn instantiate() -> Weight;
    fn freeze_instantiation() -> Weight;
    fn unfreeze_instantiation() -> Weight;
    fn transfer_template_ownership() -> Weight;
    fn change_template_fees() -> Weight;
    fn change_template_meta_url(u: u32) -> Weight;
    fn update_schedule() -> Weight;
    fn set_put_code_flag() -> Weight;
}

pub trait Trait: pallet_contracts::Trait + identity::Trait + base::Trait {
    /// Event type
    type Event: From<Event<Self>> + Into<<Self as frame_system::Trait>::Event>;
    /// Percentage distribution of instantiation fee to the validators and treasury.
    type NetworkShareInFee: Get<Perbill>;
    /// Weight information for extrinsic in this pallet.
    type WeightInfo: WeightInfo;
}

decl_event! {
    pub enum Event<T>
        where
        Balance = BalanceOf<T>,
        CodeHash = <T as frame_system::Trait>::Hash,
    {
        /// Emitted when instantiation fee of a template get changed.
        /// IdentityId of the owner, Code hash of the template, old instantiation fee, new instantiation fee.
        InstantiationFeeChanged(IdentityId, CodeHash, Balance, Balance),
        /// Emitted when the instantiation of the template get frozen.
        /// IdentityId of the owner, Code hash of the template.
        InstantiationFreezed(IdentityId, CodeHash),
        /// Emitted when the instantiation of the template gets un-frozen.
        /// IdentityId of the owner, Code hash of the template.
        InstantiationUnFreezed(IdentityId, CodeHash),
        /// Emitted when the template ownership get transferred.
        /// IdentityId of the owner, Code hash of the template, IdentityId of the new owner of the template.
        TemplateOwnershipTransferred(IdentityId, CodeHash, IdentityId),
        /// Emitted when the template usage fees gets changed.
        /// IdentityId of the owner, Code hash of the template,Old usage fee, New usage fee.
        TemplateUsageFeeChanged(IdentityId, CodeHash, Balance, Balance),
        /// Emitted when the template instantiation fees gets changed.
        /// IdentityId of the owner, Code hash of the template, Old instantiation fee, New instantiation fee.
        TemplateInstantiationFeeChanged(IdentityId, CodeHash, Balance, Balance),
        /// Emitted when the template meta url get changed.
        /// IdentityId of the owner, Code hash of the template, old meta url, new meta url.
        TemplateMetaUrlChanged(IdentityId, CodeHash, Option<MetaUrl>, Option<MetaUrl>),
        /// Executing `put_code` has been enabled or disabled.
        /// (new flag state)
        PutCodeFlagChanged(bool),
    }
}
