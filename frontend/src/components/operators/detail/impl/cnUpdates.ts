import { useQuery } from "@tanstack/react-query";
import { createContext, useContext, useMemo } from "react";
import { useCnUpdates } from "#/hooks/use-cn-updates";
import { cnChangesQueryOptions, operatorQueryOptions } from "#/lib/api/operators";
import { type ChangeKind, withCnAdditions } from "#/lib/operators/cn-changes";
import type { OperatorChange } from "#/types/generated/OperatorChange";
import type { IOperatorListItem } from "#/types/operators";
import { cnModuleTexts, type ICnModuleTexts } from "./cnText";

/** What CN's data has for the operator on the page, for the tabs to mark inline. */
export interface ICnUpdates {
    changes: readonly OperatorChange[];
    kinds: ReadonlySet<ChangeKind>;
    /** CN's own record of the operator. */
    cn: IOperatorListItem;
    /** The modules and skills merged into the page's operator from `cn`. */
    moduleIds: ReadonlySet<string>;
    skillIds: ReadonlySet<string>;
    /** The merged modules' text in this server's language, where this server's own text allows. */
    texts: ICnModuleTexts;
}

/** Null whenever the page shows only this server's data: setting off, nothing changed, or CN's record not loaded. */
export const CnUpdatesContext = createContext<ICnUpdates | null>(null);

export function useCnUpdatesContext(): ICnUpdates | null {
    return useContext(CnUpdatesContext);
}

/**
 * The operator as the page shows it, with CN's added modules and skills merged in, and what CN
 * changed. With the CN updates setting off, or on CN itself, neither the change list nor CN's
 * record is requested and `operator` comes back unchanged; CN's record is requested only for an
 * operator the change list names.
 */
export function useCnUpdatesFor(operator: IOperatorListItem | null | undefined, server: string): { operator: IOperatorListItem | null | undefined; updates: ICnUpdates | null } {
    const id = operator?.id ?? "";
    const enabled = useCnUpdates(server) && operator?.server !== "cn" && id.length > 0;
    const { data: all } = useQuery({ ...cnChangesQueryOptions(server), enabled });
    const changes = enabled ? all?.find((row) => row.id === id)?.changes : undefined;
    const { data: cn } = useQuery({ ...operatorQueryOptions(id, "cn"), enabled: enabled && (changes?.length ?? 0) > 0 });
    return useMemo(() => {
        if (!operator || !changes || changes.length === 0 || !cn) return { operator, updates: null };
        const merged = withCnAdditions(operator, cn, changes);
        const translations = changes.flatMap((c) => (c.kind === "module_added" ? c.translations : []));
        return { operator: merged.operator, updates: { changes, kinds: new Set(changes.map((c) => c.kind)), cn, moduleIds: merged.moduleIds, skillIds: merged.skillIds, texts: cnModuleTexts(operator, cn, translations) } };
    }, [operator, changes, cn]);
}
