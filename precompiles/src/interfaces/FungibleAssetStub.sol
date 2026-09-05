// SPDX-License-Identifier: MIT
// This interface combines methods from three OpenZeppelin contracts:
//
// IERC20.sol (base ERC-20 interface)
// https://github.com/OpenZeppelin/openzeppelin-contracts/blob/master/contracts/token/ERC20/IERC20.sol
//
// IERC20Metadata.sol (ERC-20 metadata extension)
// https://github.com/OpenZeppelin/openzeppelin-contracts/blob/master/contracts/token/ERC20/extensions/IERC20Metadata.sol
//
// IERC20Permit.sol (EIP-2612 permit extension)
// https://github.com/OpenZeppelin/openzeppelin-contracts/blob/master/contracts/token/ERC20/extensions/IERC20Permit.sol
//
pragma solidity ^0.8.20;

///
/// @dev Interface combining the ERC-20 standard, its metadata extension, and EIP-2612 permit.
/// Note: Due to ABI generation constraints, all interfaces are merged into a single contract.
///
interface IFungibleAsset {
    // ============================================================
    // IERC20 - Base ERC-20 Interface
    // https://github.com/OpenZeppelin/openzeppelin-contracts/blob/master/contracts/token/ERC20/IERC20.sol
    // ============================================================

    /// @dev Emitted when `value` tokens are moved from one account (`from`) to
    /// another (`to`).
    ///
    /// Note that `value` may be zero.
    event Transfer(address indexed from, address indexed to, uint256 value);

    /// @dev Emitted when the allowance of a `spender` for an `owner` is set by
    /// a call to {approve}. `value` is the new allowance.
    event Approval(address indexed owner, address indexed spender, uint256 value);

    /// @dev Returns the value of tokens in existence.
    function totalSupply() external view returns (uint256);

    /// @dev Returns the value of tokens owned by `account`.
    function balanceOf(address account) external view returns (uint256);

    /// @dev Moves a `value` amount of tokens from the caller's account to `to`.
    ///
    /// Returns a boolean value indicating whether the operation succeeded.
    ///
    /// Emits a {Transfer} event.
    function transfer(address to, uint256 value) external returns (bool);

    /// @dev Returns the remaining number of tokens that `spender` will be
    /// allowed to spend on behalf of `owner` through {transferFrom}. This is
    /// zero by default.
    ///
    /// This value changes when {approve} or {transferFrom} are called.
    function allowance(address owner, address spender) external view returns (uint256);

    /// @dev Sets a `value` amount of tokens as the allowance of `spender` over the
    /// caller's tokens.
    ///
    /// Returns a boolean value indicating whether the operation succeeded.
    ///
    /// IMPORTANT: Beware that changing an allowance with this method brings the risk
    /// that someone may use both the old and the new allowance by unfortunate
    /// transaction ordering. One possible solution to mitigate this race
    /// condition is to first reduce the spender's allowance to 0 and set the
    /// desired value afterwards:
    /// https://github.com/ethereum/EIPs/issues/20#issuecomment-263524729
    ///
    /// Emits an {Approval} event.
    function approve(address spender, uint256 value) external returns (bool);

    /// @dev Moves a `value` amount of tokens from `from` to `to` using the
    /// allowance mechanism. `value` is then deducted from the caller's
    /// allowance.
    ///
    /// Returns a boolean value indicating whether the operation succeeded.
    ///
    /// Emits a {Transfer} event.
    function transferFrom(address from, address to, uint256 value) external returns (bool);

    // ============================================================
    // IERC20Metadata - ERC-20 Metadata Extension
    // https://github.com/OpenZeppelin/openzeppelin-contracts/blob/master/contracts/token/ERC20/extensions/IERC20Metadata.sol
    // ============================================================

    /// @dev Returns the name of the token.
    function name() external view returns (string memory);

    /// @dev Returns the symbol of the token.
    function symbol() external view returns (string memory);

    /// @dev Returns the decimals places of the token.
    function decimals() external view returns (uint8);

    // ============================================================
    // IERC20Permit - EIP-2612 Permit Extension
    // https://github.com/OpenZeppelin/openzeppelin-contracts/blob/master/contracts/token/ERC20/extensions/IERC20Permit.sol
    // ============================================================

    /// @dev Sets `value` as the allowance of `spender` over ``owner``'s tokens,
    /// given ``owner``'s signed approval.
    ///
    /// IMPORTANT: The same issues {IERC20-approve} has related to transaction
    /// ordering also apply here.
    ///
    /// Emits an {Approval} event.
    ///
    /// Requirements:
    ///
    /// - `spender` cannot be the zero address.
    /// - `deadline` must be a timestamp in the future.
    /// - `v`, `r` and `s` must be a valid `secp256k1` signature from `owner`
    /// over the EIP712-formatted function arguments.
    /// - the signature must use ``owner``'s current nonce (see {nonces}).
    ///
    /// For more information on the signature format, see the
    /// https://eips.ethereum.org/EIPS/eip-2612#specification[relevant EIP section].
    function permit(
        address owner,
        address spender,
        uint256 value,
        uint256 deadline,
        uint8 v,
        bytes32 r,
        bytes32 s
    ) external;

    /// @dev Returns the current nonce for `owner`. This value must be
    /// included whenever a signature is generated for {permit}.
    ///
    /// Every successful call to {permit} increases ``owner``'s nonce by one. This
    /// prevents a signature from being used multiple times.
    function nonces(address owner) external view returns (uint256);

    /// @dev Returns the domain separator used in the encoding of the signature for {permit},
    /// as defined by {EIP712}.
    // solhint-disable-next-line func-name-mixedcase
    function DOMAIN_SEPARATOR() external view returns (bytes32);

    // ============================================================
    // Polymesh Specific Extensions
    // ============================================================

    /// @dev Mints a `value` amount of tokens to the caller's account.
    ///
    /// Returns a boolean value indicating whether the operation succeeded.
    ///
    /// Emits a {Transfer} event.
    function mint(uint256 value) external returns (bool);

    /// @dev Redeems a `value` amount of tokens from the caller's account.
    ///
    /// Returns a boolean value indicating whether the operation succeeded.
    ///
    /// Emits a {Transfer} event.
    function burn(uint256 value) external returns (bool);

    // ============================================================
    // ERC-7943
    // ============================================================

    /// @notice Emitted when tokens are taken from one address and transferred to another.
    /// @param from The address from which tokens were taken.
    /// @param to The address to which seized tokens were transferred.
    /// @param amount The amount seized.
    event ForcedTransfer(address indexed from, address indexed to, uint256 amount);

    /// @notice Emitted when `setFrozenTokens` is called, changing the frozen `amount` of tokens for `account`.
    /// @param account The address of the account whose tokens are being frozen.
    /// @param amount The amount of tokens frozen after the change.
    event Frozen(address indexed account, uint256 amount);


    /// @notice Checks if a transfer is possible according to token rules.
    /// @dev This involves compliance checks.
    /// @param from The address sending tokens.
    /// @param to The address receiving tokens.
    /// @param value The amount being transferred.
    /// @return True if the transfer is allowed, false otherwise.
    function canTransfer(address from, address to, uint256 value) external view returns (bool);

    /// @notice Takes tokens from one address and transfers them to another.
    /// @dev Requires specific authorization. Used for regulatory compliance or recovery scenarios.
    /// @param from The address from which `amount` is taken.
    /// @param to The address which receives the seized tokens.
    /// @param amount The amount to force transfer.
    /// @return True if the transfer executed correctly. Reverts on failure.
    function forcedTransfer(address from, address to, uint256 amount) external returns (bool);

    /// @notice Changes the frozen status of `amount` tokens belonging to `account`.
    /// @dev Overwrites the current value, similar to an `approve` function.
    /// Requires specific authorization. Frozen tokens cannot be transferred by the account.
    /// @param account The address of the account whose tokens are to be frozen.
    /// @param amount The amount of tokens to freeze. It can be greater than the account balance.
    /// @return True if the freezing executed correctly. Reverts on failure.
    function setFrozenTokens(address account, uint256 amount) external returns (bool);

    /// @notice Checks the frozen status/amount.
    /// @param account The address of the account.
    /// @dev It could return an amount higher than the account's balance.
    /// @return The amount of tokens currently frozen for `account`.
    function getFrozenTokens(address account) external view returns (uint256);

    /// @notice Checks if a specific account is allowed to send tokens according to token rules.
    /// @dev This is often used for allowlist/KYC/KYB/AML checks.
    /// @param account The address to check.
    /// @return True if the account is allowed to send, false otherwise.
    function canSend(address account) external view returns (bool);

    /// @notice Checks if a specific account is allowed to receive tokens according to token rules.
    /// @dev This is often used for allowlist/KYC/KYB/AML checks.
    /// @param account The address to check.
    /// @return True if the account is allowed to receive, false otherwise.
    function canReceive(address account) external view returns (bool);

    // ============================================================
    // ERC-3643
    // ============================================================

    /// @notice Emitted when the token contract is paused.
    event Paused(address userAddress);

    /// @notice Emitted when the token contract is unpaused.
    event Unpaused(address userAddress);

    /// @notice Emitted when the account of an investor is frozen or unfrozen.
    event AddressFrozen(address indexed account, bool freeze, address indexed owner);

    /// @notice Sets the token name. Only the owner of the token contract can call this function.
    function setName(string calldata name) external;

    /// @notice Sets the token symbol. Only the owner of the token contract can call this function.
    function setSymbol(string calldata symbol) external;

    /// @notice Pauses the token contract, preventing token transfers. Only an agent of the token can call this function.
    function pause() external;

    /// @notice Unpauses the token contract, allowing token transfers. Only an agent of the token can call this function.
    function unpause() external;

    /// @notice Sets the frozen status of a specific address. Only an agent of the token can call this function.
    function setAddressFrozen(address account, bool freeze) external;
}

contract FungibleAssetStub is IFungibleAsset {

    error NotExecutable();

    function totalSupply() external view override returns (uint256) {
        revert NotExecutable();
    }

    function balanceOf(address account) external view override returns (uint256) {
        account;
        revert NotExecutable();
    }

    function transfer(address to, uint256 value) external override returns (bool) {
        to;
        value;
        revert NotExecutable();
    }

    function allowance(address owner, address spender) external view override returns (uint256) {
        owner;
        spender;
        revert NotExecutable();
    }

    function approve(address spender, uint256 value) external override returns (bool) {
        spender;
        value;
        revert NotExecutable();
    }

    function transferFrom(address from, address to, uint256 value) external override returns (bool) {
        from;
        to;
        value;
        revert NotExecutable();
    }

    function name() external view override returns (string memory) {
        revert NotExecutable();
    }

    function symbol() external view override returns (string memory) {
        revert NotExecutable();
    }

    function decimals() external view override returns (uint8) {
        revert NotExecutable();
    }

    function permit(
        address owner,
        address spender,
        uint256 value,
        uint256 deadline,
        uint8 v,
        bytes32 r,
        bytes32 s
    ) external override {
        owner;
        spender;
        value;
        deadline;
        v;
        r;
        s;
        revert NotExecutable();
    }

    function nonces(address owner) external view override returns (uint256) {
        owner;
        revert NotExecutable();
    }

    // solhint-disable-next-line func-name-mixedcase
    function DOMAIN_SEPARATOR() external view override returns (bytes32) {
        revert NotExecutable();
    }

    function mint(uint256 value) external override returns (bool) {
        value;
        revert NotExecutable();
    }

    function burn(uint256 value) external override returns (bool) {
        value;
        revert NotExecutable();
    }

    function canTransfer(
        address from,
        address to,
        uint256 value
    ) external view override returns (bool) {
        from;
        to;
        value;
        revert NotExecutable();
    }

    function forcedTransfer(address from, address to, uint256 amount) external override returns (bool) {
        from;
        to;
        amount;
        revert NotExecutable();
    }

    function setFrozenTokens(address account, uint256 amount) external override returns (bool) {
        account;
        amount;
        revert NotExecutable();
    }

    function getFrozenTokens(address account) external view override returns (uint256) {
        account;
        revert NotExecutable();
    }

    function canSend(address account) external view override returns (bool) {
        account;
        revert NotExecutable();
    }

    function canReceive(address account) external view override returns (bool) {
        account;
        revert NotExecutable();
    }

    /// @notice Sets the token name. Only the owner of the token contract can call this function.
    function setName(string calldata name) external override {
        name;
        revert NotExecutable();
    }

    /// @notice Sets the token symbol. Only the owner of the token contract can call this function.
    function setSymbol(string calldata symbol) external override {
        symbol;
        revert NotExecutable();
    }

    /// @notice Pauses the token contract, preventing token transfers. Only an agent of the token can call this function.
    function pause() external override {
        revert NotExecutable();
    }

    /// @notice Unpauses the token contract, allowing token transfers. Only an agent of the token can call this function.
    function unpause() external override {
        revert NotExecutable();
    }

    /// @notice Sets the frozen status of a specific address. Only an agent of the token can call this function.
    function setAddressFrozen(address userAddress, bool freeze) external override {
        userAddress;
        freeze;
        revert NotExecutable();
    }
}
