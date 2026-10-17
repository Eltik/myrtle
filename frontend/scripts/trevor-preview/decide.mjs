// Whether the preview runs the real worker or serves canned answers, decided once at start.
import fs from "node:fs";
import path from "node:path";
import { ASK_BIN, alive, E2B, G12B, LOCK, MIN_FREE_PCT, sh } from "./config.mjs";

/** Whether the preview can run the real worker: { model, profile, notes } or { why } for canned answers. */
function decideReal() {
    const notes = [];
    if (process.env.TREVOR_PREVIEW_REAL === "0") return { why: "TREVOR_PREVIEW_REAL=0" };
    if (!fs.existsSync(ASK_BIN)) return { why: `${ASK_BIN} is missing (cargo build --release --features embed in trevor/)` };
    const model = process.env.TREVOR_PREVIEW_MODEL ? path.resolve(process.env.TREVOR_PREVIEW_MODEL) : fs.existsSync(E2B) ? E2B : G12B;
    if (/26b/i.test(path.basename(model))) return { why: `${model} is the 26B, which the preview never loads` };
    if (!fs.existsSync(model)) return { why: `${model} is missing` };
    const want = process.env.TREVOR_PREVIEW_PROFILE ?? (model === E2B ? "mid" : "full");
    // clap names the profiles the binary knows when it refuses one.
    const known = sh(`'${ASK_BIN}' --profile zz-preview-probe`)
        .match(/possible values: ([^\]]+)\]/)?.[1]
        ?.split(/,\s*/) ?? ["full", "slim"];
    let profile = want;
    if (!known.includes(want)) {
        if (want !== "mid") return { why: `profile ${want} is not one of ${known.join(", ")}` };
        profile = "slim";
        notes.push(`profile mid is not in this ask binary (it knows ${known.join(", ")}); using slim`);
    }
    const llamas = sh("pgrep -x llama-server").match(/\d+/g);
    if (llamas) return { why: `a llama-server this preview did not start is running (pid ${llamas.join(", ")}); only one runs at a time` };
    let held = "";
    try {
        held = fs.readFileSync(LOCK, "utf8").trim();
    } catch {}
    if (held) {
        const pid = Number(held.split(/\s+/)[0]);
        if (pid > 0 && alive(pid)) return { why: `the model lock ${LOCK} is held by "${held}"` };
        fs.rmSync(LOCK, { force: true });
        notes.push(`removed a stale model lock ("${held}": that pid is gone)`);
    }
    const free = Number(sh("memory_pressure | tail -1").match(/(\d+)%/)?.[1] ?? Number.NaN);
    if (!(free >= MIN_FREE_PCT)) return { why: `memory_pressure reports ${free}% free, under ${MIN_FREE_PCT}%` };
    if (!sh("pmset -g therm").includes("No thermal warning level has been recorded")) return { why: "pmset -g therm records a thermal warning" };
    notes.push(`memory ${free}% free`);
    return { model, profile, notes };
}
export const DECIDED = decideReal();
export const REAL = !DECIDED.why;
export const PROFILE = DECIDED.profile ?? "slim";
