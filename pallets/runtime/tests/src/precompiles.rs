use std::collections::BTreeSet;

use codec::Encode;
use pallet_precompiles::interface::nft::{
    ERC721_INTERFACE_ID, ERC721_METADATA_INTERFACE_ID, ERC7943_NON_FUNGIBLE_INTERFACE_ID,
};
use pallet_precompiles::interface::{
    ERC165_INTERFACE_ID, ERC20_INTERFACE_ID, ERC7943_FUNGIBLE_INTERFACE_ID,
};
use pallet_revive::precompiles::alloy::sol_types::{SolCall, SolEvent};
use polymesh_precompiles::{IFungibleAsset, INonFungibleAsset};
use sp_core::hashing::keccak_256;

/// An ERC-165 interface id is the XOR of the selectors of the interface's functions.
fn xor(selectors: &[[u8; 4]]) -> [u8; 4] {
    selectors.iter().fold([0u8; 4], |mut acc, s| {
        for i in 0..4 {
            acc[i] ^= s[i];
        }
        acc
    })
}

/// `supportsInterface` must only claim interfaces the precompile really implements, with the
/// exact standard signatures.
///
/// Recomputing each id from our own generated selectors catches accidental signature drift in
/// `NonFungibleAssetStub.sol` — for example renaming a parameter type — which would otherwise
/// leave the precompile advertising ERC-721 compliance it no longer has.
#[test]
fn erc165_interface_ids_match_our_selectors() {
    assert_eq!(
        xor(&[INonFungibleAsset::supportsInterfaceCall::SELECTOR]),
        ERC165_INTERFACE_ID,
        "IERC165 interface id mismatch"
    );

    assert_eq!(
        xor(&[
            INonFungibleAsset::balanceOfCall::SELECTOR,
            INonFungibleAsset::ownerOfCall::SELECTOR,
            INonFungibleAsset::safeTransferFrom_0Call::SELECTOR,
            INonFungibleAsset::safeTransferFrom_1Call::SELECTOR,
            INonFungibleAsset::transferFromCall::SELECTOR,
            INonFungibleAsset::approveCall::SELECTOR,
            INonFungibleAsset::setApprovalForAllCall::SELECTOR,
            INonFungibleAsset::getApprovedCall::SELECTOR,
            INonFungibleAsset::isApprovedForAllCall::SELECTOR,
        ]),
        ERC721_INTERFACE_ID,
        "IERC721 interface id mismatch"
    );

    assert_eq!(
        xor(&[
            INonFungibleAsset::nameCall::SELECTOR,
            INonFungibleAsset::symbolCall::SELECTOR,
            INonFungibleAsset::tokenURICall::SELECTOR,
        ]),
        ERC721_METADATA_INTERFACE_ID,
        "IERC721Metadata interface id mismatch"
    );

    assert_eq!(
        xor(&[
            INonFungibleAsset::forcedTransferCall::SELECTOR,
            INonFungibleAsset::setFrozenTokensCall::SELECTOR,
            INonFungibleAsset::canSendCall::SELECTOR,
            INonFungibleAsset::canReceiveCall::SELECTOR,
            INonFungibleAsset::getFrozenTokensCall::SELECTOR,
            INonFungibleAsset::canTransferCall::SELECTOR,
        ]),
        ERC7943_NON_FUNGIBLE_INTERFACE_ID,
        "IERC7943NonFungible interface id mismatch"
    );
}

/// Same check for the fungible precompile, which reports `IERC165`, `IERC20` and
/// `IERC7943Fungible`.
#[test]
fn erc7943_fungible_interface_ids_match_our_selectors() {
    assert_eq!(
        xor(&[IFungibleAsset::supportsInterfaceCall::SELECTOR]),
        ERC165_INTERFACE_ID,
        "IERC165 interface id mismatch"
    );

    assert_eq!(
        xor(&[
            IFungibleAsset::totalSupplyCall::SELECTOR,
            IFungibleAsset::balanceOfCall::SELECTOR,
            IFungibleAsset::transferCall::SELECTOR,
            IFungibleAsset::allowanceCall::SELECTOR,
            IFungibleAsset::approveCall::SELECTOR,
            IFungibleAsset::transferFromCall::SELECTOR,
        ]),
        ERC20_INTERFACE_ID,
        "IERC20 interface id mismatch"
    );

    assert_eq!(
        xor(&[
            IFungibleAsset::forcedTransferCall::SELECTOR,
            IFungibleAsset::setFrozenTokensCall::SELECTOR,
            IFungibleAsset::canSendCall::SELECTOR,
            IFungibleAsset::canReceiveCall::SELECTOR,
            IFungibleAsset::getFrozenTokensCall::SELECTOR,
            IFungibleAsset::canTransferCall::SELECTOR,
        ]),
        ERC7943_FUNGIBLE_INTERFACE_ID,
        "IERC7943Fungible interface id mismatch"
    );
}

/// The first four bytes of the keccak-256 hash of a canonical function signature.
fn selector(signature: &str) -> [u8; 4] {
    keccak_256(signature.as_bytes())[..4].try_into().unwrap()
}

/// Checks `IFungibleAsset` against the canonical `IERC7943Fungible` of the published EIP
/// (<https://eips.ethereum.org/EIPS/eip-7943>, status Final).
///
/// `erc7943_fungible_interface_ids_match_our_selectors` only shows that our constant matches
/// our own stub. It cannot catch the stub drifting from the standard, for example a renamed
/// function or a changed parameter type, and the precompile would then claim an interface it
/// does not implement. These signatures are copied from the EIP rather than generated from our
/// stub, so a drift on either side fails here.
#[test]
fn erc7943_fungible_matches_the_published_standard() {
    // (canonical signature, our generated selector)
    let functions = [
        (
            "forcedTransfer(address,address,uint256)",
            IFungibleAsset::forcedTransferCall::SELECTOR,
        ),
        (
            "setFrozenTokens(address,uint256)",
            IFungibleAsset::setFrozenTokensCall::SELECTOR,
        ),
        ("canSend(address)", IFungibleAsset::canSendCall::SELECTOR),
        (
            "canReceive(address)",
            IFungibleAsset::canReceiveCall::SELECTOR,
        ),
        (
            "getFrozenTokens(address)",
            IFungibleAsset::getFrozenTokensCall::SELECTOR,
        ),
        (
            "canTransfer(address,address,uint256)",
            IFungibleAsset::canTransferCall::SELECTOR,
        ),
    ];
    for (signature, ours) in functions {
        assert_eq!(selector(signature), ours, "{signature} selector mismatch");
    }

    // The EIP states this id; recomputing it confirms the signature list above is complete.
    let canonical: Vec<[u8; 4]> = functions.iter().map(|(sig, _)| selector(sig)).collect();
    assert_eq!(xor(&canonical), [0x3e, 0xdb, 0xb4, 0xc4]);
    assert_eq!(ERC7943_FUNGIBLE_INTERFACE_ID, [0x3e, 0xdb, 0xb4, 0xc4]);

    // Indexers match on the event topic, so the event signatures must be exact too.
    assert_eq!(
        IFungibleAsset::ForcedTransfer::SIGNATURE_HASH.0,
        keccak_256(b"ForcedTransfer(address,address,uint256)"),
        "ForcedTransfer event signature mismatch"
    );
    assert_eq!(
        IFungibleAsset::Frozen::SIGNATURE_HASH.0,
        keccak_256(b"Frozen(address,uint256)"),
        "Frozen event signature mismatch"
    );
}

/// The two Polymesh precompiles must not share an address-matcher prefix, or one would shadow
/// the other for every asset id.
#[test]
fn precompile_matchers_are_distinct() {
    use pallet_revive::precompiles::{AddressMatcher, Precompile};

    fn var_prefix_id<P: Precompile>() -> u16 {
        match P::MATCHER {
            AddressMatcher::VarPrefix { id, data_bytes } => {
                assert_eq!(data_bytes, 16, "asset id occupies 16 bytes");
                id.get()
            }
            _ => panic!("expected a VarPrefix matcher"),
        }
    }

    let fungible =
        var_prefix_id::<pallet_precompiles::FungibleAssetInterface<crate::TestStorage>>();
    let non_fungible =
        var_prefix_id::<pallet_precompiles::NonFungibleAssetInterface<crate::TestStorage>>();

    assert_ne!(fungible, non_fungible);
    assert_eq!(fungible, 8);
    assert_eq!(non_fungible, 9);
}

// ==================== In-process precompile calls ====================

use frame_support::assert_ok;
use frame_support::weights::Weight;
use pallet_revive::precompiles::alloy::primitives::{Address, U256};
use pallet_revive::precompiles::alloy::sol_types::{Revert, SolError};
use pallet_revive::{AddressMapper, ExecConfig, TransactionLimits, H160};
use polymesh_primitives::asset::{AssetId, AssetType, NonFungibleType};
use sp_core::H256;
use sp_keyring::Sr25519Keyring;

use pallet_asset::Allowances;
use polymesh_primitives::asset::AssetHolder;
use polymesh_primitives::constants::currency::ONE_UNIT;
use polymesh_primitives::settlement::AffirmationRequirement;
use polymesh_primitives::{
    AssetHolderKind, Balance, Claim, Condition, ConditionType, CountryCode, Scope, WeightMeter,
};
use rand::rngs::StdRng;
use rand::{Rng, SeedableRng};

use crate::asset_pallet::setup::ISSUE_AMOUNT;
use crate::ext_builder::ExtBuilder;
use crate::nft::{create_nft_collection, mint_nft};
use crate::storage::{add_secondary_key, root, EventTest, TestStorage, User};

type Asset = pallet_asset::Pallet<TestStorage>;
type ComplianceManager = pallet_compliance_manager::Pallet<TestStorage>;
type Revive = pallet_revive::Pallet<TestStorage>;
type Settlement = pallet_settlement::Pallet<TestStorage>;
type System = frame_system::Pallet<TestStorage>;

/// The address of the precompile registered under `prefix_id` for `asset_id`.
fn precompile_address(asset_id: AssetId, prefix_id: u8) -> H160 {
    let mut address = [0u8; 20];
    address[..16].copy_from_slice(&asset_id.to_bytes());
    address[17] = prefix_id;
    H160::from(address)
}

/// The fungible precompile address of `asset_id`.
fn fungible(asset_id: AssetId) -> H160 {
    precompile_address(asset_id, 8)
}

/// The non-fungible precompile address of `asset_id`.
fn non_fungible(asset_id: AssetId) -> H160 {
    precompile_address(asset_id, 9)
}

/// Registers `user`'s account with the address mapper and returns its ethereum address.
///
/// Without the mapping, the address resolves back to its `0xEE` fallback account, which has
/// no identity.
fn eth_address(user: &User) -> Address {
    let account = user.acc();
    if !<TestStorage as pallet_revive::Config>::AddressMapper::is_mapped(&account) {
        assert_ok!(Revive::map_account(user.origin()));
    }
    let address = <TestStorage as pallet_revive::Config>::AddressMapper::to_address(&account);
    Address::from(address.0)
}

/// A log emitted by a precompile: its topics and data.
type Log = (Vec<H256>, Vec<u8>);

/// Calls `call` on the precompile at `to` as `caller`.
///
/// Returns the decoded return value and the logs the call emitted, or the revert reason.
fn call<C: SolCall>(caller: &User, to: H160, call: C) -> Result<(C::Return, Vec<Log>), String> {
    eth_address(caller);
    let events_before = System::events().len();

    let outcome = Revive::bare_call(
        caller.origin(),
        to,
        Default::default(),
        TransactionLimits::WeightAndDeposit {
            weight_limit: Weight::from_parts(500_000_000_000, 10_000_000),
            deposit_limit: 0,
        },
        call.abi_encode(),
        &ExecConfig::new_substrate_tx(),
    );

    let returned = outcome.result.map_err(|err| format!("{err:?}"))?;
    if returned.did_revert() {
        return Err(match Revert::abi_decode(&returned.data) {
            Ok(revert) => revert.reason,
            Err(_) => format!("raw revert data: {:?}", returned.data),
        });
    }

    let logs = System::events()
        .into_iter()
        .skip(events_before)
        .filter_map(|record| match record.event {
            EventTest::Revive(pallet_revive::Event::ContractEmitted {
                contract,
                data,
                topics,
            }) if contract == to => Some((topics, data)),
            _ => None,
        })
        .collect();

    let value = C::abi_decode_returns(&returned.data).map_err(|err| format!("{err:?}"))?;
    Ok((value, logs))
}

/// Calls `c` on the precompile at `to` as `caller` and returns the logs it emitted, or the
/// revert reason.
fn send<C: SolCall>(caller: &User, to: H160, c: C) -> Result<Vec<Log>, String> {
    call(caller, to, c).map(|(_, logs)| logs)
}

/// Calls a view on the precompile at `to` and returns its value.
fn view<C: SolCall>(caller: &User, to: H160, c: C) -> C::Return {
    call(caller, to, c).expect("the view must not revert").0
}

/// Runs `test` on a fresh chain, at a block where events are recorded.
fn run(test: impl FnOnce()) {
    ExtBuilder::default().build().execute_with(|| {
        System::set_block_number(1);
        test()
    });
}

/// An asset owned by Alice, with [`ISSUE_AMOUNT`] tokens in her account key.
fn fungible_asset(owner: &User) -> AssetId {
    let asset_id = Asset::generate_asset_id(owner.acc(), false);
    assert_ok!(Asset::create_asset(
        owner.origin(),
        b"MyAsset".into(),
        true,
        Default::default(),
        Vec::new(),
        None,
    ));
    assert_ok!(Asset::issue(
        owner.origin(),
        asset_id,
        ISSUE_AMOUNT,
        AssetHolderKind::Account
    ));
    asset_id
}

/// A second account key of `user`'s identity, funded so it can be mapped.
fn secondary_key(user: &User, ring: Sr25519Keyring) -> User {
    let key = User::new_with(user.did, ring).balance(10_000_000);
    add_secondary_key(user.did, key.acc());
    key
}

/// `user`'s account key as an asset holder.
fn holder(user: &User) -> AssetHolder {
    AssetHolder::try_from(user.acc().encode()).unwrap()
}

/// `canTransfer(from, to, value)` on the fungible precompile of `asset_id`.
fn can_transfer(asset_id: AssetId, from: &User, to: &User, value: Balance) -> bool {
    let call = IFungibleAsset::canTransferCall {
        from: eth_address(from),
        to: eth_address(to),
        value: U256::from(value),
    };
    view(from, fungible(asset_id), call)
}

/// `transfer(to, value)` on the fungible precompile of `asset_id`, sent by `from`.
fn transfer(asset_id: AssetId, from: &User, to: &User, value: Balance) -> Result<Vec<Log>, String> {
    let call = IFungibleAsset::transferCall {
        to: eth_address(to),
        value: U256::from(value),
    };
    send(from, fungible(asset_id), call)
}

/// Adds a compliance requirement to `asset_id` that no receiver meets.
fn fail_compliance(owner: &User, asset_id: AssetId) {
    assert_ok!(ComplianceManager::add_compliance_requirement(
        owner.origin(),
        asset_id,
        Vec::new(),
        vec![Condition {
            condition_type: ConditionType::IsPresent(Claim::Jurisdiction(
                CountryCode::BR,
                Scope::Identity(owner.did),
            )),
            issuers: vec![owner.issuer()],
        }],
    ));
}

/// The revert reason of a transfer left waiting for an affirmation.
const NOT_EXECUTED: &str = "Instruction was not executed; Most likely the instruction is missing an affirmation from the receiver/mediator";

/// Makes `mediator` a mandatory mediator of `asset_id`.
fn add_mediator(owner: &User, asset_id: AssetId, mediator: &User) {
    let mediators = BTreeSet::from([mediator.did]);
    assert_ok!(Asset::add_mandatory_mediators(
        owner.origin(),
        asset_id,
        mediators.try_into().unwrap()
    ));
}

/// Makes `user`'s identity affirm every incoming transfer itself.
fn require_receiver_affirmation(user: &User) {
    assert_ok!(Settlement::set_mandatory_receiver_affirmation(
        user.origin(),
        AffirmationRequirement::Required
    ));
}

/// `canReceive(account)` on the fungible precompile of `asset_id`.
fn can_receive(asset_id: AssetId, account: &User) -> bool {
    let call = IFungibleAsset::canReceiveCall {
        account: eth_address(account),
    };
    view(account, fungible(asset_id), call)
}

/// `canTransfer(from, to, tokenId)` on the NFT precompile of `asset_id`.
fn nft_can_transfer(asset_id: AssetId, from: &User, to: &User, token_id: u64) -> bool {
    let call = INonFungibleAsset::canTransferCall {
        from: eth_address(from),
        to: eth_address(to),
        tokenId: U256::from(token_id),
    };
    view(from, non_fungible(asset_id), call)
}

/// `canReceive(account)` on the NFT precompile of `asset_id`.
fn nft_can_receive(asset_id: AssetId, account: &User) -> bool {
    let call = INonFungibleAsset::canReceiveCall {
        account: eth_address(account),
    };
    view(account, non_fungible(asset_id), call)
}

/// `transferFrom(from, to, tokenId)` on the NFT precompile of `asset_id`, sent by `from`.
fn nft_transfer(
    asset_id: AssetId,
    from: &User,
    to: &User,
    token_id: u64,
) -> Result<Vec<Log>, String> {
    let call = INonFungibleAsset::transferFromCall {
        from: eth_address(from),
        to: eth_address(to),
        tokenId: U256::from(token_id),
    };
    send(from, non_fungible(asset_id), call)
}

/// A collection owned by `owner`, with NFTs 1 to `n` in the owner's account key.
fn nft_collection(owner: &User, n: u64) -> AssetId {
    let asset_id = create_nft_collection(
        *owner,
        AssetType::NonFungible(NonFungibleType::Derivative),
        Vec::new().into(),
    );
    for _ in 0..n {
        mint_nft(*owner, asset_id, Vec::new(), AssetHolderKind::Account);
    }
    asset_id
}

/// `approve(spender, type(uint256).max)` sets an infinite allowance (X3a).
#[test]
fn approve_uint256_max_is_an_infinite_allowance() {
    run(|| {
        let alice = User::new(Sr25519Keyring::Alice);
        let bob = User::new(Sr25519Keyring::Bob);
        let asset_id = fungible_asset(&alice);
        let token = fungible(asset_id);
        let (owner, spender) = (eth_address(&alice), eth_address(&bob));

        let (approved, logs) = call(
            &alice,
            token,
            IFungibleAsset::approveCall {
                spender,
                value: U256::MAX,
            },
        )
        .unwrap();
        assert!(approved);

        // `Approval` carries the value the caller passed, not the stored one
        assert_eq!(logs.len(), 1);
        let approval = IFungibleAsset::Approval::decode_raw_log(
            logs[0].0.iter().map(|topic| topic.0),
            &logs[0].1,
        )
        .unwrap();
        assert_eq!(approval.owner, owner);
        assert_eq!(approval.spender, spender);
        assert_eq!(approval.value, U256::MAX);

        // Stored as the pallet's infinite allowance, which a spend does not reduce
        assert_eq!(
            Allowances::<TestStorage>::get((alice.acc(), bob.acc(), asset_id)),
            Balance::MAX
        );
        // `allowance()` still reports it as 2^128-1 (X3b changes that)
        let allowance = IFungibleAsset::allowanceCall { owner, spender };
        assert_eq!(view(&bob, token, allowance.clone()), U256::from(u128::MAX));

        let spend = IFungibleAsset::transferFromCall {
            from: owner,
            to: spender,
            value: U256::from(1_000_000u64),
        };
        assert_ok!(send(&bob, token, spend));
        assert_eq!(view(&bob, token, allowance), U256::from(u128::MAX));
    });
}

/// Any other value above 2^128-1 still reverts.
#[test]
fn approve_other_values_above_u128_max_revert() {
    run(|| {
        let alice = User::new(Sr25519Keyring::Alice);
        let bob = User::new(Sr25519Keyring::Bob);
        let asset_id = fungible_asset(&alice);
        let spender = eth_address(&bob);

        for value in [
            U256::from(u128::MAX) + U256::from(1u8),
            U256::MAX - U256::from(1u8),
        ] {
            let approve = IFungibleAsset::approveCall { spender, value };
            assert_eq!(
                call(&alice, fungible(asset_id), approve).unwrap_err(),
                "Balance conversion failed"
            );
        }
        assert_eq!(
            Allowances::<TestStorage>::get((alice.acc(), bob.acc(), asset_id)),
            0
        );
    });
}

/// `isDivisible()` reports the asset's divisibility, and follows `make_divisible` (F2).
#[test]
fn is_divisible_reports_the_assets_divisibility() {
    run(|| {
        let alice = User::new(Sr25519Keyring::Alice);
        let divisible = fungible_asset(&alice);
        assert!(view(
            &alice,
            fungible(divisible),
            IFungibleAsset::isDivisibleCall {}
        ));

        let indivisible = Asset::generate_asset_id(alice.acc(), false);
        assert_ok!(Asset::create_asset(
            alice.origin(),
            b"Whole".into(),
            false,
            Default::default(),
            Vec::new(),
            None,
        ));
        let token = fungible(indivisible);
        assert!(!view(&alice, token, IFungibleAsset::isDivisibleCall {}));
        // `decimals()` does not depend on divisibility
        assert_eq!(view(&alice, token, IFungibleAsset::decimalsCall {}), 6);

        assert_ok!(Asset::make_divisible(alice.origin(), indivisible));
        assert!(view(&alice, token, IFungibleAsset::isDivisibleCall {}));
        assert_eq!(view(&alice, token, IFungibleAsset::decimalsCall {}), 6);
    });
}

/// `getApproved` reverts for a token that does not exist, as ERC-721 requires (N3).
#[test]
fn get_approved_reverts_for_a_token_that_does_not_exist() {
    run(|| {
        let alice = User::new(Sr25519Keyring::Alice);
        let bob = User::new(Sr25519Keyring::Bob);
        let asset_id = nft_collection(&alice, 2);
        let collection = non_fungible(asset_id);
        let get_approved = |id: u64| INonFungibleAsset::getApprovedCall {
            tokenId: U256::from(id),
        };

        // An existing token with no approval still returns the zero address
        assert_eq!(view(&alice, collection, get_approved(1)), Address::ZERO);

        let approved = eth_address(&bob);
        assert_ok!(send(
            &alice,
            collection,
            INonFungibleAsset::approveCall {
                to: approved,
                tokenId: U256::from(1u8),
            },
        ));
        assert_eq!(view(&alice, collection, get_approved(1)), approved);

        // Never minted
        assert_eq!(
            call(&alice, collection, get_approved(3)).unwrap_err(),
            "NFT does not exist"
        );

        // Redeemed
        assert_ok!(send(
            &alice,
            collection,
            INonFungibleAsset::burnCall {
                tokenId: U256::from(2u8),
            },
        ));
        assert_eq!(
            call(&alice, collection, get_approved(2)).unwrap_err(),
            "NFT does not exist"
        );
    });
}

/// `balanceOf(address(0))` reverts, as ERC-721 requires (N4).
#[test]
fn nft_balance_of_the_zero_address_reverts() {
    run(|| {
        let alice = User::new(Sr25519Keyring::Alice);
        let asset_id = nft_collection(&alice, 2);
        let collection = non_fungible(asset_id);
        let balance_of = |owner: Address| INonFungibleAsset::balanceOfCall { owner };

        assert_eq!(
            view(&alice, collection, balance_of(eth_address(&alice))),
            U256::from(2u8)
        );
        assert_eq!(
            call(&alice, collection, balance_of(Address::ZERO)).unwrap_err(),
            "The zero address is not a valid owner"
        );
    });
}

/// `canTransfer` returns `false` for a value no balance can hold, where it used to revert (R1).
#[test]
fn fungible_can_transfer_is_false_for_an_out_of_range_value() {
    run(|| {
        let alice = User::new(Sr25519Keyring::Alice);
        let bob = User::new(Sr25519Keyring::Bob);
        let asset_id = fungible_asset(&alice);
        assert_ok!(ComplianceManager::pause_asset_compliance(
            alice.origin(),
            asset_id
        ));
        let can_transfer = |value: U256| IFungibleAsset::canTransferCall {
            from: eth_address(&alice),
            to: eth_address(&bob),
            value,
        };
        let token = fungible(asset_id);

        assert!(view(&alice, token, can_transfer(U256::from(1_000_000u64))));
        assert!(!view(
            &alice,
            token,
            can_transfer(U256::from(u128::MAX) + U256::from(1u8))
        ));
        assert!(!view(&alice, token, can_transfer(U256::MAX)));
    });
}

/// `canTransfer` returns `false` for a token id no NFT can have, where it used to revert (R1).
#[test]
fn nft_can_transfer_is_false_for_an_out_of_range_token_id() {
    run(|| {
        let alice = User::new(Sr25519Keyring::Alice);
        let bob = User::new(Sr25519Keyring::Bob);
        let asset_id = nft_collection(&alice, 1);
        assert_ok!(ComplianceManager::pause_asset_compliance(
            alice.origin(),
            asset_id
        ));
        let can_transfer = |token_id: U256| INonFungibleAsset::canTransferCall {
            from: eth_address(&alice),
            to: eth_address(&bob),
            tokenId: token_id,
        };
        let collection = non_fungible(asset_id);

        assert!(view(&alice, collection, can_transfer(U256::from(1u8))));
        assert!(!view(
            &alice,
            collection,
            can_transfer(U256::from(u64::MAX) + U256::from(1u8))
        ));
        assert!(!view(&alice, collection, can_transfer(U256::MAX)));
    });
}

/// A transfer between two keys of one identity skips compliance, so `canTransfer` must too (R2).
#[test]
fn fungible_can_transfer_agrees_with_a_same_identity_transfer() {
    run(|| {
        let alice = User::new(Sr25519Keyring::Alice);
        let alice_2 = secondary_key(&alice, Sr25519Keyring::Charlie);
        let bob = User::new(Sr25519Keyring::Bob);
        let asset_id = fungible_asset(&alice);
        fail_compliance(&alice, asset_id);

        assert!(!can_transfer(asset_id, &alice, &bob, ONE_UNIT));
        assert_eq!(
            transfer(asset_id, &alice, &bob, ONE_UNIT).unwrap_err(),
            "Extrinsic returned an error: FailedAssetTransferringConditions"
        );

        assert!(can_transfer(asset_id, &alice, &alice_2, ONE_UNIT));
        assert_ok!(transfer(asset_id, &alice, &alice_2, ONE_UNIT));
        assert_eq!(
            Asset::get_holders_balance(&holder(&alice_2), &asset_id),
            ONE_UNIT
        );
    });
}

/// The same-identity path still applies its own checks.
#[test]
fn fungible_can_transfer_applies_the_same_identity_checks() {
    run(|| {
        let alice = User::new(Sr25519Keyring::Alice);
        let alice_2 = secondary_key(&alice, Sr25519Keyring::Charlie);
        let asset_id = fungible_asset(&alice);
        let mut meter = WeightMeter::max_limit_no_minimum();
        let mut can = |from: &User, to: &User, value| {
            let pallet =
                Asset::can_transfer_funds(&holder(from), &holder(to), &asset_id, value, &mut meter);
            // The precompile reports what the pallet helper does
            assert_eq!(pallet, can_transfer(asset_id, from, to, value));
            // and a transfer never succeeds when it says no
            if !pallet {
                assert!(transfer(asset_id, from, to, value).is_err());
            }
            pallet
        };

        assert!(can(&alice, &alice_2, ISSUE_AMOUNT));
        // More than the balance
        assert!(!can(&alice, &alice_2, ISSUE_AMOUNT + 1));
        // The same holder on both sides
        assert!(!can(&alice, &alice, ONE_UNIT));

        // Frozen tokens are not available
        assert_ok!(Asset::set_frozen_tokens(
            alice.origin(),
            asset_id,
            holder(&alice),
            ISSUE_AMOUNT - ONE_UNIT,
        ));
        assert!(can(&alice, &alice_2, ONE_UNIT));
        assert!(!can(&alice, &alice_2, ONE_UNIT + 1));
        assert_ok!(Asset::set_frozen_tokens(
            alice.origin(),
            asset_id,
            holder(&alice),
            0
        ));

        // A frozen sender
        assert_ok!(Asset::set_holder_frozen(
            alice.origin(),
            holder(&alice),
            asset_id,
            true
        ));
        assert!(!can(&alice, &alice_2, ONE_UNIT));
        assert_ok!(Asset::set_holder_frozen(
            alice.origin(),
            holder(&alice),
            asset_id,
            false
        ));

        // A frozen asset
        assert_ok!(Asset::freeze(alice.origin(), asset_id));
        assert!(!can(&alice, &alice_2, ONE_UNIT));
        assert_ok!(Asset::unfreeze(alice.origin(), asset_id));
        assert!(can(&alice, &alice_2, ONE_UNIT));
    });
}

/// An indivisible asset only moves in whole units, between keys of one identity too.
#[test]
fn fungible_can_transfer_checks_granularity_for_a_same_identity_transfer() {
    run(|| {
        let alice = User::new(Sr25519Keyring::Alice);
        let alice_2 = secondary_key(&alice, Sr25519Keyring::Charlie);
        let asset_id = Asset::generate_asset_id(alice.acc(), false);
        assert_ok!(Asset::create_asset(
            alice.origin(),
            b"Whole".into(),
            false,
            Default::default(),
            Vec::new(),
            None,
        ));
        assert_ok!(Asset::issue(
            alice.origin(),
            asset_id,
            10 * ONE_UNIT,
            AssetHolderKind::Account
        ));

        assert!(can_transfer(asset_id, &alice, &alice_2, ONE_UNIT));
        assert!(!can_transfer(asset_id, &alice, &alice_2, ONE_UNIT / 2));
        assert_eq!(
            transfer(asset_id, &alice, &alice_2, ONE_UNIT / 2).unwrap_err(),
            "Extrinsic returned an error: InvalidGranularity"
        );
    });
}

/// ERC-7943: a public transfer must not succeed when `canTransfer` is false.
///
/// Each round builds a random state, asks `canTransfer`, then sends the transfer. The state
/// covers the asset and holder freezes, frozen tokens, compliance, divisibility, mandatory
/// mediators, receiver affirmation with and without a pre-approval, and senders and receivers of
/// the same and of different identities.
#[test]
fn a_fungible_transfer_that_succeeds_implies_can_transfer() {
    let mut rng = StdRng::seed_from_u64(0x7943);
    let (mut succeeded, mut same_identity_succeeded, mut refused) = (0, 0, 0);

    for _ in 0..300 {
        run(|| {
            let alice = User::new(Sr25519Keyring::Alice);
            let alice_2 = secondary_key(&alice, Sr25519Keyring::Charlie);
            let bob = User::new(Sr25519Keyring::Bob);
            let bob_2 = secondary_key(&bob, Sr25519Keyring::Dave);
            let users = [alice, alice_2, bob, bob_2];

            let divisible = rng.gen_bool(0.7);
            let asset_id = Asset::generate_asset_id(alice.acc(), false);
            assert_ok!(Asset::create_asset(
                alice.origin(),
                b"Random".into(),
                divisible,
                Default::default(),
                Vec::new(),
                None,
            ));
            let supply = 100 * ONE_UNIT;
            assert_ok!(Asset::issue(
                alice.origin(),
                asset_id,
                supply,
                AssetHolderKind::Account
            ));

            if rng.gen_bool(0.4) {
                fail_compliance(&alice, asset_id);
            }
            if rng.gen_bool(0.3) {
                let frozen = rng.gen_range(0..=supply / ONE_UNIT) * ONE_UNIT;
                assert_ok!(Asset::set_frozen_tokens(
                    alice.origin(),
                    asset_id,
                    holder(&alice),
                    frozen,
                ));
            }
            if rng.gen_bool(0.15) {
                assert_ok!(Asset::set_holder_frozen(
                    alice.origin(),
                    holder(&alice),
                    asset_id,
                    true
                ));
            }
            if rng.gen_bool(0.15) {
                assert_ok!(Asset::freeze(alice.origin(), asset_id));
            }
            if rng.gen_bool(0.15) {
                add_mediator(&alice, asset_id, &bob);
            }
            if rng.gen_bool(0.25) {
                require_receiver_affirmation(&bob);
                if rng.gen_bool(0.5) {
                    assert_ok!(Asset::pre_approve_asset(bob.origin(), asset_id));
                }
            }

            // Alice holds everything, so she is the usual sender
            let from = if rng.gen_bool(0.9) {
                alice
            } else {
                users[rng.gen_range(0..users.len())]
            };
            let to = users[rng.gen_range(0..users.len())];
            let value = match rng.gen_range(0..6) {
                0 => 0,
                1 => 1,
                2 => ONE_UNIT / 2,
                3 => supply,
                4 => supply + ONE_UNIT,
                _ => rng.gen_range(1..=supply / ONE_UNIT) * ONE_UNIT,
            };

            let allowed = can_transfer(asset_id, &from, &to, value);
            let sent = transfer(asset_id, &from, &to, value).is_ok();
            assert!(
                allowed || !sent,
                "a transfer of {value} succeeded although canTransfer was false \
                 (same identity: {})",
                from.did == to.did
            );

            succeeded += sent as u32;
            same_identity_succeeded += (sent && from.did == to.did) as u32;
            refused += !allowed as u32;
        });
    }

    // The property is only worth something if both outcomes were exercised
    assert!(succeeded > 30, "only {succeeded} transfers succeeded");
    assert!(same_identity_succeeded > 10);
    assert!(refused > 30, "canTransfer was false only {refused} times");
}

/// A transfer between two identities of an asset with mandatory mediators never completes in
/// one call, so `canTransfer` is `false` (R4).
#[test]
fn can_transfer_is_false_when_the_asset_has_mandatory_mediators() {
    run(|| {
        let alice = User::new(Sr25519Keyring::Alice);
        let alice_2 = secondary_key(&alice, Sr25519Keyring::Charlie);
        let bob = User::new(Sr25519Keyring::Bob);
        let dave = User::new(Sr25519Keyring::Dave);

        let asset_id = fungible_asset(&alice);
        assert!(can_transfer(asset_id, &alice, &bob, ONE_UNIT));
        add_mediator(&alice, asset_id, &dave);
        assert!(!can_transfer(asset_id, &alice, &bob, ONE_UNIT));
        assert_eq!(
            transfer(asset_id, &alice, &bob, ONE_UNIT).unwrap_err(),
            NOT_EXECUTED
        );
        // Mediators are not a rule about the receiver
        assert!(can_receive(asset_id, &bob));
        // Two keys of one identity move funds without an instruction, so without mediators
        assert!(can_transfer(asset_id, &alice, &alice_2, ONE_UNIT));
        assert_ok!(transfer(asset_id, &alice, &alice_2, ONE_UNIT));

        let collection = nft_collection(&alice, 2);
        assert!(nft_can_transfer(collection, &alice, &bob, 1));
        add_mediator(&alice, collection, &dave);
        assert!(!nft_can_transfer(collection, &alice, &bob, 1));
        assert_eq!(
            nft_transfer(collection, &alice, &bob, 1).unwrap_err(),
            NOT_EXECUTED
        );
        assert!(nft_can_receive(collection, &bob));
        assert!(nft_can_transfer(collection, &alice, &alice_2, 1));
        assert_ok!(nft_transfer(collection, &alice, &alice_2, 1));
    });
}

/// A receiver that affirms its incoming transfers itself, and has not pre-approved the asset,
/// cannot be sent to in one call, so `canTransfer` and `canReceive` are `false` (R4).
#[test]
fn can_transfer_and_can_receive_are_false_when_the_receiver_must_affirm() {
    run(|| {
        let alice = User::new(Sr25519Keyring::Alice);
        let alice_2 = secondary_key(&alice, Sr25519Keyring::Charlie);
        let bob = User::new(Sr25519Keyring::Bob);
        let asset_id = fungible_asset(&alice);
        let collection = nft_collection(&alice, 3);

        assert!(can_transfer(asset_id, &alice, &bob, ONE_UNIT));
        assert!(can_receive(asset_id, &bob));
        assert!(nft_can_transfer(collection, &alice, &bob, 1));
        assert!(nft_can_receive(collection, &bob));

        require_receiver_affirmation(&bob);

        assert!(!can_transfer(asset_id, &alice, &bob, ONE_UNIT));
        assert!(!can_receive(asset_id, &bob));
        assert_eq!(
            transfer(asset_id, &alice, &bob, ONE_UNIT).unwrap_err(),
            NOT_EXECUTED
        );
        assert!(!nft_can_transfer(collection, &alice, &bob, 1));
        assert!(!nft_can_receive(collection, &bob));
        assert_eq!(
            nft_transfer(collection, &alice, &bob, 1).unwrap_err(),
            NOT_EXECUTED
        );

        // The sender's own setting is irrelevant, and so is the receiver's when the funds stay
        // with one identity
        require_receiver_affirmation(&alice);
        assert!(can_transfer(asset_id, &alice, &alice_2, ONE_UNIT));
        assert_ok!(transfer(asset_id, &alice, &alice_2, ONE_UNIT));
        assert!(nft_can_transfer(collection, &alice, &alice_2, 3));
        assert_ok!(nft_transfer(collection, &alice, &alice_2, 3));

        // A pre-approval by the receiver lifts it, for that asset only
        assert_ok!(Asset::pre_approve_asset(bob.origin(), asset_id));
        assert!(can_transfer(asset_id, &alice, &bob, ONE_UNIT));
        assert!(can_receive(asset_id, &bob));
        assert_ok!(transfer(asset_id, &alice, &bob, ONE_UNIT));
        assert!(!nft_can_transfer(collection, &alice, &bob, 1));
        assert!(!nft_can_receive(collection, &bob));

        // So does an exemption of the asset
        assert_ok!(Asset::exempt_asset_affirmation(root(), collection));
        assert!(nft_can_transfer(collection, &alice, &bob, 1));
        assert!(nft_can_receive(collection, &bob));
        assert_ok!(nft_transfer(collection, &alice, &bob, 1));
    });
}

/// The one transfer that completes although the receiver must affirm: the receiver pulling the
/// funds itself with `transferFrom`. Its call is its affirmation. `canTransfer` takes no caller,
/// so it still answers `false` for that pair.
#[test]
fn a_receiver_that_must_affirm_can_still_pull_with_transfer_from() {
    run(|| {
        let alice = User::new(Sr25519Keyring::Alice);
        let bob = User::new(Sr25519Keyring::Bob);
        let asset_id = fungible_asset(&alice);
        require_receiver_affirmation(&bob);

        assert_ok!(Asset::approve(
            alice.origin(),
            asset_id,
            bob.acc(),
            ONE_UNIT
        ));
        assert!(!can_transfer(asset_id, &alice, &bob, ONE_UNIT));

        let pull = IFungibleAsset::transferFromCall {
            from: eth_address(&alice),
            to: eth_address(&bob),
            value: U256::from(ONE_UNIT),
        };
        assert_ok!(send(&bob, fungible(asset_id), pull));
        assert_eq!(
            Asset::get_holders_balance(&holder(&bob), &asset_id),
            ONE_UNIT
        );
    });
}

/// `paused()` follows `pause`/`unpause`, and a freeze of the asset made natively (T1).
#[test]
fn paused_reports_whether_the_asset_is_frozen() {
    run(|| {
        let alice = User::new(Sr25519Keyring::Alice);
        let asset_id = fungible_asset(&alice);
        let token = fungible(asset_id);
        let paused = || view(&alice, token, IFungibleAsset::pausedCall {});

        assert!(!paused());
        assert_ok!(send(&alice, token, IFungibleAsset::pauseCall {}));
        assert!(paused());
        assert_ok!(send(&alice, token, IFungibleAsset::unpauseCall {}));
        assert!(!paused());

        assert_ok!(Asset::freeze(alice.origin(), asset_id));
        assert!(paused());
    });
}

/// `isFrozen(account)` follows `setAddressFrozen`, and only that freeze (T1).
#[test]
fn is_frozen_reports_the_account_freeze() {
    run(|| {
        let alice = User::new(Sr25519Keyring::Alice);
        let bob = User::new(Sr25519Keyring::Bob);
        let asset_id = fungible_asset(&alice);
        let token = fungible(asset_id);
        let account = eth_address(&bob);
        let is_frozen = |account| view(&alice, token, IFungibleAsset::isFrozenCall { account });
        let set_frozen = |freeze| IFungibleAsset::setAddressFrozenCall { account, freeze };

        assert!(!is_frozen(account));
        assert_ok!(send(&alice, token, set_frozen(true)));
        assert!(is_frozen(account));
        // Per account, and per asset
        assert!(!is_frozen(eth_address(&alice)));
        assert_ok!(send(&alice, token, set_frozen(false)));
        assert!(!is_frozen(account));

        // Frozen tokens and a paused asset are separate states
        assert_ok!(Asset::set_frozen_tokens(
            alice.origin(),
            asset_id,
            holder(&bob),
            ONE_UNIT
        ));
        assert_ok!(Asset::freeze(alice.origin(), asset_id));
        assert!(!is_frozen(account));
    });
}

/// The ERC-3643 events the fungible precompile emits have the signatures of the published
/// interface (`IERC3643.sol` of the EIP, the same as T-REX's `IToken`). Indexers match on the
/// signature hash, so it must be exact.
#[test]
fn erc3643_events_match_the_published_standard() {
    let events = [
        (
            "UpdatedTokenInformation(string,string,uint8,string,address)",
            IFungibleAsset::UpdatedTokenInformation::SIGNATURE_HASH,
        ),
        ("Paused(address)", IFungibleAsset::Paused::SIGNATURE_HASH),
        (
            "Unpaused(address)",
            IFungibleAsset::Unpaused::SIGNATURE_HASH,
        ),
        (
            "AddressFrozen(address,bool,address)",
            IFungibleAsset::AddressFrozen::SIGNATURE_HASH,
        ),
        (
            "TokensFrozen(address,uint256)",
            IFungibleAsset::TokensFrozen::SIGNATURE_HASH,
        ),
        (
            "TokensUnfrozen(address,uint256)",
            IFungibleAsset::TokensUnfrozen::SIGNATURE_HASH,
        ),
    ];
    for (signature, ours) in events {
        assert_eq!(keccak_256(signature.as_bytes()), ours.0, "{signature}");
    }
}

/// Checks that `log` is an `UpdatedTokenInformation` for `name` and `symbol`.
///
/// The name, the symbol and the ONCHAINID are indexed, as in the published interface, so the
/// strings are logged as their hashes.
fn assert_token_information(log: &Log, name: &str, symbol: &str) {
    let (topics, data) = log;
    assert_eq!(topics.len(), 4);
    assert_eq!(
        topics[0].0,
        IFungibleAsset::UpdatedTokenInformation::SIGNATURE_HASH.0
    );
    assert_eq!(topics[1].0, keccak_256(name.as_bytes()));
    assert_eq!(topics[2].0, keccak_256(symbol.as_bytes()));
    // No ONCHAINID
    assert_eq!(topics[3], H256::zero());

    let event = IFungibleAsset::UpdatedTokenInformation::decode_raw_log(
        topics.iter().map(|topic| topic.0),
        data,
    )
    .unwrap();
    assert_eq!(event.newDecimals, 6);
    assert_eq!(event.newVersion, "");
}

/// `setName` and `setSymbol` emit ERC-3643 `UpdatedTokenInformation` (T2).
#[test]
fn set_name_and_set_symbol_emit_updated_token_information() {
    run(|| {
        let alice = User::new(Sr25519Keyring::Alice);
        let asset_id = fungible_asset(&alice);
        let token = fungible(asset_id);
        let set_symbol = |symbol: &str| IFungibleAsset::setSymbolCall {
            symbol: symbol.into(),
        };

        // No symbol yet
        let logs = send(
            &alice,
            token,
            IFungibleAsset::setNameCall {
                name: "Renamed".into(),
            },
        )
        .unwrap();
        assert_eq!(logs.len(), 1);
        assert_token_information(&logs[0], "Renamed", "");

        let logs = send(&alice, token, set_symbol("FIRST")).unwrap();
        assert_eq!(logs.len(), 1);
        assert_token_information(&logs[0], "Renamed", "FIRST");

        // Replacing a linked ticker
        let logs = send(&alice, token, set_symbol("SECOND")).unwrap();
        assert_eq!(logs.len(), 1);
        assert_token_information(&logs[0], "Renamed", "SECOND");

        // Setting the current symbol again changes nothing, and still emits the event
        let logs = send(&alice, token, set_symbol("SECOND")).unwrap();
        assert_eq!(logs.len(), 1);
        assert_token_information(&logs[0], "Renamed", "SECOND");

        let logs = send(
            &alice,
            token,
            IFungibleAsset::setNameCall {
                name: "Again".into(),
            },
        )
        .unwrap();
        assert_eq!(logs.len(), 1);
        assert_token_information(&logs[0], "Again", "SECOND");
    });
}

/// A `setName` or `setSymbol` that fails emits nothing.
#[test]
fn a_failed_set_name_or_set_symbol_emits_nothing() {
    run(|| {
        let alice = User::new(Sr25519Keyring::Alice);
        let bob = User::new(Sr25519Keyring::Bob);
        let asset_id = fungible_asset(&alice);
        let token = fungible(asset_id);
        assert_ok!(send(
            &alice,
            token,
            IFungibleAsset::setSymbolCall {
                symbol: "MINE".into(),
            },
        ));
        // Mapping an account emits an event, so do it before counting
        eth_address(&bob);
        let events_before = System::events().len();

        // Not an agent, for a rename and for both `setSymbol` paths
        assert!(send(
            &bob,
            token,
            IFungibleAsset::setNameCall {
                name: "Theirs".into(),
            },
        )
        .is_err());
        for symbol in ["MINE", "OTHER"] {
            assert!(send(
                &bob,
                token,
                IFungibleAsset::setSymbolCall {
                    symbol: symbol.into(),
                },
            )
            .is_err());
        }

        assert_eq!(System::events().len(), events_before);
    });
}
