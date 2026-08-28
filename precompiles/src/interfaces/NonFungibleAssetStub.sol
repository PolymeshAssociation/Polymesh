// SPDX-License-Identifier: MIT
// This interface combines methods from several OpenZeppelin contracts:
//
// IERC721.sol (base ERC-721 interface)
// https://github.com/OpenZeppelin/openzeppelin-contracts/blob/master/contracts/token/ERC721/IERC721.sol
//
// IERC721Metadata.sol (ERC-721 metadata extension)
// https://github.com/OpenZeppelin/openzeppelin-contracts/blob/master/contracts/token/ERC721/extensions/IERC721Metadata.sol
//
// IERC165.sol (standard interface detection)
// https://github.com/OpenZeppelin/openzeppelin-contracts/blob/master/contracts/utils/introspection/IERC165.sol
//
pragma solidity ^0.8.20;

///
/// @dev Interface combining the ERC-721 standard, its metadata extension, ERC-165 introspection,
/// Polymesh-specific mint/burn, and the ERC-7943 transfer-restriction surface.
/// Note: Due to ABI generation constraints, all interfaces are merged into a single contract.
///
/// Each precompile address corresponds to exactly one Polymesh NFT collection, and `tokenId` is
/// the on-chain `NFTId` within that collection.
///
/// Deviations from ERC-721, all deliberate:
///
/// - `safeTransferFrom` only accepts externally-owned accounts. Polymesh precompiles do not
///   re-enter the EVM, so the `onERC721Received` callback cannot be made; rather than skip the
///   check and risk locking a token in a contract that cannot handle it, transfers to any
///   address with code are rejected. This is stricter than ERC-721 — a compliant receiver
///   contract is refused too — but it never weakens the guarantee the method exists to provide.
///   Contract recipients that knowingly handle NFTs can use `transferFrom`.
/// - `ownerOf` reverts for an NFT held in a Polymesh portfolio rather than in an account key,
///   because portfolios have no EVM address.
///
interface INonFungibleAsset {
    // ============================================================
    // IERC721 - Base ERC-721 Interface
    // https://github.com/OpenZeppelin/openzeppelin-contracts/blob/master/contracts/token/ERC721/IERC721.sol
    // ============================================================

    /// @dev Emitted when `tokenId` token is transferred from `from` to `to`.
    event Transfer(address indexed from, address indexed to, uint256 indexed tokenId);

    /// @dev Emitted when `owner` enables `approved` to manage the `tokenId` token.
    event Approval(address indexed owner, address indexed approved, uint256 indexed tokenId);

    /// @dev Emitted when `owner` enables or disables (`approved`) `operator` to manage all of its
    /// assets.
    event ApprovalForAll(address indexed owner, address indexed operator, bool approved);

    /// @dev Returns the number of tokens in `owner`'s account.
    ///
    /// Only NFTs held by the account key are counted; NFTs the same identity holds in portfolios
    /// are not.
    function balanceOf(address owner) external view returns (uint256 balance);

    /// @dev Returns the owner of the `tokenId` token.
    ///
    /// Requirements:
    ///
    /// - `tokenId` must exist.
    /// - `tokenId` must be held by an account key, not by a portfolio.
    function ownerOf(uint256 tokenId) external view returns (address owner);

    /// @dev Transfers `tokenId` token from `from` to `to`.
    ///
    /// Requirements:
    ///
    /// - `from` must be the current owner.
    /// - `tokenId` must exist.
    /// - If the caller is not `from`, it must be approved to move this token by either {approve}
    ///   or {setApprovalForAll}.
    /// - The transfer must satisfy the collection's compliance rules.
    ///
    /// Emits a {Transfer} event.
    function transferFrom(address from, address to, uint256 tokenId) external;

    /// @dev Safely transfers `tokenId` token from `from` to `to`.
    ///
    /// Requirements:
    ///
    /// - As {transferFrom}, plus:
    /// - `to` MUST be an externally-owned account. Transfers to any address with code revert,
    ///   because the `onERC721Received` callback cannot be performed. See the deviations note.
    ///
    /// Emits a {Transfer} event.
    function safeTransferFrom(address from, address to, uint256 tokenId) external;

    /// @dev Safely transfers `tokenId` token from `from` to `to`.
    ///
    /// Identical to the overload above; `data` is accepted for ABI compatibility and ignored,
    /// since it would only ever be forwarded to the `onERC721Received` callback.
    ///
    /// Emits a {Transfer} event.
    function safeTransferFrom(address from, address to, uint256 tokenId, bytes calldata data) external;

    /// @dev Gives permission to `to` to transfer `tokenId` token to another account.
    /// The approval is cleared when the token is transferred.
    ///
    /// Only a single account can be approved at a time, so approving the zero address clears
    /// previous approvals.
    ///
    /// Requirements:
    ///
    /// - The caller must own the token or be an approved operator.
    /// - `tokenId` must exist.
    ///
    /// Emits an {Approval} event.
    function approve(address to, uint256 tokenId) external;

    /// @dev Approve or remove `operator` as an operator for the caller.
    /// Operators can call {transferFrom} or {safeTransferFrom} for any token owned by the caller.
    ///
    /// Emits an {ApprovalForAll} event.
    function setApprovalForAll(address operator, bool approved) external;

    /// @dev Returns the account approved for `tokenId` token, or the zero address if none.
    function getApproved(uint256 tokenId) external view returns (address operator);

    /// @dev Returns if the `operator` is allowed to manage all of the assets of `owner`.
    function isApprovedForAll(address owner, address operator) external view returns (bool);

    // ============================================================
    // IERC721Metadata - ERC-721 Metadata Extension
    // https://github.com/OpenZeppelin/openzeppelin-contracts/blob/master/contracts/token/ERC721/extensions/IERC721Metadata.sol
    // ============================================================

    /// @dev Returns the collection name.
    function name() external view returns (string memory);

    /// @dev Returns the collection symbol.
    function symbol() external view returns (string memory);

    /// @dev Returns the Uniform Resource Identifier (URI) for `tokenId` token.
    ///
    /// Resolved from the NFT's own `tokenUri` metadata value, falling back to the collection's
    /// `baseTokenUri`. In either case a literal `{tokenId}` placeholder is replaced with the
    /// decimal token id; if no placeholder is present the id is appended. Returns an empty
    /// string when neither value is set.
    function tokenURI(uint256 tokenId) external view returns (string memory);

    // ============================================================
    // IERC165 - Standard Interface Detection
    // https://github.com/OpenZeppelin/openzeppelin-contracts/blob/master/contracts/utils/introspection/IERC165.sol
    // ============================================================

    /// @dev Returns true if this contract implements the interface defined by `interfaceId`.
    ///
    /// Reports `true` for `IERC165`, `IERC721` and `IERC721Metadata`.
    function supportsInterface(bytes4 interfaceId) external view returns (bool);

    // ============================================================
    // Polymesh specific
    // ============================================================

    /// @dev Returns the total number of NFTs in this collection.
    function totalSupply() external view returns (uint256);

    /// @dev Issues a new NFT of this collection to the caller's account key.
    ///
    /// `metadataValues` must supply exactly one value per mandatory collection metadata key, in
    /// the order the keys were registered for the collection.
    ///
    /// Returns the id of the newly issued token.
    ///
    /// Emits a {Transfer} event from the zero address.
    function mint(bytes[] calldata metadataValues) external returns (uint256 tokenId);

    /// @dev Redeems (burns) the `tokenId` NFT held by the caller's account key.
    ///
    /// Emits a {Transfer} event to the zero address.
    function burn(uint256 tokenId) external returns (bool);

    // ============================================================
    // ERC-7943 - Transfer restrictions
    // ============================================================

    /// @dev Emitted when a token is forcibly transferred by an agent of the collection.
    event ForcedTransfer(address indexed from, address indexed to, uint256 indexed tokenId);

    /// @dev Returns true if `tokenId` can currently be transferred from `from` to `to`.
    function canTransfer(address from, address to, uint256 tokenId) external view returns (bool);

    /// @dev Forcibly transfers `tokenId` from `from` to the caller, bypassing compliance and
    /// frozen checks. The caller must be an agent of the collection.
    ///
    /// Emits {ForcedTransfer} and {Transfer} events.
    function forcedTransfer(address from, uint256 tokenId) external returns (bool);

    /// @dev Returns true if `account` is currently allowed to send tokens of this collection.
    function canSend(address account) external view returns (bool);

    /// @dev Returns true if `account` is currently allowed to receive tokens of this collection.
    function canReceive(address account) external view returns (bool);
}

contract NonFungibleAssetStub is INonFungibleAsset {

    error NotExecutable();

    function balanceOf(address owner) external view override returns (uint256) {
        owner;
        revert NotExecutable();
    }

    function ownerOf(uint256 tokenId) external view override returns (address) {
        tokenId;
        revert NotExecutable();
    }

    function transferFrom(address from, address to, uint256 tokenId) external override {
        from;
        to;
        tokenId;
        revert NotExecutable();
    }

    function safeTransferFrom(address from, address to, uint256 tokenId) external override {
        from;
        to;
        tokenId;
        revert NotExecutable();
    }

    function safeTransferFrom(address from, address to, uint256 tokenId, bytes calldata data) external override {
        from;
        to;
        tokenId;
        data;
        revert NotExecutable();
    }

    function approve(address to, uint256 tokenId) external override {
        to;
        tokenId;
        revert NotExecutable();
    }

    function setApprovalForAll(address operator, bool approved) external override {
        operator;
        approved;
        revert NotExecutable();
    }

    function getApproved(uint256 tokenId) external view override returns (address) {
        tokenId;
        revert NotExecutable();
    }

    function isApprovedForAll(address owner, address operator) external view override returns (bool) {
        owner;
        operator;
        revert NotExecutable();
    }

    function name() external view override returns (string memory) {
        revert NotExecutable();
    }

    function symbol() external view override returns (string memory) {
        revert NotExecutable();
    }

    function tokenURI(uint256 tokenId) external view override returns (string memory) {
        tokenId;
        revert NotExecutable();
    }

    function supportsInterface(bytes4 interfaceId) external view override returns (bool) {
        interfaceId;
        revert NotExecutable();
    }

    function totalSupply() external view override returns (uint256) {
        revert NotExecutable();
    }

    function mint(bytes[] calldata metadataValues) external override returns (uint256) {
        metadataValues;
        revert NotExecutable();
    }

    function burn(uint256 tokenId) external override returns (bool) {
        tokenId;
        revert NotExecutable();
    }

    function canTransfer(address from, address to, uint256 tokenId) external view override returns (bool) {
        from;
        to;
        tokenId;
        revert NotExecutable();
    }

    function forcedTransfer(address from, uint256 tokenId) external override returns (bool) {
        from;
        tokenId;
        revert NotExecutable();
    }

    function canSend(address account) external view override returns (bool) {
        account;
        revert NotExecutable();
    }

    function canReceive(address account) external view override returns (bool) {
        account;
        revert NotExecutable();
    }
}
