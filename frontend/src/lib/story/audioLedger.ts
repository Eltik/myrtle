/**
 * THE DEV INSTRUMENT: a row per effect the engine hands the audio module, and
 * an RMS trace of both buses.
 *
 * A ledger row proves a source was SCHEDULED. Only the trace proves a signal
 * reached the destination, which is the distinction that took eleven green
 * browser runs to matter: headless Chromium disables the autoplay policy, so a
 * frozen context schedules everything correctly and sounds nothing. A run that
 * reads both can say which of the two happened.
 *
 * Both structures are RINGS, because a reader leaves a tab open. Neither is
 * built in a production bundle: `audio.ts` gates every call on `import.meta.env.DEV`.
 */

import type { FetchStatus } from "./audioBuffers";

/** One row per effect the engine hands the module, in a dev build only. */
export interface AudioLedgerRow {
    /** `ctx.currentTime` at the call. */
    t: number;
    /** The halt the reader was on, from `noteHalt`. */
    halt: number;
    kind: "music" | "stopMusic" | "musicVolume" | "sound" | "stopSound" | "soundVolume";
    /** The file's basename, or the channel for a level or stop command. */
    key: string;
    url: string | null;
    channel: string | null;
    requestedVolume: number | null;
    /** The channel level in force when the voice was created. */
    cueLevel: number | null;
    /** `delay=` as the engine handed it, already multiplied by `animateRatio`. */
    delay: number | null;
    fetchStatus: FetchStatus;
    decoded: { channels: number; sampleRate: number; duration: number } | null;
    fallback: boolean;
    /** Where the sound SHOULD start, on the audio clock: `t + delay`. */
    scheduledStart: number | null;
    /** Where it was actually told to start, after the fetch and decode. */
    actualStartClock: number | null;
    killedAt?: number;
    killReason?: "stopsound" | "channel-reuse" | "crossfade" | "stopmusic" | "musicSerial" | "dispose";
    gainAtStart: number | null;
    busGainAtStart: number | null;
}

/** The ledger is a ring: a 60-halt walk of the loudest story writes about 120 rows. */
export const LEDGER_MAX = 4000;

/** The RMS trace samples both buses every 50 ms, so 40 minutes fit in the ring. */
export const TRACE_MS = 50;
export const TRACE_MAX = 48000;

/** A file's basename, which is what a ledger row is read by. */
export function basename(url: string | null): string {
    return url ? (url.split("/").pop() ?? url) : "";
}

/**
 * The fields no command carries. A caller spreads its own over these, so a
 * ledger row at a call site says what is DIFFERENT about that command instead
 * of restating sixteen keys, and a new field lands in every row at once.
 */
export function ledgerRow(kind: AudioLedgerRow["kind"], t: number, halt: number, fields: Partial<AudioLedgerRow>): AudioLedgerRow {
    return {
        t,
        halt,
        kind,
        key: "",
        url: null,
        channel: null,
        requestedVolume: null,
        cueLevel: null,
        delay: null,
        fetchStatus: null,
        decoded: null,
        fallback: false,
        scheduledStart: null,
        actualStartClock: null,
        gainAtStart: null,
        busGainAtStart: null,
        ...fields,
    };
}

/** One RMS reading of an analyser, or 0 where there is none. */
export function rmsOf(node: AnalyserNode | null, scratch: { buffer: Float32Array<ArrayBuffer> | null }): number {
    if (!node) return 0;
    if (!scratch.buffer || scratch.buffer.length !== node.fftSize) scratch.buffer = new Float32Array(node.fftSize);
    const data = scratch.buffer;
    node.getFloatTimeDomainData(data);
    let sum = 0;
    for (let i = 0; i < data.length; i += 1) sum += data[i] * data[i];
    return Math.sqrt(sum / data.length);
}

export interface IRmsSummary {
    n: number;
    mean: number;
    max: number;
}

export function rmsSummary(values: number[]): IRmsSummary {
    return { n: values.length, mean: values.length ? values.reduce((a, b) => a + b, 0) / values.length : 0, max: values.length ? Math.max(...values) : 0 };
}
