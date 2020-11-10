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
use polymesh_common_utilities::traits::asset::AssetName;
use polymesh_primitives::{IdentityId, InvestorUid, PortfolioId, Ticker};

use codec::Encode;
pub use frame_benchmarking::{account, benchmarks, whitelisted_caller};
use frame_support::traits::Currency;
use frame_system::RawOrigin;
use pallet_asset as asset;
use sp_runtime::SaturatedConversion;
// use sp_core::{sr25519::Pair as SrPair, Pair};
// use sp_io::hashing::blake2_256;
// use sp_runtime::MultiSignature;
use sp_std::prelude::*;

const SEED: u32 = 0;
const MAX_VENUE_DETAILS_LENGTH: u32 = 200;
const MAX_SIGNERS_ALLOWED: u32 = 50;
const MAX_VENUE_ALLOWED: u32 = 100;
const MAX_TM_ALLOWED: u32 = 10;
const MAX_COMPLIANCE_RESTRICTION_COMPLEXITY_ALLOWED: u32 = 50;

pub struct Account<T: Trait> {
    account_id: T::AccountId,
    origin: RawOrigin<T::AccountId>,
    did: IdentityId,
}

fn uid_from_name_and_idx(name: &'static str, u: u32) -> InvestorUid {
    InvestorUid::from((name, u).encode().as_slice())
}

fn make_account<T: Trait>(name: &'static str, u: u32) -> Account<T> {
    let account_id: T::AccountId = account(name, u, SEED);
    let origin = RawOrigin::Signed(account_id.clone());
    let _ = balances::Module::<T>::make_free_balance_be(&account_id, 1_000_000.into());
    let uid = uid_from_name_and_idx(name, u);
    let _ = identity::Module::<T>::register_did(origin.clone().into(), uid, vec![]);
    let did = identity::Module::<T>::get_identity(&account_id).unwrap_or_default();
    Account {
        account_id,
        origin,
        did,
    }
}

fn set_block_number<T: Trait>(new_block_no: u64) {
    system::Module::<T>::set_block_number(new_block_no.saturated_into::<T::BlockNumber>());
}

// Create venue.
fn create_venue_<T: Trait>(did: IdentityId, signers: Vec<T::AccountId>) -> u64 {
    // Worst case length for the venue details.
    let venue_details = VenueDetails::from(vec![b'A'; 200 as usize].as_slice());
    Module::<T>::add_venue(did, venue_details, signers);
    Module::<T>::venue_counter() - 1
}

// create asset
fn create_asset_<T: Trait>(did: IdentityId) -> Result<Ticker, DispatchError> {
    let ticker = Ticker::try_from(vec![b'A'; 8 as usize].as_slice()).unwrap();
    let name = AssetName::from(vec![b'N'; 8 as usize].as_slice());
    T::Asset::create_asset(did, ticker, name, 90000.into())?;
    Ok(ticker)
}

fn setup_leg_and_portfolio<T: Trait>(
    to_did: Option<IdentityId>,
    from_did: Option<IdentityId>,
    index: u32,
    legs: &mut Vec<Leg<T::Balance>>,
    sender_portfolios: &mut Vec<PortfolioId>,
    receiver_portfolios: &mut Vec<PortfolioId>,
) -> DispatchResult {
    let ticker = Ticker::try_from(vec![b'A'; index as usize].as_slice()).unwrap();
    let portfolio_from = generate_portfolio::<T>("", index, 100, from_did);
    let _ = T::Portfolio::fund_portfolio(&portfolio_from, &ticker, 500.into())?;
    let portfolio_to = generate_portfolio::<T>("to_did", index, 500, to_did);
    legs.push(Leg {
        from: portfolio_from,
        to: portfolio_to,
        asset: ticker,
        amount: 100.into(),
    });
    sender_portfolios.push(portfolio_from);
    receiver_portfolios.push(portfolio_to);
    Ok(())
}

fn setup_leg_and_portfolio_with_ticker<T: Trait>(
    to_did: Option<IdentityId>,
    from_did: Option<IdentityId>,
    from_ticker: Ticker,
    to_ticker: Ticker,
    index: u32,
    legs: &mut Vec<Leg<T::Balance>>,
    portfolios_from: &mut Vec<PortfolioId>,
    portfolios_to: &mut Vec<PortfolioId>,
) -> DispatchResult {
    let portfolio_from = generate_portfolio::<T>("from_did", index, 100, from_did);
    let _ = T::Portfolio::fund_portfolio(&portfolio_from, &from_ticker, 500.into())?;
    let portfolio_to = generate_portfolio::<T>("to_did", index, 500, to_did);
    let _ = T::Portfolio::fund_portfolio(&portfolio_to, &to_ticker, 500.into())?;
    legs.push(Leg {
        from: portfolio_from,
        to: portfolio_to,
        asset: from_ticker,
        amount: 100.into(),
    });
    legs.push(Leg {
        from: portfolio_to,
        to: portfolio_from,
        asset: to_ticker,
        amount: 100.into(),
    });
    portfolios_from.push(portfolio_from);
    portfolios_to.push(portfolio_to);
    Ok(())
}

fn generate_portfolio<T: Trait>(
    portfolio_to: &'static str,
    variable: u32,
    salt: u32,
    did: Option<IdentityId>,
) -> PortfolioId {
    let pusedo_random_no = variable + salt;
    let portfolio_no = (pusedo_random_no as u64).into();
    match did {
        None => PortfolioId::user_portfolio(
            make_account::<T>(portfolio_to, pusedo_random_no).did,
            portfolio_no,
        ),
        Some(id) => PortfolioId::user_portfolio(id, portfolio_no),
    }
}

fn populate_legs_for_instruction<T: Trait>(index: u32, legs: &mut Vec<Leg<T::Balance>>) {
    let ticker = Ticker::try_from(vec![b'A'; index as usize].as_slice()).unwrap();
    legs.push(Leg {
        from: generate_portfolio::<T>("from_did", index, 100, None),
        to: generate_portfolio::<T>("to_did", index, 500, None),
        asset: ticker,
        amount: 100.into(),
    });
}

benchmarks! {
    _{}

    create_venue {
        // Variations for the venue_details length.
        let d in 1 .. MAX_VENUE_DETAILS_LENGTH;
        // Variations for the no. of signers allowed.
        let s in 0 .. MAX_SIGNERS_ALLOWED;
        let mut signers = Vec::with_capacity(s as usize);
        let origin = make_account::<T>("caller", SEED).origin;
        let venue_details = VenueDetails::from(vec![b'D'; d as usize].as_slice());
        let venue_type = VenueType::Distribution;
        // Create signers vector.
        for signer in 0 .. s {
            signers.push(make_account::<T>("signers", signer).account_id);
        }
    }: _(origin, venue_details, signers, venue_type)


    update_venue {
        // Variations for the venue_details length.
        let d in 1 .. MAX_VENUE_DETAILS_LENGTH;
        let venue_details = VenueDetails::from(vec![b'D'; d as usize].as_slice());
        // Venue type.
        let venue_type = VenueType::Sto;
        let Account {account_id, origin, did} = make_account::<T>("creator", SEED);
        // create venue
        let venue_id = create_venue_::<T>(did, vec![]);
    }: _(origin, venue_id, Some(venue_details), Some(venue_type))


    add_instruction {

        let l in 1 .. T::MaxLegsInAInstruction::get() as u32; // Variation for the MAX leg count.
        let mut legs = Vec::with_capacity(l as usize);

        // create venue
        let Account {account_id, origin, did} = make_account::<T>("creator", SEED);
        let venue_id = create_venue_::<T>(did, vec![]);

        // Define settlement type
        let settlement_type = match l % 2 {
            0 => SettlementType::SettleOnAffirmation,
            _ => {
                set_block_number::<T>(50);
                SettlementType::SettleOnBlock(100.into())
            },
        };

        // Create legs vector.
        // Assuming the worst case where there is no dedup of `from` and `to` in the legs vector.
        for i in 1 .. l {
            populate_legs_for_instruction::<T>(i, &mut legs);
        }
    }: _(origin, venue_id, settlement_type, None, legs)
    verify {
        ensure!(Module::<T>::instruction_counter() == 2, "Instruction counter not increased");
        let Instruction {instruction_id, venue_id, .. } = Module::<T>::instruction_details(Module::<T>::instruction_counter() - 1);
        ensure!(instruction_id == 1, "Invalid instruction");
        ensure!(venue_id == venue_id, "Invalid venue");
    }


    add_and_affirm_instruction {
        let l in 1 .. T::MaxLegsInAInstruction::get() as u32;
        let mut legs: Vec<Leg<T::Balance>> = Vec::with_capacity(l as usize);

        // create venue
        let Account {account_id, origin, did} = make_account::<T>("creator", SEED);
        let venue_id = create_venue_::<T>(did, vec![]);

        // Define settlement type
        let settlement_type = match l % 2 {
            0 => SettlementType::SettleOnAffirmation,
            _ => {
                set_block_number::<T>(50);
                SettlementType::SettleOnBlock(100.into())
            },
        };
        let mut portfolios: Vec<PortfolioId> = Vec::with_capacity(l as usize);

        // Create legs vector.
        // Assuming the worst case where there is no dedup of `from` and `to` in the legs vector.
        // Assumption here is that instruction will never be executed as still there is one auth pending.
        for n in 1 .. l {
            setup_leg_and_portfolio::<T>(None, Some(did), n, &mut legs, &mut portfolios, &mut vec![])?;
        }
    }: _(origin, venue_id, settlement_type, None, legs, portfolios)


    set_venue_filtering {
        // Constant time function. It is only for allow venue filtering.
        let Account {account_id, origin, did} = make_account::<T>("creator", SEED);
        let ticker = create_asset_::<T>(did)?;
    }: _(origin, ticker, true)


    set_venue_filtering_disallow {
        // Constant time function. It is only for disallowing venue filtering.
        let Account {account_id, origin, did} = make_account::<T>("creator", SEED);
        let ticker = create_asset_::<T>(did)?;
    }: set_venue_filtering(origin, ticker, false)


    allow_venues {
        // Count of venue is variant for this dispatchable.
        let v in 0 .. MAX_VENUE_ALLOWED;
        let Account {account_id, origin, did} = make_account::<T>("creator", SEED);
        let ticker = create_asset_::<T>(did)?;
        let mut venues: Vec<u64> = Vec::new();
        for i in 0 .. v {
            venues.push(i.into());
        }
    }: _(origin, ticker, venues)


    disallow_venues {
        // Count of venue is variant for this dispatchable.
        let v in 0 .. MAX_VENUE_ALLOWED;
        let Account {account_id, origin, did} = make_account::<T>("creator", SEED);
        let ticker = create_asset_::<T>(did)?;
        let mut venues: Vec<u64> = Vec::new();
        for i in 0 .. v {
            venues.push(i.into());
        }
    }: _(origin, ticker, venues)


    // affirm_instruction {

    //     // Worst case for the affirm_instruction will be.
    //     // 1. An instruction can have the maximum no. of legs.
    //     //      a. Two tickers are used.
    //     //      b. All legs are between the same sender and receiver.
    //     //      c. All portfolios get affirmation at once.
    //     // 2. Tickers for every leg will be different.
    //     // 3. Maximum no. of Smart extensions are used.
    //     // 4. User's compliance get verified by the last asset compliance rules.

    //     let l in 2 .. T::MaxLegsInAInstruction::get() as u32; // At least 2 legs needed to achieve worst case.
    //     let t in 0 .. MAX_TM_ALLOWED;
    //     let c in 0 .. MAX_COMPLIANCE_RESTRICTION_COMPLEXITY_ALLOWED;

    //     // 1. Create instruction will maximum legs.

    //     // create venue
    //     let from = make_account::<T>("creator", SEED);
    //     let venue_id = create_venue_::<T>(from.did, vec![]);
    //     let settlement_type: SettlementType<T::BlockNumber> = SettlementType::SettleOnAffirmation;
    //     let to = make_account::<T>("receiver", 1);
    //     let portfolio_count = l/2;
    //     let mut portfolios_from: Vec<PortfolioId> = Vec::with_capacity(portfolio_count as usize);
    //     let mut portfolios_to: Vec<PortfolioId> = Vec::with_capacity(portfolio_count  as usize);
    //     let mut legs: Vec<Leg<T::Balance>> = Vec::with_capacity((portfolio_count * 2) as usize);

    //     // Create legs vector.
    //     // Assuming the worst case where there is no dedup of `from` and `to` in the legs vector.
    //     // Assumption here is that instruction will never be executed as still there is one auth pending.
    //     let from_ticker = create_asset_::<T>(from.did)?;
    //     let to_ticker = create_asset_::<T>(to.did)?;
    //     for n in 1 .. portfolio_count {
    //         setup_leg_and_portfolio_with_ticker::<T>(Some(to.did), Some(from.did), from_ticker, to_ticker, n, &mut legs, &mut portfolios_from, &mut portfolios_to)?;
    //     }
    //     // ensure!(legs.len() == ((portfolio_count * 2) as usize), "Incorrect length of legs");
    //     // ensure!(portfolios_from.len() == (portfolio_count as usize), "Incorrect length of from portfolio");
    //     // ensure!(portfolios_to.len() == (portfolio_count as usize), "Incorrect length of to portfolio");
    //     Module::<T>::add_and_affirm_instruction((from.origin).into(), venue_id, settlement_type, None, legs, portfolios_from)?;
    //     let instruction_id = 1; // It will always be `1` as we know there is no other instruction in the storage yet.
    //     ensure!(matches!(Module::<T>::instruction_affirms_pending(instruction_id), 0), "Instruction not set properly");
    //     ensure!(Module::<T>::user_affirmations(portfolios_to[0], instruction_id) == AffirmationStatus::Pending, "Some problem in the portfolios");
    // }: _(to.origin, instruction_id, portfolios_to)


    withdraw_affirmation {
        // Below setup is for the onchain affirmation

        let l in 0 .. T::MaxLegsInAInstruction::get() as u32;
        let mut legs: Vec<Leg<T::Balance>> = Vec::with_capacity(l as usize);
        let mut portfolios: Vec<PortfolioId> = Vec::with_capacity(l as usize);
        // create venue
        let Account {account_id, origin, did} = make_account::<T>("creator", SEED);
        let venue_id = create_venue_::<T>(did, vec![]);
        for n in 1 .. l {
            setup_leg_and_portfolio::<T>(None, Some(did), n, &mut legs, &mut portfolios, &mut vec![])?;
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
        let Account {account_id, origin, did} = make_account::<T>("creator", SEED);
        let venue_id = create_venue_::<T>(did, vec![account_id.clone()]);
        for n in 1 .. l {
            setup_leg_and_portfolio::<T>(None, Some(did), n, &mut legs, &mut portfolios, &mut vec![])?;
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
            Module::<T>::set_instruction_let_status_to_skipped(instruction_id, leg_id, account_id.clone(), leg_id);
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
        let Account {account_id, origin, did} = make_account::<T>("creator", SEED);
        let venue_id = create_venue_::<T>(did, vec![account_id.clone()]);
        for n in 1 .. l {
            setup_leg_and_portfolio::<T>(None, Some(did), n, &mut legs, &mut portfolios, &mut vec![])?;
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
            Module::<T>::set_instruction_let_status_to_skipped(instruction_id, leg_id, account_id.clone(), leg_id);
        }
    }: withdraw_affirmation(origin, instruction_id, portfolios)

    // TODO: Need to solve the signature type mismatch problem
    // claim_receipt {
    //     // There is no catalyst in this dispatchable, It will always be time constant.

    //     // create venue
    //     let Account {account_id, origin, did} = make_account::<T>("creator", SEED);
    //     let did_to = make_account::<T>("to_did", 5).did;
    //     let venue_id = create_venue_::<T>(did, vec![account_id.clone()]);

    //     let ticker = Ticker::try_from(vec![b'A'; 10 as usize].as_slice()).unwrap();
    //     let portfolio_from = PortfolioId::user_portfolio(did, 100u64);
    //     let _ = T::Portfolio::fund_portfolio(&portfolio_from, &ticker, 500.into())?;
    //     let portfolio_to = PortfolioId::user_portfolio(did_to, 500u64);
    //     let legs = vec![Leg {
    //         from: portfolio_from,
    //         to: portfolio_to,
    //         asset: ticker,
    //         amount: 100.into(),
    //     }];

    //     // Add instruction
    //     Module::<T>::base_add_instruction(did, venue_id, SettlementType::SettleOnAffirmation, None, legs.clone())?;
    //     let instruction_id = 1;

    //     let msg = Receipt {
    //         receipt_uid: 0,
    //         from: portfolio_from,
    //         to: portfolio_to,
    //         asset: ticker,
    //         amount: 100.into(),
    //     };

    //     let signature = T::OffChainSignature::from(MultiSignature::from(SrPair::from_entropy(&("creator", SEED, SEED).using_encoded(blake2_256), None).0.sign(&msg.encode())));

    //     // Receipt details.
    //     let receipt = ReceiptDetails {
    //         receipt_uid: 0,
    //         leg_id: 0,
    //         signer: account_id,
    //         signature
    //     };

    //     Module::<T>::set_instruction_leg_status_to_pending(instruction_id, 0, legs[0])?;
    // }: _(origin, instruction_id, receipt)


    unclaim_receipt {
        // There is no catalyst in this dispatchable, It will be time constant always.

        let Account {account_id, origin, did} = make_account::<T>("creator", SEED);
        let did_to = make_account::<T>("to_did", 5).did;
        let venue_id = create_venue_::<T>(did, vec![account_id.clone()]);

        let ticker = Ticker::try_from(vec![b'A'; 10 as usize].as_slice()).unwrap();
        let portfolio_from = PortfolioId::user_portfolio(did, (100u64).into());
        let _ = T::Portfolio::fund_portfolio(&portfolio_from, &ticker, 500.into())?;
        let portfolio_to = PortfolioId::user_portfolio(did_to, (500u64).into());
        let legs = vec![Leg {
            from: portfolio_from,
            to: portfolio_to,
            asset: ticker,
            amount: 100.into(),
        }];

        // Add instruction
        Module::<T>::base_add_instruction(did, venue_id, SettlementType::SettleOnAffirmation, None, legs.clone())?;
        let instruction_id = 1;
        let leg_id = 0;

        Module::<T>::set_instruction_let_status_to_skipped(instruction_id, leg_id, account_id, 0);
    }: _(origin, instruction_id, leg_id)
}
