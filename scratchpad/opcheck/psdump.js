// Dump which particle systems have LIVE particles at a given trackTime, and where they are
// on screen. Uses the same deterministic virtual clock as rec.js.
//
// Usage: node psdump.js <skel> <t> [w h]

const puppeteer = require("puppeteer");

const [, , skel, tArg, wArg, hArg] = process.argv;
const T = Number(tArg);
const W = Number(wArg || 1280);
const H = Number(hArg || 592);
const STEP = 0.033;

const initScript = () => {
    window.__PS_DEBUG = true;
    let seed = 0x2f6e2b1;
    Math.random = () => {
        seed = (seed * 1664525 + 1013904223) >>> 0;
        return seed / 4294967296;
    };
    window.__vt = 0;
    let queue = [];
    let nextId = 1;
    performance.now = () => window.__vt;
    Date.now = () => 1700000000000 + window.__vt;
    window.requestAnimationFrame = (cb) => {
        const id = nextId++;
        queue.push({ id, cb });
        return id;
    };
    window.cancelAnimationFrame = (id) => {
        queue = queue.filter((q) => q.id !== id);
    };
    window.__pump = (ms) => {
        window.__vt = ms;
        const due = queue;
        queue = [];
        for (const q of due) {
            try {
                q.cb(window.__vt);
            } catch {
                /* keep going */
            }
        }
        return queue.length;
    };
};

(async () => {
    const browser = await puppeteer.launch({
        headless: "new",
        args: ["--use-gl=swiftshader", "--enable-unsafe-swiftshader", "--no-sandbox", `--window-size=${W},${H}`],
    });
    const page = await browser.newPage();
    await page.setViewport({ width: W, height: H, deviceScaleFactor: 1 });
    await page.evaluateOnNewDocument(initScript);
    await page.goto(
        `http://localhost:3000/dyntest?skel=${encodeURIComponent(skel)}&server=en&framing=authored`,
        { waitUntil: "domcontentloaded", timeout: 120000 },
    );
    for (let i = 0; i < 40; i++) {
        await page.evaluate(() => window.__pump(window.__vt));
        await new Promise((r) => setTimeout(r, 250));
        const ready = await page.evaluate(() => !!document.querySelector("canvas"));
        if (ready && i > 8) break;
    }
    let t = 0;
    while (t < T - 1e-6) {
        t = Math.min(t + STEP, T);
        await page.evaluate((ms) => window.__pump(ms), t * 1000);
    }

    const report = await page.evaluate(() => {
        const regs = window.__psRegs || [];
        const canvas = document.querySelector("canvas");
        const cw = canvas ? canvas.width : 0;
        const ch = canvas ? canvas.height : 0;
        return regs.map((reg) => {
            const { url, data, emitters, foreground } = reg;
            const rows = emitters.map((e) => {
                const sys = e.data;
                const idx = data.systems.indexOf(sys);
                let live = 0;
                try {
                    live = e.liveCount();
                } catch {
                    live = -1;
                }
                const c = e.container;
                let b = null;
                try {
                    const r = c.getBounds();
                    b = [Math.round(r.x), Math.round(r.y), Math.round(r.width), Math.round(r.height)];
                } catch {
                    /* no bounds */
                }
                // Visible sprite children (the actual live billboards).
                let vis = 0;
                let onScreen = 0;
                const stack = [c];
                while (stack.length) {
                    const n = stack.pop();
                    if (n.children) for (const k of n.children) stack.push(k);
                    if (n !== c && n.visible && n.renderable !== false && n.worldAlpha > 0.01 && n.width !== undefined) {
                        vis++;
                        try {
                            const r = n.getBounds();
                            if (r.x + r.width > 0 && r.y + r.height > 0 && r.x < cw && r.y < ch) onScreen++;
                        } catch {
                            /* skip */
                        }
                    }
                }
                return {
                    _t: Number((e.time ?? -1).toFixed?.(2) ?? -1),
                    emitAcc: Number((e.emitAcc ?? -1).toFixed?.(3) ?? -1),
                    rate: e.rate ?? null,
                    poolN: e.pool ? e.pool.length : -1,
                    freeN: e.free ? e.free.length : -1,
                    dbox: e.displayBox ? [Math.round(e.displayBox.x), Math.round(e.displayBox.y), Math.round(e.displayBox.width), Math.round(e.displayBox.height)] : null,
                    cx: Number(c.x.toFixed(0)),
                    cy: Number(c.y.toFixed(0)),
                    boneResolved: e.boneAnchor ? !!e.boneAnchor.resolved : null,
                    boneName: e.boneAnchor ? e.boneAnchor.boneName : null,
                    chain: Array.isArray(sys.boneChain) ? sys.boneChain.slice(-3) : sys.boneChain,
                    follow: sys.followBone ?? null,
                    idx,
                    tex: sys.tex,
                    delay: sys.delay ?? null,
                    looping: !!sys.looping,
                    dur: sys.duration,
                    maxP: sys.maxParticles,
                    sort: sys.sort,
                    live,
                    vis,
                    onScreen,
                    fg: foreground.children.includes(c),
                    cAlpha: Number(c.alpha.toFixed(3)),
                    cWorldAlpha: Number(c.worldAlpha.toFixed(3)),
                    cVisible: c.visible,
                    bounds: b,
                    bone: sys.boneChain ? 1 : 0,
                };
            });
            const built = new Set(emitters.map((e) => e.data));
            const skipped = data.systems
                .map((s, i) => ({ i, s }))
                .filter((x) => !built.has(x.s))
                .map((x) => ({ idx: x.i, tex: x.s.tex, delay: x.s.delay ?? null, sort: x.s.sort }));
            return { url, canvas: [cw, ch], total: data.systems.length, rows, skipped };
        });
    });
    console.log(JSON.stringify(report, null, 1));
    await browser.close();
})().catch((e) => {
    console.error(e);
    process.exit(1);
});

// NOTE: requires a temporary hook in particles.ts `loadParticles`, removed after use:
//   if (typeof window !== "undefined" && (window as any).__PS_DEBUG) {
//       const w = window as any; w.__psRegs = w.__psRegs ?? [];
//       w.__psRegs.push({ url, data, emitters, background, foreground });
//   }
