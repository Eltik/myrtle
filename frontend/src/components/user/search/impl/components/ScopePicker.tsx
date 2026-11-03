import { useMemo } from "react";
import { ClassIcon } from "#/components/operators/list/impl/components/Icons";
import { CLASSES } from "#/components/operators/list/impl/constants";
import { ToggleGroup, ToggleGroupItem } from "#/components/ui/toggle-group";
import { Tooltip, TooltipPopup, TooltipTrigger } from "#/components/ui/tooltip";
import { useT } from "#/lib/i18n";
import type { TypedT } from "#/lib/i18n/messages";
import { formatArchetype, professionLabel, serverNames, subProfessionToProfession } from "#/lib/utils";
import type { IOperatorIndexEntry } from "#/types/operators";
import type { Scope } from "../searchControls";
import type { messages } from "./ScopePicker.messages";

interface IScopePickerProps {
    scope: Scope | null;
    onChange: (next: Scope | null) => void;
    /** The operator index; undefined while it loads. */
    operators: IOperatorIndexEntry[] | undefined;
}

/** Archetype id -> the server's own name, read off the operator index; ids without one fall back to the English table. */
function useArchetypeLabeler(operators: IOperatorIndexEntry[] | undefined): (id: string) => string {
    return useMemo(() => {
        const names = serverNames(operators ?? [], "subProfession");
        return (id: string) => names.get(id) ?? formatArchetype(id);
    }, [operators]);
}

/** Archetype ids of obtainable operators per class, each list in label order. */
export function useArchetypesByClass(operators: IOperatorIndexEntry[] | undefined): Map<string, string[]> {
    const label = useArchetypeLabeler(operators);
    return useMemo(() => {
        const byClass = new Map<string, Set<string>>();
        for (const op of operators ?? []) {
            if (op.isNotObtainable) continue;
            const set = byClass.get(op.profession) ?? new Set<string>();
            set.add(op.subProfessionId);
            byClass.set(op.profession, set);
        }
        const out = new Map<string, string[]>();
        for (const [profession, ids] of byClass) {
            out.set(
                profession,
                [...ids].sort((a, b) => label(a).localeCompare(label(b))),
            );
        }
        return out;
    }, [operators, label]);
}

/** The class an archetype belongs to, read from the index and from the static map before it loads. */
export function scopeProfession(scope: Scope | null, operators: IOperatorIndexEntry[] | undefined): string | null {
    if (!scope) return null;
    if (scope.kind === "class") return scope.profession;
    const fromIndex = operators?.find((op) => op.subProfessionId === scope.subProfessionId)?.profession;
    if (fromIndex) return fromIndex;
    const mapped = subProfessionToProfession(scope.subProfessionId);
    return mapped === "OTHER" ? null : mapped;
}

/**
 * Eight class icons, then the archetypes of the chosen class as chips. A
 * class press selects the whole class (and, pressed again, clears it); an
 * archetype narrows to that archetype; "Whole class" widens back.
 */
export function ScopePicker({ scope, onChange, operators }: IScopePickerProps) {
    const t: TypedT<typeof messages> = useT("user");
    const archetypes = useArchetypesByClass(operators);
    const archetypeName = useArchetypeLabeler(operators);
    const classNames = useMemo(() => serverNames(operators ?? [], "profession"), [operators]);
    const profession = scopeProfession(scope, operators);
    const subs = profession ? (archetypes.get(profession) ?? []) : [];

    return (
        <div className="flex flex-col gap-2">
            <fieldset className="m-0 min-w-0 border-0 p-0">
                <Legend>{t("search.scope.class")}</Legend>
                <ToggleGroup aria-label={t("search.scope.class")} className="grid w-full grid-cols-8 gap-1" value={profession ? [profession] : []} onValueChange={(next) => onChange(next[0] ? { kind: "class", profession: next[0] as (typeof CLASSES)[number] } : null)}>
                    {CLASSES.map((cls) => {
                        const name = professionLabel({ profession: cls, professionName: classNames.get(cls) });
                        return (
                            <Tooltip key={cls}>
                                <TooltipTrigger render={<ToggleGroupItem value={cls} aria-label={name} className="aspect-square h-auto min-w-0 border-border bg-secondary/50 p-0 sm:h-auto sm:min-w-0" />}>
                                    <ClassIcon profession={cls} size={20} />
                                </TooltipTrigger>
                                <TooltipPopup side="top" sideOffset={6}>
                                    {name}
                                </TooltipPopup>
                            </Tooltip>
                        );
                    })}
                </ToggleGroup>
            </fieldset>

            {profession ? (
                <fieldset className="m-0 min-w-0 border-0 p-0">
                    <Legend>{t("search.scope.archetype")}</Legend>
                    {operators === undefined ? (
                        <p className="m-0 font-sans text-[11.5px] text-muted-foreground leading-none">{t("search.scope.loading")}</p>
                    ) : (
                        <ToggleGroup
                            aria-label={t("search.scope.archetype")}
                            className="flex-wrap gap-1.5"
                            onValueChange={(next) => {
                                const id = next[0];
                                if (id === undefined) return;
                                onChange(id === ANY_ARCHETYPE ? { kind: "class", profession } : { kind: "sub", subProfessionId: id });
                            }}
                            size="sm"
                            value={scope?.kind === "class" ? [ANY_ARCHETYPE] : scope?.kind === "sub" ? [scope.subProfessionId] : []}
                            variant="outline"
                        >
                            <ToggleGroupItem className="px-3" value={ANY_ARCHETYPE}>
                                {t("search.scope.anyArchetype")}
                            </ToggleGroupItem>
                            {subs.map((id) => (
                                <ToggleGroupItem className="px-3" key={id} value={id}>
                                    {archetypeName(id)}
                                </ToggleGroupItem>
                            ))}
                        </ToggleGroup>
                    )}
                </fieldset>
            ) : null}
        </div>
    );
}

/** The archetype group's "any" item; sub-profession ids never collide with it. */
const ANY_ARCHETYPE = "__any";

/** The fieldset's own name, styled as the popup's other group headings. */
function Legend({ children }: { children: React.ReactNode }) {
    return <legend className="kicker mb-2 p-0 leading-none">{children}</legend>;
}
