/**
 * THE BOOK'S ONE STYLESHEET, for the EPUB and the HTML document alike. It is
 * written for reading systems first: relative units, no fixed widths, colour
 * only where the story itself carries one, and every rule degrades to plain
 * paragraphs when a reader ignores publisher styles (Kindle, some Kobo modes).
 */
import { colorClass } from "./render";
import type { Typeface } from "./types";

export interface FontFaces {
    regular: string;
    bold: string;
    /** `woff` or `woff2`, for the `format()` hint. */
    format: "woff" | "woff2";
}

const FAMILY: Record<Typeface, string> = {
    inter: `"Inter", sans-serif`,
    opendyslexic: `"OpenDyslexic", sans-serif`,
    device: "inherit",
};

const FACE_NAME: Record<Exclude<Typeface, "device">, string> = { inter: "Inter", opendyslexic: "OpenDyslexic" };

/** A colour name or hex the CSS can carry verbatim; anything else is left uncoloured. */
function cssColor(color: string): string | null {
    return /^#[0-9a-fA-F]{3,8}$/.test(color) || /^[a-zA-Z]+$/.test(color) ? color : null;
}

export function bookCss(typeface: Typeface, faces: FontFaces | null, colors: Iterable<string>): string {
    const out: string[] = [];
    if (typeface !== "device" && faces) {
        const name = FACE_NAME[typeface];
        out.push(`@font-face{font-family:"${name}";font-weight:400;font-style:normal;src:url("${faces.regular}") format("${faces.format}")}`);
        out.push(`@font-face{font-family:"${name}";font-weight:700;font-style:normal;src:url("${faces.bold}") format("${faces.format}")}`);
    }
    out.push(`body{font-family:${FAMILY[typeface]};line-height:1.5;margin:0 5%;}`);
    out.push(
        "h1,h2{line-height:1.2;font-weight:700;page-break-after:avoid;break-after:avoid}",
        "h1{font-size:1.6em;margin:1.5em 0 .5em}",
        ".story{page-break-before:always;break-before:page}",
        ".story-head{margin:2em 0 1.5em}",
        ".story-title{font-size:1.5em;margin:.2em 0 .6em}",
        ".eyebrow{font-size:.8em;letter-spacing:.08em;text-transform:uppercase;opacity:.7;margin:0}",
        ".synopsis{font-style:italic;opacity:.8;margin:0 0 1em}",
        "p{margin:0 0 .6em;text-indent:0}",
        ".who{font-weight:700}",
        ".narr{opacity:.85}",
        ".overlay{text-align:center;margin:1em 0}",
        ".sticker{font-size:.95em}",
        ".scene{margin:1.6em 0 1.2em;text-align:center;page-break-inside:avoid;break-inside:avoid}",
        ".scene hr{border:0;border-top:1px solid currentColor;opacity:.3;width:30%;margin:0 auto}",
        ".scene-thumb{margin:0 auto .8em;width:33%;text-align:center}",
        ".scene-thumb img{width:100%;height:auto}",
        "figure{margin:1.2em 0;text-align:center;page-break-inside:avoid;break-inside:avoid}",
        "figure img{max-width:100%;height:auto}",
        ".choice{margin:1.2em 0;padding:.6em 0 .6em 1em;border-left:3px solid currentColor}",
        ".choice.path{border-left:0;padding:0}",
        ".choice.recall{border-left-style:dashed}",
        ".choice-label,.arm-label{font-size:.8em;letter-spacing:.06em;text-transform:uppercase;opacity:.7;margin:0 0 .3em}",
        ".options{margin:0 0 .6em;padding-left:1.4em}",
        ".option{margin:0 0 .2em}",
        ".arm{margin:.8em 0 0 0}",
        ".arm .arm-label{font-style:normal}",
        ".cutscene{text-align:center;font-size:.8em;letter-spacing:.06em;text-transform:uppercase;opacity:.6}",
        ".cover{margin:0;padding:0;text-align:center}",
        ".part{text-align:center;padding-top:10%}",
        ".part-art{margin:0 0 1.5em}",
        ".cover img{max-width:100%;max-height:100%}",
        ".colophon p{font-size:.85em}",
        "nav ol{list-style:none;padding-left:1em}",
        "nav li{margin:.2em 0}",
    );
    for (const c of [...new Set(colors)].sort()) {
        const value = cssColor(c);
        if (value) out.push(`.${colorClass(c)}{color:${value}}`);
    }
    return `${out.join("\n")}\n`;
}
