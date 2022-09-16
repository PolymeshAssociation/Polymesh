import {
  initMain,
  transferAmount,
  keyToIdentityIds,
  generateEntityFromUri,
  padTicker,
  ApiSingleton,
  disconnect,
} from "../util/init";
import {
  createIdentities,
  addClaim,
  joinIdentityAsKey,
  addAuthorization,
} from "../helpers/identity_helper";
import { issueTokenToDid } from "../helpers/asset_helper";
import { distributePolyBatch } from "../helpers/poly_helper";
import {
  addComplianceRequirement,
  addDefaultTrustedClaimIssuer,
} from "../helpers/compliance_manager_helper";
import {
  createPortfolio,
  movePortfolioFunds,
} from "../helpers/portfolio_helper";
import {
  forceNewEra,
  unbond,
  nominate,
  checkEraElectionClosed,
  bond,
} from "../helpers/staking_helper";
import { setActiveAssetStats, setAssetTransferCompliance } from "../helpers/statistics_helper";

// Disconnects api after all the tests have completed
afterAll(async () => {
  await disconnect();
});

describe("15 - Portfolio Unit Test", () => {
  test("Portfolio", async () => {
    const api = await ApiSingleton.getInstance();
    const testEntities = await initMain();
    const alice = testEntities[0];
    const bob = testEntities[4];
    await keyToIdentityIds(alice.publicKey);
    const bobDid = await keyToIdentityIds(bob.publicKey);
    const dave = await generateEntityFromUri("15_dave");
    const dave2 = await generateEntityFromUri("15_dave2");
    const [daveDid] = await createIdentities(alice, [dave]);
    expect(daveDid).toBeTruthy();
    console.log("Identities Created");
    await expect(
      distributePolyBatch(alice, [bob, dave], transferAmount * 10)
    ).resolves.not.toThrow();

    // Staking
    await bond(dave, dave, 1_000_000, "Staked");
    await nominate(dave, bob.publicKey);
    console.log("Nominated validators");

    await forceNewEra(alice);
    console.log("New era, rewards paid out");

    // SecondaryKey
    await expect(
      addAuthorization(
        dave,
        { Account: dave2.publicKey },
        { JoinIdentity: { Whole: null } },
        null
      )
    ).resolves.not.toThrow();
    console.log("Added JoinIdentity authorization");
    const authorization = (
      await api.query.identity.authorizations.entries({
        Account: dave2.publicKey,
      })
    )[0][1];
    await joinIdentityAsKey(dave2, authorization.unwrap().authId);

    console.log("Portfolio: TrustedDefaultClaimIssuerAdded");
    const ticker = padTicker("15TICKER");
    await expect(
      issueTokenToDid(dave, ticker, 100000, null)
    ).resolves.not.toThrow();
    await expect(addComplianceRequirement(dave, ticker)).resolves.not.toThrow();
    await expect(
      addDefaultTrustedClaimIssuer(dave, ticker, bobDid, { Any: null })
    ).resolves.not.toThrow();

    console.log("Portfolio: ClaimAdded");
    await expect(
      addClaim(bob, daveDid, { Accredited: { Ticker: ticker } }, null)
    ).resolves.not.toThrow();

    console.log("Portfolio: Setup investor count stats and MaxInvestorCount transfer conditions.");
    await expect(
      setActiveAssetStats(dave, ticker, [{ op: { Count: null }, claim_issuer: null }])
    ).resolves.not.toThrow();
    await expect(
      setAssetTransferCompliance(dave, ticker, [{ MaxInvestorCount: 10 }])
    ).resolves.not.toThrow();

    console.log("Portfolio: PortfolioCreated");
    await expect(createPortfolio(dave, "foobar")).resolves.not.toThrow();

    console.log("Portfolio: AddAssetToAPortfolio");
    await expect(
      movePortfolioFunds(dave, dave, ticker, 10)
    ).resolves.not.toThrow();

    console.log(
      `Election Status: ${await api.query.staking.eraElectionStatus()}`
    );
    await checkEraElectionClosed();
    console.log(
      `Election Status: ${await api.query.staking.eraElectionStatus()}`
    );
    // Bound some POLYX.
    console.log("Portfolio: StopStakingAPortion");
    await expect(unbond(dave, 100)).resolves.not.toThrow();

    // Nominate Alice.
    console.log("Portfolio: StartStakingANewOperator");
    await checkEraElectionClosed();
    await expect(nominate(dave, alice.publicKey)).resolves.not.toThrow();
  });
});
