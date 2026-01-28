<p align="center">
  <img src="/logo.png" height="160">
</p>

[![CI](https://github.com/fjall-rs/lsm-tree/actions/workflows/test.yml/badge.svg)](https://github.com/fjall-rs/lsm-tree/actions/workflows/test.yml)
[![docs.rs](https://img.shields.io/docsrs/lsm-tree?color=green)](https://docs.rs/lsm-tree)
[![Crates.io](https://img.shields.io/crates/v/lsm-tree?color=blue)](https://crates.io/crates/lsm-tree)
![MSRV](https://img.shields.io/badge/MSRV-1.91.0-blue)
[![dependency status](https://deps.rs/repo/github/fjall-rs/lsm-tree/status.svg)](https://deps.rs/repo/github/fjall-rs/lsm-tree)

A K.I.S.S. implementation of log-structured merge trees (LSM-trees/LSMTs) in Rust.

> [!NOTE]
> This crate only provides a primitive LSM-tree, not a full storage engine.
> For example, it does not ship with a write-ahead log.
> You probably want to use https://github.com/fjall-rs/fjall instead.

## About

This is the most feature-rich LSM-tree implementation in Rust! It features:

- Thread-safe `BTreeMap`-like API
- Mostly [safe](./UNSAFE.md) & 100% stable Rust
- Block-based tables with compression support & prefix truncation
  - Optional block hash indexes in data blocks for faster point lookups [[3]](#footnotes)
  - Per-level filter/index block pinning configuration
- Range & prefix searching with forward and reverse iteration
- Block caching to keep hot data in memory
- File descriptor caching with upper bound to reduce fopen calls
- *AMQ* filters (currently Bloom filters) to improve point lookup performance
- Multi-versioning of KVs, enabling snapshot reads
- Optionally partitioned block index & filters for better cache efficiency [[1]](#footnotes)
- Size-tiered, (concurrent) Leveled and FIFO compaction 
- Multi-threaded flushing (immutable/sealed memtables)
- Key-value separation (optional) [[2]](#footnotes)
- Single deletion tombstones ("weak" deletion)
- Merge operator for atomic read-modify-write operations [[4]](#footnotes)

Keys are limited to 65536 bytes, values are limited to 2^32 bytes.
As is normal with any kind of storage engine, larger keys and values have a bigger performance impact.

## Sponsors

<a href="https://sqlsync.dev">
  <picture>
    <source width="240" alt="Orbitinghail" media="(prefers-color-scheme: light)" srcset="https://raw.githubusercontent.com/fjall-rs/fjall-rs.github.io/d22fcb1e6966ce08327ea3bf6cf2ea86a840b071/public/logos/orbitinghail.svg" />
    <source width="240" alt="Orbitinghail" media="(prefers-color-scheme: dark)" srcset="https://raw.githubusercontent.com/fjall-rs/fjall-rs.github.io/d22fcb1e6966ce08327ea3bf6cf2ea86a840b071/public/logos/orbitinghail_dark.svg" />
    <img width="240" alt="Orbitinghail" src="https://raw.githubusercontent.com/fjall-rs/fjall-rs.github.io/d22fcb1e6966ce08327ea3bf6cf2ea86a840b071/public/logos/orbitinghail_dark.svg" />
  </picture>
</a>

## Feature flags

### lz4

Allows using `LZ4` compression, powered by [`lz4_flex`](https://github.com/PSeitz/lz4_flex).

*Disabled by default.*

### bytes

Uses [`bytes`](https://github.com/tokio-rs/bytes) as the underlying `Slice` type.

*Disabled by default.*

## Merge Operator

Merge operators allow you to perform atomic read-modify-write operations without reading the value first. This is useful for counters, appending to lists, or updating specific fields in a record.

```rust
use lsm_tree::{Config, MergeOperator, MergeResult, UserKey, UserValue};
use std::sync::Arc;

// Define a counter merge operator
struct CounterMerge;

impl MergeOperator for CounterMerge {
    fn name(&self) -> &'static str {
        "CounterMerge"
    }

    fn full_merge(
        &self,
        _key: &UserKey,
        existing_value: Option<&UserValue>,
        operands: &[UserValue],
    ) -> MergeResult {
        // Parse existing value as i64, defaulting to 0
        let mut counter = existing_value
            .and_then(|v| std::str::from_utf8(v).ok())
            .and_then(|s| s.parse::<i64>().ok())
            .unwrap_or(0);

        // Add all operands
        for operand in operands {
            if let Some(delta) = std::str::from_utf8(operand)
                .ok()
                .and_then(|s| s.parse::<i64>().ok())
            {
                counter += delta;
            }
        }

        MergeResult::Success(counter.to_string().into_bytes().into())
    }
}

// Use the merge operator
let config = Config::new(folder)
    .merge_operator(Arc::new(CounterMerge));

let tree = config.open()?;

// Increment counter without reading first
tree.merge("hits", "1", seqno);
tree.merge("hits", "1", seqno + 1);
tree.merge("hits", "5", seqno + 2);

// Reading will return "7"
let value = tree.get("hits")?;
```

## Run unit benchmarks

```bash
cargo bench --features lz4
```

## License

All source code is licensed under MIT OR Apache-2.0.

All contributions are to be licensed as MIT OR Apache-2.0.

## Footnotes

[1] https://rocksdb.org/blog/2017/05/12/partitioned-index-filter.html

[2] https://github.com/facebook/rocksdb/wiki/BlobDB

[3] https://rocksdb.org/blog/2018/08/23/data-block-hash-index.html

[4] https://github.com/facebook/rocksdb/wiki/Merge-Operator
