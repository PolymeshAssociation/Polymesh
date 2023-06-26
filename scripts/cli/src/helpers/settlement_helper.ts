import type { KeyringPair } from "@polkadot/keyring/types";
import type { PortfolioId, Ticker, venueType } from "../types";
import type { IdentityId } from "../interfaces";
import { sendTx, getDefaultPortfolio, ApiSingleton } from "../util/init";

/**
 * @description Creates a Venue
 */
export async function createVenue(signer: KeyringPair, type: venueType): Promise<number> {
  const api = await ApiSingleton.getInstance();
  let venueCounter = (await api.query.settlement.venueCounter()).toNumber();
  let venueDetails = "created venue";
  const transaction = api.tx.settlement.createVenue(
    venueDetails,
    [signer.address],
    type
  );
  await sendTx(signer, transaction);
  return venueCounter;
}

/**
 * @description Adds an Instruction
 */
export async function addInstruction(
  signer: KeyringPair,
  venueCounter: number,
  signerDid: IdentityId,
  receiverDid: IdentityId,
  ticker: Ticker,
  amount: number
): Promise<number> {
  const api = await ApiSingleton.getInstance();
  let instructionCounter = (
    await api.query.settlement.instructionCounter()
  ).toNumber();

  let leg = {
    "Fungible": {
      sender: getDefaultPortfolio(signerDid),
      receiver: getDefaultPortfolio(receiverDid),
      ticker: ticker,
      amount: amount
    }
  };

  const transaction = api.tx.settlement.addInstruction(
    venueCounter,
    { SettleOnAffirmation: "" },
    null,
    null,
    [leg],
    null
  );
  await sendTx(signer, transaction);

  return instructionCounter;
}

/**
 * @description Affirms an Instruction
 */
export async function affirmInstruction(
  signer: KeyringPair,
  instructionCounter: number,
  did: IdentityId,
  legCounter: number
): Promise<void> {
  const api = await ApiSingleton.getInstance();
  const transaction = api.tx.settlement.affirmInstruction(
    instructionCounter,
    [getDefaultPortfolio(did)],
  );
  await sendTx(signer, transaction);
}

/**
 * @description Withdraws a Instruction
 */
export async function withdrawInstruction(
  signer: KeyringPair,
  instructionCounter: number,
  did: IdentityId
): Promise<void> {
  const api = await ApiSingleton.getInstance();
  const transaction = api.tx.settlement.withdrawInstruction(
    instructionCounter,
    [getDefaultPortfolio(did)]
  );
  await sendTx(signer, transaction);
}

/**
 * @description Rejects a Instruction
 */
 export async function rejectInstruction(
  signer: KeyringPair,
  instructionCounter: number,
  portfolioId: PortfolioId,
  numOfLegs: number,
  did: IdentityId
): Promise<void> {
  const api = await ApiSingleton.getInstance();
  const transaction = api.tx.settlement.rejectInstruction(instructionCounter, portfolioId);
  await sendTx(signer, transaction);
}

/**
 * @description Adds a group Instruction
 */
export async function addGroupInstruction(
  signer: KeyringPair,
  venueCounter: number,
  group: IdentityId[],
  ticker: Ticker,
  ticker2: Ticker,
  amount: number
): Promise<number> {
  const api = await ApiSingleton.getInstance();
  let instructionCounter = (
    await api.query.settlement.instructionCounter()
  ).toNumber();

  let leg = {
    "Fungible": {
      sender: group[1],
      receiver: group[0],
      ticker: ticker2,
      amount: amount
    }
  };


  let leg2 = {
    "Fungible": {
      sender: group[0],
      receiver: group[1],
      ticker: ticker,
      amount: amount
    }
  };

  let leg3 = {
    "Fungible": {
      sender: group[0],
      receiver: group[2],
      ticker: ticker,
      amount: amount
    }
  };

  let leg4 = {
    "Fungible": {
      sender: group[0],
      receiver: group[3],
      ticker: ticker,
      amount: amount
    }
  };

  let leg5 = {
    "Fungible": {
      sender: group[0],
      receiver: group[4],
      ticker: ticker,
      amount: amount
    }
  };

  const transaction = api.tx.settlement.addInstruction(
    venueCounter,
    { SettleOnAffirmation: "" },
    null,
    null,
    [leg, leg2, leg3, leg4, leg5],
    null
  );

  await sendTx(signer, transaction);
  return instructionCounter;
}

/**
 * @description Creates a Claim Receipt
 * @param {KeyringPair} signer - KeyringPair
 * @param {IdentityId} signerDid - IdentityId
 * @param {IdentityId} receiverDid - IdentityId
 * @param {Ticker} ticker - Ticker
 * @param {Ticker} amount - number
 * @param {number} instructionCounter - number
 * @return {Promise<void>}
 */
async function claimReceipt(
  signer: KeyringPair,
  signerDid: IdentityId,
  receiverDid: IdentityId,
  ticker: Ticker,
  amount: number,
  instructionCounter: number
): Promise<void> {
  const api = await ApiSingleton.getInstance();

  let msg = {
    receipt_uid: 0,
    from: signerDid,
    to: receiverDid,
    asset: ticker,
    amount: amount,
  };

  let receiptDetails = {
    receipt_uid: 0,
    leg_id: 0,
    signer: signer.address,
    signature: 1,
  };

  const transaction = api.tx.settlement.claimReceipt(
    instructionCounter,
    receiptDetails
  );
  await sendTx(signer, transaction);
}
