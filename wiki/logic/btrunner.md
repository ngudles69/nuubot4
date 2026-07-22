## Covers

1. `src/bin/nuubot-btrunner.rs`
2. `src/btrunner.rs`
3. `src/clock.rs`
4. `src/runtime.rs`
5. `src/botcycle.rs`
6. `src/signaler.rs`
7. `src/executor.rs`
8. `src/risk.rs`

## Intent

Run one Sweep Bot. Serve replay ticks. Drive Runtime. Stop direct children.
Each owner logs its own statistics.

## Ownership

```text
main
`-- mut BtRunner
    |-- TickClock
    |-- TickReader
    `-- Runtime
        |-- Signaler
        |-- Vec<Risk>
        `-- BotCycle -> Vec<Executor>
```

One owner. Temporary borrows. No `Arc`, lock, or shared mutable domain state.

## bin/nuubot-btrunner.rs

```text
main()
  arguments = arguments()
  exit(PROGRAM, program(arguments))

program(arguments)
  identity = parser(arguments)?
  run(identity)

parser(arguments)
  reject wrong argument count
  identity = BotIdentity
    sweep_id = parse_id(arguments[0])?
    bot_id = parse_id(arguments[1])?
  return identity

run(identity)
  log = logger(bot log filename)?
  mut runner = BtRunner::init(log, identity)?
  runner.start()?
  run_result = runner.run()
  stop_result = runner.stop()
  return run_result.and(stop_result)

parse_id(value)
  reject non-numeric or zero value
  return ID
```

## btrunner.rs

```text
BtRunner::init(&log, identity)
  ctx = load config and Bot
  clock = TickClock::init(log, interval)
  ticks = TickReader::init(log, Bot, BtRunner config)
  runtime = Runtime::init(log, runtime_config)
  stats = BtRunnerStats::init(Bot, BtRunner config)
  return stopped BtRunner

BtRunner::start(&mut self)
  guard lifecycle
  runtime.start()
  mark started

BtRunner::run(&mut self)
  guard lifecycle

  for window in LoadingWindows
    for tick in ticks.load(window)
      runtime.ingest_bbo(tick)
      record served tick

      if clock.advance(tick.time) is due
        passes_triggered += 1
        if runtime.mainloop(tick.time)
          break replay

    pause between windows when configured

  verify replay
  return OK

BtRunner::stop(&mut self)
  if already stopped => Ok
  if not started => error
  stop Runtime, TickReader, Clock in reverse init order
  each child logs its own statistics
  log BtRunner-owned statistics
  return first stop error, if any
```

## clock.rs

```text
TickClock::advance(&mut self, now)
  first tick => set next time; due
  now >= next => move next time forward; due
  otherwise => not due
```

## runtime.rs

```text
Runtime::init()
  create Signaler
  create Risks
  create BotCycle and Executors

Runtime::mainloop(&mut self, now)
  assess Risk
  stop cycle when Risk exits
  otherwise advance BotCycle
  replace completed cycle or stop at max_cycles

Runtime::stop(&mut self)
  latch stop
  stop active BotCycle
```

See [Signaler](signaler.md), [Executor](executor.md), and [Risk](risk.md).

## Code Alignment

- `common/program.rs` reads arguments and handles the final error once.
- Parser failures create the generic `errors.log` logger.
- Parsed identity creates one named Bot logger before database validation.
- Bot errors go to the Bot log and `errors.log` through that installed logger.
- Lower functions only propagate errors through `Result` and `?`.
- Each component logs its own statistics during its own `stop()`.
- BtRunner reports success only when replay verification and child teardown
  both succeed.
- Every process eagerly creates exactly one logger.
- BtRunner replay flow matches.
