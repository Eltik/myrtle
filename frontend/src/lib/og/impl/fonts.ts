// SERVER ONLY — uses node:fs and node:module.
// Imported by render.ts (the API route + prerender script). Never reaches the client.
import { readFile } from "node:fs/promises";
import { createRequire } from "node:module";
import type { SatoriOptions } from "satori";

type FontInput = NonNullable<SatoriOptions["fonts"]>[number];

const require = createRequire(import.meta.url);

function interPath(file: string): string {
    return require.resolve(`@fontsource/inter/files/${file}`);
}

function geistMonoPath(file: string): string {
    return require.resolve(`@fontsource/geist-mono/files/${file}`);
}

const FONTS = [
    {
        name: "Inter",
        weight: 400 as const,
        style: "normal" as const,
        source: interPath("inter-latin-400-normal.woff"),
    },
    {
        name: "Inter",
        weight: 700 as const,
        style: "normal" as const,
        source: interPath("inter-latin-700-normal.woff"),
    },
    {
        name: "Geist Mono",
        weight: 500 as const,
        style: "normal" as const,
        source: geistMonoPath("geist-mono-latin-500-normal.woff"),
    },
    {
        name: "Geist Mono",
        weight: 700 as const,
        style: "normal" as const,
        source: geistMonoPath("geist-mono-latin-700-normal.woff"),
    },
];

let cache: Promise<FontInput[]> | null = null;

export function getFonts(): Promise<FontInput[]> {
    if (!cache) cache = loadFonts();
    return cache;
}

async function loadFonts(): Promise<FontInput[]> {
    return Promise.all(
        FONTS.map(async (font) => {
            const data = await loadSource(font.source);
            return { name: font.name, data, weight: font.weight, style: font.style };
        }),
    );
}

async function loadSource(source: string): Promise<ArrayBuffer> {
    if (/^https?:\/\//i.test(source)) {
        const res = await fetch(source);
        if (!res.ok) throw new Error(`Failed to fetch font ${source}: ${res.status}`);
        return res.arrayBuffer();
    }
    const buf = await readFile(source);
    return buf.buffer.slice(buf.byteOffset, buf.byteOffset + buf.byteLength) as ArrayBuffer;
}
