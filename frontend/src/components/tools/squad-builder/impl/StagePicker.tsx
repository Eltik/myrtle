import { useQuery } from "@tanstack/react-query";
import { useVirtualizer } from "@tanstack/react-virtual";
import { MapIcon } from "lucide-react";
import * as React from "react";
import { StagePreview } from "#/components/stages/list/impl/StagePreview";
import { Dialog, DialogDescription, DialogHeader, DialogPanel, DialogPopup, DialogTitle } from "#/components/ui/dialog";
import { Input } from "#/components/ui/input";
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from "#/components/ui/select";
import { useMediaQuery } from "#/hooks/use-media-query";
import { type IStageIndexEntry, stageIndexQueryOptions, stagePreviewCandidates } from "#/lib/api/stages";
import { useGamedataServer, useT } from "#/lib/i18n";
import type { TypedT } from "#/lib/i18n/messages";
import { STAGE_GROUP_LABEL_KEY, STAGE_GROUP_TONE, STAGE_GROUPS, type StageGroupKey } from "#/lib/registry/stage-groups";
import { searchAndRank } from "#/lib/search/fuzzy";
import { cn } from "#/lib/utils";
import type { messages } from "./StagePicker.messages";

type PickerT = TypedT<typeof messages>;

/** Grid columns below and from the `sm` breakpoint; keep in step with the row's `grid-cols-*` classes. */
const COLS_NARROW = 2;
const COLS_WIDE = 3;
/** Rows rendered past each edge of the visible window. */
const ROW_OVERSCAN = 3;
/** First-paint row height before a row is measured; a 16:9 map plus two text lines at the wide layout. */
const ROW_ESTIMATE_PX = 170;

export interface IStagePickerProps {
    open: boolean;
    onOpenChange: (open: boolean) => void;
    onPick: (stageId: string) => void;
}

export function StagePicker({ open, onOpenChange, onPick }: IStagePickerProps): React.ReactElement {
    const t: PickerT = useT("tools");
    const tNav = useT("nav");
    const server = useGamedataServer();
    const { data: index, isLoading } = useQuery({ ...stageIndexQueryOptions(server), enabled: open });
    const [group, setGroup] = React.useState<StageGroupKey>("story");
    const [zone, setZone] = React.useState<string>("all");
    const [query, setQuery] = React.useState("");

    // Challenge-mode variants are reached from the stage card's Normal / Challenge toggle.
    const stages = React.useMemo(() => (index ?? []).filter((s) => !s.stageId.endsWith("#f#")), [index]);
    // The backend sends a preview only when the file exists, and Normal / Hard /
    // Challenge variants of a map ship one file under the main stage id, so a
    // stage without its own borrows the first alias the index has a file for.
    const previews = React.useMemo(() => {
        const own = new Map<string, string>();
        for (const s of index ?? []) if (s.preview) own.set(s.stageId, s.preview);
        const out = new Map<string, string>();
        for (const s of index ?? []) {
            const hit = stagePreviewCandidates(s)
                .map((id) => own.get(id))
                .find(Boolean);
            if (hit) out.set(s.stageId, hit);
        }
        return out;
    }, [index]);
    const groups = React.useMemo(() => STAGE_GROUPS.filter((g) => stages.some((s) => s.group === g.key)), [stages]);

    const zones = React.useMemo(() => {
        const seen = new Map<string, { id: string; name: string; order: number }>();
        for (const s of stages) {
            if (s.group !== group || seen.has(s.zoneId)) continue;
            seen.set(s.zoneId, { id: s.zoneId, name: s.zoneName ?? s.zoneId, order: s.zoneOrder });
        }
        return [...seen.values()].sort((a, b) => a.order - b.order);
    }, [stages, group]);

    const shown = React.useMemo(() => {
        const pool = stages.filter((s) => s.group === group && (zone === "all" || s.zoneId === zone));
        if (!query.trim()) return pool;
        return searchAndRank(query, pool, (s) => ({ name: s.code, aliases: s.name ? [s.name] : [], extra: s.zoneName ?? "" })).map((r) => r.item);
    }, [stages, group, zone, query]);

    return (
        <Dialog open={open} onOpenChange={onOpenChange}>
            <DialogPopup className="max-w-3xl">
                <DialogHeader>
                    <DialogTitle>{t("squad.stagePicker.title")}</DialogTitle>
                    <DialogDescription>{t("squad.stagePicker.desc")}</DialogDescription>
                </DialogHeader>
                <DialogPanel>
                    <div className="flex flex-col gap-3">
                        <div className="flex flex-wrap gap-2">
                            <Select
                                value={group}
                                onValueChange={(v) => {
                                    if (!v) return;
                                    setGroup(v as StageGroupKey);
                                    setZone("all");
                                }}
                            >
                                <SelectTrigger size="sm" className="min-w-40 flex-1 sm:flex-none" aria-label={t("squad.stagePicker.group")}>
                                    <SelectValue>{(v: string | null) => (v && v in STAGE_GROUP_LABEL_KEY ? tNav(STAGE_GROUP_LABEL_KEY[v as StageGroupKey]) : v)}</SelectValue>
                                </SelectTrigger>
                                <SelectContent>
                                    {groups.map((g) => (
                                        <SelectItem key={g.key} value={g.key}>
                                            {tNav(STAGE_GROUP_LABEL_KEY[g.key])}
                                        </SelectItem>
                                    ))}
                                </SelectContent>
                            </Select>
                            <Select value={zone} onValueChange={(v) => v && setZone(v)}>
                                <SelectTrigger size="sm" className="min-w-48 flex-1" aria-label={t("squad.stagePicker.zone")}>
                                    <SelectValue>{(v: string | null) => (v === "all" || !v ? t("squad.stagePicker.allZones") : (zones.find((z) => z.id === v)?.name ?? v))}</SelectValue>
                                </SelectTrigger>
                                <SelectContent>
                                    <SelectItem value="all">{t("squad.stagePicker.allZones")}</SelectItem>
                                    {zones.map((z) => (
                                        <SelectItem key={z.id} value={z.id}>
                                            {z.name}
                                        </SelectItem>
                                    ))}
                                </SelectContent>
                            </Select>
                        </div>
                        <Input type="search" value={query} onChange={(e) => setQuery(e.target.value)} placeholder={t("squad.stagePicker.search")} aria-label={t("squad.stagePicker.search")} />

                        {isLoading ? (
                            <p className="py-8 text-center text-[13px] text-muted-foreground">{t("squad.stagePicker.loading")}</p>
                        ) : shown.length === 0 ? (
                            <p className="py-8 text-center text-[13px] text-muted-foreground">{t("squad.stagePicker.empty")}</p>
                        ) : (
                            <StageGrid stages={shown} previews={previews} onPick={onPick} />
                        )}
                    </div>
                </DialogPanel>
            </DialogPopup>
        </Dialog>
    );
}

function StageGrid({ stages, previews, onPick }: { stages: IStageIndexEntry[]; previews: Map<string, string>; onPick: (id: string) => void }): React.ReactElement {
    const cols = useMediaQuery("sm") ? COLS_WIDE : COLS_NARROW;
    const rows = React.useMemo(() => {
        const out: IStageIndexEntry[][] = [];
        for (let i = 0; i < stages.length; i += cols) out.push(stages.slice(i, i + cols));
        return out;
    }, [stages, cols]);

    // The grid scrolls with the filters inside the dialog's scroll area, so the
    // virtualizer watches that viewport and offsets by where the grid starts.
    const gridRef = React.useRef<HTMLDivElement | null>(null);
    const [scrollEl, setScrollEl] = React.useState<HTMLElement | null>(null);
    const [scrollMargin, setScrollMargin] = React.useState(0);
    React.useLayoutEffect(() => {
        const grid = gridRef.current;
        const viewport = grid?.closest<HTMLElement>("[data-slot=scroll-area-viewport]") ?? null;
        setScrollEl(viewport);
        if (!grid || !viewport) return;
        const update = () => setScrollMargin(grid.getBoundingClientRect().top - viewport.getBoundingClientRect().top + viewport.scrollTop);
        update();
        const observer = new ResizeObserver(update);
        observer.observe(viewport);
        return () => observer.disconnect();
    }, []);

    const virtualizer = useVirtualizer({
        count: rows.length,
        getScrollElement: () => scrollEl,
        estimateSize: () => ROW_ESTIMATE_PX,
        overscan: ROW_OVERSCAN,
        scrollMargin,
    });

    return (
        <div ref={gridRef} className="relative w-full" style={{ height: virtualizer.getTotalSize() }}>
            {virtualizer.getVirtualItems().map((vi) => (
                <ul key={vi.key} data-index={vi.index} ref={virtualizer.measureElement} className={cn("absolute top-0 left-0 grid w-full gap-2 pb-2", cols === COLS_WIDE ? "grid-cols-3" : "grid-cols-2")} style={{ transform: `translateY(${vi.start - scrollMargin}px)` }}>
                    {rows[vi.index]?.map((s) => (
                        <li key={s.stageId}>
                            <StageTile stage={s} preview={previews.get(s.stageId) ?? null} onPick={onPick} />
                        </li>
                    ))}
                </ul>
            ))}
        </div>
    );
}

const StageTile = React.memo(function StageTile({ stage, preview, onPick }: { stage: IStageIndexEntry; preview: string | null; onPick: (id: string) => void }): React.ReactElement {
    const tone = STAGE_GROUP_TONE[stage.group as StageGroupKey] ?? "var(--muted-foreground)";
    return (
        <button
            type="button"
            onClick={() => onPick(stage.stageId)}
            title={[stage.code, stage.name, stage.zoneName].filter(Boolean).join(" · ")}
            className="group flex w-full cursor-pointer flex-col overflow-hidden rounded-lg border border-border bg-card text-left outline-none transition-colors hover:border-primary/60 focus-visible:ring-2 focus-visible:ring-ring"
        >
            <span className="relative block aspect-[16/9] w-full overflow-hidden bg-muted">
                {/* Sits under the map, so it shows where a stage has no preview or the image fails. */}
                <span aria-hidden="true" className="absolute inset-0 flex items-center justify-center text-muted-foreground/40">
                    <MapIcon className="size-7" strokeWidth={1.5} />
                </span>
                {preview && <StagePreview src={preview} className="opacity-90 transition-opacity group-hover:opacity-100" />}
                <span className="absolute top-1.5 left-1.5 rounded-md border bg-background/85 px-1.5 py-0.5 font-bold font-mono text-[12px]" style={{ borderColor: `color-mix(in oklch, ${tone} 55%, var(--border))` }}>
                    {stage.code}
                </span>
            </span>
            <span className="block px-2 py-1.5">
                <span className="line-clamp-2 font-medium text-[12.5px] leading-tight">{stage.name ?? stage.code}</span>
                {stage.zoneName && <span className="block truncate text-[11px] text-muted-foreground">{stage.zoneName}</span>}
            </span>
        </button>
    );
});
