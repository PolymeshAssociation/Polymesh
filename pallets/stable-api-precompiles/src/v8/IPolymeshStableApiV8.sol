// SPDX-License-Identifier: Apache-2.0
pragma solidity ^0.8.0;

/// @title Polymesh Stable API v8
/// @notice EVM precompile interface for core Polymesh chain operations.

/// @dev The on-chain address of the Polymesh Stable API v8 precompile.
address constant POLYMESH_STABLE_API_V8_ADDRESS = address(0x0000000000000000000000000000000000080000);

/// @notice Portfolio kind: Default or User-defined.
/// @dev Maps to Rust `PortfolioKind`.
enum PolymeshPortfolioKind {
    Default,
    User
}

/// @notice Settlement leg type.
/// @dev Maps to Rust `Leg` variants.
enum PolymeshLegType {
    Fungible,
    NonFungible,
    OffChain
}

/// @notice Venue type for settlement.
/// @dev Maps to Rust `VenueType`.
enum PolymeshVenueType {
    Other,
    Distribution,
    Sto,
    Exchange
}

/// @notice Asset type classification.
/// @dev Maps to Rust `AssetType`.
enum PolymeshAssetType {
    EquityCommon,
    EquityPreferred,
    Commodity,
    FixedIncome,
    REIT,
    Fund,
    RevenueShareAgreement,
    StructuredProduct,
    Derivative,
    Custom,
    StableCoin,
    NonFungible
}

/// @notice Identifies a portfolio by its owner DID and kind.
/// @dev Maps to Rust `PortfolioId { did, kind }`.
struct PolymeshPortfolioId {
    /// @dev Owner identity (32-byte DID).
    bytes32 did;
    /// @dev Portfolio kind.
    PolymeshPortfolioKind kind;
    /// @dev User portfolio number (ignored when kind == Default).
    uint64 number;
}

/// @notice A single leg in a settlement instruction.
/// @dev Flattened from/to portfolios avoid nested struct encoding.
struct PolymeshLeg {
    bytes32 from_did;
    PolymeshPortfolioKind from_portfolio_kind;
    uint64 from_portfolio_number;
    bytes32 to_did;
    PolymeshPortfolioKind to_portfolio_kind;
    uint64 to_portfolio_number;
    /// @dev 16-byte asset UUID.
    bytes16 asset_id;
    uint128 amount;
    PolymeshLegType leg_type;
}

/// @notice Corporate action identifier (asset + local sequence number).
struct PolymeshCAId {
    bytes16 asset_id;
    uint32 local_id;
}

/// @notice Summary of a capital distribution attached to a corporate action.
struct PolymeshDistributionSummary {
    /// @dev Portfolio from which the distribution pays out.
    PolymeshPortfolioId from;
    /// @dev Currency asset used for payment.
    bytes16 currency;
    uint128 per_share;
    uint128 amount;
    /// @dev UNIX timestamp for payment start.
    uint64 payment_at;
    /// @dev UNIX timestamp for expiry (0 = no expiry).
    uint64 expires_at;
}

/// @notice Parameters for creating a simple dividend distribution.
struct PolymeshSimpleDividend {
    bytes16 asset_id;
    /// @dev Declaration date (UNIX timestamp).
    uint64 decl_date;
    /// @dev Record date (UNIX timestamp).
    uint64 record_date;
    /// @dev Portfolio funding the dividend.
    PolymeshPortfolioId portfolio;
    /// @dev Currency asset used for payment.
    bytes16 currency;
    uint128 per_share;
    uint128 amount;
    /// @dev UNIX timestamp for payment start.
    uint64 payment_at;
    /// @dev UNIX timestamp for expiry (0 = no expiry).
    uint64 expires_at;
}

/// @notice Describes an asset transfer within a portfolio move.
struct PolymeshFund {
    bytes16 asset_id;
    /// @dev Fungible amount (0 for NFT-only moves).
    uint128 amount;
    /// @dev NFT token IDs (empty for fungible-only moves).
    uint64[] nft_ids;
    /// @dev Optional memo (empty bytes = no memo).
    bytes memo;
}

interface IPolymeshStableApiV8 {
    // ── Portfolio (pallet_portfolio) ──

    /// @notice Create a named portfolio under the caller's identity.
    /// @param name Portfolio name (UTF-8 encoded).
    /// @return The newly created portfolio identifier.
    function createPortfolio(bytes calldata name) external returns (PolymeshPortfolioId memory);

    /// @notice Accept custody of a portfolio using a pending authorization.
    /// @param authId Authorization ID from the portfolio owner.
    function acceptPortfolioCustody(uint64 authId) external;

    /// @notice Relinquish custody of a portfolio back to its owner.
    /// @param portfolio The portfolio to release.
    function quitPortfolioCustody(PolymeshPortfolioId calldata portfolio) external;

    /// @notice Move assets between two portfolios under the caller's custody.
    /// @param from Source portfolio.
    /// @param to Destination portfolio.
    /// @param funds Array of asset transfers to execute.
    function movePortfolioFunds(
        PolymeshPortfolioId calldata from,
        PolymeshPortfolioId calldata to,
        PolymeshFund[] calldata funds
    ) external;

    /// @notice Query the balance of an asset in a portfolio.
    /// @param portfolio Portfolio to query.
    /// @param assetId Asset identifier (16-byte UUID).
    /// @return The fungible balance held.
    function portfolioAssetBalances(PolymeshPortfolioId calldata portfolio, bytes16 assetId)
        external
        view
        returns (uint128);

    /// @notice Check whether all given portfolios are in custody of a specific identity.
    /// @param portfolios Array of portfolios to check.
    /// @param custodianDid DID of the expected custodian.
    /// @return True if every portfolio is under the custodian's custody.
    function checkPortfoliosInCustody(PolymeshPortfolioId[] calldata portfolios, bytes32 custodianDid)
        external
        view
        returns (bool);

    /// @notice Create a named portfolio under another identity and take custody.
    /// @param ownerDid DID of the portfolio owner.
    /// @param name Portfolio name (UTF-8 encoded).
    /// @return The newly created portfolio identifier.
    function createCustodyPortfolio(bytes32 ownerDid, bytes calldata name)
        external
        returns (PolymeshPortfolioId memory);

    // ── Settlement (pallet_settlement) ──

    /// @notice Create a new settlement venue.
    /// @param details Venue description (UTF-8 encoded).
    /// @param venueType Type of venue.
    /// @return venueId The newly created venue identifier.
    function createVenue(bytes calldata details, PolymeshVenueType venueType) external returns (uint64 venueId);

    /// @notice Execute a pre-existing settlement instruction.
    /// @dev Caller supplies leg counts for proportional gas charging.
    /// @param instructionId Instruction to execute.
    /// @param fungibleCount Number of fungible legs.
    /// @param nftCount Number of NFT legs.
    /// @param offchainCount Number of off-chain legs.
    function settlementExecute(uint64 instructionId, uint32 fungibleCount, uint32 nftCount, uint32 offchainCount)
        external;

    /// @notice Create a settlement instruction and immediately affirm on behalf of the caller.
    /// @param venueId Venue in which to create the instruction.
    /// @param legs Array of settlement legs.
    /// @param portfolios Caller's portfolios to affirm for.
    /// @return instructionId The newly created instruction identifier.
    function addAndAffirmInstruction(
        uint64 venueId,
        PolymeshLeg[] calldata legs,
        PolymeshPortfolioId[] calldata portfolios
    ) external returns (uint64 instructionId);

    // ── Asset (pallet_asset) ──

    /// @notice Create a new asset and issue an initial supply.
    /// @param name Asset name (UTF-8 encoded).
    /// @param assetType Asset type classification.
    /// @param divisible Whether the asset is divisible.
    /// @param amountToIssue Initial supply to mint.
    /// @param portfolioKind Destination portfolio kind.
    /// @param portfolioNumber Destination portfolio number (when kind == User).
    /// @return assetId The newly created asset identifier.
    function assetCreateAndIssue(
        bytes calldata name,
        PolymeshAssetType assetType,
        bool divisible,
        uint128 amountToIssue,
        PolymeshPortfolioKind portfolioKind,
        uint64 portfolioNumber
    ) external returns (bytes16 assetId);

    /// @notice Issue (mint) additional supply of an existing asset.
    /// @param assetId Asset to issue.
    /// @param amount Amount to mint.
    /// @param portfolioKind Destination portfolio kind.
    /// @param portfolioNumber Destination portfolio number.
    function assetIssue(bytes16 assetId, uint128 amount, PolymeshPortfolioKind portfolioKind, uint64 portfolioNumber) external;

    /// @notice Redeem (burn) tokens from the caller's portfolio.
    /// @param assetId Asset to redeem.
    /// @param amount Amount to burn.
    /// @param portfolioKind Source portfolio kind.
    /// @param portfolioNumber Source portfolio number.
    function assetRedeem(bytes16 assetId, uint128 amount, PolymeshPortfolioKind portfolioKind, uint64 portfolioNumber) external;

    /// @notice Query an identity's total balance for an asset.
    /// @param assetId Asset to query.
    /// @param did Identity to query.
    /// @return The total fungible balance.
    function assetBalanceOf(bytes16 assetId, bytes32 did) external view returns (uint128);

    /// @notice Query the total supply of an asset.
    /// @param assetId Asset to query.
    /// @return The total supply.
    function assetTotalSupply(bytes16 assetId) external view returns (uint128);

    /// @notice Resolve a local metadata name to its numeric key for an asset.
    /// @param assetId Asset to query.
    /// @param name Metadata name (UTF-8 encoded).
    /// @return exists True if the name is registered.
    /// @return key The numeric metadata key.
    function assetMetadataLocalNameToKey(bytes16 assetId, bytes calldata name)
        external
        view
        returns (bool exists, uint64 key);

    /// @notice Read a metadata value for an asset.
    /// @param assetId Asset to query.
    /// @param key Numeric metadata key.
    /// @param isLocal True for asset-local keys, false for global keys.
    /// @return exists True if a value is set.
    /// @return value The raw metadata bytes.
    function assetMetadataValue(bytes16 assetId, uint64 key, bool isLocal)
        external
        view
        returns (bool exists, bytes memory value);

    // ── Identity (pallet_identity) ──

    /// @notice Look up the Polymesh DID associated with an EVM address.
    /// @param account The EVM address to resolve.
    /// @return did The associated identity (zero if none).
    function getKeyDid(address account) external view returns (bytes32 did);

    /// @notice Get the next asset ID that would be assigned to an identity.
    /// @param account The EVM address whose identity to query.
    /// @return assetId The next asset identifier.
    function getNextAssetId(address account) external view returns (bytes16 assetId);

    // ── NFT (pallet_nft) ──

    /// @notice Look up the owner of an NFT.
    /// @param assetId NFT collection asset identifier.
    /// @param nftId Token ID within the collection.
    /// @return exists True if the NFT exists.
    /// @return owner Portfolio that holds the NFT.
    function nftOwner(bytes16 assetId, uint64 nftId)
        external
        view
        returns (bool exists, PolymeshPortfolioId memory owner);

    /// @notice Check whether a portfolio holds all the specified NFTs.
    /// @param portfolio Portfolio to check.
    /// @param assetId NFT collection asset identifier.
    /// @param nftIds Array of token IDs to verify.
    /// @return True if the portfolio holds every listed NFT.
    function holdsNfts(PolymeshPortfolioId calldata portfolio, bytes16 assetId, uint64[] calldata nftIds)
        external
        view
        returns (bool);

    // ── Corporate Actions (pallet_corporate_actions) ──

    /// @notice Query the summary of a capital distribution.
    /// @param caId Corporate action identifier.
    /// @return exists True if the distribution exists.
    /// @return summary The distribution details.
    function distributionSummary(PolymeshCAId calldata caId)
        external
        view
        returns (bool exists, PolymeshDistributionSummary memory summary);

    /// @notice Claim a dividend payout for the caller.
    /// @param caId Corporate action identifier of the distribution.
    function dividendClaim(PolymeshCAId calldata caId) external;

    /// @notice Create a dividend distribution on an existing corporate action.
    /// @dev Requires a pre-existing corporate action for the asset.
    /// @param dividend The dividend parameters.
    function createDividend(PolymeshSimpleDividend calldata dividend) external;
}
