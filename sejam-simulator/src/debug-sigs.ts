
import 'dotenv/config';
import { ApiPromise, WsProvider } from '@polkadot/api';
import { cryptoWaitReady } from '@polkadot/util-crypto';

const CHAIN_URL = process.env.SETUP_CHAIN_URL || 'ws://127.0.0.1:9945';

async function main() {
  await cryptoWaitReady();
  const api: any = await ApiPromise.create({ provider: new WsProvider(CHAIN_URL) });
  const targets = [
    ['asset', 'createAssetWithCustomType'],
    ['asset', 'createAsset'],
    ['settlement', 'createVenue'],
    ['complianceManager', 'addComplianceRequirement'],
  ];
  for (const [p, m] of targets) {
    const tx = (api.tx as any)[p]?.[m];
    if (tx) {
      const args = tx.meta.args.map((a: any) => a.name.toString() + ': ' + a.type.toString());
      console.log('
' + p + '.' + m + ' (' + args.length + ' args):');
      args.forEach((a: string, i: number) => console.log('  ' + i + '. ' + a));
    } else {
      console.log('
' + p + '.' + m + ': NOT AVAILABLE');
    }
  }
  await api.disconnect();
}
main();
