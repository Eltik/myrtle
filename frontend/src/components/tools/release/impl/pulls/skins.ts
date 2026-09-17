/**
 * What the skins planner has already committed, in Originite Prime.
 *
 * The other tab is where a player decides which outfits they are buying, and that
 * decision spends the same currency a pull does. Reading it here rather than asking
 * again is the point: an outfit picked in the Planner tab should stop showing up as
 * pulls in this one.
 *
 * This is deliberately READ-ONLY. `useStoredState` in `planStore.ts` owns that state
 * and writes it back on every change, debounced; mounting a second copy of it here
 * would give the plan two writers. So this reads the same two sources it does, the
 * local mirror and the signed-in copy, and never writes.
 */

import { useQuery } from "@tanstack/react-query";
import * as React from "react";
import { releasePlanQueryOptions, releaseSkinsQueryOptions } from "#/lib/api/release";
import { loadState } from "../plan";

export interface ISkinCommitment {
    /** Originite Prime the picked outfits cost together. */
    originite: number;
    /** How many outfits are picked. */
    count: number;
    /** Picks whose price could not be resolved, so the total understates by that many. */
    unpriced: number;
}

const EMPTY: ISkinCommitment = { originite: 0, count: 0, unpriced: 0 };

export function useSkinCommitment(uid: string | null): ISkinCommitment {
    const skins = useQuery(releaseSkinsQueryOptions());
    const server = useQuery(releasePlanQueryOptions(uid));
    const [local, setLocal] = React.useState<string[]>([]);

    // localStorage is read in an effect because the server pass has none, and a
    // mismatched first paint would warn.
    React.useEffect(() => {
        setLocal(Object.keys(loadState().picks));
    }, []);

    return React.useMemo(() => {
        // The signed-in copy wins when it exists; it is what the other tab last saved.
        const picks = uid && server.data ? server.data.picks : local;
        if (picks.length === 0) return EMPTY;

        const prices = new Map<string, number>();
        const data = skins.data;
        if (data) {
            for (const s of data.newSkins) prices.set(s.skinId, s.price.price);
            for (const r of data.rerunForecasts) for (const s of r.skins) prices.set(s.skinId, s.price.price);
            for (const o of data.reviewPool) prices.set(o.skinId, o.price.price);
        }

        let originite = 0;
        let unpriced = 0;
        for (const id of picks) {
            const price = prices.get(id);
            if (price === undefined) unpriced += 1;
            else originite += price;
        }
        return { originite, count: picks.length, unpriced };
    }, [uid, server.data, local, skins.data]);
}
