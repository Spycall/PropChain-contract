// Local types for the oracle contract (Issue #101 - extracted from lib.rs)

/// Result of an oracle batch operation
#[derive(Debug, Clone, PartialEq, Eq, scale::Encode, scale::Decode)]
#[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
pub struct OracleBatchResult {
    pub successes: Vec<u64>,
    pub failures: Vec<OracleBatchItemFailure>,
    pub total_items: u32,
    pub successful_items: u32,
    pub failed_items: u32,
    pub early_terminated: bool,
}

/// A single item failure in an oracle batch operation
#[derive(Debug, Clone, PartialEq, Eq, scale::Encode, scale::Decode)]
#[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
pub struct OracleBatchItemFailure {
    pub index: u32,
    pub item_id: u64,
    pub error: OracleError,
}

// ── Fallback Oracle Types (Issue #220) ──────────────────────────────────────

/// Configuration for the oracle fallback mechanism.
#[derive(Debug, Clone, PartialEq, Eq, scale::Encode, scale::Decode)]
#[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
#[cfg_attr(feature = "std", derive(ink::storage::traits::StorageLayout))]
pub struct FallbackConfig {
    /// Whether fallback is enabled
    pub enabled: bool,
    /// Delay in blocks before fallback is triggered
    pub fallback_delay_blocks: u32,
    /// Max number of fallback attempts per request
    pub max_fallback_attempts: u32,
    /// Whether to prefer the fallback source with the lowest latency
    pub prefer_lowest_latency: bool,
}

impl Default for FallbackConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            fallback_delay_blocks: 2,
            max_fallback_attempts: 3,
            prefer_lowest_latency: true,
        }
    }
}

/// A fallback oracle source configuration.
#[derive(Debug, Clone, PartialEq, Eq, scale::Encode, scale::Decode)]
#[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
#[cfg_attr(feature = "std", derive(ink::storage::traits::StorageLayout))]
pub struct FallbackSource {
    /// Unique identifier for this fallback source
    pub id: String,
    /// Priority order (lower = higher priority)
    pub priority: u32,
    /// Whether this fallback source is currently active
    pub active: bool,
    /// Estimated latency in milliseconds
    pub estimated_latency_ms: u32,
    /// Number of successful queries through this source
    pub success_count: u64,
    /// Number of failed queries through this source
    pub failure_count: u64,
}

/// Result from a fallback query.
#[derive(Debug, Clone, PartialEq, Eq, scale::Encode, scale::Decode)]
#[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
#[cfg_attr(feature = "std", derive(ink::storage::traits::StorageLayout))]
pub struct FallbackQueryResult {
    /// Whether the query succeeded
    pub success: bool,
    /// The price returned (0 if failed)
    pub price: u128,
    /// Which fallback source served the data
    pub source_id: String,
    /// Number of fallback attempts made
    pub attempts: u32,
    /// Timestamp of the result
    pub timestamp: u64,
    /// Error message if failed
    pub error: String,
}
