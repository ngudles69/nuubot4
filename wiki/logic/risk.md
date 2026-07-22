## Covers

1. `src/risk.rs`
2. `src/risk/balanced.rs`
3. `src/runtime.rs`
4. `src/account.rs` - future AccountSnapshot source

## Intent

Stop trading when coherent Account state breaches a risk rule.

## Ownership

Runtime owns `Vec<Risk>`. Risk receives owned snapshots. No Account reference.
No `Arc`.

## Program Flow

```text
Runtime::mainloop(&mut self, now)
  snapshots = botcycle.acct_recon(now)

  for risk in risks
    if risk.assess(snapshots)
      request graceful stop("risk")
      stop active BotCycle
      return

  botcycle.mainloop(now)

BalancedRisk::assess(snapshots)
  evaluate configured balanced-risk rule
  return stop_required

MaxDrawdownRisk::assess(snapshots)
  calculate coherent equity and peak
  return drawdown >= configured limit
```

## Dependencies

```text
Executor Accounts -> owned AccountSnapshots -> Runtime -> Risk
Risk exit -> Runtime-owned stop path
```

## Code Alignment

- BalancedRisk currently always returns false.
- Risk receives no AccountSnapshot data.
- Max drawdown and other real rules are not ported.
- BalancedRisk logs assessment statistics during stop.
