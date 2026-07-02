import { Resvg } from "@resvg/resvg-js";
import type { ReactNode } from "react";
import satori from "satori";
import { OG_CONFIG } from "./config";
import { getFonts } from "./fonts";

export interface IRenderDimensions {
    width: number;
    height: number;
}

const MAX_CONCURRENT_RENDERS = 2;
// Cap the number of requests parked waiting for a slot. A hung satori/resvg render
// would otherwise let every later cache-miss pile up here forever.
const MAX_QUEUE_WAITERS = 16;
// Give up on a parked request after this long so one wedged render can't strand it.
const SLOT_WAIT_TIMEOUT_MS = 10_000;
let activeRenders = 0;
const renderQueue: Array<() => void> = [];

/** Thrown when no render slot can be obtained (queue full or wait timed out). */
export class OgRenderUnavailableError extends Error {
    constructor(message: string) {
        super(message);
        this.name = "OgRenderUnavailableError";
    }
}

async function withRenderSlot<T>(fn: () => Promise<T>): Promise<T> {
    if (activeRenders >= MAX_CONCURRENT_RENDERS) {
        if (renderQueue.length >= MAX_QUEUE_WAITERS) {
            throw new OgRenderUnavailableError("OG render queue is full");
        }
        await new Promise<void>((resolve, reject) => {
            let settled = false;
            const waiter = () => {
                if (settled) return;
                settled = true;
                clearTimeout(timer);
                resolve();
            };
            const timer = setTimeout(() => {
                if (settled) return;
                settled = true;
                const idx = renderQueue.indexOf(waiter);
                if (idx !== -1) renderQueue.splice(idx, 1);
                reject(new OgRenderUnavailableError("Timed out waiting for an OG render slot"));
            }, SLOT_WAIT_TIMEOUT_MS);
            renderQueue.push(waiter);
        });
    }

    activeRenders++;
    try {
        return await fn();
    } finally {
        activeRenders--;
        renderQueue.shift()?.();
    }
}

export async function renderOgPng(node: ReactNode, dimensions?: IRenderDimensions): Promise<Buffer> {
    return withRenderSlot(async () => {
        const fonts = await getFonts();
        const width = dimensions?.width ?? OG_CONFIG.width;
        const height = dimensions?.height ?? OG_CONFIG.height;
        const svg = await satori(node, { width, height, fonts });
        return new Resvg(svg, {
            fitTo: { mode: "width", value: width },
        })
            .render()
            .asPng();
    });
}
