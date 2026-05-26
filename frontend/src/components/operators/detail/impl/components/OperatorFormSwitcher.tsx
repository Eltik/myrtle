import { useQuery } from "@tanstack/react-query";
import { Link } from "@tanstack/react-router";
import { operatorsIndexQueryOptions } from "#/lib/api/operators";
import { cn, formatProfession } from "#/lib/utils";
import type { IOperatorIndexEntry, IOperatorListItem } from "#/types/operators";
import { ClassIcon } from "../../../list/impl/components/Icons";

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
        <div className="-mt-2 mb-4 sm:-mt-3 sm:mb-6">
            <nav aria-label="Operator form" className="inline-flex max-w-full items-center gap-0.5 rounded-full border border-border/60 bg-card/30 p-1">
                {forms.map((form) => {
                    const isActive = form.id === operator.id;
                    return (
                        <Link
                            key={form.id}
                            to="/operators/$id"
                            params={{ id: form.id }}
                            aria-current={isActive ? "page" : undefined}
                            className={cn(
                                "group flex min-h-11 min-w-0 items-center justify-center gap-1.5 rounded-full px-3.5 font-medium text-sm transition-colors duration-200 sm:min-h-0 sm:py-1.5",
                                "motion-reduce:transition-none",
                                "focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-primary/50",
                                isActive ? "bg-card text-foreground shadow-sm ring-1 ring-border/50" : "text-muted-foreground hover:bg-card/50 hover:text-foreground",
                            )}
                        >
                            <ClassIcon profession={form.profession} size={16} className={cn("shrink-0 transition-opacity", isActive ? "opacity-100" : "opacity-60 group-hover:opacity-90")} />
                            <span className="truncate">{formatProfession(form.profession)}</span>
                        </Link>
                    );
                })}
            </nav>
        </div>
    );
}
