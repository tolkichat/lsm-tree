// Copyright (c) 2024-present, fjall-rs
// This source code is licensed under both the Apache 2.0 and MIT License
// (found in the LICENSE-* files in the repository)

//! Merge operator trait for atomic read-modify-write operations.
//!
//! Similar to RocksDB's MergeOperator, this allows efficient partial updates
//! without requiring a full read-modify-write cycle.

use crate::{UserKey, UserValue};

/// Result of a merge operation.
#[derive(Debug, Clone)]
pub enum MergeResult {
    /// The merge operation succeeded with the given value.
    Success(UserValue),

    /// The merge operation failed.
    ///
    /// When a merge fails during compaction, the operands are preserved
    /// to avoid data loss. The error can be logged or handled by the caller.
    Failure,
}

/// Trait for implementing custom merge operators.
///
/// A merge operator allows atomic read-modify-write operations without
/// requiring a full read of the existing value. This is useful for:
/// - Incrementing counters
/// - Appending to lists
/// - Updating individual fields in a structured value
///
/// # Example
///
/// ```ignore
/// use lsm_tree::{MergeOperator, MergeResult, UserKey, UserValue};
///
/// struct CounterMerge;
///
/// impl MergeOperator for CounterMerge {
///     fn name(&self) -> &'static str {
///         "CounterMerge"
///     }
///
///     fn full_merge(
///         &self,
///         _key: &UserKey,
///         existing_value: Option<&UserValue>,
///         operands: &[UserValue],
///     ) -> MergeResult {
///         let mut counter = existing_value
///             .and_then(|v| std::str::from_utf8(v).ok())
///             .and_then(|s| s.parse::<i64>().ok())
///             .unwrap_or(0);
///
///         for operand in operands {
///             if let Some(delta) = std::str::from_utf8(operand)
///                 .ok()
///                 .and_then(|s| s.parse::<i64>().ok())
///             {
///                 counter += delta;
///             }
///         }
///
///         MergeResult::Success(counter.to_string().into_bytes().into())
///     }
/// }
/// ```
pub trait MergeOperator: Send + Sync {
    /// Returns the name of the merge operator.
    ///
    /// This is used for debugging and logging purposes.
    fn name(&self) -> &'static str;

    /// Performs a full merge operation.
    ///
    /// This is called when:
    /// - A `get()` operation encounters merge operands
    /// - During compaction when merge operands need to be collapsed
    ///
    /// # Arguments
    ///
    /// * `key` - The key being merged
    /// * `existing_value` - The base value if one exists (from a Put operation),
    ///   or `None` if only merge operands exist
    /// * `operands` - The merge operands in order from oldest to newest
    ///
    /// # Returns
    ///
    /// * `MergeResult::Success(value)` - The merged value
    /// * `MergeResult::Failure` - The merge failed; operands will be preserved
    fn full_merge(
        &self,
        key: &UserKey,
        existing_value: Option<&UserValue>,
        operands: &[UserValue],
    ) -> MergeResult;

    /// Performs a partial merge of two operands.
    ///
    /// This is an optional optimization that can combine multiple merge operands
    /// into a single operand during compaction, even when no base value exists.
    ///
    /// For example, if you have three `+1` increment operands, partial merge
    /// could combine them into a single `+3` operand.
    ///
    /// # Arguments
    ///
    /// * `key` - The key being merged
    /// * `left` - The older operand
    /// * `right` - The newer operand
    ///
    /// # Returns
    ///
    /// * `Some(value)` - The combined operand
    /// * `None` - Partial merge is not possible; keep operands separate
    ///
    /// # Default Implementation
    ///
    /// Returns `None`, meaning no partial merging is performed.
    fn partial_merge(
        &self,
        _key: &UserKey,
        _left: &UserValue,
        _right: &UserValue,
    ) -> Option<UserValue> {
        None
    }
}
