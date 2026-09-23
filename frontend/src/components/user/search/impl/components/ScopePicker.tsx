import { Check } from "lucide-react";
import { useMemo } from "react";
import { ClassIcon } from "#/components/operators/list/impl/components/Icons";
import { CLASSES } from "#/components/operators/list/impl/constants";
import { Tooltip, TooltipPopup, TooltipTrigger } from "#/components/ui/tooltip";
import { useT } from "#/lib/i18n";
import type { TypedT } from "#/lib/i18n/messages";
import { cn, formatArchetype, formatProfession, subProfessionToProfession } from "#/lib/utils";
import type { IOperatorIndexEntry } from "#/types/operators";
import type { Scope } from "../searchControls";
import type { messages } from "./ScopePicker.messages";

interface IScopePickerProps {
    scope: Scope | null;
    onChange: (next: Scope | null) => void;
    /** The operator index; undefined while it loads. */
    operators: IOperatorIndexEntry[] | undefined;
}

/** Archetype ids of obtainable operators per class, each list in label order. */
export function useArchetypesByClass(operators: IOperatorIndexEntry[] | undefined): Map<string, string[]> {
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
                [...ids].sort((a, b) => formatArchetype(a).localeCompare(formatArchetype(b))),
            );
        }
        return out;
    }, [operators]);
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

const CHIP = "inline-flex h-8 cursor-pointer items-center gap-1.5 rounded-lg border px-2.5 font-medium font-sans text-xs leading-none transition-colors";
const CHIP_ON = "border-primary/40 bg-[color-mix(in_srgb,var(--primary)_10%,transparent)] text-primary";
const CHIP_OFF = "border-input bg-card text-foreground hover:border-foreground/20";

/**
 * Eight class icons, then the archetypes of the chosen class as chips. A
 * class press selects the whole class (and, pressed again, clears it); an
 * archetype narrows to that archetype; "Whole class" widens back.
 */
export function ScopePicker({ scope, onChange, operators }: IScopePickerProps) {
    const t: TypedT<typeof messages> = useT("user");
    const archetypes = useArchetypesByClass(operators);
    const profession = scopeProfession(scope, operators);
    const subs = profession ? (archetypes.get(profession) ?? []) : [];

    return (
        <div className="flex flex-col gap-2">
            <fieldset className="m-0 min-w-0 border-0 p-0">
                <Legend>{t("search.scope.class")}</Legend>
                <div className="grid grid-cols-8 gap-1">
                    {CLASSES.map((cls) => {
                        const on = profession === cls;
                        return (
                            <Tooltip key={cls}>
                                <TooltipTrigger
                                    render={
                                        <button
                                            type="button"
                                            aria-pressed={on}
                                            aria-label={formatProfession(cls)}
                                            onClick={() => onChange(on ? null : { kind: "class", profession: cls })}
                                            className={cn("inline-flex aspect-square cursor-pointer items-center justify-center rounded-md border transition-colors", on ? "border-primary bg-[color-mix(in_srgb,var(--primary)_16%,transparent)]" : "border-border bg-secondary/50 hover:border-primary/45")}
                                        >
                                            <ClassIcon profession={cls} size={20} />
                                        </button>
                                    }
                                />
                                <TooltipPopup side="top" sideOffset={6}>
                                    {formatProfession(cls)}
                                </TooltipPopup>
                            </Tooltip>
                        );
                    })}
                </div>
            </fieldset>

            {profession ? (
                <fieldset className="m-0 min-w-0 border-0 p-0">
                    <Legend>{t("search.scope.archetype")}</Legend>
                    {operators === undefined ? (
                        <p className="m-0 font-sans text-[11.5px] text-muted-foreground leading-none">{t("search.scope.loading")}</p>
                    ) : (
                        <ul className="m-0 flex list-none flex-wrap gap-1.5 p-0">
                            <li>
                                <button type="button" aria-pressed={scope?.kind === "class"} onClick={() => onChange({ kind: "class", profession })} className={cn(CHIP, scope?.kind === "class" ? CHIP_ON : CHIP_OFF)}>
                                    {scope?.kind === "class" ? <Check className="size-3" aria-hidden /> : null}
                                    {t("search.scope.anyArchetype")}
                                </button>
                            </li>
                            {subs.map((id) => {
                                const on = scope?.kind === "sub" && scope.subProfessionId === id;
                                return (
                                    <li key={id}>
                                        <button type="button" aria-pressed={on} onClick={() => onChange({ kind: "sub", subProfessionId: id })} className={cn(CHIP, on ? CHIP_ON : CHIP_OFF)}>
                                            {on ? <Check className="size-3" aria-hidden /> : null}
                                            {formatArchetype(id)}
                                        </button>
                                    </li>
                                );
                            })}
                        </ul>
                    )}
                </fieldset>
            ) : null}
        </div>
    );
}

/** The fieldset's own name, styled as the popup's other group headings. */
function Legend({ children }: { children: React.ReactNode }) {
    return <legend className="mb-2 p-0 font-medium font-mono text-[10px] text-muted-foreground uppercase leading-none tracking-[0.16em]">{children}</legend>;
}
