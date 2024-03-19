/// Add reward points to block authors: * 20 points to the block producer for producing a block
/// in the chain,
impl<T> pallet_authorship::EventHandler<T::AccountId, T::BlockNumber> for Pallet<T>
where
    T: Config + pallet_authorship::Config + pallet_session::Config
{
    fn note_author(author: T::AccountId) {
        Self::reward_by_ids(vec![(author, 20)])
    }
}

/// In this implementation `new_session(session)` must be called before `end_session(session-1)`
/// i.e. the new session must be planned before the ending of the previous session.
///
/// Once the first new_session is planned, all session must start and then end in order, though
/// some session can lag in between the newest session planned and the latest session started.
impl<T: Config> pallet_session::SessionManager<T::AccountId> for Pallet<T>
where <T as frame_system::Config>::BlockNumber: core::fmt::Display
{
    fn new_session(new_index: SessionIndex) -> Option<Vec<T::AccountId>> {
        log::trace!(
            target: LOG_TARGET,
            "[{}] planning new_session({})",
            <frame_system::Pallet<T>>::block_number(),
            new_index
        );
        Self::new_session(new_index)
    }
    fn start_session(start_index: SessionIndex) {
        log::trace!(
            target: LOG_TARGET,
            "[{}] starting start_session({})",
            <frame_system::Pallet<T>>::block_number(),
            start_index
        );
        Self::start_session(start_index)
    }
    fn end_session(end_index: SessionIndex) {
        log::trace!(
            target: LOG_TARGET,
            "[{}] ending end_session({})",
            <frame_system::Pallet<T>>::block_number(),
            end_index
        );
        Self::end_session(end_index)
    }
}

impl<T: Config>
    pallet_session::historical::SessionManager<T::AccountId, Exposure<T::AccountId, BalanceOf<T>>>
    for Pallet<T>
where
    <T as frame_system::Config>::BlockNumber: core::fmt::Display,
{
    fn new_session(
        new_index: SessionIndex,
    ) -> Option<Vec<(T::AccountId, Exposure<T::AccountId, BalanceOf<T>>)>> {
        <Self as pallet_session::SessionManager<_>>::new_session(new_index).map(|validators| {
            let current_era = Self::current_era()
                // Must be some as a new era has been created.
                .unwrap_or(0);

            validators
                .into_iter()
                .map(|v| {
                    let exposure = Self::eras_stakers(current_era, &v);
                    (v, exposure)
                })
                .collect()
        })
    }
    fn start_session(start_index: SessionIndex) {
        <Self as pallet_session::SessionManager<_>>::start_session(start_index)
    }
    fn end_session(end_index: SessionIndex) {
        <Self as pallet_session::SessionManager<_>>::end_session(end_index)
    }
}

/// This is intended to be used with `FilterHistoricalOffences`.
impl<T: Config>
    OnOffenceHandler<T::AccountId, pallet_session::historical::IdentificationTuple<T>, Weight>
    for Pallet<T>
where
    T: pallet_session::Config<ValidatorId = <T as frame_system::Config>::AccountId>,
    T: pallet_session::historical::Config<
        FullIdentification = Exposure<<T as frame_system::Config>::AccountId, BalanceOf<T>>,
        FullIdentificationOf = ExposureOf<T>,
    >,
    T::SessionHandler: pallet_session::SessionHandler<<T as frame_system::Config>::AccountId>,
    T::SessionManager: pallet_session::SessionManager<<T as frame_system::Config>::AccountId>,
    T::ValidatorIdOf: Convert<
        <T as frame_system::Config>::AccountId,
        Option<<T as frame_system::Config>::AccountId>,
    >,
{
    fn on_offence(
        offenders: &[OffenceDetails<
            T::AccountId,
            pallet_session::historical::IdentificationTuple<T>,
        >],
        slash_fraction: &[Perbill],
        slash_session: SessionIndex,
        _disable_strategy: DisableStrategy,
    ) -> Weight {
        // Polymesh-Note:
        // When slashing is off or allowed for none, set slash fraction to zero.
        // ---------------------------------------------------------------------
        let long_living_slash_fraction;
        let slash_fraction = if Self::slashing_allowed_for() == SlashingSwitch::None {
            long_living_slash_fraction = vec![Perbill::from_parts(0); slash_fraction.len()];
            long_living_slash_fraction.as_slice()
        } else {
            slash_fraction
        };
        // ---------------------------------------------------------------------

        let reward_proportion = SlashRewardFraction::<T>::get();
        let mut consumed_weight = Weight::zero();
        let mut add_db_reads_writes = |reads, writes| {
            consumed_weight += T::DbWeight::get().reads_writes(reads, writes);
        };

        let active_era = {
            let active_era = Self::active_era();
            add_db_reads_writes(1, 0);
            if active_era.is_none() {
                // this offence need not be re-submitted.
                return consumed_weight;
            }
            active_era
                .expect("value checked not to be `None`; qed")
                .index
        };
        let active_era_start_session_index = Self::eras_start_session_index(active_era)
            .unwrap_or_else(|| {
                frame_support::print("Error: start_session_index must be set for current_era");
                0
            });
        add_db_reads_writes(1, 0);

        let window_start = active_era.saturating_sub(T::BondingDuration::get());

        // fast path for active-era report - most likely.
        // `slash_session` cannot be in a future active era. It must be in `active_era` or before.
        let slash_era = if slash_session >= active_era_start_session_index {
            active_era
        } else {
            let eras = BondedEras::<T>::get();
            add_db_reads_writes(1, 0);

            // reverse because it's more likely to find reports from recent eras.
            match eras
                .iter()
                .rev()
                .filter(|&&(_, ref sesh)| sesh <= &slash_session)
                .next()
            {
                Some(&(ref slash_era, _)) => *slash_era,
                // before bonding period. defensive - should be filtered out.
                None => return consumed_weight,
            }
        };

        <Self as Store>::EarliestUnappliedSlash::mutate(|earliest| {
            if earliest.is_none() {
                *earliest = Some(active_era)
            }
        });
        add_db_reads_writes(1, 1);

        let slash_defer_duration = T::SlashDeferDuration::get();

        let invulnerables = Self::invulnerables();
        add_db_reads_writes(1, 0);

        for (details, slash_fraction) in offenders.iter().zip(slash_fraction) {
            let (stash, exposure) = &details.offender;

            // Skip if the validator is invulnerable.
            if invulnerables.contains(stash) {
                continue;
            }

            let unapplied = slashing::compute_slash::<T>(slashing::SlashParams {
                stash,
                slash: *slash_fraction,
                exposure,
                slash_era,
                window_start,
                now: active_era,
                reward_proportion,
            });

            if let Some(mut unapplied) = unapplied {
                let nominators_len = unapplied.others.len() as u64;
                let reporters_len = details.reporters.len() as u64;

                {
                    let upper_bound = 1 /* Validator/NominatorSlashInEra */ + 2 /* fetch_spans */;
                    let rw = upper_bound + nominators_len * upper_bound;
                    add_db_reads_writes(rw, rw);
                }
                unapplied.reporters = details.reporters.clone();
                if slash_defer_duration == 0 {
                    // apply right away.
                    slashing::apply_slash::<T>(unapplied, slash_era);
                    {
                        let slash_cost = (6, 5);
                        let reward_cost = (2, 2);
                        add_db_reads_writes(
                            (1 + nominators_len) * slash_cost.0 + reward_cost.0 * reporters_len,
                            (1 + nominators_len) * slash_cost.1 + reward_cost.1 * reporters_len,
                        );
                    }
                } else {
                    // defer to end of some `slash_defer_duration` from now.
                    <Self as Store>::UnappliedSlashes::mutate(active_era, move |for_later| {
                        for_later.push(unapplied)
                    });
                    add_db_reads_writes(1, 1);
                }
            } else {
                add_db_reads_writes(4 /* fetch_spans */, 5 /* kick_out_if_recent */)
            }
        }

        consumed_weight
    }
}