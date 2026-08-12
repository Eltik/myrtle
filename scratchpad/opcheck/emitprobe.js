// Live per-emitter probe for dynchar parity work.
//   node emitprobe.js <skel-url-path> [beats]
// Reports, per beat: which systems are emitting, how many particles, and where the emitter
// sits in SCREEN px. That distinguishes 'emitted nothing' from 'emitted off-frame' — the two
// causes of a missing effect that the exported JSON cannot tell apart.
const puppeteer=require("puppeteer");
const STEP=0.033;
(async()=>{
 const b=await puppeteer.launch({headless:"new",protocolTimeout:900000,
  args:["--use-gl=swiftshader","--enable-unsafe-swiftshader","--disable-dev-shm-usage","--no-sandbox","--disable-frame-rate-limit","--window-size=900,416"]});
 const p=await b.newPage(); await p.setViewport({width:900,height:416});
 await p.evaluateOnNewDocument(()=>{
   let seed=0x2f6e2b1; Math.random=()=>{seed=(seed*1664525+1013904223)>>>0;return seed/4294967296;};
   window.__vt=0; let q=[]; let id=1;
   performance.now=()=>window.__vt; Date.now=()=>1700000000000+window.__vt;
   window.requestAnimationFrame=(cb)=>{const i=id++;q.push({i,cb});return i;};
   window.cancelAnimationFrame=(i)=>{q=q.filter(x=>x.i!==i);};
   window.__pump=(ms)=>{window.__vt=ms;const d=q;q=[];for(const x of d){try{x.cb(window.__vt);}catch(e){}}return q.length;};
 });
 await p.goto("http://localhost:3000/dyntest?skel="+encodeURIComponent(process.argv[2]||"/spine/DynIllust/char_1012_skadi2_iteration%232/dyn_illust_char_1012_skadi2_iteration%232.skel")+"&server=en&framing=authored",{waitUntil:"domcontentloaded",timeout:120000});
 for(let i=0;i<40;i++){await p.evaluate(()=>window.__pump(window.__vt));await new Promise(r=>setTimeout(r,250));
   if(await p.evaluate(()=>!!document.querySelector("canvas"))&&i>8)break;}
 let t=0;
 for(const target of (process.argv[3]||'5,9,13').split(',').map(Number)){
   while(t<target-1e-6){t=Math.min(t+STEP,target);await p.evaluate((ms)=>window.__pump(ms),t*1000);}
   const r=await p.evaluate(()=>window.__dynProbe?window.__dynProbe():"NO HOOK");
   const xf=await p.evaluate(()=>window.__dynXform?window.__dynXform():null);
   const live=Array.isArray(r)?r.filter(e=>e.live>0):[];
   const dead=Array.isArray(r)?r.filter(e=>e.live===0):[];
   console.log(`t=${target}: ${Array.isArray(r)?r.length:0} emitters, ${live.length} live, ${dead.length} silent`);
   if(xf){const f=o=>o?`a=${o.a.toFixed(4)} d=${o.d.toFixed(4)} tx=${o.tx.toFixed(1)} ty=${o.ty.toFixed(1)}`:'null';
     console.log(`   sceneRoot   ${f(xf.sceneRoot)}`);
     console.log(`   particlesBg ${f(xf.particlesBg)}`);
     console.log(`   particlesFg ${f(xf.particlesFg)}`);}
   console.log('   silent sys:', dead.map(e=>e.sys).join(','));
   console.log(`   total painted ${Math.round(live.reduce((n,e)=>n+e.paintedPx,0))} px^2`);
   for(const e of live.slice(0,40)){
     const b=e.box; const bs=b?`box=(${Math.round(b.x)},${Math.round(b.y)} ${Math.round(b.w)}x${Math.round(b.h)})`:'no-box';
     const flag = e.onScreen===0 ? '  <<< NONE ON SCREEN' : '';
     console.log(`   sys${e.sys} live=${e.live} painted=${Math.round(e.paintedPx)} onScreen=${e.onScreen} ${bs}${e.paintedPx===0?'  <<< PAINTS NOTHING':''}`);
   } }
 await b.close();
})().catch(e=>{console.error(e);process.exit(1)});
