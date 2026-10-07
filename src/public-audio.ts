/** An original electronic backing track: no samples or source-song melody. */
export function createPublicAudio(context: AudioContext, duration: number, beats: number[], tier: number): AudioBuffer {
  const rate = 22050;
  const buffer = context.createBuffer(1, Math.ceil((duration + 1) * rate), rate);
  const output = buffer.getChannelData(0);
  const tones = [110, 146.832, 164.814, 130.813];
  let random = 7411 + tier;
  const add = (at:number, length:number, sound:(t:number)=>number) => {
    const start = Math.max(0, Math.floor(at * rate));
    const count = Math.min(Math.ceil(length * rate), output.length - start);
    for(let j=0;j<count;j++) output[start+j] += sound(j/rate);
  };
  beats.forEach((beat,index) => {
    if(beat < 0 || beat > duration) return;
    add(beat,.2,t=>Math.sin(2*Math.PI*(52*t+12*(1-Math.exp(-t*32))))*Math.exp(-t*22)*.3);
    add(beat,.055,t=>{random=(Math.imul(random,1664525)+1013904223)>>>0;return (random/2147483648-1)*Math.exp(-t*85)*.055;});
    if(index%2===0) { const hz=tones[(Math.floor(index/8)+tier)%tones.length]; add(beat,.3,t=>Math.sin(2*Math.PI*hz*t)*Math.exp(-t*12)*.12); }
    if(index%4===2) add(beat,.12,t=>{random=(Math.imul(random,1664525)+1013904223)>>>0;return (random/2147483648-1)*Math.exp(-t*30)*.075;});
  });
  for(let i=0;i<output.length;i++) output[i]=Math.tanh(output[i]);
  return buffer;
}
