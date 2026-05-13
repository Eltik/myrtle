export type ReadableTextColor = "white" | "black";

export function readableTextColor(color: string | null | undefined): ReadableTextColor {
    if (!color) return "white";
    const trimmed = color.trim();

    if (trimmed.startsWith("#")) {
        const hex = trimmed.slice(1);
        const expanded =
            hex.length === 3 || hex.length === 4
                ? hex
                      .slice(0, 3)
                      .split("")
                      .map((c) => c + c)
                      .join("")
                : hex.length >= 6
                  ? hex.slice(0, 6)
                  : null;
        if (!expanded) return "white";
        const r = Number.parseInt(expanded.slice(0, 2), 16);
        const g = Number.parseInt(expanded.slice(2, 4), 16);
        const b = Number.parseInt(expanded.slice(4, 6), 16);
        if ([r, g, b].some((c) => Number.isNaN(c))) return "white";
        const luminance = relativeLuminance(r / 255, g / 255, b / 255);
        return luminance > 0.55 ? "black" : "white";
    }

    const oklch = trimmed.match(/^oklch\(\s*([0-9.]+)/i);
    if (oklch) {
        const L = Number.parseFloat(oklch[1] ?? "");
        if (!Number.isFinite(L)) return "white";
        return L > 0.72 ? "black" : "white";
    }

    return "white";
}

function relativeLuminance(r: number, g: number, b: number): number {
    // sRGB → linear, then WCAG luminance.
    const lin = (c: number) => (c <= 0.03928 ? c / 12.92 : ((c + 0.055) / 1.055) ** 2.4);
    return 0.2126 * lin(r) + 0.7152 * lin(g) + 0.0722 * lin(b);
}
