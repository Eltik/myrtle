import { Check } from "lucide-react";
import { Toggle } from "#/components/ui/toggle";
import { Tooltip, TooltipPopup, TooltipTrigger } from "#/components/ui/tooltip";
import { useT } from "#/lib/i18n";
import type { TypedT } from "#/lib/i18n/messages";
import { professionLabel } from "#/lib/utils";
import { CLASSES } from "../constants";
import type { messages } from "./ClassPicker.messages";
import { ClassIcon } from "./Icons";
import styles from "./OperatorFilters.module.css";

export function toggle<T>(list: T[], value: T): T[] {
    return list.includes(value) ? list.filter((v) => v !== value) : [...list, value];
}

/**
 * `labels` maps a class code to the server's own name; absent, the English formatter names it.
 * A pressed class shows a check in the corner as well as the Toggle's fill and inset ring, so the
 * state does not rest on colour alone, the same cue FilterChip carries.
 */
export function ClassPicker({ selected, onChange, labels }: { selected: string[]; onChange: (v: string[]) => void; labels?: Record<string, string> }) {
    const t: TypedT<typeof messages> = useT("operators");

    return (
        <div className={styles.field}>
            <div className={styles.fieldLabel}>{t("filters.class")}</div>
            <div className="grid grid-cols-4 gap-1.5">
                {CLASSES.map((cls) => {
                    const on = selected.includes(cls);
                    const name = professionLabel({ profession: cls, professionName: labels?.[cls] });
                    return (
                        <Tooltip key={cls}>
                            <TooltipTrigger render={<Toggle aria-label={name} className="aspect-square h-auto w-full text-muted-foreground hover:text-foreground data-pressed:text-primary" onPressedChange={() => onChange(toggle(selected, cls))} pressed={on} variant="outline" />}>
                                <ClassIcon className="pointer-events-none" profession={cls} size={20} />
                                {on && <Check aria-hidden className="absolute top-1 right-1 size-3 opacity-100" />}
                            </TooltipTrigger>
                            <TooltipPopup side="top" sideOffset={8}>
                                {name}
                            </TooltipPopup>
                        </Tooltip>
                    );
                })}
            </div>
        </div>
    );
}
