// This file is part of the Polymesh distribution (https://github.com/PolymathNetwork/Polymesh).
// Copyright (c) 2020 Polymath

// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, version 3.

// This program is distributed in the hope that it will be useful, but
// WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the GNU
// General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with this program. If not, see <http://www.gnu.org/licenses/>.

#![cfg(feature = "runtime-benchmarks")]
use crate::*;

use pallet_balances as balances;
use pallet_identity as identity;
use polymesh_primitives::{IdentityId, InvestorUid, Ticker, PortfolioId};
use polymesh_common_utilities::traits::asset::AssetName;

pub use frame_benchmarking::{account, benchmarks, whitelisted_caller};
use frame_support::traits::Currency;
use frame_system::RawOrigin;
use sp_std::prelude::*;

const SEED: u32 = 0;
const MAX_VENUE_DETAILS_LENGTH: u32 = 200;
const MAX_SIGNERS_ALLOWED: u32 = 50;
const MAX_VENUE_ALLOWED: u32 = 100;

fn uid_from_name_and_idx(name: &'static str, u: u32) -> InvestorUid {
    InvestorUid::from((name, u).encode().as_slice())
}

fn make_account<T: Trait>(
    name: &'static str,
    u: u32,
) -> (T::AccountId, RawOrigin<T::AccountId>, IdentityId) {
    let account: T::AccountId = account(name, u, SEED);
    let origin = RawOrigin::Signed(account.clone());
    let _ = balances::Module::<T>::make_free_balance_be(&account, 1_000_000.into());
    let uid = uid_from_name_and_idx(name, u);
    let _ = identity::Module::<T>::register_did(origin.clone().into(), uid, vec![]);
    let did = identity::Module::<T>::get_identity(&account).unwrap_or_default();
    (account, origin, did)
}

// Create venue.
fn create_venue_<T: Trait>(did: IdentityId, signers: Vec<T::AccountId>) -> u64 {
    // Worst case length for the venue details.
    let venue_details = VenueDetails::from(vec![b'D'; 200 as usize].as_slice());
    Module::<T>::add_venue(did, venue_details, signers);
    Module::<T>::user_venues(did)
        .into_iter()
        .last()
        .unwrap_or_default()
}

// create asset
fn create_asset_<T: Trait>(did: IdentityId) -> Ticker {
    let ticker = Ticker::try_from(vec![b'A'; 16 as usize].as_slice()).unwrap();
    let name = AssetName::from(vec![b'N'; 16 as usize].as_slice());
    let _ = T::Asset::create_asset(did, &ticker, name, 1000.into());
    ticker
}

fn setup_leg_and_portfolio<T: Trait>(did: IdentityId, index: u32, legs: &mut Vec<Leg<T::Balance>>, portfolios: &mut Vec<PortfolioId>) -> DispatchResult {
    let ticker = Ticker::try_from(vec![b'A'; index as usize].as_slice()).unwrap();
    let portfolio_from = PortfolioId::user_portfolio(did, (index + 100).into());
    let _ = T::Portfolio::fund_portfolio(&portfolio_from, &ticker, 500.into())?;
    let portfolio_to = PortfolioId::user_portfolio(make_account::<T>("to_did", index + 500).2, (index + 500).into());
    legs.push(Leg {
        from: portfolio_from,
        to: portfolio_to,
        asset: ticker,
        amount: 100.into() 
    });
    portfolios.push(portfolio_from);
    Ok(())
}

benchmarks! {
    _{
        // User account seed
        let u in 0 .. 100 => ();
    }

    create_venue {
        // Variations for the venue_details length.
        let d in 1 .. MAX_VENUE_DETAILS_LENGTH;
        // Variations for the no. of signers allowed.
        let s in 0 .. MAX_SIGNERS_ALLOWED;
        let mut signers = Vec::with_capacity(s as usize);
        let origin = make_account::<T>("caller", SEED).1;
        let venue_details = VenueDetails::from(vec![b'D'; d as usize].as_slice());
        let venue_type = VenueType::Distribution;
        // Create signers vector.
        for signer in 0 .. s {
            signers.push(make_account::<T>("signers", signer).0);
        }
    }: _(origin, venue_details, signers, venue_type)


    update_venue {
        // Variations for the venue_details length.
        let d in 1 .. MAX_VENUE_DETAILS_LENGTH;
        let venue_details = VenueDetails::from(vec![b'D'; d as usize].as_slice());
        // Venue type.
        let venue_type = VenueType::Sto;
        let (_, origin, did) = make_account::<T>("creator", SEED);
        // create venue
        let venue_id = create_venue_::<T>(did, vec![]);
    }: _(origin, venue_id, Some(venue_details), Some(venue_type))


    add_instruction {

        // Define settlement type
        let d in 0 .. 1;
        let l in 1 .. T::MaxLegsInAInstruction::get() as u32;
        let mut legs = Vec::with_capacity(l as usize);

        // create venue
        let (_, origin, did,) = make_account::<T>("creator", SEED);
        let venue_id = create_venue_::<T>(did, vec![]);

        let settlement_type = if d == 1 {
            SettlementType::SettleOnBlock(100.into())
        } else {
            SettlementType::SettleOnAffirmation
        };
        
        // Create legs vector.
        // Assuming the worst case where there is no dedup of `from` and `to` in the legs vector.
        for n in 1 .. l {
            let ticker = Ticker::try_from(vec![b'A'; n as usize].as_slice()).unwrap();
            legs.push(Leg {
                from: PortfolioId::user_portfolio(make_account::<T>("from_did", n + 100).2 , (n + 100).into()),
                to: PortfolioId::user_portfolio(make_account::<T>("to_did", n + 100).2, (n + 500).into()),
                asset: ticker,
                amount: 100.into() 
            });
        }
    }: _(origin, venue_id, settlement_type, None, legs)


    add_and_affirm_instruction {
        // Define settlement type
        let d in 0 .. 1;
        let l in 1 .. T::MaxLegsInAInstruction::get() as u32;
        let mut legs: Vec<Leg<T::Balance>> = Vec::with_capacity(l as usize);

        // create venue
        let (_, origin, did) = make_account::<T>("creator", SEED);
        let venue_id = create_venue_::<T>(did, vec![]);

        let settlement_type = if d == 1 {
            system::Module::<T>::set_block_number(50.into());
            SettlementType::SettleOnBlock(100.into())
        } else {
            SettlementType::SettleOnAffirmation
        };
        let mut portfolios: Vec<PortfolioId> = Vec::with_capacity(l as usize);
        
        // Create legs vector.
        // Assuming the worst case where there is no dedup of `from` and `to` in the legs vector.
        // Assumption here is that instruction will never be executed as still there is one auth pending.
        for n in 1 .. l {
            setup_leg_and_portfolio::<T>(did, n, &mut legs, &mut portfolios)?;
        }
    }: _(origin, venue_id, settlement_type, None, legs, portfolios)


    set_venue_filtering {
        // Constant time function. It is only for allow venue filtering.
        let (_, origin, did) = make_account::<T>("creator", SEED);
        let ticker = create_asset_::<T>(did);
    }: _(origin, ticker, true)


    set_venue_filtering_disallow {
        // Constant time function. It is only for disallowing venue filtering.
        let (_, origin, did) = make_account::<T>("creator", SEED);
        let ticker = create_asset_::<T>(did);
    }: set_venue_filtering(origin, ticker, false)


    allow_venues {
        // Count of venue is variant for this dispatchable.
        let v in 0 .. MAX_VENUE_ALLOWED;
        let (_, origin, did) = make_account::<T>("creator", SEED);
        let ticker = create_asset_::<T>(did);
        let mut venues: Vec<u64> = Vec::new();
        for i in 0 .. v {
            venues.push(i.into());
        }
    }: _(origin, ticker, venues)


    disallow_venues {
        // Count of venue is variant for this dispatchable.
        let v in 0 .. MAX_VENUE_ALLOWED;
        let (_, origin, did) = make_account::<T>("creator", SEED);
        let ticker = create_asset_::<T>(did);
        let mut venues: Vec<u64> = Vec::new();
        for i in 0 .. v {
            venues.push(i.into());
        }
    }: _(origin, ticker, venues)


    affirm_instruction {

        // Worst case for the affirm_instruction will be.
        // 1. An instruction can have the maximum no. of legs. We are assuming
        // that the sender and receiver will be same in both case and last affirmation
        // will be done by providing all portfolios at once.
        // 2. Tickers for every leg will be different.
        // 3. Maximum no. of Smart extensions are used.
        // 4. User's compliance get verified by the last asset compliance rules.

        let l in 0 .. T::MaxLegsInAInstruction::get() as u32;
        let tm in 0 .. MAX_TM_ALLOWED;
        let cr in 0 .. MAX_COMPLIANCE_RESTRICTION_ALLOWED;

        // Create instruction will maximum legs.
        


    }: _()


    withdraw_affirmation {
        // Below setup is for the onchain affirmation

        let l in 0 .. T::MaxLegsInAInstruction::get() as u32;
        let mut legs: Vec<Leg<T::Balance>> = Vec::with_capacity(l as usize);
        let mut portfolios: Vec<PortfolioId> = Vec::with_capacity(l as usize);
        // create venue
        let (_, origin, did) = make_account::<T>("creator", SEED);
        let venue_id = create_venue_::<T>(did, vec![]);
        for n in 1 .. l {
            setup_leg_and_portfolio::<T>(did, n, &mut legs, &mut portfolios)?;
        }
        // Add instruction
        Module::<T>::base_add_instruction(did, venue_id, SettlementType::SettleOnAffirmation, None, legs)?;
        let instruction_id: u64 = 1;
        // Affirm an instruction
        let portfolios_set = portfolios.clone().into_iter().collect::<BTreeSet<_>>();
        Module::<T>::unsafe_affirm_instruction(did, instruction_id, portfolios_set)?;

    }: _(origin, instruction_id, portfolios)


    withdraw_affirmation_with_receipt {
        // Below setup is for the receipt based affirmation

        let l in 0 .. T::MaxLegsInAInstruction::get() as u32;
        let mut legs: Vec<Leg<T::Balance>> = Vec::with_capacity(l as usize);
        let mut portfolios: Vec<PortfolioId> = Vec::with_capacity(l as usize);
        // create venue
        let (signer, origin, did) = make_account::<T>("creator", SEED);
        let venue_id = create_venue_::<T>(did, vec![signer.clone()]);
        for n in 1 .. l {
            setup_leg_and_portfolio::<T>(did, n, &mut legs, &mut portfolios)?;
        }
        // Add instruction
        Module::<T>::base_add_instruction(did, venue_id, SettlementType::SettleOnAffirmation, None, legs.clone())?;
        let instruction_id: u64 = 1;
        // Affirm an instruction
        portfolios.clone().into_iter().for_each(|p| {
            Module::<T>::set_user_affirmations(instruction_id, p, AffirmationStatus::Affirmed);
        });
        for (idx, _) in legs.iter().enumerate() {
            let leg_id = u64::try_from(idx).unwrap_or_default();
            // use leg_id for the receipt_uid as well.
            Module::<T>::set_instruction_let_status_to_skipped(instruction_id, leg_id, signer.clone(), leg_id);
        }
    }: withdraw_affirmation(origin, instruction_id, portfolios)


    withdraw_affirmation_with_both_receipt_and_onchain_affirmation {
        // Below setup is for the receipt based & onchain affirmation

        let l in 1 .. T::MaxLegsInAInstruction::get() as u32;
        // TODO: Need to find a better way to make it randomize the value of p.
        let p: u32 = l / 2;
        let mut legs: Vec<Leg<T::Balance>> = Vec::with_capacity(l as usize);
        let mut portfolios: Vec<PortfolioId> = Vec::with_capacity(l as usize);
        // create venue
        let (signer, origin, did) = make_account::<T>("creator", SEED);
        let venue_id = create_venue_::<T>(did, vec![signer.clone()]);
        for n in 1 .. l {
            setup_leg_and_portfolio::<T>(did, n, &mut legs, &mut portfolios)?;
        }
        // Add instruction
        Module::<T>::base_add_instruction(did, venue_id, SettlementType::SettleOnAffirmation, None, legs.clone())?;
        let instruction_id: u64 = 1;
        let (p_onchain, p_receipt): (Vec<(usize, PortfolioId)>, Vec<(usize, PortfolioId)>) = portfolios.clone().into_iter().enumerate().partition(|(i, _)| *i <= p as usize);
        // Affirm an instruction on-chain
        let portfolios_set = p_onchain.into_iter().map(|(_, p)| p).collect::<BTreeSet<_>>();
        Module::<T>::unsafe_affirm_instruction(did, instruction_id, portfolios_set)?;

        // Mimic the affirmation using receipt.
        p_receipt.into_iter().for_each(|(_,p)| {
            Module::<T>::set_user_affirmations(instruction_id, p, AffirmationStatus::Affirmed);
        });
        for (idx, _) in legs.iter().enumerate() {
            let leg_id = u64::try_from(idx).unwrap_or_default();
            // use leg_id for the receipt_uid as well.
            Module::<T>::set_instruction_let_status_to_skipped(instruction_id, leg_id, signer.clone(), leg_id);
        }
    }: withdraw_affirmation(origin, instruction_id, portfolios)


    claim_receipt {
        // There is no catalyst in this dispatchable, It will be time constant always.

        // create venue
        let (signer, origin, did) = make_account::<T>("creator", SEED);
        let did_to = make_account::<T>("to_did", 5).2;
        let venue_id = create_venue_::<T>(did, vec![signer.clone()]);

        let ticker = Ticker::try_from(vec![b'A'; 10 as usize].as_slice()).unwrap();
        let portfolio_from = PortfolioId::user_portfolio(did, 100.into());
        let _ = T::Portfolio::fund_portfolio(&portfolio_from, &ticker, 500.into())?;
        let portfolio_to = PortfolioId::user_portfolio(did_to, 500.into());
        let amount: u64 = 100;
        let legs = vec![Leg {
            from: portfolio_from,
            to: portfolio_to,
            asset: ticker,
            amount,
        }];

        // Add instruction
        Module::<T>::base_add_instruction(did, venue_id, SettlementType::SettleOnAffirmation, None, legs.clone())?;

        let msg = Receipt {
            receipt_uid: 0,
            from: portfolio_from,
            to: portfolio_to,
            asset: ticker,
            amount,
        };

        // Receipt details.
        ReceiptDetails {
            receipt_uid: 0,
            leg_id: 0,
            signer,
            signature: OffChainSignature::from(
                signer.sign(&msg.encode())
            )
        }
    }: _()
}
