import { ToggleGroup, ToggleGroupItem } from "#/components/ui/toggle-group";
import { useT } from "#/lib/i18n";
import type { TypedT } from "#/lib/i18n/messages";
import { useBaseOptimizer } from "../base-context";
import type { messages } from "./ShiftStrip.messages";

export function ShiftStrip() {
    const t: TypedT<typeof messages> = useT("user");
    const api = useBaseOptimizer();
    if (api.shiftCount === 0) return null;

    const value = api.viewShift === null ? "now" : String(api.viewShift);

    return (
        <ToggleGroup
            aria-label={t("profile.base.shift.aria")}
            onValueChange={(next: string[]) => {
                const picked = next[0];
                if (!picked) return;
                api.setViewShift(picked === "now" ? null : Number(picked));
            }}
            value={[value]}
        >
            <ToggleGroupItem size="sm" value="now">
                {t("profile.base.shift.now")}
            </ToggleGroupItem>
            {Array.from({ length: api.shiftCount }, (_, i) => i + 1).map((shift) => (
                <ToggleGroupItem key={shift} size="sm" value={String(shift)}>
                    {t("profile.base.shift.n", { n: shift })}
                </ToggleGroupItem>
            ))}
        </ToggleGroup>
    );
}
