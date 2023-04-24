// This file is part of the Polymesh distribution (https://github.com/PolymeshAssociation/Polymesh).
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

pub use frame_benchmarking::{account, benchmarks};
use frame_support::traits::Get;
use frame_system::RawOrigin;
use scale_info::prelude::format;
use sp_std::prelude::*;

use pallet_asset::benchmarking::setup_asset_transfer;
use pallet_nft::benchmarking::{create_collection_issue_nfts, setup_nft_transfer};
use pallet_portfolio::PortfolioAssetBalances;
use polymesh_common_utilities::benchs::{make_asset, user, AccountIdOf, User, UserBuilder};
use polymesh_common_utilities::constants::currency::{ONE_UNIT, POLY};
use polymesh_common_utilities::constants::ENSURED_MAX_LEN;
use polymesh_common_utilities::TestUtilsFn;
use polymesh_primitives::asset::NonFungibleType;
use polymesh_primitives::checked_inc::CheckedInc;
use polymesh_primitives::{
    IdentityId, NFTId, NFTs, PortfolioId, PortfolioKind, PortfolioNumber, Ticker,
};

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

pub struct BaseV2Parameters<T: Config> {
    pub sender: User<T>,
    pub receiver: User<T>,
    pub fungible_ticker: Ticker,
    pub nft_ticker: Ticker,
    pub venue_id: VenueId,
    pub legs_v2: Vec<LegV2>,
    pub sender_portfolios: Vec<PortfolioId>,
    pub settlement_type: SettlementType<T::BlockNumber>,
    pub date: Option<T::Moment>,
    pub memo: Option<InstructionMemo>,
}

fn creator<T: Config + TestUtilsFn<AccountIdOf<T>>>() -> User<T> {
    UserBuilder::<T>::default().generate_did().build("creator")
}

/// Set venue related storage without any sanity checks.
fn create_venue_<T: Config>(did: IdentityId, signers: Vec<T::AccountId>) -> VenueId {
    let venue = Venue {
        creator: did,
        venue_type: VenueType::Distribution,
    };
    // NB: Venue counter starts with 1.
    let venue_counter = Module::<T>::venue_counter();
    VenueInfo::insert(venue_counter, venue);
    for signer in signers {
        <VenueSigners<T>>::insert(venue_counter, signer, true);
    }
    VenueCounter::put(venue_counter.checked_inc().unwrap());
    venue_counter
}

pub fn create_asset_<T: Config>(owner: &User<T>) -> Ticker {
    make_asset::<T>(owner, Some(&Ticker::generate(8u64)))
}

/// Creates a fungible asset for the given `ticker` and returns a `Vec<LegV2>` containing `n_legs`.
fn setup_fungible_legs_v2<T: Config>(
    sender: User<T>,
    receiver: User<T>,
    ticker: Ticker,
    n_legs: u32,
) -> Vec<LegV2> {
    make_asset(&sender, Some(ticker.as_ref()));
    (0..n_legs)
        .map(|_| LegV2 {
            from: PortfolioId {
                did: sender.did(),
                kind: PortfolioKind::Default,
            },
            to: PortfolioId {
                did: receiver.did(),
                kind: PortfolioKind::Default,
            },
            asset: LegAsset::Fungible {
                ticker: ticker.clone(),
                amount: ONE_UNIT,
            },
        })
        .collect()
}

/// Creates an nft collection for `ticker`, mints `n_nfts` for `token_sender`, and returns a `Vec<LegV2>`
/// containing `n_legs` with a total of `n_nfts` split among the legs.
/// For this function only calls with the minimum number of legs to contain all `n_nfts` are allowed.
/// E.g: If n_nfts = 78, n_legs must be equal to 8 (considering that MaxNumberOfNFTsPerLeg is equal to 10).
fn setup_nft_legs<T: Config>(
    sender: User<T>,
    receiver: User<T>,
    ticker: Ticker,
    n_legs: u32,
    n_nfts: u32,
    max_nfts_per_leg: Option<u32>,
) -> Vec<LegV2> {
    create_collection_issue_nfts::<T>(
        sender.origin().into(),
        ticker,
        Some(NonFungibleType::Derivative),
        0,
        n_nfts,
        PortfolioKind::Default,
    );

    let max_nfts_per_leg = max_nfts_per_leg.unwrap_or(T::MaxNumberOfNFTsPerLeg::get());
    let last_leg_len = n_nfts % max_nfts_per_leg;
    let full_legs = n_nfts / max_nfts_per_leg;

    // Creates the NFTs for each leg. All legs except the last one will have T::MaxNumberOfNFTsPerLeg NFTs each.
    let mut nfts: Vec<NFTs> = (0..full_legs)
        .map(|leg_index| {
            NFTs::new(
                ticker,
                (0..max_nfts_per_leg)
                    .map(|nft_index| NFTId((leg_index * max_nfts_per_leg + nft_index + 1) as u64))
                    .collect(),
            )
            .unwrap()
        })
        .collect();
    // The last leg may have less than T::MaxNumberOfNFTsPerLeg NFTs
    if last_leg_len > 0 {
        nfts.push(NFTs::new_unverified(
            ticker,
            (0..last_leg_len)
                .map(|nft_index| {
                    NFTId((max_nfts_per_leg * (nfts.len() as u32) + nft_index + 1) as u64)
                })
                .collect(),
        ));
    }
    // For this function only calls with the minimum number of legs to contain all `n_nfts` are allowed
    assert_eq!(nfts.len() as u32, n_legs);
    // Creates each leg
    (0..n_legs)
        .map(|index| LegV2 {
            from: PortfolioId {
                did: sender.did(),
                kind: PortfolioKind::Default,
            },
            to: PortfolioId {
                did: receiver.did(),
                kind: PortfolioKind::Default,
            },

            asset: LegAsset::NonFungible(nfts[index as usize].clone()),
        })
        .collect()
}

/// Creates the basic environment for executing the benchmarks for the v2 extrinsics.
/// This includes: creating one fungible asset, one NFT collection with `n_nfts`, and
/// the instruction legs filled with `n_nfts` and `fungible_transfers`.
/// All other parameters are also included in the `BaseV2Parameters` struct.
fn setup_v2_extrinsics_parameters<T>(fungible_transfers: u32, n_nfts: u32) -> BaseV2Parameters<T>
where
    T: TestUtilsFn<AccountIdOf<T>> + Config,
{
    let max_nfts = T::MaxNumberOfNFTsPerLeg::get();
    let alice = UserBuilder::<T>::default().generate_did().build("Alice");
    let bob = UserBuilder::<T>::default().generate_did().build("Bob");
    let fungible_ticker: Ticker = Ticker::from_slice_truncated(b"TICKER".as_ref());
    let nft_ticker: Ticker = Ticker::from_slice_truncated(b"TICKER0".as_ref());
    let venue_id = create_venue_::<T>(alice.did(), vec![]);
    let sender_portfolios = vec![PortfolioId {
        did: alice.did(),
        kind: PortfolioKind::Default,
    }];

    let mut fungible_legs = setup_fungible_legs_v2(
        alice.clone(),
        bob.clone(),
        fungible_ticker,
        fungible_transfers,
    );
    let n_nft_legs = if n_nfts % max_nfts == 0 {
        n_nfts / max_nfts
    } else {
        n_nfts / max_nfts + 1
    };
    let mut non_fungible_legs = setup_nft_legs(
        alice.clone(),
        bob.clone(),
        nft_ticker,
        n_nft_legs,
        n_nfts,
        None,
    );
    non_fungible_legs.append(&mut fungible_legs);
    let legs_v2 = non_fungible_legs;

    let settlement_type = SettlementType::SettleOnBlock(100u32.into());
    let date = Some(99999999u32.into());
    let memo = Some(InstructionMemo::default());

    BaseV2Parameters::<T> {
        sender: alice,
        receiver: bob,
        fungible_ticker,
        nft_ticker,
        venue_id,
        legs_v2,
        sender_portfolios,
        settlement_type,
        date,
        memo,
    }
}

/// Creates a venue, creates `f` assets and `n` collections, adds and affirms both sides of a settlement instruction.
/// Each leg will transfer between custom portfolios, all tickers have the maximum number of compliance requirements, active
/// statistics and transfer restrictions.
fn setup_execute_instruction<T>(
    sender: &User<T>,
    receiver: &User<T>,
    settlement_type: SettlementType<T::BlockNumber>,
    f: u32,
    n: u32,
    pause_compliance: bool,
    pause_restrictions: bool,
) where
    T: Config + TestUtilsFn<AccountIdOf<T>>,
{
    let venue_id = create_venue_::<T>(sender.did(), vec![]);
    let mut sender_portfolios = Vec::new();
    let mut receiver_portfolios = Vec::new();

    // Creates one asset, creates two portfolios, adds maximum compliance requirements, adds maximum transfer conditions and pauses them.
    let fungible_legs: Vec<LegV2> = (0..f)
        .map(|i| {
            let ticker = Ticker::from_slice_truncated(format!("Ticker{}", i).as_bytes());
            let sdr_portfolio_name = format!("SdrPortfolioTicker{}", i);
            let rcv_portfolio_name = format!("RcvPortfolioTicker{}", i);
            let (sdr_portfolio, rvc_portfolio) = setup_asset_transfer(
                sender,
                receiver,
                ticker,
                Some(&sdr_portfolio_name),
                Some(&rcv_portfolio_name),
                pause_compliance,
                pause_restrictions,
            );
            sender_portfolios.push(sdr_portfolio.clone());
            receiver_portfolios.push(rvc_portfolio.clone());
            LegV2 {
                from: sdr_portfolio,
                to: rvc_portfolio,
                asset: LegAsset::Fungible {
                    ticker,
                    amount: ONE_UNIT,
                },
            }
        })
        .collect();

    // Creates one collection, mints one NFT, creates two portfolios, adds maximum compliance requirements and pauses it.
    let nft_legs: Vec<LegV2> = (0..n)
        .map(|i| {
            let ticker = Ticker::from_slice_truncated(format!("NFTTicker{}", i).as_bytes());
            let sdr_portfolio_name = format!("SdrPortfolioNFTTicker{}", i);
            let rcv_portfolio_name = format!("RcvPortfolioNFTTicker{}", i);
            let (sdr_portfolio, rcv_portfolio) = setup_nft_transfer(
                sender,
                receiver,
                ticker,
                1,
                Some(&sdr_portfolio_name),
                Some(&rcv_portfolio_name),
                pause_compliance,
            );
            sender_portfolios.push(sdr_portfolio.clone());
            receiver_portfolios.push(rcv_portfolio.clone());
            LegV2 {
                from: sdr_portfolio,
                to: rcv_portfolio,
                asset: LegAsset::NonFungible(NFTs::new_unverified(ticker, vec![NFTId(1)])),
            }
        })
        .collect();

    // Adds and affirms both sides of the instruction before executing it.
    let legs_v2 = [fungible_legs, nft_legs].concat();
    Module::<T>::add_and_affirm_instruction_with_memo_v2(
        sender.origin().into(),
        venue_id,
        settlement_type,
        None,
        None,
        legs_v2,
        sender_portfolios,
        None,
        None,
    )
    .unwrap();

    Module::<T>::affirm_instruction_v2(
        receiver.origin().into(),
        InstructionId(1),
        receiver_portfolios,
        f,
        n,
        None,
    )
    .unwrap();
}

benchmarks! {
    where_clause { where T: TestUtilsFn<AccountIdOf<T>>, T: pallet_scheduler::Config }

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
        assert_eq!(Module::<T>::venue_counter(), VenueId(2), "Invalid venue counter");
        assert_eq!(Module::<T>::user_venues(did.unwrap()).into_iter().last(), Some(VenueId(1)), "Invalid venue id");
        assert!(Module::<T>::venue_info(VenueId(1)).is_some(), "Incorrect venue info set");
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
        assert_eq!(Module::<T>::details(venue_id), details2, "Incorrect venue details");
    }

    update_venue_type {
        let ty = VenueType::Sto;

        let User { account, origin, did, .. } = creator::<T>();
        let venue_id = create_venue_::<T>(did.unwrap(), vec![]);
    }: _(origin, venue_id, ty)
    verify {
        assert_eq!(Module::<T>::venue_info(VenueId(1)).unwrap().venue_type, ty, "Incorrect venue type value");
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
            assert_eq!(Module::<T>::venue_signers(venue_id, signer), true, "Incorrect venue signer");
        }
    }

    set_venue_filtering {
        // Constant time function. It is only for allow venue filtering.
        let user = creator::<T>();
        let ticker = create_asset_::<T>(&user);
    }: _(user.origin, ticker, true)
    verify {
        assert!(Module::<T>::venue_filtering(ticker), "Fail: set_venue_filtering failed");
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
            assert!(Module::<T>::venue_allow_list(ticker, v), "Fail: allow_venue dispatch");
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
            assert!(!Module::<T>::venue_allow_list(ticker, v), "Fail: allow_venue dispatch");
        }
    }

    //affirm_with_receipts {}: _()
    //verify { }

    change_receipt_validity {
        let signer = user::<T>("signer", 0);
    }: _(signer.origin(), 0, false)
    verify {
        assert!(Module::<T>::receipts_used(&signer.account(), 0), "Settlement: change_receipt_validity didn't work");
    }

    //reschedule_instruction {}: _()
    //verify {}

    execute_manual_instruction {
        let l in 1..T::MaxNumberOfFungibleAssets::get() + T::MaxNumberOfNFTs::get();

        let alice = UserBuilder::<T>::default().generate_did().build("Alice");
        let bob = UserBuilder::<T>::default().generate_did().build("Bob");
        let ticker: Ticker = Ticker::from_slice_truncated(b"Ticker0".as_ref());
        let portfolio_id = PortfolioId {
            did: alice.did(),
            kind: PortfolioKind::User(PortfolioNumber(1)),
        };

        if l <= T::MaxNumberOfFungibleAssets::get() {
            setup_execute_instruction::<T>(
                &alice,
                &bob,
                SettlementType::SettleManual(0u32.into()),
                l,
                0,
                false,
                false
            );
        } else {
            setup_execute_instruction::<T>(
                &alice,
                &bob,
                SettlementType::SettleManual(0u32.into()),
                T::MaxNumberOfFungibleAssets::get(),
                l - T::MaxNumberOfFungibleAssets::get(),
                false,
                false
            );
        }

        let before_transfer_balance = PortfolioAssetBalances::get(portfolio_id, ticker);
        assert_eq!(before_transfer_balance, POLY * ONE_UNIT);
    }: _(alice.origin, InstructionId(1), l, Some(portfolio_id), None)
    verify {
        let after_transfer_balance = PortfolioAssetBalances::get(portfolio_id, ticker);
        assert_eq!(after_transfer_balance, before_transfer_balance - ONE_UNIT);
    }

    add_instruction_with_memo_v2 {
        let f in 1..T::MaxNumberOfFungibleAssets::get() as u32;

        let parameters = setup_v2_extrinsics_parameters::<T>(f, T::MaxNumberOfNFTs::get());
    }: _(parameters.sender.origin, parameters.venue_id, parameters.settlement_type, parameters.date, parameters.date, parameters.legs_v2, parameters.memo, None)

    add_and_affirm_instruction_with_memo_v2 {
        let f in 1..T::MaxNumberOfFungibleAssets::get() as u32;
        let n in 1..T::MaxNumberOfNFTs::get() as u32;

        let parameters = setup_v2_extrinsics_parameters::<T>(f, n);
    }: _(parameters.sender.origin, parameters.venue_id, parameters.settlement_type, parameters.date, parameters.date, parameters.legs_v2, parameters.sender_portfolios, parameters.memo, None)

    affirm_instruction_v2 {
        let f in 1..T::MaxNumberOfFungibleAssets::get() as u32;
        let n in 1..T::MaxNumberOfNFTs::get() as u32;

        let parameters = setup_v2_extrinsics_parameters::<T>(f, n);
        Module::<T>::add_instruction_with_memo_v2(
            parameters.sender.clone().origin.into(),
            parameters.venue_id,
            parameters.settlement_type,
            parameters.date,
            parameters.date,
            parameters.legs_v2.clone(),
            parameters.memo,
            None
        ).expect("failed to add instruction");
    }: _(parameters.sender.origin, InstructionId(1), parameters.sender_portfolios, f, n, None)

    withdraw_affirmation_v2 {
        let f in 1..T::MaxNumberOfFungibleAssets::get() as u32;
        let n in 1..T::MaxNumberOfNFTs::get() as u32;

        let parameters = setup_v2_extrinsics_parameters::<T>(f, n);
        Module::<T>::add_and_affirm_instruction_with_memo_v2(
            parameters.sender.clone().origin.into(),
            parameters.venue_id,
            parameters.settlement_type,
            parameters.date,
            parameters.date,
            parameters.legs_v2.clone(),
            parameters.sender_portfolios.clone(),
            parameters.memo,
            None
        ).expect("failed to add instruction");
    }: _(parameters.sender.origin, InstructionId(1), parameters.sender_portfolios, f, n)

    reject_instruction_v2 {
        let f in 1..T::MaxNumberOfFungibleAssets::get() as u32;
        let n in 1..T::MaxNumberOfNFTs::get() as u32;

        let parameters = setup_v2_extrinsics_parameters::<T>(f, n);
        Module::<T>::add_and_affirm_instruction_with_memo_v2(
            parameters.sender.clone().origin.into(),
            parameters.venue_id,
            parameters.settlement_type,
            parameters.date,
            parameters.date,
            parameters.legs_v2.clone(),
            parameters.sender_portfolios.clone(),
            parameters.memo,
            None
        ).expect("failed to add instruction");
    }: _(parameters.sender.origin, InstructionId(1), parameters.sender_portfolios[0], f, n)

    ensure_allowed_venue {
        // Number of UNIQUE tickers in the legs
        let n in 1..T::MaxNumberOfFungibleAssets::get() + T::MaxNumberOfNFTs::get();

        let alice = UserBuilder::<T>::default().generate_did().build("Alice");
        let bob = UserBuilder::<T>::default().generate_did().build("Bob");
        let venue_id = create_venue_::<T>(alice.did(), vec![]);

        let instruction_legs: Vec<(LegId, LegV2)> = (0..n)
            .map(|i| {
                let ticker = Ticker::from_slice_truncated(format!("TICKER{}", i).as_bytes());
                create_collection_issue_nfts::<T>(
                    alice.origin().into(),
                    ticker,
                    Some(NonFungibleType::Derivative),
                    0,
                    1,
                    PortfolioKind::Default,
                );
                Module::<T>::set_venue_filtering(alice.origin().into(), ticker, true).unwrap();
                Module::<T>::allow_venues(alice.origin().into(), ticker, vec![venue_id]).unwrap();
                (
                    LegId(i.into()),
                    LegV2 {
                        from: PortfolioId {
                            did: alice.did(),
                            kind: PortfolioKind::Default,
                        },
                        to: PortfolioId {
                            did: bob.did(),
                            kind: PortfolioKind::Default,
                        },
                        asset: LegAsset::NonFungible(NFTs::new_unverified(ticker, vec![NFTId(1)])),
                    },
                )
            })
            .collect();
    }: {
        Module::<T>::ensure_allowed_venue(&instruction_legs, venue_id).unwrap();
    }

    execute_instruction_initial_checks {
        // Number of legs in the instruction
        let n in 1..T::MaxNumberOfFungibleAssets::get() + T::MaxNumberOfNFTs::get();

        let alice = UserBuilder::<T>::default().generate_did().build("Alice");
        let bob = UserBuilder::<T>::default().generate_did().build("Bob");
        let ticker: Ticker = Ticker::from_slice_truncated(b"TICKER".as_ref());
        let instruction_id = InstructionId(1);

        (0..n)
            .for_each(|i| {
                let leg = LegV2 {
                    from: PortfolioId {
                        did: alice.did(),
                        kind: PortfolioKind::Default,
                    },
                    to: PortfolioId {
                        did: bob.did(),
                        kind: PortfolioKind::Default,
                    },
                    asset: LegAsset::NonFungible(NFTs::new_unverified(ticker, vec![NFTId(1)])),
                };
                InstructionLegsV2::insert(instruction_id, LegId(i.into()), leg);
            });
        InstructionStatuses::<T>::insert(instruction_id, InstructionStatus::Pending);
    }: {
        assert!(Module::<T>::instruction_affirms_pending(instruction_id) == 0);
        assert!(Module::<T>::instruction_status(instruction_id) == InstructionStatus::Pending);
        let venue_id = Module::<T>::instruction_details(instruction_id).venue_id;
        let mut instruction_legs: Vec<(LegId, LegV2)> =
            InstructionLegsV2::iter_prefix(&InstructionId(1)).collect();
        instruction_legs.sort_by_key(|leg_id_leg| leg_id_leg.0);
    }

    unchecked_release_locks {
        // Number of fungible and non fungible assets in the legs
        let f in 0..T::MaxNumberOfFungibleAssets::get();
        let n in 0..T::MaxNumberOfNFTs::get();

        let parameters = setup_v2_extrinsics_parameters::<T>(f, n);
        Module::<T>::add_and_affirm_instruction_with_memo_v2(
            parameters.sender.clone().origin.into(),
            parameters.venue_id,
            parameters.settlement_type,
            parameters.date,
            parameters.date,
            parameters.legs_v2.clone(),
            parameters.sender_portfolios.clone(),
            parameters.memo,
            None
        ).expect("failed to add instruction");
        let instruction_legs: Vec<(LegId, LegV2)> =
            InstructionLegsV2::iter_prefix(&InstructionId(1)).collect();
    }: {
        Module::<T>::unchecked_release_locks(InstructionId(1), &instruction_legs);
    }

    prune_instruction {
        // Number of legs and unique parties in the instruction
        let l in 1..T::MaxNumberOfFungibleAssets::get() + T::MaxNumberOfNFTs::get();
        let p in 1..T::MaxNumberOfFungibleAssets::get() + T::MaxNumberOfNFTs::get();

        let alice = UserBuilder::<T>::default().generate_did().build("Alice");
        let alice_portfolio = PortfolioId {
            did: alice.did(),
            kind: PortfolioKind::Default,
        };
        let ticker: Ticker = Ticker::from_slice_truncated(b"TICKER".as_ref());
        let instruction_id = InstructionId(1);

        for i in 0..l {
            let sender_portfolio = {
                // Controls the number of unique portfolios
                if i + 1 < p {
                    PortfolioId {
                        did: IdentityId::from(i as u128),
                        kind: PortfolioKind::Default,
                    }
                } else {
                    alice_portfolio
                }
            };
            let leg = LegV2 {
                from: sender_portfolio,
                to: alice_portfolio,
                asset: LegAsset::NonFungible(NFTs::new_unverified(ticker, vec![NFTId(1)])),
            };
            InstructionLegsV2::insert(instruction_id, LegId(i.into()), leg);
            InstructionLegStatus::<T>::insert(instruction_id, LegId(i.into()), LegStatus::ExecutionPending);
            AffirmsReceived::insert(
                instruction_id,
                sender_portfolio,
                AffirmationStatus::Affirmed,
            );
            UserAffirmations::insert(
                sender_portfolio,
                instruction_id,
                AffirmationStatus::Affirmed,
            )
        }
    }: {
        Module::<T>::prune_instruction(InstructionId(1), true)
    }

    post_failed_execution {
        let instruction_id = InstructionId(1);
        <InstructionDetails<T>>::insert(instruction_id, Instruction::default());
    }: {
        InstructionStatuses::<T>::insert(instruction_id, InstructionStatus::Failed);
    }

    execute_instruction_paused {
        // Number of assets and nfts in the instruction
        let f in 1..T::MaxNumberOfFungibleAssets::get() as u32;
        let n in 0..T::MaxNumberOfNFTs::get() as u32;

        let alice = UserBuilder::<T>::default().generate_did().build("Alice");
        let bob = UserBuilder::<T>::default().generate_did().build("Bob");

        setup_execute_instruction::<T>(&alice, &bob, SettlementType::SettleOnAffirmation, f, n, true, true);
    }: execute_scheduled_instruction_v3(RawOrigin::Root, InstructionId(1), f, n, None)

    execute_scheduled_instruction {
        // Number of assets and nfts in the instruction
        let f in 1..T::MaxNumberOfFungibleAssets::get() as u32;
        let n in 0..T::MaxNumberOfNFTs::get() as u32;

        let alice = UserBuilder::<T>::default().generate_did().build("Alice");
        let bob = UserBuilder::<T>::default().generate_did().build("Bob");

        setup_execute_instruction::<T>(&alice, &bob, SettlementType::SettleOnAffirmation, f, n, false, false);
    }: execute_scheduled_instruction_v3(RawOrigin::Root, InstructionId(1), f, n, None)
}

pub fn next_block<T: Config + pallet_scheduler::Config>() {
    use frame_support::traits::OnInitialize;
    let block_number = frame_system::Pallet::<T>::block_number() + 1u32.into();
    frame_system::Pallet::<T>::set_block_number(block_number);
    pallet_scheduler::Pallet::<T>::on_initialize(block_number);
}
