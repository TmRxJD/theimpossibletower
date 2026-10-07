use super::evidence::Evidence;
use super::grammar::Rules;
use crate::{Action,Runner};
use serde::{Deserialize,Serialize};
use serde_json::{json,Value};
use std::collections::{HashMap,HashSet};

#[derive(Clone,Debug,Serialize,Deserialize)]
pub struct Landing { pub time:f64,pub x:f64,pub height:f64,pub action:String }

/// Record supports reached by the ordinary player replay. Roof rectangles and
/// normal jump oscillation cannot inflate the elevation score.
pub fn witness(track:&Value)->Result<Vec<Landing>,String> {
    let mut runner=Runner::new(&track.to_string())?;
    let actions:Vec<Action>=serde_json::from_value(track["grid_actions"].clone()).map_err(|e|e.to_string())?;
    let mut rows=vec![Landing{time:0.0,x:0.0,height:0.0,action:"start".into()}];
    runner.enable_contact_audit();
    let finish=track["duration"].as_f64().ok_or("duration absent")?;
    for a in actions {runner.input(&a.action,a.time);}
    runner.advance(finish+0.01);
    {
        let s=runner.snapshot();
        if s.status=="dead" {return Err(format!("{} physical witness died {} at{:.6}",track["id"],s.death_reason,s.time));}
    }
    if runner.snapshot().status!="won" {return Err("physical witness did not win".into());}
    rows.extend(runner.contact_audit_events().iter().map(|event|Landing{time:event.time,x:event.x/28.0,height:event.support_y/28.0,action:event.cause.into()}));
    let last=runner.snapshot();
    if last.grounded {push(&mut rows,Landing{time:last.time,x:last.x/28.0,height:last.support_y/28.0,action:"landing".into()});}
    Ok(rows)
}

fn push(rows:&mut Vec<Landing>, row:Landing) {
    if let Some(last)=rows.last_mut() {
        if (row.height-last.height).abs()<1e-6 && row.time-last.time<0.08 { *last=row;return; }
    }
    rows.push(row);
}

pub fn elevation(rows:&[Landing],start:f64,end:f64)->(f64,f64,f64) {
    let supports:Vec<_>=rows.iter().filter(|r|r.action=="landing"||r.action=="start").collect();
    let initial=supports.iter().rev().find(|r|r.time<=start).map(|r|r.height).unwrap_or(0.0);
    let mut low=initial;let mut high=initial;let mut previous=initial;let mut travel=0.0;
    let mut flat_start=start;let mut flat=0.0;let mut visited=start;
    for r in supports.iter().filter(|r|r.time>start&&r.time<end) {
        low=f64::min(low,r.height);high=f64::max(high,r.height);
        if (r.height-previous).abs()>1e-6 {travel+=(r.height-previous).abs();}
        previous=r.height;
    }
    previous=initial;
    for r in rows.iter().filter(|r|r.time>start&&r.time<end) {
        if (r.height-previous).abs()>1e-6 {
            flat=f64::max(flat,visited-flat_start);flat_start=r.time;
        }
        previous=r.height;visited=r.time;
    }
    // A launch begins airborne travel. Do not extend a flat shelf through
    // that flight when the next contact changes elevation. Same-height
    // rebounds still extend the plateau and expose hold-only flat sections.
    let next=rows.iter().find(|r|r.time>=end);
    if next.is_none_or(|r|r.action!="landing"||(r.height-previous).abs()<1e-6) {visited=end;}
    (high-low,travel,f64::max(flat,visited-flat_start))
}

pub fn tokens(rows:&[Landing],period:f64)->Vec<String> {
    let launches:Vec<_>=rows.iter().filter(|r|matches!(r.action.as_str(),"fresh_jump"|"held_jump"|"jump")).collect();
    launches.windows(2).map(|r|format!("{}:{}:{}",(r[1].height-r[0].height).round() as i32,
        ((r[1].time-r[0].time)/period*4.0).round() as i32,
        if r[1].action=="held_jump" {"hold"}else{"press"})).collect()
}

pub fn fingerprints(tokens:&[String],size:usize)->HashMap<String,Vec<usize>> {
    let mut found=HashMap::<String,Vec<usize>>::new();
    for (i,window) in tokens.windows(size).enumerate() {found.entry(window.join("|")).or_default().push(i);}
    found
}

/// Compare relative elevation, musical interval, and required button behavior.
/// Absolute elevation, sprite choice and spike color cannot disguise a copy.
pub fn similarity(a:&[String],b:&[String])->f64 {
    if a.len()!=b.len() || a.is_empty() {return 0.0;}
    let mut loss=0usize;let maximum_loss=3*a.len();
    for (a,b) in a.iter().zip(b) {
        let mut left=a.split(':');let mut right=b.split(':');
        if left.next()!=right.next() {loss+=9;}
        if left.next()!=right.next() {loss+=6;}
        if left.next()!=right.next() {loss+=5;}
        if loss>maximum_loss {return 0.0;}
    }
    1.0-loss as f64/(20*a.len()) as f64
}

pub fn near_repeats(sequence:&[String])->Vec<(usize,usize,f64)> {
    let mut result=Vec::new();
    for (i,a) in sequence.windows(16).enumerate() {
        for (j,b) in sequence.windows(16).enumerate().skip(i+16) {
            let score=similarity(a,b);
            if score>=0.85 {result.push((i,j,score));}
        }
    }
    result
}

pub fn audit(track:&Value,e:&Evidence,rows:&[Landing],others:&HashSet<String>)->Value {
    let id=track["id"].as_str().unwrap();let rules=Rules::for_song(id).unwrap();
    let mut failures=Vec::<String>::new();let mut windows=Vec::<Value>::new();
    let p=e.period;let span=8.0*p;
    let mut start=track["generation_hops"][0]["press"].as_f64().or_else(||
        rows.iter().find(|r|matches!(r.action.as_str(),"fresh_jump"|"held_jump"|"jump"))
        .map(|r|r.time)).unwrap_or(2.0*p);
    while start+span<e.duration-2.0*p {
        let end=start+span;
        let hot=(0..32).filter(|i|e.hot(start+*i as f64/32.0*span)).count() as f64/32.0;
        if hot>=0.35 {
            let (height,travel,flat)=elevation(rows,start,end);
            let mut reasons=Vec::new();
            if height<rules.hot_span_cells {reasons.push("insufficient elevation span");}
            if travel<rules.hot_travel_cells {reasons.push("insufficient vertical traversal");}
            if flat/p>rules.hot_flat_beats {reasons.push("long flat landing sequence");}
            if !reasons.is_empty() {failures.push(format!("{id} {start:.3}..{end:.3}: {}",reasons.join(", ")));}
            windows.push(json!({"start":start,"end":end,"intense_fraction":hot,"span_cells":height,
                "travel_cells":travel,"flat_beats":flat/p,"reasons":reasons}));
        }
        start+=2.0*p;
    }
    let sequence=tokens(rows,p);
    let launches:Vec<_>=rows.iter().filter(|r|matches!(r.action.as_str(),"fresh_jump"|"held_jump"|"jump")).collect();
    let mut alignment=Vec::new();let mut pitch_checks=Vec::new();
    if let Some(hops)=track["generation_hops"].as_array() {
        let mut cursor=0usize;
        for hop in hops {
            let press=hop["press"].as_f64().unwrap();let landing=hop["landing"].as_f64().unwrap();
            let next=hop["next_press"].as_f64().unwrap();
            while cursor<launches.len() && launches[cursor].time<press-0.081 {failures.push(format!("{id}: unintended extra launch at {:.6}",launches[cursor].time));cursor+=1;}
            let Some(actual)=launches.get(cursor).filter(|r|(r.time-press).abs()<=0.081) else {
                failures.push(format!("{id}: authored launch at {press:.6} has no ordinary-player counterpart"));continue;
            };
            cursor+=1;alignment.push((actual.time-press).abs());
            let actual_landing=rows.iter().find(|r|r.action=="landing" && r.time>actual.time+0.02 && r.time<landing+0.12);
            let mechanic=matches!(hop["kind"].as_str(),Some("dash"|"dash_route"|"gravity_route"));
            let a=e.at(press);let b=e.at(next);
            if !mechanic && hop["rule"]!="goal_landing" && hop["rule"]!="tower_approach" && a.source==b.source && e.orientation(press)==e.orientation(next)
                && e.pitch_confidence(press)>=0.7 && e.pitch_confidence(next)>=0.7 {
                if let (Some(first),Some(last),Some(actual_landing))=(a.pitch,b.pitch,actual_landing) {
                    let pitch=last-first;
                    if pitch.abs()>=1.0 {
                        let delta=(actual_landing.height-actual.height)*e.orientation(press);
                        pitch_checks.push(json!({"time":press,"pitch_delta":pitch,"reached_height_delta":delta,"agrees":pitch*delta>0.0,"source":a.source}));
                    }
                }
            }
        }
        if cursor<launches.len() {failures.push(format!("{id}: {} unintended trailing launches",launches.len()-cursor));}
        if alignment.iter().any(|offset|*offset>0.080001) {failures.push(format!("{id}: ordinary jumps exceed 80ms source-attack tolerance"));}
    }
    alignment.sort_by(f64::total_cmp);
    let pitch_agreement=if pitch_checks.is_empty(){None}else{Some(pitch_checks.iter().filter(|r|r["agrees"]==true).count() as f64/pitch_checks.len() as f64)};
    if pitch_agreement.is_some_and(|fraction|fraction<0.85) {failures.push(format!("{id}: fewer than 85% of confident pitch turns agree with reached elevation"));}
    let six=fingerprints(&sequence,6);let twelve=fingerprints(&sequence,12);let eight=fingerprints(&sequence,8);
    let repeated_six:Vec<_>=six.iter().filter(|(_,positions)|positions.len()>2).collect();
    let repeated_twelve:Vec<_>=twelve.iter().filter(|(_,positions)|positions.len()>1).collect();
    let shared:Vec<_>=eight.keys().filter(|key|others.contains(*key)).cloned().collect();
    let similar=near_repeats(&sequence);
    let shared_similar:Vec<_>=sequence.windows(16).enumerate().filter_map(|(i,w)|{
        others.iter().filter_map(|other|{
            let tokens:Vec<_>=other.split('|').map(str::to_string).collect();
            (tokens.len()==16).then(||similarity(w,&tokens))
        }).filter(|score|*score>=0.85).max_by(f64::total_cmp).map(|score|(i,score))
    }).collect();
    if !repeated_six.is_empty() {failures.push(format!("{id}: {} six-action sequences occur more than twice",repeated_six.len()));}
    if !repeated_twelve.is_empty() {failures.push(format!("{id}: {} repeated twelve-action sequences",repeated_twelve.len()));}
    if !shared.is_empty() {failures.push(format!("{id}: {} cross-song eight-action sequences",shared.len()));}
    if !similar.is_empty() {failures.push(format!("{id}: {} similar non-overlapping sixteen-action sequences",similar.len()));}
    if !shared_similar.is_empty() {failures.push(format!("{id}: {} similar cross-song sixteen-action sequences",shared_similar.len()));}
    json!({"version":1,"passed":failures.is_empty(),"failures":failures,"method":"ordinary button replay; reached support elevations, not roof rows or jump height",
        "intense_windows":windows,"repeated_six":repeated_six,"repeated_twelve":repeated_twelve,
        "shared_eight":shared,"similar_sixteen":similar,"shared_similar_sixteen":shared_similar,
        "source_alignment_seconds":{"count":alignment.len(),"median":alignment.get(alignment.len()/2),"p95":alignment.get(((alignment.len().saturating_sub(1)) as f64*0.95).round() as usize)},
        "pitch_agreement":pitch_agreement,"confident_pitch_checks":pitch_checks,"landing_count":rows.len(),"supports":rows})
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn similarity_threshold_has_exact_integer_boundary() {
        let a=vec!["0:4:press".to_string();16];let mut b=a.clone();
        b[0]="1:4:press".into();b[1]="1:4:press".into();
        for token in b.iter_mut().take(7).skip(2) {*token="0:5:press".into();}
        assert!(similarity(&a,&b)>=0.85);
        b[7]="0:4:hold".into();assert!(similarity(&a,&b)<0.85);
    }
    #[test]
    fn near_copy_with_cosmetic_changes_is_rejected() {
        let phrase:Vec<_>=(0..16).map(|i|format!("{}:{}:press:spike:1",i%5-2,i%3+3)).collect();
        let mut sequence=phrase.clone();
        sequence.extend(phrase.iter().map(|s|s.replace("spike:1","spike:3")));
        assert!(near_repeats(&sequence).iter().any(|(a,b,_)|*a==0&&*b==16));
    }
    #[test]
    fn overlapping_windows_are_not_repeat_findings() {
        let sequence:Vec<_>=(0..24).map(|i|format!("{}:{}:press",i%5-2,i%3+3)).collect();
        assert!(near_repeats(&sequence).is_empty());
    }
    #[test]
    fn flat_intense_passage_cannot_pass_by_having_many_jumps() {
        let rows:Vec<_>=(0..20).map(|i|Landing{time:i as f64*0.4,x:i as f64*4.8,height:0.0,action:"landing".into()}).collect();
        let (span,travel,flat)=elevation(&rows,0.0,6.0);
        assert_eq!(span,0.0);assert_eq!(travel,0.0);assert!(flat>5.0);
    }
    #[test]
    fn airborne_transfer_is_not_a_long_flat_platform() {
        let rows=vec![
            Landing{time:0.0,x:0.0,height:10.0,action:"landing".into()},
            Landing{time:0.5,x:2.0,height:10.0,action:"fresh_jump".into()},
            Landing{time:2.0,x:12.0,height:4.0,action:"landing".into()},
            Landing{time:2.2,x:13.0,height:4.0,action:"fresh_jump".into()}];
        let (span,travel,flat)=elevation(&rows,0.0,2.1);
        assert_eq!(span,6.0);assert_eq!(travel,6.0);assert!(flat<=0.5);
    }
    #[test]
    fn translated_repeated_topology_is_detected() {
        let make=|base:f64| (0..25).map(|i|Landing{time:i as f64*0.4,x:i as f64*4.8,
            height:base+[0.0,1.0,2.0,1.0][i%4],action:"jump".into()}).collect::<Vec<_>>();
        let a=tokens(&make(0.0),0.4);let b=tokens(&make(12.0),0.4);
        assert_eq!(a,b);
        assert!(fingerprints(&a,12).values().any(|positions|positions.len()>1));
    }
    #[test]
    fn landing_travel_does_not_count_unused_ceiling_blocks() {
        let rows=vec![Landing{time:0.0,x:0.0,height:0.0,action:"jump".into()},Landing{time:1.0,x:5.0,height:2.0,action:"landing".into()},Landing{time:2.0,x:10.0,height:0.0,action:"landing".into()}];
        assert_eq!(elevation(&rows,0.0,3.0),(2.0,4.0,1.0));
    }
}
