use bevy::prelude::*;
use bevy_kira_audio::prelude::*;

use crate::app::markers::{LaneReceptor, LaneReceptorFlash};
use crate::app::state::{PauseState, is_paused};
use crate::audio::*;
use crate::config::HitSoundPriority;
use crate::gameplay::clock::ChartClock;
use crate::gameplay::constants::*;
use crate::gameplay::gauge::apply_gauge;
use crate::gameplay::input::flash_lane_receptor;
use crate::gameplay::interp::RenderVisualClock;
use crate::gameplay::layout::PlayfieldLayout;
use crate::gameplay::rendering::playfield_viz::spawn_hit_burst;
use crate::gameplay::run::RunState;
use dtxpt::chart::{Chart, Judgement, NoteState, chart_notes_complete, resolve_empty_hit_sound};
use dtxpt::input::bindings::DrumLane;
use dtxpt::input::lanes::{
    LANE_CY, LANE_LC, LANE_RD, PadGroup, lane_pad_group, lane_to_dtx_channel,
    pad_group_lanes_for_search,
};

#[allow(clippy::too_many_arguments)]
pub(crate) fn process_lane_hit(
    lane: usize,
    elapsed: f32,
    chart: &mut Chart,
    run: &mut RunState,
    commands: &mut Commands,
    layout: &PlayfieldLayout,
    render_clock: &RenderVisualClock,
    frame: u64,
    sound_bank: &SoundBank,
    mix: &AudioMix,
    audio: &Audio,
    audio_instances: &mut Assets<AudioInstance>,
    active: &mut ActiveSounds,
    rng: &mut GameRng,
    lane_receptors: &mut Query<(&LaneReceptor, &mut Sprite, &mut LaneReceptorFlash)>,
) {
    let elapsed = elapsed_with_pedal_lag(lane, elapsed, run.pedal_lag_time_ms);
    let judgement_lanes = judgement_lanes_for_hit(lane, run.cymbal_free);
    let hit_sound_priority = hit_sound_priority_for_lane(lane, run);
    let lanes_with_notes: std::collections::HashSet<usize> =
        chart.notes.iter().map(|note| note.lane).collect();
    let chart_has_lane = |lane_index: usize| lanes_with_notes.contains(&lane_index);
    let search_lanes = |lane_index: usize| -> Vec<usize> {
        if hit_sound_priority == HitSoundPriority::PadOverChip
            && let Some(group) = lane_pad_group(lane_index)
        {
            return pad_group_lanes_for_search(group, lane_index, chart_has_lane);
        }
        vec![lane_index]
    };

    let mut best: Option<(usize, f32)> = None;
    for (index, note) in chart.notes.iter().enumerate() {
        if !judgement_lanes.contains(&note.lane) || note.state != NoteState::Pending {
            continue;
        }
        let delta = elapsed - note.time;
        if delta.abs() <= Judgement::POOR_WINDOW
            && best.is_none_or(|(_, best_delta)| delta.abs() < best_delta.abs())
        {
            best = Some((index, delta));
        }
    }

    if let Some((index, delta)) = best {
        if let Some(judgement) = Judgement::from_delta(delta) {
            chart.notes[index].state = NoteState::Hit(judgement);
            apply_judgement(run, judgement, delta, chart.notes.len(), false);
            let hit_y = layout.note_y(
                chart.notes[index].time,
                render_clock.current,
                run.lane_speed,
            );
            spawn_hit_burst(commands, layout, lane, judgement, hit_y);
            let playback_rate = (judgement == Judgement::Poor).then(|| dtx_bad_playback_rate(rng));
            if run.drum_hit_sound {
                let sound_index = resolve_hit_sound_note_index(
                    chart,
                    lane,
                    index,
                    elapsed,
                    hit_sound_priority,
                    &judgement_lanes,
                    chart_has_lane,
                );
                let sound_note = &chart.notes[sound_index];
                play_drum_sound(
                    sound_note.wav_id,
                    sound_note.channel,
                    lane,
                    playback_rate,
                    run.song_playback_rate,
                    frame,
                    run.lp_muting,
                    sound_bank,
                    mix,
                    audio,
                    audio_instances,
                    active,
                );
            }
        }
    } else {
        flash_lane_receptor(lane, lane_receptors);

        let lanes = search_lanes(lane);
        let (nearest_wav, channel) =
            resolve_empty_hit_sound(&chart.empty_hit_events, lane, &lanes, elapsed)
                .or_else(|| {
                    find_nearest_chart_note_for_empty_hit(&chart.notes, &lanes, elapsed)
                        .map(|n| (n.wav_id, n.channel))
                })
                .unwrap_or((None, lane_to_dtx_channel(lane)));
        if run.drum_hit_sound {
            play_drum_sound(
                nearest_wav,
                channel,
                lane,
                None,
                run.song_playback_rate,
                frame,
                run.lp_muting,
                sound_bank,
                mix,
                audio,
                audio_instances,
                active,
            );
        }
    }
}

fn elapsed_with_pedal_lag(lane: usize, elapsed: f32, pedal_lag_time_ms: i32) -> f32 {
    if is_pedal_lane(lane) {
        elapsed + pedal_lag_time_ms as f32 / 1000.0
    } else {
        elapsed
    }
}

fn is_pedal_lane(lane: usize) -> bool {
    matches!(
        DrumLane::from_index(lane),
        Some(DrumLane::Bd | DrumLane::Lp | DrumLane::Lbd)
    )
}

fn judgement_lanes_for_hit(lane: usize, cymbal_free: bool) -> Vec<usize> {
    if cymbal_free && matches!(lane, LANE_CY | LANE_RD | LANE_LC) {
        vec![LANE_CY, LANE_RD, LANE_LC]
    } else {
        vec![lane]
    }
}

fn hit_sound_priority_for_lane(lane: usize, run: &RunState) -> HitSoundPriority {
    match lane_pad_group(lane) {
        Some(PadGroup::Hh) => run.hit_sound_priority_hh,
        Some(PadGroup::Tom) => run.hit_sound_priority_ft,
        Some(PadGroup::Cymbal) => run.hit_sound_priority_cy,
        Some(PadGroup::Bd) => run.hit_sound_priority_lp,
        None => HitSoundPriority::ChipOverPad,
    }
}

fn resolve_hit_sound_note_index(
    chart: &Chart,
    lane: usize,
    hit_index: usize,
    elapsed: f32,
    priority: HitSoundPriority,
    judgement_lanes: &[usize],
    chart_has_lane: impl Fn(usize) -> bool,
) -> usize {
    if priority == HitSoundPriority::ChipOverPad {
        return hit_index;
    }
    let lanes = if judgement_lanes.len() > 1 {
        judgement_lanes.to_vec()
    } else {
        pad_group_lanes_for_search(
            lane_pad_group(lane).expect("pad group lane"),
            lane,
            chart_has_lane,
        )
    };
    find_nearest_pending_note_index(&chart.notes, &lanes, elapsed).unwrap_or(hit_index)
}

fn find_nearest_pending_note_index(
    notes: &[dtxpt::chart::ChartNote],
    lanes: &[usize],
    elapsed: f32,
) -> Option<usize> {
    let mut best: Option<(usize, f32)> = None;
    for (index, note) in notes.iter().enumerate() {
        if !lanes.contains(&note.lane) || note.state != NoteState::Pending {
            continue;
        }
        let delta = elapsed - note.time;
        if delta.abs() <= Judgement::POOR_WINDOW
            && best.is_none_or(|(_, best_delta)| delta.abs() < best_delta.abs())
        {
            best = Some((index, delta));
        }
    }
    best.map(|(index, _)| index)
}

#[cfg(test)]
fn find_nearest_pending_note<'a>(
    notes: &'a [dtxpt::chart::ChartNote],
    lanes: &[usize],
    elapsed: f32,
) -> Option<&'a dtxpt::chart::ChartNote> {
    find_nearest_pending_note_index(notes, lanes, elapsed).map(|index| &notes[index])
}

/// DTXMania `r指定時刻に一番近いChip_ヒット未済問わず不可視考慮`: nearest note on lane(s)
/// for empty-pad hits, without judgement window or note-state filtering.
fn find_nearest_chart_note_for_empty_hit<'a>(
    notes: &'a [dtxpt::chart::ChartNote],
    lanes: &[usize],
    elapsed: f32,
) -> Option<&'a dtxpt::chart::ChartNote> {
    notes
        .iter()
        .filter(|note| lanes.contains(&note.lane))
        .min_by(|a, b| {
            (elapsed - a.time)
                .abs()
                .total_cmp(&(elapsed - b.time).abs())
        })
}

pub fn miss_late_notes(
    mut chart: ResMut<Chart>,
    pause_state: Res<State<PauseState>>,
    mut run: ResMut<RunState>,
) {
    if run.finished
        || run.failed
        || is_paused(pause_state.get())
        || chart_notes_complete(&chart.notes)
    {
        return;
    }

    let elapsed = run.elapsed;
    let mut missed = 0;
    for note in chart.notes.iter_mut() {
        if note.state == NoteState::Pending && elapsed - note.time > Judgement::POOR_WINDOW {
            note.state = NoteState::Missed;
            missed += 1;
        }
    }

    for _ in 0..missed {
        apply_judgement(
            &mut run,
            Judgement::Miss,
            Judgement::POOR_WINDOW,
            chart.notes.len(),
            false,
        );
    }
}

/// Autoplay: simulate a perfect hit on every pending note whose lane is in
/// `active_mods.auto_lanes` and whose audio-clock time has passed. Lanes
/// not in the auto set are left for the player's input to hit (or to miss
/// via `miss_late_notes`).
///
/// Directly applies the Perfect judgement + visual burst + drum sound for each
/// due note, mirroring the success branch of process_lane_hit but without the
/// lane-search (the target note is pre-selected).
#[allow(clippy::too_many_arguments, clippy::type_complexity)]
pub(crate) fn autoplay_hit_notes(
    pause_state: Res<State<PauseState>>,
    mut chart: ResMut<Chart>,
    mut run: ResMut<RunState>,
    mut commands: Commands,
    layout: Res<PlayfieldLayout>,
    clock: Res<ChartClock>,
    render_clock: Res<RenderVisualClock>,
    mut hit_audio: crate::gameplay::input::LaneHitAudio,
    mut flashes: ParamSet<(Query<(&LaneReceptor, &mut Sprite, &mut LaneReceptorFlash)>,)>,
) {
    if run.finished
        || run.failed
        || is_paused(pause_state.get())
        || run.active_mods.auto_lanes.is_empty()
        || chart_notes_complete(&chart.notes)
    {
        return;
    }

    let now_audio = clock.audio_elapsed + run.timing_offset;
    let total_notes = chart.notes.len();

    // Collect due notes first to avoid borrow conflicts (we mutate chart below).
    let mut due: Vec<usize> = Vec::new();
    for (i, note) in chart.notes.iter().enumerate() {
        if note.state != NoteState::Pending || now_audio < note.time {
            continue;
        }
        let lane = match DrumLane::from_index(note.lane) {
            Some(l) => l,
            None => continue,
        };
        if run.active_mods.auto_lanes.contains(&lane) {
            due.push(i);
        }
    }
    if due.is_empty() {
        return;
    }
    // Process in chart order; multiple notes can be due in the same frame
    // after a slow frame or a seek into dense section.
    due.sort_unstable();

    for index in due {
        let delta = 0.0_f32; // exact hit -> Perfect
        let judgement = Judgement::Perfect;
        chart.notes[index].state = NoteState::Hit(judgement);
        chart.notes[index].autoplayed = true;
        apply_judgement(&mut run, judgement, delta, total_notes, true);
        let hit_y = layout.note_y(
            chart.notes[index].time,
            render_clock.current,
            run.lane_speed,
        );
        let lane = chart.notes[index].lane;
        spawn_hit_burst(&mut commands, &layout, lane, judgement, hit_y);
        flash_lane_receptor(lane, &mut flashes.p0());
        if run.drum_hit_sound {
            let sound_note = &chart.notes[index];
            play_drum_sound(
                sound_note.wav_id,
                sound_note.channel,
                lane,
                None,
                run.song_playback_rate,
                hit_audio.frame.0,
                run.lp_muting,
                &hit_audio.sound_bank,
                &hit_audio.mix,
                &hit_audio.audio,
                &mut hit_audio.audio_instances,
                &mut hit_audio.active,
            );
        }
    }
}

pub fn apply_judgement(
    run: &mut RunState,
    judgement: Judgement,
    delta: f32,
    total_notes: usize,
    autoplayed: bool,
) {
    run.last_judgement = judgement;
    run.last_message = judgement.label().into();
    run.last_was_auto = autoplayed;
    run.last_delta_ms = delta * 1000.0;
    run.judgement_timer = Timer::from_seconds(JUDGEMENT_SECS, TimerMode::Once);
    run.judge_units += judgement.weight();

    match judgement {
        Judgement::Perfect => run.perfect += 1,
        Judgement::Great => run.great += 1,
        Judgement::Good => run.good += 1,
        Judgement::Poor => run.poor += 1,
        Judgement::Miss => run.miss += 1,
    }

    if judgement.keeps_combo() {
        run.combo += 1;
        run.max_combo = run.max_combo.max(run.combo);
    } else {
        run.combo = 0;
    }

    // DTXManiaNX-style drum score core (SkillMode=1, no bonus chips):
    // base = 1,000,000 / (1275 + 50 * (MAXCOMBO - 50));
    // Perfect=base, Great=base*0.5, Good=base*0.2, Poor/Miss=0;
    // then multiply by current combo, capped at 50.
    let max_combo = total_notes as f32;
    let base = TARGET_SCORE / (1275.0 + 50.0 * (max_combo - 50.0));
    let judge_factor = match judgement {
        Judgement::Perfect => 1.0,
        Judgement::Great => 0.5,
        Judgement::Good => 0.2,
        Judgement::Poor | Judgement::Miss => 0.0,
    };
    let combo_factor = run.combo.min(50) as f32;
    run.score += base * judge_factor * combo_factor;

    if run.perfect as usize == total_notes {
        run.score = TARGET_SCORE;
    }

    apply_gauge(run, judgement, autoplayed);
}

#[cfg(test)]
mod tests {
    use super::*;
    use dtxpt::chart::ChartNote;

    fn note(time: f32, lane: usize, wav: u32, state: NoteState) -> ChartNote {
        ChartNote {
            time,
            lane,
            channel: 0x11 + lane as u32,
            wav_id: Some(wav),
            state,
            autoplayed: false,
        }
    }

    #[test]
    fn empty_hit_finds_nearest_note_beyond_judge_window() {
        let notes = vec![
            note(10.0, 0, 1, NoteState::Pending),
            note(20.0, 0, 2, NoteState::Pending),
        ];
        let nearest = find_nearest_chart_note_for_empty_hit(&notes, &[0], 16.0).unwrap();
        assert_eq!(nearest.wav_id, Some(2));
    }

    #[test]
    fn empty_hit_considers_hit_and_missed_notes() {
        let notes = vec![
            note(10.0, 0, 1, NoteState::Hit(Judgement::Perfect)),
            note(20.0, 0, 2, NoteState::Missed),
        ];
        let nearest = find_nearest_chart_note_for_empty_hit(&notes, &[0], 12.0).unwrap();
        assert_eq!(nearest.wav_id, Some(1));
    }

    #[test]
    fn pending_note_search_stays_within_poor_window() {
        let notes = vec![
            note(10.0, 0, 1, NoteState::Pending),
            note(20.0, 0, 2, NoteState::Pending),
        ];
        assert!(find_nearest_pending_note(&notes, &[0], 15.0).is_none());
    }

    #[test]
    fn empty_hit_prefers_chart_nosound_over_nearest_note() {
        use dtxpt::chart::EmptyHitEvent;

        let events = vec![EmptyHitEvent {
            time: 0.0,
            lane: 0,
            channel: 0xB3,
            wav_id: Some(99),
        }];
        let sound = resolve_empty_hit_sound(&events, 0, &[0], 5.0).unwrap();
        assert_eq!(sound, (Some(99), 0xB3));
    }

    #[test]
    fn pedal_lag_shifts_only_pedal_lanes() {
        assert!((elapsed_with_pedal_lag(DrumLane::Bd.index(), 10.0, 25) - 10.025).abs() < 0.001);
        assert!((elapsed_with_pedal_lag(DrumLane::Lp.index(), 10.0, -10) - 9.99).abs() < 0.001);
        assert_eq!(elapsed_with_pedal_lag(DrumLane::Sd.index(), 10.0, 25), 10.0);
    }

    #[test]
    fn pedal_lag_ignores_unknown_lane() {
        assert_eq!(elapsed_with_pedal_lag(usize::MAX, 10.0, 25), 10.0);
    }

    #[test]
    fn cymbal_free_expands_cymbal_judgement_lanes() {
        assert_eq!(
            judgement_lanes_for_hit(DrumLane::Cy.index(), true),
            vec![
                DrumLane::Cy.index(),
                DrumLane::Rd.index(),
                DrumLane::Lc.index()
            ]
        );
        assert_eq!(
            judgement_lanes_for_hit(DrumLane::Rd.index(), true),
            vec![
                DrumLane::Cy.index(),
                DrumLane::Rd.index(),
                DrumLane::Lc.index()
            ]
        );
        assert_eq!(
            judgement_lanes_for_hit(DrumLane::Lc.index(), true),
            vec![
                DrumLane::Cy.index(),
                DrumLane::Rd.index(),
                DrumLane::Lc.index()
            ]
        );
    }

    #[test]
    fn cymbal_free_disabled_keeps_exact_lane() {
        assert_eq!(
            judgement_lanes_for_hit(DrumLane::Cy.index(), false),
            vec![DrumLane::Cy.index()]
        );
        assert_eq!(
            judgement_lanes_for_hit(DrumLane::Lc.index(), false),
            vec![DrumLane::Lc.index()]
        );
        assert_eq!(
            judgement_lanes_for_hit(DrumLane::Sd.index(), true),
            vec![DrumLane::Sd.index()]
        );
    }

    #[test]
    fn apply_judgement_autoplayed_sets_last_was_auto() {
        use crate::config::GameConfig;
        use crate::gameplay::run::RunState;

        let mut run = RunState::from_config(&GameConfig::default());
        assert!(!run.last_was_auto);
        apply_judgement(&mut run, Judgement::Perfect, 0.0, 1, true);
        assert!(run.last_was_auto);
        assert_eq!(run.last_judgement, Judgement::Perfect);
        apply_judgement(&mut run, Judgement::Great, 0.04, 1, false);
        assert!(!run.last_was_auto);
        assert_eq!(run.last_judgement, Judgement::Great);
    }

    #[test]
    fn reconcile_restart_clears_autoplayed_flag() {
        use dtxpt::chart::{NoteState, reconcile_notes_for_restart};
        let mut notes = vec![note(1.0, 0, 1, NoteState::Hit(Judgement::Perfect))];
        notes[0].autoplayed = true;
        reconcile_notes_for_restart(&mut notes);
        assert!(matches!(notes[0].state, NoteState::Pending));
        assert!(!notes[0].autoplayed);
    }
}
