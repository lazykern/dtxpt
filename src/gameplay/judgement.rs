use bevy::prelude::*;
use bevy_kira_audio::prelude::*;

use crate::app::markers::{LaneReceptor, LaneReceptorFlash};
use crate::app::state::{PauseState, is_paused};
use crate::audio::*;
use crate::gameplay::clock::ChartClock;
use crate::gameplay::constants::*;
use crate::gameplay::gauge::apply_gauge;
use crate::gameplay::input::flash_lane_receptor;
use crate::gameplay::layout::PlayfieldLayout;
use crate::gameplay::rendering::playfield_viz::spawn_hit_burst;
use crate::gameplay::run::RunState;
use dtxpt::chart::{Chart, Judgement, NoteState, chart_notes_complete};
use dtxpt::input::lanes::lane_to_dtx_channel;

#[allow(clippy::too_many_arguments)]
pub(crate) fn process_lane_hit(
    lane: usize,
    elapsed: f32,
    chart: &mut Chart,
    run: &mut RunState,
    commands: &mut Commands,
    layout: &PlayfieldLayout,
    clock: &ChartClock,
    frame: u64,
    sound_bank: &SoundBank,
    mix: &AudioMix,
    audio: &Audio,
    audio_instances: &mut Assets<AudioInstance>,
    active: &mut ActiveSounds,
    rng: &mut GameRng,
    lane_receptors: &mut Query<(&LaneReceptor, &mut Sprite, &mut LaneReceptorFlash)>,
) {
    let mut best: Option<(usize, f32)> = None;
    for (index, note) in chart.notes.iter().enumerate() {
        if note.lane != lane || note.state != NoteState::Pending {
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
            apply_judgement(run, judgement, delta, chart.notes.len());
            let hit_y = layout.note_y(
                chart.notes[index].time,
                clock.visual_elapsed,
                run.lane_speed,
            );
            spawn_hit_burst(commands, layout, lane, judgement, hit_y);
            let playback_rate = (judgement == Judgement::Poor).then(|| dtx_bad_playback_rate(rng));
            play_drum_sound(
                chart.notes[index].wav_id,
                chart.notes[index].channel,
                lane,
                playback_rate,
                run.song_playback_rate,
                frame,
                sound_bank,
                mix,
                audio,
                audio_instances,
                active,
            );
        }
    } else {
        flash_lane_receptor(lane, lane_receptors);

        let nearest = chart
            .notes
            .iter()
            .filter(|n| n.lane == lane && n.state == NoteState::Pending)
            .min_by(|a, b| {
                (elapsed - a.time)
                    .abs()
                    .total_cmp(&(elapsed - b.time).abs())
            });
        let (nearest_wav, channel) = nearest
            .map(|n| (n.wav_id, n.channel))
            .unwrap_or((None, lane_to_dtx_channel(lane)));
        play_drum_sound(
            nearest_wav,
            channel,
            lane,
            None,
            run.song_playback_rate,
            frame,
            sound_bank,
            mix,
            audio,
            audio_instances,
            active,
        );
    }
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
        );
    }
}

pub fn apply_judgement(run: &mut RunState, judgement: Judgement, delta: f32, total_notes: usize) {
    run.last_judgement = judgement;
    run.last_message = judgement.label().into();
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

    apply_gauge(run, judgement);
}
