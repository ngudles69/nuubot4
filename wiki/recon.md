# Recon Ownership and Flow

This page is the canonical Nuubot4 design for Account ownership and
reconciliation. It intentionally differs from Nuubot3's shared Runtime account
list. Do not introduce `AccountBook`, `AccountId`, shared Account references, or
Runtime-owned Accounts without explicit user approval and an update to this
page.

The surrounding Runner and Runtime tree is canonical in
[Ownership and Project Structure](ownership.md).

## How does it work?

Each Executor owns the Accounts it uses.

```text
BtRunner / Runner
`-- Runtime
    `-- BotCycle
        `-- Executors
            `-- each Executor owns Vec<Account>
                `-- each Account owns Venue and Ledger
                    `-- Ledger owns Trades -> Orders -> Fills
```

The number of Accounts is ordinary data:

- no Accounts: an observer Executor;
- one Account: a simple Executor;
- multiple Accounts: an Executor such as grid plus hedge.

Runtime owns the order of work, not the Accounts. One Runtime control pass is:

```text
BotCycle reconciles every Executor's Accounts
-> Executors return AccountSnapshot values
-> Runtime evaluates Risk from one coherent post-recon state
-> BotCycle lets every Executor make decisions
```

Runtime calls one intent-level method on its direct child, BotCycle. BotCycle
calls each Executor. Each Executor reconciles its own Accounts. Runtime never
keeps an Account reference or reaches through BotCycle into an Executor.

An `AccountSnapshot` is an owned value containing the reconciled state needed
by Risk. Risk receives snapshots, not Account references. This keeps the
reconciliation phase complete before the decision phase begins.

Config and credentials may be borrowed temporarily during initialization so an
Executor can create its Accounts from role and account-name configuration.
Runtime and BotCycle pass those setup borrows downward but do not retain them or
copy credentials into their own state.

If two Executors later need to mutate the same real Account, stop and redesign
the ownership with explicit user approval. Do not silently add shared ownership
or locks.

## Rust Implementation

There is one Executor trait because Nuubot4 will have multiple real Executor
implementations. The trait provides the common reconciliation loop:

```rust
pub trait Executor {
    fn accounts_mut(&mut self) -> &mut [Account];

    fn acct_recon(
        &mut self,
        now_ms: u64,
    ) -> Result<Vec<AccountSnapshot>> {
        self.accounts_mut()
            .iter_mut()
            .map(|account| account.recon(now_ms))
            .collect()
    }

    fn mainloop(&mut self, now_ms: u64) -> Result<()>;
}
```

Each concrete Executor owns one `Vec<Account>` and exposes only a temporary
mutable slice to the default trait method:

```rust
pub struct GridExecutor {
    accounts: Vec<Account>,
}

impl Executor for GridExecutor {
    fn accounts_mut(&mut self) -> &mut [Account] {
        &mut self.accounts
    }

    fn mainloop(&mut self, now_ms: u64) -> Result<()> {
        // Make trading decisions.
        Ok(())
    }
}
```

`accounts_mut()` does not return reconciled Accounts and does not transfer
ownership. It temporarily lends the default method exclusive mutable access to
the concrete Executor's owned Accounts. Returning a slice permits changing the
Account objects but not adding, removing, or reordering them. The borrow ends
when `acct_recon()` returns.

`&mut self` is required because reconciliation changes Account and Ledger
state. The exclusive borrow follows the ownership tree:

```text
BotCycle -> Executor -> Account -> Ledger
```

BotCycle gathers the snapshots:

```rust
pub fn acct_recon(
    &mut self,
    now_ms: u64,
) -> Result<Vec<AccountSnapshot>> {
    let mut state = Vec::new();

    for executor in &mut self.executors {
        state.extend(executor.acct_recon(now_ms)?);
    }

    Ok(state)
}
```

`state` is mutable only because each Executor's snapshots are appended to it.
The Accounts themselves remain owned by their concrete Executors.

Do not add separate `SingleAccountRecon`, `MultiAccountRecon`, or `NoRecon`
traits. The same default loop correctly handles zero, one, or many Accounts.

This design requires no `Arc`, `Rc`, `Mutex`, `RwLock`, `RefCell`,
`AccountBook`, `AccountId`, self-reference, or long-lived Account reference.
