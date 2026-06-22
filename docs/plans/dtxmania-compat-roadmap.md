# dtxpt — DTXManiaNX Compat Roadmap

Date: 2026-06-21 (revised after re-check; status updated 2026-06-22,
second-pass status updated 2026-06-22)
Status: **current milestone complete through second pass (H1–H13)** —
A–F.4a from the first pass, plus BGAPAN swap pairs, result media,
stoic-mode animation suppression, StageClear/StageFail screens,
per-instrument BestRank, set.def/box.def sub-box recursion,
quick-config hotkey, and unfocused-sleep config fields all landed.
Three rows remain partial and documented in
`features_missing.md`: AVI playback renderer (blocked on a video
crate matching the system ffmpeg), lyrics / `#IF` conditionals, and
the actual frame-loop sleep wiring.
Source: [`../dev-notes/2026-06-21-dtxmania-feature-gap.md`](../dev-notes/2026-06-21-dtxmania-feature-gap.md) (the gap report this roadmap is derived from)
Replaces/supplements: [`full-game-roadmap.md`](full-game-roadmap.md) (which is the aspirational game design; this is the concrete port plan)

## Goal

Port **drum-mode DTXManiaNX** behaviour to dtxpt (Bevy 0.18.1), reaching
feature parity with `references/DTXmaniaNX-BocuD/` drum scope. After
compat is reached, redesign passes polish, accessibility, and Bevy-native
patterns in a second pass.

**Compat target = `references/DTXmaniaNX-BocuD/`, not upstream
DTXManiaNX.** BocuD is the live fork; upstream is largely dead.

## Non-goals (original drum-only roadmap; revised by BocuD port goal)

The active BocuD port goal expands this roadmap beyond its original
Drum-only compatibility scope. Guitar/Bass gameplay are now in scope and
have landed in Phase C; media/skin are Phase E work, not redesign-only.

- Video playback (AVI / MP4 / M4V) — Phase E must either integrate a decoder or document an explicit deferral rationale.
- Custom skin authoring UI — default skin/theme ships first; user skin authoring remains later.
- `Stage 09 ChangeSkin` — depends on skin system
- Automatic GitHub-updater (distribution story)
- Multi-language UI / i18n — single-language English UI for compat pass
- ImGui dev tools — keep debug HUD only; revisit in redesign

## Scope of "compat"

Three levels of parity:

1. **Behaviour parity** — same input mapping, same scoring, same rank
   thresholds, same judgement windows, same gauge behaviour. **Required.**
2. **Data parity** — `.score.ini` compatible with BocuD's 9-section
   schema; DTX directives parsed equivalently; chip kinds produce same
   visual output. **Required for chart-portability.**
3. **Visual parity** — same HUD layout, same animations, same skin
   assets. **Not required.** Use Bevy-native placeholders that are
   visually distinguishable but mechanically identical.

Compat pass measures success by: drop a real chart folder in
`chart_root`, see it in song select, see its `.score.ini` populate if
present, play it, save score in `.score.ini` format readable by BocuD.

## Source-of-truth references

Per feature, the implementation derives from:

| Feature | Reference file |
|---------|----------------|
| Stages | `references/DTXmaniaNX-BocuD/DTXMania/Stage/` |
| Score schema | `references/DTXmaniaNX-BocuD/DTXMania/Score,Song/CScoreIni.cs` |
| Score metadata (chart-attached) | `references/DTXmaniaNX-BocuD/DTXMania/Score,Song/CChartData.cs` |
| Config surface | `references/DTXmaniaNX-BocuD/DTXMania/Core/CConfigIni.cs` |
| Hit windows | `references/DTXmaniaNX-BocuD/DTXMania/Core/STHitRanges.cs` |
| Gameplay enums | `references/DTXmaniaNX-BocuD/DTXMania/Core/CConstants.cs` |
| Chart parser | `references/DTXmaniaNX-BocuD/DTXMania/Score,Song/CDTX.cs` |
| Channel taxonomy | `references/DTXmaniaNX-BocuD/DTXMania/Score,Song/EChannel.cs` |
| Song list node | `references/DTXmaniaNX-BocuD/DTXMania/Score,Song/CSongListNode.cs` |
| Song list selection UX | `references/DTXmaniaNX-BocuD/DTXMania/Stage/04.SongSelection/CActSelect*.cs` |
| Performance stage | `references/DTXmaniaNX-BocuD/DTXMania/Stage/06.Performance/` |
| Gauge model | `references/DTXmaniaNX-BocuD/DTXMania/Stage/06.Performance/CActPerfCommonGauge.cs` |
| Result stage | `references/DTXmaniaNX-BocuD/DTXMania/Stage/07.Result/CStageResult.cs` |

## Data-model expansion

Before any phase work, expand these core types. Documented once here,
referenced from each phase.

### `BestScore` (in `persistence/scores.rs`)

Add fields to match `CPerformanceEntry` for drums only:

```rust
pub struct BestScore {
    // existing
    pub score: u32,
    pub accuracy: f32,
    pub max_combo: u32,
    pub perfect: u32, pub great: u32, pub good: u32, pub poor: u32, pub miss: u32,

    // new — judgement counts excluding auto-play chips
    pub perfect_excl_auto: u32, pub great_excl_auto: u32,
    pub good_excl_auto: u32, pub poor_excl_auto: u32, pub miss_excl_auto: u32,

    // new — mod snapshot at save time
    pub mods: ModSnapshot,
    pub practice: bool,
    pub play_speed: (u32, u32),  // numerator / denominator
    pub dark: DarkMode,
    pub scroll_speed: f32,
    pub damage_level: DamageLevel,
    pub risky: u8,                // 0 = OFF, 1..=10
    pub hit_ranges_primary: HitRanges,
    pub hit_ranges_secondary: HitRanges,  // pedal-only for drums

    // new — skill rates (computed, not user-input)
    pub game_skill: f64,
    pub performance_skill: f64,
    pub skill_progress: String,    // arcade "progress" string

    // new — provenance
    pub hash: String,              // MD5 of DTX file at save time
    pub version: String,           // "dtxpt-X.Y.Z"
    pub saved_at: String,          // ISO 8601 timestamp
}
```

### `ModSnapshot` (new in `gameplay/mods.rs`)

```rust
pub struct ModSnapshot {
    pub hidden: bool,
    pub sudden: bool,
    pub left: bool,
    pub light: bool,
    pub reverse: bool,
    pub random_pad: RandomMode,
    pub random_pedal: RandomMode,
    pub tight: bool,
    pub hh_group: HhGroup,
    pub ft_group: FtGroup,
    pub cy_group: CyGroup,
    pub bd_group: BdGroup,
    pub hit_sound_priority: PlaybackPriority,
}
```

### `HitRanges` (new in `chart/model.rs`)

Mirror `STHitRanges`:

```rust
pub struct HitRanges {
    pub perfect_ms: i32,  // ±
    pub great_ms: i32,
    pub good_ms: i32,
    pub poor_ms: i32,
}

impl HitRanges {
    pub const DEFAULT_DTXMANIA: Self = Self {
        perfect_ms: 34, great_ms: 67, good_ms: 84, poor_ms: 117,
    };
    pub fn judge(&self, abs_ms: i32) -> Judgement { /* from STHitRanges.tGetJudgement */ }
}
```

### `ScoreStore` (in `persistence/scores.rs`)

Expand to mirror `CScoreIni`'s 9-section schema. Two storage forms:

- **`scores.ron` (global)** — keep current. Snapshot of best score per
  chart path for fast song-select status panel.
- **`<chart_dir>/.score.ini` (per-chart)** — new. Read/write using the
  BocuD INI format (`[File]`, `[HiScore.Drums]`, `[HiSkill.Drums]`,
  `[LastPlay.Drums]`, `History[5]`, etc.). Enables score portability
  with BocuD. Shift-JIS encoding for `strTitle` / `strArtist` /
  `strComment` round-trips (BocuD hardcodes Shift-JIS; we use UTF-8
  internally and transcode on write).

```rust
pub struct PerChartScore {
    pub file: FileMeta,           // Title, Name, Hash, History[5], BestRank
    pub hi_score_drums: PerformanceEntry,
    pub hi_skill_drums: PerformanceEntry,
    pub last_play_drums: PerformanceEntry,
}

pub struct FileMeta {
    pub title: String,
    pub name: String,
    pub hash: String,             // MD5 hex
    pub history: [String; 5],     // last 5 dates
    pub best_rank: ERank,
    pub play_count_drums: u32,
}
```

### `RunResult` → `PerformanceEntry`

Replace ad-hoc `RunResult` fields with the full BocuD-compatible
`PerformanceEntry` struct (drums only — guitar/bass fields omitted or
stubbed). This is what gets saved to both stores.

### `GaugeConfig`

```rust
pub struct GaugeConfig {
    pub mode: GaugeMode,          // Normal / Hard / Death / Extreme / EXHard
    pub damage_level: DamageLevel,
    pub risky_initial: u8,        // 0 = off, 1..=10 = Risky N
    pub auto_add_gauge: bool,     // auto-mode gauge recovery
}
```

### `GameplayConfig` (lives in `GameConfig`)

```rust
pub struct GameplayConfig {
    // existing
    pub lane_speed: f32, pub timing_offset: f32, pub song_rate: f32,
    pub practice: bool, pub lp_muting: bool, pub drum_hit_sound: bool,
    pub hit_sound_priority_hh/tom/cymbal/bd: PlaybackPriority,
    pub per_lane_auto: BTreeSet<DrumLane>, pub auto_mode: AutoMode,

    // new
    pub hit_ranges: HitRanges,           // primary (non-pedal)
    pub hit_ranges_pedal: HitRanges,     // secondary
    pub gauge: GaugeConfig,
    pub hidden: bool, pub sudden: bool, pub dark: DarkMode,
    pub light: bool, pub reverse: bool,
    pub random_pad: RandomMode, pub random_pedal: RandomMode,
    pub tight: bool,
    pub hh_group: HhGroup, pub ft_group: FtGroup,
    pub cy_group: CyGroup, pub bd_group: BdGroup,
    pub show_lag: LagDisplay,
    pub show_play_speed: PlaySpeedDisplay,
}
```

### `SongLibrary` expansion

`SongEntry` already has `preview_audio / preview_image / background_video
/ box_path`. Add chart-attached fields from `CSongListNode`:

```rust
pub struct ChartEntry {
    pub path: PathBuf,
    pub label: String,
    pub level: Option<f32>,
    pub instrument: Instrument,         // Drums only for now
    pub note_count: usize,
    pub bpm: f32,
    pub difficulty_class: ERank,        // from DLEVEL
    pub hit_ranges: Option<HitRanges>,  // chart-level override
    pub hit_ranges_pedal: Option<HitRanges>,
    pub preview_audio: Option<PathBuf>,
    pub preview_image: Option<PathBuf>,
    pub background_video: Option<PathBuf>,
    pub stage_file: Option<PathBuf>,
    pub result_image: Option<PathBuf>,
    pub result_movie: Option<PathBuf>,
    pub result_sound: Option<PathBuf>,
    pub dtx_hash: String,               // MD5 hex
}
```

## Cross-cutting

These run alongside the phase work, not as their own phase.

### Score migration

`scores.ron` schema change is unavoidable. Plan:

- Add `version: u32` field (start at 2; current is v1).
- On load, if missing or older, migrate.
- v1 → v2: drop fields that won't fit, set defaults for new ones.
- Per-chart `.score.ini` reads are tolerant: missing sections default.

### `.score.ini` codec

Pure module under `persistence/score_ini.rs`. Single function pair:

```rust
pub fn load(path: &Path) -> Result<PerChartScore, ScoreIniError>;
pub fn save(path: &Path, score: &PerChartScore) -> Result<(), ScoreIniError>;
```

Encoder writes:

- `[File]` block (Title, Name, Hash, History[5], BestRank, PlayCountDrums)
- `[HiScore.Drums]` (full `PerformanceEntry`)
- `[HiSkill.Drums]` (full `PerformanceEntry`)
- `[LastPlay.Drums]` (full `PerformanceEntry`)

Shift-JIS for legacy strings. UTF-8 fallback if Shift-JIS encode fails.

### Parser test corpus

Add `tests/dtx_fixtures/` with at least one example of each directive we
claim to support. Goal: every directive has a `#[test]` in
`chart/dtx/parser.rs` or `chart/dtx/text.rs`.

### MD5 hashing

Add `md-5` crate (RustCrypto). Used for `.score.ini` `Hash` field and
chart identity in song select. Performance cost ~negligible (charts are
small).

### `versions` resource

Add a `Versions` resource tracking:

- dtxpt build version
- loaded DTX format version
- loaded score format version

Exposed in debug HUD.

## Phases

### Phase 0 — Critical parity fixes

**Goal:** Fix items where dtxpt currently produces visibly wrong
results vs BocuD. These are correctness bugs, not feature gaps, and
must land before any other phase.

**Sub-phases** (each its own commit):

1. **0a — Rank formula correct** ✅
   - Replace `gameplay/scoring.rs::compute_rank` with two
     implementations matching BocuD:
     - `tCalculateRankOld(total, perfect, great, good, poor, miss)` —
       thresholds `SS=1.0, S≥0.95, A≥0.9, B≥0.85, C≥0.8, D≥0.7`
     - `tCalculateRank(completion_rate)` — thresholds
       `SS≥95, S≥80, A≥73, B≥63, C≥53, D≥45`
   - Switch on `GameConfig.n_skill_mode` (0 = Old, 1 = New)
   - Add `n_skill_mode: u8` to `GameConfig` (default 1)
2. **0b — Pedal-specific timing offset** ✅
   - Add `n_pedal_lag_time: i32` to `GameConfig` (default 0, range
     `[-100, +100]` ms)
   - In `gameplay/input.rs`, when computing nearest chip for BD/LP/LBD
     channels (0x13, 0x1B, 0x1C), apply
     `timing_offset + n_pedal_lag_time` instead of `timing_offset`
   - Apply same offset in judgement-time lookup
3. **0c — Chip play-time compute mode** ✅
   - Add `n_chip_play_time_compute_mode: u8` to `GameConfig`
     (0 = Original, 1 = Accurate)
   - Original: existing auto-play scan window (whatever dtxpt has)
   - Accurate: tighter window; pin exact formula after reading
     `CStagePerfCommonScreen.cs` auto-play chip scan logic
4. **0d — OS timer toggle (deferred to P11 audio)** ⚙️ config only
   - Tracked here; implementation lives in audio playback. Add
     `b_use_os_timer: bool` to `GameConfig` (default false).
5. **0e — Score.ini write toggle** ⚙️ config only
   - Add `b_write_score_ini: bool` to `GameConfig` (default true)
   - When false, persist `scores.ron` only; skip per-chart write
6. **0f — Cymbal-free mode (also P7)** ✅
   - Add `b_cymbal_free: bool` to `GameConfig` (default false)
   - When true, any cymbal input triggers any cymbal chip (CY/RD/LC)
   - Hit-detection logic in `gameplay/input.rs`

**Files touched:**

- `src/gameplay/scoring.rs` (rank formula)
- `src/gameplay/input.rs` (pedal timing, cymbal-free)
- `src/config/model.rs` (new fields)

**Tests:**

- `tCalculateRankOld`: SS for `(P=10,G=0,Go=0,Po=0,M=0)`, S for `P=9`,
  A for `P=8`, B for `P=7`, C for `P=6`, D for `P=5`, E for `P=4`
- `tCalculateRank`: SS for `completion_rate=95.0`, S for `80`, E for `44`
- Pedal lag: with `pedal_lag_time=10`, a chip at t=10ms missed-when-
  unlagged becomes hit-when-lagged
- Cymbal free: CY input triggers RD chip with `b_cymbal_free=true`
  (and not otherwise)

**Done when:**

- Score produces same rank letter as BocuD for at least 10 hand-
  computed scenarios (write a `tests/scoring_parity.rs`)
- Pedal timing behaves independently from non-pedal timing
- Cymbal-free toggles correctly

---
Phases are sequenced by user-visible impact and dependencies. Each phase
is one logical commit when the user asks.

**Phase numbering notes after re-check:**

- **Phase 0** is new (added after re-check): critical parity fixes
  (rank formula, pedal lag, OS timer, chip compute mode, score.ini
  write toggle, cymbal-free) that gate later work.
- **Phase 4** expanded: skill rate formulas now pinned exactly per
  `nSkillMode`; input device flags now required for rank validity.
- **Phase 7** expanded: pedal lag, cymbal-free, per-chart hit ranges
  application at run start, score.ini input-device flags.
- **Phase 8** expanded: bAutoAddGage explicitly mentioned.
- **Phase 11** expanded: lyrics rendering added as sub-phase.
- **Phase 13** expanded: cymbal-free, stoic mode, wave-adjust,
  focused-sleep, per-frame-sleep, compact mode, sub-box random,
  lyics rendering, wailing bonus, progress bar render, in-play panels
  all called out by file reference.

---

### Phase 1 — Metadata + chip kinds (parser expansion)

**Goal:** Parse every DTX directive BocuD handles. No visual changes;
metadata only.

**Sub-phases** (each its own commit when user asks):

1. **1a — Metadata directives**
   - `ARTIST`, `COMMENT`, `GENRE`, `MAKER`, `PANEL`, `BANNER`,
     `DESCRIPTION`, `PLAYLEVEL`
2. **1b — Difficulty + level**
   - `DLEVEL`, `GLEVEL`, `BLEVEL`, `HIDDENLEVEL`, `FORCINGXG`,
     `DLVDEC`, `GLVDEC`, `BLVDEC`
3. **1c — Media directives** (parse, store in `ChartEntry`)
   - `PREIMAGE`, `PREVIEW`, `PREMOVIE`, `BACKGROUND`, `STAGEFILE`,
     `BGA`, `BMPxx`, `BGAPAN`, `AVI`, `AVIPAN`, `MOVIE`,
     `RESULTIMAGE` (+ `_SS`), `RESULTMOVIE` (+ `_SS`), `RESULTSOUND`
     (+ `_SS`)
4. **1d — WAV extras**
   - `WAVCOL`, `SIZE`, `VOL7FTO64`
5. **1e — Mod directives**
   - `HIDDEN`, `SUDDEN`, `RANDOM`, chart-level overrides only
6. **1f — Control flow + lyrics + MIDI**
   - `#IF` / `#ENDIF` (strip during parse), `LYRIC`, `MIDIFILE`,
     `MIDINOTE`, `DTXVPLAYSPEED`

**Files touched:**

- `src/chart/dtx/parser.rs` (main directive loop)
- `src/chart/dtx/text.rs` (text helpers, `#IF`/`#ENDIF`)
- `src/chart/model.rs` (`Chart`, `ChartNote` — add metadata fields)
- `src/song_library/scanner.rs` (extract `DLEVEL`, `PREIMAGE`, etc.)
- `src/song_library/model.rs` (`ChartEntry` expansion)

**Tests:**

- One `#[test]` per directive (existing pattern in `parser.rs`)
- Loader round-trip: parse → serialize → parse = equal

**Done when:**

- `rg '"#([A-Z]+)' src/chart/dtx/` lists every directive in the gap
  report under "Missing directives"
- `cargo test` passes
- `cargo check --no-default-features` (if we add any) passes

---

### Phase 2 — Chip kinds (chart → runnable events)

**Goal:** Every channel BocuD recognises produces a `ChartNote` (or
appropriate event). Visual rendering stays Bevy-native; this phase only
ensures the chip is in the model.

**Sub-phases:**

1. **2a — BGA / image / video chips**
   - Channels: 0x04, 0x07, 0x54, 0x55-0x58, 0x5A, 0x60-0x64, 0xA0
   - Store as `ChartBgaEvent { tick, layer, path, sizing, clip }`
2. **2b — Swap / non-visual chips**
   - BGA swap pairs (196, 199, 213-217, 224)
   - BarLine (80), BeatLine (81, 193, 194)
   - FillIn (83)
   - MIDIChorus (82), Click (236), FirstSoundChip (237),
     MixerAdd/Remove (238, 239)
3. **2c — Bonus / FX chips**
   - BonusEffect (76-79)
4. **2d — Wailing / long-note chips** (drum only, no guitar/bass)
   - 0x28-0x2C drum wailing

**Files touched:**

- `src/chart/dtx/channels.rs` (`is_dtx_se_channel` extensions, new
  helpers per chip kind)
- `src/chart/dtx/parser.rs` (dispatch per chip)
- `src/chart/model.rs` (new event types)
- `src/chart/dtx/bgm.rs` (existing, may split out per-layer)

**Tests:**

- One `#[test]` per chip kind: parse, verify event fields
- Channel coverage test: every channel in `EChannel.cs` 0x01..0xEF has a
  dispatch arm (or an explicit "unhandled, log warning" arm)

**Done when:**

- `EChannel.cs` line-by-line mapped to dtxpt handlers
- `cargo test` passes
- Parser logs a single, identifiable warning per unhandled channel

---

### Phase 3 — Per-chart `.score.ini` codec ⚙️ codec/write slice started

**Goal:** Read/write `.score.ini` files compatible with BocuD.

**Files touched:**

- New `src/persistence/score_ini.rs`
- `src/persistence/scores.rs` (extend `ScoreStore` with per-chart
  cache; primary write goes to `.score.ini`)
- `Cargo.toml` (add `encoding_rs` for Shift-JIS, `md-5` for hash,
  optional `chrono` for timestamps)
- `src/song_library/scanner.rs` (read `.score.ini` on chart discovery,
  populate `ChartEntry.dtx_hash` and metadata)

**Tests:**

- Read a hand-crafted `.score.ini` (Shift-JIS) → correct `PerChartScore`
- Write a `PerChartScore` → `.score.ini` → read back → equal
- Round-trip with Japanese strings (BocuD uses Shift-JIS)
- Corrupt file: returns `ScoreIniError::ParseFailure` with line context

**Done when:**

- A chart folder with BocuD-written `.score.ini` shows the score in
  song select after re-scan
- A chart played in dtxpt writes a `.score.ini` that BocuD reads
  (manual verification on one chart)

---

### Phase 4 — Skill rate + HiSkill + BestScore expansion ⚙️ skill formula slice started

**Goal:** Compute `dbGameSkill` and `dbPerformanceSkill` per run; persist
in `HiSkill.Drums` section. Pin exact BocuD formulas.

**Algorithm source:** `CScoreIni.cs` lines 1320-1485. Two formula sets,
switched by `n_skill_mode` (config from Phase 0a).

**Formulas (drums only):**

**nSkillMode == 0 (Old):**

```rust
fn calculate_game_skill_old(
    level: f64, level_dec: i32,
    total: i32, perfect: i32, great: i32, combo: i32,
    auto_play: &AutoPlayFlags,
) -> f64 {
    if total == 0 || (perfect == 0 && combo == 0 && great == 0) { return 0.0; }
    let rate = (perfect as f64 * 0.8 + great as f64 * 0.3 + combo as f64 * 0.2)
               / total as f64;
    let mut ret = level * rate * 0.33;
    ret *= revise_for_auto(auto_play);  // 0.5 if any auto, 1.0 otherwise
    if all_auto { 0.0 } else { ret }
}
```

**nSkillMode == 1 (New):**

```rust
fn calculate_playing_skill(
    total: i32, perfect: i32, great: i32, good: i32, poor: i32, miss: i32,
    combo: i32, auto_play: &AutoPlayFlags,
) -> f64 {
    if total == 0 { return 0.0; }
    let perfect_rate = 100.0 * perfect as f64 / total as f64;
    let great_rate   = 100.0 * great   as f64 / total as f64;
    let combo_rate   = 100.0 * combo   as f64 / total as f64;
    // (note: combo rate uses nTotal, not nTotal - auto; if all auto, combo_rate=0)
    let mut ret = perfect_rate * 0.85 + great_rate * 0.35 + combo_rate * 0.15;
    ret *= revise_for_auto(auto_play);
    ret
}

fn calculate_game_skill_from_playing_skill(
    level: f64, level_dec: i32, playing_skill: f64,
) -> f64 {
    let level = if level >= 100.0 { level / 100.0 } else { level / 10.0 + level_dec as f64 / 100.0 };
    if all_auto { 0.0 } else { playing_skill * level * 0.2 }
}
```

**Input sources:**

- `total` = chart's visible chip count (post-parse in `Chart`)
- `perfect` / `great` / `combo` = `RunState` counters excluding auto
- `level` = chart's `DLEVEL` directive (parsed in Phase 1b)
- `level_dec` = chart's `DLVDEC` directive (parsed in Phase 1b)

**Inputs that require Phase 1 to land first:** `level` and `level_dec`.
Until Phase 1b is done, default both to `0.0` (which still produces
correct relative skill values).

**Files touched:**

- `src/gameplay/skills.rs` (new — pure functions, no Bevy deps)
- `src/persistence/scores.rs` (`BestScore` expansion per Data-model section)
- `src/gameplay/run.rs` (extend `RunResult`)
- `src/screens/result.rs` (render skill values)
- `src/screens/song_select.rs` (show best skill in card)

**Tests:**

- `skills::calculate_game_skill_old` against known-input / known-output
  reference values (extract 3-5 from BocuD if needed)
- `skills::calculate_playing_skill` similarly
- All-auto edge case: returns 0.0
- Empty chart: returns 0.0
- Round-trip: write to `HiSkill.Drums`, read back, equal

**Done when:**

- Skill rate appears on Result
- Skill rate appears in song-select status panel
- Saved `.score.ini` shows `[HiSkill.Drums]` populated
- For a chart with known DLEVEL=85, DLVDEC=0, full combo, all
  perfect, both formulas produce value within 0.5% of BocuD output

---

### Phase 5 — Result screen upgrade

**Goal:** Match BocuD Result features on the data side; visual stays
Bevy-native.

**Sub-phases:**

1. **5a — New-record detection**
   - Per-chart comparison of `score`, then `accuracy`, then `max_combo`
   - Show "NEW RECORD" badge on Result when applicable
2. **5b — Last-play snapshot**
   - Save `[LastPlay.Drums]` regardless of new-record status
3. **5c — Result media**
   - Render `RESULTIMAGE` if present
   - Play `RESULTSOUND` if present
4. **5d — Skill display**
   - Show both skill rates alongside score

**Files touched:**

- `src/screens/result.rs`
- `src/gameplay/scoring.rs` (new-record logic)
- `src/persistence/scores.rs` (LastPlay plumbing)

**Tests:**

- New-record badge logic: score higher, equal score + higher acc, equal
  both + higher combo
- LastPlay always written (even when score is lower)

**Done when:**

- Result screen shows: rank, new-record badge, full skill panel, result
  image (if present), result sound (if present)

---

### Phase 6 — Stage Clear / Stage Fail screens

**Goal:** Dedicated screens between `Playing` and `Result` (or
replacing Result in fail case).

**Flow:**

```
Playing → [stage clear logic] → StageClear screen → Result
       → [stage fail logic]  → StageFail screen → Result (with FAILED rank)
```

Or per BocuD convention: clear/fail screens are short interstitial with
retry/next buttons; Result is always shown with appropriate rank.

**Sub-phases:**

1. **6a — Stage detection**
   - `run.cleared` and `run.failed` already exist
   - Add `ePerfScreenReturnValue` enum (`Continue / Interruption /
     Restart / StageFailure / StageClear`)
2. **6b — Stage Clear screen**
   - Animations: clear-flash, score tally
   - Retry / Next buttons
3. **6c — Stage Fail screen**
   - Failure SFX
   - Retry / Back buttons
4. **6d — Result transition**
   - Result always follows; FAILED rank already computed

**Files touched:**

- `src/app/state.rs` (add `AppState::StageClear` / `StageFail`)
- `src/screens/stage_clear.rs` (new)
- `src/screens/stage_fail.rs` (new)
- `src/screens/result.rs` (transition from both)
- `src/gameplay/run.rs` (return value)

**Tests:**

- Stage fail screen transitions to Result with `failed=true`
- Stage clear screen transitions to Result with `cleared=true`
- Esc from clear/fail returns to Song Select

**Done when:**

- Failing a chart shows StageFail screen with retry/back, then Result
  with `FAILED` rank
- Clearing shows StageClear screen with retry/next, then Result with
  rank + skill

---

### Phase 7 — Mods: Hidden / Sudden / Dark / Random / Reverse

**Goal:** All chart-side and run-side mods BocuD supports for drums.

**Sub-phases:**

1. **7a — Mod state**
   - `ModSet` expanded to match BocuD fields (see Data-model section)
   - Per-lane mod storage (`STDGBVALUE<T>` analog → per-lane enum map)
2. **7b — Hidden / Sudden / Dark rendering**
   - Hidden: dim playfield except judgment line + last 1s of notes
   - Sudden: notes invisible until last 1s
   - Dark (OFF/HALF/FULL): black overlay
   - Stealth = Hidden + Sudden
3. **7c — Light / Reverse**
   - Light: dim non-judgment area
   - Reverse: scroll direction reversed (notes scroll down)
4. **7d — Random Pad / Random Pedal**
   - Apply `RandomMode` channel reorder at run start
   - Modes: OFF / Mirror / Random / SuperRandom / HyperRandom /
     MasterRandom / AnotherRandom
   - **Note:** Random is also subject to chart directive override
     (`#HIDDEN`, `#SUDDEN`, `#RANDOM` per-chart; covered in P1e)
5. **7e — Tight + Risky + Specialist**
   - Tight: multiply hit windows by 0.7 (configurable)
   - Risky 1-10: gauge = N misses left; miss → fail
   - Specialist: ignored (guitar/bass only)
6. **7f — Per-chart hit ranges**
   - Apply chart-level `HitRanges` (from `DLEVEL`-relative settings or
     stored `ChartEntry.hit_ranges`) at run start
   - Apply chart-level pedal `HitRanges` (secondary)
   - Fall back to `GameplayConfig.hit_ranges` when chart has no override
7. **7g — Input-device flags in score (also P3)**
   - Track `keyboard_used`, `midi_used`, `joypad_used`, `mouse_used`
     during the run (set true on any input from that device)
   - Persist into `[HiScore.Drums]` and `[LastPlay.Drums]`
   - **Rank validity rule:** if all four are false → `ERANK.UNKNOWN`
     (BocuD `tCalculateRank(part)` returns UNKNOWN in this case)

**Files touched:**

- `src/gameplay/mods.rs` (`ModSet` expansion)
- `src/gameplay/run.rs` (apply mods, track input devices)
- `src/gameplay/rendering/playfield_viz.rs` (Dark overlay)
- `src/gameplay/rendering/notes.rs` (Hidden/Sudden visibility)
- `src/gameplay/scrolling.rs` (Reverse direction)
- `src/gameplay/random.rs` (new — random lane reorder)
- `src/gameplay/run_setup.rs` (apply chart hit ranges at start)
- `src/gameplay/input.rs` (set input-device flags)
- `src/persistence/scores.rs` (rank UNKNOWN rule)

**Tests:**

- Mirror: HH maps to SD, SD to HH, etc.
- Random: determinism given seed
- SuperRandom: 2-pair swaps from Random
- Tight hit windows: `34ms * 0.7 = 24ms` exact
- Risky 3: 3 misses left; gauge fill = `n_misses_remaining / 3`
- Per-chart hit ranges: chart overrides config; pedal ranges separate
- All-auto run: rank becomes UNKNOWN
- All-keyboard run: rank computed normally

**Done when:**

- Hidden/Sudden/Dark/Stealth/Light toggleable via settings + chart
  directive
- Random Pad modes reorder lanes; settings show current mode
- Risky mode fails at 0 misses; visible gauge countdown
- Per-chart hit ranges apply at run start, override config defaults
- Input-device flags tracked and rank UNKNOWN rule correct

---

### Phase 8 — Gauge modes + Damage Level

**Goal:** Multiple gauge difficulty modes + damage scaling.

**Reference:** `CActPerfCommonGauge.cs` constants:
`GAUGE_MAX=1.0`, `GAUGE_INITIAL=2/3`, `GAUGE_MIN=-0.1`, `GAUGE_DANGER=0.3`.

**Sub-phases:**

1. **8a — Gauge modes**
   - Normal / Hard / Death / Extreme / EXHard (each with own factor)
   - Source: `fGaugeFactor[5,2]` (currently `#if false` in BocuD — may
     need to mine actual values from older BocuD revisions)
2. **8b — Damage Level**
   - Small / Normal / High: scales gauge loss on miss
3. **8c — Initial gauge**
   - Configurable start (default 2/3 per BocuD; dtxpt currently 0.80)
4. **8d — Auto-add gauge** (`bAutoAddGage`)
   - When ON, auto-played chips contribute positive gauge delta
   - Default OFF
   - BocuD behaviour: `bAutoAddGage` only matters in auto-mode runs;
     when ON, hitting auto-play chips adds gauge; when OFF, only
     manual hits add
5. **8e — Stage-fail enabled** (`bSTAGEFAILEDEnabled`)
   - When OFF, gauge can never reach `IsFailed` state; run continues
     even at zero gauge
   - Default ON; practice mode already does this implicitly

**Files touched:**

- `src/gameplay/gauge.rs` (replace constants with config-driven)
- `src/config/model.rs` (`GaugeConfig`)

**Tests:**

- Damage Level Small: 0.5× loss
- Damage Level High: 2.0× loss
- Gauge Hard mode: harder thresholds per `fGaugeFactor`
- Auto-add gauge: when ON and a chip is auto-played, gauge increases
- bSTAGEFAILEDEnabled=false: gauge can hit 0 without `failed=true`

**Done when:**

- Gauge config persists
- Damage level affects miss delta
- Auto-add gauge toggles correctly
- Stage-fail toggle works (practice mode behaviour reachable in Normal)

---

### Phase 9 — Ghost data (Auto Ghost + Target Ghost)

**Goal:** Visual playback guide from prior runs.

**Sub-phases:**

1. **9a — Ghost data source**
   - Read from `[LastPlay.Drums]`, `[HiSkill.Drums]`, `[HiScore.Drums]`
2. **9b — Ghost capture during play**
   - Record timestamped chip-hit state into `RunState.ghost_data`
   - Save on `[LastPlay.Drums]` always; update `[HiScore.Drums]` on
     new record; update `[HiSkill.Drums]` on skill new record
3. **9c — Ghost rendering**
   - Render notes from prior run at offset (semi-transparent overlay)
   - Modes: PERFECT (always), LAST_PLAY, HI_SKILL, HI_SCORE
4. **9d — Target Ghost**
   - Show goal target lane markers from selected ghost source
   - NONE / PERFECT / LAST_PLAY / HI_SKILL / HI_SCORE

**Files touched:**

- `src/gameplay/ghost.rs` (new — capture + playback)
- `src/gameplay/rendering/playfield_viz.rs` (overlay ghost notes)
- `src/overlays/settings/rows.rs` (AUTO Ghost + Target Ghost settings)

**Tests:**

- Ghost capture determinism: given fixed input, ghost bytes equal
- Ghost playback: stored ghost renders at same chart positions

**Done when:**

- AUTO Ghost shows last-play's note sequence during gameplay
- Target Ghost shows hi-score notes as goal

---

### Phase 10 — Song select UX

**Goal:** All BocuD song-select features reachable from dtxpt keyboard.

**Sub-phases:**

1. **10a — Quick Config popup** (highest value)
   - Keybind: `Tab` opens
   - Auto Mode (All/Auto LP/Auto BD/2Pedal/XG/Custom/Off for drums;
     we have Off/PerLane/AllAuto already — extend)
   - Dark (OFF/HALF/FULL)
   - AUTO Ghost (OFF/PERFECT/LAST_PLAY/HI_SKILL/HI_SCORE)
   - Target Ghost (NONE/PERFECT/LAST_PLAY/HI_SKILL/HI_SCORE)
   - Esc closes
2. **10b — Sort menu**
   - Keybind: `F4` opens
   - Title (asc/desc), Level (asc/desc), BestRank (asc/desc), BPM,
     Artist, New
3. **10c — Status panel / Perf history**
   - Show: best rank, best skill, play count, last 5 dates
   - Right side panel (BocuD places it there)
4. **10d — Density graph**
   - Notes-per-second chart across chart time
   - Shown in song select when a chart is selected
5. **10e — Artist comment**
   - Read `COMMENT` from DTX or `set.def`
   - Display when chart selected

**Files touched:**

- `src/screens/song_select.rs` (extend state, input)
- `src/screens/song_select_quick_config.rs` (new)
- `src/screens/song_select_sort.rs` (new)
- `src/screens/song_select_status.rs` (new)
- `src/screens/song_select_density.rs` (new)
- `src/song_library/model.rs` (`SongLibrary.sort_mode` field)

**Tests:**

- Sort: songs sort correctly by each mode
- Quick Config: each option writes to `GameplayConfig`
- Status panel: reads from `.score.ini` per-chart

**Done when:**

- All 5 sub-features reachable from keyboard without leaving song select

---

### Phase 11 — Media playback

**Goal:** Preview audio, BGA image layers, result media, SFX library,
lyrics.

**Sub-phases:**

1. **11a — Preview audio** (chart `PREVIEW`)
   - Replace generic menu music with chart preview when song selected
   - Fade in/out on song-select change
2. **11b — BGA static images**
   - Channels 0x04, 0x07, 0x55-0x58, 0x60-0x64
   - Spawn sprite at tick; size/clip per channel args
   - Layers 1-8 with swap pairs (196, 199, 213-217, 224)
   - BMPTEX variant: alpha-channel support (vs BMP which uses colorkey)
3. **11c — Background image**
   - `BACKGROUND` / `STAGEFILE` as static playfield background
4. **11d — Result media**
   - Render `RESULTIMAGE` on Result; use rank-variant `_SS`/`_S`/`_A`/
     `_B`/`_C`/`_D`/`_E` if present (per BocuD `_A`-`_E` dispatch)
   - Play `RESULTSOUND` on Result
5. **11e — SFX library**
   - `sounds.ron` resource mapping event name → file path
   - Events: `stage_failed`, `stage_clear`, `full_combo`, `audience`,
     `drum_hit`, `now_loading`
   - Config toggle per SFX (matches BocuD `b歓声を発声する`,
     `SOUND_FULLCOMBO`, `SOUND_STAGEFAILED`, `SOUND_AUDIENCE`,
     `SOUND_NOWLOADING` directives)
6. **11f — Lyrics rendering**
   - Parse `#LYRIC xx:` directive → `(tick, text)` pairs (P1f covers
     parse)
   - Render lyrics on screen, sync'd to chart time
   - Font/color from chart metadata or skin default
7. **11g — `bWave再生位置自動調整機能有効`** (WAV position auto-adjust)
   - BocuD behaviour: auto-correct WAV playback position drift
   - Used with DirectSound backend. kira's behaviour may already
     handle equivalent. If not, add an audio-thread drift corrector
   - Toggle in `GameConfig.audio` section
8. **11h — OS timer toggle** (`bUseOSTimer`)
   - Add `b_use_os_timer: bool` to `GameConfig` (default false)
   - When ON, schedule chip audio playback against OS high-resolution
     timer instead of kira's default clock
   - May not be needed on all platforms; verify against kira docs

**Files touched:**

- `src/audio/preview.rs` (new)
- `src/audio/sfx.rs` (new)
- `src/audio/timing.rs` (new — wave adjust, OS timer)
- `src/gameplay/bga.rs` (new)
- `src/gameplay/rendering/bga_viz.rs` (new)
- `src/gameplay/lyrics.rs` (new)
- `src/screens/result.rs`
- `src/config/model.rs` (SFX + audio toggles)

**Tests:**

- BGA parse: chip with sizing args stored correctly
- BGA render: sprite spawned at correct tick, sized correctly
- SFX resource: missing file doesn't crash, logs warning
- Lyrics: `(tick, text)` pairs preserved through parse; rendering
  advances text per tick
- Result rank variant: `RESULTIMAGE_SS` used when rank == SS

**Done when:**

- Preview audio plays per chart
- BGA images appear at correct chart timing
- SFX fire on stage events
- Lyrics render in sync
- Result rank variant media used correctly
- OS timer toggle applies to audio scheduling

**Skipped from this phase:** Video (AVI / MP4 / M4V). Tracked as a
deferred decision per `full-game-roadmap.md` Phase 5.

---

### Phase 12 — Config surface expansion

**Goal:** Reach feature parity with `CConfigIni.cs` drum-relevant fields.

**Current categories (6):** General / Audio / Gameplay / Input /
Graphics / Debug.

**New categories to add (or expand existing):**

| Category | New rows |
|----------|----------|
| **Drums** (new) | HH Group, FT Group, CY Group, BD Group, HH Priority, FT Priority, CY Priority, LaneDisp, AttackEffect, JudgePosition, RDPosition, LaneType, NumOfLanes, Input Mapping, **Cymbal Free**, **Pedal Lag Time**, **Tight** |
| **Audio** (expand) | Sound preview wait, image preview wait, auto chip volume, manual chip volume, audience SFX toggle, stage failed SFX toggle, full combo SFX toggle, **Use OS Timer**, **Wave Auto-Adjust** |
| **Graphics** (expand) | BGA enabled, AVI enabled, BG alpha, movie alpha, fullscreen exclusive, window X/Y/W/H persist |
| **Gameplay** (expand) | All Phase 7-8 fields, **Stoic Mode**, **FillIn Enabled**, **Auto-Add Gauge**, **Score Ini Write**, **Skill Mode** (0=Old, 1=New) |
| **Debug** (expand) | Log DTX detail, log song search, log create/release, output log mode, **unfocused sleep ms**, **per-frame sleep ms** |
| **System** (new) | Sound driver type (ACM/ASIO/WASAPI), WASAPI buffer size, MIDI throttle, Discord toggle, OS timer, chip play time compute mode, **Random Recurse Sub-Box** |

**Files touched:**

- `src/overlays/settings/rows.rs` (add rows)
- `src/overlays/settings/ui.rs` (add categories)
- `src/config/model.rs` (add fields)

**Done when:**

- Every drum-relevant field in `CConfigIni.cs` has a row in the
  settings overlay
- `rg 'bDrums|CYGroup|HHGroup' references/DTXmaniaNX-BocuD/DTXMania/Core/CConfigIni.cs`
  lists no field that dtxpt cannot set

---

### Phase 13 — Polish + remaining parity

**Goal:** Everything else BocuD does that doesn't fit a numbered phase.

1. **13a — Discord Rich Presence**
   - `CDTXRichPresence.cs` analog
   - Optional, behind toggle
2. **13b — Lag display modes**
   - `EShowLagType` (OFF/ON/GREAT_POOR)
   - HUD shows timing offset per hit
3. **13c — Play speed display**
   - `EShowPlaySpeed` (OFF/ON/IF_CHANGED_IN_GAME)
   - HUD shows current rate
4. **13d — Hi-hat open graphics / LBD graphics**
   - `HHOGraphics` / `LBDGraphics` toggles — change lane appearance
5. **13e — FillIn toggle**
   - `bFillInEnabled` — visual treatment of fill-in chips
6. **13f — Combo / judgement text display positions**
   - Per-lane or global position options (`E判定文字表示位置`,
     `EDrumComboTextDisplayPosition`)
7. **13g — Notes-per-lane display** (`chipCountByLane`)
   - Show chip counts in song select
8. **13h — Wailing bonus + effect**
   - Track wailing bonus per drum hit (per
     `CActPerfCommonWailingBonus.cs`)
   - Render wailing bonus counter + visual effect
     (`WailingEffect.cs`)
   - Drum wailing chip channels 0x28-0x2C (parsed in P2d)
9. **13i — Skill progress bar render**
   - `CActPerfProgressBar.cs` visualises `strProgress`
   - Render in Result and song-select status
10. **13j — In-play visual panels**
    - Skill meter during play (`CActPerfSkillMeter.cs`)
    - Scroll speed display (`CActPerfScrollSpeed.cs`)
    - Status panel (`CActPerfCommonStatusPanel.cs`)
    - Performance Information HUD (`CActPerformanceInformation.cs`)
    - Combo display with style (`CActPerfCommonCombo.cs`)
    - Note explosion (`CActPerfPerfChipFireD.cs`,
      `PerfNewChipFire.cs`, `NoteExplosion.cs`)
    - Judgment string animation (`JudgementString.cs`)
    - Danger state visual (`CActPerfDrumsDanger.cs`)
    - Fill-in effect (`CActPerfDrumsFillingEffect.cs`)
    - RGB visual (`CActPerfCommonRGB.cs`)
    - Lane flush (`CActPerfCommonLaneFlushGB.cs`)
    - All visual; correctness unaffected
11. **13k — Song-select info panel**
    - `CActSelectInformation.cs` — info panel for selected chart
12. **13l — Boot screen** (`CStageStartup.cs`)
    - Splash + version + sound device init
13. **13m — Exit screen** (`CStageEnd.cs`)
    - Confirmation before quit
14. **13n — Compact mode** (`bCompactMode`)
    - Startup skips Title; jumps to Song Loading
    - Used for kiosk/launcher; niche
15. **13o — Song-transliteration** (romaji → kana fallback)
    - README mentions this as a BocuD feature; defer unless a real
      chart suite needs it

**Files touched:** various. One commit per sub-phase when user asks.

---

## Phase ordering + dependencies

```
0 (critical parity fixes)
  ↓
1 (parser)
  ↓
2 (chip kinds)
  ↓
3 (.score.ini codec) ← 4 (skill rate) ← 5 (result)
                                              ↓
                                         6 (clear/fail screens)
                                         ↓
                                         7 (mods) ← 8 (gauge modes)
                                                     ↓
                                         9 (ghost data)
                                                     ↓
                                         10 (song select UX)
                                                     ↓
                                         11 (media)
                                                     ↓
                                         12 (config surface)
                                                     ↓
                                         13 (polish)
```

Strictly: 0 must land before 1 (rank formula needs `n_skill_mode`
config); 1b (DLEVEL/DLVDEC) must land before 4 (skill formulas need
level inputs); 3 must land before 4 and 5; 5 before 6; 7-8 independent
of 5-6 but feed into 9; 9-13 chain on output readiness.

**Recommended execution order** (impact-driven):
**0, 3, 4, 5, 6, 7, 10, 8, 9, 11, 12, 13, then 1, 2 last.**

Rationale: 0 is fast (config fields + formula port + tests); 1+2 are
enablers but don't show user-visible value alone — better to land
them after we've proven the result/score pipeline works end-to-end.

## Verification matrix

Each phase has its own "done when". End-of-roadmap verification:

- [ ] Drop a BocuD-compatible chart folder in `chart_root` → appears in
      song select with metadata (artist, genre, BPM, level)
- [ ] Chart shows BGA image layers at correct times
- [ ] Chart plays with all drums-channel chips responsive
- [ ] Score saves to `.score.ini`; BocuD reads it without error
- [ ] New record badge fires correctly
- [ ] Skill rate matches BocuD for at least 5 test charts (±0.5%)
- [ ] **Rank letter matches BocuD for the same play** (10 hand-computed
      scenarios, both `nSkillMode` settings)
- [ ] **Pedal chips use `timing_offset + pedal_lag_time`** independently
      from non-pedal chips
- [ ] **Cymbal-free mode routes any cymbal input to any cymbal chip**
- [ ] **Cymbal-free OFF preserves per-cymbal lanes** (default)
- [ ] **All-auto run produces rank UNKNOWN** (input-device flags all
      false)
- [ ] Quick Config popup reachable in song select with all 4 options
- [ ] Sort menu reachable with all modes
- [ ] Status panel shows last 5 plays
- [ ] Hidden/Sudden/Dark modes work in-game
- [ ] Random Pad modes reorder lanes correctly
- [ ] Random select recurses into sub-boxes (when toggle ON)
- [ ] Risky mode fails at 0 misses
- [ ] AUTO Ghost shows last-play
- [ ] Result image + sound play (with rank variant `_SS`/`_S`/...)
- [ ] Preview audio plays per chart
- [ ] Lyrics render in sync with chart
- [ ] All 5 judgement ranks (SS/S/A/B/C/D/E/F) achievable
- [ ] Every C# `enum` in `CConstants.cs` has a Rust counterpart
- [ ] Every `[A-Z]+` directive in `CDTX.cs` is parsed (or explicitly
      skipped with a logged warning)
- [ ] Every config field in `CConfigIni.cs` reachable in settings
      overlay

## What does NOT need to match BocuD exactly

These are deliberate divergences — defer to redesign pass:

- **Rendering pipeline** — Bevy 0.18's wgpu + bevy_ui is the target.
  Skia/SlimDX comparisons are not relevant.
- **Update mechanism** — Bevy asset loading replaces BocuD's runtime
  unpack-and-install.
- **Discord** — same RPC lib (discord-rich-presence crate) but
  feature-gated.
- **ImGui dev tools** — keep debug HUD; future inspector is a separate
  project.
- **Skin system** — Phase 6 deferred. Charts with skin-switching via
  `box.def` will not change skin until Phase 6.

## After compat pass

The user has stated a redesign pass follows. Likely scope:

- Bevy-native settings UI (drop overlay model if it doesn't scale)
- Theme system replacing hardcoded colors
- Better input ergonomics (chord detection for simultaneous presses)
- Testable gameplay core (extract chart → run → result from Bevy)
- Continuous-update Bevy version
- Multi-language support (use `bevy_fluent` or similar)
- Accessibility (colorblind-safe palettes, audio cues)

These are NOT scoped here. They follow once compat is green.

## References

- Gap inventory: [`../dev-notes/2026-06-21-dtxmania-feature-gap.md`](../dev-notes/2026-06-21-dtxmania-feature-gap.md)
- Existing game roadmap: [`full-game-roadmap.md`](full-game-roadmap.md)
- Architecture: [`../reference/architecture.md`](../reference/architecture.md)
- Score persistence: [`../reference/persistence.md`](../reference/persistence.md)
- BocuD reference fork: `references/DTXmaniaNX-BocuD/`

## Changelog

- 2026-06-21 — Initial plan derived from gap report
- 2026-06-21 (re-check) — Added Phase 0 (critical parity: rank formula,
  pedal lag, OS timer, chip compute mode, score.ini write toggle,
  cymbal-free). Expanded Phase 4 (exact skill formulas pinned, DLEVEL
  dependency). Expanded Phase 7 (per-chart hit ranges, input-device
  flags, rank UNKNOWN rule). Expanded Phase 8 (auto-add gauge, stage-
  fail toggle). Expanded Phase 11 (BMPTEX, lyrics, wave-adjust, OS
  timer). Expanded Phase 12 (Stoic mode, FillIn, Skill Mode, Pedal
  Lag, sub-box random). Expanded Phase 13 (wailing bonus, skill
  progress bar, in-play panels, info panel, boot/exit screens, compact
  mode). Added 7 verification matrix items.
