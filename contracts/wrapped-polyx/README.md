Example contract for wrapping and unwrapping POLYX as an asset

## Build

Install [`cargo-contract`](https://github.com/paritytech/cargo-contract).
```
cargo install cargo-contract --force
```

Build the contract:
`cargo +nightly contract build --release`

Contract file needed for deployment `./target/ink/wrapped_polyx.contract`.

## Deployment and Setup

Needed:
* One identities for the contract `CONTRACT_DID`.
* An un-registered ticker `TICKER`.

1. Upload and deploy the contract file `wrapped_polyx.contract` from a key of the `CONTRACT_DID`.
2. For deployment use the `new(TICKER)` contructor with the un-registered ticker.
3. Use the primary key of `CONTRACT_DID` to give the contract the permissions by calling `identity.setSecondaryKeyPermissions(contract_address, { asset: Whole, extrinsic: Whole, portfolio: Whole })`. NB - these permissions could be more restricted.
4. Call the `init()` method on the contract passing in at least 3,000 POLYX.  This will create the wrapped asset with a ticker of `TICKER`.

## Usage

Needed:
* One user identity (this must be different from the contract's identity).
* One non-default portfolio under the users identity.

1. Create a portfolio using the user's key: `Custodian` (The steps below need the `PorfolioNumber` for this portfolio).
2. Get the contract's identity DID by reading the `contractDid()` method (This is read-only and has no transaction fee).
3. Setup an authorization for the contract to be custodian of portfolio `Custodian` by calling: `identity.addAuthoration(target: Identity(CONTRACT_DID), data: PortfolioCustody(user_did, User(Custodian)))`
4. Get the `auth_id` from step #3 and call contract method `addPortfolio(auth_id, User(Custodian))`

Now the portfolio `Custodian` with be controlled by the contract.

## Wrapping

A user can call contract method `mintWrappedPolyx()`. This method is payable, and by passing in POLYX to the method, the user will receive a corresponding amount of the wrapped POLYX asset (`TICKER`) back into their `Custodian` portfolio. The POLYX sent to the contract will be stored in the contract for later redemptions.

To withdraw the wrapped POLYX from the `Custodian` portfolio to a portfolio that the user directly controls, the user can call `withdrawPolyx` specifying the amount of wrapped POLYX assets (`TICKER`) they wish to move, and the destination portfolio.

Alternatively the user can call `removePortfolio` to remove the contracts control over the `Custodian` portfolio, and hence regain direct asset to any wrapped POLYX assets (`TICKER`) remaining in the portfolio.

## Unwrapping

The user can call `burnWrappedPolyx` specifying the amount of wrapped POLYX (`TICKER`) they wish to redeem back to the native POLYX token. The user must have this amount of wrapped POLYX assets in their `Custodian` portfolio. The contract will transfer these assets to its default portfolio, then redeem (burn) these assets and transfer an equivalent amount of POLYX tokens back to the callers key.