import { useQuery } from "@tanstack/react-query";
import { type Dispatch, useEffect, useRef } from "react";
import { decodeDpsImport } from "#/components/tools/squad-builder/model";
import type { messages as routeErrorMessages } from "#/components/ui/route-error.messages";
import { toastManager } from "#/components/ui/toast";
import { dpsOperatorsQueryOptions } from "#/lib/api/dps";
import { useT } from "#/lib/i18n";
import type { TypedT } from "#/lib/i18n/messages";
import type { messages } from "../DpsCalculator.messages";
import { importSeatToConfig } from "./importSeats";
import type { DpsAction } from "./types";
import { legacyBuffsToZones } from "./zones";

/**
 * Read `?import=` once: wait for the stored state to hydrate (a later HYDRATE
 * would replace the added cards) and for the engine's operator list, add one
 * card per supported seat with its build, then let the route drop the param.
 */
export function useDpsImport(importParam: string | undefined, hydrationToken: number, dispatch: Dispatch<DpsAction>, onConsumed: (() => void) | undefined): void {
    const t: TypedT<typeof messages> = useT("tools");
    const tc: TypedT<typeof routeErrorMessages> = useT("common");
    const { data: operators, isError, errorUpdatedAt, refetch } = useQuery(dpsOperatorsQueryOptions());
    const doneRef = useRef(false);
    const toastedErrorRef = useRef(0);

    // A failed operator list would drop the squad silently: say so once per failure and keep
    // the param, so the retry (or a reload) still adds the cards.
    useEffect(() => {
        if (doneRef.current || !importParam || !isError || toastedErrorRef.current === errorUpdatedAt) return;
        toastedErrorRef.current = errorUpdatedAt;
        toastManager.add({ id: "dps-import-failed", title: t("dps.import.failed"), type: "error", actionProps: { children: tc("routeError.retry"), onClick: () => void refetch() } });
    }, [importParam, isError, errorUpdatedAt, refetch, t, tc]);

    useEffect(() => {
        if (doneRef.current || !importParam || hydrationToken === 0 || !operators) return;
        doneRef.current = true;
        const data = decodeDpsImport(importParam);
        const byId = new Map(operators.map((op) => [op.id, op]));
        let added = 0;
        let skipped = 0;
        for (const seat of data?.seats ?? []) {
            const op = byId.get(seat.charId);
            if (!op) {
                skipped++;
                continue;
            }
            // The seat's buffs arrive in the legacy shape; the card holds them as its own zones.
            const config = importSeatToConfig(seat, op);
            const zones = legacyBuffsToZones(config.buffs);
            dispatch({ type: "ADD_INSTANCE", op, config: { ...config, buffs: {}, ...(zones ? { zones } : {}) } });
            added++;
        }
        if (added > 0) toastManager.add({ id: `dps-import-${Date.now()}`, title: t("dps.import.added", { count: added }), type: "success" });
        if (skipped > 0) toastManager.add({ id: `dps-import-skip-${Date.now()}`, title: t("dps.import.skipped", { count: skipped }), type: "warning" });
        onConsumed?.();
    }, [importParam, hydrationToken, operators, dispatch, onConsumed, t]);
}
