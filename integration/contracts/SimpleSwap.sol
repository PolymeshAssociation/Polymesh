// SPDX-License-Identifier: MIT
pragma solidity ^0.8.20;

import "./IERC20.sol";

/// @dev Deliberately minimal fixed-rate swap between two ERC-20 tokens.
///
/// Either token may be a Polymesh native asset (through the ERC-20 precompile)
/// or a plain Solidity ERC-20. The contract holds its own inventory: to provide
/// liquidity simply transfer tokens to the contract address.
///
/// This is test scaffolding, not a production AMM: there is no fee, no slippage
/// protection and no access control.
contract SimpleSwap {
    IERC20 public immutable tokenA;
    IERC20 public immutable tokenB;

    /// @dev `amountB = amountA * rateNum / rateDen`.
    uint256 public immutable rateNum;
    uint256 public immutable rateDen;

    event Swap(
        address indexed caller,
        address indexed tokenIn,
        address indexed tokenOut,
        uint256 amountIn,
        uint256 amountOut
    );

    constructor(address a, address b, uint256 num, uint256 den) {
        require(a != address(0) && b != address(0), "SimpleSwap: zero token");
        require(a != b, "SimpleSwap: identical tokens");
        require(num != 0 && den != 0, "SimpleSwap: zero rate");
        tokenA = IERC20(a);
        tokenB = IERC20(b);
        rateNum = num;
        rateDen = den;
    }

    function quoteAtoB(uint256 amountIn) public view returns (uint256) {
        return (amountIn * rateNum) / rateDen;
    }

    function quoteBtoA(uint256 amountIn) public view returns (uint256) {
        return (amountIn * rateDen) / rateNum;
    }

    function swapAtoB(uint256 amountIn) external returns (uint256 amountOut) {
        amountOut = quoteAtoB(amountIn);
        _swap(tokenA, tokenB, amountIn, amountOut);
    }

    function swapBtoA(uint256 amountIn) external returns (uint256 amountOut) {
        amountOut = quoteBtoA(amountIn);
        _swap(tokenB, tokenA, amountIn, amountOut);
    }

    function _swap(IERC20 tokenIn, IERC20 tokenOut, uint256 amountIn, uint256 amountOut) private {
        require(amountIn != 0, "SimpleSwap: zero input");
        require(amountOut != 0, "SimpleSwap: zero output");
        require(
            tokenOut.balanceOf(address(this)) >= amountOut,
            "SimpleSwap: insufficient liquidity"
        );

        require(
            tokenIn.transferFrom(msg.sender, address(this), amountIn),
            "SimpleSwap: transferFrom failed"
        );
        require(tokenOut.transfer(msg.sender, amountOut), "SimpleSwap: transfer failed");

        emit Swap(msg.sender, address(tokenIn), address(tokenOut), amountIn, amountOut);
    }
}
