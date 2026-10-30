import { expect, test } from 'claude-code/testing'

const RAN = { exitCode: 0, stdout: '', stderr: '', isStdoutTruncated: false, isStderrTruncated: false }

test('the status line says celis is on when the celis command runs', async ($, on) => {
  const shown: string[] = []
  on('session.start', (_, e) => ({ cwd: e.cwd }))
  on('process.run', () => ({ value: RAN }))
  on('ui.status', (_, e) => {
    shown.push(String(e.text))
    return { value: undefined }
  })
  await $.session.start({ cwd: '/', surface: 'terminal', isInteractive: true })
  expect(shown).toEqual(['celis · on'])
})

test('the status line names celis setup when the celis command is missing', async ($, on) => {
  const shown: string[] = []
  on('session.start', (_, e) => ({ cwd: e.cwd }))
  on('process.run', () => ({ value: { ...RAN, exitCode: 127 } }))
  on('ui.status', (_, e) => {
    shown.push(String(e.text))
    return { value: undefined }
  })
  await $.session.start({ cwd: '/', surface: 'terminal', isInteractive: true })
  expect(shown).toEqual(['celis · not on PATH: run celis setup'])
})

test('a kept and a used pattern each get a toast and count in the status line', async ($, on) => {
  const toasts: string[] = []
  const shown: string[] = []
  on('tool.call', () => ({ result: {} }))
  on('ui.toast', (_, e) => {
    toasts.push(String(e.text))
    return { value: undefined }
  })
  on('ui.status', (_, e) => {
    shown.push(String(e.text))
    return { value: undefined }
  })
  await $.tool.call({ tool: 'Bash', command: 'celis pattern -- "Name: Flat logos"' })
  await $.tool.call({ tool: 'Bash', command: 'celis step add --uses 3fa2c1d0aa --uses 9b0e -- "x"' })
  await $.tool.call({ tool: 'Bash', command: 'ls' })
  expect(toasts).toEqual(['celis kept a pattern', 'celis: used pattern 3fa2c1d0, 9b0e'])
  expect(shown).toEqual(['celis · 1 kept · 0 used', 'celis · 1 kept · 2 used'])
})
