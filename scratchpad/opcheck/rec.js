// Deterministic virtual-clock dynchar recorder.
//
// The render tick clamps dt to 0.1s, so the virtual clock MUST be advanced in steps
// of <= 100ms regardless of how sparsely we screenshot. Flush step and capture cadence
// are therefore independent: we always step 33ms, and only grab a frame when the
// requested capture time is reached.
//
// Usage: node rec.js <skel-path> <outdir> <shots: "t1,t2,t3"> [width height]

const puppeteer = require("puppeteer");
const fs = require("fs");
const path = require("path");

const [, , skelArg, outdir, shotsArg, wArg, hArg] = process.argv;
// The viewer plays the ENTRANCE off the IDLE skeleton — `_Start` is a set inside it, not a
// separate thing to load. Handing dyntest the `_Start` file still renders something that looks
// right, but the composite it builds is a different one and NONE of the `?dumplayers=1` hooks
// (`__dumpLayers`, `__particleCensus`, `__dumpEmitters`, `__dumpSlots`, …) are ever installed —
// every dump then reads `null` with no other symptom, which has cost a full investigation twice.
// `score_new.sh` already globs with `grep -v _Start`; normalise here so a hand-rolled call
// cannot silently diverge from the path every measurement uses.
const skel = (() => {
    const dec = decodeURIComponent(skelArg);
    if (!/_Start\.skel$/.test(dec)) return skelArg;
    const fixed = dec.replace(/_Start\.skel$/, ".skel");
    console.error(`[rec] NOTE: '_Start' skel requested; using the IDLE skel instead so the dump hooks install.\n[rec]       ${dec.split("/").pop()} -> ${fixed.split("/").pop()}`);
    // Re-encode only the final segment, matching how callers build the path.
    const parts = skelArg.split("/");
    parts[parts.length - 1] = encodeURIComponent(fixed.split("/").pop());
    return parts.join("/");
})();
const shots = shotsArg.split(",").map(Number).sort((a, b) => a - b);
const W = Number(wArg || 1280);
const H = Number(hArg || 592);
const STEP = 0.033;

const initScript = () => {
    // Virtual clock: everything time-related reads __vt (ms). rAF callbacks queue and
    // only run when the driver flushes, so rendering advances exactly with the clock.
    // Seed Math.random so two runs are bit-comparable: particles draw from it, and an
    // unseeded diff between runs is ~65k px of pure noise that swamps any real change.
    let seed = 0x2f6e2b1;
    Math.random = () => {
        seed = (seed * 1664525 + 1013904223) >>> 0;
        return seed / 4294967296;
    };
    window.__vt = 0;
    let queue = [];
    let nextId = 1;
    const realNow = performance.now.bind(performance);
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
            } catch (e) {
                /* keep the clock running */
            }
        }
        return queue.length;
    };
    window.__realNow = realNow;
};

(async () => {
    fs.mkdirSync(outdir, { recursive: true });
    const browser = await puppeteer.launch({
        headless: "new",
        // Swiftshader software GL takes tens of seconds per screenshot and the box is
        // shared with whatever else is running; the 180s CDP default turns background
        // load into a spurious "render crashed".
        protocolTimeout: 900_000,
        args: [
            "--use-gl=swiftshader",
            "--enable-unsafe-swiftshader",
            "--disable-dev-shm-usage",
            "--no-sandbox",
            // Without this the compositor's frame-rate cap can wedge on a long-running
            // box: `Page.captureScreenshot` then never returns (all chrome processes at
            // 0% CPU) and every render reads as "crashed". Removes only the frame cap,
            // not any rasterization path — the swiftshader GL backend is unchanged.
            "--disable-frame-rate-limit",
            `--window-size=${W},${H}`,
        ],
    });
    const page = await browser.newPage();
    // SUPERSAMPLING (`DSF=<n>`). The game renders the dyn illust into a 2048^2 target and the
    // capture downscales from it, so its frames carry the detail of a much higher-resolution
    // render. The production viewer matches that on any retina display (PIXI takes its resolution
    // from `window.devicePixelRatio`), but this harness pinned dSF to 1 -- so we were scoring a
    // render SOFTER than both the game and the shipping viewer. Measured on Civilight Eterna:
    // our frame carries only ~65% of the capture's high-frequency energy (|dx| 7.73 vs 11.22 in
    // her worst region, 4.48 vs 6.24 full-frame).
    //
    // Frames are captured at DSF x the requested size and downscaled back, i.e. SSAA. Default
    // stays 1 so every historical number remains reproducible.
    const DSF = Number(process.env.DSF || 1);
    await page.setViewport({ width: W, height: H, deviceScaleFactor: DSF });
    await page.evaluateOnNewDocument(initScript);

    const url =
        `http://localhost:3000/dyntest?skel=${encodeURIComponent(skel)}` +
        `&server=en&framing=authored` +
        (process.env.EXTRA ? `&${process.env.EXTRA}` : "");
    page.on("console", (m) => { const t = m.text(); if (/error|Error|failed/.test(t)) console.error("[page]", t.slice(0, 200)); });
    await page.goto(url, { waitUntil: "domcontentloaded", timeout: 120000 });

    // Let assets load on the REAL clock (fetches don't depend on rAF), pumping a little
    // so any load-gated rAF work can proceed without advancing the scene meaningfully.
    page.on("console", (m) => { const t = m.text(); if (t.includes("DBGBG")) console.error("[page]", t); });
    for (let i = 0; i < 40; i++) {
        await page.evaluate(() => window.__pump(window.__vt));
        await new Promise((r) => setTimeout(r, 250));
        const ready = await page.evaluate(
            () => !!document.querySelector("canvas"),
        );
        if (ready && i > 8) break;
    }

    let t = 0;
    for (const target of shots) {
        while (t < target - 1e-6) {
            t = Math.min(t + STEP, target);
            await page.evaluate((ms) => window.__pump(ms), t * 1000);
        }
        const file = path.join(outdir, `t${target.toFixed(2)}.png`);
        // Clip to the render canvas AND hide everything that isn't on its ancestor
        // chain: the dyntest page's fixed header paints OVER the canvas rect, so
        // clipping alone still pollutes every luminance measurement.
        //
        // KNOWN CONTAMINATION, unfixed: the TanStack devtools launcher paints a 56x56
        // badge over the bottom-right of the canvas and this pass does NOT remove it. It
        // reaches scored frames (cel t=2, mly t=4, ska t=3). Three attempts all changed
        // zero pixels: `visibility:hidden !important`, a double pass with a settle between,
        // and hiding whatever `document.elementsFromPoint` reports painting over the canvas
        // on a 25x25 grid. After killing 164 elements the button still computes
        // `visibility: visible`, so the inline rule is not landing on whatever paints —
        // do not retry DOM hiding without first explaining that. Measured cost: it lifts the
        // affected beat by 0.34-0.79 MAD and the skin mean by 0.043 (mly) / 0.086 (ska) /
        // 0.113 (cel) MADC, always AGAINST us, so every published figure is pessimistic by
        // about that much. The clean fixes are to stop the app mounting devtools on
        // /dyntest, or to mask the badge rect in both images the way `--inner` masks the
        // capture's UI border.
        const box = await page.evaluate(() => {
            const c = document.querySelector("canvas");
            if (!c) return null;
            const keep = new Set();
            for (let n = c; n; n = n.parentElement) keep.add(n);
            for (const el of document.body.querySelectorAll("*")) {
                if (!keep.has(el) && !el.contains(c)) el.style.visibility = "hidden";
            }
            const r = c.getBoundingClientRect();
            return { x: r.x, y: r.y, width: r.width, height: r.height };
        });
        await page.screenshot(box ? { path: file, clip: box } : { path: file });
        if (process.env.CAMDUMP) {
            const cb = await page.evaluate(() => window.__camBox ?? null);
            console.log("CAMBOX " + file + " " + JSON.stringify(cb));
        }
        if (process.env.EXTRA && /dumplayers/.test(process.env.EXTRA)) {
            // FIRST and guarded: the probes below throw when a ref is null (`__dumpSlots` does
            // at the settled idle), and one throw aborts the whole block — which silently ate
            // this dump. Sampling-independent, so it covers composites already destroyed.
            try {
                const pc = await page.evaluate(() => (window.__particleCensus ? window.__particleCensus() : "NO-HOOK"));
                console.log("PCENSUS " + JSON.stringify(pc));
            } catch (e) {
                console.log("PCENSUS null");
            }
            try {
                const rows = await page.evaluate(() => (window.__dumpLayers ? window.__dumpLayers() : "NO-HOOK"));
                console.log("LAYERDUMP " + file + " " + JSON.stringify(rows));
            } catch (e) {
                console.log("LAYERDUMP null");
            }
            try {
                const st = await page.evaluate(() => (window.__dumpStage ? window.__dumpStage() : "NO-HOOK"));
                console.log("STAGEDUMP " + file + " " + JSON.stringify(st));
            } catch (e) {
                console.log("STAGEDUMP null");
            }
            try {
                const tex = await page.evaluate(() => (window.__dumpTex ? window.__dumpTex() : "NO-HOOK"));
                console.log("TEXDUMP " + JSON.stringify(tex));
            } catch (e) {
                console.log("TEXDUMP null");
            }
            try {
                const slots = await page.evaluate(() => (window.__dumpSlots ? window.__dumpSlots() : "NO-HOOK"));
                console.log("SLOTDUMP " + file + " " + JSON.stringify(slots));
            } catch (e) {
                console.log("SLOTDUMP null");
            }
            const cls = await page.evaluate(() => window.__classDump ?? null);
            if (cls) console.log("CLASSDUMP " + JSON.stringify(cls));
            try {
                const sk = await page.evaluate(() => (window.__dumpSkins ? window.__dumpSkins() : null));
                console.log("SKINDUMP " + file + " " + JSON.stringify(sk));
            } catch (e) {
                console.log("SKINDUMP null");
            }
            if (process.env.MESHSLOT) {
                const mesh = await page.evaluate((n) => (window.__dumpMesh ? window.__dumpMesh(n) : null), process.env.MESHSLOT);
                require("fs").writeFileSync(file.replace(/\.png$/, ".mesh.json"), JSON.stringify(mesh));
                console.log("MESHDUMP " + file);
            }
            try {
                const em = await page.evaluate(() => (window.__dumpEmitters ? window.__dumpEmitters() : "NO-HOOK"));
                console.log("EMDUMP " + file + " " + JSON.stringify(em));
            } catch (e) {
                console.log("EMDUMP null");
            }
        }
        process.stdout.write(`${file}\n`);
    }
    await browser.close();
})().catch((e) => {
    console.error(e);
    process.exit(1);
});
