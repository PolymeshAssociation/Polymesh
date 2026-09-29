# Tehran Bourse On-Chain (Polymesh local testnet) — Run Guide

A permissioned stock-exchange simulator on a local Polymesh (Substrate) chain.

Components:
- Polymesh node            -> ws://127.0.0.1:9945
- Sejam (KYC) simulator    -> http://localhost:3001
- Brokerage API            -> http://localhost:3002
- Next.js web UI           -> http://localhost:3000

## Prerequisites
- Rust toolchain (building the node takes ~30-60 min)
- Node.js 20+ and npm

## 1) Build & run the node
    cargo build --release
    ./target/release/polymesh --chain tehran-raw.json --base-path ./tehran-test-data \
      --rpc-port 9945 --port 30334 --alice \
      --node-key 8b14779b4187c71bf6107cb053138f4fe9fd83ae9daf0b2e3f161e116be45fe6
Wait until you see "Imported #..." blocks.

## 2) Bootstrap chain state (asset + compliance + test accounts)
    cd sejam-simulator
    npm install
    SETUP_CHAIN_URL=ws://127.0.0.1:9945 npx tsx src/setup-chain.ts
    node src/setup-trusted-issuer-final.cjs
    node src/setup-dev-accounts-v2.cjs

NOTE: If the created asset id is NOT 0x1fb383fc367588cb8de25ca5474caa22,
replace it everywhere with the new id:
    grep -rl '0x1fb383fc367588cb8de25ca5474caa22' . | xargs sed -i 's/0x1fb383fc367588cb8de25ca5474caa22/NEW_ASSET_ID/g'

## 3) Run the three services (3 separate terminals)
    cd sejam-simulator && npx tsx src/sejam-server.ts       # port 3001
    cd sejam-simulator && npx tsx src/brokerage-server.ts    # port 3002
    cd tehran-bourse-ui && npm install && npm run dev        # port 3000

## 4) Pages
- http://localhost:3000            landing + connect
- http://localhost:3000/onboarding register via Sejam simulator
- http://localhost:3000/dashboard  view identity/balances by DID
- http://localhost:3000/trade      transfer shares (Compliance enforced)
- http://localhost:3000/accounts   dev accounts book (all DIDs)

## Deterministic test accounts (no mnemonic needed)
- Alice (issuer)   : //Alice
- Reza (investor)  : //TehranInv//1
- Sara (investor)  : //TehranInv//2
- NoCdd (rejected) : //TehranInv//3
