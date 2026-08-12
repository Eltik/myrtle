// Per-system TRAIL (rope) probe: dumps `probe().rope` so a ribbon that exists but draws
// nothing can be told apart from one that was never built. Usage: node ropeprobe.js <beats> [sys]
const puppeteer=require("puppeteer");
const STEP=0.033;
(async()=>{
 const b=await puppeteer.launch({headless:"new",protocolTimeout:900000,
  args:["--use-gl=swiftshader","--enable-unsafe-swiftshader","--disable-dev-shm-usage","--no-sandbox","--window-size=900,416"]});
 const p=await b.newPage(); await p.setViewport({width:900,height:416});
 await p.evaluateOnNewDocument(()=>{
   let seed=0x2f6e2b1; Math.random=()=>{seed=(seed*1664525+1013904223)>>>0;return seed/4294967296;};
   window.__vt=0; let q=[]; let id=1;
   performance.now=()=>window.__vt; Date.now=()=>1700000000000+window.__vt;
   window.requestAnimationFrame=(cb)=>{const i=id++;q.push({i,cb});return i;};
   window.cancelAnimationFrame=(i)=>{q=q.filter(x=>x.i!==i);};
   window.__pump=(ms)=>{window.__vt=ms;const d=q;q=[];for(const x of d){try{x.cb(window.__vt);}catch(e){}}return q.length;};
 });
 await p.goto("http://localhost:3000/dyntest?skel="+encodeURIComponent("/spine/DynIllust/char_1012_skadi2_iteration%232/dyn_illust_char_1012_skadi2_iteration%232.skel")+"&server=en&framing=authored",{waitUntil:"domcontentloaded",timeout:120000});
 for(let i=0;i<40;i++){await p.evaluate(()=>window.__pump(window.__vt));await new Promise(r=>setTimeout(r,250));
   if(await p.evaluate(()=>!!document.querySelector("canvas"))&&i>8)break;}
 const only=process.argv[3]!==undefined?Number(process.argv[3]):null;
 let t=0;
 for(const target of (process.argv[2]||'6').split(',').map(Number)){
   while(t<target-1e-6){t=Math.min(t+STEP,target);await p.evaluate((ms)=>window.__pump(ms),t*1000);}
   const r=await p.evaluate(()=>window.__dynProbe?window.__dynProbe():null);
   console.log(`t=${target}`);
   for(const e of (r||[])){
     if(only!==null&&e.sys!==only) continue;
     if(!e.rope&&only===null) continue;
     console.log(`  sys${e.sys} live=${e.live} painted=${Math.round(e.paintedPx)} rope=${JSON.stringify(e.rope)}`);
   }
 }
 await b.close();
})();
