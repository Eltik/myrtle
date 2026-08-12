// Catch start-of-animation transients that the virtual-clock recorder hides.
//
// rec.js pumps the clock ~40 times during load before it ever screenshots, so anything that
// only exists on the first rendered frames — an uninitialised render target, a texture that
// has not decoded yet — is invisible to it. This loads the page on the REAL clock and grabs
// frames as fast as the browser will produce them.
//
// Usage: node earlycap.js <skel-path> <outdir> [shots] [intervalMs]

const puppeteer = require("puppeteer");
const fs = require("fs");
const path = require("path");

const [, , skel, outdir, shotsArg, intervalArg] = process.argv;
const SHOTS = Number(shotsArg || 40);
const INTERVAL = Number(intervalArg || 60);
const W = 900;
const H = 416;

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
    const url = `http://localhost:3000/dyntest?skel=${encodeURIComponent(skel)}&server=en&framing=authored`;
    await page.goto(url, { waitUntil: "domcontentloaded", timeout: 120000 });

    // Hide the page chrome once, so each shot is just the canvas rect.
    let box = null;
    for (let i = 0; i < SHOTS; i++) {
        if (!box) {
            box = await page.evaluate(() => {
                const c = document.querySelector("canvas");
                if (!c) return null;
                const keep = new Set();
                for (let n = c; n; n = n.parentElement) keep.add(n);
                for (const el of document.body.querySelectorAll("*")) if (!keep.has(el) && !el.contains(c)) el.style.visibility = "hidden";
                const r = c.getBoundingClientRect();
                return { x: r.x, y: r.y, width: r.width, height: r.height };
            });
        }
        if (box) {
            await page.screenshot({ path: path.join(outdir, `e${String(i).padStart(3, "0")}.png`), clip: box });
        }
        await new Promise((r) => setTimeout(r, INTERVAL));
    }
    await browser.close();
})().catch((e) => { console.error(e); process.exit(1); });
