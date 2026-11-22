# market-data-service (deployment seam)

This is a **deployment seam**, not a separate codebase. The implementation lives
in the Rust workspace at `services/trading-engine-rust` as the `market-data`
binary crate.

- **Single VPS (current):** the `engine` supervisor runs market-data in-process.
- **Regional split (future):** deploy this binary on its own by overriding the
  container command:

  ```yaml
  command: ["market-data"]
  ```

Market data is **shared per region** (public data — not tenant-partitioned).
Source is selected by `MARKET_DATA_SOURCE` (`synthetic` | `exchange`).
