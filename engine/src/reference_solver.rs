//! Bounded input search over the actual reference-adapter simulation.
use super::*;
use std::collections::HashMap;

#[derive(Clone, Copy)]
struct State {
    time: f64,
    y: f64,
    support: f64,
    vy: f64,
    elapsed: f64,
    grounded: bool,
    held: bool,
    jumps: u32,
    status: &'static str,
    dash_started: f64,
    dash_until: f64,
    air_dash_used: bool,
    carry: f64,
    captured: Option<usize>,
}
impl State {
    fn read(g: &Runner) -> Self {
        Self {
            time: g.time,
            y: g.y,
            support: g.support_y,
            vy: g.vy,
            elapsed: g.jump_elapsed,
            grounded: g.grounded,
            held: g.held,
            jumps: g.jump_count,
            status: g.status,
            dash_started: g.dash_started,
            dash_until: g.dash_until,
            air_dash_used: g.air_dash_used,
            carry: g.dash_carry_total,
            captured: g.captured_well,
        }
    }
    fn restore(self, g: &mut Runner) {
        g.time = self.time;
        g.y = self.y;
        g.support_y = self.support;
        g.vy = self.vy;
        g.jump_elapsed = self.elapsed;
        g.grounded = self.grounded;
        g.held = self.held;
        g.jump_count = self.jumps;
        g.status = self.status;
        g.death_reason = "";
        g.air_dash_used = self.air_dash_used;
        g.dash_carry_total = self.carry;
        g.captured_well = self.captured;
        g.dash_started = self.dash_started;
        g.dash_until = self.dash_until;
    }
    fn key(self) -> (i64, i64, i64, i64, bool, bool) {
        (
            (self.y * 10000.0).round() as i64,
            (self.vy * 10000.0).round() as i64,
            (self.support * 10000.0).round() as i64,
            (self.elapsed * 10000.0).round() as i64,
            self.grounded,
            self.held,
        )
    }
}
struct Edge {
    parent: usize,
    action: Option<Action>,
    count: usize,
}

impl Runner {
    /// Offline polish for an already proven physical down/up replay. Choose
    /// the midpoint of the connected safe press window, rather than accepting
    /// a witness sitting against a collision boundary. This uses ordinary
    /// input() and the same collision resolver as manual play.
    pub fn center_grid_controls(&mut self) -> Result<Vec<Action>, String> {
        if !self.map.grid_snap || self.map.reference_geometry.is_some() {
            return Err("Timing centering accepts custom grid geometry only".into());
        }
        let original = self.actions.clone();
        if original
            .iter()
            .any(|a| !matches!(a.action.as_str(), "down" | "up"))
        {
            return Err("Timing centering requires physical down/up controls".into());
        }
        let mut authored_map = self.map.clone();
        authored_map.grid_actions.clear();
        authored_map.grid_cues.clear();
        let authored_json = serde_json::to_string(&authored_map)
            .map_err(|error| format!("Canonical score serialization: {error}"))?;
        let authored = Runner::new(&authored_json)?.actions;
        let mut centered = original.clone();
        self.restart();
        for index in 0..centered.len() {
            let cue = centered[index].clone();
            if cue.action != "down" {
                self.input(&cue.action, cue.time);
                continue;
            }
            let prefix = State::read(self);
            let previous = index
                .checked_sub(1)
                .map(|i| centered[i].time)
                .unwrap_or(0.0);
            let next = centered
                .get(index + 1)
                .map(|a| a.time)
                .unwrap_or(self.map.duration);
            let horizon = centered
                .get(index + 4)
                .map(|a| a.time)
                .unwrap_or(self.map.duration);
            // A press can arm a held landing while airborne outside the
            // dash window. Preserve that control's original meaning too.
            self.input("down", cue.time);
            let expected_launch = self.jump_count > prefix.jumps;
            let expected_dash = self.dash_started > prefix.dash_started + 1e-9;
            prefix.restore(self);
            // Dash presses are authored musical accents. Their existing
            // retiming budget is already spent; do not add centering drift.
            if expected_dash {
                self.input("down", cue.time);
                continue;
            }
            let source_time = authored
                .iter()
                .filter(|action| {
                    if expected_launch {
                        matches!(action.action.as_str(), "jump" | "hold")
                    } else {
                        action.action == "hold"
                    }
                })
                .filter(|action| (action.time - cue.time).abs() <= 0.12)
                .min_by(|a, b| {
                    (a.time - cue.time)
                        .abs()
                        .total_cmp(&(b.time - cue.time).abs())
                })
                .map(|action| action.time);
            let Some(source_time) = source_time else {
                // A redundant role cue is not a license to match an unrelated
                // neighboring attack. Leave unmatched physical edges intact.
                self.input("down", cue.time);
                continue;
            };
            let valid = |runner: &mut Runner, time: f64| {
                if time < previous || time >= next || (time - source_time).abs() > 0.08 + 1e-9 {
                    return false;
                }
                prefix.restore(runner);
                let old_jumps = runner.jump_count;
                let old_dash = runner.dash_started;
                runner.input("down", time);
                let launch = runner.jump_count > old_jumps;
                let dash = runner.dash_started > old_dash + 1e-9;
                if launch != expected_launch || dash != expected_dash || !runner.held {
                    return false;
                }
                for following in centered[index + 1..]
                    .iter()
                    .take_while(|a| a.time <= horizon)
                {
                    runner.input(&following.action, following.time);
                }
                runner.advance_exact(horizon);
                runner.status != "dead"
            };
            if !valid(self, cue.time) {
                return Err(format!("Existing physical replay fails at press {index}"));
            }
            let mut low = cue.time;
            let mut high = cue.time;
            // Never move a musical cue more than 30ms. Keep the connected
            // window: another distant valid solution can be a different hop.
            for direction in [-1.0, 1.0] {
                for offset in 1..=7 {
                    let candidate = cue.time + direction * offset as f64 * STEP;
                    if !valid(self, candidate) {
                        break;
                    }
                    if direction < 0.0 {
                        low = candidate;
                    } else {
                        high = candidate;
                    }
                }
            }
            let midpoint = (low + high) * 0.5;
            prefix.restore(self);
            centered[index].time = midpoint;
            self.input("down", midpoint);
        }
        self.advance_exact(self.map.duration);
        if self.status != "won" {
            // Local windows can interact across a later held chain. Never
            // publish the polished result unless the complete replay wins.
            self.restart();
            for cue in &original {
                self.input(&cue.action, cue.time);
            }
            self.advance_exact(self.map.duration);
            return Err("Centered replay failed its complete physical replay".into());
        }
        Ok(centered)
    }

    /// Offline, constrained timing search for a quantized authored course.
    /// Keeps each musical action within 80ms of its source cue.
    pub fn retime_grid_controls(&mut self, bound: usize) -> Result<Vec<Action>, String> {
        let source = self.actions.clone();
        self.restart();
        let started = std::time::Instant::now();
        let mut history = vec![(0usize, None::<Action>)];
        let mut frontier = vec![(State::read(self), 0usize, 0.0f64)];
        let diagnostic_path = std::env::var("TOWER_GRID_DIAGNOSTIC").ok();
        for (index, cue) in source.iter().enumerate() {
            if started.elapsed().as_secs() > 180 {
                return Err(format!(
                    "Grid timing search exceeded 180 seconds at cue {index}"
                ));
            }
            let radius = if cue.action == "dash" { 0.035 } else { 0.08 };
            let mut next = HashMap::new();
            let mut failure: Option<(f64, serde_json::Value)> = None;
            macro_rules! diagnose_failure {
                ($phase:expr, $time:expr, $parent:expr, $score:expr) => {
                    if diagnostic_path.is_some()
                        && failure.as_ref().is_none_or(|(score, _)| $score < *score)
                    {
                        let mut prefix = Vec::new();
                        let mut parent = $parent;
                        while parent != 0 {
                            if let Some(action) = &history[parent].1 {
                                prefix.push(action.clone());
                            }
                            parent = history[parent].0;
                        }
                        prefix.reverse();
                        let x = self.x();
                        let nearby: Vec<_> = self.objects.iter()
                            .filter(|o| o.x + o.w > x - 2.0 * SIZE && o.x < x + 5.0 * SIZE)
                            .collect();
                        let contacts: Vec<_> = nearby.iter().copied().filter(|o| {
                            if o.kind == "spike" {
                                return Self::triangle_hits_player(o, x, self.y);
                            }
                            o.kind != "terrain" && o.kind != "tower"
                                && x + SIZE / 2.0 > o.x && x - SIZE / 2.0 < o.x + o.w
                                && self.y < o.y + o.h && self.y + SIZE > o.y
                        }).collect();
                        failure = Some(($score, serde_json::json!({
                            "map":self.map.id, "cue_index":index, "cue":cue,
                            "candidate_time":$time, "phase":$phase,
                            "state":{"time":self.time,"x":x,"y":self.y,"vy":self.vy,
                                "grounded":self.grounded,"held":self.held,
                                "jump_elapsed":self.jump_elapsed,"support_y":self.support_y,
                                "status":self.status,"death_reason":self.death_reason,
                                "dash_started":self.dash_started,"dash_until":self.dash_until,
                                "dash_carry":self.dash_carry_total},
                            "following_cue":source.get(index+1),
                            "nearby_objects":nearby,"contact_candidates":contacts,
                            "proven_prefix":prefix
                        })));
                    }
                };
            }
            for (state, parent, cost) in frontier {
                for step in -((radius / STEP) as i32)..=((radius / STEP) as i32) {
                    let time = cue.time + step as f64 * STEP;
                    if time < state.time || time < 0.0 || time >= self.map.duration {
                        continue;
                    }
                    state.restore(self);
                    self.advance_exact(time);
                    if self.status == "dead" {
                        diagnose_failure!(
                            "before_input",
                            time,
                            parent,
                            cost + (time - cue.time).powi(2)
                        );
                        continue;
                    }
                    let jumps = self.jump_count;
                    let prior_dash = self.dash_started;
                    self.input(&cue.action, time);
                    if self.status == "dead"
                        || cue.action == "jump" && self.jump_count == jumps
                        || cue.action == "dash" && self.dash_started <= prior_dash + 1e-9
                        || cue.action == "hold" && self.dash_started != prior_dash
                    {
                        diagnose_failure!(
                            "input_semantics",
                            time,
                            parent,
                            cost + (time - cue.time).powi(2)
                        );
                        continue;
                    }
                    if let Some(following) = source.get(index + 1) {
                        let next_radius = if following.action == "dash" {
                            0.035
                        } else {
                            0.08
                        };
                        self.advance_exact((following.time - next_radius).max(time));
                        if self.status == "dead" {
                            diagnose_failure!(
                                "following_horizon",
                                time,
                                parent,
                                cost + (time - cue.time).powi(2)
                            );
                            continue;
                        }
                    }
                    let saved = State::read(self);
                    let score = cost + (time - cue.time).powi(2);
                    let key = (
                        saved.key(),
                        (saved.time * 10000.0).round() as i64,
                        (saved.dash_started * 10000.0).round() as i64,
                        saved.air_dash_used,
                    );
                    if next.get(&key).is_some_and(|(_, _, prior)| *prior <= score) {
                        continue;
                    }
                    history.push((
                        parent,
                        Some(Action {
                            time,
                            action: cue.action.clone(),
                        }),
                    ));
                    next.insert(key, (saved, history.len() - 1, score));
                }
            }
            if next.is_empty() {
                if let (Some(path), Some((_, diagnostic))) = (&diagnostic_path, failure) {
                    let encoded = serde_json::to_string_pretty(&diagnostic)
                        .map_err(|error| format!("Grid diagnostic serialization: {error}"))?;
                    std::fs::write(path, encoded)
                        .map_err(|error| format!("Grid diagnostic write: {error}"))?;
                    let id: String = self
                        .map
                        .id
                        .chars()
                        .filter(|c| c.is_ascii_alphanumeric() || *c == '-' || *c == '_')
                        .collect();
                    let per_map = std::path::Path::new(path).with_extension(format!("{id}.json"));
                    let encoded = serde_json::to_string_pretty(&diagnostic)
                        .map_err(|error| format!("Grid diagnostic serialization: {error}"))?;
                    std::fs::write(per_map, encoded)
                        .map_err(|error| format!("Grid per-map diagnostic write: {error}"))?;
                }
                return Err(format!(
                    "No grid trajectory at cue {index} {} {:.6}s",
                    cue.action, cue.time
                ));
            }
            // A cheap input-time score can fill the beam with launches that
            // all reach an eventual raised sidewall. Rank a complete local
            // flight before pruning, keeping the actual timing cost unchanged.
            // The forecast is a preference, never an extra rejection rule:
            // the following release still gets its full search next iteration.
            let lookahead = matches!(cue.action.as_str(), "jump" | "hold")
                && source.get(index + 1).is_some_and(|a| a.action == "release");
            let mut ranked = Vec::with_capacity(next.len());
            for candidate in next.into_values() {
                let mut predicted_collision = false;
                if lookahead {
                    let release = &source[index + 1];
                    let mut horizon = source
                        .get(index + 2)
                        .map(|a| a.time - if a.action == "dash" { 0.035 } else { 0.08 })
                        .unwrap_or(self.map.duration);
                    candidate.0.restore(self);
                    // A +2-cell landing has a very small apex window. An
                    // early following hold can precede the sidewall by 80ms;
                    // stopping there hides a doomed arc from beam ranking.
                    // Extend only to the nearby raised face's actual contact,
                    // not past an arbitrary later obstacle/next full hop.
                    let launch_x = self.x() - self.jump_elapsed * self.speed;
                    let first = self
                        .collision_indices
                        .partition_point(|&i| self.objects[i].x < launch_x + 0.5 * SIZE);
                    if let Some(face) = self.collision_indices[first..]
                        .iter()
                        .map(|&i| &self.objects[i])
                        .take_while(|object| object.x < launch_x + 3.5 * SIZE)
                        .find(|object| {
                            object.kind == "enemy"
                                && object.y + object.h - self.support_y > 1.5 * SIZE
                                && object.y + object.h - self.support_y <= 2.05 * SIZE
                        })
                    {
                        let contact = self.time
                            + ((face.x - SIZE / 2.0 - self.x()) / self.speed).max(0.0)
                            + 2.0 * STEP;
                        horizon = horizon.max(contact).min(self.map.duration);
                    }
                    if release.time >= self.time && horizon >= release.time {
                        self.input("release", release.time);
                        self.advance_exact(horizon);
                        predicted_collision = self.status == "dead";
                    }
                }
                ranked.push((predicted_collision, candidate));
            }
            ranked.sort_by(|a, b| a.0.cmp(&b.0).then_with(|| a.1 .2.total_cmp(&b.1 .2)));
            frontier = ranked.into_iter().map(|(_, candidate)| candidate).collect();
            frontier.truncate(bound);
        }
        for (state, mut id, _) in frontier {
            state.restore(self);
            self.advance_exact(self.map.duration);
            if self.status != "won" {
                continue;
            }
            let mut actions = Vec::new();
            while id != 0 {
                if let Some(action) = &history[id].1 {
                    actions.push(action.clone());
                }
                id = history[id].0;
            }
            actions.reverse();
            self.restart();
            for action in &actions {
                self.input(&action.action, action.time);
            }
            self.advance_exact(self.map.duration);
            if self.status == "won" {
                return Ok(actions);
            }
        }
        Err("Grid trajectory failed its final replay".into())
    }
    pub fn solve_reference_controls(&mut self, bound: usize) -> Result<Vec<Action>, String> {
        if self.map.reference_geometry.is_none() {
            return Err("Solver accepts direct reference geometry only".into());
        }
        self.restart();
        let started = std::time::Instant::now();
        let mut history = vec![Edge {
            parent: 0,
            action: None,
            count: 0,
        }];
        let mut frontier = vec![(State::read(self), 0usize)];
        let step = 1.0 / 60.0;
        let ticks = (self.map.duration / step).ceil() as usize;
        let mut winner = None;
        let mut peak_frontier = 0;
        let mut truncated_ticks = 0;
        for tick in 0..ticks {
            if started.elapsed().as_secs() > 180 {
                return Err(format!(
                    "Reference search exceeded 180 seconds at tick {tick}"
                ));
            }
            let at = tick as f64 * step;
            let mut next: HashMap<_, (State, usize)> = HashMap::new();
            for (state, parent) in frontier {
                for held in [state.held, !state.held] {
                    state.restore(self);
                    let changed = held != state.held;
                    if changed {
                        self.input(if held { "hold" } else { "release" }, at);
                    }
                    self.advance((tick + 1) as f64 * step);
                    if self.time >= self.map.duration - step && tick + 1 == ticks {
                        self.advance(self.playback_end());
                    }
                    if self.status == "dead" {
                        continue;
                    }
                    let count = history[parent].count + usize::from(changed);
                    let saved = State::read(self);
                    let key = saved.key();
                    if next.get(&key).is_some_and(|(prior, id)| {
                        (prior.jumps, history[*id].count) <= (saved.jumps, count)
                    }) {
                        continue;
                    }
                    let id = if changed {
                        history.push(Edge {
                            parent,
                            action: Some(Action {
                                time: at,
                                action: if held { "hold" } else { "release" }.into(),
                            }),
                            count,
                        });
                        history.len() - 1
                    } else {
                        parent
                    };
                    if self.status == "won" {
                        let score = (saved.jumps, count);
                        if winner.as_ref().is_none_or(|(prior, _)| score < *prior) {
                            winner = Some((score, id));
                        }
                        continue;
                    }
                    next.insert(key, (saved, id));
                }
            }
            if winner.is_some() {
                break;
            }
            if next.is_empty() {
                return Err(format!(
                    "No surviving reference trajectory at {:.6}s (tile {:.3}); peak frontier {peak_frontier}, truncated ticks {truncated_ticks}",
                    (tick + 1) as f64 * step,
                    self.x() / SIZE
                ));
            }
            frontier = next.into_values().collect();
            peak_frontier = peak_frontier.max(frontier.len());
            if frontier.len() > bound {
                truncated_ticks += 1;
                frontier.sort_by_key(|(state, id)| (state.jumps, history[*id].count));
                frontier.truncate(bound);
            }
        }
        let (_, mut id) = winner.ok_or("No winning reference trajectory within bounded search")?;
        let mut actions = Vec::new();
        while id != 0 {
            if let Some(a) = &history[id].action {
                actions.push(a.clone());
            }
            id = history[id].parent;
        }
        actions.reverse();
        self.restart();
        for a in &actions {
            self.input(&a.action, a.time);
        }
        self.advance(self.playback_end());
        if self.status != "won" {
            return Err(format!(
                "Reference witness failed replay: {} at {:.6}s",
                self.death_reason, self.time
            ));
        }
        Ok(actions)
    }
}
