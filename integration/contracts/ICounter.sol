// SPDX-License-Identifier: MIT
pragma solidity ^0.8.20;

/// @dev ABI of `Counter.sol`, used to generate the Rust bindings.
interface ICounter {
    /// @dev Emitted whenever the stored value changes.
    event Incremented(address indexed caller, uint256 newValue);

    /// @dev Returns the currently stored value.
    function number() external view returns (uint256);

    /// @dev Increments the stored value by one and returns the new value.
    function increment() external returns (uint256);

    /// @dev Overwrites the stored value.
    function setNumber(uint256 newValue) external;

    /// @dev Always reverts. Used to test revert-reason propagation.
    function boom() external;
}
