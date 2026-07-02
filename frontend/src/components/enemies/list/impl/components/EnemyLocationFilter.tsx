import { Check, ChevronDown, MapPin, Minus, Search, X } from "lucide-react";
import { useMemo, useState } from "react";
import { Collapsible, CollapsibleContent, CollapsibleTrigger } from "#/components/ui/collapsible";
import { Popover, PopoverPopup, PopoverTrigger } from "#/components/ui/popover";
import { STAGE_GROUPS, type StageGroupKey } from "#/lib/registry/stage-groups";
import { compactForSearch } from "#/lib/search/fuzzy";
import { cn } from "#/lib/utils";

// ── Tree types ──────────────────────────────────────────────────────────────

export interface IRawStage {
    stageId: string;
    code: string;
    stageName: string | null;
    isHard: boolean;
}
export interface IRawZone {
    zoneId: string;
    name: string;
    /** Fine group, resolved by the backend (`EnemyStageRef.group`). */
    group: StageGroupKey;
    stages: IRawStage[];
}

interface IStageNode {
    token: string;
    label: string;
}
interface IZoneNode {
    token: string;
    label: string;
    stages: IStageNode[];
}
interface IGroupNode {
    key: StageGroupKey;
    label: string;
    zones: IZoneNode[];
}

/** Annihilation maps store the region in `code` and the map in `name`; Paradox
 *  records read best by their record name. Everything else uses the stage code. */
function stageLabel(group: StageGroupKey, s: IRawStage): string {
    const base = group === "annihilation" || group === "paradox" ? s.stageName || s.code || s.stageId : s.code || s.stageName || s.stageId;
    // Normal and Adverse share a code (e.g. "10-10"); mark the hard one apart.
    return s.isHard ? `${base} (Adverse)` : base;
}
function zoneLabel(group: StageGroupKey, zone: IRawZone): string {
    // Annihilation zones are unnamed; the region lives in each map's stage code.
    if (group === "annihilation") return zone.stages[0]?.code || zone.name || zone.zoneId;
    return zone.name || zone.zoneId;
}

/** Turn the raw zone/stage data into an ordered, labelled group tree. The group
 *  comes from the backend; the frontend only handles presentation. */
export function buildLocationTree(zones: IRawZone[]): IGroupNode[] {
    const byGroup = new Map<StageGroupKey, IZoneNode[]>();
    for (const zone of zones) {
        const stages = zone.stages.map((s) => ({ token: s.stageId, label: stageLabel(zone.group, s) })).sort((a, b) => a.label.localeCompare(b.label, undefined, { numeric: true }));
        const node: IZoneNode = { token: zone.zoneId, label: zoneLabel(zone.group, zone), stages };
        const list = byGroup.get(zone.group);
        if (list) list.push(node);
        else byGroup.set(zone.group, [node]);
    }
    return STAGE_GROUPS.filter((g) => byGroup.has(g.key)).map((g) => ({
        key: g.key,
        label: g.label,
        zones: (byGroup.get(g.key) ?? []).sort((a, b) => a.label.localeCompare(b.label, undefined, { numeric: true })),
    }));
}

// ── Component ────────────────────────────────────────────────────────────────

interface IProps {
    tree: IGroupNode[];
    selected: string[];
    onChange: (values: string[]) => void;
}

export function EnemyLocationFilter({ tree, selected, onChange }: IProps) {
    const [query, setQuery] = useState("");
    const [openOverride, setOpenOverride] = useState<Partial<Record<StageGroupKey, boolean>>>({});
    const [zoneOpen, setZoneOpen] = useState<Record<string, boolean>>({});
    const sel = useMemo(() => new Set(selected), [selected]);
    const searching = query.trim().length > 0;

    const labelByToken = useMemo(() => {
        const m = new Map<string, string>();
        for (const g of tree) {
            for (const z of g.zones) {
                m.set(z.token, z.label);
                for (const s of z.stages) m.set(s.token, s.label);
            }
        }
        return m;
    }, [tree]);

    // Search-filter the tree.
    const visible = useMemo<IGroupNode[]>(() => {
        const q = compactForSearch(query);
        if (!q) return tree;
        const out: IGroupNode[] = [];
        for (const g of tree) {
            const groupHit = compactForSearch(g.label).includes(q);
            const zones: IZoneNode[] = [];
            for (const z of g.zones) {
                const zoneHit = groupHit || compactForSearch(z.label).includes(q);
                const stages = zoneHit ? z.stages : z.stages.filter((s) => compactForSearch(s.label).includes(q));
                if (zoneHit || stages.length > 0) zones.push({ ...z, stages });
            }
            if (zones.length > 0) out.push({ ...g, zones });
        }
        return out;
    }, [tree, query]);

    const zoneSelCount = (z: IZoneNode) => (sel.has(z.token) ? z.stages.length : z.stages.reduce((n, s) => n + (sel.has(s.token) ? 1 : 0), 0));
    const zoneFull = (z: IZoneNode) => sel.has(z.token) || (z.stages.length > 0 && z.stages.every((s) => sel.has(s.token)));

    const toggleStage = (z: IZoneNode, s: IStageNode) => {
        const next = new Set(sel);
        if (next.has(z.token)) {
            // Whole-zone selection: break it into stages, minus the one toggled off.
            next.delete(z.token);
            for (const st of z.stages) if (st.token !== s.token) next.add(st.token);
        } else if (next.has(s.token)) {
            next.delete(s.token);
        } else {
            next.add(s.token);
            // Collapse a fully-selected zone into one "Section" token (the zone).
            if (z.stages.length > 1 && z.stages.every((st) => next.has(st.token))) {
                for (const st of z.stages) next.delete(st.token);
                next.add(z.token);
            }
        }
        onChange([...next]);
    };
    const toggleZone = (z: IZoneNode) => {
        const next = new Set(sel);
        const full = zoneFull(z);
        next.delete(z.token);
        for (const s of z.stages) next.delete(s.token);
        if (!full) next.add(z.token);
        onChange([...next]);
    };
    const toggleGroup = (g: IGroupNode) => {
        const next = new Set(sel);
        const full = g.zones.every((z) => sel.has(z.token) || z.stages.every((s) => sel.has(s.token)));
        for (const z of g.zones) {
            next.delete(z.token);
            for (const s of z.stages) next.delete(s.token);
        }
        if (!full) for (const z of g.zones) next.add(z.token);
        onChange([...next]);
    };

    return (
        <div className="flex flex-col gap-2">
            <Popover>
                <PopoverTrigger
                    className="inline-flex h-9.5 w-fit cursor-pointer items-center gap-2 rounded-lg border border-border bg-[color-mix(in_oklch,var(--secondary)_50%,transparent)] px-3.5 font-medium font-sans text-[12.5px] text-foreground transition-colors hover:border-[color-mix(in_oklch,var(--primary)_55%,var(--border))] hover:bg-card data-popup-open:border-primary"
                    aria-label="Filter by where enemies appear"
                >
                    <MapPin className="h-3.5 w-3.5 text-muted-foreground" aria-hidden="true" />
                    <span>Appears In</span>
                    {selected.length > 0 && <span className="inline-flex h-4.5 min-w-4.5 items-center justify-center rounded-full bg-primary px-1 font-mono text-[10px] text-primary-foreground tabular-nums leading-none">{selected.length}</span>}
                    <ChevronDown className="h-3.5 w-3.5 text-muted-foreground" aria-hidden="true" />
                </PopoverTrigger>
                <PopoverPopup align="start" className="w-[min(24rem,calc(100vw-2rem))]">
                    <div className="flex max-h-[min(30rem,72vh)] w-full flex-col">
                        <div className="relative flex h-9 shrink-0 items-center gap-2 rounded-md border border-border bg-[color-mix(in_oklch,var(--secondary)_50%,transparent)] px-3 [&>svg]:shrink-0 [&>svg]:text-muted-foreground">
                            <Search className="h-3.5 w-3.5" aria-hidden="true" />
                            <input
                                type="text"
                                value={query}
                                onChange={(e) => setQuery(e.target.value)}
                                placeholder="Search events, modes, or stages…"
                                aria-label="Search locations"
                                className="min-w-0 flex-1 appearance-none border-0 bg-transparent p-0 font-sans text-[13px] text-foreground leading-none outline-none placeholder:text-muted-foreground"
                            />
                            {query && (
                                <button type="button" onClick={() => setQuery("")} aria-label="Clear search" className="-mr-1 inline-flex size-5 cursor-pointer items-center justify-center rounded text-muted-foreground hover:text-foreground">
                                    <X className="h-3.5 w-3.5" aria-hidden="true" />
                                </button>
                            )}
                        </div>

                        <div className="-mx-1 mt-2 min-h-0 flex-1 overflow-y-auto px-1">
                            {visible.length === 0 ? (
                                <p className="py-6 text-center font-sans text-[13px] text-muted-foreground">No matching locations.</p>
                            ) : (
                                <div className="flex flex-col gap-0.5">
                                    {visible.map((g) => {
                                        const gFull = g.zones.every(zoneFull);
                                        const gSel = g.zones.some((z) => zoneSelCount(z) > 0);
                                        const open = searching || (openOverride[g.key] ?? gSel);
                                        return (
                                            <Collapsible key={g.key} open={open} onOpenChange={(o) => setOpenOverride((p) => ({ ...p, [g.key]: o }))}>
                                                <div className="flex items-center gap-1 rounded-md hover:bg-[color-mix(in_oklch,var(--muted)_30%,transparent)]">
                                                    <CheckButton checked={gSel} indeterminate={gSel && !gFull} onClick={() => toggleGroup(g)} label={`${gFull ? "Deselect" : "Select"} all ${g.label}`} />
                                                    <CollapsibleTrigger className="flex flex-1 cursor-pointer items-center gap-1.5 py-1.5 pr-2 text-left">
                                                        <ChevronDown className={cn("h-3.5 w-3.5 shrink-0 text-muted-foreground transition-transform duration-150 motion-reduce:transition-none", open && "rotate-180")} aria-hidden="true" />
                                                        <span className="flex-1 truncate font-sans font-semibold text-[12.5px] text-foreground">{g.label}</span>
                                                        <span className="shrink-0 font-mono text-[10px] text-muted-foreground tabular-nums">{g.zones.length}</span>
                                                    </CollapsibleTrigger>
                                                </div>
                                                <CollapsibleContent>
                                                    <div className="flex flex-col gap-0.5 py-1 pl-6">
                                                        {g.zones.map((z) => {
                                                            const zOpen = searching || (zoneOpen[z.token] ?? zoneSelCount(z) > 0);
                                                            return (
                                                                <Collapsible key={z.token} open={zOpen} onOpenChange={(o) => setZoneOpen((p) => ({ ...p, [z.token]: o }))}>
                                                                    <div className="flex items-center gap-1 rounded-md hover:bg-[color-mix(in_oklch,var(--muted)_30%,transparent)]">
                                                                        <CheckButton checked={zoneSelCount(z) > 0} indeterminate={zoneSelCount(z) > 0 && !zoneFull(z)} onClick={() => toggleZone(z)} label={`${zoneFull(z) ? "Deselect" : "Select"} all ${z.label}`} />
                                                                        <CollapsibleTrigger className="flex flex-1 cursor-pointer items-center gap-1.5 py-1 pr-2 text-left">
                                                                            <ChevronDown className={cn("h-3 w-3 shrink-0 text-muted-foreground transition-transform duration-150 motion-reduce:transition-none", zOpen && "rotate-180")} aria-hidden="true" />
                                                                            <span className="min-w-0 flex-1 truncate font-medium font-sans text-[12px] text-foreground">{z.label}</span>
                                                                            <span className="shrink-0 font-mono text-[9.5px] text-muted-foreground tabular-nums">{z.stages.length}</span>
                                                                        </CollapsibleTrigger>
                                                                    </div>
                                                                    <CollapsibleContent>
                                                                        <div className="flex flex-col pl-7">
                                                                            {z.stages.map((s) => {
                                                                                const on = sel.has(s.token) || sel.has(z.token);
                                                                                return (
                                                                                    <button key={s.token} type="button" aria-pressed={on} onClick={() => toggleStage(z, s)} className="flex min-h-7 cursor-pointer items-center gap-2 rounded-md px-1.5 text-left hover:bg-[color-mix(in_oklch,var(--muted)_30%,transparent)]">
                                                                                        <CheckGlyph checked={on} />
                                                                                        <span className="min-w-0 flex-1 truncate font-sans text-[12px] text-muted-foreground">{s.label}</span>
                                                                                    </button>
                                                                                );
                                                                            })}
                                                                        </div>
                                                                    </CollapsibleContent>
                                                                </Collapsible>
                                                            );
                                                        })}
                                                    </div>
                                                </CollapsibleContent>
                                            </Collapsible>
                                        );
                                    })}
                                </div>
                            )}
                        </div>

                        {selected.length > 0 && (
                            <div className="mt-2 flex shrink-0 items-center justify-between border-border/60 border-t pt-2">
                                <span className="font-sans text-[12px] text-muted-foreground">{selected.length} selected</span>
                                <button type="button" onClick={() => onChange([])} className="cursor-pointer font-medium font-sans text-[12px] text-primary hover:underline">
                                    Clear all
                                </button>
                            </div>
                        )}
                    </div>
                </PopoverPopup>
            </Popover>

            {selected.length > 0 && (
                <div className="flex flex-wrap gap-1.5">
                    {selected.map((v) => (
                        <span key={v} className="inline-flex items-center gap-1 rounded-md border border-border bg-card py-0.75 ps-2 pe-0.5 font-medium font-sans text-[11.5px] text-foreground leading-none">
                            <span className="max-w-44 truncate">{labelByToken.get(v) ?? v}</span>
                            <button type="button" onClick={() => onChange(selected.filter((t) => t !== v))} aria-label={`Remove ${labelByToken.get(v) ?? v}`} className="inline-flex size-4.5 cursor-pointer items-center justify-center rounded text-muted-foreground hover:text-foreground">
                                <X className="h-3 w-3" aria-hidden="true" />
                            </button>
                        </span>
                    ))}
                </div>
            )}
        </div>
    );
}

function CheckButton({ checked, indeterminate, onClick, label }: { checked: boolean; indeterminate?: boolean; onClick: () => void; label: string }) {
    return (
        <button type="button" aria-pressed={indeterminate ? "mixed" : checked} aria-label={label} onClick={onClick} className="inline-flex size-7 shrink-0 cursor-pointer items-center justify-center">
            <CheckGlyph checked={checked} indeterminate={indeterminate} />
        </button>
    );
}

function CheckGlyph({ checked, indeterminate }: { checked: boolean; indeterminate?: boolean }) {
    const on = checked || indeterminate;
    return (
        <span aria-hidden="true" className={cn("flex size-4 shrink-0 items-center justify-center rounded-sm border transition-colors", on ? "border-primary bg-primary text-primary-foreground" : "border-input bg-background")}>
            {indeterminate ? <Minus className="size-3" strokeWidth={3} /> : checked ? <Check className="size-3" strokeWidth={3} /> : null}
        </span>
    );
}
