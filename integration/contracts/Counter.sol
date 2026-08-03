// SPDX-License-Identifier: MIT
pragma solidity ^0.8.20;

/// @dev Trivial state-holding contract used to exercise contract deployment and
/// calls from both Substrate extrinsics and Ethereum transactions.
contract Counter {
    uint256 private _number;

    event Incremented(address indexed caller, uint256 newValue);

    constructor(uint256 initialValue) {
        _number = initialValue;
    }

    function number() external view returns (uint256) {
        return _number;
    }

    function increment() external returns (uint256) {
        _number += 1;
        emit Incremented(msg.sender, _number);
        return _number;
    }

    function setNumber(uint256 newValue) external {
        _number = newValue;
        emit Incremented(msg.sender, newValue);
    }

    function boom() external pure {
        revert("Counter: boom");
    }
}
