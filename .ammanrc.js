const { LOCALHOST } = require('@metaplex-foundation/amman')
const path = require('path')

const ledgerDir = path.join(__dirname, 'ledger')
const NO_GEYSER = process.env.NO_GEYSER != null
const geyserPluginConfig = NO_GEYSER
  ? undefined
  : path.join(__dirname, 'config.json')

const FOO_PROGRAM = '7w4ooixh9TFgfmcCUsDJzHd9QqDKyxz4Mq1Bke6PVXaY'
const fooBinary = path.joins(__dirname, 'fixtures', 'anchor-osx-prog.so')

module.exports = {
  validator: {
    killRunningValidators: true,
    programs: [
      {
        label: 'Foo',
        programId: FOO_PROGRAM,
        deployPath: fooBinary,
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
