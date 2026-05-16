import { Resvg } from "@resvg/resvg-js";
import type { ReactNode } from "react";
import satori from "satori";
import { OG_CONFIG } from "./config";
import { getFonts } from "./fonts";

export interface IRenderDimensions {
    width: number;
    height: number;
}

export async function renderOgPng(node: ReactNode, dimensions?: IRenderDimensions): Promise<Buffer> {
    const fonts = await getFonts();
    const width = dimensions?.width ?? OG_CONFIG.width;
    const height = dimensions?.height ?? OG_CONFIG.height;
    const svg = await satori(node, { width, height, fonts });
    return new Resvg(svg, {
        fitTo: { mode: "width", value: width },
    })
        .render()
        .asPng();
}
