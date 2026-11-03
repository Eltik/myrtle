import { X } from "lucide-react";
import { useT } from "#/lib/i18n";
import type { TypedT } from "#/lib/i18n/messages";
import type { messages } from "./ActiveFilterChips.messages";

export type IActiveChip = { key: string; label: string; onRemove: () => void };

export function ActiveFilterChips({ chips, onClearAll }: { chips: IActiveChip[]; onClearAll: () => void }) {
    const t: TypedT<typeof messages> = useT("operators");

    if (chips.length === 0) return null;

    return (
        <div className="flex flex-wrap items-center gap-1.5 font-medium font-sans text-[12px] text-muted-foreground leading-none">
            <span className="mr-0.5">{t("chips.active")}</span>
            {chips.map((chip) => (
                <span
                    className="inline-flex items-center gap-1.5 rounded-full bg-[color-mix(in_oklch,var(--primary)_10%,transparent)] py-1 pr-1 pl-2.25 font-medium font-sans text-[11.5px] text-foreground leading-none [&>button:hover]:bg-[color-mix(in_oklch,var(--primary)_32%,transparent)] [&>button]:relative [&>button]:inline-flex [&>button]:h-3.75 [&>button]:w-3.75 [&>button]:cursor-pointer [&>button]:items-center [&>button]:justify-center [&>button]:rounded-full [&>button]:border-0 [&>button]:bg-[color-mix(in_oklch,var(--primary)_22%,transparent)] [&>button]:p-0 [&>button]:pointer-coarse:after:absolute [&>button]:pointer-coarse:after:min-h-11 [&>button]:pointer-coarse:after:min-w-11"
                    key={chip.key}
                >
                    {chip.label}
                    <button type="button" onClick={chip.onRemove} aria-label={t("chips.remove", { label: chip.label })}>
                        <X className="h-2 w-2" aria-hidden="true" />
                    </button>
                </span>
            ))}
            <button
                type="button"
                className="relative cursor-pointer appearance-none border-0 bg-transparent p-0 font-medium font-sans text-[12px] text-muted-foreground leading-none underline underline-offset-[3px] pointer-coarse:after:absolute pointer-coarse:after:inset-x-0 pointer-coarse:after:top-1/2 pointer-coarse:after:min-h-11 pointer-coarse:after:-translate-y-1/2 hover:text-foreground"
                onClick={onClearAll}
            >
                {t("chips.clearAll")}
            </button>
        </div>
    );
}
