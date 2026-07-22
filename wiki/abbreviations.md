# Abbreviations

Use these names consistently.

- `mgr` = manager
- `ctx` = context

## Domain Fields

- `qty` = quantity
- `req` = requested or required
- `fees` = fees charged by exchange/simulator
- `cloid` = client order id
- `raw_json` = raw JSON payload as a Python dict
- `bbo` = best bid/offer
- `Side` = Hyperliquid side, `B` or `A`
- `pnl` = profit and loss
- `tf` = timeframe

## Rules

- Prefer these abbreviations in field names.
- Do not abbreviate when it makes the name unclear.
- Do not create first-class fields for every exchange payload key.
- Keep unused exchange payload values in `raw_json`.
