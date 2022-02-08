import { reqImports } from "./init.mjs";

async function main() {
  try {
  await reqImports.createApi();
  }
  catch(err) {
    console.log(err);
    console.log("ErrorOccurred");
    process.exitCode = 1;
  }
  process.exit();
}

main().catch(console.error);
