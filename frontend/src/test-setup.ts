/**
 * jsdom implements neither `matchMedia` nor `ResizeObserver`, both of which
 * the theme store and the popover primitives call during render. Without these
 * every component test that mounts site chrome dies before it asserts
 * anything.
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
}
