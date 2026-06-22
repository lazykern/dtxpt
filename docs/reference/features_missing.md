# Compatibility deferrals / partial parity

Current after the 2026-06-22 A–F milestone. Items here are known gaps or explicit deferrals, not accidental omissions.

| Area | Status | Rationale / next owner |
|------|--------|------------------------|
| AVI / MOVIE / MovieFull / PREMOVIE / result movies | Partial | Parser now recognizes `#AVIxx`, `#AVIPANxx`, `#PREMOVIE`, and channels 0x54/0x5A into `Chart.avi_files`/`video_events` (smoke-tested on a real Tsukinami chart with `bg.avi`). Result-movie side lands in H4. Playback renderer requires a video crate whose FFI matches the system ffmpeg — currently blocked (see `docs/dev-notes/2026-06-22-video-decoder-choice.md`). |
| BGAPAN crop/pan + exact BGA swap-pair semantics | [x] | Static BMPxx/BACKGROUND/STAGEFILE image layers render. `#BGAPANxx` parsed into a registry keyed by BGAPAN number and attached to the `BGALayerN` chip whose integer value matches (`CDTX.cs:1384`). Swap channels (`BGALayer1_Swap`..`BGALayer8_Swap`, `EChannel.cs:154,157,170,..,181`) now map to the correct layers and are no longer confused with BeatLine/Bass-Y channels. Renderer linearly interpolates source rect, display rect, and size from start to end over `transition_seconds`. |
| Result media (RESULTIMAGE / RESULTSOUND variants) | Deferred | Result data parity landed first; media assets are a later result-screen polish pass. |
| Lyrics / #IF conditionals / specialty DTX directives | Deferred | Core chart portability works for supported gameplay slices; rare directives remain parser expansion work. |
| Dedicated StageClear / StageFail screens and SFX | Partial | Gauge fail/result state exists; separate interstitial screens and SFX are visual/audio polish. |
| Stoic mode full animation suppression | Partial | Config + Settings row exist. Full suppression across song-select/performance animations remains visual parity work. |
| OS sleep / unfocused frame sleep knobs | Deferred | dtxpt currently relies on Bevy/winit + bevy_framepace; BocuD Windows sleep knobs are not wired. |
| bUseOSTimer | Deferred | BocuD Windows timer mode does not map directly; dtxpt uses the audio clock as timing master. |
| Guitar/Bass detailed BestRank panels | Partial | Rank imports and song-select badge exist for current best score; per-instrument panel polish remains. |
| Custom user skin authoring UI / Stage 09 ChangeSkin | Deferred | Default procedural skin ships. User skin authoring and skin-change stage are out of current goal scope. |
| Discord Rich Presence / ImGui tools / updater | Deferred | Explicitly out of scope or debug-only for this port milestone. |
