Example contract for enforcing NFT royalty for secondary NFT sales in a P2P environment.

## Build

Install [`cargo-contract`](https://github.com/paritytech/cargo-contract).
```
cargo install cargo-contract --force
```

Build the contract:
`cargo contract build --release`

## Polymesh App Deployment 

1. Go to the developer page and select the contracts tab;
2. Drag and drop the `/target/ink/nft_royalty.contract` file;
3. Use the `new` constructor to create an instance of `NftRoyalty`;

## Mandatory Metadata

After creating the asset for the NFT collection, the artist must set the metadata containing the information for the royalty. This can be done in the `asset` pallet calling the `register_and_set_local_asset_metadata` extrinsic. Currently, the expected metadata contains the following data: 

```Rust
/// All mandatoty information NFT artists must set as metadata.
pub struct NFTArtistRules {
    /// All [`Ticker`] the artist is willing to receive as royalty.
    pub allowed_purchase_tickers: BTreeSet<Ticker>,
    /// The royalty percentage the artist will receive for each transfer.
    pub royalty_percentage: Perbill,
    /// The identity that will receive royalty payments.
    pub artist_identity: IdentityId,
}
```

## Contract Initialization

Make sure to call `initialize_contract` after deploying the contract. This will create a venue that is owned by the contract and is required for creating transfers.

## Creating a Portofolio for Receiving Royalty

Each artist should have their own portfolio for receiving royalty payments. The artist must call the `create_custody_portfolio` method passing in a `PortfolioName` for identifying the portfolio. This will call the `create_custody_portfolio` extrinsic from the `portfolio` pallet, which creates a portfolio owned by the artist, but transfers its custody to the contract. In order for the call to be successful, the artist must have authorized the contract to create portfolios on their behalf (this can be achived by calling the `allow_identity_to_create_portfolios` extrisinc in the portfolio pallet).

## Creating an NFT Transfer

In order to create an NFT transfer, the `create_transfer` method must be called. This method needs two parameters: the details of the NFT transfer and the NFT offer details. This will call the `add_and_affirm_instruction` extrinsic from the `settlement` pallet, and will create an instruction containg three legs. One leg where `NFTTransferDetails::nft_owner_portfolio` is transferring `NFTTransferDetails::nfts` to `NFTTransferDetails::nft_receiver_portfolio`, another leg where `NFTOffer::payer_portfolio` sends `NFTOffer::transfer_price` - `royalty_amount` to `NFTOffer::receiver_portfolio`, and one leg where the payer is transferring the royalty to the artist.

```Rust
/// The details of an NFT transfer.
pub struct NFTTransferDetails {
    /// The [`Ticker`] of the NFT collection.
    pub collection_ticker: Ticker,
    /// All NFTs being transferred.
    pub nfts: Vec<NFTId>,
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
