//! Sub-frame interpolation between FixedUpdate ticks.
//!
//! Bevy's `FixedUpdate` schedule runs at a fixed cadence (e.g. 60Hz). A render
//! frame at higher rate (e.g. 144Hz) will see 0, 1, or 2 fixed ticks during
//! the same span. To produce sub-frame visual motion, we lerp between the
//! values saved at the start and end of the latest fixed tick, using
//! `Time<Fixed>::overstep_fraction()` as the alpha.
//!
//! `interp_visual_clock` runs in `RunFixedMainLoop::AfterFixedMainLoop` and
//! snapshots `prev_visual_elapsed` and `visual_elapsed` from `ChartClock` into
//! `RenderVisualClock`. Render systems read `RenderVisualClock` for note
//! position; they read `ChartClock::visual_elapsed` only for spawn windows and
//! SE scheduling (which need the *current tick end* value, not the
//! interpolated value).
//!
//! See `docs/plans/fixedupdate-refactor.md` for the full design.

use bevy::prelude::*;

use crate::gameplay::clock::ChartClock;

/// Snapshot of the visual clock at the boundaries of the most recent fixed
/// tick, plus the sub-frame alpha for interpolation.
///
/// `current` is the value of `ChartClock::visual_elapsed` at the end of the
/// most recent fixed tick. `prev` is the value at the start of the same tick
/// (i.e. the end of the previous tick). `alpha` is
/// `Time<Fixed>::overstep_fraction()` — how far into the current fixed tick we
/// are at the moment of the snapshot.
///
/// Render: `lerp(prev, current, alpha)` gives the sub-frame visual position.
#[derive(Resource, Debug, Clone, Copy, Default)]
pub struct RenderVisualClock {
    /// `ChartClock::prev_visual_elapsed` at the end of the most recent fixed tick.
    pub prev: f32,
    /// `ChartClock::visual_elapsed` at the end of the most recent fixed tick.
    pub current: f32,
    /// `Time<Fixed>::overstep_fraction()` at snapshot time. In [0.0, 1.0).
    pub alpha: f32,
}

impl RenderVisualClock {
    /// Sub-frame interpolated visual clock value. `lerp(prev, current, alpha)`.
    pub fn now(&self) -> f32 {
        self.prev + (self.current - self.prev) * self.alpha
    }
}

/// Snapshot the visual clock at the end of every fixed tick. Runs in
/// `RunFixedMainLoop::AfterFixedMainLoop` so it sees the latest fixed tick's
/// output and runs before Update render systems.
///
/// Reads `Time<Fixed>::overstep_fraction()` for sub-frame alpha. Reads
/// `ChartClock::prev_visual_elapsed` and `ChartClock::visual_elapsed` for the
/// tick boundary values.
pub(crate) fn interp_visual_clock(
    fixed_time: Res<Time<Fixed>>,
    clock: Res<ChartClock>,
    mut render: ResMut<RenderVisualClock>,
) {
    render.prev = clock.prev_visual_elapsed;
    render.current = clock.visual_elapsed;
    render.alpha = fixed_time.overstep_fraction();
}
