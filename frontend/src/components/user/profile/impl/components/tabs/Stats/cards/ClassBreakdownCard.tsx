import { BarChart3, ChevronDown } from "lucide-react";
import { useCallback, useId, useMemo, useState } from "react";
import { ClassIcon, SubProfessionIcon } from "#/components/operators/list/impl/components/Icons";
import { cn } from "#/lib/utils";
import { PALETTE } from "../palette";
import { Bar, CARD_PADDING, Kicker, StatCard } from "../primitives";

interface ISubProfession {
    subProfessionId: string;
    displayName: string;
    owned: number;
    total: number;
    percentage: number;
}

interface IProfession {
    profession: string;
    displayName: string;
    owned: number;
    total: number;
    percentage: number;
    subProfessions: ISubProfession[];
}

/** Drop the trailing word so e.g. "Pioneer Vanguard" displays as "Pioneer". */
function shortSubName(name: string): string {
    return name.replace(/\s+\S+$/, "");
}

export function ClassBreakdownCard({ professions }: { professions: IProfession[] }) {
    const colors = useMemo(() => professions.map((p) => PALETTE.classes[p.profession] ?? PALETTE.collection), [professions]);

    // Pre-compute the longest short name across all sub-professions so the
    // expanded sub-rows align in a grid.
    const maxSubNameLen = useMemo(() => {
        let max = 0;
        for (const prof of professions) {
            for (const sub of prof.subProfessions) {
                const n = shortSubName(sub.displayName).length;
                if (n > max) max = n;
            }
        }
        return max;
    }, [professions]);

    const [expanded, setExpanded] = useState<ReadonlySet<string>>(() => new Set());
    const toggle = useCallback((id: string) => {
        setExpanded((prev) => {
            const next = new Set(prev);
            if (next.has(id)) next.delete(id);
            else next.add(id);
            return next;
        });
    }, []);

    return (
        <StatCard className="sm:col-span-2" color={colors}>
            <div className={cn(CARD_PADDING, "pb-2")}>
                <Kicker icon={BarChart3} label="Class Breakdown" />
            </div>
            <div className="flex flex-col px-4 pb-4 sm:px-5 sm:pb-5">
                {professions.map((prof) => (
                    <ProfessionRow key={prof.profession} expanded={expanded.has(prof.profession)} maxSubNameLen={maxSubNameLen} onToggle={toggle} prof={prof} />
                ))}
            </div>
        </StatCard>
    );
}

interface IProfessionRowProps {
    prof: IProfession;
    expanded: boolean;
    maxSubNameLen: number;
    onToggle: (profession: string) => void;
}

function ProfessionRow({ prof, expanded, maxSubNameLen, onToggle }: IProfessionRowProps) {
    const panelId = useId();
    const hasSubs = prof.subProfessions.length > 0;
    const classColor = PALETTE.classes[prof.profession] ?? PALETTE.collection;

    return (
        <div>
            <button aria-controls={hasSubs ? panelId : undefined} aria-expanded={hasSubs ? expanded : undefined} className="group -mx-3 w-[calc(100%+1.5rem)] cursor-pointer rounded-lg px-3 py-2.5 text-left transition-colors hover:bg-muted/40" onClick={() => onToggle(prof.profession)} type="button">
                <div className="flex items-center gap-3">
                    <div className="flex h-9 w-9 shrink-0 items-center justify-center rounded-xl" style={{ background: `color-mix(in oklch, ${classColor} 14%, transparent)` }}>
                        <ClassIcon className="opacity-80" profession={prof.profession} size={22} />
                    </div>
                    <div className="flex w-full min-w-0 flex-col gap-1.5">
                        <div className="flex items-center justify-between">
                            <div className="flex items-center gap-1.5">
                                <span className="font-semibold text-sm">{prof.displayName}</span>
                                {hasSubs && <ChevronDown className={cn("h-3.5 w-3.5 text-muted-foreground/40 transition-transform duration-200", expanded && "rotate-180")} />}
                            </div>
                            <div className="flex items-baseline gap-1">
                                <span className="font-bold text-sm tabular-nums">{prof.owned}</span>
                                <span className="font-mono text-[10.5px] text-muted-foreground/50">/ {prof.total}</span>
                            </div>
                        </div>
                        <Bar color={classColor} pct={prof.percentage} />
                    </div>
                </div>
            </button>

            {hasSubs && (
                <div className="grid transition-[grid-template-rows] duration-300 ease-in-out" id={panelId} style={{ gridTemplateRows: expanded ? "1fr" : "0fr" }}>
                    <div className="min-h-0 overflow-hidden">
                        <div
                            className="ml-4 grid py-1"
                            style={{
                                gridTemplateColumns: "auto auto 1fr auto",
                                ["--sub-name-w" as string]: `${maxSubNameLen}ch`,
                            }}
                        >
                            {prof.subProfessions.map((sub, idx) => (
                                <SubProfessionRow color={classColor} isLast={idx === prof.subProfessions.length - 1} key={sub.subProfessionId} sub={sub} />
                            ))}
                        </div>
                    </div>
                </div>
            )}
        </div>
    );
}

interface ISubProfessionRowProps {
    sub: ISubProfession;
    color: string;
    isLast: boolean;
}

function SubProfessionRow({ sub, color, isLast }: ISubProfessionRowProps) {
    return (
        <div className="group/row relative col-span-4 grid grid-cols-subgrid items-center gap-x-2 rounded-md py-1.5 pr-2 transition-colors hover:bg-muted/20">
            <div className={cn("absolute left-0 w-px bg-muted-foreground/20", isLast ? "top-0 h-1/2" : "top-0 h-full")} />
            <div className="absolute top-1/2 left-0 h-px w-6 bg-muted-foreground/20" />
            <div className="ml-6">
                <SubProfessionIcon className="shrink-0 opacity-60" size={15} subProfession={sub.subProfessionId} />
            </div>
            <span className={cn("max-w-20 truncate font-mono text-[10.5px] text-muted-foreground/70 sm:max-w-none")} style={{ minWidth: "var(--sub-name-w)" }}>
                {shortSubName(sub.displayName)}
            </span>
            <Bar color={color} dim pct={sub.percentage} thin />
            <span className="w-10 shrink-0 text-right font-mono text-[10.5px] text-muted-foreground/70 tabular-nums">
                {sub.owned}/{sub.total}
            </span>
        </div>
    );
}
