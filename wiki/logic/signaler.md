## Covers

1. `src/signaler.rs`
2. `src/signaler/macross.rs`
3. `src/runtime.rs`

## Intent

Turn admitted Bars into trading Signals.

## Ownership

Runtime owns one mutable Signaler. Bars are copied values. No `Arc`.

## Program Flow

```text
Runtime::ingest_bars(&mut self, bars)
  for bar in bars
    signal = signaler.on_bar(bar)
    send bar to active BotCycle
    route actionable signal into BotCycle creation

MacrossSignaler::init(config)
  store MA config
  start with no closes and no previous side

MacrossSignaler::on_bar(&mut self, bar)
  append close
  keep slow-MA window
  return None until window is ready
  calculate fast and slow MA
  determine long, short, or flat side
  emit only a side change
```

## Dependencies

```text
Runtime -> Signaler -> Bar + SignalerConfig
Signaler output -> BotCycle / Executor flow
```

## Code Alignment

- Macross calculation is ported.
- Runtime currently discards the returned Signal.
- Signal-driven BotCycle creation is not ported.
- MacrossSignaler logs its own evaluation and signal statistics during stop.
