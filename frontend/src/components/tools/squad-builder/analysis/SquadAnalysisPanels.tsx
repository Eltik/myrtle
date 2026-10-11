import * as React from "react";
import { useLocalStorageState } from "#/hooks/use-local-storage-state";
import { useT } from "#/lib/i18n";
import type { TypedT } from "#/lib/i18n/messages";
import type { SquadAnalysis } from "#/types/generated/SquadAnalysis";
import type { SquadAnalyzeRequest } from "#/types/generated/SquadAnalyzeRequest";
import type { IOperatorIndexEntry } from "#/types/operators";
import type { IResolvedSeat, IStageContext } from "../model";
import type { StartDpFrom } from "../state";
import { CoveragePanel } from "./CoveragePanel";
import { EconomyPanel } from "./EconomyPanel";
import { ReadinessPanel } from "./ReadinessPanel";
import { SynergyPanel } from "./SynergyPanel";
import { type IChipContext, type ISeatOperator, opName, type PanelState } from "./shared";
import type { messages as sharedMessages } from "./shared.messages";
import { ThemesPanel } from "./ThemesPanel";

/**
 * The five analysis panels (Coverage, Economy, Synergy, Readiness, Themes),
 * fed by the page's one debounced `POST /squads/analyze` query (`analysis`),
 * which the page also reads for the DPS calculator hand-off.
 *
 * `seats` and `support` are the resolved seats the request was built from, so
 * a panel can show a seat's build without re-deriving it; `operators` is the
 * index by `char_id` for names, rarities, classes and portraits.
 */
export interface ISquadAnalysisPanelsProps {
    /** `null` when no seat is filled: every panel shows its empty state. */
    request: SquadAnalyzeRequest | null;
    /** The page's analysis query for the debounced request. */
    analysis: { data: SquadAnalysis | undefined; isError: boolean; isPlaceholderData: boolean };
    seats: ReadonlyArray<IResolvedSeat | null>;
    support: IResolvedSeat | null;
    operators: ReadonlyMap<string, IOperatorIndexEntry>;
    startDp: number;
    /** The Economy stepper and the draggable starting-DP line call this. */
    onStartDpChange: (startDp: number) => void;
    /** Where the starting DP came from: the stage card or the page itself. */
    startDpFrom: StartDpFrom;
    stage: IStageContext | null;
    /** Readiness rows and Synergy chips highlight a seat card on hover. */
    onHoverSeat: (seat: number | null) => void;
    /** "Open in DPS calculator" from a Synergy receiver row. */
    onOpenInDps: (seat: number) => void;
}

type PanelId = "coverage" | "economy" | "synergy" | "readiness" | "themes";

const COLLAPSED_KEY = "squad-builder:panels";

export function SquadAnalysisPanels({ request, analysis: query, support, operators, startDp, onStartDpChange, startDpFrom, stage, onHoverSeat, onOpenInDps }: ISquadAnalysisPanelsProps): React.ReactElement | null {
    const t: TypedT<typeof sharedMessages> = useT("tools");
    const data = query.data;

    const [collapsed, setCollapsed] = useLocalStorageState<PanelId[]>(COLLAPSED_KEY, [], {
        parse: (raw) => {
            const parsed = JSON.parse(raw) as unknown;
            return Array.isArray(parsed) ? (parsed.filter((p) => typeof p === "string") as PanelId[]) : undefined;
        },
    });
    const toggle = (id: PanelId) => () => setCollapsed((prev) => (prev.includes(id) ? prev.filter((p) => p !== id) : [...prev, id]));
    const isCollapsed = (id: PanelId) => collapsed.includes(id);

    const state: PanelState = request === null ? "empty" : query.isError ? "error" : data ? "ready" : "loading";
    // Dimmed only while a newer answer loads over the last one; the Economy ladder follows a drag on its own.
    const stale = query.isPlaceholderData;

    const ctx = React.useMemo<IChipContext>(() => ({ operators, onHoverSeat }), [operators, onHoverSeat]);
    // Seats as the response names them, so a chip never points at a seat edited since.
    const seatOps = React.useMemo(() => {
        const out = new Map<number, ISeatOperator>();
        for (const row of data?.readiness.rows ?? []) out.set(row.seat, { charId: row.char_id, support: row.support });
        return out;
    }, [data]);

    const common = { state, stale };
    // Seats whose operator's texts have no English reading yet: the panels left out what reads their wording.
    const unread = (data?.unread_seats ?? []).map((seat) => seatOps.get(seat)).filter((o): o is ISeatOperator => o !== undefined);
    return (
        <div className="grid grid-cols-1 items-start gap-4 lg:grid-cols-2">
            {state === "ready" && unread.length > 0 && <p className="rounded-lg border border-border bg-muted/40 px-3 py-2 text-[12.5px] text-muted-foreground lg:col-span-2">{t("squad.analysis.unread", { names: unread.map((o) => opName(operators, o.charId)).join(", ") })}</p>}
            <CoveragePanel {...common} collapsed={isCollapsed("coverage")} onToggle={toggle("coverage")} coverage={data?.coverage} stage={stage} ctx={ctx} />
            <EconomyPanel {...common} collapsed={isCollapsed("economy")} onToggle={toggle("economy")} economy={data?.economy} startDp={startDp} onStartDpChange={onStartDpChange} startDpFrom={startDpFrom} stage={stage} hasSupport={support !== null} ctx={ctx} />
            <SynergyPanel {...common} className="lg:col-span-2" collapsed={isCollapsed("synergy")} onToggle={toggle("synergy")} synergy={data?.synergy} seatOps={seatOps} onOpenInDps={onOpenInDps} ctx={ctx} />
            <ReadinessPanel {...common} collapsed={isCollapsed("readiness")} onToggle={toggle("readiness")} readiness={data?.readiness} ctx={ctx} />
            <ThemesPanel {...common} collapsed={isCollapsed("themes")} onToggle={toggle("themes")} themes={data?.themes} seatOps={seatOps} ctx={ctx} />
        </div>
    );
}
