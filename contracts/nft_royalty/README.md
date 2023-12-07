Example contract for enforcing NFT royalty for secondary NFT sales.

## Build

Install [`cargo-contract`](https://github.com/paritytech/cargo-contract).
```
cargo install cargo-contract --force
```

Build the contract:
`cargo contract build --release`

Contract file needed for deployment `./target/ink/settlements.contract`.

## Deployment and setup.


## Creating a Portofolio for Receiving Royalty

Each artist should have their own portfolio for receiving royalty payments. The artist (or the account that will handle their portfolio) must call the `create_custody_portfolio` method passing in a `PortfolioName` for identifying the portfolio. This will call the `create_custody_portfolio` extrinsic from the `portfolio` pallet, which creates a portfolio owned by the artist, but transfers its custody to the contract. 

## Creating an NFT Transfer

In order to create an NFT transfer, the `create_transfer` method must be called. This method needs two parameters: the details of the NFT transfer and the NFT offer details. This will call the `add_and_affirm_instruction` extrinsic from the `settlement` pallet, and will create an instruction containg three legs. One leg where `NFTTransferDetails::nft_owner_portfolio` is transferring `NFTTransferDetails::nft_id` to `NFTTransferDetails::nft_receiver_portfolio`, another leg where `NFTOffer::payer_portfolio` sends `NFTOffer::transfer_price` to `NFTOffer::receiver_portfolio`, and one leg where the payer is transferring the royalty to the artist.

```Rust
/// The details of an NFT transfer.
pub struct NFTTransferDetails {
    /// The [`Ticker`] of the NFT collection.
    pub collection_ticker: Ticker,
    /// The [`NFTId`] of the non-fungible token being transferred.
    pub nft_id: NFTId,
    /// The [`PortfolioId`] that contains the NFT being sold.
    pub nft_owner_portfolio: PortfolioId,
    /// The [`PortfolioId`] that will receive the NFT.
    pub nft_receiver_portfolio: PortfolioId,
}

/// The details of the proposed offer in exchange for the NFT.
pub struct NFTOffer {
    /// The [`Ticker`] of the asset being used for buying the NFT.
    pub purchase_ticker: Ticker,
    /// The price the buyer is paying for the NFT.
    pub transfer_price: Balance,
    /// The [`PortfolioId`] that is paying for the NFT.
    pub payer_portfolio: PortfolioId,
    /// The [`PortfolioId`] that is receiving the payment for the NFT.
    pub receiver_portfolio: PortfolioId,
}
```

