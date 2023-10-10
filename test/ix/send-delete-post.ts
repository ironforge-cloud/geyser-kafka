import test from 'node:test'
import { Processes } from './utils'
import { Tasks } from './utils'
import assert from 'assert/strict'
import spok from 'spok'

function sleep(ms: number) {
  return new Promise((resolve) => setTimeout(resolve, ms))
}

test('ix: send-delete-post', async (t) => {
  const processes = new Processes()
  const tasks = new Tasks()
  processes.startAmman()
  processes.startGeyserStore()
  {
    await sleep(5000)

    await tasks.solanaxSendDelete()
    const updates = await tasks.fetchAccounts()

    assert.equal(updates.length, 2)
    spok(t, updates, [
      {
        slot: spok.gtz,
        pubkey: spok.string,
        lamports: 9103680,
        owner: 'SoLXmnP9JvL6vJ7TN1VqtTxqsc2izmPfF9CsMDEuRzJ',
        executable: false,
        rent_epoch: 0,
        write_version: spok.gtz,
        txn_signature: spok.string,
      },
      {
        slot: spok.gtz,
        pubkey: spok.string,
        lamports: 0,
        owner: 'SoLXmnP9JvL6vJ7TN1VqtTxqsc2izmPfF9CsMDEuRzJ',
        executable: false,
        rent_epoch: 0,
        write_version: spok.gtz,
        txn_signature: spok.string,
      },
    ])
    const up1 = updates[0]
    const up2 = updates[1]

    assert(up1.slot <= up2.slot, 'slots are in order')
    assert(up1.write_version < up2.write_version, 'write_versions are in order')
  }
  processes.stopAmman()
  processes.stopGeyserStore()

  sleep(1000)
})
