use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;

const STEP: f64 = 1.0 / 240.0;
const SIZE: f64 = 28.0;
#[cfg(not(target_arch = "wasm32"))]
mod reference_solver;
#[cfg(not(target_arch = "wasm32"))]
pub mod authoring;

#[derive(Clone, Deserialize, Serialize)]
pub struct MapInput {
    pub id: String,
    pub title: String,
    pub duration: f64,
    pub beats: Vec<f64>,
    pub difficulty: u8,
    #[serde(default)]
    pub bpm: Option<f64>,
    #[serde(default)]
    pub level_segments: Vec<LevelSegment>,
    #[serde(default)]
    pub reference_geometry: Option<ReferenceGeometry>,
    #[serde(default)]
    pub reference_actions: Vec<Action>,
    #[serde(default)]
    pub reference_events: ReferenceEvents,
    #[serde(default)]
    pub grid_snap: bool,
    #[serde(default)]
    pub grid_actions: Vec<Action>,
    #[serde(default)]
    pub grid_cues: Vec<Action>,
    #[serde(default)]
    pub dash_carry: bool,
    #[serde(default)]
    pub reference_outro_seconds: f64,
    #[serde(default)]
    pub gravity_wells: Vec<GravityWell>,
}

#[derive(Clone, Deserialize, Serialize)]
pub struct GravityWell {
    pub cue: f64,
    // Field geometry uses the same world units and bottom-origin as Object.
    pub x: f64,
    pub y: f64,
    pub w: f64,
    pub h: f64,
    // Fraction of the normal launch velocity, not an arbitrary acceleration.
    pub impulse: f64,
    #[serde(default)]
    pub target_x: Option<f64>,
    #[serde(default)]
    pub target_y: Option<f64>,
    #[serde(default)]
    pub strength: f64,
    #[serde(default)]
    pub duration: f64,
    #[serde(default)]
    pub start_y: Option<f64>,
    #[serde(default)]
    pub start_vy: Option<f64>,
    #[serde(default)]
    pub arrival_time: Option<f64>,
    // Authored slingshots require a real near-apex dash on this musical cue.
    // The force starts after the dash's normal half-beat hang.
    #[serde(default)]
    pub dash_cue: Option<f64>,
    #[serde(default)]
    pub capture: Option<SlingshotCapture>,
}

#[derive(Clone, Deserialize, Serialize)]
pub struct SlingshotCapture {
    pub x: f64,
    pub y: f64,
    pub radius: f64,
    pub redirect_vy: f64,
    pub landing_time: f64,
}

#[derive(Clone, Default, Deserialize, Serialize)]
pub struct ReferenceEvents {
    #[serde(default)]
    pub gravity_tiles: Vec<f64>,
    #[serde(default)]
    pub rising_tiles: Vec<[f64; 2]>,
    #[serde(default)]
    pub falling_tiles: Vec<[f64; 2]>,
}

#[derive(Clone, Deserialize, Serialize)]
pub struct ReferenceGeometry {
    pub objects: Vec<ReferenceObject>,
    pub pits: Vec<ReferencePit>,
    pub end_tile: f64,
}
#[derive(Clone, Deserialize, Serialize)]
pub struct ReferenceObject {
    pub kind: String,
    pub x: f64,
    pub y: f64,
}
#[derive(Clone, Deserialize, Serialize)]
pub struct ReferencePit {
    pub start: f64,
    pub end: f64,
}

#[derive(Clone, Deserialize, Serialize)]
pub struct LevelSegment {
    pub beat: f64,
    #[serde(rename = "type")]
    pub kind: String,
    #[serde(default)]
    pub count: u8,
    #[serde(default)]
    pub height: u8,
    #[serde(default)]
    pub uw: String,
    #[serde(default)]
    pub ceiling_beat: Option<f64>,
    #[serde(default)]
    pub dash_beat: Option<f64>,
    #[serde(default)]
    pub chain_dash_beat: Option<f64>,
    #[serde(default)]
    pub exit_beat: Option<f64>,
    #[serde(default)]
    pub heights: Vec<u8>,
    #[serde(default)]
    pub rests: Vec<f64>,
    #[serde(default)]
    pub gate_releases: Vec<Option<f64>>,
    #[serde(default)]
    pub entry_height: u8,
    #[serde(default)]
    pub continuation: bool,
    #[serde(default)]
    pub widths: Vec<u8>,
    #[serde(default)]
    pub rest_types: Vec<String>,
    #[serde(default)]
    pub dash_variant: String,
    #[serde(default)]
    pub spike_counts: Vec<u8>,
    #[serde(default)]
    pub spike_offsets: Vec<i32>,
    #[serde(default)]
    pub release_roofs: Vec<u8>,
    #[serde(default)]
    pub roof_offsets: Vec<u8>,
    #[serde(default)]
    pub cue_beat: Option<f64>,
    #[serde(default)]
    pub well_strength: f64,
}

#[derive(Clone, Serialize, Debug)]
pub struct Object {
    pub kind: String,
    pub x: f64,
    pub y: f64,
    pub w: f64,
    pub h: f64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cue: Option<f64>,
}

#[derive(Clone, Deserialize, Serialize, Debug)]
pub struct Action {
    pub time: f64,
    pub action: String,
}

#[cfg(not(target_arch = "wasm32"))]
#[derive(Clone, Serialize, Debug)]
pub struct ContactAuditEvent {
    pub time: f64,
    pub x: f64,
    pub support_y: f64,
    pub cause: &'static str,
}

#[derive(Serialize)]
pub struct Snapshot<'a> {
    pub time: f64,
    pub x: f64,
    pub y: f64,
    pub vy: f64,
    pub grounded: bool,
    pub held: bool,
    pub jump_count: u32,
    pub dash_active: bool,
    pub slingshot_active: bool,
    pub dash_ready: bool,
    pub input_feedback: &'a str,
    pub inverted: bool,
    pub dash_offset: f64,
    pub dash_phase: f64,
    pub status: &'a str,
    pub progress: f64,
    pub death_reason: &'a str,
    pub objects: &'a [Object],
    pub actions: &'a [Action],
    pub duration: f64,
    pub speed: f64,
    pub jump_phase: f64,
    pub tile_size: f64,
    pub beat_period: f64,
    pub route_x: f64,
    pub finish_x: f64,
    pub support_y: f64,
}

/// Per-frame state; immutable route objects and actions stay cached by the host.
#[derive(Serialize)]
struct MotionSnapshot<'a> {
    time: f64,
    x: f64,
    y: f64,
    vy: f64,
    grounded: bool,
    held: bool,
    jump_count: u32,
    dash_active: bool,
    slingshot_active: bool,
    dash_ready: bool,
    input_feedback: &'a str,
    inverted: bool,
    dash_offset: f64,
    dash_phase: f64,
    status: &'a str,
    progress: f64,
    death_reason: &'a str,
    duration: f64,
    speed: f64,
    jump_phase: f64,
    tile_size: f64,
    beat_period: f64,
    route_x: f64,
    finish_x: f64,
    support_y: f64,
}

/// Audio seconds anchor the route. A dash temporarily advances the player,
/// then returns to the route clock without moving backwards.
#[derive(Clone)]
pub struct Runner {
    pub map: MapInput,
    pub objects: Vec<Object>,
    pub actions: Vec<Action>,
    time: f64,
    speed: f64,
    jump: f64,
    gravity: f64,
    flight: f64,
    beat_period: f64,
    dash_motion: f64,

    collision_indices: Vec<usize>,
    collision_max_width: f64,
    jump_elapsed: f64,
    y: f64,
    support_y: f64,
    vy: f64,
    grounded: bool,
    held: bool,
    jump_count: u32,
    dash_until: f64,
    dash_started: f64,
    dash_carry_total: f64,
    captured_well: Option<usize>,
    route_finish: f64,

    air_dash_used: bool,
    status: &'static str,
    death_reason: &'static str,
    input_feedback: &'static str,
    feedback_until: f64,
    #[cfg(not(target_arch = "wasm32"))]
    contact_audit: Option<Vec<ContactAuditEvent>>,
}

impl Runner {
    pub fn new(json: &str) -> Result<Self, String> {
        let mut map: MapInput = serde_json::from_str(json).map_err(|e| e.to_string())?;
        if !map.reference_outro_seconds.is_finite()
            || !(0.0..=10.0).contains(&map.reference_outro_seconds)
        {
            return Err("Invalid reference outro".into());
        }
        if !map.duration.is_finite()
            || !(3.0..=600.0).contains(&map.duration)
            || !(1..=4).contains(&map.difficulty)
            || map.beats.is_empty()
            || map
                .beats
                .iter()
                .any(|b| !b.is_finite() || *b < 0.0 || *b >= map.duration)
            || map.beats.windows(2).any(|b| b[1] <= b[0])
        {
            return Err("Invalid map: duration 3..600, difficulty 1..4, and strictly increasing finite beats required".into());
        }
        if map.gravity_wells.len() > 8
            || map.reference_geometry.is_some() && !map.gravity_wells.is_empty()
            || map
                .gravity_wells
                .windows(2)
                .any(|pair| pair[1].cue <= pair[0].cue)
            || map.gravity_wells.iter().any(|well| {
                !well.cue.is_finite()
                    || well.cue <= 0.0
                    || well.cue >= map.duration
                    || !well.impulse.is_finite()
                    || !(0.0..=0.5).contains(&well.impulse)
                    || [well.x, well.y, well.w, well.h].iter().any(|value| {
                        !value.is_finite()
                            || *value < 0.0
                            || (*value / SIZE - (*value / SIZE).round()).abs() > 1e-7
                    })
                    || well.w < SIZE
                    || well.h < SIZE
                    || well.y + well.h > 128.0 * SIZE
                    || !well.strength.is_finite()
                    || !(0.0..=3.0).contains(&well.strength)
                    || !well.duration.is_finite()
                    || !(0.0..=if well.dash_cue.is_some() { 2.5 } else { 1.5 })
                        .contains(&well.duration)
                    || well.target_x.is_some() != well.target_y.is_some()
                    || well.target_x.is_some()
                        && (well.strength <= 0.0
                            || well.duration < 0.15
                            || well.cue + well.duration > map.duration)
                    || well
                        .target_x
                        .iter()
                        .chain(well.target_y.iter())
                        .any(|v| !v.is_finite() || *v < 0.0)
                    || [well.start_y, well.start_vy, well.arrival_time]
                        .iter()
                        .flatten()
                        .any(|v| !v.is_finite())
                    || well
                        .dash_cue
                        .is_some_and(|dash| !dash.is_finite() || dash <= 0.0 || dash >= well.cue)
                    || well.capture.as_ref().is_some_and(|c| {
                        [c.x, c.y, c.radius, c.redirect_vy, c.landing_time]
                            .iter()
                            .any(|v| !v.is_finite())
                            || c.x < 0.0
                            || c.y < 0.0
                            || !(SIZE * 0.25..=SIZE).contains(&c.radius)
                            || c.landing_time <= well.cue
                            || c.landing_time >= map.duration
                            || well.dash_cue.is_none()
                    })
                    || (well.start_y.is_some()
                        || well.start_vy.is_some()
                        || well.arrival_time.is_some())
                        && (well.start_y.is_none()
                            || well.start_vy.is_none()
                            || well.arrival_time.is_none()
                            || well
                                .arrival_time
                                .is_some_and(|end| (end - well.cue - well.duration).abs() > 1e-7))
            })
        {
            return Err("Invalid gravity well: ordered in-track cues, whole-grid world geometry and impulse 0..0.5 required".into());
        }
        let mut intervals: Vec<_> = map.beats.windows(2).map(|p| p[1] - p[0]).collect();
        intervals.sort_by(f64::total_cmp);
        let beat_period = map
            .bpm
            .filter(|b| b.is_finite() && (60.0..=240.0).contains(b))
            .map(|b| 60.0 / b)
            .unwrap_or_else(|| intervals.get(intervals.len() / 2).copied().unwrap_or(0.5));
        let beat_period = beat_period.clamp(0.25, 1.0);
        // Original proportions: about two tiles high and 4.8 tiles across.
        // One beat per flight keeps held chains on the chart's musical grid.
        let flight = beat_period;
        // Source reference 90.9 px / 45 px; flight and run speed stay fixed.
        let apex = 2.02 * SIZE;
        let jump = 4.0 * apex / flight;
        let gravity = 8.0 * apex / (flight * flight);
        let speed = 4.8 * SIZE / flight;
        let mut score = map.level_segments.clone();
        if score.is_empty() && map.reference_geometry.is_none() {
            let segment = |beat: usize, kind: &str, count: u8, height: u8| LevelSegment {
                beat: beat as f64,
                kind: kind.into(),
                count,
                height,
                uw: String::new(),
                ceiling_beat: None,
                dash_beat: None,
                chain_dash_beat: None,
                exit_beat: None,
                heights: Vec::new(),
                rests: Vec::new(),
                gate_releases: Vec::new(),
                entry_height: 0,
                continuation: false,
                widths: Vec::new(),
                rest_types: Vec::new(),
                dash_variant: String::new(),
                spike_counts: Vec::new(),
                spike_offsets: Vec::new(),
                release_roofs: Vec::new(),
                roof_offsets: Vec::new(),
                cue_beat: None,
                well_strength: 0.0,
            };
            for start in (4..map.beats.len().saturating_sub(6)).step_by(32) {
                for local in [0, 2, 4, 6] {
                    score.push(segment(
                        start + local,
                        "jump_spike",
                        if map.difficulty == 1 { 2 } else { 3 },
                        0,
                    ));
                }
                score.push(segment(start + 8, "jump_pit", 0, 0));
                score.push(segment(start + 12, "jump_spike", 3, 0));
                if start + 32 < map.beats.len().saturating_sub(5) {
                    for (local, height) in [(16, 1), (18, 2), (20, 3), (23, 2), (26, 1), (29, 0)] {
                        score.push(segment(start + local, "jump_step", 0, height));
                    }
                }
            }
            score.retain(|segment| segment.beat < map.beats.len().saturating_sub(5) as f64);
        }
        score.sort_by(|a, b| a.beat.total_cmp(&b.beat));
        if score.iter().any(|s| {
            !s.beat.is_finite()
                || s.beat < 0.0
                || s.beat >= map.beats.len() as f64 - 1.0
                || s.height
                    > if matches!(s.kind.as_str(), "dash_route" | "gravity_route") {
                        24
                    } else {
                        6
                    }
                || s.count > 6
                || s.heights.len() > 48
                || s.heights.iter().any(|h| *h > 24)
                || (!s.rests.is_empty() && s.rests.len() != s.heights.len())
                || s.rests
                    .iter()
                    .any(|r| !r.is_finite() || *r < 0.0 || *r > 1.5)
                || s.entry_height > 24
                || [s.dash_beat, s.chain_dash_beat, s.exit_beat, s.cue_beat]
                    .iter()
                    .flatten()
                    .any(|b| !b.is_finite() || *b < s.beat || *b >= map.beats.len() as f64 - 1.0)
                || (!s.widths.is_empty()
                    && (s.widths.len() != s.heights.len()
                        || s.widths.iter().any(|w| !(1..=3).contains(w))))
                || (!s.rest_types.is_empty()
                    && (s.rest_types.len() != s.heights.len()
                        || s.rest_types.iter().any(|t| {
                            !matches!(t.as_str(), "ceiling" | "spike" | "gap" | "gap_ceiling")
                        })))
                || (!s.gate_releases.is_empty() && s.gate_releases.len() != s.heights.len())
                || (!s.spike_counts.is_empty()
                    && (s.spike_counts.len() != s.heights.len()
                        || s.spike_counts.iter().any(|n| !(1..=3).contains(n))))
                || (!s.spike_offsets.is_empty()
                    && (s.spike_offsets.len() != s.heights.len()
                        || s.spike_offsets.iter().any(|n| !(-1..=1).contains(n))))
                || (!s.release_roofs.is_empty()
                    && (s.release_roofs.len() != s.heights.len()
                        || s.release_roofs.iter().any(|h| !matches!(h, 0 | 2 | 3))))
                || (!s.roof_offsets.is_empty()
                    && (s.roof_offsets.len() != s.heights.len()
                        || s.roof_offsets.iter().any(|v| *v > 1)))
                || s.gate_releases
                    .iter()
                    .flatten()
                    .any(|b| !b.is_finite() || *b < 0.0 || *b >= map.beats.len() as f64 - 1.0)
        }) {
            return Err("Invalid authored score".into());
        }
        let time_at = |beat: f64| Self::beat_time(&map, beat);
        let dash_times: Vec<_> = score
            .iter()
            .filter(|s| matches!(s.kind.as_str(), "dash_route" | "gravity_route"))
            .flat_map(|s| {
                [s.dash_beat, s.chain_dash_beat]
                    .into_iter()
                    .flatten()
                    .map(time_at)
            })
            .collect();
        let position_at = |time: f64| {
            time * speed
                + if map.dash_carry {
                    dash_times
                        .iter()
                        .map(|d| {
                            let f = ((time - d) / (0.5 * beat_period)).clamp(0.0, 1.0);
                            3.0 * SIZE * f * f * (3.0 - 2.0 * f)
                        })
                        .sum::<f64>()
                } else {
                    0.0
                }
        };
        let mut objects = Vec::new();
        let mut actions = Vec::new();
        let mut pits = Vec::new();
        let mut stair_start = None;
        let mut continuation_end: Option<(f64, u8)> = None;
        let mut relay_end: Option<(f64, u8)> = None;
        let mut generated_wells = map.gravity_wells.clone();
        let mut high_rise_last: Option<(f64, f64)> = None;
        let event_end = position_at(map.duration) / SIZE + 1.0;
        if map.reference_events.gravity_tiles.len() > 16
            || map
                .reference_events
                .gravity_tiles
                .iter()
                .any(|x| !x.is_finite() || *x < 0.0 || *x > event_end)
            || map
                .reference_events
                .gravity_tiles
                .windows(2)
                .any(|p| p[1] <= p[0])
            || map.reference_events.rising_tiles.len() > 16
            || map.reference_events.falling_tiles.len() > 16
            || map
                .reference_events
                .rising_tiles
                .iter()
                .chain(&map.reference_events.falling_tiles)
                .any(|r| {
                    !r[0].is_finite()
                        || !r[1].is_finite()
                        || r[0] < 0.0
                        || r[1] <= r[0]
                        || r[1] > event_end
                })
        {
            return Err("Invalid reference presentation event".into());
        }
        if let Some(reference) = &map.reference_geometry {
            if !score.is_empty()
                || reference.objects.len() > 5000
                || !reference.end_tile.is_finite()
                || reference.end_tile <= 0.0
                || (reference.end_tile * SIZE - (map.duration * speed + SIZE / 2.0)).abs() > 1e-5
            {
                return Err("Invalid direct-reference geometry or finish".into());
            }
            for o in &reference.objects {
                if !matches!(o.kind.as_str(), "block" | "spike")
                    || !o.x.is_finite()
                    || !o.y.is_finite()
                    || o.x < 0.0
                    || o.x > 10000.0
                    || !(0.0..=100.0).contains(&o.y)
                {
                    return Err("Invalid reference object".into());
                }
                objects.push(Object {
                    kind: if o.kind == "block" { "enemy" } else { "spike" }.into(),
                    x: o.x * SIZE,
                    y: o.y * SIZE,
                    w: SIZE,
                    h: if o.kind == "block" {
                        SIZE
                    } else {
                        SIZE * 123.0 / 128.0
                    },
                    cue: None,
                });
            }
            for pit in &reference.pits {
                if !pit.start.is_finite()
                    || !pit.end.is_finite()
                    || pit.start < 0.0
                    || pit.end <= pit.start
                    || pit.end > 10000.0
                {
                    return Err("Invalid reference pit".into());
                }
                pits.push((pit.start * SIZE, pit.end * SIZE));
            }
            for action in &map.reference_actions {
                if !action.time.is_finite()
                    || action.time < 0.0
                    || action.time >= map.duration
                    || !matches!(action.action.as_str(), "hold" | "release")
                {
                    return Err("Invalid reference control".into());
                }
            }
            actions = map.reference_actions.clone();
        }
        for segment in &score {
            let time = time_at(segment.beat);
            let connected_relay = relay_end.is_some();
            let connected_boundary = connected_relay || continuation_end.is_some();
            if let Some((end, height)) = relay_end.take() {
                if !matches!(
                    segment.kind.as_str(),
                    "dash_route" | "terrain_run" | "gravity_route"
                ) || segment.entry_height != height
                    || (time - end).abs() > 1e-6
                {
                    return Err(
                        "Dash relay must preserve its raised landing and next launch".into(),
                    );
                }
            }
            if let Some((end, height)) = continuation_end.take() {
                if !matches!(
                    segment.kind.as_str(),
                    "terrain_run" | "dash_route" | "gravity_route"
                ) || segment.entry_height != height
                    || (time - end).abs() > 1e-6
                {
                    return Err(format!("Terrain continuation must preserve time and elevation: expected {} at {:.9}s, got {} height{} at {:.9}s",height,end,segment.kind,segment.entry_height,time));
                }
            } else if segment.entry_height != 0 && !connected_boundary {
                return Err("Raised terrain entry requires a preceding continuation".into());
            }
            let x = position_at(time);
            if matches!(segment.kind.as_str(), "dash_route" | "gravity_route") {
                // The preceding terrain emits a handoff hold. This route
                // owns that same launch as a fresh jump; sub-microsecond
                // serialization differences must not turn it into two inputs.
                actions.retain(|a| !(a.action == "hold" && (a.time - time).abs() < 1e-6));
            }
            match segment.kind.as_str() {
                "gravity_route" => {
                    high_rise_last = None;
                    let cue = time_at(
                        segment
                            .cue_beat
                            .ok_or("Gravity route needs a musical pull cue")?,
                    );
                    let dash = segment.dash_beat.map(time_at);
                    if dash.is_some_and(|d| {
                        !map.dash_carry
                            || !((0.35 - 1e-6)..=(0.65 + 1e-6))
                                .contains(&((d - time) / beat_period))
                            || (d - cue).abs() > 1e-7
                    }) {
                        return Err(
                            "Slingshot requires a measured near-apex dash cue and forward carry"
                                .into(),
                        );
                    }
                    let pull_cue = dash.map_or(cue, |d| d + 0.5 * beat_period);
                    let exit = time_at(
                        segment
                            .exit_beat
                            .ok_or("Gravity route needs a landing handoff")?,
                    );
                    let landing = exit - 0.08 * beat_period;
                    let strength = if segment.well_strength == 0.0 {
                        2.0
                    } else {
                        segment.well_strength
                    };
                    if cue <= time + 0.1 * beat_period
                        || cue >= time + 0.75 * beat_period
                        || landing <= pull_cue + 0.15
                        || landing - pull_cue > if dash.is_some() { 2.5 } else { 1.5 }
                        || !(0.1..=3.0).contains(&strength)
                        || !segment.continuation
                    {
                        return Err("Gravity route requires a bounded musical cue, attraction strength, and supported continuation".into());
                    }
                    let top = segment.height as f64 * SIZE;
                    // A whole-cell landing starts half a cube before its
                    // center arrives. The target sits inside real support.
                    let left = ((position_at(landing) - 0.5 * SIZE) / SIZE).floor() * SIZE;
                    let right = if segment.dash_variant == "capture_down" {
                        (((position_at(exit) - 0.5 * SIZE) / SIZE).ceil() * SIZE).max(left + SIZE)
                    } else {
                        ((position_at(exit) + 1.5 * SIZE) / SIZE).ceil() * SIZE
                    };
                    objects.push(Object {
                        kind: "enemy".into(),
                        x: left,
                        y: top - SIZE,
                        w: (right - left).max(SIZE),
                        h: SIZE,
                        cue: None,
                    });
                    pits.push((((x + SIZE) / SIZE).round() * SIZE, right));
                    if segment.dash_variant == "capture_down" {
                        // A dive threads beneath this enemy bridge. A plain
                        // horizontal dash remains high and hits the bridge.
                        objects.push(Object {
                            kind: "ceiling".into(),
                            x: left,
                            y: top + 4.0 * SIZE,
                            w: 3.0 * SIZE,
                            h: ((segment.entry_height - segment.height).max(1) as f64 + 4.0) * SIZE,
                            cue: None,
                        });
                    }
                    // A preceding walk/terrain continuation may already arm
                    // a hold at this exact handoff. The route owns one fresh
                    // jump, not two launch instructions for the same press.
                    actions.retain(|action| {
                        !(action.action == "hold" && (action.time - time).abs() < 1e-6)
                    });
                    actions.push(Action {
                        time,
                        action: "jump".into(),
                    });
                    actions.push(Action {
                        time: time + 0.12 * beat_period,
                        action: "release".into(),
                    });
                    if let Some(dash) = dash {
                        actions.push(Action {
                            time: dash,
                            action: "dash".into(),
                        });
                        actions.push(Action {
                            time: dash + 0.12 * beat_period,
                            action: "release".into(),
                        });
                    }
                    if let Some(chain) = segment.chain_dash_beat.map(time_at) {
                        if !segment.dash_variant.starts_with("capture_up")
                            || chain <= pull_cue
                            || chain + 0.5 * beat_period >= landing
                        {
                            return Err(
                                "A well dash chain must fit inside an upward slingshot arc".into(),
                            );
                        }
                        actions.push(Action {
                            time: chain,
                            action: "dash".into(),
                        });
                        actions.push(Action {
                            time: chain + 0.12 * beat_period,
                            action: "release".into(),
                        });
                        objects.push(Object {
                            kind: "chain_beam".into(),
                            x: position_at(chain) - SIZE,
                            y: top,
                            w: 2.0 * SIZE,
                            h: 4.0 * SIZE,
                            cue: Some(chain),
                        });
                    }
                    let field_left = ((position_at(cue) - SIZE) / SIZE).floor() * SIZE;
                    let field_top = (segment.entry_height.max(segment.height) as f64 + 6.0) * SIZE;
                    let target_x = left + SIZE;
                    let arrival_time = pull_cue
                        + (left
                            - SIZE / 2.0
                            - position_at(pull_cue)
                            - if segment.chain_dash_beat.is_some() {
                                3.0 * SIZE
                            } else {
                                0.0
                            })
                            / speed;
                    if arrival_time - pull_cue < 0.15 {
                        return Err("Gravity route pull cue leaves less than150ms before the actual landing face".into());
                    }
                    let incoming = cue - time;
                    // A resolved route in an exported manifest replaces its
                    // prior generated copy rather than duplicating the event.
                    generated_wells.retain(|well| {
                        (well.cue - pull_cue).abs() > 1e-7
                            && (well.dash_cue.unwrap_or(well.cue) - cue).abs() > 1e-7
                    });
                    generated_wells.push(GravityWell {
                        cue: pull_cue,
                        x: field_left,
                        y: 0.0,
                        w: right - field_left,
                        h: field_top,
                        impulse: 0.0,
                        target_x: Some(target_x),
                        target_y: Some(top),
                        strength,
                        duration: arrival_time - pull_cue,
                        start_y: Some(
                            segment.entry_height as f64 * SIZE + jump * incoming
                                - 0.5 * gravity * incoming * incoming,
                        ),
                        start_vy: Some(jump - gravity * incoming),
                        arrival_time: Some(arrival_time),
                        dash_cue: dash,
                        capture: if segment.dash_variant.starts_with("capture_") {
                            let foot = segment.entry_height as f64 * SIZE + jump * incoming
                                - 0.5 * gravity * incoming * incoming;
                            let travel = arrival_time
                                - pull_cue
                                - if segment.chain_dash_beat.is_some() {
                                    0.5 * beat_period
                                } else {
                                    0.0
                                };
                            let redirect_vy =
                                (top + 0.25 * SIZE - foot) / travel + 0.5 * gravity * travel;
                            if redirect_vy.abs() > 3.0 * jump
                                || (segment.dash_variant == "capture_up" && redirect_vy <= 0.0)
                                || (segment.dash_variant == "capture_down" && redirect_vy >= 0.0)
                            {
                                return Err(
                                    "Capture slingshot needs a bounded signed ballistic exit"
                                        .into(),
                                );
                            }
                            if let Some(chain) = segment.chain_dash_beat.map(time_at) {
                                let phase = (chain - pull_cue) * gravity / (2.0 * redirect_vy);
                                if !(0.35..=0.65).contains(&phase) {
                                    return Err(
                                        "Well chain cue must meet the new ballistic arc's apex"
                                            .into(),
                                    );
                                }
                                let dt = chain - pull_cue;
                                let chain_foot = foot + redirect_vy * dt - 0.5 * gravity * dt * dt;
                                for attack in objects.iter_mut().filter(|o| {
                                    o.kind == "chain_beam"
                                        && o.cue.is_some_and(|c| (c - chain).abs() < 1e-7)
                                }) {
                                    attack.y = ((chain_foot - 0.5 * SIZE) / SIZE).floor() * SIZE;
                                    attack.h = 2.0 * SIZE;
                                }
                            }
                            Some(SlingshotCapture {
                                x: position_at(pull_cue),
                                y: foot + SIZE / 2.0,
                                radius: SIZE * 0.75,
                                redirect_vy,
                                landing_time: arrival_time,
                            })
                        } else {
                            None
                        },
                    });
                    continuation_end = Some((exit, segment.height));
                }
                "dash_route" => {
                    high_rise_last = None;
                    let bounce = segment.dash_variant == "bounce";
                    if segment.entry_height > 0 && !connected_boundary {
                        return Err("Raised dash entry needs a preceding landing".into());
                    }
                    let dash = time_at(
                        segment
                            .dash_beat
                            .ok_or("Dash route needs a measured apex cue")?,
                    );
                    let exit = time_at(
                        segment
                            .exit_beat
                            .ok_or("Dash route needs its recovery boundary")?,
                    );
                    let phase = (dash - time) / beat_period;
                    let delta = segment.height as f64 - segment.entry_height as f64;
                    if !(-6.0..=1.0).contains(&delta)
                        || !(0.35 - 1e-6..=0.65 + 1e-6).contains(&phase)
                    {
                        return Err("Dash route needs a reachable landing and apex cue".into());
                    }
                    let root = (1.0 + (1.0 - delta / 2.02).sqrt()) / 2.0;
                    let landing = time + (root + 0.5) * beat_period;
                    if phase >= root
                        || exit + 1e-6 < landing + if bounce { 0.0 } else { 0.2 * beat_period }
                        || (!bounce
                            && exit + 1e-6
                                < dash + if map.dash_carry { 0.5 } else { 1.75 } * beat_period)
                    {
                        return Err(format!("Dash route must land and recover before the next launch: {} J{:.9} D{:.9} L{:.9} exit{:.9}",map.id,time,dash,landing,exit));
                    }
                    let age = (landing - dash) / beat_period;
                    let fraction = if age <= 0.25 {
                        age / 0.25
                    } else {
                        1.0 - (age - 0.25) / 1.25
                    }
                    .clamp(0.0, 1.0);
                    let landing_x = position_at(landing)
                        + if map.dash_carry {
                            0.0
                        } else {
                            3.0 * SIZE * fraction * fraction * (3.0 - 2.0 * fraction)
                        };
                    let left = landing_x - 0.5 * SIZE;
                    objects.push(Object {
                        kind: "enemy".into(),
                        x: left,
                        y: segment.height as f64 * SIZE - SIZE,
                        w: position_at(exit) + 0.5 * SIZE - left,
                        h: SIZE,
                        cue: None,
                    });
                    // The landing enemies provide support. A second ground
                    // row underneath the wait made raised routes look flat.
                    pits.push((
                        ((x + SIZE) / SIZE).round() * SIZE,
                        (position_at(exit) + SIZE).max(left),
                    ));
                    let kind = if matches!(segment.uw.as_str(), "lightning" | "blackhole") {
                        segment.uw.clone()
                    } else {
                        "shockwave".into()
                    };
                    if segment.dash_variant.is_empty() || segment.dash_variant == "transfer" {
                        objects.push(Object {
                            kind,
                            x: position_at(dash) + SIZE / 2.0 + speed * 0.060,
                            y: segment.entry_height as f64 * SIZE,
                            w: SIZE,
                            h: 3.0 * SIZE,
                            cue: Some(dash),
                        });
                    }
                    if segment.dash_variant == "underpass" {
                        let roof_left = ((landing_x / SIZE) + 1.5).ceil() * SIZE;
                        let roof_right = ((position_at(exit) / SIZE) - 1.0).floor() * SIZE;
                        if roof_right > roof_left {
                            objects.push(Object {
                                kind: "ceiling".into(),
                                x: roof_left,
                                y: (segment.height as f64 + 2.0) * SIZE,
                                w: roof_right - roof_left,
                                h: SIZE,
                                cue: None,
                            });
                        }
                    }
                    actions.extend([
                        Action {
                            time,
                            action: "jump".into(),
                        },
                        Action {
                            time: time + 0.12 * beat_period,
                            action: "release".into(),
                        },
                        Action {
                            time: dash,
                            action: "dash".into(),
                        },
                        Action {
                            time: if bounce {
                                exit + 0.12 * beat_period
                            } else {
                                dash + 0.12 * beat_period
                            },
                            action: "release".into(),
                        },
                    ]);
                    if bounce {
                        actions.pop();
                    }
                    relay_end = Some((exit, segment.height));
                }
                "terrain_run" => {
                    if segment.heights.is_empty()
                        || (!segment.continuation
                            && (segment.heights.len() < 2 || segment.heights.last() != Some(&0)))
                    {
                        return Err(
                            "Terrain run needs a landing; completed runs need two and end on ground".into(),
                        );
                    }
                    actions.retain(|a| !(a.action == "hold" && (a.time - time).abs() < 1e-6));
                    actions.push(Action {
                        time,
                        action: "hold".into(),
                    });
                    let mut landing_time = time;
                    let mut height = segment.entry_height as f64 * SIZE;
                    let mut last_launch = time;
                    for (index, target) in segment.heights.iter().enumerate() {
                        let next = *target as f64 * SIZE;
                        if next - height > 2.0 * SIZE + 1e-7 {
                            return Err("Terrain rises must be at most two cubes per hop".into());
                        }
                        last_launch = landing_time;
                        let discriminant = jump * jump - 2.0 * gravity * (next - height);
                        landing_time += (jump + discriminant.sqrt()) / gravity;
                        let final_landing = index + 1 == segment.heights.len();
                        let rest = segment.rests.get(index).copied().unwrap_or(0.0);
                        let spike_count = segment
                            .spike_counts
                            .get(index)
                            .copied()
                            .unwrap_or(segment.count.max(1));
                        let spike_offset =
                            segment.spike_offsets.get(index).copied().unwrap_or(0) as f64 * SIZE;
                        let spike_rest = rest > 0.0
                            && segment.rest_types.get(index).is_some_and(|t| t == "spike");
                        let gap_rest = rest > 0.0
                            && segment
                                .rest_types
                                .get(index)
                                .is_some_and(|t| t == "gap" || t == "gap_ceiling");
                        let gap_overhang = rest > 0.0
                            && segment
                                .rest_types
                                .get(index)
                                .is_some_and(|t| t == "gap_ceiling");
                        let mut width = if rest > 0.0 {
                            if gap_rest || spike_rest {
                                (rest * 4.8 + 1.0) * SIZE
                            } else {
                                (rest * 4.8 + 2.0).ceil().max(6.0) * SIZE
                            }
                        } else if final_landing && !segment.continuation {
                            4.0 * SIZE
                        } else {
                            segment.widths.get(index).copied().unwrap_or(2) as f64 * SIZE
                        };
                        if final_landing && rest > 0.0 && !segment.continuation {
                            return Err("Final terrain landing cannot require another hop".into());
                        }
                        if final_landing {
                            if let Some(exit) = segment.exit_beat.map(time_at) {
                                if exit < landing_time || !segment.continuation {
                                    return Err(
                                        "Terrain handoff needs a supported continuation".into()
                                    );
                                }
                            }
                        }
                        let mut left = if next - height > 1.5 * SIZE {
                            position_at(landing_time) + 0.25 * SIZE
                        } else {
                            position_at(landing_time)
                                - if rest > 0.0
                                    || final_landing
                                        && (!segment.continuation || segment.exit_beat.is_some())
                                {
                                    0.5 * SIZE
                                } else {
                                    width / 2.0
                                }
                        };
                        if next > height {
                            // An early jump must clear the last triangle, not
                            // land on the next raised shelf over its tip.
                            let spike_end = objects
                                .iter()
                                .filter(|o| {
                                    o.kind == "spike"
                                        && (o.y - height).abs() < 1e-7
                                        && o.x > position_at(last_launch)
                                        && o.x < position_at(last_launch) + 4.0 * SIZE
                                })
                                .map(|o| o.x + o.w)
                                .fold(f64::NEG_INFINITY, f64::max);
                            if spike_end.is_finite() {
                                left = left.max(spike_end + 0.1 * SIZE);
                            }
                        }
                        if map.grid_snap && next - height > 1.5 * SIZE {
                            if let Some((previous_face, previous_top)) = high_rise_last {
                                if (previous_top - height).abs() < 1e-7 {
                                    // Two-cell face spacing is shorter than
                                    // the 2.16..2.64-cell clearance interval
                                    // of a normal +2 ascent. Keep whole cells,
                                    // leaving a short walk to align the hop.
                                    left = left.max(previous_face + 3.0 * SIZE);
                                }
                            }
                            high_rise_last = Some(((left / SIZE).round() * SIZE, next));
                        } else {
                            high_rise_last = None;
                        }
                        // Walking support follows the cube's half-width at
                        // landing and launch; whole-cube padding made almost
                        // every release passage a broad uninterrupted bridge.
                        if gap_rest || spike_rest {
                            width = (position_at(landing_time + rest * beat_period) + 0.5 * SIZE
                                - left)
                                .max(SIZE);
                        }
                        if final_landing {
                            if let Some(exit) = segment.exit_beat.map(time_at) {
                                width = width.max(position_at(exit) + 0.5 * SIZE - left);
                            }
                        }
                        objects.push(Object {
                            kind: "enemy".into(),
                            x: left,
                            y: next - SIZE,
                            w: width,
                            h: SIZE,
                            cue: None,
                        });
                        if rest > 0.0 {
                            let roof_row = segment.release_roofs.get(index).copied().unwrap_or(0);
                            if roof_row > 0 {
                                // A held touchdown hop enters this whole-cell
                                // overhang near its apex. The authored rest
                                // carries the deliberate next jump past it.
                                let near_peak =
                                    (1.0 - (1.0 - (roof_row as f64 - 1.0) / 2.02).sqrt()) / 2.0
                                        * flight;
                                let held_left = position_at(landing_time + near_peak) - 0.5 * SIZE;
                                let fresh_left =
                                    position_at(landing_time + rest * beat_period + near_peak)
                                        - 0.5 * SIZE;
                                let right = (fresh_left / SIZE).floor() * SIZE
                                    + segment.roof_offsets.get(index).copied().unwrap_or(0) as f64
                                        * SIZE;
                                if right <= held_left + 1e-7 {
                                    return Err("Release roof needs whole-cell clearance between held and timed hops".into());
                                }
                                objects.push(Object {
                                    kind: "ceiling".into(),
                                    x: right - SIZE,
                                    y: next + roof_row as f64 * SIZE,
                                    w: SIZE,
                                    h: SIZE,
                                    cue: None,
                                });
                            }
                            // Release on the incoming arc, walk beneath a low
                            // enemy bridge, then re-press on the next attack.
                            let release = segment
                                .gate_releases
                                .get(index)
                                .copied()
                                .flatten()
                                .map(time_at)
                                .unwrap_or(landing_time - 0.25 * flight);
                            if release <= last_launch + 0.05 * flight
                                || release >= landing_time - 0.05 * flight
                            {
                                return Err(
                                    "Raised gate release must occur inside its incoming jump"
                                        .into(),
                                );
                            }
                            actions.push(Action {
                                time: release,
                                action: "release".into(),
                            });
                            if spike_rest {
                                // The spike group sits on its own enemy island.
                                // A broad filled bridge here made every passage
                                // look alike and gave unwanted recovery landings.
                                let count = spike_count;
                                let spacing = if map.difficulty == 4 && count == 2 && !map.grid_snap
                                {
                                    2.0
                                } else {
                                    1.0
                                };
                                let span = 1.0 + (count - 1) as f64 * spacing;
                                objects.push(Object {
                                    kind: "enemy".into(),
                                    x: position_at(landing_time + rest * beat_period)
                                        + 1.9 * SIZE
                                        + spike_offset
                                        - (count - 1) as f64 * spacing * SIZE / 2.0,
                                    y: next - SIZE,
                                    w: span * SIZE,
                                    h: SIZE,
                                    cue: None,
                                });
                            }
                            for n in 0..if gap_rest && !gap_overhang {
                                0
                            } else if spike_rest {
                                spike_count
                            } else {
                                1
                            } {
                                objects.push(Object {
                                    kind: if spike_rest {
                                        "spike".into()
                                    } else {
                                        "ceiling".into()
                                    },
                                    x: if spike_rest {
                                        position_at(landing_time + rest * beat_period)
                                            + 1.9 * SIZE
                                            + spike_offset
                                            + (n as f64 - (spike_count - 1) as f64 / 2.0)
                                                * SIZE
                                                * if map.difficulty == 4
                                                    && spike_count == 2
                                                    && !map.grid_snap
                                                {
                                                    2.0
                                                } else {
                                                    1.0
                                                }
                                    } else {
                                        let raw = position_at(landing_time) + 0.25 * SIZE;
                                        if map.grid_snap && gap_overhang && next < height - SIZE {
                                            // A descending cube must fit
                                            // below the roof before its full
                                            // right edge enters the first
                                            // cell. Nearest rounding can put
                                            // a roof across the incoming arc.
                                            let clearance = next + SIZE;
                                            let discriminant =
                                                jump * jump - 2.0 * gravity * (clearance - height);
                                            let clear_time = last_launch
                                                + (jump + discriminant.sqrt()) / gravity;
                                            ((raw / SIZE).round() * SIZE).max(
                                                ((position_at(clear_time) + 0.5 * SIZE) / SIZE)
                                                    .ceil()
                                                    * SIZE,
                                            )
                                        } else {
                                            raw
                                        }
                                    },
                                    y: if spike_rest { next } else { next + 2.0 * SIZE },
                                    w: SIZE,
                                    h: SIZE,
                                    cue: None,
                                });
                            }
                            landing_time += rest * beat_period;
                            actions.push(Action {
                                time: landing_time,
                                action: "hold".into(),
                            });
                        }
                        height = next;
                    }
                    if landing_time >= map.duration - beat_period {
                        return Err("Terrain run exceeds the musical route".into());
                    }
                    if let (true, Some(release_beat)) = (segment.continuation, segment.ceiling_beat)
                    {
                        let release = time_at(release_beat);
                        if release <= last_launch + 0.05 * flight
                            || release >= landing_time - 0.05 * flight
                        {
                            return Err(
                                "Terrain handoff release must occur during its final jump".into()
                            );
                        }
                        actions.push(Action {
                            time: release,
                            action: "release".into(),
                        });
                    }
                    if segment.continuation {
                        continuation_end = Some((
                            segment.exit_beat.map(time_at).unwrap_or(landing_time),
                            segment.heights.last().copied().unwrap(),
                        ));
                    }
                    if !segment.continuation {
                        let release = segment
                            .ceiling_beat
                            .map(time_at)
                            .unwrap_or(last_launch + 0.5 * (landing_time - last_launch));
                        if release <= last_launch + 0.05 * flight
                            || release >= landing_time - 0.05 * flight
                        {
                            return Err("Terrain release must occur inside its final jump".into());
                        }
                        actions.push(Action {
                            time: release,
                            action: "release".into(),
                        });
                        // The final platform is a real low passage: holding would
                        // launch into enemy bodies instead of accepting every input.
                        objects.push(Object {
                            kind: "ceiling".into(),
                            x: position_at(landing_time) + SIZE,
                            y: 2.0 * SIZE,
                            w: 2.0 * SIZE,
                            h: SIZE,
                            cue: None,
                        });
                    }
                    pits.push((
                        ((x + SIZE) / SIZE).ceil() * SIZE,
                        ((position_at(landing_time) + 3.0 * SIZE) / SIZE).ceil() * SIZE,
                    ));
                }
                "jump_spike" => {
                    actions.push(Action {
                        time,
                        action: "jump".into(),
                    });
                    let count = segment.count.max(1);
                    let width = SIZE;
                    for n in 0..count {
                        let center = x + 2.4 * SIZE + (n as f64 - (count - 1) as f64 / 2.0) * SIZE;
                        objects.push(Object {
                            kind: "spike".into(),
                            x: center - width / 2.0,
                            y: 0.0,
                            w: width,
                            h: width * 123.0 / 128.0,
                            cue: None,
                        });
                    }
                }
                "hold_chain" => {
                    actions.push(Action {
                        time,
                        action: "hold".into(),
                    });
                    for hop in 0..segment.count.max(1) {
                        let origin = position_at(time_at(segment.beat + hop as f64));
                        for n in 0..2 {
                            let width = SIZE;
                            objects.push(Object {
                                kind: "spike".into(),
                                x: origin + 2.4 * SIZE + (n as f64 - 0.5) * SIZE - width / 2.0,
                                y: 0.0,
                                w: width,
                                h: width * 123.0 / 128.0,
                                cue: None,
                            });
                        }
                    }
                }
                "release_corridor" => {
                    actions.push(Action {
                        time,
                        action: "release".into(),
                    });
                    objects.push(Object {
                        kind: "ceiling".into(),
                        x: position_at(time_at(
                            segment.ceiling_beat.unwrap_or(segment.beat + 0.65),
                        )),
                        y: 2.0 * SIZE,
                        w: 4.0 * SIZE,
                        h: SIZE,
                        cue: None,
                    });
                }
                "jump_pit" | "dash_void_start" => {
                    actions.push(Action {
                        time,
                        action: "jump".into(),
                    });
                    let left = ((x + SIZE) / SIZE).round() * SIZE;
                    let width = if segment.kind == "dash_void_start" {
                        5.0 * SIZE
                    } else {
                        3.0 * SIZE
                    };
                    pits.push((left, left + width));
                }
                "dash_trigger_required" => {
                    actions.push(Action {
                        time,
                        action: "dash".into(),
                    });
                    let kind = if matches!(segment.uw.as_str(), "lightning" | "blackhole") {
                        segment.uw.clone()
                    } else {
                        "shockwave".into()
                    };
                    objects.push(Object {
                        kind,
                        x: x + SIZE / 2.0 + speed * 0.060,
                        y: 0.0,
                        w: SIZE,
                        h: 3.0 * SIZE,
                        cue: Some(time),
                    });
                }
                "safe_landing_platform" => {}
                "release_input" => actions.push(Action {
                    time,
                    action: "release".into(),
                }),
                "jump_step" => {
                    actions.push(Action {
                        time,
                        action: "jump".into(),
                    });
                    if segment.height > 0 {
                        if stair_start.is_none() {
                            stair_start = Some(((x + SIZE) / SIZE).floor() * SIZE);
                        }
                        let width = 5.0 * SIZE;
                        objects.push(Object {
                            kind: "enemy".into(),
                            x: x + 1.8 * SIZE,
                            y: 0.0,
                            w: width,
                            h: segment.height as f64 * SIZE,
                            cue: None,
                        });
                    } else if let Some(left) = stair_start.take() {
                        pits.push((left, ((x + 3.0 * SIZE) / SIZE).ceil() * SIZE));
                    }
                }
                _ => return Err(format!("Unknown score segment: {}", segment.kind)),
            }
        }
        if stair_start.is_some() {
            return Err("Incomplete stair phrase".into());
        }
        if actions.is_empty() && map.reference_geometry.is_none() {
            return Err("Map needs an authored musical encounter".into());
        }
        actions.sort_by(|a, b| a.time.total_cmp(&b.time));
        if continuation_end.is_some() || relay_end.is_some() {
            return Err("Terrain continuation has no following scene".into());
        }
        let finish_x = position_at(map.duration) + SIZE / 2.0 - 1e-7;
        generated_wells.sort_by(|a, b| a.cue.total_cmp(&b.cue));
        if generated_wells.len() > 8
            || generated_wells
                .windows(2)
                .any(|pair| (pair[1].cue - pair[0].cue).abs() < 1e-7)
        {
            return Err("At most eight distinct musical gravity wells are supported".into());
        }
        for well in &generated_wells {
            if well.x + well.w > finish_x + SIZE {
                return Err("Gravity well field extends beyond the course".into());
            }
            if well.target_x.is_some_and(|target| target > finish_x + SIZE)
                || well.target_y.is_some_and(|target| target > 128.0 * SIZE)
            {
                return Err("Gravity well target extends beyond the course".into());
            }
            objects.push(Object {
                kind: "gravitywell".into(),
                x: well.x,
                y: well.y,
                w: well.w,
                h: well.h,
                cue: Some(if well.capture.is_some() {
                    well.cue
                } else {
                    well.dash_cue.unwrap_or(well.cue)
                }),
            });
        }
        for tile in -10..((finish_x / SIZE).ceil() as i32 + 8) {
            let x = tile as f64 * SIZE;
            if pits
                .iter()
                .any(|&(left, right)| x >= left - 1e-7 && x < right - 1e-7)
            {
                continue;
            }
            objects.push(Object {
                kind: "terrain".into(),
                x,
                y: 0.0,
                w: SIZE,
                h: 0.0,
                cue: None,
            });
        }
        objects.push(Object {
            kind: "tower".into(),
            x: finish_x,
            y: 0.0,
            w: 4.0 * SIZE,
            h: 9.0 * SIZE,
            cue: None,
        });
        if map.grid_snap {
            for object in &mut objects {
                if !matches!(object.kind.as_str(), "terrain" | "tower") {
                    let right = ((object.x + object.w) / SIZE).round() * SIZE;
                    object.x = (object.x / SIZE).round() * SIZE;
                    object.y = (object.y / SIZE).round() * SIZE;
                    object.w = (right - object.x).max(SIZE);
                    if object.kind != "spike" {
                        object.h = (object.h / SIZE).round().max(1.0) * SIZE;
                    }
                }
            }
            // A raised shelf must not occupy the same grid cell as a mounted
            // triangle. Split spans into real cells, retaining the original
            // obstacle and excluding only intersecting platform material.
            let spikes: Vec<_> = objects
                .iter()
                .filter(|o| o.kind == "spike")
                .cloned()
                .collect();
            let mut cells = Vec::new();
            let mut occupied = std::collections::HashSet::new();
            for object in objects {
                if !matches!(object.kind.as_str(), "enemy" | "ceiling") {
                    cells.push(object);
                    continue;
                }
                for column in 0..(object.w / SIZE).round() as usize {
                    for row in 0..(object.h / SIZE).round() as usize {
                        let x = object.x + column as f64 * SIZE;
                        let y = object.y + row as f64 * SIZE;
                        if spikes.iter().any(|s| {
                            x < s.x + s.w - 1e-7
                                && x + SIZE > s.x + 1e-7
                                && y < s.y + s.h - 1e-7
                                && y + SIZE > s.y + 1e-7
                        }) {
                            continue;
                        }
                        if occupied.insert((
                            object.kind.clone(),
                            (x / SIZE).round() as i64,
                            (y / SIZE).round() as i64,
                        )) {
                            cells.push(Object {
                                kind: object.kind.clone(),
                                x,
                                y,
                                w: SIZE,
                                h: SIZE,
                                cue: object.cue,
                            });
                        }
                    }
                }
            }
            objects = cells;
            if !map.grid_actions.is_empty() {
                if map
                    .grid_actions
                    .iter()
                    .any(|a| !a.time.is_finite() || a.time < 0.0 || a.time >= map.duration)
                    || map.grid_actions.windows(2).any(|a| a[0].time > a[1].time)
                {
                    return Err("Invalid grid replay controls".into());
                }
                actions = map.grid_actions.clone();
            }
        }
        let mut runner = Self {
            map: {
                map.gravity_wells = generated_wells;
                map
            },
            objects,
            actions,
            time: 0.0,
            speed,
            jump,
            gravity,
            flight,
            beat_period,
            dash_motion: beat_period * 0.5,

            collision_indices: Vec::new(),
            collision_max_width: 0.0,
            jump_elapsed: 0.0,
            y: 0.0,
            support_y: 0.0,
            vy: 0.0,
            grounded: true,
            held: false,
            jump_count: 0,
            dash_until: -1.0,
            dash_started: -1.0,
            dash_carry_total: 0.0,
            captured_well: None,
            route_finish: finish_x,

            air_dash_used: false,
            status: "ready",
            death_reason: "",
            input_feedback: "",
            feedback_until: -1.0,
            #[cfg(not(target_arch = "wasm32"))]
            contact_audit: None,
        };
        runner.reindex();
        Ok(runner)
    }

    fn beat_time(map: &MapInput, beat: f64) -> f64 {
        let index = beat.floor() as usize;
        let fraction = beat - index as f64;
        match (map.beats.get(index), map.beats.get(index + 1)) {
            (Some(&a), Some(&b)) => a + (b - a) * fraction,
            (Some(&a), None) if fraction == 0.0 => a,
            _ => map.duration,
        }
    }

    fn reindex(&mut self) {
        self.collision_indices = (0..self.objects.len()).collect();
        self.collision_indices
            .sort_by(|&a, &b| self.objects[a].x.total_cmp(&self.objects[b].x));
        self.collision_max_width = self.objects.iter().map(|o| o.w).fold(0.0, f64::max);
    }

    pub fn restart(&mut self) {
        #[cfg(not(target_arch = "wasm32"))]
        if let Some(events) = &mut self.contact_audit { events.clear(); }
        self.time = 0.0;
        self.y = 0.0;
        self.support_y = 0.0;
        self.jump_elapsed = 0.0;
        self.vy = 0.0;
        self.grounded = true;
        self.dash_until = -1.0;
        self.dash_started = -1.0;
        self.dash_carry_total = 0.0;
        self.captured_well = None;

        self.air_dash_used = false;
        self.held = false;
        self.jump_count = 0;
        self.status = "ready";
        self.death_reason = "";
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub fn enable_contact_audit(&mut self) { self.contact_audit = Some(Vec::new()); }
    #[cfg(not(target_arch = "wasm32"))]
    pub fn contact_audit_events(&self) -> &[ContactAuditEvent] { self.contact_audit.as_deref().unwrap_or(&[]) }
    #[cfg(not(target_arch = "wasm32"))]
    fn audit_launch(&mut self, cause: &'static str) {
        let event=ContactAuditEvent{time:self.time,x:self.x(),support_y:self.support_y,cause};
        if let Some(events)=&mut self.contact_audit { events.push(event); }
    }

    /// Native omission audit: remove an assist while retaining its authored
    /// gap and landing geometry. Browser/manual control has no bypass API.
    #[cfg(not(target_arch = "wasm32"))]
    pub fn disable_gravity_well(&mut self, cue: f64) -> bool {
        let before = self.map.gravity_wells.len();
        self.map
            .gravity_wells
            .retain(|well| (well.cue - cue).abs() > 1e-7);
        self.map.gravity_wells.len() < before
    }

    pub fn advance(&mut self, audio_time: f64) {
        self.advance_to(audio_time, false);
    }

    fn advance_exact(&mut self, audio_time: f64) {
        self.advance_to(audio_time, true);
    }

    fn advance_to(&mut self, audio_time: f64, exact: bool) {
        if !audio_time.is_finite()
            || audio_time < self.time
            || matches!(self.status, "dead" | "won")
        {
            return;
        }
        let target = audio_time.min(self.playback_end());
        if target > 0.0 {
            self.status = "running";
        }
        if self.map.reference_geometry.is_some() {
            let step = 1.0 / 60.0;
            while self.time + step <= target + 1e-9 && self.status == "running" {
                self.step(step);
            }
            if target == self.playback_end() && self.time < target && self.status == "running" {
                self.step(target - self.time);
            }
            return;
        }
        while self.time + 1e-9 < target && self.status == "running" {
            // Keep simulation boundaries on one clock grid, regardless of how
            // frequently rendering or the replay caller advances the runner.
            let mut boundary = ((self.time / STEP + 1e-7).floor() + 1.0) * STEP;
            if self.map.gravity_wells.iter().any(|w| w.capture.is_some())
                && self.dash_until > self.time + 1e-9
                && self.dash_until < boundary
            {
                boundary = self.dash_until;
            }
            for well in &self.map.gravity_wells {
                for event in [well.cue, well.cue + well.duration] {
                    if event > self.time + 1e-9 && event < boundary {
                        boundary = event;
                    }
                }
            }
            if !exact && boundary > target + 1e-9 && target < self.playback_end() {
                break;
            }
            let dt = (boundary - self.time).min(target - self.time);
            self.step(dt);
        }
        if self.status == "running" && (exact || target == self.playback_end()) {
            self.time = target;
        }
    }

    /// The host must deliver event timestamps before advancing past them. Old
    /// input is rejected rather than rewinding a collision or buffering a jump.
    pub fn input(&mut self, action: &str, audio_time: f64) {
        if !audio_time.is_finite()
            || audio_time < self.time - 1e-9
            || !matches!(
                action,
                "jump" | "dash" | "press" | "down" | "up" | "hold" | "release"
            )
        {
            return;
        }
        self.advance_exact(audio_time);
        if self.map.reference_geometry.is_some() && self.time >= self.map.duration {
            return;
        }
        if matches!(action, "up" | "release") {
            self.held = false;
            return;
        }
        if matches!(self.status, "dead" | "won") {
            return;
        }
        if action == "dash" && self.dash_available() {
            self.held = true;
        }
        if matches!(action, "down" | "hold") {
            if self.held {
                if action == "down" && !self.grounded {
                    self.reject_dash("Release first");
                }
                return;
            }
            self.held = true;
        }
        let action = if matches!(action, "press" | "down" | "hold") {
            if self.grounded {
                "jump"
            } else {
                "dash"
            }
        } else {
            action
        };
        match action {
            "jump" if self.grounded => {
                self.launch();
                #[cfg(not(target_arch = "wasm32"))]
                self.audit_launch("fresh_jump");
            }
            "dash" if self.dash_available() => {
                if self.map.dash_carry && self.dash_started >= 0.0 {
                    self.dash_carry_total += 3.0 * SIZE;
                }
                self.dash_started = self.time;
                self.dash_until = self.time + self.dash_motion;
                self.air_dash_used = true;
                self.input_feedback = "";
            }
            "dash" => {
                let reason = if self.grounded {
                    "Jump first"
                } else if self.air_dash_used {
                    "Dash spent"
                } else if self.dash_input_phase() < 0.35 {
                    "Too early"
                } else {
                    "Too late"
                };
                self.reject_dash(reason);
            }
            _ => {}
        }
    }
    fn reject_dash(&mut self, reason: &'static str) {
        self.input_feedback = reason;
        self.feedback_until = self.time + 0.55;
    }
    fn launch(&mut self) {
        self.input_feedback = "";
        self.vy = if self.map.reference_geometry.is_some() {
            SIZE * (15.0 / 45.0) * 60.0
        } else {
            self.jump
        };
        self.jump_elapsed = 0.0;
        self.grounded = false;
        self.air_dash_used = false;
        self.captured_well = None;
        self.jump_count += 1;
    }
    fn dash_offset(&self) -> f64 {
        let age = self.time - self.dash_started;
        if self.dash_started < 0.0 || age < 0.0 {
            return 0.0;
        }
        if self.map.dash_carry {
            let f = (age / self.dash_motion).clamp(0.0, 1.0);
            return self.dash_carry_total + 3.0 * SIZE * f * f * (3.0 - 2.0 * f);
        }
        let rise = self.beat_period * 0.25;
        let phase = if age <= rise {
            age / rise
        } else {
            1.0 - (age - rise) / (self.beat_period * 1.25)
        }
        .clamp(0.0, 1.0);
        3.0 * SIZE * phase * phase * (3.0 - 2.0 * phase)
    }
    fn x(&self) -> f64 {
        self.time * self.speed + self.dash_offset()
    }
    fn route_x(&self) -> f64 {
        self.time * self.speed
            + if self.map.dash_carry {
                self.dash_carry_total
                    + if self.dash_started >= 0.0 && self.time >= self.dash_until {
                        3.0 * SIZE
                    } else {
                        0.0
                    }
            } else {
                0.0
            }
    }
    fn finish_x(&self) -> f64 {
        self.route_finish
    }
    fn dash_input_phase(&self) -> f64 {
        self.captured_well
            .and_then(|index| self.map.gravity_wells.get(index))
            .and_then(|well| {
                well.capture.as_ref().map(|capture| {
                    if capture.redirect_vy > 0.0 {
                        (capture.redirect_vy - self.vy) / (2.0 * capture.redirect_vy)
                    } else {
                        0.5 + (self.time - well.cue) / self.beat_period
                    }
                })
            })
            .unwrap_or(self.jump_elapsed / self.flight)
    }
    fn dash_available(&self) -> bool {
        // Ignore early/late re-presses rather than turning ordinary hopping
        // mistakes into forward bursts. A fresh press near the apex is required.
        let phase = self.dash_input_phase();
        self.map.reference_geometry.is_none()
            && !self.grounded
            && !self.air_dash_used
            && (self.dash_started < 0.0
                || self.time
                    >= self.dash_started
                        + if self.map.dash_carry {
                            self.dash_motion
                        } else {
                            1.5 * self.beat_period
                        }
                        - 1e-9)
            && (0.35..=0.65).contains(&phase)
    }

    fn triangle_hits_player(object: &Object, x: f64, y: f64) -> bool {
        // The physical body matches the displayed cube. Triangle geometry
        // remains sloped; there is no hidden half-width/height forgiveness.
        let left = (x - SIZE / 2.0).max(object.x);
        let right = (x + SIZE / 2.0).min(object.x + object.w);
        if left >= right || y + SIZE <= object.y {
            return false;
        }
        let apex = object.x + object.w / 2.0;
        let nearest = apex.clamp(left, right);
        let height = object.h * (1.0 - (nearest - apex).abs() / (object.w / 2.0));
        y < object.y + height
    }

    fn step(&mut self, dt: f64) {
        if self.map.reference_geometry.is_some() && self.time >= self.map.duration {
            self.held = true;
        }
        if self.grounded && self.held {
            self.launch();
            #[cfg(not(target_arch = "wasm32"))]
            self.audit_launch("held_jump");
        }
        let old_y = self.y;
        let was_airborne = !self.grounded;
        let mut old_vy = self.vy;
        let old_x = self.x();
        let old_time = self.time;
        self.time += dt;
        let x = self.x();
        if self.map.reference_geometry.is_some() {
            // Recovered XNA profile: move with the old velocity, then increase
            // downward acceleration by 1.35 source pixels per 60Hz update.
            self.y += self.vy * dt;
            self.vy = (self.vy - SIZE * (1.3499999 / 45.0) * 3600.0 * dt).max(-SIZE * 40.0);
            self.jump_elapsed += dt;
            self.grounded = false;
            self.reference_contacts(old_y, old_x);
            return;
        }
        // Freeze height and vertical velocity throughout the active dash,
        // including the partial simulation step crossing its exact end time.
        let hang =
            (self.time.min(self.dash_until) - old_time.max(self.dash_started)).clamp(0.0, dt);
        let gravity_dt = if old_time >= self.dash_started && self.time <= self.dash_until {
            0.0
        } else {
            dt - hang
        };
        let mut attraction = 0.0;
        // A slingshot is a physical catch at dash end. It changes velocity
        // once; the remaining flight uses ordinary gravity, without homing.
        if was_airborne
            && old_time <= self.dash_until + 1e-9
            && self.time >= self.dash_until - 1e-9
            && self.dash_started >= 0.0
        {
            for (index, well) in self.map.gravity_wells.iter().enumerate() {
                let Some(capture) = &well.capture else {
                    continue;
                };
                if self.captured_well == Some(index)
                    || (self.dash_until - well.cue).abs() > self.beat_period * 0.08
                    || (x - capture.x).hypot(old_y + SIZE / 2.0 - capture.y) > capture.radius
                {
                    continue;
                }
                self.vy = capture.redirect_vy;
                old_vy = self.vy;
                self.captured_well = Some(index);
                self.air_dash_used = false;
                break;
            }
        }
        if was_airborne && gravity_dt > 0.0 {
            for well in &self.map.gravity_wells {
                if well.capture.is_some() {
                    continue;
                }
                if well.dash_cue.is_some_and(|cue| {
                    (self.dash_started - cue).abs() > 0.08 * self.beat_period
                        || old_time < self.dash_until - 1e-9
                }) {
                    continue;
                }
                if let (Some(target_x), Some(target_y)) = (well.target_x, well.target_y) {
                    if old_time >= well.cue - 1e-9
                        && old_time < well.cue + well.duration - 1e-9
                        && old_x < target_x
                        && old_x + SIZE / 2.0 > well.x
                        && old_x - SIZE / 2.0 < well.x + well.w
                        && old_y + SIZE > well.y
                        && old_y < well.y + well.h
                    {
                        let required = if let (Some(start_y), Some(start_vy), Some(arrival)) =
                            (well.start_y, well.start_vy, well.arrival_time)
                        {
                            // Meet the leading face with clearance AND a
                            // descending body. A center-only positional goal
                            // drives late-cue players upward into the sidewall.
                            let duration = arrival - well.cue;
                            let phase = ((old_time - well.cue) / duration).clamp(0.0, 1.0);
                            let end_y = target_y + 0.25 * SIZE;
                            let end_vy = -0.25 * self.jump;
                            let a = 2.0 * start_y - 2.0 * end_y + duration * (start_vy + end_vy);
                            let b =
                                -3.0 * start_y + 3.0 * end_y - duration * (2.0 * start_vy + end_vy);
                            let c = duration * start_vy;
                            let desired_y = ((a * phase + b) * phase + c) * phase + start_y;
                            let desired_vy =
                                (3.0 * a * phase * phase + 2.0 * b * phase + c) / duration;
                            let desired_acceleration =
                                (6.0 * a * phase + 2.0 * b) / (duration * duration);
                            desired_acceleration
                                + 8.0 * (desired_y - old_y) / (duration * duration)
                                + 4.0 * (desired_vy - old_vy) / duration
                        } else {
                            // Backwards-compatible explicit targeted fields.
                            let remaining = ((target_x - old_x) / self.speed).max(0.04);
                            2.0 * (target_y - old_y - old_vy * remaining) / (remaining * remaining)
                        };
                        attraction += (required + self.gravity)
                            .clamp(-well.strength * self.gravity, well.strength * self.gravity);
                    }
                }
            }
        }
        let effective_gravity =
            self.gravity - attraction.clamp(-3.0 * self.gravity, 3.0 * self.gravity);
        self.y += self.vy * gravity_dt - 0.5 * effective_gravity * gravity_dt * gravity_dt;
        self.vy -= effective_gravity * gravity_dt;
        self.jump_elapsed += gravity_dt;
        self.grounded = false;
        if self.collision_indices.len() != self.objects.len() {
            self.reindex();
        }
        let left = x - SIZE / 2.0 - self.collision_max_width;
        let right = x + SIZE / 2.0;
        let first = self
            .collision_indices
            .partition_point(|&i| self.objects[i].x < left);
        let last = self
            .collision_indices
            .partition_point(|&i| self.objects[i].x <= right);
        // Resolve support before hazards so a tile's top is the real floor,
        // independent of where that segment appeared in the generation list.
        for &index in &self.collision_indices[first..last] {
            let object = &self.objects[index];
            if !matches!(object.kind.as_str(), "terrain" | "enemy") {
                continue;
            }
            if x + SIZE / 2.0 <= object.x || old_x - SIZE / 2.0 >= object.x + object.w {
                continue;
            }
            let top = object.y + object.h;
            if self.vy <= 0.0 && old_y >= top - 1e-7 && self.y <= top + 1e-7 {
                let tau = if attraction.abs() < 1e-12 {
                    ((old_vy
                        + (old_vy * old_vy + 2.0 * self.gravity * (old_y - top))
                            .max(0.0)
                            .sqrt())
                        / self.gravity)
                        .clamp(0.0, gravity_dt)
                } else {
                    // Attraction may invert acceleration. A bounded monotone
                    // descending contact solve avoids selecting the wrong
                    // quadratic root while keeping legacy hops unchanged.
                    let (mut lo, mut hi) = (0.0, gravity_dt);
                    for _ in 0..24 {
                        let mid = (lo + hi) * 0.5;
                        if old_y + old_vy * mid - 0.5 * effective_gravity * mid * mid > top {
                            lo = mid;
                        } else {
                            hi = mid;
                        }
                    }
                    (lo + hi) * 0.5
                };
                let remaining = gravity_dt - tau;
                let contact_x = old_x + (x - old_x) * ((hang + tau) / dt).clamp(0.0, 1.0);
                if contact_x + SIZE / 2.0 <= object.x
                    || contact_x - SIZE / 2.0 >= object.x + object.w
                {
                    continue;
                }
                self.y = top;
                #[cfg(not(target_arch = "wasm32"))]
                if was_airborne || (self.support_y-top).abs()>1e-7 {
                    if let Some(events)=&mut self.contact_audit { events.push(ContactAuditEvent{
                        time:old_time+hang+tau,x:contact_x,support_y:top,cause:"landing"}); }
                }
                self.support_y = top;
                self.jump_elapsed = 0.0;
                self.vy = 0.0;
                self.grounded = true;
                self.captured_well = None;
                self.dash_until = -1.0;
                if self.held && remaining > 1e-9 {
                    #[cfg(not(target_arch = "wasm32"))]
                    if let Some(events)=&mut self.contact_audit { events.push(ContactAuditEvent{
                        time:old_time+hang+tau,x:contact_x,support_y:top,cause:"held_jump"}); }
                    self.vy = self.jump;
                    self.grounded = false;
                    self.air_dash_used = false;
                    self.jump_count += 1;
                    self.y += self.jump * remaining - 0.5 * self.gravity * remaining * remaining;
                    self.vy -= self.gravity * remaining;
                    self.jump_elapsed = remaining;
                }
            } else if self.y < top - 1e-7 && self.y + SIZE > object.y {
                self.status = "dead";
                self.death_reason = "enemy";
                return;
            }
        }
        if self.y < -2.0 * SIZE {
            self.status = "dead";
            self.death_reason = "pit";
            return;
        }
        for &index in &self.collision_indices[first..last] {
            let object = &self.objects[index];
            if matches!(object.kind.as_str(), "terrain" | "enemy") {
                continue;
            }
            if object.kind == "gravitywell" {
                continue;
            }
            if object.kind == "chain_beam"
                && object.cue.is_some_and(|cue| {
                    self.time < cue + 0.012 || self.time > cue + 0.35 * self.beat_period
                })
            {
                continue;
            }
            if object.kind == "tower" {
                if self.time >= self.map.duration - 1e-9
                    && x + SIZE / 2.0 > object.x
                    && x - SIZE / 2.0 < object.x + object.w
                    && self.y < object.y + object.h
                    && self.y + SIZE > object.y
                {
                    self.time = self.map.duration;
                    self.status = "won";
                }
                continue;
            }
            if x + SIZE / 2.0 <= object.x || x - SIZE / 2.0 >= object.x + object.w {
                continue;
            }
            if object.kind == "spike" {
                if Self::triangle_hits_player(object, x, self.y) {
                    self.status = "dead";
                    self.death_reason = "spike";
                    break;
                }
            } else if self.y < object.y + object.h && self.y + SIZE > object.y {
                let uw = matches!(
                    object.kind.as_str(),
                    "shockwave" | "lightning" | "blackhole" | "chain_beam"
                );
                if !uw || self.time > self.dash_until {
                    self.status = "dead";
                    self.death_reason = if uw { "ultimate weapon" } else { "spike" };
                    break;
                }
            }
        }
        // Resolve ordinary contacts first: a cube which lands at the cue is
        // grounded, and a dead cube cannot be rescued by an assist impulse.
        if was_airborne && !self.grounded && self.status == "running" {
            for well in &self.map.gravity_wells {
                if well.capture.is_some() {
                    continue;
                }
                if old_time < well.cue
                    && well.cue <= self.time
                    && !(well.cue >= self.dash_started && well.cue <= self.dash_until)
                    && x + SIZE / 2.0 > well.x
                    && x - SIZE / 2.0 < well.x + well.w
                    && self.y + SIZE > well.y
                    && self.y < well.y + well.h
                {
                    self.vy += well.impulse * self.jump;
                }
            }
        }
        debug_assert!(x >= old_x);
    }

    /// Source-inspired reference contact profile. Original source-order and
    /// half-tile thresholds are retained alongside the 60Hz source movement.
    /// This source build is separate from Steam, whose parity is unverified.
    fn reference_contacts(&mut self, old_y: f64, x: f64) {
        let player_left = x - SIZE / 2.0;
        let mut candidate = None;
        for object in &self.objects {
            let reach = match object.kind.as_str() {
                "enemy" => SIZE,
                "spike" => SIZE * 0.75,
                _ => continue,
            };
            let relative = object.x - player_left;
            if relative >= -reach
                && relative < reach
                && self.y > object.y - SIZE
                && self.y < object.y + SIZE
            {
                candidate = Some(object);
            }
        }
        if let Some(object) = candidate {
            let block = object.kind == "enemy";
            let collision = if block {
                self.y < object.y + SIZE * 0.5
                    && self.y >= object.y - SIZE * 0.5
                    && old_y < object.y + SIZE
            } else {
                self.y < object.y + SIZE * 0.5 && self.y >= object.y
            };
            if collision && self.time < self.map.duration {
                self.status = "dead";
                self.death_reason = if block { "enemy" } else { "spike" };
                return;
            }
            if block && self.y <= object.y + SIZE && self.y > object.y {
                self.y = object.y + SIZE;
                self.support_y = self.y;
                self.vy = 0.0;
                self.jump_elapsed = 0.0;
                self.grounded = true;
            }
        }
        if !self.grounded && self.y <= 0.0 {
            let ground = self
                .objects
                .iter()
                .any(|o| o.kind == "terrain" && x >= o.x && x < o.x + o.w);
            if (ground && old_y >= 0.0) || self.time >= self.map.duration {
                self.y = 0.0;
                self.support_y = 0.0;
                self.vy = 0.0;
                self.jump_elapsed = 0.0;
                self.grounded = true;
            } else {
                self.status = "dead";
                self.death_reason = "pit";
                return;
            }
        }
        if self.time >= self.playback_end() - 1e-9 {
            self.time = self.playback_end();
            self.status = "won";
        }
    }

    fn playback_end(&self) -> f64 {
        self.map.duration
            + if self.map.reference_geometry.is_some() {
                self.map.reference_outro_seconds
            } else {
                0.0
            }
    }

    pub fn snapshot(&self) -> Snapshot<'_> {
        Snapshot {
            time: self.time,
            x: self.x(),
            y: self.y,
            vy: self.vy,
            grounded: self.grounded,
            held: self.held,
            jump_count: self.jump_count,
            dash_active: self.time <= self.dash_until,
            slingshot_active: !self.grounded
                && self
                    .map
                    .gravity_wells
                    .iter()
                    .enumerate()
                    .any(|(index, well)| {
                        if let Some(capture) = &well.capture {
                            self.captured_well == Some(index) && self.time < capture.landing_time
                        } else {
                            well.dash_cue.is_some_and(|cue| {
                                (self.dash_started - cue).abs() <= 0.08 * self.beat_period
                            }) && self.time >= well.cue
                                && self.time < well.cue + well.duration
                        }
                    }),
            dash_ready: self.dash_available() && !self.held,
            input_feedback: if self.time < self.feedback_until {
                self.input_feedback
            } else {
                ""
            },
            inverted: self
                .map
                .reference_events
                .gravity_tiles
                .iter()
                .filter(|x| self.route_x() / SIZE >= **x)
                .count()
                % 2
                == 1,
            dash_offset: self.dash_offset(),
            dash_phase: ((self.time - self.dash_started) / self.dash_motion).clamp(0.0, 1.0),
            status: self.status,
            progress: (self.time / self.map.duration).min(1.0),
            death_reason: self.death_reason,
            objects: &self.objects,
            actions: &self.actions,
            duration: self.playback_end(),
            speed: self.speed,
            jump_phase: (self.jump_elapsed / self.flight).clamp(0.0, 1.0),
            tile_size: SIZE,
            beat_period: self.beat_period,
            route_x: self.route_x(),
            finish_x: self.finish_x(),
            support_y: self.support_y,
        }
    }
    fn json(&self) -> String {
        serde_json::to_string(&self.snapshot()).expect("finite engine state")
    }

    fn motion_json(&self) -> String {
        let state = self.snapshot();
        serde_json::to_string(&MotionSnapshot {
            time: state.time,
            x: state.x,
            y: state.y,
            vy: state.vy,
            grounded: state.grounded,
            held: state.held,
            jump_count: state.jump_count,
            dash_active: state.dash_active,
            slingshot_active: state.slingshot_active,
            dash_ready: state.dash_ready,
            input_feedback: state.input_feedback,
            inverted: state.inverted,
            dash_offset: state.dash_offset,
            dash_phase: state.dash_phase,
            status: state.status,
            progress: state.progress,
            death_reason: state.death_reason,
            duration: state.duration,
            speed: state.speed,
            jump_phase: state.jump_phase,
            tile_size: state.tile_size,
            beat_period: state.beat_period,
            route_x: state.route_x,
            finish_x: state.finish_x,
            support_y: state.support_y,
        })
        .expect("finite engine motion state")
    }

    // Preview the fractional render tick through the same support/hazard
    // resolver as gameplay. Analytic extrapolation alone overshoots a landing
    // and then snaps upward when the next committed tick resolves contact.
    // Restore every mutable physics field; rendering never advances gameplay.
    fn motion_json_at(&mut self, audio_time: f64) -> String {
        if !audio_time.is_finite() || self.status != "running" {
            return self.motion_json();
        }
        #[cfg(not(target_arch = "wasm32"))]
        let audit_length=self.contact_audit.as_ref().map(Vec::len);
        let saved_feedback=(self.input_feedback,self.feedback_until);
        let saved = (
            self.time,
            self.y,
            self.vy,
            self.jump_elapsed,
            self.support_y,
            self.grounded,
            self.held,
            self.jump_count,
            self.dash_until,
            self.dash_started,
            self.dash_carry_total,
            self.air_dash_used,
            self.status,
            self.death_reason,
            self.captured_well,
        );
        // A caller must advance before asking for a render sample. Bound a
        // stale request to one tick rather than sweeping through obstacles.
        let tick = if self.map.reference_geometry.is_some() {
            1.0 / 60.0
        } else {
            STEP
        };
        let dt = (audio_time.min(self.playback_end()) - self.time).clamp(0.0, tick);
        if dt > 0.0 {
            if self.map.reference_geometry.is_some() {
                self.step(dt);
            } else {
                self.advance_exact(self.time + dt);
            }
        }
        // A disposable render sample may touch a hazard or the finish. Only
        // committed gameplay can publish terminal status to the host loop.
        #[cfg(not(target_arch = "wasm32"))]
        if let (Some(length),Some(events))=(audit_length,&mut self.contact_audit) { events.truncate(length); }
        (self.input_feedback,self.feedback_until)=saved_feedback;
        self.status = saved.12;
        self.death_reason = saved.13;
        let json = self.motion_json();
        (
            self.time,
            self.y,
            self.vy,
            self.jump_elapsed,
            self.support_y,
            self.grounded,
            self.held,
            self.jump_count,
            self.dash_until,
            self.dash_started,
            self.dash_carry_total,
            self.air_dash_used,
            self.status,
            self.death_reason,
            self.captured_well,
        ) = saved;
        json
    }
}

#[wasm_bindgen]
pub struct Game {
    core: Runner,
}

#[wasm_bindgen]
impl Game {
    #[wasm_bindgen(constructor)]
    pub fn new(map_json: &str) -> Result<Game, JsValue> {
        Runner::new(map_json)
            .map(|core| Game { core })
            .map_err(|e| JsValue::from_str(&e))
    }
    pub fn advance(&mut self, audio_time: f64) -> String {
        self.core.advance(audio_time);
        self.core.json()
    }
    pub fn advance_motion(&mut self, audio_time: f64) -> String {
        self.core.advance(audio_time);
        self.core.motion_json_at(audio_time)
    }
    pub fn input(&mut self, action: &str, audio_time: f64) -> String {
        self.core.input(action, audio_time);
        self.core.json()
    }
    pub fn input_motion(&mut self, action: &str, audio_time: f64) -> String {
        self.core.input(action, audio_time);
        self.core.motion_json()
    }
    pub fn restart(&mut self) {
        self.core.restart();
    }
    pub fn snapshot(&self) -> String {
        self.core.json()
    }
    pub fn map_json(&self) -> String {
        serde_json::to_string(&self.core.map).expect("validated finite map")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn terrain_dash_handoff_has_one_launch_despite_roundoff() {
        let boundary=3.7500000001;
        let map=serde_json::json!({"id":"handoff","title":"Handoff","difficulty":2,"bpm":120,"duration":10.0,
            "grid_snap":true,"dash_carry":true,"beats":(0..20).map(|i|i as f64*0.5).collect::<Vec<_>>(),
            "level_segments":[
                {"type":"terrain_run","beat":2.0,"heights":[0],"rests":[0.75],"rest_types":["gap"],"continuation":true,"exit_beat":boundary},
                {"type":"dash_route","beat":boundary,"entry_height":0,"height":1,"dash_beat":boundary+0.5,"exit_beat":boundary+2.0,"dash_variant":"transfer"},
                {"type":"terrain_run","beat":boundary+2.0,"entry_height":1,"heights":[0,0],"rests":[0,0]}]});
        let runner=Runner::new(&map.to_string()).unwrap();
        let launches:Vec<_>=runner.actions.iter().filter(|a|(a.time-boundary*0.5).abs()<1e-6).collect();
        assert_eq!(launches.len(),1);assert_eq!(launches[0].action,"jump");
    }
    #[test]
    fn contact_trace_records_rebounds_and_preview_restores_feedback() {
        let mut runner=assist_fixture(0.0,false);
        runner.held=true;
        runner.advance_exact(0.399);
        runner.enable_contact_audit();
        runner.input_feedback="Too early";runner.feedback_until=1.0;
        let previous=runner.snapshot().jump_count;
        runner.motion_json_at(0.401);
        assert!(runner.contact_audit_events().is_empty());
        assert_eq!(runner.snapshot().jump_count,previous);
        assert_eq!(runner.snapshot().input_feedback,"Too early");
        runner.advance_exact(0.401);
        let events=runner.contact_audit_events();
        let landing=events.iter().find(|e|e.cause=="landing").unwrap();
        let launch=events.iter().find(|e|e.cause=="held_jump").unwrap();
        assert!((landing.time-0.4).abs()<1e-7);
        assert!((launch.time-landing.time).abs()<1e-9);
        assert_eq!(landing.support_y,0.0);
    }
    #[test]
    fn release_overhang_requires_a_timed_hop_on_unchanged_cues() {
        let map = serde_json::json!({"id":"roof-route","title":"Roof route","difficulty":3,"bpm":150,
            "duration":6.0,"grid_snap":true,"beats":(0..15).map(|i|i as f64*0.4).collect::<Vec<_>>(),
            "level_segments":[{"beat":2.0,"type":"terrain_run","heights":[0,0],
                "rests":[0.2,0.0],"rest_types":["spike","ceiling"],"spike_counts":[2,1],
                "gate_releases":[2.65,null],"release_roofs":[3,0]}]});
        let mut timed = Runner::new(&map.to_string()).unwrap();
        let mut held = timed.clone();
        for action in timed.actions.clone() {
            timed.input(&action.action, action.time);
            if action.action != "release" || (action.time - 1.06).abs() > 1e-7 {
                held.input(&action.action, action.time);
            }
        }
        timed.advance(6.0);
        held.advance(6.0);
        assert_eq!(timed.status, "won");
        assert_eq!(held.status, "dead");
        assert!(timed
            .objects
            .iter()
            .filter(|o| o.kind == "ceiling")
            .all(|o| o.x % SIZE == 0.0 && o.y % SIZE == 0.0 && o.w == SIZE && o.h == SIZE));
    }
    #[test]
    fn authored_spike_offsets_move_the_whole_supported_group() {
        let mut map = serde_json::json!({"id":"offset-route","title":"Offset route","difficulty":3,"bpm":150,
            "duration":6.0,"beats":(0..15).map(|i|i as f64*0.4).collect::<Vec<_>>(),
            "level_segments":[{"beat":2.0,"type":"terrain_run","heights":[0,0],
                "rests":[0.5,0.0],"rest_types":["spike","ceiling"],"spike_counts":[2,1]}]});
        let normal = Runner::new(&map.to_string()).unwrap();
        map["level_segments"][0]["spike_offsets"] = serde_json::json!([-1, 0]);
        let shifted = Runner::new(&map.to_string()).unwrap();
        assert_eq!(normal.actions.len(), shifted.actions.len());
        for (a, b) in normal.actions.iter().zip(&shifted.actions) {
            assert_eq!(a.time, b.time);
            assert_eq!(a.action, b.action);
        }
        let normal_group: Vec<_> = normal
            .objects
            .iter()
            .filter(|o| o.kind == "spike" || o.kind == "enemy" && (o.w - 2.0 * SIZE).abs() < 1e-7)
            .collect();
        let shifted_group: Vec<_> = shifted
            .objects
            .iter()
            .filter(|o| o.kind == "spike" || o.kind == "enemy" && (o.w - 2.0 * SIZE).abs() < 1e-7)
            .collect();
        assert_eq!(normal_group.len(), 3);
        assert_eq!(shifted_group.len(), 3);
        for (a, b) in normal_group.iter().zip(shifted_group) {
            assert!((a.x - b.x - SIZE).abs() < 1e-7);
            assert_eq!((a.y, a.w, a.h), (b.y, b.w, b.h));
        }
        map["level_segments"][0]["spike_offsets"] = serde_json::json!([-1]);
        assert!(Runner::new(&map.to_string()).is_err());
        map["level_segments"][0]["spike_offsets"] = serde_json::json!([-2, 0]);
        assert!(Runner::new(&map.to_string()).is_err());
    }
    #[test]
    fn targeted_well_has_a_visible_continuous_arc_without_horizontal_drift() {
        let mut assisted = assist_fixture(28.0, false);
        let mut plain = assist_fixture(28.0, false);
        assisted.map.gravity_wells = vec![GravityWell {
            cue: 0.08,
            x: 0.0,
            y: 0.0,
            w: 224.0,
            h: 224.0,
            impulse: 0.0,
            target_x: Some(200.0),
            target_y: Some(84.0),
            strength: 2.0,
            duration: 0.6,
            start_y: None,
            start_vy: None,
            arrival_time: None,
            dash_cue: None,
            capture: None,
        }];
        let mut frames = assist_fixture(28.0, false);
        frames.map.gravity_wells = assisted.map.gravity_wells.clone();
        assisted.advance(0.3);
        for tick in 3..=72 {
            frames.advance(tick as f64 * STEP);
        }
        assert_eq!(assisted.json(), frames.json());
        plain.advance(0.3);
        assert!((assisted.x() - plain.x()).abs() < 1e-7);
        assert!(assisted.y - plain.y > SIZE);
        let first_difference = assisted.vy - plain.vy;
        assisted.advance(0.35);
        plain.advance(0.35);
        assert!((assisted.vy - plain.vy - first_difference).abs() > 1.0);
    }

    #[test]
    fn gravity_route_requires_assist_and_serialization_does_not_duplicate_wells() {
        let map = serde_json::json!({"id":"well-route","title":"Well route","difficulty":2,"bpm":150,
            "duration":6.0,"grid_snap":true,"beats":(0..15).map(|i|i as f64*0.4).collect::<Vec<_>>(),
            "level_segments":[{"beat":2.0,"type":"gravity_route","cue_beat":2.25,"exit_beat":3.6,"entry_height":0,"height":3,"continuation":true,"well_strength":2.0},
                {"beat":3.6,"type":"terrain_run","entry_height":3,"heights":[0,0]}]});
        let mut assisted = Runner::new(&map.to_string()).unwrap();
        let resolved = serde_json::to_string(&assisted.map).unwrap();
        let mut omitted = Runner::new(&resolved).unwrap();
        assert_eq!(omitted.map.gravity_wells.len(), 1);
        let cue = omitted.map.gravity_wells[0].cue;
        #[cfg(not(target_arch = "wasm32"))]
        assert!(omitted.disable_gravity_well(cue));
        #[cfg(target_arch = "wasm32")]
        omitted.map.gravity_wells.clear();
        for action in assisted.actions.clone() {
            assisted.input(&action.action, action.time);
            omitted.input(&action.action, action.time);
        }
        assisted.advance(6.0);
        omitted.advance(6.0);
        assert_eq!(assisted.status, "won");
        assert_eq!(omitted.status, "dead");
    }
    #[test]
    fn render_preview_cannot_publish_a_disposable_death() {
        let mut runner = assist_fixture(28.0, false);
        runner.restart();
        runner.status = "running";
        runner.time = 0.1;
        let spike_x = runner.x() + SIZE / 2.0 + 0.1;
        runner.objects = vec![
            Object {
                kind: "terrain".into(),
                x: 0.0,
                y: 0.0,
                w: 1000.0,
                h: 0.0,
                cue: None,
            },
            Object {
                kind: "spike".into(),
                x: spike_x,
                y: 0.0,
                w: SIZE,
                h: SIZE,
                cue: None,
            },
        ];
        runner.reindex();
        let committed = runner.json();
        let rendered: serde_json::Value =
            serde_json::from_str(&runner.motion_json_at(0.101)).unwrap();
        assert_eq!(rendered["status"], "running");
        assert_eq!(rendered["death_reason"], "");
        assert_eq!(runner.json(), committed);
        runner.advance(0.1 + STEP);
        assert_eq!(runner.status, "dead");
        assert_eq!(runner.death_reason, "spike");
    }
    #[test]
    fn gravity_routes_can_raise_four_cells_or_bridge_a_long_flat_gap() {
        for (height, exit) in [(4, 4.0), (0, 4.2)] {
            let map = serde_json::json!({"id":"well-variants","title":"Well variants","difficulty":2,"bpm":150,
                "duration":6.0,"grid_snap":true,"beats":(0..15).map(|i|i as f64*0.4).collect::<Vec<_>>(),
                "level_segments":[{"beat":2.0,"type":"gravity_route","cue_beat":2.25,"exit_beat":exit,"entry_height":0,"height":height,"continuation":true,"well_strength":2.0},
                    {"beat":exit,"type":"terrain_run","entry_height":height,"heights":[0,0]}]});
            let mut assisted = Runner::new(&map.to_string()).unwrap();
            let mut omitted = Runner::new(&map.to_string()).unwrap();
            omitted.map.gravity_wells.clear();
            for action in assisted.actions.clone() {
                assisted.input(&action.action, action.time);
                omitted.input(&action.action, action.time);
            }
            assisted.advance(6.0);
            omitted.advance(6.0);
            assert_eq!(
                assisted.status, "won",
                "height{height} exit{exit} {} at{}",
                assisted.death_reason, assisted.time
            );
            assert_eq!(omitted.status, "dead");
        }
    }
    #[test]
    fn gravity_route_can_arc_down_across_a_long_gap() {
        let map = serde_json::json!({"id":"well-descent","title":"Well descent","difficulty":2,"bpm":150,
            "duration":6.0,"grid_snap":true,"beats":(0..15).map(|i|i as f64*0.4).collect::<Vec<_>>(),
            "level_segments":[{"beat":2.0,"type":"gravity_route","cue_beat":2.25,"exit_beat":3.6,"entry_height":0,"height":3,"continuation":true,"well_strength":2.0},
                {"beat":3.6,"type":"gravity_route","cue_beat":3.85,"exit_beat":6.0,"entry_height":3,"height":1,"continuation":true,"well_strength":2.0},
                {"beat":6.0,"type":"terrain_run","entry_height":1,"heights":[0,0]}]});
        let mut assisted = Runner::new(&map.to_string()).unwrap();
        let mut omitted = Runner::new(&map.to_string()).unwrap();
        omitted.map.gravity_wells.remove(1);
        for action in assisted.actions.clone() {
            assisted.input(&action.action, action.time);
            omitted.input(&action.action, action.time);
        }
        assisted.advance(6.0);
        omitted.advance(6.0);
        assert_eq!(
            assisted.status, "won",
            "{} at{}",
            assisted.death_reason, assisted.time
        );
        assert_eq!(omitted.status, "dead");
    }
    fn assist_fixture(field_x: f64, with_well: bool) -> Runner {
        let wells = if with_well {
            serde_json::json!([{"cue":0.137,"x":field_x,"y":0.0,"w":84.0,"h":112.0,"impulse":0.3}])
        } else {
            serde_json::json!([])
        };
        let map = serde_json::json!({"id":"assist","title":"Assist","difficulty":1,"bpm":150,
            "duration":6.0,"beats":(0..15).map(|i|i as f64*0.4).collect::<Vec<_>>(),"gravity_wells":wells});
        let mut runner = Runner::new(&map.to_string()).unwrap();
        runner
            .objects
            .retain(|o| matches!(o.kind.as_str(), "tower" | "gravitywell"));
        runner.objects.push(Object {
            kind: "terrain".into(),
            x: -SIZE,
            y: 0.0,
            w: runner.route_finish + 8.0 * SIZE,
            h: 0.0,
            cue: None,
        });
        runner.reindex();
        runner.input("down", 0.0);
        runner.input("up", 0.01);
        runner
    }
    fn capture_fixture(direction: f64) -> Runner {
        let mut runner = assist_fixture(0.0, false);
        runner.map.dash_carry = true;
        runner.input("down", 0.2);
        let mut nominal = runner.clone();
        nominal.advance_exact(0.4);
        runner.map.gravity_wells = vec![GravityWell {
            cue: 0.4,
            dash_cue: Some(0.2),
            x: 0.0,
            y: 0.0,
            w: 560.0,
            h: 280.0,
            impulse: 0.0,
            target_x: None,
            target_y: None,
            strength: 0.0,
            duration: 0.4,
            start_y: None,
            start_vy: None,
            arrival_time: None,
            capture: Some(SlingshotCapture {
                x: nominal.x(),
                y: nominal.y + SIZE / 2.0,
                radius: SIZE * 0.75,
                redirect_vy: direction * 1.5 * runner.jump,
                landing_time: 0.8,
            }),
        }];
        runner
    }
    #[test]
    fn dash_readiness_requires_release_and_reports_actual_rejection() {
        let mut runner = assist_fixture(0.0, false);
        runner.map.dash_carry = true;
        runner.held = true;
        runner.input("down", 0.10);
        assert_eq!(runner.snapshot().input_feedback, "Release first");
        assert!(!runner.snapshot().dash_ready);
        runner.input("up", 0.10);
        runner.input("down", 0.11);
        assert_eq!(runner.snapshot().input_feedback, "Too early");
        runner.input("up", 0.11);
        runner.input("down", 0.2);
        assert!(runner.snapshot().dash_active);
        assert_eq!(runner.snapshot().input_feedback, "");
        runner.input("up", 0.21);
        runner.input("down", 0.22);
        assert_eq!(runner.snapshot().input_feedback, "Dash spent");
    }

    #[test]
    fn capture_redirects_once_at_dash_end_then_uses_normal_gravity() {
        for direction in [-1.0, 1.0] {
            let mut runner = capture_fixture(direction);
            runner.advance_exact(0.39);
            assert_eq!(runner.captured_well, None);
            let foot = runner.y;
            runner.advance_exact(0.4);
            assert_eq!(runner.captured_well, Some(0));
            assert!((runner.y - foot).abs() < 1e-7, "Capture must not teleport");
            assert!((runner.vy - direction * 1.5 * runner.jump).abs() < 1e-7);
            let velocity = runner.vy;
            runner.advance_exact(0.41);
            assert!(
                (runner.vy - (velocity - runner.gravity * 0.01)).abs() < 1e-6,
                "After capture there is ordinary gravity, not a sustained pull"
            );
        }
        let mut missed = capture_fixture(1.0);
        missed.map.gravity_wells[0].capture.as_mut().unwrap().x += SIZE * 4.0;
        missed.advance_exact(0.41);
        assert_eq!(
            missed.captured_well, None,
            "A remote well cannot catch the player"
        );
    }
    #[test]
    fn capture_preview_rolls_back_and_replay_is_frame_independent() {
        let mut preview = capture_fixture(1.0);
        preview.advance_exact(0.398);
        let before = preview.json();
        let _: serde_json::Value = serde_json::from_str(&preview.motion_json_at(0.401)).unwrap();
        assert_eq!(preview.captured_well, None);
        assert_eq!(preview.json(), before);
        for direction in [-1.0, 1.0] {
            let mut states = Vec::new();
            for fps in [60.0, 144.0, 240.0] {
                let mut runner = capture_fixture(direction);
                for tick in 1..=(0.46 * fps) as usize {
                    runner.advance(tick as f64 / fps);
                }
                runner.advance(0.46);
                states.push(runner.json());
            }
            assert!(states.windows(2).all(|pair| pair[0] == pair[1]));
        }
    }
    #[test]
    fn well_rearms_one_dash_on_the_new_arc_without_allowing_immediate_spam() {
        let mut runner = capture_fixture(1.0);
        runner.advance_exact(0.4);
        assert!(!runner.air_dash_used, "The physical catch rearms the dash");
        assert!(
            !runner.dash_available(),
            "A new upward arc has not reached its peak yet"
        );
        let peak = 0.4 + 1.5 * runner.jump / runner.gravity;
        runner.advance_exact(peak);
        assert!(runner.dash_available());
        runner.input("up", peak);
        runner.input("down", peak);
        assert!(runner.air_dash_used);
        runner.advance_exact(peak + 0.5 * runner.beat_period);
        assert!(
            !runner.dash_available(),
            "One well grants one additional dash"
        );
    }
    #[test]
    fn chain_spotlight_is_a_timed_tower_attack_not_a_permanent_wall() {
        let mut plain = assist_fixture(0.0, false);
        plain.objects.push(Object {
            kind: "chain_beam".into(),
            x: 0.0,
            y: 0.0,
            w: 560.0,
            h: 112.0,
            cue: Some(0.2),
        });
        plain.reindex();
        let mut dashed = plain.clone();
        plain.advance_exact(0.205);
        assert_eq!(
            plain.status, "running",
            "The warning beam must not kill before the shot"
        );
        plain.advance_exact(0.214);
        assert_eq!(plain.death_reason, "ultimate weapon");
        dashed.input("down", 0.2);
        dashed.advance_exact(0.214);
        assert_eq!(
            dashed.status, "running",
            "A real on-cue dash crosses the fired spotlight"
        );
        let mut expired = assist_fixture(0.0, false);
        expired.advance_exact(0.25);
        expired.objects.push(Object {
            kind: "chain_beam".into(),
            x: 0.0,
            y: 0.0,
            w: 560.0,
            h: 112.0,
            cue: Some(0.05),
        });
        expired.reindex();
        expired.advance_exact(0.3);
        assert_eq!(
            expired.status, "running",
            "Expired shots leave no invisible wall"
        );
    }
    #[test]
    fn slingshot_requires_a_real_dash_and_keeps_its_half_beat_hang() {
        let mut armed = assist_fixture(0.0, false);
        let mut unarmed = armed.clone();
        let mut plain = armed.clone();
        let well = GravityWell {
            cue: 0.4,
            dash_cue: Some(0.2),
            x: 0.0,
            y: 0.0,
            w: 560.0,
            h: 280.0,
            impulse: 0.0,
            target_x: Some(300.0),
            target_y: Some(84.0),
            strength: 3.0,
            duration: 0.6,
            start_y: None,
            start_vy: None,
            arrival_time: None,
            capture: None,
        };
        armed.map.gravity_wells = vec![well.clone()];
        unarmed.map.gravity_wells = vec![well];
        armed.input("down", 0.2);
        plain.input("down", 0.2);
        armed.advance(0.4);
        plain.advance(0.4);
        assert!(
            (armed.y - plain.y).abs() < 1e-7,
            "Pull must not interrupt normal dash hang"
        );
        armed.advance(0.5);
        plain.advance(0.5);
        assert!(
            armed.y > plain.y + 2.0,
            "Real dash must arm an actual curved assist"
        );
        assert!(
            (armed.x() - plain.x()).abs() < 1e-7,
            "Assist must not alter the musical X clock"
        );
        let mut untouched = assist_fixture(0.0, false);
        unarmed.advance(0.5);
        untouched.advance(0.5);
        assert!(
            (unarmed.y - untouched.y).abs() < 1e-7,
            "Well alone must stay inactive"
        );
    }
    #[test]
    fn gravity_route_handoff_uses_one_real_press_after_a_held_walk() {
        let map = serde_json::json!({"id":"well-handoff","title":"Well handoff","difficulty":2,"bpm":150,
            "duration":6.0,"grid_snap":true,"beats":(0..15).map(|i|i as f64*0.4).collect::<Vec<_>>(),
            "level_segments":[{"beat":2.0,"type":"terrain_run","entry_height":0,"heights":[0],"rests":[1.0],"continuation":true,"exit_beat":4.0},
                {"beat":4.0,"type":"gravity_route","cue_beat":4.25,"exit_beat":5.6,"entry_height":0,"height":3,"continuation":true,"well_strength":2.0},
                {"beat":5.6,"type":"terrain_run","entry_height":3,"heights":[0,0]}]});
        let mut runner = Runner::new(&map.to_string()).unwrap();
        let mut edges = Vec::new();
        for action in &runner.actions {
            edges.push(Action {
                time: action.time,
                action: if action.action == "release" {
                    "up"
                } else {
                    "down"
                }
                .into(),
            });
            if action.action == "jump" {
                edges.push(Action {
                    time: action.time + 0.12 * runner.beat_period,
                    action: "up".into(),
                });
            }
        }
        edges.sort_by(|a, b| {
            a.time
                .total_cmp(&b.time)
                .then_with(|| b.action.cmp(&a.action))
        });
        let mut held = false;
        edges.retain(|a| {
            let next = a.action == "down";
            if held == next {
                false
            } else {
                held = next;
                true
            }
        });
        assert_eq!(
            edges
                .iter()
                .filter(|a| a.action == "down" && (a.time - 1.6).abs() < 1e-7)
                .count(),
            1
        );
        for edge in &edges {
            runner.input(&edge.action, edge.time);
        }
        runner.advance(6.0);
        assert_eq!(
            runner.status, "won",
            "{} at{}",
            runner.death_reason, runner.time
        );
    }
    #[test]
    fn gravity_well_applies_one_real_impulse_only_inside_its_field() {
        let mut assisted = assist_fixture(28.0, true);
        let mut plain = assist_fixture(28.0, false);
        let mut outside = assist_fixture(560.0, true);
        for runner in [&mut assisted, &mut plain, &mut outside] {
            runner.input("up", 0.137);
        }
        assert!((assisted.vy - plain.vy - 0.3 * assisted.jump).abs() < 1e-7);
        assert!((outside.vy - plain.vy).abs() < 1e-7);
        for runner in [&mut assisted, &mut plain, &mut outside] {
            runner.input("up", 0.3);
        }
        assert!((assisted.vy - plain.vy - 0.3 * assisted.jump).abs() < 1e-7);
        assert_eq!(assisted.status, "running");
    }
    #[test]
    fn gravity_well_clock_split_is_frame_deterministic_and_preview_is_reversible() {
        let mut batched = assist_fixture(28.0, true);
        let mut frames = assist_fixture(28.0, true);
        batched.advance(0.3);
        for tick in 3..=72 {
            frames.advance(tick as f64 * STEP);
        }
        assert_eq!(batched.json(), frames.json());
        let mut preview = assist_fixture(28.0, true);
        preview.advance(0.136);
        let committed = preview.json();
        let rendered: serde_json::Value =
            serde_json::from_str(&preview.motion_json_at(0.138)).unwrap();
        assert!(rendered["vy"].as_f64().unwrap() > preview.vy);
        assert_eq!(preview.json(), committed);
    }
    #[test]
    fn gravity_well_does_not_cancel_an_active_dash() {
        let mut assisted = assist_fixture(28.0, true);
        let mut plain = assist_fixture(28.0, false);
        for runner in [&mut assisted, &mut plain] {
            runner.dash_started = 0.1;
            runner.dash_until = 0.2;
            runner.input("up", 0.137);
        }
        assert!((assisted.vy - plain.vy).abs() < 1e-7);
        assert!((assisted.y - plain.y).abs() < 1e-7);
    }
    #[test]
    fn render_preview_resolves_landings_without_mutating_gameplay() {
        let map = serde_json::json!({"id":"preview","title":"Preview","difficulty":1,"bpm":150,
            "duration":6.0,"beats":(0..15).map(|i|i as f64*0.4).collect::<Vec<_>>()});
        let mut runner = Runner::new(&map.to_string()).unwrap();
        runner.objects = vec![Object {
            kind: "enemy".into(),
            x: 0.0,
            y: 0.0,
            w: 10.0 * SIZE,
            h: SIZE,
            cue: None,
        }];
        runner.reindex();
        runner.status = "running";
        runner.time = 0.1;
        runner.y = SIZE + 0.05;
        runner.vy = -100.0;
        runner.grounded = false;
        let committed = runner.json();
        let rendered: serde_json::Value =
            serde_json::from_str(&runner.motion_json_at(0.102)).unwrap();
        assert_eq!(rendered["y"].as_f64().unwrap(), SIZE);
        assert_eq!(rendered["grounded"], true);
        assert_eq!(runner.json(), committed);
        runner.held = true;
        let committed = runner.json();
        let rendered: serde_json::Value =
            serde_json::from_str(&runner.motion_json_at(0.102)).unwrap();
        assert!(rendered["y"].as_f64().unwrap() > SIZE);
        assert_eq!(rendered["jump_count"].as_u64().unwrap(), 1);
        assert_eq!(runner.json(), committed);
    }

    #[test]
    fn triangle_contacts_match_full_unrotated_cube_body() {
        let spike = Object {
            kind: "spike".into(),
            x: 0.0,
            y: 0.0,
            w: SIZE,
            h: SIZE,
            cue: None,
        };
        assert!(Runner::triangle_hits_player(
            &spike,
            SIZE / 2.0,
            SIZE - 0.001
        ));
        assert!(!Runner::triangle_hits_player(&spike, SIZE / 2.0, SIZE));
        assert!(Runner::triangle_hits_player(
            &spike,
            -SIZE / 2.0 + 0.01,
            0.0
        ));
        assert!(!Runner::triangle_hits_player(&spike, -SIZE / 2.0, 0.0));
    }

    #[test]
    fn custom_finish_requires_contact_with_actual_tower_body() {
        let map = serde_json::json!({"id":"contact","title":"Contact","difficulty":1,"bpm":150,
            "duration":6.0,"beats":(0..15).map(|i|i as f64*0.4).collect::<Vec<_>>()});
        let mut runner = Runner::new(&map.to_string()).unwrap();
        runner.objects.retain(|o| o.kind == "tower");
        runner.objects.push(Object {
            kind: "terrain".into(),
            x: -SIZE,
            y: 0.0,
            w: runner.route_finish + 8.0 * SIZE,
            h: 0.0,
            cue: None,
        });
        // A moved tower cannot be won by reaching the timer alone.
        runner
            .objects
            .iter_mut()
            .find(|o| o.kind == "tower")
            .unwrap()
            .x += SIZE;
        runner.reindex();
        runner.advance(6.0);
        assert_ne!(runner.status, "won");
        runner.restart();
        runner
            .objects
            .iter_mut()
            .find(|o| o.kind == "tower")
            .unwrap()
            .x -= SIZE;
        runner.reindex();
        runner.advance(6.0);
        assert_eq!(runner.status, "won");
    }

    #[test]
    #[cfg(not(target_arch = "wasm32"))]
    fn autoplay_centering_uses_real_press_edges_and_complete_replay() {
        let map = serde_json::json!({"id":"center","title":"Center","difficulty":1,"bpm":150,
            "duration":6.0,"grid_snap":true,"beats":(0..15).map(|i|i as f64*0.4).collect::<Vec<_>>(),
            "level_segments":[{"beat":0.0,"type":"jump_spike","count":1} ]});
        let mut runner = Runner::new(&map.to_string()).unwrap();
        runner.objects.retain(|o| o.kind == "tower");
        runner.objects.push(Object {
            kind: "terrain".into(),
            x: -SIZE,
            y: 0.0,
            w: runner.route_finish + 8.0 * SIZE,
            h: 0.0,
            cue: None,
        });
        runner.reindex();
        runner.actions = vec![
            Action {
                time: 0.0,
                action: "down".into(),
            },
            Action {
                time: 0.05,
                action: "up".into(),
            },
        ];
        let controls = runner.center_grid_controls().unwrap();
        assert!(controls[0].time > 0.0 && controls[0].time < 0.03);
        assert_eq!(controls[1].time, 0.05);
        assert_eq!(runner.status, "won");
        assert_eq!(runner.jump_count, 1);
        runner.actions = vec![
            Action {
                time: 0.079,
                action: "down".into(),
            },
            Action {
                time: 0.13,
                action: "up".into(),
            },
        ];
        let bounded = runner.center_grid_controls().unwrap();
        assert!(bounded[0].time <= 0.08 + 1e-9);
        assert_eq!(runner.status, "won");
    }
    fn manifest() -> Vec<MapInput> {
        let path = std::env::var("TOWER_TEST_MANIFEST")
            .map(std::path::PathBuf::from)
            .unwrap_or_else(|_| {
                std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../public/music/tracks.json")
            });
        serde_json::from_str(&std::fs::read_to_string(path).unwrap()).unwrap()
    }
    fn game(map: &MapInput) -> Runner {
        let mut runner = Runner::new(&serde_json::to_string(map).unwrap()).unwrap();
        // Role-labelled cues support targeted omission/window checks. The
        // separate frame replay test exercises the actual physical edges.
        if map.grid_snap && !map.grid_cues.is_empty() {
            runner.actions = map.grid_cues.clone();
        }
        runner
    }
    fn cue_index(runner: &Runner, role: &str, time: f64) -> usize {
        let (index, cue) = runner
            .actions
            .iter()
            .enumerate()
            .filter(|(_, a)| a.action == role)
            .min_by(|(_, a), (_, b)| (a.time - time).abs().total_cmp(&(b.time - time).abs()))
            .unwrap();
        assert!(
            (cue.time - time).abs() <= 0.081,
            "Missing {role} cue near {time}"
        );
        index
    }

    #[test]
    fn landing_shelves_do_not_recreate_long_safe_walks() {
        for map in manifest() {
            let route = game(&map);
            for shelf in route.objects.iter().filter(|object| object.kind == "enemy") {
                assert!(
                    shelf.w <= 12.0 * SIZE + 1e-6,
                    "{} has a {:.3}-tile safe shelf at {:.3}s",
                    map.id,
                    shelf.w / SIZE,
                    shelf.x / route.speed
                );
            }
        }
    }
    #[test]
    fn direct_reference_keeps_uniform_integer_tiles_and_disables_dash() {
        let json = serde_json::json!({"id":"reference","title":"Reference","difficulty":4,"bpm":150,
            "duration":(60.0-0.5)/12.0,"beats":[0.0,0.4,0.8,1.2,1.6,2.0,2.4,2.8,3.2,3.6,4.0,4.4,4.8],
            "reference_geometry":{"end_tile":60,"objects":[{"kind":"block","x":6,"y":1},{"kind":"spike","x":4,"y":0}],"pits":[{"start":8,"end":10}]}});
        let mut runner = Runner::new(&json.to_string()).unwrap();
        assert!(runner.actions.is_empty());
        let block = runner.objects.iter().find(|o| o.kind == "enemy").unwrap();
        assert_eq!(
            (block.x, block.y, block.w, block.h),
            (6.0 * SIZE, SIZE, SIZE, SIZE)
        );
        assert!(!runner
            .objects
            .iter()
            .any(|o| o.kind == "terrain" && (8.0 * SIZE..10.0 * SIZE).contains(&o.x)));
        runner.input("down", 0.0);
        runner.input("up", 0.05);
        runner.input("down", 0.2);
        assert!(!runner.air_dash_used);
        let mut with_events = json.clone();
        with_events["reference_geometry"]["objects"] = serde_json::json!([]);
        with_events["reference_geometry"]["pits"] = serde_json::json!([]);
        with_events["reference_events"] = serde_json::json!({"gravity_tiles":[2,4],"rising_tiles":[[5,8]],"falling_tiles":[[10,15]]});
        let mut effects = Runner::new(&with_events.to_string()).unwrap();
        assert!(!effects.snapshot().inverted);
        effects.advance(0.2);
        assert!(effects.snapshot().inverted);
        effects.advance(0.4);
        assert!(!effects.snapshot().inverted);
        with_events["reference_events"]["falling_tiles"] = serde_json::json!([[15, 10]]);
        assert!(Runner::new(&with_events.to_string()).is_err());
        let mut invalid = json.clone();
        invalid["reference_geometry"]["objects"][0]["x"] = serde_json::json!(-1);
        assert!(Runner::new(&invalid.to_string()).is_err());
    }
    #[test]
    fn reference_replay_is_independent_of_advance_batch_size() {
        let map = serde_json::json!({"id":"clock","title":"Clock","difficulty":1,"bpm":150,
            "duration":(60.0-0.5)/12.0,"beats":[0.0,0.4,0.8,1.2,1.6,2.0,2.4,2.8,3.2,3.6,4.0,4.4,4.8],
            "reference_geometry":{"end_tile":60,"objects":[],"pits":[]}}).to_string();
        let mut batched = Runner::new(&map).unwrap();
        let mut ticked = Runner::new(&map).unwrap();
        batched.input("hold", 0.0);
        ticked.input("hold", 0.0);
        batched.advance(3.6);
        for tick in 1..=864 {
            ticked.advance(tick as f64 * STEP);
        }
        assert_eq!(batched.status, ticked.status);
        assert_eq!(batched.jump_count, ticked.jump_count);
        assert!((batched.y - ticked.y).abs() < 1e-7);
        assert!((batched.vy - ticked.vy).abs() < 1e-7);
    }
    #[test]
    #[cfg(not(target_arch = "wasm32"))]
    fn reference_autoplay_does_not_hop_on_empty_ground() {
        let map = serde_json::json!({"id":"quiet","title":"Quiet","difficulty":1,"bpm":150,
            "duration":(60.0-0.5)/12.0,"beats":[0.0,0.4,0.8,1.2,1.6,2.0,2.4,2.8,3.2,3.6,4.0,4.4,4.8],
            "reference_geometry":{"end_tile":60,"objects":[],"pits":[]}}).to_string();
        let mut runner = Runner::new(&map).unwrap();
        assert!(runner.solve_reference_controls(4096).unwrap().is_empty());
        assert_eq!(runner.jump_count, 0);
        assert_eq!(runner.status, "won");
        let mut outro: serde_json::Value = serde_json::from_str(&map).unwrap();
        outro["reference_outro_seconds"] = serde_json::json!(2.0);
        let mut automatic = Runner::new(&outro.to_string()).unwrap();
        automatic.advance(automatic.map.duration);
        assert_eq!(automatic.status, "running");
        automatic.advance(automatic.playback_end());
        assert_eq!(automatic.status, "won");
        assert!(automatic.jump_count > 0);
    }
    #[test]
    fn carried_dash_has_no_slow_recovery_tail() {
        let map = serde_json::json!({"id":"carry","title":"Carry","difficulty":2,"bpm":150,"duration":6.0,
            "beats":(0..15).map(|i|i as f64*0.4).collect::<Vec<_>>(),"dash_carry":true,
            "level_segments":[{"beat":2.0,"type":"dash_route","dash_beat":2.5,"exit_beat":4.0,"height":0,"dash_variant":"gap"},
                {"beat":4.0,"type":"terrain_run","heights":[0,0],"rests":[0,0]}]}).to_string();
        let mut runner = Runner::new(&map).unwrap();
        runner.input("down", 0.8);
        runner.input("up", 0.85);
        runner.input("down", 1.0);
        runner.input("up", 1.05);
        let mut previous = runner.x();
        let mut previous_time = runner.time;
        for tick in 253..=360 {
            runner.advance(tick as f64 * STEP);
            assert_eq!(runner.status, "running");
            assert!(runner.x() - previous >= runner.speed * (runner.time - previous_time) - 1e-7);
            previous = runner.x();
            previous_time = runner.time;
        }
        assert!((runner.dash_offset() - 3.0 * SIZE).abs() < 1e-7);
    }
    #[test]
    fn enemy_bodies_do_not_overlap_triangle_faces() {
        for map in manifest() {
            let route = game(&map);
            for spike in route.objects.iter().filter(|o| o.kind == "spike") {
                for block in route
                    .objects
                    .iter()
                    .filter(|o| matches!(o.kind.as_str(), "enemy" | "ceiling"))
                {
                    let left = spike.x.max(block.x);
                    let right = (spike.x + spike.w).min(block.x + block.w);
                    if left >= right - 1e-7 || block.y + block.h <= spike.y + 1e-7 {
                        continue;
                    }
                    let apex = spike.x + spike.w / 2.0;
                    let nearest = apex.clamp(left, right);
                    let top = spike.y + spike.h * (1.0 - (nearest - apex).abs() / (spike.w / 2.0));
                    assert!(
                        block.y >= top - 1e-7,
                        "{} block x{:.3} y{:.3} overlaps triangle x{:.3} y{:.3}",
                        map.id,
                        block.x / SIZE,
                        block.y / SIZE,
                        spike.x / SIZE,
                        spike.y / SIZE
                    );
                }
            }
        }
    }
    #[test]
    fn phrase_local_press_windows_are_measured() {
        let mut medians = Vec::new();
        for map in manifest() {
            let baseline = game(&map);
            let mut widths = Vec::new();
            for (index, action) in baseline.actions.iter().enumerate() {
                if action.action != "hold"
                    || action.time >= 100.0
                    || index == 0
                    || baseline.actions[index - 1].action != "release"
                {
                    continue;
                }
                let checkpoint = baseline
                    .actions
                    .iter()
                    .skip(index + 1)
                    .filter(|a| a.action == "hold" || a.action == "dash")
                    .nth(1)
                    .map(|a| a.time - 0.01)
                    .unwrap_or(map.duration);
                let mut surviving = Vec::new();
                let mut prefix = baseline.clone();
                for a in &baseline.actions[..index] {
                    prefix.input(&a.action, a.time);
                }
                for milliseconds in (-80..=80).step_by(4) {
                    let changed = action.time + milliseconds as f64 / 1000.0;
                    if changed <= baseline.actions[index - 1].time {
                        continue;
                    }
                    let mut replay = prefix.clone();
                    replay.input("hold", changed);
                    for a in baseline
                        .actions
                        .iter()
                        .skip(index + 1)
                        .take_while(|a| a.time < checkpoint)
                    {
                        replay.input(&a.action, a.time);
                    }
                    replay.advance(checkpoint);
                    if replay.status != "dead" {
                        surviving.push(milliseconds);
                    }
                }
                assert!(
                    surviving.contains(&0),
                    "{} nominal press at {:.6} fails its local phrase",
                    map.id,
                    action.time
                );
                let mut early = 0;
                let mut late = 0;
                while surviving.contains(&(early - 4)) {
                    early -= 4;
                }
                while surviving.contains(&(late + 4)) {
                    late += 4;
                }
                widths.push(late - early);
            }
            widths.sort();
            assert!(!widths.is_empty());
            medians.push(widths[widths.len() / 2]);
            eprintln!("{} local press windows: {} samples, median {}ms, min {}ms, max {}ms (scan capped at +/-80ms)", map.id, widths.len(), widths[widths.len()/2], widths[0], widths.last().unwrap());
        }
        assert!(
            medians[2] <= medians[1],
            "Eye must demand at least Crystallize's measured press precision"
        );
        assert!(
            medians[3] < *medians[..3].iter().min().unwrap(),
            "Cliffs must have the tightest measured press windows"
        );
    }
    fn replay(g: &mut Runner, fps: f64) {
        let actions = g.actions.clone();
        let mut cursor = 0;
        for frame in 1..=((g.map.duration + 1.0) * fps).ceil() as usize {
            let t = frame as f64 / fps;
            while cursor < actions.len() && actions[cursor].time <= t {
                let a = &actions[cursor];
                g.input(&a.action, a.time);
                cursor += 1;
            }
            g.advance(t);
            if matches!(g.status, "won" | "dead") {
                break;
            }
        }
    }
    #[test]
    fn reference_proportions_and_two_minute_excerpts() {
        for m in manifest() {
            let g = game(&m);
            assert!((100.0..=210.0).contains(&m.duration));
            assert!((0.31..=0.43).contains(&g.flight));
            assert!((g.jump * g.jump / (2.0 * g.gravity) / SIZE - 2.02).abs() < 1e-9);
            assert!((g.speed * g.flight / SIZE - 4.8).abs() < 1e-9);
        }
    }
    #[test]
    fn raised_dash_handoffs_land_and_walk_before_a_fresh_jump() {
        let mut checked = 0;
        for m in manifest() {
            for route in m
                .level_segments
                .iter()
                .filter(|s| s.kind == "dash_route" && s.entry_height > 0)
            {
                let mut g = game(&m);
                let j = g.actions[cue_index(&g, "jump", Runner::beat_time(&m, route.beat))].time;
                let d = g.actions
                    [cue_index(&g, "dash", Runner::beat_time(&m, route.dash_beat.unwrap()))]
                .time;
                for a in g.actions.clone().iter().filter(|a| a.time < j - 1e-7) {
                    g.input(&a.action, a.time);
                }
                g.advance(j - 1e-5);
                assert_eq!(
                    g.status, "running",
                    "{} raised entry {}: {}",
                    m.id, j, g.death_reason
                );
                assert!(
                    g.grounded && (g.y - route.entry_height as f64 * SIZE).abs() < 0.01,
                    "{} unsupported raised launch at {}: y{} grounded{}",
                    m.id,
                    j,
                    g.y / SIZE,
                    g.grounded
                );
                let count = g.jump_count;
                g.input("down", j);
                assert_eq!(g.jump_count, count + 1);
                g.input("up", j + 0.12 * g.beat_period);
                g.input("down", d);
                assert!(
                    g.air_dash_used,
                    "{} apex dash at {} was not accepted",
                    m.id, d
                );
                checked += 1;
            }
        }
        assert!(
            checked >= 10,
            "not enough raised transitions were exercised: {checked}"
        );
    }
    #[test]
    fn dash_pockets_work_with_physical_button_edges_and_require_dash() {
        for m in manifest() {
            let baseline = game(&m);
            let dash_times: Vec<f64> = baseline
                .actions
                .iter()
                .filter(|a| a.action == "dash")
                .map(|a| a.time)
                .collect();
            assert!(
                dash_times.len() >= 6,
                "{} has only {} dash pockets",
                m.id,
                dash_times.len()
            );
            for omitted in std::iter::once(None).chain(dash_times.iter().map(|t| Some(*t))) {
                let mut g = game(&m);
                let mut edges = Vec::new();
                for a in g.actions.clone() {
                    match a.action.as_str() {
                        "jump" => {
                            edges.push(Action {
                                time: a.time,
                                action: "down".into(),
                            });
                            edges.push(Action {
                                time: a.time + g.beat_period * 0.12,
                                action: "up".into(),
                            });
                        }
                        "dash" if omitted != Some(a.time) => edges.push(Action {
                            time: a.time,
                            action: "down".into(),
                        }),
                        "dash" => {}
                        "hold" => edges.push(Action {
                            time: a.time,
                            action: "down".into(),
                        }),
                        "release" => edges.push(Action {
                            time: a.time,
                            action: "up".into(),
                        }),
                        _ => edges.push(a),
                    }
                }
                edges.sort_by(|a, b| a.time.total_cmp(&b.time));
                g.actions = edges;
                replay(&mut g, 60.0);
                assert_eq!(
                    g.status,
                    if omitted.is_none() { "won" } else { "dead" },
                    "{} omitted {:?} at {}: {}",
                    m.id,
                    omitted,
                    g.time,
                    g.death_reason
                );
            }
        }
    }
    #[test]
    fn all_actual_scores_survive_varied_frames() {
        for m in manifest() {
            for fps in [1.0, 15.0, 60.0, 120.0, 144.0, 240.0] {
                let mut g = Runner::new(&serde_json::to_string(&m).unwrap()).unwrap();
                replay(&mut g, fps);
                assert_eq!(
                    g.status,
                    "won",
                    "{} fps{} at{}: {} x{} y{} support{} jumps{} nearby{:?} recent{:?}",
                    m.id,
                    fps,
                    g.time,
                    g.death_reason,
                    g.x() / SIZE,
                    g.y / SIZE,
                    g.support_y / SIZE,
                    g.jump_count,
                    g.objects
                        .iter()
                        .filter(|o| o.x < g.x() + SIZE
                            && o.x + o.w > g.x() - SIZE
                            && o.kind != "terrain")
                        .map(|o| (&o.kind, o.x / SIZE, o.y / SIZE, o.w / SIZE))
                        .collect::<Vec<_>>(),
                    g.actions
                        .iter()
                        .filter(|a| a.time <= g.time)
                        .rev()
                        .take(5)
                        .collect::<Vec<_>>()
                );
            }
        }
    }
    #[test]
    fn hold_and_release_have_measured_leeway() {
        for m in manifest() {
            for error in [-0.012, 0.0, 0.012] {
                let mut g = game(&m);
                let hold = g.actions.iter().find(|a| a.action == "hold").unwrap().time;
                let release = g
                    .actions
                    .iter()
                    .find(|a| a.action == "release")
                    .unwrap()
                    .time;
                let prefix = g
                    .actions
                    .iter()
                    .take_while(|a| a.action != "hold")
                    .cloned()
                    .collect::<Vec<_>>();
                for a in prefix {
                    g.input(&a.action, a.time);
                }
                let before_count = g.jump_count;
                g.input("down", hold + error);
                g.input("down", hold + error + 0.1);
                assert!(!g.air_dash_used, "repeated down must not dash");
                g.input("up", release + error);
                let count = g.jump_count;
                assert!(count > before_count);
                // The opening is now an articulated musical phrase, rather
                // than a hard-coded four-hop hold. Observe its actual landing.
                let resume = g.actions.iter().find(|a| a.time > release).unwrap().time;
                g.advance(resume - 0.03);
                assert_eq!(
                    g.status, "running",
                    "{} error{} at{} {}",
                    m.id, error, g.time, g.death_reason
                );
                assert_eq!(g.jump_count, count);
                assert!(!g.held);
            }
        }
    }
    #[test]
    fn continuing_hold_hits_release_ceiling_and_no_input_dies() {
        for m in manifest() {
            let mut g = game(&m);
            let prefix = g
                .actions
                .iter()
                .take_while(|a| a.action != "hold")
                .cloned()
                .collect::<Vec<_>>();
            for a in prefix {
                g.input(&a.action, a.time);
            }
            let t = g.actions.iter().find(|a| a.action == "hold").unwrap().time;
            g.input("down", t);
            g.advance(t + 6.0 * g.beat_period);
            assert_eq!(g.status, "dead");
            let mut g = game(&m);
            g.advance(t + 6.0 * g.beat_period);
            assert_eq!(g.status, "dead");
        }
    }
    #[test]
    fn apex_fresh_press_dashes_three_tiles_and_freezes() {
        let mut g = game(&manifest()[0]);
        let peak = g.flight * 0.5;
        g.input("down", 0.0);
        g.input("up", 0.05);
        g.input("down", peak);
        assert!(g.air_dash_used);
        let y = g.y;
        let vy = g.vy;
        let phase = g.jump_elapsed;
        g.advance_exact(peak + g.beat_period * 0.25);
        assert!(
            (g.dash_offset()
                - if g.map.dash_carry {
                    1.5 * SIZE
                } else {
                    3.0 * SIZE
                })
            .abs()
                < 1e-7
        );
        assert_eq!(g.y, y);
        assert_eq!(g.vy, vy);
        assert_eq!(g.jump_elapsed, phase);
        g.input("up", g.time);
        g.advance_exact(peak + g.beat_period * 1.5);
        assert!((g.dash_offset() - if g.map.dash_carry { 3.0 * SIZE } else { 0.0 }).abs() < 1e-7);
        g.restart();
        assert!(!g.held);
        assert_eq!(g.jump_count, 0);
    }
    #[test]
    fn real_manual_triples_have_collision_windows_not_exact_frame_targets() {
        for m in manifest() {
            if m.id == "cliffs-of-dover" {
                continue;
            } // Timed endurance route uses singles/pairs.
            let baseline = game(&m);
            let triple_time = m
                .level_segments
                .iter()
                .find(|s| s.kind == "jump_spike" && s.count == 3)
                .map(|s| Runner::beat_time(&m, s.beat));
            let Some(triple_time) = triple_time else {
                continue;
            };
            let index = cue_index(&baseline, "jump", triple_time);
            let cue = baseline.actions[index].time;
            let mut successful = Vec::new();
            for ms in -120..=120 {
                let mut g = game(&m);
                for action in &baseline.actions[..index] {
                    g.input(&action.action, action.time);
                }
                g.input("press", cue + ms as f64 / 1000.0);
                g.advance(cue + baseline.beat_period * 1.1);
                if g.status == "running" {
                    successful.push(ms);
                }
            }
            println!(
                "{} actual triple window: {}..{} ms",
                m.id,
                successful.first().unwrap(),
                successful.last().unwrap()
            );
            for ms in [-12, 0, 12] {
                assert!(successful.contains(&ms), "{} missing {}ms leeway", m.id, ms);
            }
            assert!(
                !successful.contains(&-40) && !successful.contains(&40),
                "triples still require deliberate timing"
            );
        }
    }
    #[test]
    fn sustained_terrain_allows_timed_entries_and_releases() {
        for m in manifest() {
            let baseline = game(&m);
            let run = m
                .level_segments
                .iter()
                .find(|s| s.kind == "terrain_run")
                .unwrap();
            let first = baseline.actions
                [cue_index(&baseline, "hold", Runner::beat_time(&m, run.beat))]
            .time;
            let end = m
                .level_segments
                .iter()
                .find(|s| s.kind == "terrain_run" && s.ceiling_beat.is_some())
                .unwrap();
            let release = baseline.actions[cue_index(
                &baseline,
                "release",
                Runner::beat_time(&m, end.ceiling_beat.unwrap()),
            )]
            .time;
            for error in [-0.012, 0.0, 0.012] {
                let mut g = game(&m);
                for a in &baseline.actions {
                    let shifted = if (a.time - first).abs() < 1e-7 && a.action == "hold"
                        || (a.time - release).abs() < 1e-7 && a.action == "release"
                    {
                        a.time + error
                    } else {
                        a.time
                    };
                    g.input(&a.action, shifted);
                }
                g.advance(m.duration);
                assert_eq!(
                    g.status,
                    "won",
                    "{} terrain entry/release {}ms: {}",
                    m.id,
                    error * 1000.0,
                    g.death_reason
                );
            }
        }
    }
    #[test]
    fn raised_passages_require_release_and_allow_input_leeway() {
        for m in manifest() {
            let baseline = game(&m);
            let gate = m
                .level_segments
                .iter()
                .flat_map(|s| s.gate_releases.iter().flatten())
                .find(|beat| {
                    let release = Runner::beat_time(&m, **beat);
                    let index = cue_index(&baseline, "release", release);
                    let mut held = baseline.clone();
                    for a in &baseline.actions[..index] {
                        held.input(&a.action, a.time);
                    }
                    held.advance(baseline.actions[index + 2].time - 0.03);
                    held.status == "dead"
                })
                .expect("course needs a locally mandatory release passage");
            let release = Runner::beat_time(&m, *gate);
            let index = cue_index(&baseline, "release", release);
            let release = baseline.actions[index].time;
            let resume = baseline.actions[index + 1].time;
            assert_eq!(baseline.actions[index + 1].action, "hold");
            for error in [-0.012, 0.0, 0.012] {
                let mut g = game(&m);
                for a in &baseline.actions[..index] {
                    g.input(&a.action, a.time);
                }
                g.input("up", release + error);
                g.input("down", resume + error);
                g.advance(baseline.actions[index + 2].time - 0.03);
                assert_eq!(
                    g.status,
                    "running",
                    "{} gate {}ms: {}",
                    m.id,
                    error * 1000.0,
                    g.death_reason
                );
            }
            let mut held = game(&m);
            for a in &baseline.actions[..index] {
                held.input(&a.action, a.time);
            }
            // A missing gap release fails at the next landing, whereas the
            // older overhead gate failed immediately. Compare both inputs at
            // the same next-control checkpoint used by the leeway replay.
            held.advance(baseline.actions[index + 2].time - 0.03);
            assert_eq!(
                held.status, "dead",
                "{} bridge must require releasing hold",
                m.id
            );
        }
    }
    #[test]
    fn ignoring_timed_releases_fails_and_cliffs_requires_the_most() {
        let mut counts = Vec::new();
        for m in manifest() {
            let baseline = game(&m);
            // Compare a common-length opening. Extended second halves may
            // intentionally intensify beyond another course's opening tier.
            let audit_end = m.duration.min(100.0);
            let mut required = 0;
            let mut total = 0;
            for beat in m.level_segments.iter().flat_map(|s| {
                s.gate_releases
                    .iter()
                    .flatten()
                    .chain(s.ceiling_beat.iter())
            }) {
                let time = Runner::beat_time(&m, *beat);
                if time >= audit_end {
                    continue;
                }
                let index = cue_index(&baseline, "release", time);
                let mut held = game(&m);
                // Skip exactly this release/re-press pair, then continue the
                // remaining controls. A mistimed jump can miss its landing
                // after the next key-up; stopping there undercounts failures.
                for (i, a) in baseline.actions.iter().enumerate() {
                    if a.time > audit_end {
                        break;
                    }
                    if i == index || i == index + 1 {
                        continue;
                    }
                    held.input(&a.action, a.time);
                    if held.status == "dead" {
                        break;
                    }
                }
                held.advance(audit_end);
                total += 1;
                required += usize::from(held.status == "dead");
                if held.status != "dead" {
                    eprintln!(
                        "{} bypassable release at beat {:.9}, time {:.6}s",
                        m.id, beat, time
                    );
                }
            }
            eprintln!("{} mandatory release passages {required}/{total}", m.id);
            assert!(
                total > 10 && required as f64 / audit_end >= 0.65,
                "{} has too few physically necessary release passages: {required}/{total}",
                m.id
            );
            let dashes = baseline
                .actions
                .iter()
                .filter(|a| a.action == "dash")
                .filter(|a| a.time < audit_end)
                .count();
            // A relay requires separate jump and apex re-press edges. Both
            // families contribute to actual control cadence after omission checks.
            counts.push((required + 2 * dashes) as f64 / audit_end);
        }
        let eye = &manifest()[2];
        assert!(
            counts[2] >= counts[1] * 0.95 && eye.reference_events.gravity_tiles.len() >= 2,
            "Eye must retain comparable necessary input density and its authored inversion section"
        );
        assert!(
            counts[3] > counts[..3].iter().copied().fold(0.0, f64::max) * 1.1,
            "Cliffs must require more necessary timed controls per second"
        );
        let jumps: Vec<_> = manifest()
            .iter()
            .map(|map| {
                let mut runner = Runner::new(&serde_json::to_string(map).unwrap()).unwrap();
                replay(&mut runner, 144.0);
                assert_eq!(runner.status, "won");
                runner.jump_count as f64 / map.duration
            })
            .collect();
        assert!(jumps[3] > jumps[..3].iter().copied().fold(0.0, f64::max) * 1.3,
            "Cliffs must have substantially denser physical traversal as well as tighter press windows");
    }
    #[test]
    fn courses_have_distinct_elevated_programs_and_occupied_pits() {
        let maps = manifest();
        let programs: Vec<_> = maps
            .iter()
            .map(|m| {
                m.level_segments
                    .iter()
                    .filter(|s| s.kind == "terrain_run")
                    .map(|s| s.heights.clone())
                    .collect::<Vec<_>>()
            })
            .collect();
        for (i, m) in maps.iter().enumerate() {
            assert!(programs[i].len() >= 8);
            assert!(programs[i].iter().flatten().copied().max().unwrap() >= 3);
            let unique: std::collections::HashSet<_> = programs[i].iter().collect();
            assert!(
                unique.len() >= 6,
                "{} repeats too few terrain profiles",
                m.id
            );
            let changes = m
                .level_segments
                .iter()
                .filter(|s| s.kind == "terrain_run")
                .map(|s| {
                    let mut height = s.entry_height;
                    s.heights
                        .iter()
                        .filter(|&&next| {
                            let changed = height != next;
                            height = next;
                            changed
                        })
                        .count()
                })
                .sum::<usize>();
            assert!(
                changes > 30,
                "{} has only {changes} actual elevation changes",
                m.id
            );
            for j in 0..i {
                assert_ne!(programs[i], programs[j]);
            }
        }
        assert!(
            maps[3]
                .level_segments
                .iter()
                .map(|s| s.heights.len())
                .sum::<usize>()
                > maps[0]
                    .level_segments
                    .iter()
                    .map(|s| s.heights.len())
                    .sum::<usize>()
        );
    }
    #[test]
    fn dash_only_accepts_a_fresh_press_near_the_apex() {
        for m in manifest() {
            for phase in [0.1, 0.34, 0.5, 0.66, 0.85] {
                let mut g = game(&m);
                g.input("down", 0.0);
                g.input("up", 0.01);
                g.input("down", g.flight * phase);
                assert_eq!(g.air_dash_used, phase == 0.5, "{} phase{}", m.id, phase);
                if phase != 0.5 {
                    assert!(g.dash_offset().abs() < 1e-9);
                }
            }
        }
    }
}
