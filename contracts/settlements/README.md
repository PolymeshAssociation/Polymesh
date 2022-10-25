Example contract for custodianship and settlement transfers.

## Build

Install [`cargo-contract`](https://github.com/paritytech/cargo-contract).
```
cargo install cargo-contract --force
```

Build the contract:
`cargo +nightly-2022-05-10 contract build --release`

Contract file needed for deployment `./target/ink/settlements.contract`.

## Deployment and setup.

Needed:
* One identities for the contract `CONTRACT_DID`.
* Two different un-registered tickers `TICKER1` and `TICKER2`.

1. Upload and deploy the contract file `settlements.contract` from a key of the `CONTRACT_DID`.
2. For deployment use the `new(ticker1, ticker2)` contructor with the two un-registered tickers.
3. Use the primary key of `CONTRACT_DID` to give the contract the permissions by calling `identity.setSecondaryKeyPermissions(contract_address, { asset: Whole, extrinsic: Whole, portfolio: Whole })`.
4. Transfer at least 6,000 POLYX to the contract (this is needed for creating the two assets).
5. Call the `init()` method on the contract.  This will create the two assets.

## Setup an investors.

Needed:
* One investor identity (this must be different from the contract's identity).
* One non-default portfolio.  (It is best not to use the default portfolio here)

1. Create a portfolio using the investor's key: `Custodian1` (The steps below need the `PorfolioNuber` for this portfolio).
2. Get the contract's identity DID by reading the `contractDid()` method (This is read-only and has no transaction fee).
3. Setup an authorization for the contract to be custodian of portfolio `Custodian1` by calling: `identity.addAuthoration(target: Identity(CONTRACT_DID), data: PortfolioCustody(investor_did, User(Custodian1)))`
4. Get the `auth_id` from step #3 and call contract method `addPortfolio(auth_id, User(Custodian1))`

Now the portfolio `Custodian1` with be controlled by the contract and have some funds for both tickers.

## Trading

An investor can call contract method `trade(sell, sell_amount, buy, buy_amount)` trade the `sell` tokens for `buy` tokens.  Only the two contract tickers can be used.
The `trade` method will use a settlement to move `sell` tokens from the caller's portfolio (`Custodian1`) into the contract's default portfolio
and move `buy` tokens from the contract's default portfolio into the caller's portfolio.

## Deposits

The investor can deposit fund into their portfolio that the contract has custodianship of (`Custodian1`).

## Withdrawals

The investor can withdrawal funds from the contract by calling the `withdrawal(ticker, amount, destination_portfolio)` method.

The investor can also withdrawal all their funds from the contract by calling `withdrawal_all()` method.
This will remove the contract as the custodian of the investor's portfolio.

