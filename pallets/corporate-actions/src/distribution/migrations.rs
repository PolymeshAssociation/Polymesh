use frame_support::storage::migration::move_prefix;
use sp_runtime::runtime_logger::RuntimeLogger;

use super::*;

mod v0 {
    use super::*;
    use polymesh_primitives::Ticker;

    #[derive(Copy, Clone, PartialEq, Eq, Debug, Encode, Decode, TypeInfo)]
    pub struct Distribution {
        pub from: PortfolioId,
        pub currency: Ticker,
        pub per_share: Balance,
        pub amount: Balance,
        pub remaining: Balance,
        pub reclaimed: bool,
        pub payment_at: Moment,
        pub expires_at: Option<Moment>,
    }

    decl_storage! {
        trait Store for Module<T: Config> as CapitalDistribution {
            // CAId and Distribution have changed types.
            pub(crate) OldDistributions get(fn distributions):
                map hasher(blake2_128_concat) crate::migrations::v0::CAId => Option<Distribution>;

            // The CAId type has changed.
            pub(crate) OldHolderPaid get(fn holder_paid):
                map hasher(blake2_128_concat) (crate::migrations::v0::CAId, IdentityId) => bool;
        }
    }

    decl_module! {
        pub struct Module<T: Config> for enum Call where origin: T::RuntimeOrigin { }
    }
}

impl From<v0::Distribution> for Distribution {
    fn from(v0_distribution: v0::Distribution) -> Self {
        Self {
            from: v0_distribution.from,
            currency: v0_distribution.currency.into(),
            per_share: v0_distribution.per_share,
            amount: v0_distribution.amount,
            remaining: v0_distribution.remaining,
            reclaimed: v0_distribution.reclaimed,
            payment_at: v0_distribution.payment_at,
            expires_at: v0_distribution.expires_at,
        }
    }
}

pub(crate) fn migrate_to_v1<T: Config>() {
    RuntimeLogger::init();

    let mut count = 0;
    log::info!("Updating types for the Distributions storage");
    move_prefix(
        &Distributions::final_prefix(),
        &v0::OldDistributions::final_prefix(),
    );
    v0::OldDistributions::drain().for_each(|(ca_id, distribution)| {
        count += 1;
        Distributions::insert(CAId::from(ca_id), Distribution::from(distribution));
    });
    log::info!("Migrated {:?} Distribution.Distributions entries.", count);

    let mut count = 0;
    log::info!("Updating types for the HolderPaid storage");
    move_prefix(
        &HolderPaid::final_prefix(),
        &v0::OldHolderPaid::final_prefix(),
    );
    v0::OldHolderPaid::drain().for_each(|((ca_id, did), paid)| {
        count += 1;
        HolderPaid::insert((CAId::from(ca_id), did), paid);
    });
    log::info!("Migrated {:?} Distribution.HolderPaid entries.", count);
}
