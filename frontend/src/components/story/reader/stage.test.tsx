/**
 * The stage's GEOMETRY, on hand-built frames. Every number here comes from the
 * shipped prefabs (`docs/story-reader-avg-prefab.md`) or the binary read
 * (`docs/story-reader-il2cpp-characters.md`). The wire now carries `facePos`
 * (14 of 21 characters on `main_15_level_main_15-08_end`), so the composite is
 * checkable in the browser too; it stays pinned here on hand-built sprites
 * because a test must not depend on which hubs a story happens to load.
 *
 * jsdom does not resolve `cqw` or `cqh`, so the assertions read the declared
 * lengths rather than computed pixels. That is the point: one canvas pixel IS
 * `min(0.078125cqw, 0.1388889cqh)` by construction, the largest 16:9 box over
 * 1280, so a declared `calc(1024 * var(--story-cpx))` is 80% of that box at
 * every viewport with no measurement.
 */
import { cleanup, fireEvent, render } from "@testing-library/react";
import { afterEach, describe, expect, it } from "vitest";
import type { Frame, SceneState, SlotState } from "#/lib/story/scene";
import type { CharacterSprite } from "#/types/generated/CharacterSprite";
import { Stage } from "./Stage";

afterEach(cleanup);

type Sprite = CharacterSprite & { facePos?: { x: number; y: number; w: number; h: number }; bodySize?: { w: number; h: number }; plate?: { x: number; y: number; w: number; h: number } };

/** Haak's hub: `facePos = (512, 120)`, `faceSize = (51, 71)`, body 1024 square. */
const HAAK: Sprite = {
    bodyUrl: "/textures/avg/characters/avg_225_haak_1/avg_225_haak_1$1.png",
    faceUrl: "/textures/avg/characters/avg_225_haak_1/1$1.png",
    facePos: { x: 512, y: 120, w: 51, h: 71 },
    bodySize: { w: 1024, h: 1024 },
};

/**
 * Amiya's hub: `facePos = (570, 233)`, `faceSize = (117, 95)` on a 1280 px
 * body texture, drawn into the same 1024 canvas px plate as Haak's. The
 * fractions are 570/1280 and NOT 570/1024, which is the whole defect.
 */
const AMIYA: Sprite = {
    bodyUrl: "/textures/avg/characters/avg_1037_amiya3_1/avg_1037_amiya3_1$1.png",
    faceUrl: "/textures/avg/characters/avg_1037_amiya3_1/1$1.png",
    facePos: { x: 570, y: 233, w: 117, h: 95 },
    bodySize: { w: 1280, h: 1280 },
};

/** The legacy hub writes the sentinels: `FacePos (-1,-1)`, `FaceSize (0,0)`. */
const LEGACY: Sprite = {
    bodyUrl: "/textures/avg/characters/char_002_amiya_1/char_002_amiya_1.png",
    faceUrl: "/textures/avg/characters/char_002_amiya_1/char_002_amiya_5.png",
    facePos: { x: -1, y: -1, w: 0, h: 0 },
};

function slot(sprite: CharacterSprite, name: string, lit: boolean, over: Partial<SlotState> = {}): SlotState {
    return { sprite, name, lit, x: 0, y: 0, alpha: 1, scale: 1, pivotX: 0.5, pivotY: 0.5, swap: 1, ...over };
}

function state(over: Partial<SceneState> = {}): SceneState {
    return { blocker: { a: 0, r: 0, g: 0, b: 0 }, slots: {}, dialogVisible: false, stickers: {}, effects: {}, focus: {}, interludes: {}, ...over };
}

function frame(over: Partial<SceneState> = {}, transitionSec = 0): Frame {
    return { state: state(over), transitionSec, holdSec: 0, blocking: true };
}

function renderStage(f: Frame): HTMLElement {
    const { container } = render(<Stage frame={f} label="stage" />);
    return container.querySelector("[data-story-stage]") as HTMLElement;
}

describe("stage geometry", () => {
    it("the canvas is the largest 16:9 box in the stage, and one canvas pixel is its width over 1280", () => {
        const stage = renderStage(frame());
        // `fit_mode="BLACK_MASK"` is the canvas, not an overlay: on the captured
        // 2340x1080 screen the non-black columns are 210..2129, a centred
        // 1920x1080 box, 1280 canvas px across it at 1.5 device px per canvas px.
        expect(stage.style.getPropertyValue("--story-cpx")).toBe("min(0.078125cqw, 0.1388889cqh)");
        const box = stage.querySelector("[data-story-canvas]") as HTMLElement;
        expect(box.style.width).toBe("calc(1280 * var(--story-cpx, min(0.078125cqw, 0.1388889cqh)))");
        expect(box.style.height).toBe("calc(720 * var(--story-cpx, min(0.078125cqw, 0.1388889cqh)))");
        expect([box.style.left, box.style.top, box.style.translate]).toEqual(["50%", "50%", "-50% -50%"]);
    });

    it("?canvas=stretch restores the pre-capture mapping: the canvas IS the stage", () => {
        const { container } = render(<Stage frame={frame()} label="stage" canvasMode="stretch" />);
        const stage = container.querySelector("[data-story-stage]") as HTMLElement;
        expect(stage.style.getPropertyValue("--story-cpx")).toBe("0.078125cqw");
        const box = container.querySelector("[data-story-canvas]") as HTMLElement;
        expect([box.style.width, box.style.height]).toEqual(["100%", "100%"]);
    });

    it("the scene, the blocker, the curtain and the overlays all live INSIDE the box; the reader's chrome does not", () => {
        const { container } = render(
            <Stage frame={frame({ curtain: { direction: 0, fill: 0.5, grad: false }, subtitle: { text: "sub", x: 0, y: 0, width: 1280, size: 24, alignment: "left" } })} label="stage">
                <div data-testid="chrome" />
            </Stage>,
        );
        const box = container.querySelector("[data-story-canvas]") as HTMLElement;
        for (const sel of ["[data-story-blocker]", "[data-story-curtain]", '[data-story-overlay="subtitle"]']) {
            expect(box.querySelector(sel), sel).not.toBeNull();
        }
        // The toolbar and the text box are the reader's own chrome and keep
        // their stage-relative margins, so they sit outside the box.
        expect(box.querySelector('[data-testid="chrome"]')).toBeNull();
        expect(container.querySelector('[data-testid="chrome"]')).not.toBeNull();
    });

    it("the trade: extend redraws the background across the whole stage, ?mask=1 gives the client's black", () => {
        const bg = { name: "bg", url: "/bg.png", x: 0, y: 0, xScale: 1, yScale: 1, adapt: "coverall" as const, widthMul: 1, heightMul: 1, rotate: 0 };
        const stage = renderStage(frame({ background: bg }));
        const fillImg = stage.querySelector("[data-story-extend-fill]") as HTMLImageElement;
        expect(fillImg.src).toContain("/bg.png");
        expect(fillImg.style.filter).toBe("blur(24px) brightness(0.45)");
        expect(fillImg.getAttribute("aria-hidden")).toBe("true");
        cleanup();
        const { container } = render(<Stage frame={frame({ background: bg })} label="stage" fill="mask" />);
        expect(container.querySelector("[data-story-extend-fill]")).toBeNull();
        cleanup();
        // Nothing behind the scene is nothing to continue: the stage stays black.
        expect(renderStage(frame()).querySelector("[data-story-extend-fill]")).toBeNull();
        cleanup();
        // Under `?canvas=stretch` the canvas IS the stage, so there is no region
        // outside it and no fill is drawn.
        const stretched = render(<Stage frame={frame({ background: bg })} label="stage" canvasMode="stretch" />);
        expect(stretched.container.querySelector("[data-story-extend-fill]")).toBeNull();
    });

    it("a body draws at its OWN wire plate, and falls back to the 1024-at-203 slot template", () => {
        // Measured: Amiya 1091.7 canvas px against her bundle's own root
        // RectTransform of 1090, Dobermann 958.9, `avg_npc_935_1` 1094.5, three
        // sizes in two frames that no shared template can produce.
        const AMIYA_PLATE: Sprite = { ...LEGACY, plate: { x: 0, y: 203, w: 1090, h: 1090 } };
        const DOBERM: Sprite = { ...LEGACY, plate: { x: 0, y: 203, w: 957, h: 957 } };
        const stage = renderStage(frame({ slots: { l: slot(AMIYA_PLATE, "amiya", true), r: slot(DOBERM, "doberm", true) } }));
        const left = stage.querySelector('[data-story-slot="l"]') as HTMLElement;
        expect(left.style.width).toBe("calc(1090 * var(--story-cpx, min(0.078125cqw, 0.1388889cqh)))");
        // The plate's bottom edge is `plate.y - plate.h/2` from the canvas bottom.
        expect(left.style.bottom).toBe("calc(-342 * var(--story-cpx, min(0.078125cqw, 0.1388889cqh)))");
        const right = stage.querySelector('[data-story-slot="r"]') as HTMLElement;
        expect(right.style.width).toBe("calc(957 * var(--story-cpx, min(0.078125cqw, 0.1388889cqh)))");
        expect(right.style.height).toBe("calc(957 * var(--story-cpx, min(0.078125cqw, 0.1388889cqh)))");
        cleanup();
        // `?plate=0` and a backend with no `plate` both take the template.
        const off = render(<Stage frame={frame({ slots: { l: slot(AMIYA_PLATE, "amiya", true) } })} label="stage" plateFromWire={false} />);
        expect((off.container.querySelector('[data-story-slot="l"]') as HTMLElement).style.width).toBe("calc(1024 * var(--story-cpx, min(0.078125cqw, 0.1388889cqh)))");
    });

    it("a plate offset moves the figure sideways inside its slot", () => {
        const shifted: Sprite = { ...LEGACY, plate: { x: -40, y: 250, w: 1094.5, h: 1094.5 } };
        const stage = renderStage(frame({ slots: { m: slot(shifted, "npc935", true) } }));
        const mid = stage.querySelector('[data-story-slot="m"]') as HTMLElement;
        expect(mid.style.left).toBe("calc(50% + calc(-40 * var(--story-cpx, min(0.078125cqw, 0.1388889cqh))))");
        expect(mid.style.width).toBe("calc(1094.5 * var(--story-cpx, min(0.078125cqw, 0.1388889cqh)))");
        // 250 - 1094.5/2 = -297.25.
        expect(mid.style.bottom).toBe("calc(-297.25 * var(--story-cpx, min(0.078125cqw, 0.1388889cqh)))");
    });

    it("a body is 1024 canvas px square at the slot's x, hung 309 px below the canvas bottom", () => {
        const stage = renderStage(frame({ slots: { l: slot(HAAK, "haak", true), r: slot(LEGACY, "amiya", false) } }));
        const left = stage.querySelector('[data-story-slot="l"]') as HTMLElement;
        // 1024 canvas px is 80% of the stage width, at every viewport.
        expect(left.style.width).toBe("calc(1024 * var(--story-cpx, min(0.078125cqw, 0.1388889cqh)))");
        expect(left.style.height).toBe("calc(1024 * var(--story-cpx, min(0.078125cqw, 0.1388889cqh)))");
        // `slot_left` sits at canvas x -200 and the body centre 203 px above the
        // slot origin, so the body's bottom edge is 309 px BELOW the canvas
        // bottom and the figure is cropped at the knees.
        expect(left.style.left).toBe("calc(50% + calc(-200 * var(--story-cpx, min(0.078125cqw, 0.1388889cqh))))");
        expect(left.style.bottom).toBe("calc(-309 * var(--story-cpx, min(0.078125cqw, 0.1388889cqh)))");
        const right = stage.querySelector('[data-story-slot="r"]') as HTMLElement;
        expect(right.style.left).toBe("calc(50% + calc(200 * var(--story-cpx, min(0.078125cqw, 0.1388889cqh))))");
    });

    it("the dim is an RGB multiply of 0.5 with alpha untouched, and the LIT slot draws last", () => {
        const stage = renderStage(frame({ slots: { l: slot(HAAK, "haak", false), r: slot(LEGACY, "amiya", true) } }));
        const lit = stage.querySelector('[data-story-lit="true"]') as HTMLElement;
        const dim = stage.querySelector('[data-story-lit="false"]') as HTMLElement;
        expect(dim.style.filter).toBe("brightness(0.5)");
        expect(lit.style.filter).toBe("");
        expect(lit.style.opacity).toBe("1");
        // `SetFocus` raises the focused slot with `SetAsLastSibling`.
        const order = Array.from(stage.querySelectorAll("[data-story-slot]")).map((n) => n.getAttribute("data-story-lit"));
        expect(order).toEqual(["false", "true"]);
    });

    it("the lighting is instant on a NEW sprite and tweened when only the focus moved", () => {
        const { container, rerender } = render(<Stage frame={frame({ slots: { m: slot(HAAK, "haak", true) } }, 0.8)} label="stage" />);
        const read = () => (container.querySelector('[data-story-slot="m"]') as HTMLElement).style.transitionDuration;
        expect(read()).toBe("0s");
        rerender(<Stage frame={frame({ slots: { m: slot(HAAK, "haak", false) } }, 0.8)} label="stage" />);
        expect(read()).toBe("0.8s");
        rerender(<Stage frame={frame({ slots: { m: slot(LEGACY, "amiya", false, { swap: 2 }) } }, 0.8)} label="stage" />);
        expect(read()).toBe("0s");
    });

    it("a face patch draws at its body-pixel rect on top of the body, and a sentinel hub draws none", () => {
        const stage = renderStage(frame({ slots: { m: slot(HAAK, "haak", true), l: slot(LEGACY, "amiya", true) } }));
        const face = stage.querySelector('[data-story-face="haak"]') as HTMLImageElement;
        // Haak: x 512, y 120, w 51, h 71 against a 1024 body.
        expect(face.style.left).toBe("50%");
        expect(face.style.top).toBe("11.71875%");
        expect(face.style.width).toBe("4.98046875%");
        expect(face.style.height).toBe("6.93359375%");
        expect(face.src).toContain("/api/assets/textures/avg/characters/avg_225_haak_1/1$1.png");
        // The face is a LATER sibling than the body, so it composites on top.
        const body = stage.querySelector('[data-story-sprite="haak"]') as HTMLElement;
        expect(body.compareDocumentPosition(face) & Node.DOCUMENT_POSITION_FOLLOWING).toBeTruthy();
        // (-1,-1) and (0,0) are the "no face patch" sentinels.
        expect(stage.querySelector('[data-story-face="amiya"]')).toBeNull();
    });

    it("a 1280 px body divides by its OWN texture size, not by the 1024 canvas plate", () => {
        const stage = renderStage(frame({ slots: { m: slot(AMIYA, "amiya3", true) } }));
        const face = stage.querySelector('[data-story-face="amiya3"]') as HTMLImageElement;
        // 570/1280, 233/1280, 117/1280, 95/1280.
        expect(face.style.left).toBe("44.53125%");
        expect(face.style.top).toBe("18.203125%");
        expect(face.style.width).toBe("9.140625%");
        expect(face.style.height).toBe("7.421875%");
        // The defect divided by 1024: 55.6640625%, 130 canvas px to the right.
        expect(face.style.left).not.toBe("55.6640625%");
    });

    it("with no bodySize on the wire the texture is measured off the body img, never assumed 1024", () => {
        const { bodySize: _drop, ...noSize } = AMIYA;
        const stage = renderStage(frame({ slots: { m: slot(noSize, "amiya3", true) } }));
        // Nothing is known about the texture yet, so no patch is placed.
        expect(stage.querySelector('[data-story-face="amiya3"]')).toBeNull();
        const body = stage.querySelector('[data-story-sprite="amiya3"]') as HTMLImageElement;
        Object.defineProperty(body, "naturalWidth", { value: 1280, configurable: true });
        Object.defineProperty(body, "naturalHeight", { value: 1280, configurable: true });
        fireEvent.load(body);
        const face = stage.querySelector('[data-story-face="amiya3"]') as HTMLImageElement;
        expect(face.style.left).toBe("44.53125%");
        expect(face.style.top).toBe("18.203125%");
        expect(face.style.width).toBe("9.140625%");
        expect(face.style.height).toBe("7.421875%");
    });

    it("a sprite with no facePos at all renders the body and nothing else: a legacy hub sends none", () => {
        const stage = renderStage(frame({ slots: { m: slot({ bodyUrl: "/b.png", faceUrl: "/f.png" }, "nofacepos", true) } }));
        expect(stage.querySelector('[data-story-sprite="nofacepos"]')).not.toBeNull();
        expect(stage.querySelector("[data-story-face]")).toBeNull();
    });
});

describe("stage layers and overlays", () => {
    const layer = (over: Record<string, unknown> = {}) => ({ name: "bg", url: "/bg.png", x: 0, y: 0, xScale: 1, yScale: 1, adapt: "coverall" as const, widthMul: 1, heightMul: 1, rotate: 0, ...over });

    it("coverall covers, showall contains and fill stretches", () => {
        for (const [adapt, fit] of [
            ["coverall", "cover"],
            ["showall", "contain"],
            ["fill", "fill"],
            // The skipped arm keeps the prefab's own 1280x720 rect, which the
            // Image stretches the sprite into: `preserveAspect` is off.
            ["native", "fill"],
        ] as const) {
            const stage = renderStage(frame({ background: layer({ adapt }) }));
            const plate = stage.querySelector('[data-story-layer="background"]') as HTMLElement;
            expect(plate.getAttribute("data-story-adapt")).toBe(adapt);
            expect((plate.querySelector("img") as HTMLImageElement).style.objectFit).toBe(fit);
            cleanup();
        }
    });

    it("the skipped adapt arm draws the image at its own texture size over its own ppu", () => {
        // `bg_cher_1` is 1024x576 at ppu 68.25, so 1024*100/68.25 = 1500.4 and
        // 576*100/68.25 = 844.0 canvas px BEFORE the script's 1.1. The game
        // measured 1500.7 x 843.5; the prefab's 1280x720 rect the reader used
        // to draw is out by 17.2%.
        const w = (1024 * 100) / 68.25;
        const h = (576 * 100) / 68.25;
        expect([Number(w.toFixed(1)), Number(h.toFixed(1))]).toEqual([1500.4, 844.0]);
        const stage = renderStage(frame({ background: layer({ adapt: "native", nativeW: w, nativeH: h, xScale: 1.1, yScale: 1.1 }) }));
        const img = stage.querySelector('[data-story-layer="background"] img') as HTMLImageElement;
        expect(img.style.width).toBe(`calc(${w} * var(--story-cpx, min(0.078125cqw, 0.1388889cqh)))`);
        expect(img.style.height).toBe(`calc(${h} * var(--story-cpx, min(0.078125cqw, 0.1388889cqh)))`);
        // 1500.4 canvas px is 117.2% of the 1280 px box and 844.0 is 117.2% of
        // 720, so the plate is WIDER than the canvas, and that is the whole
        // reason `max-w-none` has to be on the element: the preflight's
        // `img { max-width: 100% }` is a different property from `width`, an
        // inline `width` never overrides it, and 100% resolves against the
        // plate. Without the class the page rendered the declared 1500.4 as
        // exactly 1280 canvas px while the height, capped by nothing, kept its
        // 844.0 (measured on `main_00-01_beg` at 2340x1145: 1921.8 x 1267.2
        // device px where the rect is 2253.4 x 1267.2).
        expect(img.className).toContain("max-w-none");
        expect(Number(((w / 1280) * 100).toFixed(1))).toBe(117.2);
        expect(Number(((h / 720) * 100).toFixed(1))).toBe(117.2);
        // The 1.1 stays a separate scale channel on the plate, not baked in.
        expect((stage.querySelector('[data-story-layer="background"]') as HTMLElement).style.transform).toBe("scale(1.1, 1.1) rotate(0deg)");
        cleanup();
        // No size on the wire: the pre-capture 1280x720 rect, unchanged.
        const none = renderStage(frame({ background: layer({ adapt: "native" }) }));
        const noneImg = none.querySelector('[data-story-layer="background"] img') as HTMLImageElement;
        expect([noneImg.style.width, noneImg.style.height]).toEqual(["calc(1280 * var(--story-cpx, min(0.078125cqw, 0.1388889cqh)))", "calc(720 * var(--story-cpx, min(0.078125cqw, 0.1388889cqh)))"]);
        cleanup();
        // The adapting arms are unmoved: they fill the canvas box.
        const cover = renderStage(frame({ background: layer({ adapt: "coverall" }) }));
        const coverImg = cover.querySelector('[data-story-layer="background"] img') as HTMLImageElement;
        expect([coverImg.style.width, coverImg.style.height]).toEqual(["100%", "100%"]);
    });

    it("x and y are raw canvas pixels, y UP, and width/height multiply the adapted size", () => {
        const stage = renderStage(frame({ background: layer({ x: 120, y: 40, xScale: 1.1, widthMul: 1.2, rotate: -60 }) }));
        const plate = stage.querySelector('[data-story-layer="background"]') as HTMLElement;
        expect(plate.style.left).toBe("calc(120 * var(--story-cpx, min(0.078125cqw, 0.1388889cqh)))");
        expect(plate.style.top).toBe("calc(-40 * var(--story-cpx, min(0.078125cqw, 0.1388889cqh)))");
        // 1.1 localScale times a 1.2 width multiplier is one scale about the centre.
        expect(plate.style.transform).toBe("scale(1.32, 1) rotate(-60deg)");
    });

    it("a new image fades IN over the still-opaque old one, and a bare image fades the old one OUT", async () => {
        const { container, rerender } = render(<Stage frame={frame({ image: layer({ name: "cg1", url: "/cg1.png" }) }, 1)} label="stage" />);
        rerender(<Stage frame={frame({ image: layer({ name: "cg2", url: "/cg2.png" }) }, 1)} label="stage" />);
        const plates = Array.from(container.querySelectorAll('[data-story-layer="image"]')) as HTMLElement[];
        expect(plates).toHaveLength(2);
        // The OLD plate holds at full opacity; the new one fades in over it.
        expect(plates[0].style.animationName).toBe("");
        expect(plates[1].style.animationName).toBe("story-fade-in");
        expect(plates[1].style.animationDuration).toBe("1s");
        rerender(<Stage frame={frame({}, 1)} label="stage" />);
        const out = container.querySelector('[data-story-layer="image"]') as HTMLElement;
        expect(out.style.animationName).toBe("story-fade-out");
    });

    it("the blocker is an rgba of CLAMPED 0..1 channels, so a=1 r=g=b=1 is white", () => {
        const stage = renderStage(frame({ blocker: { a: 1, r: 1, g: 1, b: 1 } }));
        expect((stage.querySelector("[data-story-blocker]") as HTMLElement).style.backgroundColor).toBe("rgb(255, 255, 255)");
    });

    it("focusout is a blur and nothing else", () => {
        const stage = renderStage(frame({ background: layer(), focus: { bg: 1 } }));
        expect((stage.querySelector('[data-story-layer="background"]') as HTMLElement).style.filter).toBe("blur(12.00px)");
    });

    it("a subtitle is anchored top-left with y DOWNWARD, sized in canvas pixels", () => {
        const stage = renderStage(frame({ subtitle: { text: "sub", x: 300, y: 370, width: 700, size: 24, alignment: "center" } }));
        const node = stage.querySelector('[data-story-overlay="subtitle"]') as HTMLElement;
        expect(node.style.left).toBe("calc(300 * var(--story-cpx, min(0.078125cqw, 0.1388889cqh)))");
        expect(node.style.top).toBe("calc(370 * var(--story-cpx, min(0.078125cqw, 0.1388889cqh)))");
        expect(node.style.width).toBe("calc(700 * var(--story-cpx, min(0.078125cqw, 0.1388889cqh)))");
        expect(node.style.fontSize).toBe("calc(24 * var(--story-cpx, min(0.078125cqw, 0.1388889cqh)))");
        expect(node.style.textAlign).toBe("center");
    });

    it("curtain direction 6 grows from the LEFT edge and 2 from the RIGHT, measured", () => {
        // Measured across a single tap each: at 6 the visible columns run
        // 0..1919, 1472..1919, 1677..1919, 1813..1919, 1890..1919, so the black
        // grows from the left; at 2 they run 0..1584 down to 0..29.
        for (const [direction, edge, anchored] of [
            [6, "left", "0px"],
            [2, "right", ""],
        ] as const) {
            const stage = renderStage(frame({ curtain: { direction, fill: 0.5, grad: false } }));
            const node = stage.querySelector("[data-story-curtain]") as HTMLElement;
            expect(node.getAttribute("data-story-curtain")).toBe(edge);
            expect(node.style.left).toBe(anchored);
            cleanup();
        }
        // 0 = top and 4 = bottom stay the marked guess: the corpus only ever
        // issues them as a pair on one halt, so a capture cannot tell them apart.
        for (const [direction, edge] of [
            [0, "top"],
            [4, "bottom"],
        ] as const) {
            const stage = renderStage(frame({ curtain: { direction, fill: 0.5, grad: false } }));
            expect((stage.querySelector("[data-story-curtain]") as HTMLElement).getAttribute("data-story-curtain")).toBe(edge);
            cleanup();
        }
    });

    it("curtain directions 2 and 6 scale WIDTH and every other one scales height", () => {
        for (const [direction, axis] of [
            [0, "height"],
            [2, "width"],
            [4, "height"],
            [6, "width"],
            [1, "height"],
            [5, "height"],
        ] as const) {
            const stage = renderStage(frame({ curtain: { direction, fill: 0.5, grad: false } }));
            const node = stage.querySelector("[data-story-curtain]") as HTMLElement;
            expect([node.style.width, node.style.height]).toEqual(axis === "width" ? ["50%", "100%"] : ["100%", "50%"]);
            cleanup();
        }
    });

    it("a largebg strip moves in canvas pixels, not in fractions of its own width", () => {
        const stage = renderStage(frame({ panels: { urls: ["/p1.png", "/p2.png"], widths: [920, 920], height: 720, rows: 1, x: -720, y: 0 } }));
        const strip = stage.querySelector("[data-story-panels]") as HTMLElement;
        expect(strip.style.width).toBe("calc(1840 * var(--story-cpx, min(0.078125cqw, 0.1388889cqh)))");
        expect(strip.style.transform).toBe("translate(calc(-720 * var(--story-cpx, min(0.078125cqw, 0.1388889cqh))), calc(0 * var(--story-cpx, min(0.078125cqw, 0.1388889cqh))))");
    });
});
