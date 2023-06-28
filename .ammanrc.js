const { LOCALHOST } = require('@metaplex-foundation/amman')
const path = require('path')

const ledgerDir = path.join(__dirname, 'ledger')

module.exports = {
  validator: {
    killRunningValidators: true,
    programs: [],
    jsonRpcUrl: LOCALHOST,
    websocketUrl: '',
    commitment: 'confirmed',
    ledgerDir,
    resetLedger: true,
    verifyFees: false,
    detached: process.env.CI != null,
    limitLedgerSize: 1,
  },
  relay: {
    enabled: process.env.CI == null,
    killlRunningRelay: true,
  },
  storage: {
    enabled: false,
  },
}
