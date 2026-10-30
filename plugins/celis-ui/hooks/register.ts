import type { Register } from 'claude-code'

const KEPT = /(^|[\s;&|(])celis pattern\s/
const USES = /--uses[\s=]+([0-9a-f]{4,})/g

/** The status line with `kept` and `used` patterns. */
function shown(kept: number, used: number): string {
  return `celis · ${kept} kept · ${used} used`
}

/**
 * Shows in the status line whether celis is on, with the patterns the agent kept and used in this
 * session, and a toast for each one.
 */
export const register: Register = on => {
  let kept = 0
  let used = 0

  on('session.start', async ($, e, next) => {
    const started = await next(e)
    const found = await $.process.run(['celis', '--version']).catch(() => null)
    $.ui.status(found?.exitCode === 0 ? 'celis · on' : 'celis · not on PATH: run celis setup')
    return started
  })

  on('tool.call', { tool: 'Bash' }, async ($, e, next) => {
    const ran = await next(e)
    if (ran.deny !== undefined || ran.isError === true) return ran
    if (KEPT.test(e.command)) {
      kept += 1
      $.ui.toast('celis kept a pattern')
      $.ui.status(shown(kept, used))
    }
    const ids = [...e.command.matchAll(USES)].map(m => m[1].slice(0, 8))
    if (ids.length > 0) {
      used += ids.length
      $.ui.toast(`celis: used pattern ${ids.join(', ')}`)
      $.ui.status(shown(kept, used))
    }
    return ran
  })
}
