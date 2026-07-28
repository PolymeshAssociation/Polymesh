// SPDX-License-Identifier: GPL-3.0-only
pragma solidity ^0.8.0;

/// @title Polymesh Identity Precompile Interface
/// @notice Fixed-address precompile exposing Polymesh identity (DID) operations.
interface IPolymeshIdentity {
    /// Emitted when a new DID is registered through the precompile.
    event DidRegistered(address indexed account, bytes32 did);
    /// Emitted when a claim is added to a DID.
    event ClaimAdded(bytes32 indexed target, uint8 claimType, bytes16 assetId);
    /// Emitted when a claim is revoked from a DID.
    event ClaimRevoked(bytes32 indexed target, uint8 claimType, bytes16 assetId);

    // ==================== Views ====================

    /// Returns the DID linked to `account`, or zero if none.
    function identity(address account) external view returns (bytes32);

    /// Returns true if `account` is linked to an active DID.
    function isVerified(address account) external view returns (bool);

    /// Returns true if `account` is linked to an active DID.
    /// @dev CDD claims are no longer enforced on Polymesh; DID existence is
    ///      sufficient for onboarding. Provided for ERC-3643 compatibility.
    function hasValidCdd(address account) external view returns (bool);

    // ==================== Writes ====================

    /// Register a new DID for the caller's account.
    function selfRegisterDid() external returns (bool);

    /// Register a new DID for `target`. Caller must be a DID registrar.
    function registerDid(address target) external returns (bool);

    /// Add a claim to the `target` DID. The caller's DID is the claim issuer.
    ///
    /// `claimType`: 1=Accredited, 2=Affiliate, 3=BuyLockup, 4=SellLockup,
    ///              5=KnowYourCustomer, 6=Jurisdiction, 7=Exempted, 8=Blocked,
    ///              9=Custom.
    /// `assetId`: the asset scope of the claim. For Custom claims a zero
    ///            asset id means no scope.
    /// `claimData`: country code for Jurisdiction claims, custom claim type
    ///              id for Custom claims, unused otherwise.
    /// `expiry`: unix timestamp in milliseconds; 0 means no expiry.
    function addClaim(
        bytes32 target,
        uint8 claimType,
        bytes16 assetId,
        uint256 claimData,
        uint64 expiry
    ) external returns (bool);

    /// Revoke a claim previously issued by the caller's DID.
    /// Parameters mirror `addClaim`.
    function revokeClaim(
        bytes32 target,
        uint8 claimType,
        bytes16 assetId,
        uint256 claimData
    ) external returns (bool);
}
