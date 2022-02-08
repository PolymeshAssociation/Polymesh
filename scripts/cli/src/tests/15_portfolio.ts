import {
  initMain,
  transferAmount,
  keyToIdentityIds,
  generateEntityFromUri,
  padTicker,
  sendTx,
  ApiSingleton,
} from "../util/init";
import { createIdentities } from "../helpers/identity_helper";
import { issueTokenToDid } from "../helpers/asset_helper";
import { distributePolyBatch } from "../helpers/poly_helper";
import PrettyError from "pretty-error";
import { addComplianceRequirement } from "../helpers/compliance_manager_helper";
import { createTable } from "../util/sqlite3";
import {
  forceNewEra,
  unbond,
  nominate,
  checkEraElectionClosed,
} from "../helpers/staking_helper";

async function main(): Promise<void> {
  createTable();
  const api = await ApiSingleton.getInstance();
  const testEntities = await initMain();
  const alice = testEntities[0];
  const bob = testEntities[4];
  await keyToIdentityIds(alice.publicKey);
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

  await forceNewEra(alice);
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
  )[0][1].unwrap().authId;
  await sendTx(dave2, api.tx.identity.joinIdentityAsKey(authorization));

  console.log("Portfolio: TrustedDefaultClaimIssuerAdded");
  const ticker = padTicker("15TICKER");
  await issueTokenToDid(dave, ticker, 100000, null);
  await addComplianceRequirement(dave, ticker);
  await sendTx(
    dave,
    api.tx.complianceManager.addDefaultTrustedClaimIssuer(ticker, {
      issuer: bobDid,
      trustedFor: { Any: null },
    })
  );

  console.log("Portfolio: ClaimAdded");
  await sendTx(
    bob,
    api.tx.identity.addClaim(daveDid, { Accredited: { Ticker: ticker } }, null)
  );

  console.log("Portfolio: ConfigureAdvancedTokenRules");
  await sendTx(
    dave,
    api.tx.statistics.addTransferManager(ticker, { CountTransferManager: 10 })
  );

  console.log("Portfolio: PortfolioCreated");
  await sendTx(dave, api.tx.portfolio.createPortfolio("foobar"));

  console.log("Portfolio: AddAssetToAPortfolio");
  const portfolioId = await api.query.portfolio.nameToNumber(daveDid, "foobar");

  await sendTx(
    dave,
    api.tx.portfolio.movePortfolioFunds(
      { did: daveDid, kind: { Default: null } },
      { did: daveDid, kind: { User: portfolioId } },
      [{ amount: 10, ticker }]
    )
  );

  console.log(
    `Election Status: ${await api.query.staking.eraElectionStatus()}`
  );
  await checkEraElectionClosed();
  console.log(
    `Election Status: ${await api.query.staking.eraElectionStatus()}`
  );
  // AddAPortfolioManager is not possible because of old permission format
  console.log("Portfolio: StopStakingAPortion");
  await unbond(dave, 100);

  console.log("Portfolio: StartStakingANewOperator");
  await nominate(dave, alice.publicKey);
}

main()
  .catch((err: any) => {
    const pe = new PrettyError();
    console.error(pe.render(err));
    process.exit(1);
  })
  .finally(() => {
    console.log("Completed: Portfolio Test");
    process.exit();
  });
