// Does our canvas actually reach alpha = 1?
//
// `page.screenshot` composites a transparent canvas over the PAGE background. If any part
// of the render leaves alpha < 1, the screenshot is lifted toward that background colour —
// which would look exactly like a global tone error and would vary per skin with coverage.
//
// Renders the same beat twice over two very different page backgrounds and diffs. Identical
// output proves alpha == 1 everywhere and kills the hypothesis; any difference localises it.
//
// Usage: node alphacheck.js <skel-path> <outdir> <t>

const puppeteer = require("puppeteer");
const fs = require("fs");
const path = require("path");

const [, , skel, outdir, tArg] = process.argv;
const T = Number(tArg);
const W = 900;
const H = 416;
const STEP = 0.033;

const initScript = () => {
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
    window.requestAnimationFrame = (cb) => { const id = nextId++; queue.push({ id, cb }); return id; };
    window.cancelAnimationFrame = (id) => { queue = queue.filter((q) => q.id !== id); };
    window.__pump = (ms) => {
        window.__vt = ms;
        const due = queue; queue = [];
        for (const q of due) { try { q.cb(window.__vt); } catch (e) { /* keep going */ } }
        return queue.length;
    };
};

(async () => {
    fs.mkdirSync(outdir, { recursive: true });
    const browser = await puppeteer.launch({
        headless: "new",
        protocolTimeout: 900_000,
        args: ["--use-gl=swiftshader", "--enable-unsafe-swiftshader", "--disable-dev-shm-usage",
               "--no-sandbox", "--disable-frame-rate-limit", `--window-size=${W},${H}`],
    });
    const page = await browser.newPage();
    await page.setViewport({ width: W, height: H, deviceScaleFactor: 1 });
    await page.evaluateOnNewDocument(initScript);
    await page.goto(`http://localhost:3000/dyntest?skel=${encodeURIComponent(skel)}&server=en&framing=authored`,
        { waitUntil: "domcontentloaded", timeout: 120000 });
    for (let i = 0; i < 40; i++) {
        await page.evaluate(() => window.__pump(window.__vt));
        await new Promise((r) => setTimeout(r, 250));
        if (await page.evaluate(() => !!document.querySelector("canvas")) && i > 8) break;
    }
    let t = 0;
    while (t < T - 1e-6) { t = Math.min(t + STEP, T); await page.evaluate((ms) => window.__pump(ms), t * 1000); }

    // Read the canvas's OWN alpha channel straight out of the drawing buffer, before any
    // page compositing can hide it.
    const alpha = await page.evaluate(() => {
        const c = document.querySelector("canvas");
        const gl = c.getContext("webgl2") || c.getContext("webgl");
        if (!gl) return { err: "no gl context" };
        const w = gl.drawingBufferWidth, h = gl.drawingBufferHeight;
        const px = new Uint8Array(w * h * 4);
        gl.readPixels(0, 0, w, h, gl.RGBA, gl.UNSIGNED_BYTE, px);
        let min = 255, n255 = 0, sum = 0;
        const hist = new Array(9).fill(0);
        for (let i = 3; i < px.length; i += 4) {
            const a = px[i];
            if (a < min) min = a;
            if (a === 255) n255++;
            sum += a;
            hist[Math.min(8, a >> 5)]++;
        }
        const total = w * h;
        return { w, h, min, meanAlpha: sum / total, fracOpaque: n255 / total, hist,
                 attrs: gl.getContextAttributes() };
    });
    console.log("ALPHA " + JSON.stringify(alpha));

    const box = await page.evaluate(() => {
        const c = document.querySelector("canvas");
        const keep = new Set();
        for (let n = c; n; n = n.parentElement) keep.add(n);
        for (const el of document.body.querySelectorAll("*")) if (!keep.has(el) && !el.contains(c)) el.style.visibility = "hidden";
        const r = c.getBoundingClientRect();
        return { x: r.x, y: r.y, width: r.width, height: r.height };
    });
    for (const [name, bg] of [["onblack", "#000000"], ["onwhite", "#ffffff"]]) {
        await page.evaluate((c) => {
            document.documentElement.style.background = c;
            document.body.style.background = c;
        }, bg);
        await page.screenshot({ path: path.join(outdir, `${name}.png`), clip: box });
    }
    await browser.close();
})().catch((e) => { console.error(e); process.exit(1); });
