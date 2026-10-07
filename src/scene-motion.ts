export interface CameraBody {x:number;y:number;vy:number;support_y:number;tile_size:number;objects:{kind:string;x:number;y:number;w:number;h:number}[]}
export interface CameraSpring {position:number;velocity:number}
export interface TowerPose {x:number;y:number;w:number;h:number}
export function gravityTransition(route:number,markers:number[],tile:number,speed:number,beat:number):{blend:number;warning:number;marker?:number;next:number}{
  let blend=0,warning=0,next=1,marker:number|undefined;
  const span=speed*beat;
  for(let i=0;i<markers.length;i++){
    const distance=markers[i]*tile-route;
    const phase=Math.max(0,Math.min(1,.5-distance/span));
    const ease=phase*phase*(3-2*phase);
    if(phase>0)blend=i%2===0?ease:1-ease;
    if(distance>=-span*.5&&distance<=span*2){marker=markers[i]*tile;warning=Math.max(0,1-Math.abs(distance/span)/2);next=i%2===0?1:0;break;}
  }
  return {blend,warning,marker,next};
}
// A portal wipe preserves rightward travel and full-sized cells. Regions are
// disjoint; no pixel contains two competing versions of the foreground.
export function gravityBoundary(width:number,blend:number,anchor=width*.28):number {
  return blend<=.5?anchor*blend*2:anchor+(width-anchor)*(blend-.5)*2;
}
export function gravityProject(x:number,y:number,width:number,blend:number,anchor=width*.28):{x:number;y:number}{
  return {x,y:blend>=1||(blend>0&&x<=gravityBoundary(width,blend,anchor))?280-y:y};
}
export function gravityPlayerProject(x:number,y:number,blend:number):{x:number;y:number}{
  const crossing=Math.max(0,Math.min(1,(blend-.32)/.36));
  const ease=crossing*crossing*(3-2*crossing);
  return {x,y:y+(280-2*y)*ease};
}
export function smoothAudioOutput(previous:number,observed:number,elapsed:number):number {
  if(!Number.isFinite(previous)||elapsed>.25)return observed;
  const step=Math.max(0,elapsed),predicted=previous+step,error=observed-predicted;
  const correction=error*(1-Math.exp(-step/2));
  return predicted+Math.max(-step*.003,Math.min(step*.003,correction));
}
export function distantTower(progress:number,width:number,duration=120,speed=313.6):TowerPose {
  const travel=Math.max(0,Math.min(1,progress));
  const entryLead=Math.max(.2,(width*.72-14)/speed);
  const offscreenAt=duration-2-entryLead;
  const exit=Math.max(0,Math.min(1,(travel*duration-(offscreenAt-2.75))/2.75));
  const ease=exit*exit*(3-2*exit);
  const height=48+90*travel**1.65+40*ease;
  const near=width*(.58+.23*travel**1.4);
  const center=near+(width+.01+height*.275-near)*ease;
  const ground=120+94*travel**1.2+48*ease;
  return {x:center-height*.275,y:ground-height,w:height*.55,h:height};
}
// Projectile hits follow continuous world travel. Collapse begins while the
// struck cells remain in the visible strip behind the player.
export function fallingTile(playerX:number,tileX:number,speed:number,tile:number,reduced=false):{age:number;drop:number;angle:number;alpha:number}{
  const age=(playerX-tileX-tile*1.55)/speed;
  const elapsed=Math.max(0,age),phase=Math.min(1,elapsed*speed/(tile*3));
  return {age,drop:elapsed*45+elapsed*elapsed*2100,angle:reduced?0:phase*phase*.55,alpha:Math.max(.2,1-elapsed*.6)};
}
const landingCache=new WeakMap<CameraBody['objects'],Map<number,CameraBody['objects']>>();

// Follow the supported route rather than every jump arc. Nearby landing
// surfaces provide gentle anticipation, while a dead zone keeps the cube visible.
export function cameraTarget(body:CameraBody):number {
  const probe=body.x+body.tile_size*2.5;
  let columns=landingCache.get(body.objects);
  if(!columns){
    columns=new Map();
    for(const object of body.objects){
      if(object.kind!=='enemy'&&object.kind!=='terrain')continue;
      for(let column=Math.floor(object.x/body.tile_size);column<Math.ceil((object.x+object.w)/body.tile_size);column++){
        const entries=columns.get(column)||[];entries.push(object);columns.set(column,entries);
      }
    }
    landingCache.set(body.objects,columns);
  }
  let ahead=body.support_y,found=false;
  for(const object of columns.get(Math.floor(probe/body.tile_size))||[]){
    const top=object.y+object.h;
    if(object.x<=probe&&object.x+object.w>probe&&top<=body.y+body.tile_size*2.05){
      ahead=found?Math.max(ahead,top):top;found=true;
    }
  }
  const route=body.support_y+(ahead-body.support_y)*.65;
  const target=Math.max(0,route-56,body.y-154);
  const descentBound=Math.max(0,body.y+Math.min(0,body.vy)*.14+36);
  return Math.min(target,descentBound);
}

export function springCamera(spring:CameraSpring,target:number,dt:number):CameraSpring {
  const step=Math.max(0,Math.min(.05,dt)),frequency=10;
  const difference=spring.position-target,decay=Math.exp(-frequency*step);
  const drift=spring.velocity+frequency*difference;
  return {position:target+(difference+drift*step)*decay,velocity:(spring.velocity-frequency*drift*step)*decay};
}

export function spotlightAim(from:{x:number;y:number},to:{x:number;y:number},cue:number,time:number,beat:number):number {
  const target=Math.atan2(to.y-from.y,to.x-from.x);
  const phase=Math.max(0,Math.min(1,(time-(cue-2*beat))/(1.75*beat)));
  const ease=phase*phase*(3-2*phase);
  return target+(1-ease)*.7;
}
