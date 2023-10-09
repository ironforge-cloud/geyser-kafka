import execa from 'execa'
import path from 'path'
import assert from 'assert/strict'
import { logDebug } from './log'

export class Tasks {
  constructor(
    readonly solanaxPath: string = path.join(process.cwd(), '..', 'solanax'),
    readonly geyserStorePort = 9999
  ) {}

  solanaxSendDelete() {
    logDebug('Posting and Deleting Post')
    return execa(
      'yarn',
      ['post+delete', '1', '"Some post that will be immediately deleted"'],
      {
        cwd: this.solanaxPath,
        stdio: 'inherit',
        env: { LOCAL: 1 } as unknown as NodeJS.ProcessEnv,
      }
    )
  }

  async fetchAccounts() {
    // @ts-ignore fetch is global damnit
    const res = await fetch(`http://127.0.0.1:${this.geyserStorePort}/accounts`)
    assert(res.ok, `fetch failed: ${res.status} ${res.statusText}`)
    return res.json()
  }
}
