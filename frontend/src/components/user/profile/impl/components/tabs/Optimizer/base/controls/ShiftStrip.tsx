import { ToggleGroup, ToggleGroupItem } from "#/components/ui/toggle-group";
import { useBaseOptimizer } from "../base-context";

export function ShiftStrip() {
    const api = useBaseOptimizer();
    if (api.shiftCount === 0) return null;

    const value = api.viewShift === null ? "now" : String(api.viewShift);

    return (
        <ToggleGroup
            aria-label="Rotation shift"
            onValueChange={(next: string[]) => {
                const picked = next[0];
                if (!picked) return;
                api.setViewShift(picked === "now" ? null : Number(picked));
            }}
            value={[value]}
        >
            <ToggleGroupItem size="sm" value="now">
                Stationed now
            </ToggleGroupItem>
            {Array.from({ length: api.shiftCount }, (_, i) => i + 1).map((shift) => (
                <ToggleGroupItem key={shift} size="sm" value={String(shift)}>
                    Shift {shift}
                </ToggleGroupItem>
            ))}
        </ToggleGroup>
    );
}
