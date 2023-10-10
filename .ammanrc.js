const { LOCALHOST } = require('@metaplex-foundation/amman')
const path = require('path')

const ledgerDir = path.join(__dirname, 'ledger')
const NO_GEYSER = process.env.NO_GEYSER != null
const geyserPluginConfig = NO_GEYSER
  ? undefined
  : path.join(__dirname, 'config-dev.json')

const DEVNET = process.env.IF_HELIUS_DEV ?? 'https://api.devnet.solana.com'
const SOLANAX_PROGRAM = 'SoLXmnP9JvL6vJ7TN1VqtTxqsc2izmPfF9CsMDEuRzJ'

module.exports = {
  validator: {
    killRunningValidators: true,
    accountsCluster: DEVNET,
    accounts: [
      {
        label: 'Solanax Program',
        accountId: SOLANAX_PROGRAM,
        executable: true,
      },
    ],
    jsonRpcUrl: LOCALHOST,
    websocketUrl: '',
    commitment: 'confirmed',
    ledgerDir,
    resetLedger: true,
    verifyFees: false,
    detached: process.env.CI != null,
    limitLedgerSize: 100000,
    geyserPluginConfig,
  },
  relay: {
    enabled: process.env.CI == null,
    killlRunningRelay: true,
  },
  storage: {
    enabled: false,
  },
}
