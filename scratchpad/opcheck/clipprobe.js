// Spine CLIPPING-attachment probe for dynchar parity work.
//   node clipprobe.js <skel-url-path> [beats]
// Civilight Eterna's entrance is the only captured skin the game pillarboxes (a hard 1920x1080
// aperture inside 2340x1080) and hers is the only `_Start` skeleton that ships clipping
// attachments. This reports, per beat, each clip's polygon extent in skeleton AND screen space,
// plus whether pixi-spine actually built the clippingContainer for it — so "the skeleton has no
// clip" can be told apart from "it has one and our draw-order splicing tore it apart".
const puppeteer = require("puppeteer");
const STEP = 0.033;
(async () => {
    const b = await puppeteer.launch({
        headless: "new",
        protocolTimeout: 900000,
        args: ["--use-gl=swiftshader", "--enable-unsafe-swiftshader", "--disable-dev-shm-usage", "--no-sandbox", "--disable-frame-rate-limit", "--window-size=900,416"],
    });
    const p = await b.newPage();
    await p.setViewport({ width: 900, height: 416 });
    await p.evaluateOnNewDocument(() => {
        let seed = 0x2f6e2b1;
        Math.random = () => { seed = (seed * 1664525 + 1013904223) >>> 0; return seed / 4294967296; };
        window.__vt = 0; let q = []; let id = 1;
        performance.now = () => window.__vt; Date.now = () => 1700000000000 + window.__vt;
        window.requestAnimationFrame = (cb) => { const i = id++; q.push({ i, cb }); return i; };
        window.cancelAnimationFrame = (i) => { q = q.filter((x) => x.i !== i); };
        window.__pump = (ms) => { window.__vt = ms; const d = q; q = []; for (const x of d) { try { x.cb(window.__vt); } catch (e) {} } return q.length; };
    });
    const skel = process.argv[2];
    await p.goto("http://localhost:3000/dyntest?skel=" + encodeURIComponent(skel) + "&server=en&framing=authored", { waitUntil: "domcontentloaded", timeout: 120000 });
    for (let i = 0; i < 40; i++) {
        await p.evaluate(() => window.__pump(window.__vt));
        await new Promise((r) => setTimeout(r, 250));
        if ((await p.evaluate(() => !!document.querySelector("canvas"))) && i > 8) break;
    }
    let t = 0;
    for (const target of (process.argv[3] || "2,6,10,14,18").split(",").map(Number)) {
        while (t < target - 1e-6) { t = Math.min(t + STEP, target); await p.evaluate((ms) => window.__pump(ms), t * 1000); }
        const r = await p.evaluate(() => (window.__dynClip ? window.__dynClip() : "NO HOOK"));
        if (typeof r === "string") { console.log(`t=${target}: ${r}`); continue; }
        console.log(`t=${target}: ${r.clips.length} clipping attachment(s)   screen ${r.screen.w}x${r.screen.h}`);
        for (const c of r.clips) {
            const l = c.localBox, s = c.screenBox;
            console.log(
                `   slot=${c.slot} att=${c.attachment} end=${c.endSlot} verts=${c.verts}\n` +
                `      local  ${l.w.toFixed(1)}x${l.h.toFixed(1)} aspect=${l.aspect.toFixed(4)}\n` +
                `      screen x=${s.x.toFixed(1)} y=${s.y.toFixed(1)} ${s.w.toFixed(1)}x${s.h.toFixed(1)}  fracW=${(s.w / r.screen.w).toFixed(4)}\n` +
                `      applied graphics=${c.applied.hasGraphics} clipChildren=${c.applied.clipChildren}`
            );
        }
    }
    await b.close();
})().catch((e) => { console.error(e); process.exit(1); });
