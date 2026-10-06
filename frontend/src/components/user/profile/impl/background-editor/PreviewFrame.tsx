import { type CSSProperties, type ReactNode, useEffect, useState } from "react";
import { createPortal } from "react-dom";

interface IPreviewFrameProps {
    /** The frame's title, for the tools that list frames; the frame itself is hidden from assistive tech. */
    title: string;
    /** The viewport width the content lays out at: its media queries read this, not the editor's width. */
    viewportWidth: number;
    height: number;
    style?: CSSProperties;
    /** Told the frame's document once it is ready, `null` on unmount, so the caller can measure inside it. */
    onDocument: (doc: Document | null) => void;
    children: ReactNode;
}

const BLANK = "<!doctype html><html><head></head><body></body></html>";

/**
 * Renders `children` into a same-origin iframe `viewportWidth` px wide, through a React
 * portal (context, queries and i18n carry over). An iframe and not a scaled div because
 * the header's layout is media queries: only a document of its own sees a 390 px viewport
 * while the editor sits on a 1500 px one. The page's stylesheets and `<html>` attributes
 * (the theme class) are mirrored into it and kept in sync, so a Vite hot update, a lazy
 * chunk's CSS or a theme switch shows here too. The frame is decoration: it takes no
 * pointer events and is hidden from assistive tech, and the caller lays its own
 * interactive layer over it.
 *
 * A constraint on what goes inside: the portal moves the DOM, not the JavaScript. A
 * component in `children` that reads `window`, `document`, `matchMedia` or a hook built on
 * them (`useMediaQuery`) reads the EDITOR's window, not the frame's, so at Phone it would
 * see the 1500 px editor. Only CSS (media queries, container sizes) sees the previewed
 * viewport; the header must lay itself out by CSS alone to preview right.
 */
export function PreviewFrame({ title, viewportWidth, height, style, onDocument, children }: IPreviewFrameProps) {
    const [doc, setDoc] = useState<Document | null>(null);

    useEffect(() => {
        if (!doc) return;
        const stop = mirrorPage(document, doc);
        onDocument(doc);
        return () => {
            stop();
            onDocument(null);
        };
    }, [doc, onDocument]);

    return (
        <>
            <iframe title={title} aria-hidden="true" tabIndex={-1} srcDoc={BLANK} onLoad={(e) => setDoc(e.currentTarget.contentDocument)} style={{ ...style, width: viewportWidth, height, border: 0, pointerEvents: "none", colorScheme: "normal", background: "transparent" }} />
            {doc && createPortal(children, doc.body)}
        </>
    );
}

/** The `<head>` nodes that style the page. */
const STYLE_NODES = 'link[rel="stylesheet"], style';
/** Marks a node this mirror put in the frame, so a sync never touches anything else. */
const MIRRORED = "data-preview-mirror";
/** How long a burst of page mutations is coalesced before one sync; a timer, so a tab that paints no frames still syncs. */
const SYNC_DELAY_MS = 16;

/**
 * Copies `src`'s stylesheets and `<html>` attributes into `dst` and keeps them in step
 * until the returned stop function runs. A changed `<style>` (a hot update) has its text
 * replaced in place; a link already copied is left alone, so it does not reload.
 */
function mirrorPage(src: Document, dst: Document): () => void {
    const clones = new Map<Element, Element>();
    for (const el of [dst.documentElement, dst.body]) {
        el.style.setProperty("margin", "0");
        el.style.setProperty("background", "transparent", "important");
        // Never a scrollbar: one would narrow the viewport, rewrap the header, and change the height the frame is sized to.
        el.style.setProperty("overflow", "hidden", "important");
    }

    const sync = () => {
        const sources = [...src.head.querySelectorAll(STYLE_NODES)];
        const live = new Set(sources);
        for (const [source, clone] of clones) {
            if (live.has(source)) continue;
            clone.remove();
            clones.delete(source);
        }
        for (const source of sources) {
            const clone = clones.get(source);
            if (!clone) {
                const copy = source.cloneNode(true) as Element;
                copy.setAttribute(MIRRORED, "");
                clones.set(source, copy);
            } else if (source.tagName === "STYLE" && clone.textContent !== source.textContent) {
                clone.textContent = source.textContent;
            }
        }
        // The cascade is source order: re-append only when the order differs, since moving a link refetches it.
        const wanted = sources.map((source) => clones.get(source)).filter((c): c is Element => c !== undefined);
        const current = [...dst.head.querySelectorAll(`[${MIRRORED}]`)];
        if (wanted.length !== current.length || wanted.some((c, i) => c !== current[i])) dst.head.append(...wanted);

        const html = dst.documentElement;
        for (const { name, value } of [...src.documentElement.attributes]) {
            if (name === "style") continue;
            if (html.getAttribute(name) !== value) html.setAttribute(name, value);
        }
        for (const { name } of [...html.attributes]) if (name !== "style" && !src.documentElement.hasAttribute(name)) html.removeAttribute(name);
        // The theme's custom properties can sit on `<html style>`; the frame keeps its own margin and transparency.
        // The dialog's scroll lock writes `overflow-x`, `overflow-y` and `scrollbar-gutter: stable` there too. Copied, the
        // gutter reserved a classic scrollbar's width inside the frame, so the header laid out at 1470 px in a 1485 px frame
        // (1438 px wide at 16 px, against the page's 1440 px at 22.5 px; 15 px scrollbar, 2026-10-06). `item()` lists
        // longhands, so `overflow` alone never matched `overflow-y`.
        const rootStyle = src.documentElement.style;
        for (let i = 0; i < rootStyle.length; i++) {
            const name = rootStyle.item(i);
            if (name.startsWith("margin") || name.startsWith("background") || name.startsWith("overflow") || name.startsWith("padding") || name.startsWith("scrollbar")) continue;
            html.style.setProperty(name, rootStyle.getPropertyValue(name));
        }
    };

    let timer: ReturnType<typeof setTimeout> | undefined;
    const schedule = () => {
        timer ??= setTimeout(() => {
            timer = undefined;
            sync();
        }, SYNC_DELAY_MS);
    };
    sync();
    const observer = new MutationObserver(schedule);
    observer.observe(src.head, { childList: true, subtree: true, characterData: true });
    observer.observe(src.documentElement, { attributes: true });
    return () => {
        observer.disconnect();
        clearTimeout(timer);
    };
}
