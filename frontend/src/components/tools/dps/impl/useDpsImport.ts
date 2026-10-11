import { useQuery } from "@tanstack/react-query";
import { type Dispatch, useEffect, useRef } from "react";
import { decodeDpsImport } from "#/components/tools/squad-builder/model";
import { toastManager } from "#/components/ui/toast";
import { dpsOperatorsQueryOptions } from "#/lib/api/dps";
import { useT } from "#/lib/i18n";
import type { TypedT } from "#/lib/i18n/messages";
import type { messages } from "../DpsCalculator.messages";
import { importSeatToConfig } from "./importSeats";
import type { DpsAction } from "./types";

/**
 * Read `?import=` once: wait for the stored state to hydrate (a later HYDRATE
 * would replace the added cards) and for the engine's operator list, add one
 * card per supported seat with its build, then let the route drop the param.
 */
export function useDpsImport(importParam: string | undefined, hydrationToken: number, dispatch: Dispatch<DpsAction>, onConsumed: (() => void) | undefined): void {
    const t: TypedT<typeof messages> = useT("tools");
    const { data: operators } = useQuery(dpsOperatorsQueryOptions());
    const doneRef = useRef(false);

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
            dispatch({ type: "ADD_INSTANCE", op, config: importSeatToConfig(seat, op) });
            added++;
        }
        if (added > 0) toastManager.add({ id: `dps-import-${Date.now()}`, title: t("dps.import.added", { count: added }), type: "success" });
        if (skipped > 0) toastManager.add({ id: `dps-import-skip-${Date.now()}`, title: t("dps.import.skipped", { count: skipped }), type: "warning" });
        onConsumed?.();
    }, [importParam, hydrationToken, operators, dispatch, onConsumed, t]);
}
