// SPDX-License-Identifier: MIT
pragma solidity ^0.8.20;

///
/// @dev Interface exposing a subset of general-purpose Polymesh runtime extrinsics that are not
/// scoped to a single asset.
///
/// Covers:
///   - `pallet_asset::create_asset`
///   - `pallet_asset::register_unique_ticker`
///   - `pallet_identity::register_did`
///   - `pallet_identity::self_register_did`
///
interface IPolymeshRuntime {
    // ============================================================
    // Shared types
    // ============================================================

    /// @dev Mirrors `polymesh_primitives::asset::AssetType`. The `NonFungible*` variants
    /// correspond to `AssetType::NonFungible(NonFungibleType::*)`.
    ///
    /// When `kind` is `Custom` or `NonFungibleCustom`, `customTypeId` must be set to the id of a
    /// previously registered custom asset type; for every other variant it is ignored.
    enum AssetTypeKind {
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
        NonFungibleDerivative,
        NonFungibleFixedIncome,
        NonFungibleInvoice,
        NonFungibleCustom
    }

    /// @dev A `polymesh_primitives::asset::AssetType`, see {AssetTypeKind}.
    struct AssetType {
        AssetTypeKind kind;
        uint32 customTypeId;
    }

    /// @dev Mirrors the variants of `polymesh_primitives::AssetIdentifier`.
    enum AssetIdentifierKind {
        CUSIP,
        CINS,
        ISIN,
        LEI,
        FIGI
    }

    /// @dev A `polymesh_primitives::AssetIdentifier`. `value` must be exactly as long as required
    /// by `identifierType`: CUSIP = 9 bytes, CINS = 9 bytes, ISIN = 12 bytes, LEI = 20 bytes,
    /// FIGI = 12 bytes.
    struct AssetIdentifier {
        AssetIdentifierKind identifierType;
        bytes value;
    }

    // ============================================================
    // pallet_asset:
    // - create_asset
    // - register_unique_ticker
    // ============================================================

    /// @dev Emitted when a new asset is created, mirroring `pallet_asset::Event::AssetCreated`.
    /// @param did The DID of the caller/owner that created the asset.
    /// @param assetId The id of the newly created asset.
    /// @param assetName The name of the asset.
    event AssetCreated(bytes32 indexed did, bytes16 indexed assetId, string assetName);

    /// @dev Emitted when a ticker is registered, mirroring `pallet_asset::Event::TickerRegistered`.
    /// @param did The DID that now owns the ticker.
    /// @param ticker The ticker that was registered.
    /// @param expiry The unix timestamp (in milliseconds) at which the registration expires, or
    /// `0` if the registration does not expire.
    event TickerRegistered(bytes32 indexed did, string ticker, uint64 expiry);

    /// @notice Creates a new asset, registering it under the caller's identity.
    /// @dev Calls `pallet_asset::create_asset`. Emits {AssetCreated}.
    /// @param assetName The name of the new asset.
    /// @param divisible Whether the asset is divisible.
    /// @param assetType The type of security represented by the asset, see {AssetType}.
    /// @param assetIdentifiers The identifiers to attach to the asset, see {AssetIdentifier}.
    /// @param fundingRoundName The name of the funding round; pass an empty string for `None`.
    /// @return assetId The id assigned to the newly created asset.
    function assetCreateAsset(
        string calldata assetName,
        bool divisible,
        AssetType calldata assetType,
        AssetIdentifier[] calldata assetIdentifiers,
        string calldata fundingRoundName
    ) external returns (bytes16 assetId);

    /// @notice Registers a ticker symbol to the caller's identity.
    /// @dev Calls `pallet_asset::register_unique_ticker`. Emits {TickerRegistered}.
    /// @param ticker The ticker symbol to register. Must be at most 12 characters long.
    function assetRegisterTicker(string calldata ticker) external;

    // ============================================================
    // pallet_identity:
    // - register_did
    // - self_register_did
    // ============================================================

    /// @dev Emitted when a new DID is created, mirroring `pallet_identity::Event::DidCreated`.
    /// @param did The newly created DID.
    /// @param targetAccount The account the new DID was linked to as its primary key.
    event DidCreated(bytes32 indexed did, address indexed targetAccount);

    /// @notice Registers a new DID for `targetAccount`. The caller must be an active DID
    /// registrar (formerly CDD provider). The new identity has no secondary keys.
    /// @dev Calls `pallet_identity::register_did`. Emits {DidCreated}.
    /// @param targetAccount The account to become the primary key of the new identity.
    /// @return did The newly created DID.
    function identityRegisterDid(address targetAccount) external returns (bytes32 did);

    /// @notice Registers a new DID for the caller's own account, allowing self onboarding without
    /// a DID registrar.
    /// @dev Calls `pallet_identity::self_register_did`. Emits {DidCreated}.
    /// @return did The newly created DID.
    function identitySelfRegisterDid() external returns (bytes32 did);
}

contract PolymeshRuntimeStub is IPolymeshRuntime {

    error NotExecutable();

    function assetCreateAsset(
        string calldata assetName,
        bool divisible,
        IPolymeshRuntime.AssetType calldata assetType,
        IPolymeshRuntime.AssetIdentifier[] calldata assetIdentifiers,
        string calldata fundingRoundName
    ) external override returns (bytes16) {
        assetName;
        divisible;
        assetType;
        assetIdentifiers;
        fundingRoundName;
        revert NotExecutable();
    }

    function assetRegisterTicker(string calldata ticker) external override {
        ticker;
        revert NotExecutable();
    }

    function identityRegisterDid(address targetAccount) external override returns (bytes32) {
        targetAccount;
        revert NotExecutable();
    }

    function identitySelfRegisterDid() external override returns (bytes32) {
        revert NotExecutable();
    }
}
