import * as init from "../util/init";
import {
  createIdentities,
  authorizeJoinToIdentities,
  setSecondaryKeyPermissions,
} from "../helpers/identity_helper";
import { distributePolyBatch } from "../helpers/poly_helper";
import { addSecondaryKeys } from "../helpers/key_management_helper";
import {
  setAsset,
  setExtrinsic,
  setPortfolio,
  setDoc,
} from "../helpers/permission_helper";
import {
  createPortfolio,
  movePortfolioFunds,
} from "../helpers/portfolio_helper";
import { addDocuments, issueTokenToDid } from "../helpers/asset_helper";
import {
  AssetPermissions,
  Document,
  ExtrinsicPermissions,
  PortfolioPermissions,
} from "../types";

// Disconnects api after all the tests have completed
afterAll(async () => {
  await init.disconnect();
});

describe("4 - Permission Management Unit Test", () => {
  test("Adding Permissions", async () => {
    const ticker = init.padTicker("4TICKER");
    const portfolioName = "4_portfolio";
    const testEntities = await init.initMain();
    const alice = testEntities[0];
    const primaryDevSeed = "4_primary";
    const secondaryDevSeed = "4_secondary";
    const primaryKeys = await init.generateKeys(1, primaryDevSeed);
    const secondaryKeys = await init.generateKeys(1, secondaryDevSeed);
    const dids = await createIdentities(alice, primaryKeys);
    expect(dids).toBeTruthy();
    let extrinsics: ExtrinsicPermissions = { These: [] };
    let portfolios: PortfolioPermissions = { These: [] };
    let assets: AssetPermissions = { These: [] };
    let documents: Document[] = [];

    await expect(
      distributePolyBatch(alice, [primaryKeys[0]], init.transferAmount * 2)
    ).resolves.not.toThrow();
    await expect(
      addSecondaryKeys(primaryKeys, secondaryKeys)
    ).resolves.not.toThrow();
    await expect(
      authorizeJoinToIdentities(secondaryKeys, primaryKeys)
    ).resolves.not.toThrow();
    await expect(
      distributePolyBatch(primaryKeys[0], [secondaryKeys[0]], init.transferAmount)
    ).resolves.not.toThrow();
    await expect(
      issueTokenToDid(primaryKeys[0], ticker, 1000000, null)
    ).resolves.not.toThrow();

    let portfolioOutput = await createPortfolio(
      secondaryKeys[0],
      portfolioName
    );
    expect(portfolioOutput).toBeFalsy();

    setExtrinsic(extrinsics, "Portfolio", "create_portfolio");
    await setSecondaryKeyPermissions(
      primaryKeys,
      secondaryKeys,
      extrinsics,
      portfolios,
      assets
    );
    portfolioOutput = await createPortfolio(secondaryKeys[0], portfolioName);
    expect(portfolioOutput).toBeTruthy();

    setExtrinsic(extrinsics, "Portfolio", "move_portfolio_funds");
    await setSecondaryKeyPermissions(
      primaryKeys,
      secondaryKeys,
      extrinsics,
      portfolios,
      assets
    );
    let portfolioFundsOutput = await movePortfolioFunds(
      secondaryKeys[0],
      primaryKeys[0],
      ticker,
      100
    );
    expect(portfolioFundsOutput).toBeFalsy();

    await setPortfolio(portfolios, primaryKeys[0], "Default");
    await setPortfolio(portfolios, secondaryKeys[0], "User");
    await setSecondaryKeyPermissions(
      primaryKeys,
      secondaryKeys,
      extrinsics,
      portfolios,
      assets
    );

    portfolioFundsOutput = await movePortfolioFunds(
      secondaryKeys[0],
      primaryKeys[0],
      ticker,
      100
    );
    expect(portfolioFundsOutput).toBeTruthy();

    setExtrinsic(extrinsics, "Asset", "add_documents");
    await setSecondaryKeyPermissions(
      primaryKeys,
      secondaryKeys,
      extrinsics,
      portfolios,
      assets
    );
    setDoc(documents, "www.google.com", { None: "" }, "google");
    let addDocsOutput = await addDocuments(secondaryKeys[0], ticker, documents);
    expect(addDocsOutput).toBeFalsy();
    console.log("after docs");

    setAsset(ticker, assets);
    await setSecondaryKeyPermissions(
      primaryKeys,
      secondaryKeys,
      extrinsics,
      portfolios,
      assets
    );
    addDocsOutput = await addDocuments(secondaryKeys[0], ticker, documents);
    expect(addDocsOutput).toBeTruthy();
  });
});
