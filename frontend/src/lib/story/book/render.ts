/**
 * BLOCKS TO MARKUP: the one mapping from the IR to (X)HTML elements, shared by
 * the single-document HTML and by every EPUB section, so there is one
 * typography. Class names are semantic (`line`, `who`, `narr`, `overlay`,
 * `scene`, `cg`, `choice`, `arm`, `cutscene`) and the look lives in `css.ts`.
 */
import type { TextNode } from "../text";
import type { Block, BookOptions, ChoiceBlock, ImageVariant, Section } from "./types";
import { el, type XElement, type XNode } from "./xml";

/**
 * The words the book itself prints, in the export's language. English
 * defaults; the export sheet passes its own translations.
 */
export interface BookLabels {
    contents: string;
    cover: string;
    colophon: string;
    /** Printed above the list of a decision's options. */
    choice: string;
    /** `If you chose "{option}"`, one arm's heading. */
    ifChose: (options: string[]) => string;
    /** A later gate on an earlier decision. */
    earlier: (options: string[]) => string;
    cutscene: string;
    figureAlt: string;
    synopsis: string;
    /** Colophon lines. */
    credit: string;
    madeWith: string;
    rights: string;
}

const quote = (options: string[]) => options.map((o) => `“${o}”`).join(" / ");

export const DEFAULT_LABELS: BookLabels = {
    contents: "Contents",
    cover: "Cover",
    colophon: "Colophon",
    choice: "Choose",
    ifChose: (options) => `If you chose ${quote(options)}`,
    earlier: (options) => `If you had chosen ${quote(options)}`,
    cutscene: "Cutscene",
    figureAlt: "Illustration",
    synopsis: "Synopsis",
    credit: "Story text and art by Hypergryph and Yostar, from Arknights.",
    madeWith: "Made with myrtle.moe.",
    rights: "Arknights and all of its story text and art belong to Hypergryph and Yostar. This book is a personal reading copy.",
};

export interface RenderContext {
    options: BookOptions;
    labels: BookLabels;
    /** Prefixed to every block id; a single document needs unique ids across sections, an EPUB file does not. */
    idPrefix: string;
    /** The `src` for an asset path, or null to leave the picture out. A scene background is asked for as a `thumb`. */
    imageSrc: (path: string, variant?: ImageVariant) => string | null;
    /** Records every colour a span used, so the stylesheet can carry a class for it. */
    colors: Set<string>;
}

/** `#FF6600` -> `tc-ff6600`; a named colour keeps its name. */
export function colorClass(color: string): string {
    return `tc-${color
        .replace(/^#/, "")
        .toLowerCase()
        .replace(/[^a-z0-9]/g, "")}`;
}

/** Text with its newlines as `<br/>`. */
function lines(value: string): XNode[] {
    const parts = value.split("\n");
    return parts.flatMap((p, i) => (i === 0 ? [p] : [el("br"), p]));
}

export function inline(nodes: readonly TextNode[], ctx: RenderContext): XNode[] {
    return nodes.flatMap((n): XNode[] => {
        if (n.kind === "text") return lines(n.value);
        if (n.kind === "inline") return [el(n.tag, {}, inline(n.children, ctx))];
        if (n.kind === "color") {
            ctx.colors.add(n.color);
            return [el("span", { class: colorClass(n.color) }, inline(n.children, ctx))];
        }
        return [el("span", { class: "para" }, inline(n.children, ctx)), el("br")];
    });
}

/** Plain option text for a heading. */
function plain(nodes: readonly TextNode[]): string {
    return nodes.map((n) => (n.kind === "text" ? n.value : plain(n.children))).join("");
}

function choice(b: ChoiceBlock, ctx: RenderContext): XElement {
    const id = `${ctx.idPrefix}${b.id}`;
    const optionText = (values: string[]) => b.options.filter((o) => values.includes(o.value)).map((o) => plain(o.nodes));
    if (b.chosen !== undefined) {
        // One path: the Doctor says the option they picked, and the branch reads on as ordinary lines.
        const picked = b.options.find((o) => o.value === b.chosen);
        return el(
            "div",
            { class: "choice path", id },
            b.asked && picked ? el("p", { class: "line doctor" }, el("b", { class: "who" }, ctx.options.nickname), " ", inline(picked.nodes, ctx)) : null,
            b.arms.flatMap((arm) => blocks(arm.blocks, ctx)),
        );
    }
    return el(
        "div",
        { class: b.asked ? "choice" : "choice recall", id },
        b.asked ? el("p", { class: "choice-label" }, ctx.labels.choice) : null,
        b.asked
            ? el(
                  "ol",
                  { class: "options" },
                  b.options.map((o) => el("li", { class: "option" }, inline(o.nodes, ctx))),
              )
            : null,
        b.arms.map((arm) => el("div", { class: "arm" }, el("p", { class: "arm-label" }, b.asked ? ctx.labels.ifChose(optionText(arm.values)) : ctx.labels.earlier(optionText(arm.values))), blocks(arm.blocks, ctx))),
    );
}

export function block(b: Block, ctx: RenderContext): XNode {
    const id = `${ctx.idPrefix}${b.id}`;
    switch (b.kind) {
        case "line":
            return el("p", { class: "line", id }, el("b", { class: "who" }, b.speaker), " ", inline(b.nodes, ctx));
        case "narration":
            return el("p", { class: "narr", id }, inline(b.nodes, ctx));
        case "overlay":
            return el("p", { class: `overlay ${b.style}`, id }, inline(b.nodes, ctx));
        case "scene": {
            // A scene is a RULE. The data names a location only by its file key
            // (`bg_cher_1`), which is a name for nobody, so nothing is captioned;
            // with backgrounds on, a small uncaptioned thumbnail sits above it.
            const src = ctx.options.images === "cg+bg" && b.url ? ctx.imageSrc(b.url, "thumb") : null;
            return el("div", { class: "scene", id }, src ? el("figure", { class: "scene-thumb" }, el("img", { src, alt: "" })) : null, el("hr"));
        }
        case "figure": {
            if (ctx.options.images === "none") return null;
            const imgs = b.images.flatMap((i) => {
                const src = ctx.imageSrc(i.url);
                return src ? [el("img", { src, alt: ctx.labels.figureAlt })] : [];
            });
            return imgs.length > 0 ? el("figure", { class: `cg ${b.source}`, id }, imgs) : null;
        }
        case "choice":
            return choice(b, ctx);
        case "cutscene":
            return el("p", { class: "cutscene", id }, ctx.labels.cutscene);
    }
}

export function blocks(list: readonly Block[], ctx: RenderContext): XNode[] {
    return list.map((b) => block(b, ctx));
}

/** A story's heading: code and phase, title, synopsis. */
export function sectionHead(s: Section, level: "h1" | "h2"): XElement {
    const eyebrow = [s.code, s.tag].filter(Boolean).join(" · ");
    return el("header", { class: "story-head" }, eyebrow ? el("p", { class: "eyebrow" }, eyebrow) : null, el(level, { class: "story-title" }, s.name), s.synopsis ? el("p", { class: "synopsis" }, lines(s.synopsis)) : null);
}

export function sectionBody(s: Section, ctx: RenderContext, level: "h1" | "h2"): XElement {
    return el("section", { class: "story", id: `${ctx.idPrefix}top` }, sectionHead(s, level), blocks(s.blocks, ctx));
}
