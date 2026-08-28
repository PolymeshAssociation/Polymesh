use pallet_precompiles::interface::nft::{
    ERC165_INTERFACE_ID, ERC721_INTERFACE_ID, ERC721_METADATA_INTERFACE_ID,
};
use pallet_revive::precompiles::alloy::sol_types::SolCall;
use polymesh_precompiles::INonFungibleAsset;

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
