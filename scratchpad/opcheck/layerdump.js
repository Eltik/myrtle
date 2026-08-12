// Dump every SCENE LAYER's live draw state + screen box at a given trackTime, via the
// `?dumplayers=1` diagnostic. Same deterministic virtual clock as rec.js/psdump.js.
//
// Usage: node layerdump.js <skel> <t> [w h]

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
        `http://localhost:3000/dyntest?skel=${encodeURIComponent(skel)}&server=en&framing=authored&dumplayers=1`,
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
    const report = await page.evaluate(() => (window.__dumpLayers ? window.__dumpLayers() : null));
    console.log(JSON.stringify(report, null, 1));
    await browser.close();
})().catch((e) => {
    console.error(e);
    process.exit(1);
});
