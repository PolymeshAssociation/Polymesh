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
use polymesh_primitives::asset::AssetId;
use sp_core::H256;
use sp_keyring::Sr25519Keyring;

use pallet_asset::Allowances;
use polymesh_primitives::{AssetHolderKind, Balance};

use crate::asset_pallet::setup::ISSUE_AMOUNT;
use crate::ext_builder::ExtBuilder;
use crate::storage::{EventTest, TestStorage, User};

type Asset = pallet_asset::Pallet<TestStorage>;
type Revive = pallet_revive::Pallet<TestStorage>;
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
        assert_ok!(call(&bob, token, spend));
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
