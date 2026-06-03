import { toBlob } from "html-to-image";
import { downloadBlob } from "#/lib/utils";

function resolveBackground(node: HTMLElement): string {
    let el: HTMLElement | null = node;
    while (el) {
        const bg = getComputedStyle(el).backgroundColor;
        if (bg && bg !== "rgba(0, 0, 0, 0)" && bg !== "transparent") return bg;
        el = el.parentElement;
    }
    return getComputedStyle(document.documentElement).getPropertyValue("--background").trim() || "#ffffff";
}

export async function exportPlanAsImage(node: HTMLElement, filename: string): Promise<void> {
    const blob = await toBlob(node, {
        backgroundColor: resolveBackground(node),
        pixelRatio: Math.max(2, window.devicePixelRatio || 1),
        cacheBust: true,
        width: node.scrollWidth,
        height: node.scrollHeight,
    });
    if (!blob) throw new Error("Could not render the plan image");
    downloadBlob(blob, filename);
}
