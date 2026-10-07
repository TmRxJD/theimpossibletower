use super::evidence::Evidence;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

#[derive(Clone, Copy, Serialize, Deserialize)]
pub struct Rules {
    pub low_register: f64,
    pub high_register: f64,
    pub hot_flat_beats: f64,
    pub hot_span_cells: f64,
    pub hot_travel_cells: f64,
    pub hold_limit: u8,
    pub gap_bias: f64,
    pub release_bias: f64,
}

impl Rules {
    pub fn for_song(id: &str) -> Result<Self, String> {
        let (high, span, travel, hold, gap, release) = match id {
            "elements" => (17.0, 3.0, 6.0, 3, 0.35, 0.35),
            "crystallize" => (21.0, 4.0, 8.0, 2, 0.60, 0.65),
            "eye-of-the-untold-her" => (22.0, 4.0, 10.0, 2, 0.80, 0.90),
            _ => return Err("Cliffs is locked; no implicit fallback profile".into()),
        };
        Ok(Self { low_register: 3.0, high_register: high, hot_flat_beats: 2.0,
            hot_span_cells: span, hot_travel_cells: travel, hold_limit: hold,
            gap_bias: gap, release_bias: release })
    }
}

#[derive(Clone, Debug, Serialize)]
pub struct Hop {
    pub press: f64,
    pub landing: f64,
    pub next_press: f64,
    pub from: u8,
    pub to: u8,
    pub rest: f64,
    pub rule: String,
    pub kind: String,
    pub count: u8,
    pub width: u8,
    pub held: bool,
}

impl Hop {
    pub fn token(&self, period: f64) -> String {
        format!("{}:{}:{}", self.to as i32-self.from as i32,
            ((self.next_press-self.press)/period*4.0).round() as i32,
            if self.held {"hold"}else{"press"})
    }
}

#[derive(Clone)]
struct State {
    time: f64,
    height: u8,
    cost: f64,
    path: Vec<Hop>,
    flat_beats: f64,
    held: u8,
    last_delta: i8,
}

#[derive(Default)]
pub struct History {
    pub hops: Vec<Hop>,
    pub tokens: Vec<String>,
    pub sixes: HashMap<String, usize>,
    pub eights: HashSet<String>,
    pub twelves: HashSet<String>,
}

impl History {
    pub fn append(&mut self, hops: &[Hop], period: f64) {
        self.hops.extend_from_slice(hops);
        for h in hops {
            self.tokens.push(h.token(period));
            if self.tokens.len()>=6 { *self.sixes.entry(self.tokens[self.tokens.len()-6..].join("|")).or_default()+=1; }
            if self.tokens.len()>=8 { self.eights.insert(self.tokens[self.tokens.len()-8..].join("|")); }
            if self.tokens.len()>=12 { self.twelves.insert(self.tokens[self.tokens.len()-12..].join("|")); }
        }
    }
}

fn noise(seed: u64, event: usize, height: u8, delta: i8) -> f64 {
    let mut x=seed ^ (event as u64).wrapping_mul(0x9e3779b97f4a7c15) ^ ((height as u64)<<16) ^ delta as u64;
    x ^= x>>30; x=x.wrapping_mul(0xbf58476d1ce4e5b9); x^=x>>27;
    (x&65535) as f64 / 65535.0
}

/// A bounded DAG search over actual recording/MIDI attacks and reachable
/// full-cell landings. No prefab height arrays, fixed bar loops, or floor reset.
pub fn phrase(e: &Evidence, rules: Rules, start: f64, end: f64, entry: u8,
              target_range: (u8,u8), history: &History, forbidden: &HashSet<String>, seed: u64) -> Result<Vec<Hop>, String> {
    let mut last=String::new();
    for bound in [64,128,256] {
        match phrase_with_bound(e,rules,start,end,entry,target_range,history,forbidden,seed,bound) {
            Ok(path)=>return Ok(path),Err(error)=>last=error,
        }
    }
    Err(format!("{last}; bounded search exhausted beams64/128/256"))
}

fn phrase_with_bound(e: &Evidence, rules: Rules, start: f64, end: f64, entry: u8,
              target_range: (u8,u8), history: &History, forbidden: &HashSet<String>, seed: u64,bound:usize) -> Result<Vec<Hop>, String> {
    if end-start<0.45*e.period { return Err(format!("phrase too short {start:.3}..{end:.3}")); }
    let mut events: Vec<f64> = e.attacks.iter().copied().filter(|t| *t>start+0.5*e.period && *t<end-0.04*e.period).collect();
    // A sustained recording region may contain no transient for several beats.
    // Fill only those holes with explicit, labelled musical subdivisions.
    let first=(start/e.period*4.0).ceil() as usize;
    let last=(end/e.period*4.0).floor() as usize;
    for quarter in first..last {
        let at=quarter as f64*e.period/4.0;
        if at>start+0.5*e.period && e.attacks.iter().all(|t|(*t-at).abs()>0.45*e.period) {events.push(at);}
    }
    events.push(end);events.sort_by(f64::total_cmp);events.dedup_by(|a,b|(*a-*b).abs()<1e-7);
    let other_phrases:Vec<Vec<String>>=forbidden.iter().filter(|s|s.split('|').count()==16)
        .map(|s|s.split('|').map(str::to_string).collect()).collect();
    let event_features:Vec<_>=events.iter().map(|at|e.at(*at)).collect();
    let event_targets:Vec<_>=events.iter().map(|at|e.target(*at,rules.low_register,rules.high_register)).collect();
    let mut feature_cache=HashMap::new();
    let opening=history.hops.first().map(|h|h.press).unwrap_or(start);
    let mut hot_windows=Vec::new();let mut window=opening;
    while window+8.0*e.period<=end {
        if window+8.0*e.period>start && (0..32).filter(|i|e.hot(window+*i as f64/32.0*8.0*e.period)).count()>=12 {
            hot_windows.push((window,window+8.0*e.period));
        }
        window+=2.0*e.period;
    }
    let mut nodes: Vec<Vec<State>> = vec![Vec::new();events.len()+1];
    let mut rejected_window=None;
    let mut furthest=start;
    nodes[0].push(State { time:start,height:entry,cost:0.0,path:Vec::new(),flat_beats:0.0,held:0,last_delta:0 });
    for node in 0..events.len() {
        let mut current=std::mem::take(&mut nodes[node]);
        current.sort_by(|a,b|a.cost.total_cmp(&b.cost));
        // Keep distinct entry heights/directions, rather than only the best
        // greedy pitch choice; lookahead needs alternatives at the next turn.
        let mut seen=HashSet::new();
        current.retain(|s|seen.insert((s.height,s.last_delta,(s.flat_beats*4.0).round() as u8,s.held,
            (s.time*240.0).round() as u64,s.path.iter().rev().take(8).map(|h|h.token(e.period)).collect::<Vec<_>>())));
        current.truncate(bound);
        for state in current {
            furthest=f64::max(furthest,state.time);
            let from=*feature_cache.entry(state.time.to_bits()).or_insert_with(||e.at(state.time));
            for (j,&at) in events.iter().enumerate().skip(node) {
                let interval=(at-state.time)/e.period;
                if interval<0.52 { continue; }
                if interval>2.65 { break; }
                let last=j+1==events.len();
                let to=event_features[j];
                let hot=from.hot(e.lead_cut)||to.hot(e.lead_cut);
                let tower_approach=target_range==(0,0)&&at>=end-6.0*e.period;
                let desired=event_targets[j]*if tower_approach {((end-at)/(6.0*e.period)).clamp(0.0,1.0)}else{1.0};
                let pitch_delta=if !tower_approach && from.source==to.source && e.orientation(state.time)==e.orientation(at) { from.pitch.zip(to.pitch).map(|(a,b)|(b-a)*e.orientation(at)) } else { None };
                for delta in -6i8..=2 {
                    // A held rebound inherits the snapped touchdown phase.
                    // The narrow +2-cell face requires a deliberate timed
                    // launch, so never attach it to an automatic rebound.
                    if delta==2 && state.path.last().or_else(||history.hops.last()).is_some_and(|h|h.held) {continue;}
                    let target=state.height as i16+delta as i16;
                    if !(0..=24).contains(&target) || last && !(target_range.0 as i16..=target_range.1 as i16).contains(&target) {continue;}
                    if pitch_delta.is_some_and(|p|p.abs()>0.5 && p*(delta as f64)<0.0) {continue;}
                    let flight=(1.0+(1.0-delta as f64/2.02).sqrt())/2.0;
                    let mut rest=interval-flight;
                    let hold=!last && rest.abs()<0.025 && delta<=1 && from.sustained && state.held<if hot {1}else{rules.hold_limit}
                        && noise(seed,j,target as u8,delta)>rules.release_bias;
                    if hold {rest=0.0;}
                    if rest<if hold {0.0}else{0.06} || rest>if last {1.2}else if hot {0.75}else{1.1} {continue;}
                    let next=if hold {state.time+flight*e.period}else{at};
                    let flat=if delta==0 {state.flat_beats+interval}else{0.0};
                    if hot && flat>rules.hot_flat_beats {continue;}
                    let distance=(target as f64-desired).abs();
                    let onset_score=to.lead+to.energy*0.2;
                    let desired_delta=pitch_delta.map(|d|if d>0.5 {2.0}else if d < -0.5 {-3.0}else{0.0}).unwrap_or((to.energy-from.energy)*6.0);
                    let phase=if tower_approach {"tower_approach"}else if delta>0 {"ascending_attack"}else if delta < -1 {"descending_release"}else if hold {"sustained_chain"}else if from.sustained {"melody_turn"}else{"articulated_attack"};
                    let gap=delta < -1 || delta>0 && to.lead<0.35 || noise(seed,j,target as u8,delta)<rules.gap_bias*0.45;
                    let kind=if hold {"ceiling"}else if gap||last {"gap"}else{"spike"};
                    let count=if hot&&to.lead>0.65 {3}else if to.lead>0.35 {2}else{1};
                    let hop=Hop {press:state.time,landing:state.time+flight*e.period,next_press:next,
                        from:state.height,to:target as u8,rest,rule:phase.into(),kind:kind.into(),count,
                        width:if hot {1}else if from.sustained {2}else{1},held:hold};
                    let mut path=state.path.clone();path.push(hop);
                    let begin=next-8.0*e.period;
                    let recent:Vec<_>=history.hops.iter().chain(path.iter()).filter(|h|h.next_press>=begin&&h.press<=next).collect();
                    let intense=hot;
                    let mut height_span=0.0;let mut vertical=0.0;
                    if let Some(first)=recent.first() {
                        let mut lo=first.from;let mut hi=first.from;
                        for h in &recent {lo=lo.min(h.to);hi=hi.max(h.to);vertical+=(h.to as f64-h.from as f64).abs();}
                        height_span=(hi-lo) as f64;
                    }
                    let lack=if intense {(rules.hot_span_cells-height_span).max(0.0)*1.2+(rules.hot_travel_cells-vertical).max(0.0)*0.5}else{0.0};
                    // Reject completed musical windows during search, using
                    // the same reached-support metric as the replay audit.
                    // Carry contacts from previous phrases and mechanics;
                    // resetting height/history at an anchor hides flat runs.
                    let mut contacts=vec![super::audit::Landing{time:0.0,x:0.0,height:0.0,action:"start".into()}];
                    contacts.extend(history.hops.iter().chain(path.iter()).flat_map(|h|[
                        super::audit::Landing{time:h.press,x:0.0,height:h.from as f64,action:"fresh_jump".into()},
                        super::audit::Landing{time:h.landing,x:0.0,height:h.to as f64,action:"landing".into()}]));
                    contacts.sort_by(|a,b|a.time.total_cmp(&b.time));
                    let mut rejected=false;
                    for &(window,finish) in hot_windows.iter().filter(|(_,finish)|*finish>state.time&&*finish<=next) {
                                let (span,travel,plateau)=super::audit::elevation(&contacts,window,finish);
                                if span<rules.hot_span_cells || travel<rules.hot_travel_cells || plateau/e.period>rules.hot_flat_beats {
                                    rejected_window=Some(serde_json::json!({"window_start":window,"window_end":finish,"span":span,"travel":travel,"flat_beats":plateau/e.period,
                                        "candidate_time":next,"tail":path.iter().rev().take(8).collect::<Vec<_>>()}));
                                    rejected=true;break;
                                }
                    }
                    if rejected {continue;}
                    let mut tokens: Vec<_>=history.tokens.iter().rev().take(15).cloned().collect();tokens.reverse();
                    tokens.extend(path.iter().map(|h|h.token(e.period)));
                    if tokens.len()>=8 && forbidden.contains(&tokens[tokens.len()-8..].join("|")) {continue;}
                    if tokens.len()>=12 {
                        let tail=tokens[tokens.len()-12..].join("|");
                        if history.twelves.contains(&tail) || tokens[..tokens.len()-1].windows(12).any(|w|w.join("|")==tail) {continue;}
                    }
                    if tokens.len()>=16 {
                        let tail=&tokens[tokens.len()-16..];
                        let prior:Vec<_>=history.tokens.iter().cloned().chain(path[..path.len()-1].iter().map(|h|h.token(e.period))).collect();
                        if prior.windows(16).take(prior.len().saturating_sub(30)).any(|w|super::audit::similarity(tail,w)>=0.85)
                            || other_phrases.iter().any(|w|super::audit::similarity(tail,w)>=0.85) {continue;}
                    }
                    let repeats=if tokens.len()>=6 {
                        let tail=tokens[tokens.len()-6..].join("|");
                        *history.sixes.get(&tail).unwrap_or(&0)
                            +path[..path.len()-1].windows(6).filter(|w|w.iter().map(|h|h.token(e.period)).collect::<Vec<_>>().join("|")==tail).count()
                    }else{0};
                    if repeats>=2 {continue;}
                    let cost=state.cost+distance*0.12*interval+(delta as f64-desired_delta).abs()*0.28
                        +if delta==0 && hot {2.4}else{0.0}
                        +if delta==state.last_delta && delta!=0 {0.6}else{0.0}
                        +rest*if hot {2.0}else{0.5}-1.2-onset_score+ lack
                        +repeats as f64*4.0+noise(seed,j,target as u8,delta)*0.5;
                    nodes[j+1].push(State{time:next,height:target as u8,cost,path,flat_beats:flat,
                        held:if hold {state.held+1}else{0},last_delta:delta});
                    if nodes[j+1].len()>bound*8 {nodes[j+1].sort_by(|a,b|a.cost.total_cmp(&b.cost));nodes[j+1].truncate(bound*2);}
                }
            }
        }
    }
    let mut finals=std::mem::take(nodes.last_mut().unwrap());
    finals.sort_by(|a,b|a.cost.total_cmp(&b.cost));
    if finals.is_empty() {
        if let Ok(path)=std::env::var("TOWER_AUTHOR_DIAGNOSTIC") {
            let destination=std::path::Path::new(&path).with_file_name("grammar-rejection.json");
            std::fs::write(destination,serde_json::to_vec_pretty(&serde_json::json!({"start":start,"end":end,"entry":entry,"range":target_range,"furthest":furthest,"rejected_window":rejected_window})).map_err(|e|e.to_string())?).map_err(|e|e.to_string())?;
        }
    }
    let mut path=finals.into_iter().next().ok_or(format!("no musical grammar route {start:.3}..{end:.3} from{entry}; furthest{furthest:.3}; window{rejected_window:?}"))?.path;
    // A spike immediately before a two-cell face has only one-cell clearance.
    // Restrict the group itself instead of flattening the next musical hop.
    for i in 0..path.len().saturating_sub(1) {
        let rise=path[i+1].to as i16-path[i].to as i16;
        if rise>0 {path[i].count=path[i].count.min(if rise>1 {1}else{2});}
    }
    Ok(path)
}
