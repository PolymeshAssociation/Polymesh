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
