// This file is part of the Polymesh distribution (https://github.com/PolymeshAssociation/Polymesh).
// Copyright (c) 2020 Polymesh Association

// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, version 3.

// This program is distributed in the hope that it will be useful, but
// WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the GNU
// General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with this program. If not, see <http://www.gnu.org/licenses/>.

pub use frame_benchmarking::{account, benchmarks};
use frame_support::traits::{Get, TryCollect};
use frame_system::RawOrigin;
use scale_info::prelude::format;
use sp_runtime::MultiSignature;
use sp_std::prelude::*;

use pallet_asset::benchmarking::setup_asset_transfer;
use pallet_identity::benchmarking::{User, UserBuilder};
use pallet_nft::benchmarking::setup_nft_transfer;
use polymesh_primitives::bench::create_and_issue_sample_asset;
use polymesh_primitives::checked_inc::CheckedInc;
use polymesh_primitives::constants::currency::ONE_UNIT;
use polymesh_primitives::constants::ENSURED_MAX_LEN;
use polymesh_primitives::crypto::{ChainScopedMessage, SETTLEMENT_RECEIPT_LABEL};
use polymesh_primitives::portfolio::{Fund, FundDescription};
use polymesh_primitives::settlement::{AffirmationRequirement, ReceiptMetadata};
use polymesh_primitives::{AssetHolder, IdentityId, Memo, NFTId, NFTs, Ticker, WeightMeter};

use crate::*;

const MAX_VENUE_DETAILS_LENGTH: u32 = ENSURED_MAX_LEN;
const MAX_SIGNERS_ALLOWED: u32 = 50;
const MAX_VENUE_ALLOWED: u32 = 100;

#[derive(Encode, Decode, Clone, Copy)]
pub struct UserData<T: Config> {
    pub account: T::AccountId,
    pub did: IdentityId,
}

impl<T: Config> From<&User<T>> for UserData<T> {
    fn from(user: &User<T>) -> Self {
        Self {
            account: user.account(),
            did: user.did(),
        }
    }
}

pub struct Parameters<T: Config> {
    pub legs: Vec<Leg>,
    pub asset_holders: AssetHolders,
    pub asset_mediators: Vec<User<T>>,
}

impl<T: Config> Parameters<T> {
    pub fn sdr_holdings(&self) -> BoundedBTreeSet<AssetHolder, T::MaxNumberOfAssetHolders> {
        self.asset_holders
            .senders
            .clone()
            .into_iter()
            .try_collect()
            .expect("shouldn't be too many")
    }

    pub fn rcv_holdings(&self) -> BoundedBTreeSet<AssetHolder, T::MaxNumberOfAssetHolders> {
        self.asset_holders
            .receivers
            .clone()
            .into_iter()
            .try_collect()
            .expect("shouldn't be too many")
    }
}

#[derive(Default)]
pub struct AssetHolders {
    pub senders: Vec<AssetHolder>,
    pub receivers: Vec<AssetHolder>,
}

fn creator<T: Config>() -> User<T> {
    UserBuilder::<T>::default().generate_did().build("creator")
}

/// Set venue related storage without any sanity checks.
fn create_venue_<T: Config>(did: IdentityId, signers: Vec<T::AccountId>) -> VenueId {
    let venue = Venue {
        creator: did,
        venue_type: VenueType::Distribution,
    };
    // NB: Venue counter starts with 1.
    let venue_counter = VenueCounter::<T>::get();
    VenueInfo::<T>::insert(venue_counter, venue);
    for signer in signers {
        VenueSigners::<T>::insert(venue_counter, signer, true);
    }
    VenueCounter::<T>::put(venue_counter.checked_inc().unwrap());
    venue_counter
}

pub fn create_asset_<T: Config>(owner: &User<T>) -> AssetId {
    create_and_issue_sample_asset::<T>(owner.account(), true, None, b"MyAsset", true)
}

fn setup_legs<T>(
    sender: &User<T>,
    receiver: &User<T>,
    f: u32,
    n: u32,
    o: u32,
    pause_compliance: bool,
    pause_restrictions: bool,
) -> Parameters<T>
where
    T: Config,
{
    // Opt-in receiver to mandatory affirmation so benchmarks measure worst-case weights.
    Pallet::<T>::set_mandatory_receiver_affirmation(
        receiver.origin().into(),
        AffirmationRequirement::Required,
    )
    .unwrap();

    let mut asset_holders = AssetHolders::default();
    let mut asset_mediators = Vec::new();

    // Creates offchain legs and new portfolios for each leg
    let offchain_legs: Vec<Leg> = (0..o)
        .map(|i| {
            let ticker = Ticker::from_slice_truncated(format!("OFFTICKER{}", i).as_bytes());
            Leg::OffChain {
                sender_identity: sender.did(),
                receiver_identity: receiver.did(),
                ticker,
                amount: ONE_UNIT,
            }
        })
        .collect();

    // Creates f assets, creates two portfolios, adds maximum compliance requirements, adds maximum transfer conditions and pauses them
    let fungible_legs: Vec<Leg> = (0..f)
        .map(|i| {
            let sdr_portfolio_name = format!("SdrPortfolioTicker{}", i);
            let rcv_portfolio_name = format!("RcvPortfolioTicker{}", i);
            let (sdr_holding, rvc_holding, mut mediators, asset_id) = setup_asset_transfer(
                sender,
                receiver,
                Some(&sdr_portfolio_name),
                Some(&rcv_portfolio_name),
                pause_compliance,
                pause_restrictions,
                4,
                true,
                false,
            );
            asset_mediators.append(&mut mediators);
            asset_holders.senders.push(sdr_holding.clone());
            asset_holders.receivers.push(rvc_holding.clone());
            Leg::Fungible {
                sender: sdr_holding,
                receiver: rvc_holding,
                asset_id,
                amount: ONE_UNIT,
            }
        })
        .collect();

    // Creates n collections, mints one NFT, creates two portfolios, adds maximum compliance requirements and pauses it
    let nft_legs: Vec<Leg> = (0..n)
        .map(|i| {
            let sdr_portfolio_name = format!("SdrPortfolioNFTTicker{}", i);
            let rcv_portfolio_name = format!("RcvPortfolioNFTTicker{}", i);
            let (asset_id, sdr_holding, rcv_holding, mut mediators) = setup_nft_transfer(
                sender,
                receiver,
                1,
                Some(&sdr_portfolio_name),
                Some(&rcv_portfolio_name),
                pause_compliance,
                4,
            );
            asset_mediators.append(&mut mediators);
            asset_holders.senders.push(sdr_holding.clone());
            asset_holders.receivers.push(rcv_holding.clone());
            Leg::NonFungible {
                sender: sdr_holding,
                receiver: rcv_holding,
                nfts: NFTs::new_unverified(asset_id, vec![NFTId(1)]),
            }
        })
        .collect();

    Parameters {
        legs: [offchain_legs, fungible_legs, nft_legs].concat(),
        asset_holders,
        asset_mediators,
    }
}

/// Creates and affirms an instruction with `f` fungible legs, `n` non-fungible legs and `o` offchain legs.
/// All legs have unique tickers, use custom portfolios, and have the maximum number of compliance requirements,
/// active statistics and transfer restrictions,
fn setup_execute_instruction<T>(
    sender: &User<T>,
    receiver: &User<T>,
    settlement_type: SettlementType<BlockNumberFor<T>>,
    venue_id: VenueId,
    f: u32,
    n: u32,
    o: u32,
    m: u32,
    pause_compliance: bool,
    pause_restrictions: bool,
) -> Parameters<T>
where
    T: Config,
{
    let (m_user, m_identity) = set_instruction_mediators::<T>(m);
    // Creates the instruction. All assets, collections, portfolios and rules are created here.
    let parameters = setup_legs::<T>(
        sender,
        receiver,
        f,
        n,
        o,
        pause_compliance,
        pause_restrictions,
    );
    Pallet::<T>::add_instruction_with_mediators(
        sender.origin.clone().into(),
        Some(venue_id),
        settlement_type,
        None,
        None,
        parameters.legs.clone(),
        Some(Memo::default()),
        m_identity.try_into().unwrap(),
    )
    .unwrap();
    // Affirms the sender side of the instruction
    let receipt_details: Vec<_> = (0..o)
        .map(|i| {
            setup_receipt_details(
                &sender,
                sender.did(),
                receiver.did(),
                ONE_UNIT,
                InstructionId(1),
                i,
            )
        })
        .collect();
    let sdr_holdings = parameters.sdr_holdings();
    Pallet::<T>::affirm_with_receipts(
        sender.origin.clone().into(),
        InstructionId(1),
        receipt_details.clone(),
        sdr_holdings,
    )
    .unwrap();
    let rcv_holdings = parameters.rcv_holdings();
    Pallet::<T>::affirm_with_receipts(
        receiver.origin.clone().into(),
        InstructionId(1),
        Vec::new(),
        rcv_holdings,
    )
    .unwrap();
    // All mediators must affirm the instruction
    parameters.asset_mediators.iter().for_each(|u| {
        Pallet::<T>::affirm_instruction_as_mediator(
            u.origin.clone().into(),
            InstructionId(1),
            None,
        )
        .unwrap();
    });
    m_user.into_iter().for_each(|u| {
        Pallet::<T>::affirm_instruction_as_mediator(u.origin.into(), InstructionId(1), None)
            .unwrap();
    });
    parameters
}

/// Returns the receipt details, signed by `signer`, of a transfer of `amount` for `format!("OFFTicker{}", leg_id).
fn setup_receipt_details<T: Config>(
    signer: &User<T>,
    sender_identity: IdentityId,
    receiver_identity: IdentityId,
    amount: Balance,
    instruction_id: InstructionId,
    leg_id: u32,
) -> ReceiptDetails<T::AccountId, T::OffChainSignature, T::Moment> {
    let expires_at = pallet_timestamp::Pallet::<T>::get() + 1000u32.into();
    let receipt = ChainScopedMessage::<T, _>::new_unchecked(
        leg_id as u64,
        SETTLEMENT_RECEIPT_LABEL,
        expires_at,
        Receipt::new(
            instruction_id,
            LegId(leg_id as u64),
            sender_identity,
            receiver_identity,
            Ticker::from_slice_truncated(format!("OFFTICKER{}", leg_id).as_bytes()),
            amount,
        ),
    );
    let signature = receipt
        .native_sign(
            &signer
                .secret
                .as_ref()
                .expect("User without secret key")
                .to_bytes(),
        )
        .expect("Failed to sign receipt");
    let encoded_signature = MultiSignature::from(signature).encode();
    let signature = T::OffChainSignature::decode(&mut &encoded_signature[..]).unwrap();
    ReceiptDetails::new(
        leg_id as u64,
        instruction_id,
        LegId(leg_id as u64),
        signer.account(),
        signature,
        expires_at,
        Some(ReceiptMetadata::default()),
    )
}

fn set_instruction_mediators<T>(m: u32) -> (Vec<User<T>>, BTreeSet<IdentityId>)
where
    T: Config,
{
    let m_user: Vec<User<T>> = (0..m)
        .map(|i| {
            UserBuilder::<T>::default()
                .generate_did()
                .build(&format!("InstMediator{}", i))
        })
        .collect();
    let m_identity: BTreeSet<IdentityId> = m_user.iter().map(|u| u.did()).collect();
    (m_user, m_identity)
}

benchmarks! {
    where_clause { where T: pallet_scheduler::Config }

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
        assert_eq!(VenueCounter::<T>::get(), VenueId(2), "Invalid venue counter");
        assert!(UserVenues::<T>::contains_key(did.unwrap(), VenueId(1)), "Invalid venue id");
        assert!(VenueInfo::<T>::get(VenueId(1)).is_some(), "Incorrect venue info set");
    }

    update_venue_details {
        // Variations for the venue_details length.
        let d in 1 .. MAX_VENUE_DETAILS_LENGTH;
        let details1 = VenueDetails::from(vec![b'D'; d as usize].as_slice());
        let details2 = details1.clone();

        let User { origin, did, .. } = creator::<T>();
        let venue_id = create_venue_::<T>(did.unwrap(), vec![]);
    }: _(origin, venue_id, details1)
    verify {
        assert_eq!(Details::<T>::get(venue_id), details2, "Incorrect venue details");
    }

    update_venue_type {
        let ty = VenueType::Sto;

        let User { account, origin, did, .. } = creator::<T>();
        let venue_id = create_venue_::<T>(did.unwrap(), vec![]);
    }: _(origin, venue_id, ty)
    verify {
        assert_eq!(VenueInfo::<T>::get(VenueId(1)).unwrap().venue_type, ty, "Incorrect venue type value");
    }

    update_venue_signers {
        // Variations for the no. of signers allowed.
        let s in 0 .. MAX_SIGNERS_ALLOWED;
        let mut signers = Vec::with_capacity(s as usize);
        let User {account, origin, did, .. } = creator::<T>();
        let venue_id = create_venue_::<T>(did.unwrap(), vec![account.clone()]);
        // Create signers vector.
        for signer in 0 .. s {
            signers.push(UserBuilder::<T>::default().generate_did().seed(signer).build("signers").account());
        }
    }: _(origin, venue_id, signers.clone(), true)
    verify {
        for signer in signers.iter() {
            assert_eq!(VenueSigners::<T>::get(venue_id, signer), true, "Incorrect venue signer");
        }
    }

    set_venue_filtering {
        // Constant time function. It is only for allow venue filtering.
        let user = creator::<T>();
        let ticker = create_asset_::<T>(&user);
    }: _(user.origin, ticker, true)
    verify {
        assert!(VenueFiltering::<T>::get(ticker), "Fail: set_venue_filtering failed");
    }

    allow_venues {
        // Count of venue is variant for this dispatchable.
        let v in 0 .. MAX_VENUE_ALLOWED;
        let user = creator::<T>();
        let ticker = create_asset_::<T>(&user);
        let mut venues = Vec::new();
        for i in 0 .. v {
            venues.push(VenueId(i.into()));
        }
        let s_venues = venues.clone();
    }: _(user.origin, ticker, s_venues)
    verify {
        for v in venues.iter() {
            assert!(VenueAllowList::<T>::get(ticker, v), "Fail: allow_venue dispatch");
        }
    }

    disallow_venues {
        // Count of venue is variant for this dispatchable.
        let v in 0 .. MAX_VENUE_ALLOWED;
        let user = creator::<T>();
        let ticker = create_asset_::<T>(&user);
        let mut venues = Vec::new();
        for i in 0 .. v {
            venues.push(VenueId(i.into()));
        }
        let s_venues = venues.clone();
    }: _(user.origin, ticker, s_venues)
    verify {
        for v in venues.iter() {
            assert!(!VenueAllowList::<T>::get(ticker, v), "Fail: allow_venue dispatch");
        }
    }

    affirm_with_receipts {
        // Number of fungible, non fungible and offchain assets
        let f in 1..T::MaxNumberOfFungibleAssets::get();
        let n in 0..T::MaxNumberOfNFTs::get();
        let o in 0..T::MaxNumberOfOffChainAssets::get();

        let alice = UserBuilder::<T>::default().generate_did().build("Alice");
        let bob = UserBuilder::<T>::default().generate_did().build("Bob");
        let venue_id = create_venue_::<T>(alice.did(), vec![alice.account()]);

        let parameters = setup_legs::<T>(&alice, &bob, f, n, o, false, false);
        let sdr_holdings = parameters.sdr_holdings();
        Pallet::<T>::add_instruction(
            alice.origin.clone().into(),
            Some(venue_id),
            SettlementType::SettleOnAffirmation,
            None,
            None,
            parameters.legs,
            Some(Memo::default()),
        ).unwrap();

        let receipt_details = (0..o)
            .map(|i| {
                setup_receipt_details(
                    &alice,
                    alice.did(),
                    bob.did(),
                    ONE_UNIT,
                    InstructionId(1),
                    i,
                )
            })
            .collect();
    }: _(alice.origin, InstructionId(1), receipt_details, sdr_holdings)

    execute_manual_instruction {
        let f in 0..T::MaxNumberOfFungibleAssets::get();
        let n in 0..T::MaxNumberOfNFTs::get();
        let o in 0..T::MaxNumberOfOffChainAssets::get();

        let m = T::MaxInstructionMediators::get();

        let alice = UserBuilder::<T>::default().generate_did().build("Alice");
        let bob = UserBuilder::<T>::default().generate_did().build("Bob");
        let settlement_type = SettlementType::SettleManual(0u32.into());
        let venue_id = create_venue_::<T>(alice.did(), vec![alice.account(), bob.account()]);

        let p = setup_execute_instruction::<T>(&alice, &bob, settlement_type, venue_id, f, n, o, m, false, false);
        let weight = {
            frame_support_with_transaction(|| {
                let inst_info =
                    Pallet::<T>::manual_execution_weight(InstructionId(1)).unwrap();
                TransactionOutcome::Rollback(Ok::<Weight, DispatchError>(inst_info.consumed_weight()))
            })
            .unwrap()
        };
    }: _(p.asset_mediators[0].clone().origin, InstructionId(1), None, f, n, o, Some(weight))

    add_instruction{
        // Number of fungible, non-fungible and offchain LEGS in the instruction
        let f in 1..T::MaxNumberOfFungibleAssets::get();
        let n in 0..T::MaxNumberOfNFTs::get();
        let o in 0..T::MaxNumberOfOffChainAssets::get();

        let alice = UserBuilder::<T>::default().generate_did().build("Alice");
        let bob = UserBuilder::<T>::default().generate_did().build("Bob");
        let memo = Some(Memo::default());
        let settlement_type = SettlementType::SettleOnBlock(100u32.into());
        let venue_id = create_venue_::<T>(alice.did(), vec![alice.account()]);

        let parameters = setup_legs::<T>(&alice, &bob, f, n, o, false, false);
    }: _(alice.origin, Some(venue_id), settlement_type, None, None, parameters.legs, memo)

    add_and_affirm_instruction {
        // Number of fungible, non-fungible and offchain LEGS in the instruction
        let f in 1..T::MaxNumberOfFungibleAssets::get();
        let n in 0..T::MaxNumberOfNFTs::get();
        let o in 0..T::MaxNumberOfOffChainAssets::get();

        let alice = UserBuilder::<T>::default().generate_did().build("Alice");
        let bob = UserBuilder::<T>::default().generate_did().build("Bob");
        let memo = Some(Memo::default());
        let settlement_type = SettlementType::SettleOnBlock(100u32.into());
        let venue_id = create_venue_::<T>(alice.did(), vec![alice.account()]);

        let parameters = setup_legs::<T>(&alice, &bob, f, n, o, false, false);
        let sdr_holdings = parameters.sdr_holdings();
    }: _(alice.origin, Some(venue_id), settlement_type, None, None, parameters.legs, sdr_holdings, memo)

    affirm_instruction {
        // Number of fungible and non-fungible assets in the portfolios
        let f in 1..T::MaxNumberOfFungibleAssets::get() as u32;
        let n in 1..T::MaxNumberOfNFTs::get() as u32;

        let alice = UserBuilder::<T>::default().generate_did().build("Alice");
        let bob = UserBuilder::<T>::default().generate_did().build("Bob");
        let memo = Some(Memo::default());
        let venue_id = create_venue_::<T>(alice.did(), vec![alice.account()]);

        let parameters = setup_legs::<T>(&alice, &bob, f, n, T::MaxNumberOfOffChainAssets::get(), false, false);
        let sdr_holdings = parameters.sdr_holdings();
        Pallet::<T>::add_instruction(
            alice.origin.clone().into(),
            Some(venue_id),
            SettlementType::SettleOnAffirmation,
            None,
            None,
            parameters.legs,
            memo,
        ).unwrap();
    }: _(alice.origin, InstructionId(1), sdr_holdings)

    withdraw_affirmation {
        // Number of fungible, non-fungible and offchain LEGS in the portfolios
        let f in 1..T::MaxNumberOfFungibleAssets::get();
        let n in 0..T::MaxNumberOfNFTs::get();
        let o in 0..T::MaxNumberOfOffChainAssets::get();

        let m = T::MaxInstructionMediators::get();

        let alice = UserBuilder::<T>::default().generate_did().build("Alice");
        let bob = UserBuilder::<T>::default().generate_did().build("Bob");
        let settlement_type = SettlementType::SettleOnBlock(100u32.into());
        let venue_id = create_venue_::<T>(alice.did(), vec![alice.account(), bob.account()]);

        let parameters = setup_execute_instruction::<T>(&alice, &bob, settlement_type, venue_id, f, n, o, m, false, false);
        let sdr_holdings = parameters.sdr_holdings();
    }: _(alice.origin, InstructionId(1),  sdr_holdings)

    execute_instruction_paused {
        // Number of fungible, non-fungible and offchain assets in the instruction
        let f in 1..T::MaxNumberOfFungibleAssets::get() as u32;
        let n in 0..T::MaxNumberOfNFTs::get() as u32;
        let o in 0..T::MaxNumberOfOffChainAssets::get();

        let m = T::MaxInstructionMediators::get();

        let alice = UserBuilder::<T>::default().generate_did().build("Alice");
        let bob = UserBuilder::<T>::default().generate_did().build("Bob");
        let venue_id = create_venue_::<T>(alice.did(), vec![alice.account(), bob.account()]);

        setup_execute_instruction::<T>(&alice, &bob, SettlementType::SettleOnAffirmation, venue_id, f, n, o, m, true, true);
    }: execute_scheduled_instruction(RawOrigin::Root, InstructionId(1), Weight::MAX)

    execute_scheduled_instruction {
        // Number of fungible, non-fungible and offchain assets in the instruction
        let f in 1..T::MaxNumberOfFungibleAssets::get() as u32;
        let n in 0..T::MaxNumberOfNFTs::get() as u32;
        let o in 0..T::MaxNumberOfOffChainAssets::get();

        let m = T::MaxInstructionMediators::get();

        let alice = UserBuilder::<T>::default().generate_did().build("Alice");
        let bob = UserBuilder::<T>::default().generate_did().build("Bob");
        let venue_id = create_venue_::<T>(alice.did(), vec![alice.account(), bob.account()]);

        setup_execute_instruction::<T>(&alice, &bob, SettlementType::SettleOnAffirmation, venue_id, f, n, o, m, false, false);
    }: _(RawOrigin::Root, InstructionId(1), Weight::MAX)

    ensure_root_origin {
        let origin = RawOrigin::Root;
    }: {
        assert!(Pallet::<T>::ensure_root_origin(origin.into()).is_ok());
    }

    affirm_with_receipts_rcv {
        // Number of fungible, non fungible and offchain assets
        let f in 1..T::MaxNumberOfFungibleAssets::get();
        let n in 0..T::MaxNumberOfNFTs::get();
        let o in 0..T::MaxNumberOfOffChainAssets::get();

        let alice = UserBuilder::<T>::default().generate_did().build("Alice");
        let bob = UserBuilder::<T>::default().generate_did().build("Bob");
        let venue_id = create_venue_::<T>(alice.did(), vec![alice.account()]);

        let parameters = setup_legs::<T>(&alice, &bob, f, n, o, false, false);
        let rcv_holdings = parameters.rcv_holdings();
        Pallet::<T>::add_instruction(
            alice.origin.clone().into(),
            Some(venue_id),
            SettlementType::SettleOnAffirmation,
            None,
            None,
            parameters.legs,
            Some(Memo::default()),
        ).unwrap();

        let receipt_details = (0..o)
            .map(|i| {
                setup_receipt_details(
                    &alice,
                    alice.did(),
                    bob.did(),
                    ONE_UNIT,
                    InstructionId(1),
                    i,
                )
            })
            .collect();
    }: affirm_with_receipts(bob.origin, InstructionId(1), receipt_details, rcv_holdings)

    affirm_instruction_rcv {
        // Number of fungible and non-fungible assets in the portfolios
        let f in 1..T::MaxNumberOfFungibleAssets::get() as u32;
        let n in 1..T::MaxNumberOfNFTs::get() as u32;

        let alice = UserBuilder::<T>::default().generate_did().build("Alice");
        let bob = UserBuilder::<T>::default().generate_did().build("Bob");
        let memo = Some(Memo::default());
        let venue_id = create_venue_::<T>(alice.did(), vec![alice.account()]);

        let parameters = setup_legs::<T>(&alice, &bob, f, n, T::MaxNumberOfOffChainAssets::get(), false, false);
        let rcv_holdings = parameters.rcv_holdings();
        Pallet::<T>::add_instruction(
            alice.origin.clone().into(),
            Some(venue_id),
            SettlementType::SettleOnAffirmation,
            None,
            None,
            parameters.legs,
            memo,
        ).unwrap();
    }: affirm_instruction(bob.origin, InstructionId(1), rcv_holdings)

    withdraw_affirmation_rcv {
        // Number of fungible, non-fungible and offchain LEGS in the portfolios
        let f in 1..T::MaxNumberOfFungibleAssets::get();
        let n in 0..T::MaxNumberOfNFTs::get();
        let o in 0..T::MaxNumberOfOffChainAssets::get();

        let m = T::MaxInstructionMediators::get();

        let alice = UserBuilder::<T>::default().generate_did().build("Alice");
        let bob = UserBuilder::<T>::default().generate_did().build("Bob");
        let settlement_type = SettlementType::SettleOnBlock(100u32.into());
        let venue_id = create_venue_::<T>(alice.did(), vec![alice.account(), bob.account()]);

        let parameters = setup_execute_instruction::<T>(&alice, &bob, settlement_type, venue_id, f, n, o, m, false, false);
        let rcv_holdings = parameters.rcv_holdings();
    }: withdraw_affirmation(bob.origin, InstructionId(1),  rcv_holdings)

    add_instruction_with_mediators {
        // Number of fungible, non-fungible, offchain legs and mediators
        let f in 1..T::MaxNumberOfFungibleAssets::get();
        let n in 0..T::MaxNumberOfNFTs::get();
        let o in 0..T::MaxNumberOfOffChainAssets::get();
        let m in 0..T::MaxInstructionMediators::get();

        let bob = UserBuilder::<T>::default().generate_did().build("Bob");
        let alice = UserBuilder::<T>::default().generate_did().build("Alice");
        let memo = Some(Memo::default());
        let settlement_type = SettlementType::SettleOnBlock(100u32.into());
        let venue_id = create_venue_::<T>(alice.did(), vec![alice.account()]);
        let mediators: BTreeSet<IdentityId> = (0..m).map(|i| IdentityId::from(i as u128)).collect();

        let parameters = setup_legs::<T>(&alice, &bob, f, n, o, false, false);
    }: _(alice.origin, Some(venue_id), settlement_type, None, None, parameters.legs, memo, mediators.try_into().unwrap())

    add_and_affirm_with_mediators {
        // Number of fungible, non-fungible, offchain legs and mediators
        let f in 1..T::MaxNumberOfFungibleAssets::get();
        let n in 0..T::MaxNumberOfNFTs::get();
        let o in 0..T::MaxNumberOfOffChainAssets::get();
        let m in 0..T::MaxInstructionMediators::get();

        let alice = UserBuilder::<T>::default().generate_did().build("Alice");
        let bob = UserBuilder::<T>::default().generate_did().build("Bob");
        let memo = Some(Memo::default());
        let settlement_type = SettlementType::SettleOnBlock(100u32.into());
        let venue_id = create_venue_::<T>(alice.did(), vec![alice.account()]);
        let mediators: BTreeSet<IdentityId> = (0..m).map(|i| IdentityId::from(i as u128)).collect();

        let parameters = setup_legs::<T>(&alice, &bob, f, n, o, false, false);
        let sdr_holdings = parameters.sdr_holdings();
    }: _(alice.origin, Some(venue_id), settlement_type, None, None, parameters.legs, sdr_holdings, memo, mediators.try_into().unwrap())

    affirm_instruction_as_mediator {
        let bob = UserBuilder::<T>::default().generate_did().build("Bob");
        let david = UserBuilder::<T>::default().generate_did().build("David");
        let alice = UserBuilder::<T>::default().generate_did().build("Alice");
        let memo = Some(Memo::default());
        let venue_id = create_venue_::<T>(alice.did(), vec![alice.account()]);
        let mediators = BTreeSet::from([david.did()]);

        let parameters = setup_legs::<T>(
            &alice,
            &bob,
            T::MaxNumberOfFungibleAssets::get(),
            T::MaxNumberOfNFTs::get(),
            T::MaxNumberOfOffChainAssets::get(),
            false,
            false,
        );
        Pallet::<T>::add_instruction_with_mediators(
            alice.origin.clone().into(),
            Some(venue_id),
            SettlementType::SettleOnAffirmation,
            None,
            None,
            parameters.legs,
            memo,
            mediators.try_into().unwrap()
        ).unwrap();
    }: _(david.origin, InstructionId(1), None)


    withdraw_affirmation_as_mediator {
        let bob = UserBuilder::<T>::default().generate_did().build("Bob");
        let david = UserBuilder::<T>::default().generate_did().build("David");
        let alice = UserBuilder::<T>::default().generate_did().build("Alice");
        let memo = Some(Memo::default());
        let venue_id = create_venue_::<T>(alice.did(), vec![alice.account()]);
        let mediators = BTreeSet::from([david.did()]);

        let parameters = setup_legs::<T>(
            &alice,
            &bob,
            T::MaxNumberOfFungibleAssets::get(),
            T::MaxNumberOfNFTs::get(),
            T::MaxNumberOfOffChainAssets::get(),
            false,
            false,
        );
        Pallet::<T>::add_instruction_with_mediators(
            alice.origin.clone().into(),
            Some(venue_id),
            SettlementType::SettleOnAffirmation,
            None,
            None,
            parameters.legs,
            memo,
            mediators.try_into().unwrap()
        ).unwrap();
        Pallet::<T>::affirm_instruction_as_mediator(
            david.origin.clone().into(),
            InstructionId(1),
            None
        ).unwrap();
    }: _(david.origin, InstructionId(1))

    base_reject_instruction {
        let f in 0..T::MaxNumberOfFungibleAssets::get();
        let n in 0..T::MaxNumberOfNFTs::get();
        let o in 0..T::MaxNumberOfOffChainAssets::get();

        let m = T::MaxInstructionMediators::get();

        let alice = UserBuilder::<T>::default().generate_did().build("Alice");
        let bob = UserBuilder::<T>::default().generate_did().build("Bob");
        let settlement_type = SettlementType::SettleManual(0u32.into());
        let venue_id = create_venue_::<T>(alice.did(), vec![alice.account(), bob.account()]);

        let inst_id = InstructionId(1);
        let parameters = setup_execute_instruction::<T>(&alice, &bob, settlement_type, venue_id, f, n, o, m, false, false);
        let inst_legs: Vec<_> = InstructionLegs::<T>::iter_prefix(&inst_id).collect();
    }: {
        Pallet::<T>::base_reject_instruction(
            parameters.asset_mediators[0].clone().origin.into(),
            inst_id,
            None,
            Some(AssetCount::new(f, n, o)),
            false,
            &mut WeightMeter::max_limit_no_minimum()
        )
        .unwrap()
    }

    lock_instruction_extrinsic {
        let f in 0..T::MaxNumberOfFungibleAssets::get();
        let n in 0..T::MaxNumberOfNFTs::get();
        let o in 0..T::MaxNumberOfOffChainAssets::get();

        let m = T::MaxInstructionMediators::get();

        let alice = UserBuilder::<T>::default().generate_did().build("Alice");
        let bob = UserBuilder::<T>::default().generate_did().build("Bob");
        let settlement_type = SettlementType::SettleAfterLock;
        let venue_id = create_venue_::<T>(alice.did(), vec![alice.account(), bob.account()]);

        let inst_id = InstructionId(1);
        let parameters = setup_execute_instruction::<T>(&alice, &bob, settlement_type, venue_id, f, n, o, m, true, true);
        let weight = {
            frame_support_with_transaction(|| {
                let weight = Pallet::<T>::lock_instruction_weight(InstructionId(1)).unwrap();
                TransactionOutcome::Rollback(Ok::<Weight, DispatchError>(weight))
            })
            .unwrap()
        };
    }: {
        Pallet::<T>::lock_instruction(
            parameters.asset_mediators[0].clone().origin.into(),
            inst_id,
            weight
        )
        .unwrap();
    }

    execute_locked_instruction {
        let f in 0..T::MaxNumberOfFungibleAssets::get();
        let n in 0..T::MaxNumberOfNFTs::get();
        let o in 0..T::MaxNumberOfOffChainAssets::get();

        let m = T::MaxInstructionMediators::get();

        let alice = UserBuilder::<T>::default().generate_did().build("Alice");
        let bob = UserBuilder::<T>::default().generate_did().build("Bob");
        let settlement_type = SettlementType::SettleAfterLock;
        let venue_id = create_venue_::<T>(alice.did(), vec![alice.account(), bob.account()]);

        let p = setup_execute_instruction::<T>(&alice, &bob, settlement_type, venue_id, f, n, o, m, false, false);

        Pallet::<T>::base_lock_instruction(
            p.asset_mediators[0].clone().origin.into(),
            InstructionId(1),
            false,
            &mut WeightMeter::max_limit_no_minimum(),
        )
        .unwrap();

        let weight = {
            frame_support_with_transaction(|| {
                let inst_info =
                    Pallet::<T>::manual_execution_weight(InstructionId(1)).unwrap();
                TransactionOutcome::Rollback(Ok::<Weight, DispatchError>(inst_info.consumed_weight()))
            })
            .unwrap()
        };
    }: {
        Pallet::<T>::execute_manual_instruction(
            p.asset_mediators[0].clone().origin.into(),
            InstructionId(1),
            None,
            f,
            n,
            o,
            Some(weight),
        )
        .unwrap();
    }

    execute_manual_instruction_paused {
        let f in 0..T::MaxNumberOfFungibleAssets::get();
        let n in 0..T::MaxNumberOfNFTs::get();
        let o in 0..T::MaxNumberOfOffChainAssets::get();

        let m = T::MaxInstructionMediators::get();

        let alice = UserBuilder::<T>::default().generate_did().build("Alice");
        let bob = UserBuilder::<T>::default().generate_did().build("Bob");
        let settlement_type = SettlementType::SettleManual(0u32.into());
        let venue_id = create_venue_::<T>(alice.did(), vec![alice.account(), bob.account()]);

        let p = setup_execute_instruction::<T>(&alice, &bob, settlement_type, venue_id, f, n, o, m, true, true);

        let weight = {
            frame_support_with_transaction(|| {
                let inst_info =
                    Pallet::<T>::manual_execution_weight(InstructionId(1)).unwrap();
                TransactionOutcome::Rollback(Ok::<Weight, DispatchError>(inst_info.consumed_weight()))
            })
            .unwrap()
        };
    }: {
        Pallet::<T>::execute_manual_instruction(
            p.asset_mediators[0].clone().origin.into(),
            InstructionId(1),
            None,
            f,
            n,
            o,
            Some(weight),
        )
        .unwrap();
    }

    set_mandatory_receiver_affirmation {
        let alice = UserBuilder::<T>::default().generate_did().build("Alice");
    }: _(alice.origin, AffirmationRequirement::Required)

    transfer_funds_portfolio_same_did {
        let alice = UserBuilder::<T>::default().generate_did().build("Alice");
        let bob = UserBuilder::<T>::default().generate_did().build("Bob");

        let (sender_holding, _receiver_holding, _, asset_id) =
            setup_asset_transfer::<T>(&alice, &bob, None, None, true, true, 0, true, false);

        let to = AssetHolder::from(polymesh_primitives::PortfolioId {
            did: alice.did(),
            kind: polymesh_primitives::PortfolioKind::Default,
        });
        let fund = Fund {
            description: FundDescription::Fungible { asset_id, amount: ONE_UNIT },
            memo: None,
        };
    }: transfer_funds(alice.origin.clone(), Some(sender_holding), to.clone(), fund)
    verify {
        assert_eq!(pallet_asset::Pallet::<T>::get_holders_balance(&to, &asset_id), ONE_UNIT);
    }

    transfer_funds_portfolio_diff_did {
        let alice = UserBuilder::<T>::default().generate_did().build("Alice");
        let bob = UserBuilder::<T>::default().generate_did().build("Bob");

        let (sender_holding, receiver_holding, _, asset_id) =
            setup_asset_transfer::<T>(&alice, &bob, None, None, false, false, 0, true, false);

        // Undo mandatory affirmation so receiver auto-affirms and instruction executes.
        Pallet::<T>::set_mandatory_receiver_affirmation(
            bob.origin().into(),
            AffirmationRequirement::Automatic,
        )
        .unwrap();

        let fund = Fund {
            description: FundDescription::Fungible { asset_id, amount: ONE_UNIT },
            memo: None,
        };
    }: transfer_funds(alice.origin.clone(), Some(sender_holding), receiver_holding.clone(), fund)
    verify {
        // Receiver auto-affirms → instruction executes with compliance checks.
        assert_eq!(pallet_asset::Pallet::<T>::get_holders_balance(&receiver_holding, &asset_id), ONE_UNIT);
    }

    transfer_funds_account_same_did {
        let alice = UserBuilder::<T>::default().generate_did().build("Alice");
        let bob = UserBuilder::<T>::default().generate_did().build("Bob");

        let (_sender_holding, _receiver_holding, _, asset_id) =
            setup_asset_transfer::<T>(&alice, &bob, None, None, true, true, 0, false, true);

        pallet_asset::Pallet::<T>::approve(
            alice.origin().into(),
            asset_id,
            bob.account(),
            ONE_UNIT * 10,
        )
        .unwrap();

        let from = Some(AssetHolder::try_from(alice.account().encode()).unwrap());
        let to = AssetHolder::from(polymesh_primitives::PortfolioId {
            did: alice.did(),
            kind: polymesh_primitives::PortfolioKind::Default,
        });
        let fund = Fund {
            description: FundDescription::Fungible { asset_id, amount: ONE_UNIT },
            memo: None,
        };
    }: transfer_funds(bob.origin.clone(), from, to.clone(), fund)
    verify {
        assert_eq!(pallet_asset::Pallet::<T>::get_holders_balance(&to, &asset_id), ONE_UNIT);
    }

    transfer_funds_account_diff_did {
        let alice = UserBuilder::<T>::default().generate_did().build("Alice");
        let bob = UserBuilder::<T>::default().generate_did().build("Bob");

        let (_sender_holding, _receiver_holding, _, asset_id) =
            setup_asset_transfer::<T>(&alice, &bob, None, None, false, false, 0, false, true);

        pallet_asset::Pallet::<T>::approve(
            alice.origin().into(),
            asset_id,
            bob.account(),
            ONE_UNIT * 10,
        )
        .unwrap();

        let from = Some(AssetHolder::try_from(alice.account().encode()).unwrap());
        let to = AssetHolder::try_from(bob.account().encode()).unwrap();
        let fund = Fund {
            description: FundDescription::Fungible { asset_id, amount: ONE_UNIT },
            memo: None,
        };
    }: transfer_funds(bob.origin.clone(), from, to.clone(), fund)
    verify {
        // Allowance decremented.
        assert_eq!(
            pallet_asset::Allowances::<T>::get((&alice.account(), &bob.account(), asset_id)),
            ONE_UNIT * 9
        );
        // Instruction executed (caller is receiver → auto-affirm → execution).
        assert_eq!(pallet_asset::Pallet::<T>::get_holders_balance(&to, &asset_id), ONE_UNIT);
    }
}
