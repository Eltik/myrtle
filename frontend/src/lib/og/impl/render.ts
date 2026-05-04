import { Resvg } from "@resvg/resvg-js";
import type { ReactNode } from "react";
import satori from "satori";
import { OG_CONFIG } from "./config";
import { getFonts } from "./fonts";

export async function renderOgPng(node: ReactNode): Promise<Buffer> {
    const fonts = await getFonts();
    const svg = await satori(node, {
        width: OG_CONFIG.width,
        height: OG_CONFIG.height,
        fonts,
    });
    return new Resvg(svg, {
        fitTo: { mode: "width", value: OG_CONFIG.width },
    })
        .render()
        .asPng();
}
