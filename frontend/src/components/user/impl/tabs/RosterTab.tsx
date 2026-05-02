import { useMemo, useState } from "react";
import type { IRosterEntry } from "#/lib/api/user";
import { formatProfession } from "#/lib/utils";
import type { IOperatorIndexEntry, OperatorRarityTier } from "#/types/operators";

interface IEnrichedEntry extends IRosterEntry {
    name: string;
    rarity: number;
    cls: string;
}

interface IRosterTabProps {
    roster: IRosterEntry[];
    operatorsIndex: IOperatorIndexEntry[];
}

type SortKey = "level" | "rarity" | "obtained" | "potential";
type SortOrder = "asc" | "desc";
type RarityFilter = "all" | OperatorRarityTier;
type ViewMode = "detailed" | "compact";
type OwnershipFilter = "all" | "owned" | "unowned";

export function RosterTab({ roster, operatorsIndex }: IRosterTabProps) {
    const [_activeClass, _setActiveClass] = useState("All");
    const [_sort, _setSort] = useState<SortKey>("rarity");

    const indexMap = useMemo(() => {
        const m = new Map<string, IOperatorIndexEntry>();
        for (const op of operatorsIndex) m.set(op.id, op);
        return m;
    }, [operatorsIndex]);

    const _enriched = useMemo<IEnrichedEntry[]>(() => {
        return roster.map((entry) => {
            const meta = indexMap.get(entry.operator_id);
            return {
                ...entry,
                name: meta?.name ?? entry.operator_id,
                rarity: meta?.rarity ?? 3,
                cls: meta?.profession ? (formatProfession(meta.profession) ?? "Guard") : "Guard",
            };
        });
    }, [roster, indexMap]);

    return <section className="flex flex-col gap-4" aria-label="Operator roster"></section>;
}
