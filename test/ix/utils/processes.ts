import execa, { ExecaChildProcess } from 'execa'
import path from 'path'
import assert from 'assert/strict'
import { logDebug } from './log'

const AMMAN = 'amman_'

export class Processes {
  ammanProcess?: ExecaChildProcess<string>
  geyserStoreProcess?: ExecaChildProcess<string>
  constructor(
    readonly geyserStorePath: string = path.join(
      process.cwd(),
      '..',
      'geyser-store'
    )
  ) {}

  startAmman() {
    assert(this.ammanProcess == null, 'Amman already started')
    logDebug('Starting Amman')
    this.ammanProcess = execa(AMMAN, ['start'], { stdio: 'inherit' })
  }

  async stopAmman() {
    logDebug('Stopping Amman')
    assert(this.ammanProcess != null, 'Amman not running')
    await execa(AMMAN, ['stop'], { stdio: 'inherit' })
    this.ammanProcess.kill('SIGTERM', { forceKillAfterTimeout: 200 })
  }

  startGeyserStore() {
    assert(this.geyserStoreProcess == null, 'Geyser Store already started')
    logDebug('Starting Geyser Store')
    this.geyserStoreProcess = execa('cargo', ['run'], {
      cwd: this.geyserStorePath,
      stdio: 'inherit',
    })
  }

  stopGeyserStore() {
    assert(this.geyserStoreProcess != null, 'Geyser Store not running')
    logDebug('Stopping Geyser Store')
    this.geyserStoreProcess.kill('SIGTERM', { forceKillAfterTimeout: 200 })
  }
}
