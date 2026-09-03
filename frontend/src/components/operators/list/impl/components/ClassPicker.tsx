import { Tooltip, TooltipPopup, TooltipTrigger } from "#/components/ui/tooltip";
import { cn, formatProfession } from "#/lib/utils";
import { CLASSES } from "../constants";
import { ClassIcon } from "./Icons";
import styles from "./OperatorFilters.module.css";

export function toggle<T>(list: T[], value: T): T[] {
    return list.includes(value) ? list.filter((v) => v !== value) : [...list, value];
}

export function ClassPicker({ selected, onChange }: { selected: string[]; onChange: (v: string[]) => void }) {
    return (
        <div className={styles.field}>
            <div className={styles.fieldLabel}>Class</div>
            <div className={styles.classRow}>
                {CLASSES.map((cls) => {
                    const on = selected.includes(cls);
                    return (
                        <Tooltip key={`tooltip-${cls}`}>
                            <TooltipTrigger
                                render={
                                    <button key={cls} type="button" title={formatProfession(cls)} className={cn(styles.classBtn, on && styles.on)} onClick={() => onChange(toggle(selected, cls))} aria-pressed={on}>
                                        <ClassIcon profession={cls} size={20} />
                                    </button>
                                }
                            />
                            <TooltipPopup side="top" sideOffset={8}>
                                {formatProfession(cls)}
                            </TooltipPopup>
                        </Tooltip>
                    );
                })}
            </div>
        </div>
    );
}
