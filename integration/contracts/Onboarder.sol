// SPDX-License-Identifier: MIT
pragma solidity ^0.8.20;

import "./IPolymeshRuntime.sol";

/// @dev Exercises calling several `IPolymeshRuntime` extrinsics from inside a single contract
/// call: this contract self-registers its own identity, then creates an asset and registers a
/// ticker under it, all attributed to the contract's own account rather than the caller's.
///
/// Ordinarily a contract can't sign extrinsics, so it needs a DID registrar to onboard it before
/// it can hold assets. `identitySelfRegisterDid` lets it onboard itself instead.
contract Onboarder {
    IPolymeshRuntime private constant RUNTIME = IPolymeshRuntime(address(uint160(0xFFFF0000)));

    function onboardAndCreateAsset(
        string calldata assetName,
        bool divisible,
        string calldata ticker
    ) external returns (bytes32 did, bytes16 assetId) {
        did = RUNTIME.identitySelfRegisterDid();

        assetId = RUNTIME.assetCreateAsset(
            assetName,
            divisible,
            IPolymeshRuntime.AssetType({
                kind: IPolymeshRuntime.AssetTypeKind.EquityCommon,
                customTypeId: 0
            }),
            new IPolymeshRuntime.AssetIdentifier[](0),
            ""
        );

        RUNTIME.assetRegisterTicker(ticker);
    }
}
