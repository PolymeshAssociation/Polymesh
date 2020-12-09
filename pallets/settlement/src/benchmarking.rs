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

//#![cfg(feature = "runtime-benchmarks")]
use crate::*;

pub use frame_benchmarking::{account, benchmarks, whitelisted_caller};
use frame_support::traits::Currency;
use frame_system::RawOrigin;
use pallet_asset::{benchmarking::make_base_asset, BalanceOf, SecurityToken, Tokens};
use pallet_balances as balances;
use pallet_identity::{
    self as identity,
    benchmarking::{uid_from_name_and_idx, User, UserBuilder},
};
use pallet_portfolio::PortfolioAssetBalances;
use polymesh_common_utilities::traits::asset::{AssetName, AssetType};
use polymesh_primitives::{CddId, Claim, InvestorUid, Scope};
use polymesh_primitives::{IdentityId, PortfolioId, Ticker};
use sp_runtime::SaturatedConversion;
use sp_std::prelude::*;

#[cfg(not(feature = "std"))]
use hex_literal::hex;

#[cfg(feature = "std")]
use sp_core::sr25519::Signature;
#[cfg(feature = "std")]
use sp_runtime::MultiSignature;

const SEED: u32 = 0;
const MAX_VENUE_DETAILS_LENGTH: u32 = 50000;
const MAX_SIGNERS_ALLOWED: u32 = 50;
const MAX_VENUE_ALLOWED: u32 = 100;
const MAX_TM_ALLOWED: u32 = 10;
const MAX_COMPLIANCE_RESTRICTION_COMPLEXITY_ALLOWED: u32 = 400;

type Portfolio<T> = pallet_portfolio::Module<T>;

pub struct Account<T: Trait> {
    account_id: T::AccountId,
    origin: RawOrigin<T::AccountId>,
    did: IdentityId,
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

/// Set venue related storage without any sanity checks.
fn create_venue_<T: Trait>(did: IdentityId, signers: Vec<T::AccountId>) -> u64 {
    // Worst case length for the venue details.
    let venue_details = VenueDetails::from(vec![b'A'; 200 as usize].as_slice());
    let venue = Venue::new(did, venue_details, VenueType::Distribution);
    // NB: Venue counter starts with 1.
    let venue_counter = Module::<T>::venue_counter();
    <VenueInfo>::insert(venue_counter, venue);
    for signer in signers {
        <VenueSigners<T>>::insert(venue_counter, signer, true);
    }
    <VenueCounter>::put(venue_counter + 1);
    Module::<T>::venue_counter() - 1
}

/// Set instruction leg status to `LegStatus::ExecutionToBeSkipped` without any sanity checks.
fn set_instruction_let_status_to_skipped<T: Trait>(
    instruction_id: u64,
    leg_id: u64,
    signer: T::AccountId,
    receipt_uid: u64,
) {
    <ReceiptsUsed<T>>::insert(&signer, receipt_uid, true);
    <InstructionLegStatus<T>>::insert(
        instruction_id,
        leg_id,
        LegStatus::ExecutionToBeSkipped(signer, receipt_uid),
    );
}

/// Set Leg status to `LegStatus::ExecutionPending`
fn set_instruction_leg_status_to_pending<T: Trait>(instruction_id: u64, leg_id: u64) {
    <InstructionLegStatus<T>>::insert(instruction_id, leg_id, LegStatus::ExecutionPending);
}

/// Set user affirmation without any sanity checks.
fn set_user_affirmations(instruction_id: u64, portfolio: PortfolioId, affirm: AffirmationStatus) {
    UserAffirmations::insert(portfolio, instruction_id, affirm);
}

// create asset
fn create_asset_<T: Trait>(owner_did: IdentityId) -> Result<Ticker, DispatchError> {
    let ticker = Ticker::try_from(vec![b'A'; 8 as usize].as_slice()).unwrap();
    let name = AssetName::from(vec![b'N'; 8 as usize].as_slice());
    let total_supply: T::Balance = 90000.into();
    let token = SecurityToken {
        name,
        total_supply,
        owner_did,
        divisible: true,
        asset_type: AssetType::EquityCommon,
        primary_issuance_agent: Some(owner_did),
    };
    <Tokens<T>>::insert(ticker, token);
    <BalanceOf<T>>::insert(ticker, owner_did, total_supply);
    Portfolio::<T>::set_default_portfolio_balance(owner_did, &ticker, total_supply);
    Ok(ticker)
}

// fund portfolio
fn fund_portfolio<T: Trait>(portfolio: &PortfolioId, ticker: &Ticker, amount: T::Balance) {
    <PortfolioAssetBalances<T>>::insert(portfolio, ticker, amount);
}

fn setup_leg_and_portfolio<T: Trait>(
    to_did: Option<IdentityId>,
    from_did: Option<IdentityId>,
    index: u32,
    legs: &mut Vec<Leg<T::Balance>>,
    sender_portfolios: &mut Vec<PortfolioId>,
) {
    let ticker = Ticker::try_from(vec![b'A'; index as usize].as_slice()).unwrap();
    let portfolio_from = generate_portfolio::<T>("", index, 100, from_did);
    let _ = fund_portfolio::<T>(&portfolio_from, &ticker, 500.into());
    let portfolio_to = generate_portfolio::<T>("to_did", index, 500, to_did);
    legs.push(Leg {
        from: portfolio_from,
        to: portfolio_to,
        asset: ticker,
        amount: 100.into(),
    });
    sender_portfolios.push(portfolio_from);
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

fn verify_add_instruction<T: Trait>(
    v_id: u64,
    s_type: SettlementType<T::BlockNumber>,
) -> DispatchResult {
    ensure!(
        Module::<T>::instruction_counter() == 2,
        "Instruction counter not increased"
    );
    let Instruction {
        instruction_id,
        venue_id,
        settlement_type,
        ..
    } = Module::<T>::instruction_details(Module::<T>::instruction_counter() - 1);
    assert_eq!(instruction_id, 1u64);
    assert_eq!(venue_id, v_id);
    assert_eq!(settlement_type, s_type);
    Ok(())
}

fn verify_add_and_affirm_instruction<T: Trait>(
    venue_id: u64,
    settlement_type: SettlementType<T::BlockNumber>,
    portfolios: Vec<PortfolioId>,
) -> DispatchResult {
    let _ = verify_add_instruction::<T>(venue_id, settlement_type)?;
    for portfolio_id in portfolios.iter() {
        ensure!(
            matches!(
                Module::<T>::affirms_received(1, portfolio_id),
                AffirmationStatus::Affirmed
            ),
            "Affirmation fails"
        );
    }
    Ok(())
}

fn emulate_add_instruction<T: Trait>(
    l: u32,
    create_portfolios: bool,
) -> Result<
    (
        Vec<Leg<T::Balance>>,
        u64,
        RawOrigin<T::AccountId>,
        IdentityId,
        Vec<PortfolioId>,
        T::AccountId,
    ),
    DispatchError,
> {
    let mut legs: Vec<Leg<T::Balance>> = Vec::with_capacity(l as usize);
    let mut portfolios: Vec<PortfolioId> = Vec::with_capacity(l as usize);
    // create venue
    let Account {
        account_id,
        origin,
        did,
    } = make_account::<T>("creator", SEED);
    let venue_id = create_venue_::<T>(did, vec![]);

    // Create legs vector.
    // Assuming the worst case where there is no dedup of `from` and `to` in the legs vector.
    if create_portfolios {
        // Assumption here is that instruction will never be executed as still there is one auth pending.
        for n in 0..l {
            setup_leg_and_portfolio::<T>(None, Some(did), n, &mut legs, &mut portfolios);
        }
    } else {
        for i in 1..l {
            populate_legs_for_instruction::<T>(i, &mut legs);
        }
    }
    Ok((legs, venue_id, origin, did, portfolios, account_id))
}

fn setup_leg_and_portfolio_with_ticker<T: Trait>(
    to_did: Option<IdentityId>,
    from_did: Option<IdentityId>,
    from_ticker: Ticker,
    to_ticker: Ticker,
    index: u32,
    legs: &mut Vec<Leg<T::Balance>>,
    sender_portfolios: &mut Vec<PortfolioId>,
    receiver_portfolios: &mut Vec<PortfolioId>,
) {
    let mut emulate_portfolios =
        |sender: Option<IdentityId>,
         receiver: Option<IdentityId>,
         portfolios: &mut Vec<PortfolioId>,
         ticker: &Ticker,
         default_portfolio: &mut Vec<PortfolioId>| {
            let sender_portfolio = generate_portfolio::<T>("", index, 500, sender);
            let receiver_portfolio = generate_portfolio::<T>("", index, 500, receiver);
            let _ = fund_portfolio::<T>(&sender_portfolio, ticker, 500.into());
            portfolios.push(sender_portfolio);
            default_portfolio.push(receiver_portfolio);
            legs.push(Leg {
                from: sender_portfolio,
                to: receiver_portfolio,
                asset: *ticker,
                amount: 500.into(),
            })
        };
    emulate_portfolios(
        from_did,
        to_did,
        sender_portfolios,
        &from_ticker,
        receiver_portfolios,
    );
    emulate_portfolios(
        to_did,
        from_did,
        receiver_portfolios,
        &to_ticker,
        sender_portfolios,
    );
}

// Generate signature.
fn get_encoded_signature<T: Trait>(signer: &User<T>, msg: &Receipt<T::Balance>) -> Vec<u8> {
    #[cfg(feature = "std")]
    let encoded = {
        // Signer signs the relay call.
        // NB: Decode as T::OffChainSignature because there is not type constraints in
        // `T::OffChainSignature` to limit it.
        let raw_signature: [u8; 64] = signer.sign(&msg.encode()).0;
        let encoded = MultiSignature::from(Signature::from_raw(raw_signature)).encode();

        // Native execution can generate a hard-coded signature using the following code:
        // ```ignore
        let hex_encoded = hex::encode(&encoded);
        frame_support::debug::info!("encoded signature :{:?}", &hex_encoded);
        //  ```

        encoded
    };
    #[cfg(not(feature = "std"))]
    let encoded = hex!("015e902873fc7de21faaccd58569ba0d6bde06d81c425abf136448625910713735d1905121a2368fcc254439ab50302c8c2169a55c2816182e10ea0a937e79548d").to_vec();

    encoded
}

fn add_investor_uniqueness_claim<T: Trait>(did: IdentityId, ticker: Ticker) {
    identity::Module::<T>::base_add_claim(
        did,
        Claim::InvestorUniqueness(
            Scope::Ticker(ticker),
            did,
            CddId::new(did, InvestorUid::from(did.to_bytes())),
        ),
        did,
        None,
    );
}

fn compliance_setup<T: Trait>(
    ticker: Ticker,
    origin: RawOrigin<T::AccountId>,
    from_did: IdentityId,
    to_did: IdentityId,
) {
    // Add investor uniqueness claim.
    add_investor_uniqueness_claim::<T>(from_did, ticker);
    add_investor_uniqueness_claim::<T>(to_did, ticker);

    //pallet_compliance_manager::Module::<T>::add_compliance_requirement(origin.into(), ticker).expect("Failed to pause the asset compliance");
}

benchmarks! {
    _{}

    create_venue {
        // Variations for the venue_details length.
        let d in 1 .. MAX_VENUE_DETAILS_LENGTH;
        // Variations for the no. of signers allowed.
        let s in 0 .. MAX_SIGNERS_ALLOWED;
        let mut signers = Vec::with_capacity(s as usize);
        let Account {origin, did, .. } = make_account::<T>("caller", SEED);
        let venue_details = VenueDetails::from(vec![b'D'; d as usize].as_slice());
        let venue_type = VenueType::Distribution;
        // Create signers vector.
        for signer in 0 .. s {
            signers.push(make_account::<T>("signers", signer).account_id);
        }
    }: _(origin, venue_details, signers, venue_type)
    verify {
        ensure!(matches!(Module::<T>::venue_counter(), 2), "Invalid venue counter");
        ensure!(matches!(Module::<T>::user_venues(did).into_iter().last(), Some(1)), "Invalid venue id");
        ensure!(Module::<T>::venue_info(1).is_some(), "Incorrect venue info set");
    }


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
    verify {
        let updated_venue_details = Module::<T>::venue_info(1).unwrap();
        ensure!(matches!(updated_venue_details.venue_type, VenueType::Sto), "Incorrect venue type value");
        ensure!(matches!(updated_venue_details.details, venue_details), "Incorrect venue details");
    }


    add_instruction {

        let l in 1 .. T::MaxLegsInInstruction::get() as u32; // Variation for the MAX leg count.
        // Define settlement type
        let settlement_type = SettlementType::SettleOnAffirmation;
        // Emulate the add instruction and get all the necessary arguments.
        let (legs, venue_id, origin, did , _, _ ) = emulate_add_instruction::<T>(l, false)?;

    }: _(origin, venue_id, settlement_type, Some(99999999.into()), legs)
    verify {
        verify_add_instruction::<T>(venue_id, settlement_type)?;
    }


    add_instruction_with_settle_on_block_type {
        let l in 1 .. T::MaxLegsInInstruction::get() as u32; // Variation for the MAX leg count.
        // Define settlement type
        let settlement_type = SettlementType::SettleOnBlock(100.into());
        set_block_number::<T>(50);

        // Emulate the add instruction and get all the necessary arguments.
        let (legs, venue_id, origin, did , _,_ ) = emulate_add_instruction::<T>(l, false)?;

    }: add_instruction(origin, venue_id, settlement_type, Some(99999999.into()), legs)
    verify {
        verify_add_instruction::<T>(venue_id, settlement_type)?;
    }


    add_and_affirm_instruction {
        let l in 1 .. T::MaxLegsInInstruction::get() as u32;
        // Define settlement type
        let settlement_type = SettlementType::SettleOnAffirmation;
        // Emulate the add instruction and get all the necessary arguments.
        let (legs, venue_id, origin, did , portfolios, _) = emulate_add_instruction::<T>(l, true)?;

    }: _(origin, venue_id, settlement_type, Some(99999999.into()), legs, portfolios.clone())
    verify {
        verify_add_and_affirm_instruction::<T>(venue_id, settlement_type, portfolios)?;
    }


    add_and_affirm_instruction_with_settle_on_block_type {
        let l in 1 .. T::MaxLegsInInstruction::get() as u32;
        // Define settlement type.
        let settlement_type = SettlementType::SettleOnBlock(100.into());
        set_block_number::<T>(50);
        // Emulate the add instruction and get all the necessary arguments.
        let (legs, venue_id, origin, did , portfolios, _) = emulate_add_instruction::<T>(l, true)?;

    }: add_and_affirm_instruction(origin, venue_id, settlement_type, Some(99999999.into()), legs, portfolios.clone())
    verify {
        verify_add_and_affirm_instruction::<T>(venue_id, settlement_type, portfolios)?;
    }


    set_venue_filtering {
        // Constant time function. It is only for allow venue filtering.
        let Account {account_id, origin, did} = make_account::<T>("creator", SEED);
        let ticker = create_asset_::<T>(did)?;
    }: _(origin, ticker, true)
    verify {
        ensure!(Module::<T>::venue_filtering(ticker), "Fail: set_venue_filtering failed");
    }


    set_venue_filtering_disallow {
        // Constant time function. It is only for disallowing venue filtering.
        let Account {account_id, origin, did} = make_account::<T>("creator", SEED);
        let ticker = create_asset_::<T>(did)?;
    }: set_venue_filtering(origin, ticker, false)
    verify {
        ensure!(!Module::<T>::venue_filtering(ticker), "Fail: set_venue_filtering failed");
    }


    allow_venues {
        // Count of venue is variant for this dispatchable.
        let v in 0 .. MAX_VENUE_ALLOWED;
        let Account {account_id, origin, did} = make_account::<T>("creator", SEED);
        let ticker = create_asset_::<T>(did)?;
        let mut venues: Vec<u64> = Vec::new();
        for i in 0 .. v {
            venues.push(i.into());
        }
    }: _(origin, ticker, venues.clone())
    verify {
        for v in venues.iter() {
            ensure!(Module::<T>::venue_allow_list(ticker, v), "Fail: allow_venue dispatch");
        }
    }


    disallow_venues {
        // Count of venue is variant for this dispatchable.
        let v in 0 .. MAX_VENUE_ALLOWED;
        let Account {account_id, origin, did} = make_account::<T>("creator", SEED);
        let ticker = create_asset_::<T>(did)?;
        let mut venues: Vec<u64> = Vec::new();
        for i in 0 .. v {
            venues.push(i.into());
        }
    }: _(origin, ticker, venues.clone())
    verify {
        for v in venues.iter() {
            ensure!(!Module::<T>::venue_allow_list(ticker, v), "Fail: allow_venue dispatch");
        }
    }


    withdraw_affirmation {
        // Below setup is for the onchain affirmation.

        let l in 0 .. T::MaxLegsInInstruction::get() as u32;
        // Emulate the add instruction and get all the necessary arguments.
        let (legs, venue_id, origin, did , portfolios, _) = emulate_add_instruction::<T>(l, true)?;
        // Add instruction
        Module::<T>::base_add_instruction(did, venue_id, SettlementType::SettleOnAffirmation, None, legs.clone())?;
        let instruction_id: u64 = 1;
        // Affirm an instruction
        let portfolios_set = portfolios.clone().into_iter().collect::<BTreeSet<_>>();
        Module::<T>::unsafe_affirm_instruction(did, instruction_id, portfolios_set, None)?;

    }: _(origin, instruction_id, portfolios)
    verify {
        for (idx, leg) in legs.iter().enumerate() {
            ensure!(matches!(Module::<T>::instruction_leg_status(instruction_id, u64::try_from(idx).unwrap_or_default()), LegStatus::PendingTokenLock), "Fail: withdraw affirmation dispatch");
        }
    }


    withdraw_affirmation_with_receipt {
        // Below setup is for the receipt based affirmation

        let l in 0 .. T::MaxLegsInInstruction::get() as u32;
        // Emulate the add instruction and get all the necessary arguments.
        let (legs, venue_id, origin, did , portfolios, account_id) = emulate_add_instruction::<T>(l, true)?;
        // Add instruction
        Module::<T>::base_add_instruction(did, venue_id, SettlementType::SettleOnAffirmation, None, legs.clone())?;
        let instruction_id: u64 = 1;
        // Affirm an instruction
        portfolios.clone().into_iter().for_each(|p| {
            set_user_affirmations(instruction_id, p, AffirmationStatus::Affirmed);
        });
        for (idx, _) in legs.clone().iter().enumerate() {
            let leg_id = u64::try_from(idx).unwrap_or_default();
            // use leg_id for the receipt_uid as well.
            set_instruction_let_status_to_skipped::<T>(instruction_id, leg_id, account_id.clone(), leg_id);
        }
    }: withdraw_affirmation(origin, instruction_id, portfolios)
    verify {
        for (idx, leg) in legs.iter().enumerate() {
            ensure!(matches!(Module::<T>::instruction_leg_status(instruction_id, u64::try_from(idx).unwrap_or_default()), LegStatus::PendingTokenLock), "Fail: withdraw affirmation dispatch");
        }
    }


    unclaim_receipt {
        // There is no catalyst in this dispatchable, It will be time constant always.

        let Account {account_id, origin, did} = make_account::<T>("creator", SEED);
        let did_to = make_account::<T>("to_did", 5).did;
        let venue_id = create_venue_::<T>(did, vec![account_id.clone()]);

        let ticker = Ticker::try_from(vec![b'A'; 10 as usize].as_slice()).unwrap();
        let portfolio_from = PortfolioId::user_portfolio(did, (100u64).into());
        let _ = fund_portfolio::<T>(&portfolio_from, &ticker, 500.into());
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

        set_instruction_let_status_to_skipped::<T>(instruction_id, leg_id, account_id.clone(), 0);
    }: _(origin, instruction_id, leg_id)
    verify {
        ensure!(matches!(Module::<T>::instruction_leg_status(instruction_id, leg_id), LegStatus::ExecutionPending), "Fail: unclaim_receipt dispatch");
        ensure!(!Module::<T>::receipts_used(&account_id, 0), "Fail: Receipt status didn't get update");
    }


    reject_instruction_with_all_pre_affirmations {
        // At least one portfolio needed
        let l in 1 .. T::MaxLegsInInstruction::get() as u32;
        // Emulate the add instruction and get all the necessary arguments.
        let (legs, venue_id, origin, did , portfolios, account_id) = emulate_add_instruction::<T>(l, true)?;
        // Add and affirm instruction.
        Module::<T>::add_and_affirm_instruction((origin.clone()).into(), venue_id, SettlementType::SettleOnAffirmation, None, legs, portfolios.clone()).expect("Unable to add and affirm the instruction");
        let instruction_id: u64 = 1;
    }: reject_instruction(origin, instruction_id, portfolios.clone())
    verify {
        for p in portfolios.iter() {
            ensure!(Module::<T>::affirms_received(instruction_id, p) == AffirmationStatus::Rejected, "Settlement: Failed to reject instruction");
        }
    }


    reject_instruction_with_no_pre_affirmations {
        // At least one portfolio needed
        let l in 1 .. T::MaxLegsInInstruction::get() as u32;
        // Emulate the add instruction and get all the necessary arguments.
        let (legs, venue_id, origin, did , portfolios, account_id) = emulate_add_instruction::<T>(l, true)?;
        // Add instruction
        Module::<T>::base_add_instruction(did, venue_id, SettlementType::SettleOnAffirmation, None, legs.clone())?;
        let instruction_id: u64 = 1;
    }: reject_instruction(origin, instruction_id, portfolios.clone())
    verify {
        for p in portfolios.iter() {
            ensure!(Module::<T>::affirms_received(instruction_id, p) == AffirmationStatus::Rejected, "Settlement: Failed to reject instruction");
        }
    }


    affirm_instruction {

        let l in 2 .. T::MaxLegsInInstruction::get() as u32; // At least 2 legs needed to achieve worst case.

        // create venue
        let from = UserBuilder::<T>::default().build_with_did("creator", SEED);
        let venue_id = create_venue_::<T>(from.did(), vec![]);
        let settlement_type: SettlementType<T::BlockNumber> = SettlementType::SettleOnAffirmation;
        let to = UserBuilder::<T>::default().build_with_did("receiver", 1);
        let mut portfolios_from: Vec<PortfolioId> = Vec::with_capacity(l as usize);
        let mut portfolios_to: Vec<PortfolioId> = Vec::with_capacity(l as usize);
        let mut legs: Vec<Leg<T::Balance>> = Vec::with_capacity(l as usize);

        // Create legs vector.
        // Assuming the worst case where there is no dedup of `from` and `to` in the legs vector.
        // Assumption here is that instruction will never be executed as still there is one auth pending.
        let from_ticker = make_base_asset::<T>(&from, true, Some(Ticker::try_from(vec![b'A'; 8 as usize].as_slice()).unwrap()));
        let to_ticker = make_base_asset::<T>(&to, true, Some(Ticker::try_from(vec![b'B'; 8 as usize].as_slice()).unwrap()));
        for n in 1 .. l/2 {
            setup_leg_and_portfolio_with_ticker::<T>(Some(to.did()), Some(from.did()), from_ticker, to_ticker, n, &mut legs, &mut portfolios_from, &mut portfolios_to);
        }
        Module::<T>::add_and_affirm_instruction((from.origin).into(), venue_id, settlement_type, None, legs, portfolios_from).expect("Unable to add and affirm the instruction");
        let instruction_id = 1; // It will always be `1` as we know there is no other instruction in the storage yet.
    }: _(to.origin, instruction_id, portfolios_to.clone())
    verify {
        for p in portfolios_to.iter() {
            ensure!(Module::<T>::affirms_received(instruction_id, p) == AffirmationStatus::Affirmed, "Settlement: Failed to affirm instruction");
        }
    }


    claim_receipt {
        // There is no catalyst in this dispatchable, It will always be time constant.

        // create venue
        let creator = UserBuilder::<T>::default().build_with_did("creator", SEED);
        let to = UserBuilder::<T>::default().build_with_did("to_did", 5);
        let venue_id = create_venue_::<T>(creator.did(), vec![creator.account().clone()]);

        let ticker = Ticker::try_from(vec![b'A'; 10 as usize].as_slice()).unwrap();
        let portfolio_from = PortfolioId::user_portfolio(creator.did(), (100u64).into());
        let _ = fund_portfolio::<T>(&portfolio_from, &ticker, 500.into());
        let portfolio_to = PortfolioId::user_portfolio(to.did(), (500u64).into());
        let legs = vec![Leg {
            from: portfolio_from,
            to: portfolio_to,
            asset: ticker,
            amount: 100.into(),
        }];

        // Add instruction
        Module::<T>::base_add_instruction(creator.did(), venue_id, SettlementType::SettleOnAffirmation, None, legs.clone())?;
        let instruction_id = 1;

        let msg = Receipt {
            receipt_uid: 0,
            from: portfolio_from,
            to: portfolio_to,
            asset: ticker,
            amount: 100.into(),
        };

        // #[cfg(feature = "std")]
        // frame_support::debug::info!("Receipt data :{:?}", &msg);
        // #[cfg(feature = "std")]
        // frame_support::debug::info!("Creator did :{:?}", &creator.did());
        // #[cfg(feature = "std")]
        // frame_support::debug::info!("Creator Secret :{:?}", &creator.secret);

        let encoded = get_encoded_signature::<T>(&creator, &msg);
        let signature = T::OffChainSignature::decode(&mut &encoded[..])
            .expect("OffChainSignature cannot be decoded from a MultiSignature");
        let leg_id = 0;
        // Receipt details.
        let receipt = ReceiptDetails {
            receipt_uid: 0,
            leg_id,
            signer: creator.account(),
            signature,
            metadata: ReceiptMetadata::from(vec![b'D'; 10 as usize].as_slice())
        };

        set_instruction_leg_status_to_pending::<T>(instruction_id, leg_id);
    }: _(creator.origin, instruction_id, receipt.clone())
    verify {
        ensure!(Module::<T>::instruction_leg_status(instruction_id, leg_id) ==  LegStatus::ExecutionToBeSkipped(
            receipt.signer,
            receipt.receipt_uid,
        ), "Settlement: Fail to unclaim the receipt");
    }
}
