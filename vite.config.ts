import { defineConfig } from 'vite';
import { createReadStream, existsSync } from 'node:fs';
import { resolve } from 'node:path';
export default defineConfig(({mode})=>({publicDir:mode==='pages'?'.pages-public':'public',base:'./',server:{host:'127.0.0.1'},build:{target:'es2022'},plugins:[{
  name:'private-owned-reference',apply:'serve',
  configureServer(server){server.middlewares.use((request,response,next)=>{
    const route=request.url?.split('?')[0];
    const audioRoutes:Record<string,string>={fireaura:'soundtrack.ogg',fireauraxbox:'soundtrack.ogg',chaozfantasy:'soundtrack2.ogg',heaven:'soundtrack3.ogg','phazd-editor':'soundtrack4.ogg'};
    const source=Object.entries(audioRoutes).find(([id])=>route===`/__reference/${id}/audio.ogg`)?.[1];
    const file=route==='/__reference/tracks.json'?resolve('.local/original-baselines/tracks.json'):
      source?`D:/SteamLibrary/steamapps/common/TheImpossibleGame/res/${source}`:undefined;
    if(!file)return next();
    if(!existsSync(file)){response.statusCode=404;response.end();return;}
    if(request.method!=='GET'&&request.method!=='HEAD'){response.statusCode=405;response.end();return;}
    response.setHeader('Content-Type',route?.endsWith('.json')?'application/json':'audio/ogg');
    response.setHeader('Cache-Control','no-store');
    if(request.method==='HEAD'){response.end();return;}
    const stream=createReadStream(file);stream.on('error',()=>{response.statusCode=500;response.end();});
    response.on('close',()=>stream.destroy());stream.pipe(response);
  });}
}]}));
