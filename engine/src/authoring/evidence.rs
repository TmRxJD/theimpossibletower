use serde_json::Value;

#[derive(Clone, Copy, Debug)]
pub struct Feature {
    pub pitch: Option<f64>,
    pub source: &'static str,
    pub lead: f64,
    pub energy: f64,
    pub density: f64,
    pub recording_density: f64,
    pub sustained: bool,
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    #[test]
    fn dense_recording_attacks_stay_intense_after_midi_coverage_ends() {
        let track=json!({"bpm":120,"duration":4.0,"audio_sha256":"fixture","audio_start":0,
            "melody_notes":[{"start":0.0,"end":0.5,"pitch":67}],"level_segments":[]});
        let onsets=[2.0,2.3,2.6,2.9,3.2,3.5].map(|time|json!({"time":time,"lead":0.3,"low":0.1}));
        let analysis=json!({"source_sha256":"fixture","audio_start":0,"envelope_step_seconds":0.1,
            "onsets":onsets,
            "timeline":{"lead_attack":vec![0.1;40],"energy":vec![0.2;40],"harmonic_register":vec![60.0;40],"harmonic_salience":vec![0.6;40]}});
        let evidence=Evidence::new(&track,&analysis).unwrap();
        assert_eq!(evidence.at(2.7).density,0.0);
        assert!(evidence.at(2.7).recording_density>=1.0);
        assert!(evidence.hot(2.7));
    }
}

impl Feature {
    pub fn hot(self,lead_cut:f64)->bool {
        self.density>=2.0 || self.recording_density>=1.0 || self.lead>=lead_cut.max(0.15)
    }
}

pub struct Evidence {
    pub period: f64,
    pub duration: f64,
    pub attacks: Vec<f64>,
    lead_attacks: Vec<f64>,
    notes: Vec<(f64, f64, f64)>,
    lead: Vec<f64>,
    energy: Vec<f64>,
    register: Vec<f64>,
    salience: Vec<f64>,
    step: f64,
    pub pitch_low: f64,
    pub pitch_high: f64,
    pub lead_cut: f64,
    register_low: f64,
    register_high: f64,
    gravity_tiles: Vec<f64>,
    dashes: Vec<f64>,
}

pub fn quantile(values: &[f64], fraction: f64) -> f64 {
    if values.is_empty() { return 0.0; }
    let mut rows = values.to_vec();
    rows.sort_by(f64::total_cmp);
    rows[((rows.len() - 1) as f64 * fraction).round() as usize]
}

impl Evidence {
    pub fn new(track: &Value, analysis: &Value) -> Result<Self, String> {
        let period = 60.0 / track["bpm"].as_f64().ok_or("BPM absent")?;
        let duration = track["duration"].as_f64().ok_or("duration absent")?;
        if !period.is_finite() || period<=0.0 || !duration.is_finite() || duration<=0.0 {return Err("invalid musical clock".into());}
        if track["audio_sha256"]!=analysis["source_sha256"] || track["audio_start"]!=analysis["audio_start"] {return Err("analysis does not match the selected audio excerpt".into());}
        let notes: Vec<_> = track["melody_notes"].as_array().ok_or("validated melody notes absent")?.iter()
            .filter_map(|n| Some((n["start"].as_f64()?, n["end"].as_f64()?, n["pitch"].as_f64()?)))
            .collect();
        let mut attacks: Vec<_> = analysis["onsets"].as_array().ok_or("recording attacks absent")?.iter()
            .filter_map(|n| n["time"].as_f64()).chain(notes.iter().map(|n| n.0))
            .filter(|t| t.is_finite() && *t >= 0.0 && *t < duration).collect();
        attacks.sort_by(f64::total_cmp);
        attacks.dedup_by(|a, b| (*a - *b).abs() < 0.001);
        let lead_attacks=analysis["onsets"].as_array().unwrap().iter().filter(|onset|
            onset["lead"].as_f64().unwrap_or(0.0)>=0.15
            && onset["lead"].as_f64().unwrap_or(0.0)>=onset["low"].as_f64().unwrap_or(0.0))
            .filter_map(|onset|onset["time"].as_f64()).collect();
        let timeline = &analysis["timeline"];
        let read = |key: &str| -> Result<Vec<f64>, String> {
            timeline[key].as_array().ok_or(format!("timeline {key} absent"))?.iter()
                .map(|v| v.as_f64().filter(|v| v.is_finite()).ok_or(format!("invalid {key}"))).collect()
        };
        let lead = read("lead_attack")?;
        let step=analysis["envelope_step_seconds"].as_f64().filter(|s|s.is_finite()&&*s>0.0).ok_or("invalid analysis step")?;
        for key in ["energy","harmonic_register","harmonic_salience"] {
            if read(key)?.len()!=lead.len() {return Err(format!("timeline {key} length mismatch"));}
        }
        if lead.len() as f64*step<duration-step {return Err("audio evidence ends before the course".into());}
        let pitches: Vec<_> = notes.iter().map(|n| n.2).collect();
        let register=read("harmonic_register")?;let salience=read("harmonic_salience")?;
        let reliable_register:Vec<_>=register.iter().zip(&salience).filter(|(_,s)|**s>=0.45).map(|(r,_)|*r).collect();
        Ok(Self { period, duration, attacks, lead_attacks, notes, energy: read("energy")?,
            register, salience,
            step,
            pitch_low: quantile(&pitches, 0.1), pitch_high: quantile(&pitches, 0.9),
            lead_cut: quantile(&lead, 0.8), register_low:quantile(&reliable_register,0.1),register_high:quantile(&reliable_register,0.9),
            gravity_tiles:track["reference_events"]["gravity_tiles"].as_array().into_iter().flatten().filter_map(Value::as_f64).collect(),
            dashes:track["level_segments"].as_array().into_iter().flatten().filter(|s|matches!(s["type"].as_str(),Some("dash_route"|"gravity_route")))
                .flat_map(|s|[s["dash_beat"].as_f64(),s["chain_dash_beat"].as_f64()]).flatten().map(|b|b*period).collect(), lead })
    }

    fn sample(&self, values: &[f64], time: f64) -> f64 {
        values[((time / self.step).round().max(0.0) as usize).min(values.len()-1)]
    }

    pub fn at(&self, time: f64) -> Feature {
        let mut pitches: Vec<_> = self.notes.iter().filter(|n|
            (n.0 - time).abs() < self.period * 0.15 || n.0 <= time && time < n.1)
            .map(|n| n.2).collect();
        pitches.sort_by(f64::total_cmp);
        let midi = !pitches.is_empty();
        let pitch = if pitches.is_empty() {
            // A separated melody is not available in every recording region.
            // This estimate remains labelled as recording register, not MIDI.
            (self.sample(&self.salience, time) >= 0.45).then(|| self.sample(&self.register, time))
        } else { Some(pitches[pitches.len() / 2]) };
        let notes = self.notes.iter().filter(|n| (n.0 - time).abs() < 2.0 * self.period).count();
        Feature { pitch, source: if midi { "region-validated MIDI" } else if pitch.is_some() { "measured harmonic-register estimate" } else { "recording attack-energy contour" }, lead: self.sample(&self.lead, time), energy: self.sample(&self.energy, time),
            density: notes as f64 / 4.0,
            recording_density:self.lead_attacks.iter().filter(|at|(**at-time).abs()<2.0*self.period).count() as f64/4.0,
            sustained: self.notes.iter().any(|n| n.0 <= time && time < n.1 && n.1 - n.0 > 1.2 * self.period) }
    }

    pub fn hot(&self, time: f64) -> bool {
        self.at(time).hot(self.lead_cut)
    }

    pub fn target(&self, time: f64, low: f64, high: f64) -> f64 {
        let f = self.at(time);
        let (lo,hi)=if f.source=="region-validated MIDI" {(self.pitch_low,self.pitch_high)}else{(self.register_low,self.register_high)};
        let fraction = f.pitch.map(|pitch| ((pitch - lo) /
            (hi - lo).max(6.0)).clamp(0.0, 1.0))
            .unwrap_or_else(|| f.energy.clamp(0.0, 1.0));
        low + (if self.orientation(time)>0.0 {fraction}else{1.0-fraction}) * (high - low)
    }

    pub fn pitch_confidence(&self,time:f64)->f64 {
        if self.at(time).source=="region-validated MIDI" {1.0}else{self.sample(&self.salience,time)}
    }

    pub fn orientation(&self,time:f64)->f64 {
        let route=time*4.8/self.period+self.dashes.iter().map(|dash|{
            let f=((time-dash)/(0.5*self.period)).clamp(0.0,1.0);3.0*f*f*(3.0-2.0*f)
        }).sum::<f64>();
        if self.gravity_tiles.iter().filter(|tile|route>=**tile).count()%2==0 {1.0}else{-1.0}
    }
}
