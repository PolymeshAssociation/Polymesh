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

pub use frame_benchmarking::{account, benchmarks};
use frame_support::weights::Weight;
use frame_system::RawOrigin;
use pallet_asset::{
    benchmarking::make_base_asset, AggregateBalance, BalanceOf, BalanceOfAtScope, SecurityToken,
    Tokens,
};
use pallet_compliance_manager::benchmarking::make_issuers;
use pallet_contracts::ContractAddressFor;
use pallet_identity as identity;
use pallet_portfolio::{PortfolioAssetBalances, Portfolios};
use polymesh_common_utilities::{
    benchs::{User, UserBuilder},
    constants::currency::POLY,
    traits::asset::{AssetName, AssetType},
};
use polymesh_contracts::benchmarking::emulate_blueprint_in_storage;
use polymesh_primitives::{
    CddId, Claim, Condition, ConditionType, CountryCode, IdentityId, InvestorUid, PortfolioId,
    PortfolioName, Scope, SmartExtension, SmartExtensionType, TargetIdentity, Ticker,
    TrustedIssuer,
};
use sp_runtime::traits::Hash;
use sp_runtime::SaturatedConversion;
use sp_std::prelude::*;

#[cfg(not(feature = "std"))]
use hex_literal::hex;

#[cfg(feature = "std")]
use sp_core::sr25519::Signature;
#[cfg(feature = "std")]
use sp_runtime::MultiSignature;

const MAX_VENUE_DETAILS_LENGTH: u32 = 100000;
const MAX_SIGNERS_ALLOWED: u32 = 50;
const MAX_VENUE_ALLOWED: u32 = 100;
const MAX_COMPLIANCE_RESTRICTION: u32 = 45;
const MAX_TRUSTED_ISSUER: u32 = 5;

type Portfolio<T> = pallet_portfolio::Module<T>;

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
    let pseudo_random_no = variable + salt;
    let portfolio_no = (pseudo_random_no as u64).into();
    let portfolio_name =
        PortfolioName::try_from(vec![b'P'; pseudo_random_no as usize].as_slice()).unwrap();
    match did {
        None => {
            let user = UserBuilder::<T>::default()
                .generate_did()
                .seed(pseudo_random_no)
                .build(portfolio_to);
            Portfolios::insert(user.did(), portfolio_no, portfolio_name);
            PortfolioId::user_portfolio(user.did(), portfolio_no)
        }
        Some(id) => {
            Portfolios::insert(id, portfolio_no, portfolio_name);
            PortfolioId::user_portfolio(id, portfolio_no)
        }
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
    let User {
        origin,
        did,
        account,
        ..
    } = UserBuilder::<T>::default().generate_did().build("creator");
    let venue_id = create_venue_::<T>(did.unwrap(), vec![]);

    // Create legs vector.
    // Assuming the worst case where there is no dedup of `from` and `to` in the legs vector.
    if create_portfolios {
        // Assumption here is that instruction will never be executed as still there is one auth pending.
        for n in 0..l {
            setup_leg_and_portfolio::<T>(None, Some(did.unwrap()), n, &mut legs, &mut portfolios);
        }
    } else {
        for i in 1..l {
            populate_legs_for_instruction::<T>(i, &mut legs);
        }
    }
    Ok((legs, venue_id, origin, did.unwrap(), portfolios, account))
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
            let transacted_amount = 500 * POLY;
            let sender_portfolio = generate_portfolio::<T>("", index, 500, sender);
            let receiver_portfolio = generate_portfolio::<T>("", index, 500, receiver);
            let _ = fund_portfolio::<T>(&sender_portfolio, ticker, transacted_amount.into());
            portfolios.push(sender_portfolio);
            default_portfolio.push(receiver_portfolio);
            legs.push(Leg {
                from: sender_portfolio,
                to: receiver_portfolio,
                asset: *ticker,
                amount: transacted_amount.into(),
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

// Add investor uniqueness claim directly in the storage.
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
    let current_balance = <pallet_asset::Module<T>>::balance_of(ticker, did);
    <AggregateBalance<T>>::insert(ticker, &did, current_balance);
    <BalanceOfAtScope<T>>::insert(did, did, current_balance);
}

fn add_trusted_issuer<T: Trait>(
    origin: RawOrigin<T::AccountId>,
    ticker: Ticker,
    issuer: TrustedIssuer,
) {
    pallet_compliance_manager::Module::<T>::add_default_trusted_claim_issuer(
        origin.into(),
        ticker,
        issuer,
    )
    .expect("Default trusted claim issuer cannot be added");
}

pub fn create_condition<T: Trait>(
    c_count: u32,
    t_count: u32,
    ticker: Ticker,
    token_owner: RawOrigin<T::AccountId>,
) -> Vec<Condition> {
    let condition_type = get_condition_type::<T>(&c_count, Scope::Ticker(ticker));
    let trusted_issuers = make_issuers::<T>(t_count);
    trusted_issuers.clone().into_iter().for_each(|issuer| {
        let t_issuer = <pallet_compliance_manager::Module<T>>::trusted_claim_issuer(ticker);
        if !t_issuer.contains(&issuer) {
            add_trusted_issuer::<T>(token_owner.clone().into(), ticker, issuer);
        }
    });
    vec![Condition::new(condition_type, trusted_issuers)]
}

pub fn get_condition_type<T: Trait>(condition_count: &u32, scope: Scope) -> ConditionType {
    let target_identity = UserBuilder::<T>::default()
        .generate_did()
        .build("TargetIdentity")
        .did();
    if (2..8).contains(condition_count) {
        ConditionType::IsPresent(Claim::Affiliate(scope))
    } else if (9..18).contains(condition_count) {
        ConditionType::IsAbsent(Claim::KnowYourCustomer(scope))
    } else if (19..27).contains(condition_count) {
        ConditionType::IsAnyOf(vec![
            Claim::Affiliate(scope.clone()),
            Claim::BuyLockup(scope.clone()),
            Claim::SellLockup(scope.clone()),
        ])
    } else if (28..36).contains(condition_count) {
        ConditionType::IsNoneOf(vec![
            Claim::Accredited(scope.clone()),
            Claim::Exempted(scope.clone()),
            Claim::Jurisdiction(CountryCode::AF, scope.clone()),
        ])
    } else {
        ConditionType::IsIdentity(TargetIdentity::Specific(target_identity))
    }
}

fn compliance_setup<T: Trait>(
    max_condition: u32,
    max_trusted_issuer: u32,
    ticker: Ticker,
    origin: RawOrigin<T::AccountId>,
    from_did: IdentityId,
    to_did: IdentityId,
    trusted_issuer: TrustedIssuer,
    t_issuer_origin: RawOrigin<T::AccountId>,
) {
    // Add investor uniqueness claim.
    add_investor_uniqueness_claim::<T>(from_did, ticker);
    add_investor_uniqueness_claim::<T>(to_did, ticker);
    // Add trusted issuer.
    add_trusted_issuer::<T>(origin.clone(), ticker, trusted_issuer.clone());

    let claim_to_pass = Claim::Accredited(Scope::Ticker(ticker));
    for i in 0..max_condition {
        let (s_cond, r_cond) = if i == (max_condition - 1) {
            let cond = vec![Condition::new(
                ConditionType::IsPresent(claim_to_pass.clone()),
                vec![trusted_issuer.clone()],
            )];
            (cond.clone(), cond)
        } else {
            let cond = create_condition::<T>(i, max_trusted_issuer, ticker, origin.clone());
            (cond.clone(), cond)
        };
        pallet_compliance_manager::Module::<T>::add_compliance_requirement(
            origin.clone().into(),
            ticker,
            s_cond,
            r_cond,
        )
        .expect("Failed to add the asset compliance");
    }
    // Provide Claim to the sender and receiver
    <identity::Module<T>>::add_claim(
        t_issuer_origin.clone().into(),
        from_did,
        claim_to_pass.clone(),
        None,
    )
    .expect("Settlement: Failed to add claim");
    <identity::Module<T>>::add_claim(t_issuer_origin.into(), to_did, claim_to_pass, None)
        .expect("Settlement: Failed to add claim");
}

fn setup_affirm_instruction<T: Trait>(
    l: u32,
) -> (Vec<PortfolioId>, User<T>, User<T>, Ticker, Ticker) {
    // create venue
    let from = UserBuilder::<T>::default().generate_did().build("creator");
    let venue_id = create_venue_::<T>(from.did(), vec![]);
    let settlement_type: SettlementType<T::BlockNumber> = SettlementType::SettleOnAffirmation;
    let to = UserBuilder::<T>::default().generate_did().build("receiver");
    let mut portfolios_from: Vec<PortfolioId> = Vec::with_capacity(l as usize);
    let mut portfolios_to: Vec<PortfolioId> = Vec::with_capacity(l as usize);
    let mut legs: Vec<Leg<T::Balance>> = Vec::with_capacity(l as usize);

    // Create legs vector.
    // Assuming the worst case where there is no dedup of `from` and `to` in the legs vector.
    // Assumption here is that instruction will never be executed as still there is one auth pending.
    let from_ticker = make_base_asset::<T>(
        &from,
        true,
        Some(Ticker::try_from(vec![b'A'; 8 as usize].as_slice()).unwrap()),
    );
    let to_ticker = make_base_asset::<T>(
        &to,
        true,
        Some(Ticker::try_from(vec![b'B'; 8 as usize].as_slice()).unwrap()),
    );
    for n in 1..l / 2 {
        setup_leg_and_portfolio_with_ticker::<T>(
            Some(to.did()),
            Some(from.did()),
            from_ticker,
            to_ticker,
            n,
            &mut legs,
            &mut portfolios_from,
            &mut portfolios_to,
        );
    }
    Module::<T>::add_and_affirm_instruction(
        (from.origin.clone()).into(),
        venue_id,
        settlement_type,
        None,
        legs,
        portfolios_from,
    )
    .expect("Unable to add and affirm the instruction");
    (portfolios_to, from, to, from_ticker, to_ticker)
}

fn add_smart_extension_to_ticker<T: Trait>(
    code_hash: <T::Hashing as Hash>::Output,
    origin: RawOrigin<T::AccountId>,
    account: T::AccountId,
    ticker: Ticker,
) {
    let data = vec![
        209, 131, 81, 43, 160, 134, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1,
    ]; // Allow 100% as percentage ownership and allow primary issuance.
    <polymesh_contracts::Module<T>>::instantiate(
        origin.clone().into(),
        0.into(),
        Weight::max_value(),
        code_hash,
        data.clone(),
        0.into(),
    )
    .expect("Settlement: Failed to instantiate the contract");
    let extension_id =
        T::DetermineContractAddress::contract_address_for(&code_hash, &data, &account);
    let extension_details = SmartExtension {
        extension_type: SmartExtensionType::TransferManager,
        extension_name: b"PTM".into(),
        extension_id: extension_id.clone(),
        is_archive: false,
    };
    <pallet_asset::Module<T>>::add_extension(origin.into(), ticker, extension_details)
        .expect("Settlement: Fail to add the smart extension to a given asset");
}

benchmarks! {
    _{}

    create_venue {
        // Variations for the venue_details length.
        let d in 1 .. MAX_VENUE_DETAILS_LENGTH;
        // Variations for the no. of signers allowed.
        let s in 0 .. MAX_SIGNERS_ALLOWED;
        let mut signers = Vec::with_capacity(s as usize);
        let User {origin, did, .. } = UserBuilder::<T>::default().generate_did().build("caller");
        let venue_details = VenueDetails::from(vec![b'D'; d as usize].as_slice());
        let venue_type = VenueType::Distribution;
        // Create signers vector.
        for signer in 0 .. s {
            signers.push(UserBuilder::<T>::default().generate_did().seed(signer).build("signers").account());
        }
    }: _(origin, venue_details, signers, venue_type)
    verify {
        ensure!(matches!(Module::<T>::venue_counter(), 2), "Invalid venue counter");
        ensure!(matches!(Module::<T>::user_venues(did.unwrap()).into_iter().last(), Some(1)), "Invalid venue id");
        ensure!(Module::<T>::venue_info(1).is_some(), "Incorrect venue info set");
    }


    update_venue {
        // Variations for the venue_details length.
        let d in 1 .. MAX_VENUE_DETAILS_LENGTH;
        let venue_details = VenueDetails::from(vec![b'D'; d as usize].as_slice());
        // Venue type.
        let venue_type = VenueType::Sto;
        let User {account, origin, did, ..} = UserBuilder::<T>::default().generate_did().build("creator");
        // create venue
        let venue_id = create_venue_::<T>(did.unwrap(), vec![]);
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
        let User {account, origin, did, ..} = UserBuilder::<T>::default().generate_did().build("creator");
        let ticker = create_asset_::<T>(did.unwrap())?;
    }: _(origin, ticker, true)
    verify {
        ensure!(Module::<T>::venue_filtering(ticker), "Fail: set_venue_filtering failed");
    }


    set_venue_filtering_disallow {
        // Constant time function. It is only for disallowing venue filtering.
        let User {account, origin, did, ..} = UserBuilder::<T>::default().generate_did().build("creator");
        let ticker = create_asset_::<T>(did.unwrap())?;
    }: set_venue_filtering(origin, ticker, false)
    verify {
        ensure!(!Module::<T>::venue_filtering(ticker), "Fail: set_venue_filtering failed");
    }


    allow_venues {
        // Count of venue is variant for this dispatchable.
        let v in 0 .. MAX_VENUE_ALLOWED;
        let User {account, origin, did, .. } = UserBuilder::<T>::default().generate_did().build("creator");
        let ticker = create_asset_::<T>(did.unwrap())?;
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
        let User {account, origin, did, .. } = UserBuilder::<T>::default().generate_did().build("creator");
        let ticker = create_asset_::<T>(did.unwrap())?;
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

        let creator = UserBuilder::<T>::default().generate_did().build("creator");
        let did_to = UserBuilder::<T>::default().generate_did().build("to_did").did();
        let venue_id = create_venue_::<T>(creator.did(), vec![creator.account().clone()]);

        let ticker = Ticker::try_from(vec![b'A'; 10 as usize].as_slice()).unwrap();
        let portfolio_from = PortfolioId::user_portfolio(creator.did(), (100u64).into());
        let _ = fund_portfolio::<T>(&portfolio_from, &ticker, 500.into());
        let portfolio_to = PortfolioId::user_portfolio(did_to, (500u64).into());
        let legs = vec![Leg {
            from: portfolio_from,
            to: portfolio_to,
            asset: ticker,
            amount: 100.into(),
        }];

        // Add instruction
        Module::<T>::base_add_instruction(creator.did(), venue_id, SettlementType::SettleOnAffirmation, None, legs.clone())?;
        let instruction_id = 1;
        let leg_id = 0;

        set_instruction_let_status_to_skipped::<T>(instruction_id, leg_id, creator.account().clone(), 0);
    }: _(creator.origin.clone(), instruction_id, leg_id)
    verify {
        ensure!(matches!(Module::<T>::instruction_leg_status(instruction_id, leg_id), LegStatus::ExecutionPending), "Fail: unclaim_receipt dispatch");
        ensure!(!Module::<T>::receipts_used(&creator.account(), 0), "Fail: Receipt status didn't get update");
    }


    reject_instruction {
        // At least one portfolio needed
        let l in 1 .. T::MaxLegsInInstruction::get() as u32;
        // Emulate the add instruction and get all the necessary arguments.
        let (legs, venue_id, origin, did , portfolios, account_id) = emulate_add_instruction::<T>(l, true)?;
        // Add and affirm instruction.
        Module::<T>::add_and_affirm_instruction((origin.clone()).into(), venue_id, SettlementType::SettleOnAffirmation, None, legs, portfolios.clone()).expect("Unable to add and affirm the instruction");
        let instruction_id: u64 = 1;
    }: _(origin, instruction_id, portfolios.clone())
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
        let (portfolios_to, _, to, _, _) = setup_affirm_instruction::<T>(l);
        let instruction_id = 1; // It will always be `1` as we know there is no other instruction in the storage yet.
    }: _(to.origin, instruction_id, portfolios_to.clone())
    verify {
        for p in portfolios_to.iter() {
            ensure!(Module::<T>::affirms_received(instruction_id, p) == AffirmationStatus::Affirmed, "Settlement: Failed to affirm instruction");
        }
    }


    // claim_receipt {
    //     // There is no catalyst in this dispatchable, It will always be time constant.

    //     // create venue
    //     let creator = UserBuilder::<T>::default().generate_did().build("creator");
    //     let to = UserBuilder::<T>::default().generate_did().build("to_did");
    //     let venue_id = create_venue_::<T>(creator.did(), vec![creator.account().clone()]);

    //     let ticker = Ticker::try_from(vec![b'A'; 10 as usize].as_slice()).unwrap();
    //     let portfolio_from = PortfolioId::user_portfolio(creator.did(), (100u64).into());
    //     let _ = fund_portfolio::<T>(&portfolio_from, &ticker, 500.into());
    //     let portfolio_to = PortfolioId::user_portfolio(to.did(), (500u64).into());
    //     let legs = vec![Leg {
    //         from: portfolio_from,
    //         to: portfolio_to,
    //         asset: ticker,
    //         amount: 100.into(),
    //     }];

    //     // Add instruction
    //     Module::<T>::base_add_instruction(creator.did(), venue_id, SettlementType::SettleOnAffirmation, None, legs.clone())?;
    //     let instruction_id = 1;

    //     let msg = Receipt {
    //         receipt_uid: 0,
    //         from: portfolio_from,
    //         to: portfolio_to,
    //         asset: ticker,
    //         amount: 100.into(),
    //     };

    //     // #[cfg(feature = "std")]
    //     // frame_support::debug::info!("Receipt data :{:?}", &msg);
    //     // #[cfg(feature = "std")]
    //     // frame_support::debug::info!("Creator did :{:?}", &creator.did());
    //     // #[cfg(feature = "std")]
    //     // frame_support::debug::info!("Creator Secret :{:?}", &creator.secret);

    //     let encoded = get_encoded_signature::<T>(&creator, &msg);
    //     let signature = T::OffChainSignature::decode(&mut &encoded[..])
    //         .expect("OffChainSignature cannot be decoded from a MultiSignature");
    //     let leg_id = 0;
    //     // Receipt details.
    //     let receipt = ReceiptDetails {
    //         receipt_uid: 0,
    //         leg_id,
    //         signer: creator.account(),
    //         signature,
    //         metadata: ReceiptMetadata::from(vec![b'D'; 10 as usize].as_slice())
    //     };

    //     set_instruction_leg_status_to_pending::<T>(instruction_id, leg_id);
    // }: _(creator.origin, instruction_id, receipt.clone())
    // verify {
    //     ensure!(Module::<T>::instruction_leg_status(instruction_id, leg_id) ==  LegStatus::ExecutionToBeSkipped(
    //         receipt.signer,
    //         receipt.receipt_uid,
    //     ), "Settlement: Fail to unclaim the receipt");
    // }

    execute_scheduled_instruction {
        // This dispatch execute an instruction.
        //
        // Worst case scenarios.
        // 1. Create maximum legs and both traded assets are different assets/securities.
        // 2. Assets should have worst compliance restrictions ?
        // 3. Assets have maximum no. of TMs.

        let l in 2 .. T::MaxLegsInInstruction::get() as u32; // At least 2 legs needed to achieve worst case.
        let s in 0 .. T::MaxNumberOfTMExtensionForAsset::get() as u32;
        let c in 1 .. MAX_COMPLIANCE_RESTRICTION; // At least 1 compliance restriction needed.
        let t in 1 .. MAX_TRUSTED_ISSUER; // At least 1 trusted issuer needed.
        // Setup affirm instruction (One party (i.e from) already affirms the instruction)
        let (portfolios_to, from, to, from_ticker, to_ticker) = setup_affirm_instruction::<T>(l);
        // It always be one as no other instruction is already scheduled.
        let instruction_id = 1;
        let origin = RawOrigin::Root;
        // Do another affirmations that lead to scheduling an instruction.
        Module::<T>::affirm_instruction((to.origin.clone()).into(), instruction_id, portfolios_to).expect("Settlement: Failed to affirm instruction");

        // Create trusted issuer for both the ticker
        let t_issuer = UserBuilder::<T>::default().generate_did().build("TrustedClaimIssuer");
        let trusted_issuer = TrustedIssuer::from(t_issuer.did());

        // Need to provide the Investor uniqueness claim for both sender and receiver,
        // for both assets also add the compliance rules as per the `MAX_COMPLIANCE_RESTRICTION`.
        compliance_setup::<T>(c, t, from_ticker, from.origin.clone(), from.did(), to.did(), trusted_issuer.clone(), t_issuer.origin.clone());
        compliance_setup::<T>(c, t, to_ticker, to.origin.clone(), from.did(), to.did(), trusted_issuer, t_issuer.origin);
        let code_hash = emulate_blueprint_in_storage::<T>(0, from.origin.clone(), "ptm")?;
        for i in 0 .. s {
            add_smart_extension_to_ticker::<T>(code_hash, from.origin.clone(), from.account().clone(), from_ticker);
            add_smart_extension_to_ticker::<T>(code_hash, to.origin.clone(), to.account().clone(), to_ticker);
        }
    }: _(origin, instruction_id)
}
