// SPDX-License-Identifier: MIT
pragma solidity ^0.8.20;

/// @dev ABI of `Onboarder.sol`, used to generate the Rust bindings.
interface IOnboarder {
    /// @dev Self-registers a DID for this contract's own account, then creates an asset and
    /// registers a ticker under that identity - all in one transaction.
    /// @param assetName The name of the new asset.
    /// @param divisible Whether the asset is divisible.
    /// @param ticker The ticker symbol to register. Must be at most 12 characters long.
    /// @return did The DID self-registered for this contract.
    /// @return assetId The id assigned to the newly created asset.
    function onboardAndCreateAsset(
        string calldata assetName,
        bool divisible,
        string calldata ticker
    ) external returns (bytes32 did, bytes16 assetId);
}
