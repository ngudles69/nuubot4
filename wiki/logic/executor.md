## Covers

1. `src/executor.rs`
2. `src/executor/observer.rs`
3. `src/botcycle.rs`
4. `src/runtime.rs`
5. `src/account.rs` - not ported

## Intent

Turn coherent Account state and Signals into trading actions.

## Ownership

```text
Runtime
`-- BotCycle
    `-- Vec<Executor>
        `-- each Executor owns Vec<Account>
```

BotCycle mutably borrows each Executor. Executor mutably borrows its Accounts.
No `Arc` or shared Account.

## Program Flow

```text
BotCycle::init(configs)
  create each configured Executor

BotCycle::acct_recon(&mut self, now)
  for mut executor
    snapshots += executor.acct_recon(now)
  return owned snapshots

BotCycle::mainloop(&mut self, now)
  for mut non-terminal executor
    executor.mainloop(now)
  return whether all Executors are terminal

Executor::acct_recon(&mut self, now)
  for mut account
    snapshots += account.recon(now)

Executor::mainloop(&mut self, now)
  read latest owned Account state
  place, cancel, or manage Orders

BotCycle::stop(&mut self)
  stop Executors in reverse order
```

## Main Flow

```text
Runtime -> BotCycle reconcile -> Risk -> BotCycle decisions -> Executor -> Account
```

## Code Alignment

- ObserverExecutor only counts events and becomes terminal.
- Executor trait, Accounts, reconciliation, and trading actions are not ported.
- Current Runtime calls Risk before Executor because Account truth is absent.
