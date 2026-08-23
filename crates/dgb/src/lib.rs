#![forbid(unsafe_code)]

//! DigiByte consensus-adjacent mining rules pinned to Core v9.26.5.

use core::fmt;
use stratum_primitives::{CompactTarget, PrimitiveError, Target256};

pub const MULTISHIELD_V4_WINDOW_BLOCKS: usize = 50;
pub const MTP_WINDOW_BLOCKS: usize = 11;
pub const MULTISHIELD_V4_REQUIRED_HISTORY: usize = MULTISHIELD_V4_WINDOW_BLOCKS + MTP_WINDOW_BLOCKS;
pub const MULTISHIELD_V4_TARGET_TIMESPAN: i64 = 750;
pub const MULTISHIELD_V4_MIN_TIMESPAN: i64 = 690;
pub const MULTISHIELD_V4_MAX_TIMESPAN: i64 = 870;
pub const MULTISHIELD_V4_LOCAL_ADJUSTMENT_PERCENT: u32 = 4;
pub const DGB_NUM_ALGOS: i64 = 5;

/// DigiByte mainnet powLimit: `~uint256(0) >> 20`.
pub const DGB_MAINNET_POW_LIMIT: Target256 = Target256::from_le_limbs([
    0xffff_ffff,
    0xffff_ffff,
    0xffff_ffff,
    0xffff_ffff,
    0xffff_ffff,
    0xffff_ffff,
    0xffff_ffff,
    0x0000_0fff,
]);

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[repr(i8)]
pub enum DgbAlgo {
    Sha256d = 0,
    Scrypt = 1,
    Groestl = 2,
    Skein = 3,
    Qubit = 4,
    Odo = 7,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct DgbHeaderMeta {
    pub height: u32,
    pub timestamp: u32,
    pub bits: CompactTarget,
    pub algo: DgbAlgo,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DgbConsensusError {
    InsufficientHistory { required: usize, actual: usize },
    NonContiguousHistory { previous_height: u32, next_height: u32 },
    MissingPreviousAlgo { algo: DgbAlgo },
    Target(PrimitiveError),
}

impl fmt::Display for DgbConsensusError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InsufficientHistory { required, actual } => {
                write!(f, "insufficient DGB history: need at least {required}, got {actual}")
            }
            Self::NonContiguousHistory { previous_height, next_height } => write!(
                f,
                "non-contiguous DGB history between heights {previous_height} and {next_height}"
            ),
            Self::MissingPreviousAlgo { algo } => {
                write!(f, "history contains no previous block for algorithm {algo:?}")
            }
            Self::Target(err) => write!(f, "target conversion failed: {err}"),
        }
    }
}

impl std::error::Error for DgbConsensusError {}

impl From<PrimitiveError> for DgbConsensusError {
    fn from(value: PrimitiveError) -> Self {
        Self::Target(value)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct DgbMultiShieldV4 {
    pow_limit: Target256,
}

impl Default for DgbMultiShieldV4 {
    fn default() -> Self {
        Self::mainnet()
    }
}

impl DgbMultiShieldV4 {
    pub const fn new(pow_limit: Target256) -> Self {
        Self { pow_limit }
    }

    pub const fn mainnet() -> Self {
        Self::new(DGB_MAINNET_POW_LIMIT)
    }

    /// Reproduce DigiByte Core v9.26.5 `GetNextWorkRequiredV4`.
    ///
    /// `history` must be a contiguous oldest-to-newest main-chain suffix ending
    /// at `pindexLast`. At least 61 blocks are required so the kernel can derive
    /// both median-time-past values from raw timestamps: tip, tip-50, and the ten
    /// ancestors needed for the older MTP. More history may be supplied so a
    /// previous block of `algo` can be found even after an unusually long gap.
    pub fn next_work_required(
        &self,
        history: &[DgbHeaderMeta],
        algo: DgbAlgo,
    ) -> Result<CompactTarget, DgbConsensusError> {
        self.validate_history(history)?;

        let tip_index = history.len() - 1;
        let first_index = tip_index - MULTISHIELD_V4_WINDOW_BLOCKS;
        let tip = history[tip_index];

        let prev_algo = history
            .iter()
            .rev()
            .find(|block| block.algo == algo)
            .copied()
            .ok_or(DgbConsensusError::MissingPreviousAlgo { algo })?;

        let tip_mtp = median_time_past(history, tip_index);
        let first_mtp = median_time_past(history, first_index);
        let raw_timespan = tip_mtp - first_mtp;

        // C++ signed integer division truncates toward zero; Rust's i64 `/` has
        // the same semantics. Preserve the exact operation order from Core.
        let damped = MULTISHIELD_V4_TARGET_TIMESPAN
            + (raw_timespan - MULTISHIELD_V4_TARGET_TIMESPAN) / 4;
        let actual_timespan = damped.clamp(
            MULTISHIELD_V4_MIN_TIMESPAN,
            MULTISHIELD_V4_MAX_TIMESPAN,
        );

        let mut target = Target256::from_compact(prev_algo.bits)?;
        target = target
            .mul_u32(actual_timespan as u32)
            .div_u32(MULTISHIELD_V4_TARGET_TIMESPAN as u32);

        let adjustments = i64::from(prev_algo.height)
            + DGB_NUM_ALGOS
            - 1
            - i64::from(tip.height);

        if adjustments > 0 {
            for _ in 0..adjustments {
                target = target
                    .mul_u32(100)
                    .div_u32(100 + MULTISHIELD_V4_LOCAL_ADJUSTMENT_PERCENT);
            }
        } else if adjustments < 0 {
            for _ in 0..(-adjustments) {
                target = target
                    .mul_u32(100 + MULTISHIELD_V4_LOCAL_ADJUSTMENT_PERCENT)
                    .div_u32(100);
                if target > self.pow_limit {
                    target = self.pow_limit;
                    break;
                }
            }
        }

        if target > self.pow_limit {
            target = self.pow_limit;
        }

        Ok(target.to_compact())
    }

    fn validate_history(&self, history: &[DgbHeaderMeta]) -> Result<(), DgbConsensusError> {
        if history.len() < MULTISHIELD_V4_REQUIRED_HISTORY {
            return Err(DgbConsensusError::InsufficientHistory {
                required: MULTISHIELD_V4_REQUIRED_HISTORY,
                actual: history.len(),
            });
        }
        for pair in history.windows(2) {
            if pair[1].height != pair[0].height + 1 {
                return Err(DgbConsensusError::NonContiguousHistory {
                    previous_height: pair[0].height,
                    next_height: pair[1].height,
                });
            }
        }
        Ok(())
    }
}

fn median_time_past(history: &[DgbHeaderMeta], index: usize) -> i64 {
    let start = index.saturating_sub(MTP_WINDOW_BLOCKS - 1);
    let mut timestamps = history[start..=index]
        .iter()
        .map(|block| i64::from(block.timestamp))
        .collect::<Vec<_>>();
    timestamps.sort_unstable();
    timestamps[timestamps.len() / 2]
}

#[cfg(test)]
mod tests {
    use super::*;

    const BASE_HEIGHT: u32 = 2_000_000;
    const BASE_TIME: u32 = 1_700_000_000;
    const BASE_BITS: CompactTarget = CompactTarget(0x190d1fce);

    fn steady_history(spacing: u32) -> Vec<DgbHeaderMeta> {
        // Place SHA256d four blocks behind the tip, which is the equilibrium
        // position for a five-algorithm chain and therefore gives 0 local steps.
        let cycle = [
            DgbAlgo::Scrypt,
            DgbAlgo::Sha256d,
            DgbAlgo::Skein,
            DgbAlgo::Qubit,
            DgbAlgo::Odo,
        ];
        (0..MULTISHIELD_V4_REQUIRED_HISTORY)
            .map(|idx| DgbHeaderMeta {
                height: BASE_HEIGHT + idx as u32,
                timestamp: BASE_TIME + idx as u32 * spacing,
                bits: BASE_BITS,
                algo: cycle[idx % cycle.len()],
            })
            .collect()
    }

    #[test]
    fn equilibrium_preserves_target() {
        let history = steady_history(15);
        assert_eq!(
            DgbMultiShieldV4::mainnet()
                .next_work_required(&history, DgbAlgo::Sha256d)
                .unwrap(),
            BASE_BITS
        );
    }

    #[test]
    fn fast_window_clamps_to_690_seconds() {
        let history = steady_history(10);
        assert_eq!(
            DgbMultiShieldV4::mainnet()
                .next_work_required(&history, DgbAlgo::Sha256d)
                .unwrap(),
            CompactTarget(0x190c1305)
        );
    }

    #[test]
    fn slow_window_clamps_to_870_seconds() {
        let history = steady_history(30);
        assert_eq!(
            DgbMultiShieldV4::mainnet()
                .next_work_required(&history, DgbAlgo::Sha256d)
                .unwrap(),
            CompactTarget(0x190f395f)
        );
    }

    #[test]
    fn recent_sha_block_applies_four_hardening_steps() {
        let mut history = steady_history(15);
        history.last_mut().unwrap().algo = DgbAlgo::Sha256d;
        assert_eq!(
            DgbMultiShieldV4::mainnet()
                .next_work_required(&history, DgbAlgo::Sha256d)
                .unwrap(),
            CompactTarget(0x190b37f9)
        );
    }

    #[test]
    fn delayed_sha_block_applies_easing_step() {
        let mut history = (0..MULTISHIELD_V4_REQUIRED_HISTORY)
            .map(|idx| DgbHeaderMeta {
                height: BASE_HEIGHT + idx as u32,
                timestamp: BASE_TIME + idx as u32 * 15,
                bits: BASE_BITS,
                algo: DgbAlgo::Scrypt,
            })
            .collect::<Vec<_>>();
        let sha_index = history.len() - 6;
        history[sha_index].algo = DgbAlgo::Sha256d;
        assert_eq!(
            DgbMultiShieldV4::mainnet()
                .next_work_required(&history, DgbAlgo::Sha256d)
                .unwrap(),
            CompactTarget(0x190da632)
        );
    }

    #[test]
    fn history_must_be_contiguous() {
        let mut history = steady_history(15);
        history[30].height += 1;
        assert!(matches!(
            DgbMultiShieldV4::mainnet().next_work_required(&history, DgbAlgo::Sha256d),
            Err(DgbConsensusError::NonContiguousHistory { .. })
        ));
    }

    #[test]
    fn history_must_cover_both_mtp_windows() {
        let history = steady_history(15);
        let short = &history[1..];
        assert!(matches!(
            DgbMultiShieldV4::mainnet().next_work_required(short, DgbAlgo::Sha256d),
            Err(DgbConsensusError::InsufficientHistory { required: 61, actual: 60 })
        ));
    }
}
