/**
 * The share image: a 1600x900 PNG of the squad drawn on a `<canvas>`, client
 * side. The layout is plain math (`shareImageLayout`) so it is tested without
 * a canvas; `renderSquadImage` only paints it.
 *
 * A fixed dark palette rather than the page's tokens: the image leaves the
 * site, so it must look the same whichever theme the sender had on.
 */

import { loadImage, RARITY_HEX } from "#/lib/utils";
import { SQUAD_SIZE } from "./constants";

export const IMAGE_WIDTH = 1600;
export const IMAGE_HEIGHT = 900;

const PALETTE = {
    background: "#121214",
    card: "#1c1c20",
    cardEmpty: "#17171a",
    border: "#2e2e35",
    text: "#f4f4f5",
    muted: "#9d9da7",
    accent: "#ff8a6b",
};

const FONT = 'ui-sans-serif, system-ui, -apple-system, "Segoe UI", Roboto, sans-serif';

export interface IShareImageSeat {
    name: string;
    rarity: number;
    phase: number;
    level: number;
    portraitUrl: string;
}

export interface IShareImageData {
    title: string;
    /** Twelve entries, `null` for an empty seat. */
    seats: ReadonlyArray<IShareImageSeat | null>;
    support: IShareImageSeat | null;
    facts: ReadonlyArray<{ label: string; value: string }>;
    labels: {
        leader: string;
        empty: string;
        support: string;
        /** e.g. "E2 Lv 90"; called per seat. */
        build: (phase: number, level: number) => string;
        facts: string;
    };
    footer: string;
}

export interface IRect {
    x: number;
    y: number;
    w: number;
    h: number;
}

export interface IShareImageLayout {
    title: IRect;
    seats: IRect[];
    supportLabel: IRect;
    support: IRect;
    facts: IRect;
    footer: IRect;
    /** Height of the art inside a seat card; the rest is the caption and the rarity bar. */
    artHeight: number;
}

const PAD = 56;
const GAP = 16;
const ROW_GAP = 20;
const SIDE_W = 220;
const SIDE_GAP = 32;
const COLS = 6;

export function shareImageLayout(): IShareImageLayout {
    const contentW = IMAGE_WIDTH - PAD * 2;
    const gridW = contentW - SIDE_W - SIDE_GAP;
    const cardW = Math.floor((gridW - GAP * (COLS - 1)) / COLS);
    const top = 150;
    const footerH = 28;
    const bottom = IMAGE_HEIGHT - PAD - footerH - 16;
    const cardH = Math.floor((bottom - top - ROW_GAP) / 2);
    const seats: IRect[] = [];
    for (let i = 0; i < SQUAD_SIZE; i++) {
        const col = i % COLS;
        const row = Math.floor(i / COLS);
        seats.push({ x: PAD + col * (cardW + GAP), y: top + row * (cardH + ROW_GAP), w: cardW, h: cardH });
    }
    const sideX = IMAGE_WIDTH - PAD - SIDE_W;
    const labelH = 22;
    return {
        title: { x: PAD, y: PAD, w: contentW, h: 60 },
        seats,
        supportLabel: { x: sideX, y: top - labelH - 6, w: SIDE_W, h: labelH },
        support: { x: sideX, y: top, w: SIDE_W, h: cardH },
        facts: { x: sideX, y: top + cardH + ROW_GAP, w: SIDE_W, h: cardH },
        footer: { x: PAD, y: IMAGE_HEIGHT - PAD - footerH, w: contentW, h: footerH },
        artHeight: Math.round(cardH * 0.76),
    };
}

/** Load every portrait, `null` for one that failed so a placeholder is drawn instead. */
async function loadAll(urls: string[]): Promise<Map<string, HTMLImageElement | null>> {
    const unique = [...new Set(urls)];
    const loaded = await Promise.all(unique.map((u) => loadImage(u, { crossOrigin: true }).catch(() => null)));
    return new Map(unique.map((u, i) => [u, loaded[i]]));
}

function roundRect(ctx: CanvasRenderingContext2D, r: IRect, radius: number): void {
    ctx.beginPath();
    ctx.roundRect(r.x, r.y, r.w, r.h, radius);
}

function fitText(ctx: CanvasRenderingContext2D, text: string, maxW: number): string {
    if (ctx.measureText(text).width <= maxW) return text;
    let lo = 0;
    let hi = text.length;
    while (lo < hi) {
        const mid = Math.ceil((lo + hi) / 2);
        if (ctx.measureText(`${text.slice(0, mid)}…`).width <= maxW) lo = mid;
        else hi = mid - 1;
    }
    return `${text.slice(0, lo)}…`;
}

/** `object-fit: cover`, anchored to the top so faces stay in frame. */
function drawCover(ctx: CanvasRenderingContext2D, img: HTMLImageElement, r: IRect): void {
    const scale = Math.max(r.w / img.naturalWidth, r.h / img.naturalHeight);
    const w = img.naturalWidth * scale;
    const h = img.naturalHeight * scale;
    ctx.drawImage(img, r.x + (r.w - w) / 2, r.y, w, h);
}

function drawSeat(ctx: CanvasRenderingContext2D, rect: IRect, seat: IShareImageSeat | null, img: HTMLImageElement | null | undefined, artHeight: number, data: IShareImageData, leader: boolean): void {
    ctx.save();
    roundRect(ctx, rect, 12);
    ctx.fillStyle = seat ? PALETTE.card : PALETTE.cardEmpty;
    ctx.fill();
    ctx.clip();

    if (!seat) {
        ctx.setLineDash([6, 6]);
        ctx.strokeStyle = PALETTE.border;
        ctx.lineWidth = 2;
        roundRect(ctx, { x: rect.x + 1, y: rect.y + 1, w: rect.w - 2, h: rect.h - 2 }, 11);
        ctx.stroke();
        ctx.setLineDash([]);
        ctx.fillStyle = PALETTE.muted;
        ctx.font = `500 18px ${FONT}`;
        ctx.textAlign = "center";
        ctx.textBaseline = "middle";
        ctx.fillText(data.labels.empty, rect.x + rect.w / 2, rect.y + rect.h / 2);
        ctx.restore();
        return;
    }

    const art: IRect = { x: rect.x, y: rect.y, w: rect.w, h: artHeight };
    if (img) drawCover(ctx, img, art);
    else {
        ctx.fillStyle = PALETTE.border;
        ctx.fillRect(art.x, art.y, art.w, art.h);
        ctx.fillStyle = PALETTE.muted;
        ctx.font = `700 44px ${FONT}`;
        ctx.textAlign = "center";
        ctx.textBaseline = "middle";
        ctx.fillText(seat.name.slice(0, 1), art.x + art.w / 2, art.y + art.h / 2);
    }

    // A fade under the art so the caption reads on any portrait.
    const fade = ctx.createLinearGradient(0, art.y + art.h - 60, 0, art.y + art.h);
    fade.addColorStop(0, "rgba(28,28,32,0)");
    fade.addColorStop(1, PALETTE.card);
    ctx.fillStyle = fade;
    ctx.fillRect(art.x, art.y + art.h - 60, art.w, 60);

    const color = RARITY_HEX[seat.rarity] ?? PALETTE.muted;
    if (leader) {
        ctx.font = `600 14px ${FONT}`;
        const label = data.labels.leader;
        const w = ctx.measureText(label).width + 16;
        roundRect(ctx, { x: rect.x + 8, y: rect.y + 8, w, h: 24 }, 6);
        ctx.fillStyle = PALETTE.accent;
        ctx.fill();
        ctx.fillStyle = "#1a0f0b";
        ctx.textAlign = "left";
        ctx.textBaseline = "middle";
        ctx.fillText(label, rect.x + 16, rect.y + 20);
    }

    const textX = rect.x + 12;
    const maxW = rect.w - 24;
    ctx.textAlign = "left";
    ctx.textBaseline = "alphabetic";
    ctx.fillStyle = PALETTE.text;
    ctx.font = `600 18px ${FONT}`;
    ctx.fillText(fitText(ctx, seat.name, maxW), textX, rect.y + artHeight + 24);
    ctx.fillStyle = PALETTE.muted;
    ctx.font = `500 14px ${FONT}`;
    ctx.fillText(fitText(ctx, data.labels.build(seat.phase, seat.level), maxW), textX, rect.y + artHeight + 46);

    ctx.fillStyle = color;
    ctx.fillRect(rect.x, rect.y + rect.h - 6, rect.w, 6);
    ctx.restore();

    ctx.strokeStyle = PALETTE.border;
    ctx.lineWidth = 1;
    roundRect(ctx, { x: rect.x + 0.5, y: rect.y + 0.5, w: rect.w - 1, h: rect.h - 1 }, 12);
    ctx.stroke();
}

export async function renderSquadImage(canvas: HTMLCanvasElement, data: IShareImageData): Promise<void> {
    canvas.width = IMAGE_WIDTH;
    canvas.height = IMAGE_HEIGHT;
    const ctx = canvas.getContext("2d");
    if (!ctx) throw new Error("Canvas 2D context unavailable");
    const layout = shareImageLayout();

    const all = [...data.seats, data.support].filter((s): s is IShareImageSeat => s !== null);
    const images = await loadAll(all.map((s) => s.portraitUrl));

    ctx.fillStyle = PALETTE.background;
    ctx.fillRect(0, 0, IMAGE_WIDTH, IMAGE_HEIGHT);

    ctx.fillStyle = PALETTE.text;
    ctx.font = `700 44px ${FONT}`;
    ctx.textAlign = "left";
    ctx.textBaseline = "top";
    ctx.fillText(fitText(ctx, data.title, layout.title.w), layout.title.x, layout.title.y);

    for (let i = 0; i < SQUAD_SIZE; i++) {
        const seat = data.seats[i] ?? null;
        drawSeat(ctx, layout.seats[i], seat, seat ? images.get(seat.portraitUrl) : null, layout.artHeight, data, i === 0);
    }

    ctx.fillStyle = PALETTE.muted;
    ctx.font = `600 15px ${FONT}`;
    ctx.textAlign = "left";
    ctx.textBaseline = "top";
    ctx.fillText(data.labels.support, layout.supportLabel.x, layout.supportLabel.y);
    drawSeat(ctx, layout.support, data.support, data.support ? images.get(data.support.portraitUrl) : null, layout.artHeight, data, false);

    const f = layout.facts;
    ctx.save();
    roundRect(ctx, f, 12);
    ctx.fillStyle = PALETTE.card;
    ctx.fill();
    ctx.strokeStyle = PALETTE.border;
    ctx.stroke();
    ctx.restore();
    ctx.fillStyle = PALETTE.muted;
    ctx.font = `600 15px ${FONT}`;
    ctx.textBaseline = "top";
    ctx.fillText(data.labels.facts, f.x + 16, f.y + 16);
    let y = f.y + 52;
    for (const fact of data.facts) {
        ctx.fillStyle = PALETTE.muted;
        ctx.font = `500 15px ${FONT}`;
        ctx.textAlign = "left";
        ctx.fillText(fitText(ctx, fact.label, f.w - 90), f.x + 16, y);
        ctx.fillStyle = PALETTE.text;
        ctx.font = `700 17px ${FONT}`;
        ctx.textAlign = "right";
        ctx.fillText(fact.value, f.x + f.w - 16, y - 1);
        y += 34;
    }

    ctx.textAlign = "left";
    ctx.textBaseline = "middle";
    ctx.fillStyle = PALETTE.accent;
    ctx.font = `700 20px ${FONT}`;
    ctx.fillText("myrtle", layout.footer.x, layout.footer.y + layout.footer.h / 2);
    const brandW = ctx.measureText("myrtle").width;
    ctx.fillStyle = PALETTE.muted;
    ctx.font = `500 16px ${FONT}`;
    ctx.fillText(data.footer, layout.footer.x + brandW + 14, layout.footer.y + layout.footer.h / 2);
}

/** `canvas.toBlob` as a promise. */
export function canvasToBlob(canvas: HTMLCanvasElement): Promise<Blob> {
    return new Promise((resolve, reject) => canvas.toBlob((b) => (b ? resolve(b) : reject(new Error("Canvas export failed"))), "image/png"));
}
