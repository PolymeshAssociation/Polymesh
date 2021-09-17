import {
  initMain,
  transferAmount,
  keyToIdentityIds,
  generateEntityFromUri,
  padTicker,
  sendTx,
  ApiSingleton,
  waitNextBlock,
  waitNextEra,
} from "../util/init";
import { createIdentities } from "../helpers/identity_helper";
import { issueTokenToDid } from "../helpers/asset_helper";
import { distributePolyBatch } from "../helpers/poly_helper";
import { ExtrinsicPermissions } from "../types";
import PrettyError from "pretty-error";
import {
  addInstruction,
  affirmInstruction,
  createVenue,
} from "../helpers/settlement_helper";
import { addComplianceRequirement } from "../helpers/compliance_manager_helper";
import { send } from "process";

async function main(): Promise<void> {
  const api = await ApiSingleton.getInstance();
  const testEntities = await initMain();
  const alice = testEntities[0];
  const bob = testEntities[4];
  const aliceDid = await keyToIdentityIds(alice.publicKey);
  const bobDid = await keyToIdentityIds(bob.publicKey);
  const dave = await generateEntityFromUri("15_dave");
  const dave2 = await generateEntityFromUri("15_dave2");
  const [daveDid] = await createIdentities(alice, [dave]);

  console.log("Identities Created");
  await distributePolyBatch(alice, [bob, dave], transferAmount * 10);

  // Staking
  await sendTx(
    dave,
    api.tx.staking.bond(dave.publicKey, 1000000, { Staked: null })
  );
  await sendTx(dave, api.tx.staking.nominate([bob.publicKey]));
  console.log("Nominated validators");

  await waitNextEra();
  console.log("New era, rewards paid out");

  // SecondaryKey
  await sendTx(
    dave,
    api.tx.identity.addAuthorization(
      { Account: dave2.publicKey },
      { JoinIdentity: { Whole: null } },
      null
    )
  );
  console.log("Added JoinIdentity authorization");
  const authorization = (
    await api.query.identity.authorizations.entries({
      Account: dave2.publicKey,
    })
  )[0][1].unwrap().auth_id;
  await sendTx(dave2, api.tx.identity.joinIdentityAsKey(authorization));

  console.log("ItnRewards: TrustedDefaultClaimIssuerAdded");
  const ticker = padTicker("15TICKER");
  issueTokenToDid(dave, ticker, 100000, null);
  await addComplianceRequirement(dave, ticker);
  await sendTx(
    dave,
    api.tx.complianceManager.addDefaultTrustedClaimIssuer(ticker, {
      issuer: bobDid,
      trusted_for: { Any: null },
    })
  );

  console.log("ItnRewards: ClaimAdded");
  await sendTx(
    bob,
    api.tx.identity.addClaim(daveDid, { Accredited: { Ticker: ticker } }, null)
  );

  console.log("ItnRewards: ConfigureAdvancedTokenRules");
  await sendTx(
    dave,
    api.tx.statistics.addTransferManager(ticker, { CountTransferManager: 10 })
  );

  console.log("ItnRewards: PortfolioCreated");
  await sendTx(dave, api.tx.portfolio.createPortfolio("foobar"));

  console.log("ItnRewards: AddAssetToAPortfolio");
  const portfolioId = await api.query.portfolio.nameToNumber(daveDid, "foobar");

  await sendTx(
    dave,
    api.tx.portfolio.movePortfolioFunds(
      { did: daveDid, kind: { Default: null } },
      { did: daveDid, kind: { User: portfolioId } },
      [{ amount: 10, ticker }]
    )
  );

  // AddAPortfolioManager is not possible because of old permission format

  console.log("ItnRewards: StopStakingAPortion");
  await sendTx(dave, api.tx.staking.unbond(100));

  console.log("ItnRewards: StartStakingANewOperator");
  await sendTx(dave, api.tx.staking.nominate([alice.publicKey]));
}

main()
  .catch((err: any) => {
    const pe = new PrettyError();
    console.error(pe.render(err));
    process.exit(1);
  })
  .finally(() => {
    console.log("Completed: ITN REWARDS");
    process.exit();
  });
