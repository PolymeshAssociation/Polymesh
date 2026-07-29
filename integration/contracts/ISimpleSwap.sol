// SPDX-License-Identifier: MIT
pragma solidity ^0.8.20;

/// @dev ABI of `SimpleSwap.sol`, used to generate the Rust bindings.
interface ISimpleSwap {
    /// @dev Emitted after a successful swap.
    event Swap(
        address indexed caller,
        address indexed tokenIn,
        address indexed tokenOut,
        uint256 amountIn,
        uint256 amountOut
    );

    function tokenA() external view returns (address);

    function tokenB() external view returns (address);

    function rateNum() external view returns (uint256);

    function rateDen() external view returns (uint256);

    /// @dev Quotes `swapAtoB` without changing any state.
    function quoteAtoB(uint256 amountIn) external view returns (uint256);

    /// @dev Quotes `swapBtoA` without changing any state.
    function quoteBtoA(uint256 amountIn) external view returns (uint256);

    /// @dev Pulls `amountIn` of token A from the caller and sends back token B.
    ///
    /// The caller must have approved this contract on token A first.
    function swapAtoB(uint256 amountIn) external returns (uint256 amountOut);

    /// @dev Pulls `amountIn` of token B from the caller and sends back token A.
    ///
    /// The caller must have approved this contract on token B first.
    function swapBtoA(uint256 amountIn) external returns (uint256 amountOut);
}
