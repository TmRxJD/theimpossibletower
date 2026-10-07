export interface PlatformObject {kind:string;x:number;y:number;w:number;h:number}
export interface PlatformSurface {x:number;top:number;w:number;h:number;tiles:number}

// The floor and raised landing objects can occupy the same row. Their union
// is a single visual surface; resetting a pattern for each collider doubled
// enemy outlines and cut the last tile at fractional landing coordinates.
export function platformSurfaces(objects:PlatformObject[],tile:number):PlatformSurface[]{
  const rows=new Map<string,PlatformSurface[]>();
  for(const o of objects){
    if(!['terrain','enemy','ceiling'].includes(o.kind))continue;
    const top=o.y+o.h,h=o.h||tile,key=`${top.toFixed(6)}:${h.toFixed(6)}`;
    const row=rows.get(key)||[];
    row.push({x:o.x,top,w:o.w,h,tiles:0});rows.set(key,row);
  }
  const surfaces:PlatformSurface[]=[];
  for(const row of rows.values()){
    row.sort((a,b)=>a.x-b.x);let current:PlatformSurface|undefined;
    for(const next of row){
      if(current&&next.x<=current.x+current.w+.00001)current.w=Math.max(current.x+current.w,next.x+next.w)-current.x;
      else{current={...next};surfaces.push(current);}
    }
  }
  for(const s of surfaces)s.tiles=Math.max(1,Math.round(s.w/tile));
  return surfaces;
}

export function platformTiles(objects:PlatformObject[],tile:number):PlatformSurface[]{
  const cells=new Map<string,PlatformSurface>();
  for(const object of objects){
    if(!['terrain','enemy','ceiling'].includes(object.kind))continue;
    const left=Math.round(object.x/tile),right=Math.max(left+1,Math.round((object.x+object.w)/tile));
    const bottom=object.kind==='terrain'?-1:Math.round(object.y/tile);
    const top=object.kind==='terrain'?0:Math.max(bottom+1,Math.round((object.y+object.h)/tile));
    for(let column=left;column<right;column++)for(let row=bottom;row<top;row++){
      cells.set(`${column}:${row}`,{x:column*tile,top:(row+1)*tile,w:tile,h:tile,tiles:1});
    }
  }
  return [...cells.values()];
}
