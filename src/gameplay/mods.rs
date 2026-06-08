use std::collections::BTreeSet;

use serde::{Deserialize, Serialize};

use dtxpt::input::bindings::DrumLane;

/// How the per-lane auto set is derived for a run. Picked in song-select
/// (or settings). Persistent in `GameConfig` so the user's last choice
/// is remembered. See `docs/reference/dimensions.md`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum AutoMode {
    /// No auto for this run. `active_mods.auto_lanes` resolves to empty.
    Off,
    /// Use the user's saved per-lane config as-is.
    /// `active_mods.auto_lanes` = `per_lane_auto`.
    #[default]
    PerLane,
    /// All 10 lanes auto for this run. Overrides the per-lane config for
    /// the duration of the run; user's saved config is preserved untouched.
    AllAuto,
}

impl AutoMode {
    pub fn label(self) -> &'static str {
        match self {
            Self::Off => "Off",
            Self::PerLane => "Per-lane",
            Self::AllAuto => "All auto",
        }
    }

    pub fn next(self) -> Self {
        match self {
            Self::Off => Self::PerLane,
            Self::PerLane => Self::AllAuto,
            Self::AllAuto => Self::Off,
        }
    }
}

/// Resolve the effective auto set for a run, given the user's saved
/// per-lane config and the picked `AutoMode`. Source of truth during
/// the run is the returned set; the user's `per_lane_auto` is not
/// mutated by this call.
pub fn resolve_auto_lanes(per_lane: &BTreeSet<DrumLane>, mode: AutoMode) -> BTreeSet<DrumLane> {
    match mode {
        AutoMode::Off => BTreeSet::new(),
        AutoMode::PerLane => per_lane.clone(),
        AutoMode::AllAuto => DrumLane::ALL.iter().copied().collect(),
    }
}

/// A mod is a session-local ruleset change that **affects the score**.
/// In dtxpt v0.2 the only mod is the per-lane autoplay set. See
/// `docs/reference/dimensions.md#mods` for the rationale.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ModSet {
    /// Lanes that are pre-emptively auto-played. Empty = no auto.
    pub auto_lanes: BTreeSet<DrumLane>,
}

impl ModSet {
    pub fn is_all_lanes(&self) -> bool {
        self.auto_lanes.len() == DrumLane::ALL.len()
    }

    /// Display label for the run, derived from the effective set.
    /// Used by the HUD and result screen.
    pub fn display_label(&self, practice: bool) -> &'static str {
        let base = if self.auto_lanes.is_empty() {
            "Normal"
        } else if self.is_all_lanes() {
            "All auto"
        } else {
            "Custom"
        };
        if practice && self.auto_lanes.is_empty() {
            "Practice"
        } else if practice && self.is_all_lanes() {
            "All auto + Practice"
        } else if practice {
            "Custom + Practice"
        } else {
            base
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resolve_off_yields_empty() {
        let per_lane = BTreeSet::from([DrumLane::Hh]);
        assert!(resolve_auto_lanes(&per_lane, AutoMode::Off).is_empty());
    }

    #[test]
    fn resolve_per_lane_clones_user_config() {
        let per_lane = BTreeSet::from([DrumLane::Bd, DrumLane::Hh]);
        let resolved = resolve_auto_lanes(&per_lane, AutoMode::PerLane);
        assert_eq!(resolved, per_lane);
        // The user's config is not mutated by the resolution.
        assert_eq!(per_lane.len(), 2);
    }

    #[test]
    fn resolve_all_auto_yields_all_lanes_without_mutating_user() {
        let per_lane = BTreeSet::from([DrumLane::Hh]);
        let resolved = resolve_auto_lanes(&per_lane, AutoMode::AllAuto);
        assert_eq!(resolved.len(), DrumLane::ALL.len());
        // User's per-lane config is preserved.
        assert_eq!(per_lane.len(), 1);
    }

    #[test]
    fn modset_label_derives_from_effective_set() {
        let mut modset = ModSet::default();
        assert_eq!(modset.display_label(false), "Normal");
        assert_eq!(modset.display_label(true), "Practice");
        modset.auto_lanes = BTreeSet::from([DrumLane::Hh]);
        assert_eq!(modset.display_label(false), "Custom");
        assert_eq!(modset.display_label(true), "Custom + Practice");
        modset.auto_lanes = DrumLane::ALL.iter().copied().collect();
        assert_eq!(modset.display_label(false), "All auto");
        assert_eq!(modset.display_label(true), "All auto + Practice");
    }

    #[test]
    fn automode_cycle_through_three_variants() {
        assert_eq!(AutoMode::default(), AutoMode::PerLane);
        assert_eq!(AutoMode::Off.next(), AutoMode::PerLane);
        assert_eq!(AutoMode::PerLane.next(), AutoMode::AllAuto);
        assert_eq!(AutoMode::AllAuto.next(), AutoMode::Off);
    }
}
