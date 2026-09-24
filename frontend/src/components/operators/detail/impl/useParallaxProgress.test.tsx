/**
 * WHICH SCROLLER THE HOOK LISTENS ON, which is the whole of what the second
 * argument buys and the one thing that can silently do nothing: a dialog's
 * content box scrolls while the window never moves, so a window listener there
 * fires never and the progress sits at 0 forever with no error to show for it.
 *
 * The discriminator is arranged so the two paths CANNOT agree. The hero's
 * viewport top is -60 over a height of 80, which is 0.7500 measured against
 * the viewport; the root's own top is -20, so root-relative the hero is 40
 * past the edge and the answer is 0.5000. One rect, two answers, and the test
 * reads which one the hook wrote.
 */
import { cleanup, render } from "@testing-library/react";
import type React from "react";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { useParallaxProgress } from "./useParallaxProgress";

/**
 * jsdom ships no IntersectionObserver and the hook constructs one on every
 * non-reduced-motion mount. This stub records the root it was given and hands
 * back its callback, because the hook gates every scroll frame on
 * `isIntersecting` and a stub that never calls back would freeze the progress
 * at whatever the mount wrote.
 */
const observed: { root: Element | null; enter: () => void }[] = [];

class StubIntersectionObserver {
    constructor(cb: IntersectionObserverCallback, options?: IntersectionObserverInit) {
        observed.push({
            root: (options?.root as Element | null) ?? null,
            enter: () => cb([{ isIntersecting: true } as IntersectionObserverEntry], this as unknown as IntersectionObserver),
        });
    }
    observe(): void {}
    unobserve(): void {}
    disconnect(): void {}
}

function Probe({ scrollRoot }: { scrollRoot?: React.RefObject<HTMLElement | null> }): React.ReactElement {
    const ref = useParallaxProgress<HTMLDivElement>("--parallax-progress", scrollRoot);
    return <div data-testid="hero" ref={ref} />;
}

function rect(top: number, height: number): () => DOMRect {
    return () => ({ top, height, bottom: top + height, left: 0, right: 0, width: 0, x: 0, y: top, toJSON: () => ({}) }) as DOMRect;
}

describe("useParallaxProgress", () => {
    beforeEach(() => {
        observed.length = 0;
        vi.stubGlobal("IntersectionObserver", StubIntersectionObserver);
    });

    afterEach(() => {
        cleanup();
        vi.unstubAllGlobals();
        vi.restoreAllMocks();
    });

    it("listens on the window by default, and measures against the viewport", () => {
        const onWindow = vi.spyOn(window, "addEventListener");
        const hero = render(<Probe />).getByTestId("hero");
        hero.getBoundingClientRect = rect(-60, 80);
        observed[0].enter();

        expect(onWindow.mock.calls.map(([type]) => type)).toContain("scroll");
        expect(observed[0].root).toBeNull();
        expect(hero.style.getPropertyValue("--parallax-progress")).toBe("0.7500");
    });

    it("listens on the scroll root when it is given one, and never on the window", () => {
        const root = document.createElement("div");
        document.body.appendChild(root);
        const onRoot = vi.spyOn(root, "addEventListener");
        const onWindow = vi.spyOn(window, "addEventListener");

        render(<Probe scrollRoot={{ current: root }} />);

        expect(onRoot.mock.calls.map(([type]) => type)).toContain("scroll");
        expect(onWindow.mock.calls.map(([type]) => type)).not.toContain("scroll");
        // Resize stays on the window: a scroll container emits none of its own.
        expect(onWindow.mock.calls.map(([type]) => type)).toContain("resize");
        expect(observed[0].root).toBe(root);
    });

    it("measures the offset against the scroll root rather than the viewport", () => {
        const root = document.createElement("div");
        document.body.appendChild(root);
        root.getBoundingClientRect = rect(-20, 400);

        const hero = render(<Probe scrollRoot={{ current: root }} />).getByTestId("hero");
        hero.getBoundingClientRect = rect(-60, 80);
        observed[0].enter();

        expect(hero.style.getPropertyValue("--parallax-progress")).toBe("0.5000");
    });

    it("drops both listeners on unmount", () => {
        const root = document.createElement("div");
        document.body.appendChild(root);
        const offRoot = vi.spyOn(root, "removeEventListener");
        const offWindow = vi.spyOn(window, "removeEventListener");

        render(<Probe scrollRoot={{ current: root }} />).unmount();

        expect(offRoot.mock.calls.map(([type]) => type)).toContain("scroll");
        expect(offWindow.mock.calls.map(([type]) => type)).toContain("resize");
    });
});
