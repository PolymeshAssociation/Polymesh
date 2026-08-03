// SPDX-License-Identifier: MIT
pragma solidity ^0.8.20;

import "./IERC20.sol";

/// @dev Minimal, unaudited ERC-20 used only by the integration tests.
///
/// Uses 6 decimals so that amounts line up with Polymesh native assets, which
/// the ERC-20 precompile always reports as having 6 decimals.
contract TestERC20 is IERC20 {
    string private _name;
    string private _symbol;
    uint256 private _totalSupply;

    mapping(address => uint256) private _balances;
    mapping(address => mapping(address => uint256)) private _allowances;

    constructor(string memory name_, string memory symbol_) {
        _name = name_;
        _symbol = symbol_;
    }

    function name() external view returns (string memory) {
        return _name;
    }

    function symbol() external view returns (string memory) {
        return _symbol;
    }

    function decimals() external pure returns (uint8) {
        return 6;
    }

    function totalSupply() external view returns (uint256) {
        return _totalSupply;
    }

    function balanceOf(address account) external view returns (uint256) {
        return _balances[account];
    }

    function allowance(address owner, address spender) external view returns (uint256) {
        return _allowances[owner][spender];
    }

    /// @dev Open mint: this token exists purely for testing.
    function mint(address to, uint256 value) external returns (bool) {
        require(to != address(0), "TestERC20: mint to zero address");
        _totalSupply += value;
        _balances[to] += value;
        emit Transfer(address(0), to, value);
        return true;
    }

    function transfer(address to, uint256 value) external returns (bool) {
        _transfer(msg.sender, to, value);
        return true;
    }

    function approve(address spender, uint256 value) external returns (bool) {
        _allowances[msg.sender][spender] = value;
        emit Approval(msg.sender, spender, value);
        return true;
    }

    function transferFrom(address from, address to, uint256 value) external returns (bool) {
        uint256 allowed = _allowances[from][msg.sender];
        require(allowed >= value, "TestERC20: insufficient allowance");
        if (allowed != type(uint256).max) {
            _allowances[from][msg.sender] = allowed - value;
        }
        _transfer(from, to, value);
        return true;
    }

    function _transfer(address from, address to, uint256 value) private {
        require(to != address(0), "TestERC20: transfer to zero address");
        uint256 balance = _balances[from];
        require(balance >= value, "TestERC20: insufficient balance");
        unchecked {
            _balances[from] = balance - value;
        }
        _balances[to] += value;
        emit Transfer(from, to, value);
    }
}
