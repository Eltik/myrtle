import { useQuery } from "@tanstack/react-query";
import { useEffect, useState } from "react";
import { useAuth } from "#/hooks/use-auth";
import { maintenanceQueryOptions } from "#/lib/api/maintenance";
import { type IMaintenanceView, maintenanceAppliesTo, maintenanceView } from "#/lib/maintenance";
import type { MaintenanceWindow } from "#/types/generated/MaintenanceWindow";

export interface IMaintenanceState extends IMaintenanceView {
    window: MaintenanceWindow;
}

/**
 * The maintenance window this visitor should see, re-read once a minute and re-timed every second.
 *
 * `null` on the server and the first client render (the clock is the reader's, so the countdown
 * only exists after hydration), when nothing is announced, and for a signed-in player on another
 * server than the one the window is for.
 */
export function useMaintenance(): IMaintenanceState | null {
    const { user } = useAuth();
    const { data } = useQuery(maintenanceQueryOptions());
    const win = data?.window ?? null;
    const [nowSec, setNowSec] = useState<number | null>(null);

    useEffect(() => {
        if (!win) return;
        setNowSec(Math.floor(Date.now() / 1000));
        const id = setInterval(() => setNowSec(Math.floor(Date.now() / 1000)), 1000);
        return () => clearInterval(id);
    }, [win]);

    if (!win || nowSec === null) return null;
    if (!maintenanceAppliesTo(win.server, user?.server)) return null;
    return { window: win, ...maintenanceView(win, nowSec) };
}
