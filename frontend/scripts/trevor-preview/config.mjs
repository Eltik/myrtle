// The preview's settings (env), the panel manifest, the gatePos scale and the HTTP helpers every module shares.
import { spawnSync } from "node:child_process";
import crypto from "node:crypto";
import fs from "node:fs";
import path from "node:path";

export const ROOT = path.resolve(import.meta.dir, "../../..");
export const TREVOR = path.join(ROOT, "trevor");
export const PANELS = path.join(TREVOR, "artifacts/panels");
export const PORT = Number(process.env.MOCK_PORT ?? 3999);
export const VITE_PORT = Number(process.env.VITE_PORT ?? 3005);
export const UPSTREAM = process.env.MOCK_UPSTREAM ?? "http://localhost:3060";
export const MODES = ["stream", "quick", "passages", "busy", "offline"];
export const DEMO = "trevor-demo";
export const DEMO_UID = process.env.MOCK_UID ?? "55699327";
export const STORY = "act18side_level_act18side_05_beg";
// Backend MAX_OPEN_PER_USER.
export const MAX_OPEN = Number(process.env.MOCK_MAX_OPEN ?? 3);
// Jobs queued or running past which an ask gets a quick look and a deferred job.
export const QUEUE_MAX = Number(process.env.TREVOR_PREVIEW_QUEUE_MAX ?? 2);
export const ASK_BIN = path.join(TREVOR, "target/release/ask");
export const E2B = path.join(TREVOR, "models/llm/gemma-4-e2b-it-q4_k_m.gguf");
export const G12B = path.join(TREVOR, "models/llm/gemma-4-12b-it-qat-q4_0.gguf");
export const LOCK = "/private/tmp/claude-501/trevor-llm.lock";
export const MIN_FREE_PCT = 30;
export const WORKER_KEY = crypto.randomBytes(24).toString("hex");

if (!fs.existsSync(path.join(PANELS, "manifest.json"))) {
    console.error(`no panel artifacts at ${PANELS}; build or copy them first`);
    process.exit(1);
}
export const MANIFEST = JSON.parse(fs.readFileSync(path.join(PANELS, "manifest.json"), "utf8"));
export const VERSION = MANIFEST.version;

// The gatePos scale (contract v1 section 1).
export const GATE_POS = new Map(MANIFEST.storyOrder.map((id, i) => [id, i + 1]));
const ERAS = [...MANIFEST.eras].sort((a, b) => a.position - b.position);
const eraAt = (pos) => ERAS.filter((e) => e.position > 0 && e.position <= pos).at(-1) ?? null;
/** The story the demo profile has read up to (null: nothing read). */
export let profileStory = STORY;
/** Moves the demo profile to a story (null: nothing read). */
export function setProfileStory(storyId) {
    profileStory = storyId;
}
const profilePos = () => (profileStory ? (GATE_POS.get(profileStory) ?? 0) : 0);
/** The job horizon: pos = min(gatePos(horizonStory), profilePos), its era, or { error } for an unknown story. */
export function horizonOf(storyId) {
    let pos = profilePos();
    if (storyId) {
        const s = GATE_POS.get(storyId);
        if (s === undefined) return { error: `horizonStory ${storyId} is not in the active Trevor version` };
        pos = Math.min(s, pos);
    }
    const era = eraAt(pos);
    return { pos, eraPos: era?.position ?? 0, spoiler: era ? { eraId: era.id, name: era.name, position: era.position } : null };
}
/** The contract v1.1 normQ: NFKC, lowercase, punctuation folded, whitespace collapsed (a trailing "?" goes with it). */
export const normQ = (q) =>
    q
        .normalize("NFKC")
        .toLowerCase()
        .replace(/[\p{P}\p{S}]+/gu, " ")
        .replace(/\s+/g, " ")
        .trim();

export const sh = (cmd) => {
    const r = spawnSync("/bin/sh", ["-c", cmd], { encoding: "utf8" });
    return `${r.stdout ?? ""}${r.stderr ?? ""}`;
};
export const alive = (pid) => {
    try {
        process.kill(pid, 0);
        return true;
    } catch (e) {
        return e.code === "EPERM";
    }
};
export const cors = { "access-control-allow-origin": `http://127.0.0.1:${VITE_PORT}`, "access-control-allow-credentials": "true", "access-control-allow-headers": "*", "access-control-allow-methods": "GET, POST, OPTIONS" };
export const json = (body, status = 200, headers = {}) => new Response(JSON.stringify(body), { status, headers: { "content-type": "application/json", ...cors, ...headers } });
/** One SSE event frame. */
export const sseEvent = (event, data) => `event: ${event}\ndata: ${JSON.stringify(data)}\n\n`;
export const iso = (ms) => (ms ? new Date(ms).toISOString() : null);
export const rand = (lo, hi) => lo + Math.random() * (hi - lo);
