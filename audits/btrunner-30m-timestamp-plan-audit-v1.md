# BtRunner 30-Month Timestamp Plan Audit V1

## Verdict

PASS.

## Findings

- Accept close-time microsecond remainders `999000..=999999`.
- Normalize each accepted close to the next whole-second `ts_ms`.
- Require admitted timestamps to advance by exactly 1,000ms.
- Own admission, normalization, and sequence validation at the shared
  `src/replay.rs` boundary; both CSV and Parquet loaders already route there.
- Keep half-open raw timestamp range filtering unchanged.
- Prove checked input conversion and overflow rejection.
- Preserve 2025+ behavior while correcting pre-2025 timestamps.
- The expected 30-month totals agree: 78,883,200 ticks and 7,888,320
  callbacks.
