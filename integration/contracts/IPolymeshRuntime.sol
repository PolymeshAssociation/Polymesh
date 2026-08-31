// SPDX-License-Identifier: MIT
pragma solidity ^0.8.20;

/// @dev Local mirror of the subset of `IPolymeshRuntime` that `Onboarder.sol` calls; see
/// `precompiles/src/interfaces/PolymeshRuntime.sol` for the authoritative interface. Kept
/// self-contained like the other test contracts instead of importing across packages.
interface IPolymeshRuntime {
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

    struct AssetType {
        AssetTypeKind kind;
        uint32 customTypeId;
    }

    enum AssetIdentifierKind {
        CUSIP,
        CINS,
        ISIN,
        LEI,
        FIGI
    }

    struct AssetIdentifier {
        AssetIdentifierKind identifierType;
        bytes value;
    }

    function assetCreateAsset(
        string calldata assetName,
        bool divisible,
        AssetType calldata assetType,
        AssetIdentifier[] calldata assetIdentifiers,
        string calldata fundingRoundName
    ) external returns (bytes16 assetId);

    function assetRegisterTicker(string calldata ticker) external;

    function identitySelfRegisterDid() external returns (bytes32 did);
}
