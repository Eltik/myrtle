// Per-slot visibility over time, via the `__dynSlots` DEV hook.
//   node slotprobe.js <skel-url-path> <beats> [name-filter]
const puppeteer = require("puppeteer");
const STEP = 0.033;
(async () => {
  const b = await puppeteer.launch({ headless: "new", protocolTimeout: 900000,
    args: ["--use-gl=swiftshader","--enable-unsafe-swiftshader","--disable-dev-shm-usage","--no-sandbox","--disable-frame-rate-limit","--window-size=900,416"] });
  const p = await b.newPage(); await p.setViewport({ width: 900, height: 416 });
  await p.evaluateOnNewDocument(() => {
    let seed = 0x2f6e2b1; Math.random = () => { seed = (seed*1664525+1013904223)>>>0; return seed/4294967296; };
    window.__vt = 0; let q = []; let id = 1;
    performance.now = () => window.__vt; Date.now = () => 1700000000000 + window.__vt;
    window.requestAnimationFrame = (cb) => { const i = id++; q.push({i,cb}); return i; };
    window.cancelAnimationFrame = (i) => { q = q.filter(x => x.i !== i); };
    window.__pump = (ms) => { window.__vt = ms; const d = q; q = []; for (const x of d) { try { x.cb(window.__vt); } catch (e) {} } return q.length; };
  });
  const skel = process.argv[2];
  await p.goto("http://localhost:3000/dyntest?skel=" + encodeURIComponent(skel) + "&server=en&framing=authored", { waitUntil: "domcontentloaded", timeout: 120000 });
  for (let i = 0; i < 40; i++) { await p.evaluate(() => window.__pump(window.__vt)); await new Promise(r => setTimeout(r, 250));
    if ((await p.evaluate(() => !!document.querySelector("canvas"))) && i > 8) break; }
  const filt = (process.argv[4] || "").toLowerCase();
  let t = 0;
  for (const target of (process.argv[3] || "2,8,11,14").split(",").map(Number)) {
    while (t < target - 1e-6) { t = Math.min(t + STEP, target); await p.evaluate(ms => window.__pump(ms), t*1000); }
    const r = await p.evaluate(() => (window.__dynSlots ? window.__dynSlots() : "NO HOOK"));
    if (typeof r === "string") { console.log(`t=${target}: ${r}`); continue; }
    const rows = r.filter(x => !filt || (x.slot||"").toLowerCase().includes(filt));
    console.log(`\n=== t=${target}  (${rows.length} of ${r.length} slots)`);
    for (const x of rows) {
      const bx = x.box ? `x=${x.box.x.toFixed(0)}..${(x.box.x+x.box.w).toFixed(0)} y=${x.box.y.toFixed(0)}..${(x.box.y+x.box.h).toFixed(0)}` : "no box";
      console.log(`   ${String(x.i).padStart(3)} ${String(x.slot).padEnd(24)} att=${String(x.att).padEnd(24)} a=${x.alpha==null?"-":x.alpha.toFixed(3)} vis=${x.vis} ${bx}`);
    }
  }
  await b.close();
})();
