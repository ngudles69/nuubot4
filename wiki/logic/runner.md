## Covers

1. `src/bin/nuubot-runner.rs` - not ported
2. `src/runner.rs` - not ported
3. `src/clock.rs` - TickClock only; WallClock not ported
4. `src/runtime.rs`
5. `src/botcycle.rs`
6. `src/signaler.rs`
7. `src/executor.rs`
8. `src/risk.rs`

## Intent

Run one live, Testnet, or Simnet Bot until stopped.

## Ownership

```text
RunnerService
`-- mut Runner
    |-- WallClock
    |-- BarFeed
    |-- optional BboFeed
    `-- Runtime
```

Runner owns its children. Feeds return owned values. No `Arc` unless real task
boundaries later require shared ownership.

## bin/nuubot-runner.rs

```text
main()
  started_at = now()
  identity = parser()
    Err(error)
      log = logger(None)
      log.error(error)
      return FAILURE

  log = logger(Some(identity))
  result = run(&log, identity)

  match result
    Ok(summary) => log.info(summary)
    Err(error)  => log.error(error)

  log.info("main completed", result, elapsed)

parser()
  read network
  read runtime bot_id
  validate identity
  return identity
```

## runner.rs

```text
Runner::init(&log, identity)
  setup runtime infrastructure
  load runtime Bot
  create WallClock
  create Runtime
  create required live feeds
  register callbacks

Runner::start(&mut self)
  runtime.start()
  feeds.start()
  clock.start()

Runner::run(&mut self)
  supervise Clock and feeds until stop

Runner::stop(&mut self)
  close Runtime admission
  stop feeds
  stop Clock
  stop Runtime
  publish terminal state
```

See [Signaler](signaler.md), [Executor](executor.md), and [Risk](risk.md).

## Code Alignment

- Live Runner entrypoint, Runner, WallClock, feeds, and RunnerService are not ported.
- Do not copy BtRunner replay logic into Runner.
- Runtime is the current shared control core.
