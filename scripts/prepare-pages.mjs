import {cpSync,readFileSync,writeFileSync,mkdirSync,readdirSync} from 'node:fs';
import {resolve,join,extname} from 'node:path';
import assert from 'node:assert/strict';
const root=resolve(import.meta.dirname,'..');
const staging=resolve(root,'.pages-public');
mkdirSync(join(staging,'music'),{recursive:true});
cpSync(join(root,'public/tower-assets'),join(staging,'tower-assets'),{recursive:true});
const tracks=JSON.parse(readFileSync(join(root,'public/pages-data/tracks.json'),'utf8'));
assert.equal(tracks.length,4);
for(const t of tracks){assert(t.duration>0&&t.beats.length>10&&t.level_segments.length>0);assert.equal(t.audio_start,0);assert.equal(t.audio,'procedural');assert(!t.artist&&!t.audio_sha256&&!t.reference_geometry);}
// These authored inversion/moving-platform events are gameplay, not recovered geometry.
assert.equal(tracks[2].reference_events.gravity_tiles.length,2,'Violet Eclipse inversion section preserved');
assert.equal(tracks[3].reference_events.gravity_tiles.length,4,'Inferno Circuit inversion sections preserved');
writeFileSync(join(staging,'music/tracks.json'),JSON.stringify(tracks));
const forbidden=new Set(['.ogg','.mp3','.wav','.flac','.m4a','.mid','.midi']);
function audit(path){for(const entry of readdirSync(path,{withFileTypes:true})){const p=join(path,entry.name);if(entry.isDirectory())audit(p);else assert(!forbidden.has(extname(p).toLowerCase()),`Private audio must not be published: ${p}`);}}
audit(staging);
console.log('Pages assets verified: four playable courses, procedural audio, zero commercial recordings.');
