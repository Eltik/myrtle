/**
 * jsdom implements none of `matchMedia`, `ResizeObserver` or
 * `IntersectionObserver`, which the theme store, the popover primitives and
 * the parallax heroes respectively call during render. Without these every
 * component test that mounts site chrome dies before it asserts anything.
 */
if (typeof window !== "undefined") {
    if (!window.matchMedia) {
        window.matchMedia = (query: string): MediaQueryList =>
            ({
                matches: false,
                media: query,
                onchange: null,
                addListener: () => {},
                removeListener: () => {},
                addEventListener: () => {},
                removeEventListener: () => {},
                dispatchEvent: () => false,
            }) as MediaQueryList;
    }

    if (!window.ResizeObserver) {
        window.ResizeObserver = class {
            observe() {}
            unobserve() {}
            disconnect() {}
        } as unknown as typeof ResizeObserver;
    }

    // The stub never reports an intersection, which is the honest answer from a
    // DOM with no layout: a test that needs the callback supplies its own.
    if (!window.IntersectionObserver) {
        window.IntersectionObserver = class {
            readonly root = null;
            readonly rootMargin = "";
            readonly thresholds: readonly number[] = [];
            observe() {}
            unobserve() {}
            disconnect() {}
            takeRecords(): IntersectionObserverEntry[] {
                return [];
            }
        } as unknown as typeof IntersectionObserver;
    }
}
