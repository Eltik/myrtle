import { downloadBlob, loadImage } from "#/lib/utils";

/**
 * Render an in-DOM SVG element to a PNG and trigger a browser download.
 *
 * Recharts renders an SVG that inherits CSS custom properties (e.g.
 * `var(--border)`, `var(--muted-foreground)`) from the document. Once the SVG
 * is detached and rasterised by the browser via `<img src=blob:…svg>`, those
 * variables are no longer in scope, so the chart would render with the
 * defaults (typically black on transparent). To work around that we walk the
 * source tree in parallel with a clone and inline each element's computed
 * paint properties (fill, stroke, color, font-*) as inline styles.
 */
export interface IExportSvgOptions {
    filename?: string;
    /** Background fill applied behind the SVG (CSS color string). Defaults to the closest ancestor card background, then `#ffffff`. */
    background?: string;
    /** Pixel-density multiplier. Defaults to `window.devicePixelRatio` (clamped to ≥2). */
    pixelRatio?: number;
    /** Optional title rendered above the chart in the exported PNG. */
    title?: string;
    /** Optional legend rows rendered above the chart. The HTML legend recharts ships is outside the SVG, so it isn't captured by default. */
    legend?: ILegendEntry[];
    /** Optional snapshot callout drawn next to the chart's reference line, listing each operator's value at the snapshot X. */
    snapshot?: ISnapshotCallout;
}

export interface ILegendEntry {
    color: string;
    label: string;
    /** Optional secondary text rendered in muted style after the label (e.g. mastery / module summary). */
    sublabel?: string;
}

export interface ISnapshotCallout {
    /** Heading shown at the top of the callout, e.g. "@ DEF 1000". */
    heading: string;
    entries: { color: string; name: string; value: string }[];
}

const PAINT_PROPS = ["fill", "fill-opacity", "stroke", "stroke-width", "stroke-opacity", "stroke-dasharray", "stroke-linecap", "stroke-linejoin", "color", "opacity", "font-family", "font-size", "font-weight", "text-anchor", "dominant-baseline", "letter-spacing"] as const;

export async function exportSvgAsPng(svg: SVGSVGElement, options: IExportSvgOptions = {}): Promise<void> {
    const { filename = "dps-chart.png", pixelRatio = Math.max(2, window.devicePixelRatio || 1) } = options;

    const rect = svg.getBoundingClientRect();
    const chartWidth = Math.max(1, Math.round(rect.width || svg.clientWidth || Number(svg.getAttribute("width")) || 0));
    const chartHeight = Math.max(1, Math.round(rect.height || svg.clientHeight || Number(svg.getAttribute("height")) || 0));
    if (chartWidth <= 1 || chartHeight <= 1) throw new Error("Chart has no measurable dimensions");

    const cloned = svg.cloneNode(true) as SVGSVGElement;
    inlineComputedStyles(svg, cloned);

    cloned.setAttribute("xmlns", "http://www.w3.org/2000/svg");
    cloned.setAttribute("xmlns:xlink", "http://www.w3.org/1999/xlink");

    const fgColor = window.getComputedStyle(svg).color || "#0a0a0a";
    const mutedColor = resolveCssVar(svg, "--muted-foreground", fgColor);
    const bgColor = options.background ?? resolveBackgroundColor(svg);
    const borderColor = resolveCssVar(svg, "--border", "rgba(0,0,0,0.12)");

    // Snapshot callout sits inside the chart's coordinate space, anchored to
    // the reference line. Inject before wrapping so it ends up inside the
    // translate group along with the rest of the chart.
    if (options.snapshot && options.snapshot.entries.length > 0) {
        injectSnapshotCallout(cloned, options.snapshot, fgColor, mutedColor, bgColor, borderColor, chartWidth);
    }

    // Compute header (title + legend) layout BEFORE mutating the DOM tree.
    const titleBlock = options.title ? buildTitleBlock(options.title, chartWidth, fgColor) : null;
    const legendBlock = options.legend && options.legend.length > 0 ? buildLegendBlock(options.legend, chartWidth, fgColor, mutedColor) : null;
    const headerHeight = (titleBlock?.height ?? 0) + (legendBlock?.height ?? 0);

    if (headerHeight > 0) {
        // Wrap the cloned chart content in a translate group so the original
        // chart sits below the header without overlapping the new elements.
        const moved = document.createElementNS("http://www.w3.org/2000/svg", "g");
        moved.setAttribute("transform", `translate(0, ${headerHeight})`);
        while (cloned.firstChild) moved.appendChild(cloned.firstChild);
        cloned.appendChild(moved);

        let yOffset = 0;
        if (titleBlock) {
            titleBlock.element.setAttribute("transform", `translate(0, ${yOffset})`);
            cloned.insertBefore(titleBlock.element, cloned.firstChild);
            yOffset += titleBlock.height;
        }
        if (legendBlock) {
            legendBlock.element.setAttribute("transform", `translate(0, ${yOffset})`);
            cloned.insertBefore(legendBlock.element, titleBlock ? titleBlock.element.nextSibling : cloned.firstChild);
        }
    }

    const totalHeight = chartHeight + headerHeight;
    cloned.setAttribute("width", String(chartWidth));
    cloned.setAttribute("height", String(totalHeight));
    cloned.setAttribute("viewBox", `0 0 ${chartWidth} ${totalHeight}`);

    const bg = options.background ?? resolveBackgroundColor(svg);
    if (bg) {
        const bgRect = document.createElementNS("http://www.w3.org/2000/svg", "rect");
        bgRect.setAttribute("width", "100%");
        bgRect.setAttribute("height", "100%");
        bgRect.setAttribute("fill", bg);
        cloned.insertBefore(bgRect, cloned.firstChild);
    }

    const xml = new XMLSerializer().serializeToString(cloned);
    // Use a data URL: same-origin so the resulting canvas is never tainted by
    // the rasterised image, which avoids `canvas.toBlob` returning null.
    const dataURL = `data:image/svg+xml;charset=utf-8,${encodeURIComponent(xml)}`;

    const img = await loadImage(dataURL);

    const canvas = document.createElement("canvas");
    canvas.width = Math.ceil(chartWidth * pixelRatio);
    canvas.height = Math.ceil(totalHeight * pixelRatio);
    const ctx = canvas.getContext("2d");
    if (!ctx) throw new Error("Could not acquire 2d canvas context");
    ctx.scale(pixelRatio, pixelRatio);
    ctx.drawImage(img, 0, 0, chartWidth, totalHeight);

    const blob = await new Promise<Blob | null>((resolve) => canvas.toBlob(resolve, "image/png"));
    if (!blob) {
        // Fallback path for the rare case where toBlob returns null - convert
        // the dataURL into a Blob ourselves so the user still gets a download.
        const dataPng = canvas.toDataURL("image/png");
        const fallback = dataURLToBlob(dataPng);
        downloadBlob(fallback, filename);
        return;
    }
    downloadBlob(blob, filename);
}

function dataURLToBlob(dataURL: string): Blob {
    const [header, base64] = dataURL.split(",", 2);
    const mime = /data:(.*?);base64/.exec(header)?.[1] ?? "image/png";
    const bytes = atob(base64);
    const buf = new Uint8Array(bytes.length);
    for (let i = 0; i < bytes.length; i++) buf[i] = bytes.charCodeAt(i);
    return new Blob([buf], { type: mime });
}

function inlineComputedStyles(src: Element, dst: Element): void {
    const computed = window.getComputedStyle(src);
    const target = dst as SVGElement | HTMLElement;
    for (const prop of PAINT_PROPS) {
        const value = computed.getPropertyValue(prop);
        if (value && value !== "none" && value !== "rgba(0, 0, 0, 0)") {
            target.style.setProperty(prop, value);
        }
    }
    const srcChildren = src.children;
    const dstChildren = dst.children;
    const len = Math.min(srcChildren.length, dstChildren.length);
    for (let i = 0; i < len; i++) {
        inlineComputedStyles(srcChildren[i], dstChildren[i]);
    }
}

function resolveBackgroundColor(svg: SVGSVGElement): string {
    let node: Element | null = svg;
    while (node) {
        const bg = window.getComputedStyle(node).backgroundColor;
        if (bg && bg !== "rgba(0, 0, 0, 0)" && bg !== "transparent") return bg;
        node = node.parentElement;
    }
    return "#ffffff";
}

function resolveCssVar(svg: SVGSVGElement, varName: string, fallback: string): string {
    const value = window.getComputedStyle(svg).getPropertyValue(varName).trim();
    if (value) return value;
    const docValue = window.getComputedStyle(document.documentElement).getPropertyValue(varName).trim();
    return docValue || fallback;
}

const TITLE_FONT = "system-ui, -apple-system, 'Segoe UI', sans-serif";
const TITLE_FONT_SIZE = 14;
const LEGEND_FONT = "system-ui, -apple-system, 'Segoe UI', sans-serif";
const LEGEND_FONT_SIZE = 12;
const LEGEND_LINE_HEIGHT = 22;
const LEGEND_PAD_X = 16;
const LEGEND_PAD_Y = 10;
const LEGEND_ITEM_GAP = 18;
const LEGEND_SWATCH_RADIUS = 5;
const LEGEND_SWATCH_TO_TEXT = 8;

function injectSnapshotCallout(svg: SVGSVGElement, snapshot: ISnapshotCallout, fgColor: string, mutedColor: string, bgColor: string, borderColor: string, chartWidth: number): void {
    const refLine = svg.querySelector<SVGLineElement>(".recharts-reference-line line");
    if (!refLine) return;

    const lineX = parseFloat(refLine.getAttribute("x1") ?? "0");
    const lineY1 = parseFloat(refLine.getAttribute("y1") ?? "0");
    const topY = Math.min(lineY1, parseFloat(refLine.getAttribute("y2") ?? "0"));

    const headingFontSize = 10.5;
    const itemFontSize = 11;
    const padX = 10;
    const padY = 8;
    const swatchToText = 6;
    const valueGap = 14;
    const swatchRadius = 3.5;
    const lineHeight = 16;

    // Pre-measure widths for each row (name + value) so the callout sizes itself.
    const probe = document.createElementNS("http://www.w3.org/2000/svg", "svg");
    probe.style.position = "absolute";
    probe.style.visibility = "hidden";
    probe.style.left = "-99999px";
    probe.style.top = "-99999px";
    probe.style.width = "0";
    probe.style.height = "0";
    document.body.appendChild(probe);

    let headingWidth = 0;
    const itemWidths: { name: number; value: number; total: number }[] = [];
    try {
        headingWidth = measureText(probe, snapshot.heading, LEGEND_FONT, headingFontSize, "600");
        for (const entry of snapshot.entries) {
            const nameW = measureText(probe, entry.name, LEGEND_FONT, itemFontSize, "500");
            const valueW = measureText(probe, entry.value, LEGEND_FONT, itemFontSize, "600");
            itemWidths.push({ name: nameW, value: valueW, total: swatchRadius * 2 + swatchToText + nameW + valueGap + valueW });
        }
    } finally {
        probe.remove();
    }

    const contentWidth = Math.max(headingWidth, ...itemWidths.map((w) => w.total));
    const boxWidth = contentWidth + 2 * padX;
    const boxHeight = padY + headingFontSize + 6 + snapshot.entries.length * lineHeight + padY - 4;

    // Decide which side of the line to anchor on. Prefer right if there's room.
    const offsetFromLine = 8;
    const placeRight = lineX + offsetFromLine + boxWidth + 8 <= chartWidth;
    const boxX = placeRight ? lineX + offsetFromLine : Math.max(8, lineX - offsetFromLine - boxWidth);
    const boxY = topY + 4;

    const g = document.createElementNS("http://www.w3.org/2000/svg", "g");
    g.setAttribute("transform", `translate(${boxX}, ${boxY})`);
    g.setAttribute("font-family", LEGEND_FONT);

    const bg = document.createElementNS("http://www.w3.org/2000/svg", "rect");
    bg.setAttribute("width", String(boxWidth));
    bg.setAttribute("height", String(boxHeight));
    bg.setAttribute("rx", "6");
    bg.setAttribute("ry", "6");
    bg.setAttribute("fill", bgColor);
    bg.setAttribute("stroke", borderColor);
    bg.setAttribute("stroke-width", "1");
    bg.setAttribute("fill-opacity", "0.96");
    g.appendChild(bg);

    const heading = document.createElementNS("http://www.w3.org/2000/svg", "text");
    heading.setAttribute("x", String(padX));
    heading.setAttribute("y", String(padY + headingFontSize - 1));
    heading.setAttribute("font-size", String(headingFontSize));
    heading.setAttribute("font-weight", "600");
    heading.setAttribute("fill", mutedColor);
    heading.textContent = snapshot.heading;
    g.appendChild(heading);

    const itemTopY = padY + headingFontSize + 6;
    snapshot.entries.forEach((entry, i) => {
        const cy = itemTopY + i * lineHeight + lineHeight / 2;

        const dot = document.createElementNS("http://www.w3.org/2000/svg", "circle");
        dot.setAttribute("cx", String(padX + swatchRadius));
        dot.setAttribute("cy", String(cy));
        dot.setAttribute("r", String(swatchRadius));
        dot.setAttribute("fill", entry.color);
        g.appendChild(dot);

        const name = document.createElementNS("http://www.w3.org/2000/svg", "text");
        name.setAttribute("x", String(padX + swatchRadius * 2 + swatchToText));
        name.setAttribute("y", String(cy));
        name.setAttribute("dominant-baseline", "central");
        name.setAttribute("font-size", String(itemFontSize));
        name.setAttribute("font-weight", "500");
        name.setAttribute("fill", fgColor);
        name.textContent = entry.name;
        g.appendChild(name);

        const value = document.createElementNS("http://www.w3.org/2000/svg", "text");
        value.setAttribute("x", String(boxWidth - padX));
        value.setAttribute("y", String(cy));
        value.setAttribute("dominant-baseline", "central");
        value.setAttribute("text-anchor", "end");
        value.setAttribute("font-size", String(itemFontSize));
        value.setAttribute("font-weight", "600");
        value.setAttribute("fill", fgColor);
        value.textContent = entry.value;
        g.appendChild(value);
    });

    svg.appendChild(g);
}

function buildTitleBlock(title: string, width: number, color: string): { element: SVGGElement; height: number } {
    const height = TITLE_FONT_SIZE + 12;
    const g = document.createElementNS("http://www.w3.org/2000/svg", "g");
    const text = document.createElementNS("http://www.w3.org/2000/svg", "text");
    text.setAttribute("x", String(LEGEND_PAD_X));
    text.setAttribute("y", String(height - 6));
    text.setAttribute("font-family", TITLE_FONT);
    text.setAttribute("font-size", String(TITLE_FONT_SIZE));
    text.setAttribute("font-weight", "600");
    text.setAttribute("fill", color);
    text.textContent = title;
    g.appendChild(text);
    void width;
    return { element: g, height };
}

function buildLegendBlock(entries: ILegendEntry[], width: number, color: string, mutedColor: string): { element: SVGGElement; height: number } {
    const measurements = measureLegendWidths(entries);
    const maxRowWidth = Math.max(50, width - 2 * LEGEND_PAD_X);

    interface IRow {
        items: { entry: ILegendEntry; widths: { label: number; sub: number; total: number } }[];
        width: number;
    }

    const rows: IRow[] = [];
    let current: IRow = { items: [], width: 0 };
    rows.push(current);
    for (let i = 0; i < entries.length; i++) {
        const w = measurements[i].total;
        const projectedWidth = current.items.length === 0 ? w : current.width + LEGEND_ITEM_GAP + w;
        if (current.items.length > 0 && projectedWidth > maxRowWidth) {
            current = { items: [], width: 0 };
            rows.push(current);
        }
        current.items.push({ entry: entries[i], widths: measurements[i] });
        current.width = current.items.length === 1 ? w : current.width + LEGEND_ITEM_GAP + w;
    }

    const height = rows.length * LEGEND_LINE_HEIGHT + 2 * LEGEND_PAD_Y;

    const g = document.createElementNS("http://www.w3.org/2000/svg", "g");
    g.setAttribute("font-family", LEGEND_FONT);
    g.setAttribute("font-size", String(LEGEND_FONT_SIZE));

    rows.forEach((row, rowIdx) => {
        const cy = LEGEND_PAD_Y + rowIdx * LEGEND_LINE_HEIGHT + LEGEND_LINE_HEIGHT / 2;
        let x = LEGEND_PAD_X;
        for (const item of row.items) {
            const cx = x + LEGEND_SWATCH_RADIUS;
            const circle = document.createElementNS("http://www.w3.org/2000/svg", "circle");
            circle.setAttribute("cx", String(cx));
            circle.setAttribute("cy", String(cy));
            circle.setAttribute("r", String(LEGEND_SWATCH_RADIUS));
            circle.setAttribute("fill", item.entry.color);
            g.appendChild(circle);

            const labelX = cx + LEGEND_SWATCH_RADIUS + LEGEND_SWATCH_TO_TEXT;
            const labelText = document.createElementNS("http://www.w3.org/2000/svg", "text");
            labelText.setAttribute("x", String(labelX));
            labelText.setAttribute("y", String(cy));
            labelText.setAttribute("dominant-baseline", "central");
            labelText.setAttribute("fill", color);
            labelText.setAttribute("font-weight", "500");
            labelText.textContent = item.entry.label;
            g.appendChild(labelText);

            if (item.entry.sublabel) {
                const subText = document.createElementNS("http://www.w3.org/2000/svg", "text");
                subText.setAttribute("x", String(labelX + item.widths.label + 6));
                subText.setAttribute("y", String(cy));
                subText.setAttribute("dominant-baseline", "central");
                subText.setAttribute("fill", mutedColor);
                subText.textContent = item.entry.sublabel;
                g.appendChild(subText);
            }

            x += item.widths.total + LEGEND_ITEM_GAP;
        }
    });

    return { element: g, height };
}

function measureLegendWidths(entries: ILegendEntry[]): { label: number; sub: number; total: number }[] {
    const probe = document.createElementNS("http://www.w3.org/2000/svg", "svg");
    probe.style.position = "absolute";
    probe.style.visibility = "hidden";
    probe.style.left = "-99999px";
    probe.style.top = "-99999px";
    probe.style.width = "0";
    probe.style.height = "0";
    document.body.appendChild(probe);
    try {
        const swatchPart = LEGEND_SWATCH_RADIUS * 2 + LEGEND_SWATCH_TO_TEXT;
        return entries.map((entry) => {
            const labelWidth = measureText(probe, entry.label, LEGEND_FONT, LEGEND_FONT_SIZE, "500");
            const subWidth = entry.sublabel ? measureText(probe, entry.sublabel, LEGEND_FONT, LEGEND_FONT_SIZE, "400") + 6 : 0;
            return { label: labelWidth, sub: subWidth, total: swatchPart + labelWidth + subWidth };
        });
    } finally {
        probe.remove();
    }
}

function measureText(host: SVGSVGElement, text: string, fontFamily: string, fontSize: number, fontWeight: string): number {
    const t = document.createElementNS("http://www.w3.org/2000/svg", "text");
    t.style.fontFamily = fontFamily;
    t.style.fontSize = `${fontSize}px`;
    t.style.fontWeight = fontWeight;
    t.textContent = text;
    host.appendChild(t);
    const w = t.getBBox().width;
    host.removeChild(t);
    return w;
}
