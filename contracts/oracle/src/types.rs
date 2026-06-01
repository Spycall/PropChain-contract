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

// ── Enhanced Slashing Types (Issue #226) ──────────────────────────────────────

/// Severity level for oracle source slashing.
#[derive(Debug, Clone, PartialEq, Eq, scale::Encode, scale::Decode)]
#[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
pub enum SlashingSeverity {
    /// Minor infraction: stale data submission (slash 5% of stake)
    Minor,
    /// Moderate infraction: inaccurate data within 10% deviation (slash 15%)
    Moderate,
    /// Severe infraction: inaccurate data >10% deviation or repeated offenses (slash 30%)
    Severe,
    /// Critical infraction: malicious data or collusion (slash 50% + ban)
    Critical,
}

/// Record of a single slashing event for an oracle source.
#[derive(Debug, Clone, PartialEq, Eq, scale::Encode, scale::Decode)]
#[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
pub struct SlashingRecord {
    /// Block number when the slashing occurred
    pub block: u64,
    /// Severity of the slashing
    pub severity: SlashingSeverity,
    /// Amount slashed
    pub amount_slashed: u128,
    /// Reason for the slashing
    pub reason: String,
    /// Whether the source was banned as a result
    pub banned: bool,
}

/// Slashing configuration parameters.
#[derive(Debug, Clone, PartialEq, Eq, scale::Encode, scale::Decode)]
#[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
pub struct SlashingConfig {
    /// Percentage of stake slashed for minor offenses (basis points)
    pub minor_slash_bps: u32,
    /// Percentage for moderate offenses (basis points)
    pub moderate_slash_bps: u32,
    /// Percentage for severe offenses (basis points)
    pub severe_slash_bps: u32,
    /// Percentage for critical offenses (basis points)
    pub critical_slash_bps: u32,
    /// Reputation penalty for minor offenses
    pub minor_reputation_penalty: u32,
    /// Reputation penalty for moderate offenses
    pub moderate_reputation_penalty: u32,
    /// Reputation penalty for severe offenses
    pub severe_reputation_penalty: u32,
    /// Reputation penalty for critical offenses
    pub critical_reputation_penalty: u32,
    /// Number of slashing events before auto-suspension
    pub suspension_threshold: u32,
    /// Duration (in blocks) before a banned source can re-register
    pub ban_duration_blocks: u64,
}

impl Default for SlashingConfig {
    fn default() -> Self {
        Self {
            minor_slash_bps: 500,           // 5%
            moderate_slash_bps: 1500,        // 15%
            severe_slash_bps: 3000,          // 30%
            critical_slash_bps: 5000,        // 50%
            minor_reputation_penalty: 50,
            moderate_reputation_penalty: 150,
            severe_reputation_penalty: 300,
            critical_reputation_penalty: 500,
            suspension_threshold: 3,
            ban_duration_blocks: 100_000,    // ~1 week at 6s blocks
        }
    }
}

/// Status of an oracle source after slashing evaluation.
#[derive(Debug, Clone, PartialEq, Eq, scale::Encode, scale::Decode)]
#[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
pub struct SourceStatus {
    /// Current source reputation (0-1000)
    pub reputation: u32,
    /// Current stake amount
    pub stake: u128,
    /// Whether the source is currently active
    pub is_active: bool,
    /// Whether the source is banned
    pub is_banned: bool,
    /// Block number when ban expires (0 if not banned)
    pub ban_expires_at: u64,
    /// Total number of slashing events
    pub total_slashes: u32,
    /// Total amount slashed
    pub total_amount_slashed: u128,
}

/// Summary of all slashing records for querying.
#[derive(Debug, Clone, PartialEq, Eq, scale::Encode, scale::Decode)]
#[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
pub struct SlashingSummary {
    pub recent_slashes: Vec<SlashingRecord>,
    pub total_slashes: u32,
    pub total_amount_slashed: u128,
}
