# CLOID

CLOID is the 128-bit client Order identity sent to Hyperliquid. Account owns
its creation. Nuubot4 exposes only `encode_cloid` and `decode_cloid`.

## Layout

| Field | Bits | Allowed values |
|---|---:|---:|
| `botcycle_id` | 24 | 0-16,777,215 |
| `symbol_id` | 16 | 0-65,535 |
| `exchange` | 4 | 0-15 |
| `network` | 2 | 0-3 |
| `side` | 1 | 0-1 |
| `reduce_only` | 1 | false/true |
| `purpose` | 8 | 0-255 |
| `trade_no` | 21 | 1-2,097,151 |
| `batch_no` | 10 | 1-1,000 |
| `order_pos` | 10 | 1-1,000 |
| `timestamp_s` | 31 | 0-2,147,483,647 |

Encoding validates every range and never truncates or wraps. Decoding validates
the exact hexadecimal shape and rejects invalid Order identity fields. There
is one current layout and no compatibility decoder.
