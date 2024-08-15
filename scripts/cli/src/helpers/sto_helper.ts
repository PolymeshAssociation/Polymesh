import type { KeyringPair } from "@polkadot/keyring/types";
import type { Option, u64, u128 } from '@polkadot/types-codec';
import type { Ticker, PriceTier, PortfolioId } from "../types";
import { sendTx, ApiSingleton } from "../util/init";

export async function createFundraiser(
  signer: KeyringPair,
  offeringPortfolio: PortfolioId,
  offeringAsset: Ticker,
  raisingPortfolio: PortfolioId,
  raisingAsset: Ticker,
  tiers: PriceTier[],
  venueCounter: number,
  start: Option<u64> | null,
  end: Option<u64> | null,
  minimumInvestment: number,
  fundraiserName: string
) {
  const api = await ApiSingleton.getInstance();
  const transaction = api.tx.sto.createFundraiser(
    offeringPortfolio,
    offeringAsset,
    raisingPortfolio,
    raisingAsset,
    tiers,
    venueCounter,
    start,
    end,
    minimumInvestment,
    fundraiserName
  );
  await sendTx(signer, transaction);
}

export async function freezeFundraiser(
  signer: KeyringPair,
  offeringAsset: Ticker,
  fundraiserId: number
) {
  const api = await ApiSingleton.getInstance();
  const transaction = api.tx.sto.freezeFundraiser(offeringAsset, fundraiserId);
  await sendTx(signer, transaction);
}

export async function unfreezeFundraiser(
  signer: KeyringPair,
  offeringAsset: Ticker,
  fundraiserId: number
) {
  const api = await ApiSingleton.getInstance();
  const transaction = api.tx.sto.unfreezeFundraiser(
    offeringAsset,
    fundraiserId
  );
  await sendTx(signer, transaction);
}

export async function modifyFundraiserWindow(
  signer: KeyringPair,
  offeringAsset: Ticker,
  fundraiserId: number,
  start: number,
  end: Option<u64> | null,
) {
  const api = await ApiSingleton.getInstance();
  const transaction = api.tx.sto.modifyFundraiserWindow(
    offeringAsset,
    fundraiserId,
    start,
    end
  );
  await sendTx(signer, transaction);
}

export async function stop(
  signer: KeyringPair,
  offeringAsset: Ticker,
  fundraiserId: number
) {
  const api = await ApiSingleton.getInstance();
  const transaction = api.tx.sto.stop(offeringAsset, fundraiserId);
  await sendTx(signer, transaction);
}

export async function invest(
  signer: KeyringPair,
  investmentPortfolio: PortfolioId,
  fundingPortfolio: PortfolioId,
  offeringAsset: Ticker,
  fundraiserId: number,
  purchaseAmount: number,
  maxPrice: Option<u128> | number,
  receipt: string | null
) {
  const api = await ApiSingleton.getInstance();

  const transaction = api.tx.sto.invest(
    investmentPortfolio,
    fundingPortfolio,
    offeringAsset,
    fundraiserId,
    purchaseAmount,
    maxPrice,
    receipt
  );
  await sendTx(signer, transaction);
}
