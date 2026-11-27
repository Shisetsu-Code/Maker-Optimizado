// Isolated read-only static reference server. Only the pinned local tree is served.
import {createServer} from 'node:http';
import {createReadStream} from 'node:fs';
import {stat,realpath} from 'node:fs/promises';
import path from 'node:path';
const root=await realpath(process.argv[2]||'reference/public');
const MIME={'.html':'text/html; charset=utf-8','.js':'text/javascript; charset=utf-8','.mjs':'text/javascript; charset=utf-8','.css':'text/css; charset=utf-8','.json':'application/json','.png':'image/png','.webp':'image/webp','.jpg':'image/jpeg','.jpeg':'image/jpeg','.avif':'image/avif','.gif':'image/gif','.svg':'image/svg+xml','.woff2':'font/woff2'};
createServer(async(req,res)=>{
 try{
  if(!['GET','HEAD'].includes(req.method)){res.writeHead(405);res.end();return;}
  let url=new URL(req.url,'http://127.0.0.1:8788'),name=decodeURIComponent(url.pathname);
  if(name==='/')name='/index.html';
  if(name==='/api/winners'){res.writeHead(200,{'content-type':'application/json'});res.end('{"winners":[],"demo":true}');return;}
  if(name.startsWith('/api/')){res.writeHead(404);res.end();return;}
  if(name.startsWith('/muestra/games-cartas/'))name=name.slice('/muestra'.length);
  const target=await realpath(path.resolve(root,'.'+name));if(!target.startsWith(root+path.sep))throw Error('outside root');
  const info=await stat(target);if(!info.isFile())throw Error('not a file');
  let ext=path.extname(target).toLowerCase();
  if(!MIME[ext]){const m=target.match(/\.(png|jpe?g|avif|webp|gif|css|woff2)\.[a-f0-9]{12}$/i);if(m)ext='.'+m[1].toLowerCase();}
  res.writeHead(200,{'content-type':MIME[ext]||'application/octet-stream','content-length':info.size,'cache-control':'no-store','referrer-policy':'no-referrer'});
  if(req.method==='HEAD')res.end();else createReadStream(target).pipe(res);
 }catch{res.writeHead(404);res.end();}
}).listen(8788,'127.0.0.1');
