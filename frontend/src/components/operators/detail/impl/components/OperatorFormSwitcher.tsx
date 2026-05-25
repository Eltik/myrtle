import { useQuery } from "@tanstack/react-query";
import { Link } from "@tanstack/react-router";
import { operatorsIndexQueryOptions } from "#/lib/api/operators";
import { cn, formatProfession } from "#/lib/utils";
import type { IOperatorIndexEntry, IOperatorListItem } from "#/types/operators";
import { ClassIcon } from "../../../list/impl/components/Icons";
import { RARITY_COLORS } from "../constants";

interface IOperatorFormSwitcherProps {
    operator: IOperatorListItem;
}

export function OperatorFormSwitcher({ operator }: IOperatorFormSwitcherProps) {
    const hasForms = (operator.tmplIds?.length ?? 0) >= 2;
    const { data: index } = useQuery({ ...operatorsIndexQueryOptions(), enabled: hasForms });
    if (!hasForms || !operator.tmplIds || !index) return null;

    const indexById = new Map<string, IOperatorIndexEntry>(index.map((e) => [e.id, e]));
    const forms = operator.tmplIds.map((id) => indexById.get(id)).filter((e): e is IOperatorIndexEntry => Boolean(e));
    if (forms.length < 2) return null;

    return (
        <div className="-mt-2 mb-4 sm:-mt-4 sm:mb-6">
            <div
                role="tablist"
                aria-label="Operator form"
                className={cn(
                    // Mobile: full-width container so the pills stretch edge-to-edge
                    "flex w-full items-stretch gap-1 rounded-xl border border-border/60 bg-card/60 p-1 backdrop-blur-md",
                    // Desktop: shrink to content width and align left
                    "sm:inline-flex sm:w-auto sm:items-center",
                )}
            >
                {forms.map((form) => {
                    const isActive = form.id === operator.id;
                    const rarityColor = RARITY_COLORS[`TIER_${form.rarity}` as keyof typeof RARITY_COLORS] ?? "";
                    return (
                        <Link
                            key={form.id}
                            to="/operators/$id"
                            params={{ id: form.id }}
                            role="tab"
                            aria-selected={isActive}
                            aria-current={isActive ? "page" : undefined}
                            className={cn(
                                // Mobile: equal-width cells, ≥44px touch target, press feedback
                                "group flex min-w-0 flex-1 items-center justify-center gap-2 rounded-lg px-3 py-3 font-medium text-sm transition-colors",
                                "active:bg-accent/80",
                                // Desktop: shrink to content, tighter padding
                                "sm:flex-initial sm:justify-start sm:py-1.5",
                                "focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-primary/50",
                                isActive ? "bg-primary/10 text-foreground ring-1 ring-primary/30" : "text-muted-foreground hover:bg-accent hover:text-foreground",
                            )}
                        >
                            <ClassIcon profession={form.profession} size={18} className={cn("shrink-0", isActive ? "opacity-100" : "opacity-70 group-hover:opacity-100")} />
                            <span className="truncate">{formatProfession(form.profession)}</span>
                            <span aria-hidden className={cn("shrink-0 text-xs tabular-nums", isActive ? rarityColor : "text-muted-foreground/70")}>
                                {form.rarity}★
                            </span>
                        </Link>
                    );
                })}
            </div>
        </div>
    );
}
