// src/index.ts
import 'dotenv/config';
import { ApiPromise, WsProvider } from '@polkadot/api';

async function main() {
  console.log('🔄 Attempting to connect to Polymesh node...');
  
  try {
    const provider = new WsProvider(process.env.POLKADOT_WS_URL || 'ws://127.0.0.1:9944');
    const api = await ApiPromise.create({ provider });
    
    console.log('✅ Connected successfully!');
    
    // خواندن موجودی Alice (برای تست)
    // توجه: اینجا آدرس عمومی Alice را وارد کن، نه DID
    const aliceAddress = '5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY'; 
    const { data: { free } } = await api.query.system.account(aliceAddress);
    
    console.log(`💰 Alice Free Balance: ${free.toHuman()}`);
    
    process.exit(0);
  } catch (error) {
    console.error('❌ Connection failed:', error);
    process.exit(1);
  }
}

main();