/**
 * The DPS and HPS calculators keep the visitor's setup in `localStorage` and can also
 * open a share link (`?s=`). These cases pin who may write the stored setup: a link is
 * shown without being saved, and the visitor's first change saves as before.
 */
import { act, renderHook } from "@testing-library/react";
import { beforeEach, describe, expect, it } from "vitest";
import { type ExternalHydration, usePersistedReducer } from "./usePersistedReducer";

interface ISetup {
    cards: string[];
    defense: number;
}

type Action = { type: "HYDRATE"; state: ISetup } | { type: "SET_DEFENSE"; defense: number } | { type: "SAME_CONTENT" };

const KEY = "persisted-reducer-test-v1";
const INITIAL: ISetup = { cards: [], defense: 0 };
const SAVED: ISetup = { cards: ["char_saved"], defense: 300 };
const LINK: ISetup = { cards: ["char_link_a", "char_link_b"], defense: 1200 };

function reducer(state: ISetup, action: Action): ISetup {
    switch (action.type) {
        case "HYDRATE":
            return action.state;
        case "SET_DEFENSE":
            return { ...state, defense: action.defense };
        // What a refreshed operator list does: equal content, new objects.
        case "SAME_CONTENT":
            return { ...state, cards: [...state.cards] };
    }
}

function merge(raw: unknown): ISetup | null {
    return raw && typeof raw === "object" && Array.isArray((raw as ISetup).cards) ? (raw as ISetup) : null;
}

function render(external: ExternalHydration<ISetup>) {
    return renderHook(({ ext }) => usePersistedReducer<ISetup, Action>(KEY, reducer, INITIAL, merge, (state) => ({ type: "HYDRATE", state }), ext), { initialProps: { ext: external } });
}

function stored(): unknown {
    const raw = window.localStorage.getItem(KEY);
    return raw === null ? null : JSON.parse(raw);
}

describe("usePersistedReducer", () => {
    beforeEach(() => {
        window.localStorage.clear();
        window.localStorage.setItem(KEY, JSON.stringify(SAVED));
    });

    it("shows a link's setup without replacing the saved one", () => {
        const { result } = render({ status: "ready", state: LINK });
        expect(result.current.state).toEqual(LINK);
        expect(result.current.hydrationToken).toBe(1);
        expect(stored()).toEqual(SAVED);
    });

    it("saves the link's setup once the visitor changes it", () => {
        const { result } = render({ status: "ready", state: LINK });
        act(() => result.current.dispatch({ type: "SET_DEFENSE", defense: 1500 }));
        expect(stored()).toEqual({ ...LINK, defense: 1500 });
        // Every later change saves too, including one back to the link's own value.
        act(() => result.current.dispatch({ type: "SET_DEFENSE", defense: 1200 }));
        expect(stored()).toEqual(LINK);
    });

    it("does not count a change of identity with the same content as an edit", () => {
        const { result } = render({ status: "ready", state: LINK });
        act(() => result.current.dispatch({ type: "SAME_CONTENT" }));
        expect(result.current.state).not.toBe(LINK);
        expect(stored()).toEqual(SAVED);
    });

    it("waits for a pending link and still keeps the saved setup", () => {
        const { result, rerender } = render({ status: "pending" });
        expect(result.current.hydrationToken).toBe(0);
        expect(stored()).toEqual(SAVED);
        rerender({ ext: { status: "ready", state: LINK } });
        expect(result.current.state).toEqual(LINK);
        expect(stored()).toEqual(SAVED);
    });

    it("hydrates from storage when the link resolved to nothing, and saves changes as before", () => {
        const { result } = render({ status: "ready", state: null });
        expect(result.current.state).toEqual(SAVED);
        act(() => result.current.dispatch({ type: "SET_DEFENSE", defense: 600 }));
        expect(stored()).toEqual({ ...SAVED, defense: 600 });
    });

    it("hydrates from storage without a link, and saves changes as before", () => {
        const { result } = render({ status: "none" });
        expect(result.current.state).toEqual(SAVED);
        act(() => result.current.dispatch({ type: "SET_DEFENSE", defense: 900 }));
        expect(stored()).toEqual({ ...SAVED, defense: 900 });
    });
});
