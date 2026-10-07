//! Offline rule-based course authoring. Audio analysis remains an input, never
//! a claim that a deterministic replay establishes musical or visual quality.
pub mod evidence;
pub mod grammar;
pub mod audit;

use evidence::Evidence;
use grammar::{History, Hop, Rules};
use serde_json::{json, Value};
use std::collections::HashSet;

fn num(v: &Value, key: &str) -> f64 { v[key].as_f64().unwrap_or(0.0) }
fn height(v: &Value, key: &str) -> i16 { v[key].as_u64().unwrap_or(0) as i16 }
fn run_end(s: &Value, period: f64) -> f64 {
    if let Some(end)=s["exit_beat"].as_f64() {return end*period;}
    let mut at=num(s,"beat")*period;
    let mut h=height(s,"entry_height");
    if let Some(rows)=s["heights"].as_array() {
        for (i,v) in rows.iter().enumerate() {
            let target=v.as_u64().unwrap() as i16;
            at+=((1.0+(1.0-(target-h) as f64/2.02).sqrt())/2.0
                +s["rests"][i].as_f64().unwrap_or(0.0))*period;
            h=target;
        }
    }
    at
}

struct Anchor { start:f64,end:f64,entry:i16,exit:i16,range:(u8,u8),parts:Vec<Value> }

fn anchors(track:&Value,period:f64)->Vec<Anchor> {
    let segments=track["level_segments"].as_array().unwrap();
    let mut result=Vec::new();
    for (index,s) in segments.iter().enumerate() {
        if !matches!(s["type"].as_str(),Some("dash_route"|"gravity_route")) {continue;}
        let mut parts=vec![s.clone()];
        if s["type"]=="gravity_route" {
            if let Some(next)=segments.get(index+1).filter(|s|s["capture_sequence"]==true) {parts.push(next.clone());}
        }
        let entry=height(s,"entry_height");
        let last=parts.last().unwrap();
        let exit=last["heights"].as_array().and_then(|a|a.last()).and_then(Value::as_u64)
            .map(|v|v as i16).unwrap_or_else(||height(last,"height"));
        let mut all=vec![entry];
        for part in &parts {
            all.push(height(part,"entry_height"));
            if matches!(part["type"].as_str(),Some("dash_route"|"gravity_route")) {all.push(height(part,"height"));}
            if let Some(h)=part["heights"].as_array() {all.extend(h.iter().filter_map(Value::as_u64).map(|v|v as i16));}
        }
        let lo=*all.iter().min().unwrap();let hi=*all.iter().max().unwrap();
        result.push(Anchor{start:num(s,"beat")*period,end:run_end(last,period),entry,exit,
            range:if s["type"]=="dash_route" {(2,22)}else{((entry-lo).max(0) as u8,(24-hi+entry).min(24) as u8)},parts});
    }
    result
}

fn emit(hops:&[Hop],e:&Evidence,segments:&mut Vec<Value>,scenes:&mut Vec<Value>,contours:&mut Vec<Value>,id:&str,closing:bool) {
    for chunk in hops.chunks(24) {
        let first=chunk.first().unwrap();let last=chunk.last().unwrap();
        let index=scenes.len()+1;
        let end_release=(closing && std::ptr::eq(last,hops.last().unwrap()))
            .then_some((last.press+0.65*(last.landing-last.press))/e.period);
        let mut run=json!({"type":"terrain_run","beat":first.press/e.period,"entry_height":first.from,
            "heights":chunk.iter().map(|h|h.to).collect::<Vec<_>>(),
            "rests":chunk.iter().map(|h|h.rest).collect::<Vec<_>>(),
            "widths":chunk.iter().map(|h|h.width).collect::<Vec<_>>(),
            "rest_types":chunk.iter().map(|h|h.kind.clone()).collect::<Vec<_>>(),
            "spike_counts":chunk.iter().map(|h|h.count).collect::<Vec<_>>(),
            "gate_releases":chunk.iter().map(|h|if h.held {None}else{Some((h.press+0.65*(h.landing-h.press))/e.period)}).collect::<Vec<_>>(),
            "continuation":true,"exit_beat":last.next_press/e.period,"count":1,
            "scene":format!("{id}-grammar-{index}"),"timing_source":if e.attacks.iter().any(|t|(*t-first.press).abs()<1e-5) {"physical_continuation"}else{"dash_landing"},
            "grammar_rules":chunk.iter().map(|h|h.rule.clone()).collect::<Vec<_>>(),
            "instrument":"recording/MIDI constrained grammar"});
        if let Some(release)=end_release {run["ceiling_beat"]=json!(release);}
        if end_release.is_some() {run["continuation"]=json!(false);run.as_object_mut().unwrap().remove("exit_beat");}
        else if last.rest==0.0 {run.as_object_mut().unwrap().remove("exit_beat");}
        segments.push(run);
        let gates:Vec<_>=chunk.iter().filter(|h|!h.held).map(|h|json!({"landing":h.landing/e.period,
            "release":(h.press+0.65*(h.landing-h.press))/e.period,"resume":h.next_press/e.period,
            "height":h.to,"type":h.kind,"timing_source":if e.attacks.iter().any(|t|(*t-h.next_press).abs()<1e-5) {"recording_or_validated_midi"}else{"quarter_beat_grid"}})).collect();
        scenes.push(json!({"entry":first.press/e.period,"landing":last.landing/e.period,
            "hops":chunk.len(),"peak":chunk.iter().map(|h|h.to).max().unwrap(),
            "gates":gates,"entry_height":first.from,"continuation":true,"family":"parametric_grammar"}));
        for h in chunk {
            let a=e.at(h.press);let b=e.at(h.next_press);
            let same_frame=e.orientation(h.press)==e.orientation(h.next_press)&&h.rule!="tower_approach"&&h.rule!="goal_landing";
            contours.push(json!({"time":h.press,"pitch_start":a.pitch,"pitch_end":if same_frame {b.pitch}else{None},
                "height_start":h.from,"height_end":h.to,"start_source":a.source,"end_source":b.source,
                "visual_height_start":h.from as f64*e.orientation(h.press),"visual_height_end":h.to as f64*e.orientation(h.next_press),
                "source":if matches!(h.rule.as_str(),"tower_approach"|"goal_landing") {"terminal tower connection; source direction comparison exempt"}else if !same_frame {"gravity transition; source direction comparison exempt"}else if a.source==b.source {a.source}else{"mixed evidence; level traversal"},
                "next_press_time":h.next_press,"planned_rest_beats":h.rest,"rule":h.rule,
                "flat_reason":if h.from==h.to {Some("sustained register or constrained source attack")}else{None},
                "cue_source":"recording and region-validated melody attacks"}));
        }
    }
}

fn translate(part:&mut Value,delta:i16) {
    for key in ["entry_height","height"] {
        if part[key].is_number() {part[key]=json!(height(part,key)+delta);}
    }
    if let Some(rows)=part.get_mut("heights").and_then(Value::as_array_mut) {
        for h in rows {*h=json!(h.as_i64().unwrap()+delta as i64);}
    }
}

/// Mechanics with independently measured musical cues are hard constraints.
/// Terrain between them is regenerated as a connected course; anchor heights
/// are free within reachability bounds, not inherited floor reset points.
pub fn generate(track:&Value,analysis:&Value,seed:u64,forbidden:&HashSet<String>)->Result<(Value,History),String> {
    let id=track["id"].as_str().ok_or("track id absent")?;
    let rules=Rules::for_song(id)?;let e=Evidence::new(track,analysis)?;
    let anchors=anchors(track,e.period);
    let mut segments=Vec::new();let mut scenes=Vec::new();let mut contours=Vec::new();let mut history=History::default();
    let mut time=e.attacks.iter().copied().find(|t|*t>=0.10*e.period).ok_or("opening attack absent")?;
    let mut h=0u8;
    for (index,anchor) in anchors.iter().enumerate() {
        eprintln!("{id}: grammar boundary {index}/{} at {:.3}..{:.3}",anchors.len(),time,anchor.start);
        if let Ok(path)=std::env::var("TOWER_AUTHOR_DIAGNOSTIC") {
            std::fs::write(path,serde_json::to_vec_pretty(&json!({"id":id,"boundary":index,"start":time,"end":anchor.start,"entry":h,"hops":history.hops})).map_err(|e|e.to_string())?).map_err(|e|e.to_string())?;
        }
        if anchor.start-time>0.45*e.period {
            let path=grammar::phrase(&e,rules,time,anchor.start,h,anchor.range,&history,forbidden,seed.wrapping_add(index as u64))?;
            h=path.last().unwrap().to;
            emit(&path,&e,&mut segments,&mut scenes,&mut contours,id,false);
            history.append(&path,e.period);
        } else if anchor.start-time>1e-7 {return Err(format!("unfilled anchor {index} at{time:.3}..{:.3}",anchor.start));}
        let delta=h as i16-anchor.entry;
        let mut block=anchor.parts.clone();
        if block[0]["type"]=="dash_route" && block[0]["dash_variant"]!="bounce" {
            let desired=(e.target(anchor.end,rules.low_register,rules.high_register)-h as f64).round() as i16;
            let a=e.at(anchor.start);let z=e.at(anchor.end);
            let direction=if a.source==z.source {a.pitch.zip(z.pitch).map(|(a,z)|(z-a)*e.orientation(anchor.start))}else{None};
            let mut step=desired.clamp(-4,1);
            if direction.is_some_and(|d|d>0.5) {step=step.max(1);}
            if direction.is_some_and(|d|d< -0.5) {step=step.min(-1);}
            let plateau_origin=history.hops.iter().rev().find(|h|h.from!=h.to).map(|h|h.landing).unwrap_or(time);
            let active=(0..32).filter(|i|e.hot(anchor.end-8.0*e.period+*i as f64/32.0*8.0*e.period)).count()>=12;
            let transfer_required=active && anchor.end-plateau_origin>rules.hot_flat_beats*e.period;
            if step==0 && transfer_required {
                step=if z.energy+z.lead*0.2>=a.energy+a.lead*0.2 {1}else{-1};
            }
            step=step.max(-(h as i16)).min(24-h as i16);
            while step<0 && ((1.0+(1.0-step as f64/2.02).sqrt())/2.0+0.7)*e.period>anchor.end-anchor.start {step+=1;}
            if step==0 && transfer_required && h<24 {step=1;}
            let target=(h as i16+step) as u8;
            block[0]["entry_height"]=json!(h);block[0]["height"]=json!(target);
            block[0]["dash_variant"]=json!(if step>0 {"transfer"}else if step<0 {"underpass"}else{"gap"});
            let flight=((1.0+(1.0-step as f64/2.02).sqrt())/2.0+0.5)*e.period;
            history.append(&[Hop{press:anchor.start,landing:anchor.start+flight,next_press:anchor.end,
                from:h,to:target,rest:((anchor.end-anchor.start-flight)/e.period).max(0.0),
                rule:"recurring_percussion_transfer".into(),kind:"dash".into(),count:1,width:1,held:false}],e.period);
            h=target;
        } else {
            for part in &mut block {translate(part,delta);}
            let mut at=anchor.start;let mut entry=h;
            for part in &block {
                if part["type"]=="terrain_run" {
                    if let Some(heights)=part["heights"].as_array() {
                        for (i,target) in heights.iter().enumerate() {
                            let target=target.as_u64().unwrap() as u8;let flight=(1.0+(1.0-(target as f64-entry as f64)/2.02).sqrt())/2.0*e.period;
                            let rest=part["rests"][i].as_f64().unwrap_or(0.0);let next=at+flight+rest*e.period;
                            history.append(&[Hop{press:at,landing:at+flight,next_press:next,from:entry,to:target,rest,
                                rule:"capture_rebound".into(),kind:part["rest_types"][i].as_str().unwrap_or("gap").into(),count:part["spike_counts"][i].as_u64().unwrap_or(1) as u8,width:1,held:rest==0.0}],e.period);
                            at=next;entry=target;
                        }
                    }
                } else {
                    let target=height(part,"height") as u8;let next=num(part,"exit_beat")*e.period;
                    history.append(&[Hop{press:at,landing:next-0.08*e.period,next_press:next,from:entry,to:target,rest:0.08,
                        rule:"reserved_mechanic".into(),kind:part["type"].as_str().unwrap().into(),count:1,width:1,held:false}],e.period);
                    at=next;entry=target;
                }
            }
            h=(anchor.exit+delta) as u8;
        }
        segments.extend(block);time=anchor.end;
    }
    let end=e.attacks.iter().copied().filter(|t|*t<e.duration-2.0*e.period).last().ok_or("closing attack absent")?;
    let mut path=grammar::phrase(&e,rules,time,end,h,(0,0),&history,forbidden,seed.wrapping_add(1000))?;
    if path.len()<2 {return Err("closing grammar needs two supported landings".into());}
    let last=path.last_mut().unwrap();last.rest=0.0;last.next_press=last.landing;last.held=true;last.kind="ceiling".into();last.rule="goal_landing".into();
    emit(&path,&e,&mut segments,&mut scenes,&mut contours,id,true);history.append(&path,e.period);
    let mut result=track.clone();
    result["level_segments"]=json!(segments);result["contour_evidence"]=json!(contours);
    result["score_source"]=json!("Rust rule-based search over recording/MIDI attacks, continuous whole-cell terrain and ordinary-input replay gates");
    result["grid_actions"]=json!([]);result["grid_cues"]=json!([]);result["gravity_wells"]=json!([]);
    result["course_metrics"]["scenes"]=json!(scenes);
    result["course_metrics"]["terrain_runs"]=json!(segments.iter().filter(|s|s["type"]=="terrain_run").count());
    result["course_metrics"]["terrain_hops"]=json!(segments.iter().filter_map(|s|s["heights"].as_array()).map(Vec::len).sum::<usize>());
    result["course_metrics"]["continuation_scenes"]=json!(segments.iter().filter(|s|s["type"]=="terrain_run"&&s["continuation"]==true).count());
    result["course_metrics"]["raised_release_gates"]=json!(history.hops.iter().filter(|h|!h.held&&h.to>0).count());
    result["course_metrics"]["elevated_spike_walks"]=json!(history.hops.iter().filter(|h|h.kind=="spike"&&h.to>0).count());
    if let Some(pockets)=result["course_metrics"]["dash_pockets"].as_array_mut() {
        for pocket in pockets {
            if let Some(hop)=history.hops.iter().find(|h|(h.press-num(pocket,"jump")).abs()<1e-5) {
                pocket["entry_height"]=json!(hop.from);pocket["landing_height"]=json!(hop.to);
                if hop.kind=="dash" {pocket["landing_time"]=json!(hop.landing);}
            }
        }
    }
    if let Some(routes)=result["course_metrics"]["gravity_routes"].as_array_mut() {
        for receipt in routes {
            if let Some(segment)=segments.iter().find(|s|s["type"]=="gravity_route"&&(num(s,"beat")*e.period-num(receipt,"launch")).abs()<1e-5) {
                receipt["entry_height"]=segment["entry_height"].clone();receipt["target_height"]=segment["height"].clone();
            }
        }
    }
    result["course_metrics"]["max_height_tiles"]=json!(segments.iter().flat_map(|s|s["heights"].as_array().into_iter().flatten()).filter_map(Value::as_u64).max().unwrap_or(0));
    result["generation_rules"]=json!({"version":1,"seed":seed,"profile":rules,
        "planner":"bounded DAG over actual recording/MIDI attacks and full-cell reachable landings",
        "height_state":"continuous across phrase and mechanic boundaries",
        "novelty":"six-hop repeats limited; twelve-hop and cross-song eight-hop sequences rejected",
        "cliffs_locked":true});
    result["generation_hops"]=json!(history.hops);
    Ok((result,history))
}
