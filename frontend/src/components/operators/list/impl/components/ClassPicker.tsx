import { Tooltip, TooltipPopup, TooltipTrigger } from "#/components/ui/tooltip";
import { useT } from "#/lib/i18n";
import type { TypedT } from "#/lib/i18n/messages";
import { cn, professionLabel } from "#/lib/utils";
import { CLASSES } from "../constants";
import type { messages } from "./ClassPicker.messages";
import { ClassIcon } from "./Icons";
import styles from "./OperatorFilters.module.css";

export function toggle<T>(list: T[], value: T): T[] {
    return list.includes(value) ? list.filter((v) => v !== value) : [...list, value];
}

/** `labels` maps a class code to the server's own name; absent, the English formatter names it. */
export function ClassPicker({ selected, onChange, labels }: { selected: string[]; onChange: (v: string[]) => void; labels?: Record<string, string> }) {
    const t: TypedT<typeof messages> = useT("operators");

    return (
        <div className={styles.field}>
            <div className={styles.fieldLabel}>{t("filters.class")}</div>
            <div className={styles.classRow}>
                {CLASSES.map((cls) => {
                    const on = selected.includes(cls);
                    const name = professionLabel({ profession: cls, professionName: labels?.[cls] });
                    return (
                        <Tooltip key={`tooltip-${cls}`}>
                            <TooltipTrigger
                                render={
                                    <button key={cls} type="button" title={name} className={cn(styles.classBtn, on && styles.on)} onClick={() => onChange(toggle(selected, cls))} aria-pressed={on}>
                                        <ClassIcon profession={cls} size={20} />
                                    </button>
                                }
                            />
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
