import { Toggle, ToggleGroupPrimitive } from "frontend";
import { Grid3x3Icon, LayoutGridIcon, RowsIcon } from "lucide-react";

// ToggleGroupPrimitive is the unstyled Base UI root re-exported from toggle-group.tsx.
// It supplies the shared pressed state and roving focus; the layout and the item
// styling are yours. `ToggleGroup` is this root plus the DS's welded-segment rules.

/** Roving-focus state with a layout the DS's own ToggleGroup does not provide. */
export const CustomLayout = () => (
    <ToggleGroupPrimitive aria-label="Roster view" className="grid w-full max-w-xs grid-cols-3 gap-1 rounded-xl border bg-card p-1" defaultValue={["detailed"]}>
        <Toggle className="w-full" value="detailed">
            <LayoutGridIcon />
            Detailed
        </Toggle>
        <Toggle className="w-full" value="compact">
            <Grid3x3Icon />
            Compact
        </Toggle>
        <Toggle className="w-full" value="list">
            <RowsIcon />
            List
        </Toggle>
    </ToggleGroupPrimitive>
);

/** `multiple` on the root — several rarity filters pressed at once. */
export const MultiSelect = () => (
    <ToggleGroupPrimitive aria-label="Rarity filter" className="flex w-fit flex-wrap items-center gap-1.5" defaultValue={["6", "5"]} multiple>
        {["6★", "5★", "4★", "3★"].map((stars) => (
            <Toggle key={stars} value={stars.charAt(0)} variant="outline">
                {stars}
            </Toggle>
        ))}
    </ToggleGroupPrimitive>
);

/** Vertical orientation with the group disabled — a locked filter rail. */
export const DisabledVertical = () => (
    <ToggleGroupPrimitive aria-label="Depot filters" className="flex w-40 flex-col gap-1 rounded-xl border bg-card p-1" defaultValue={["t5"]} disabled orientation="vertical">
        <Toggle className="w-full justify-start" value="t5">
            T5 materials
        </Toggle>
        <Toggle className="w-full justify-start" value="t4">
            T4 materials
        </Toggle>
        <Toggle className="w-full justify-start" value="missing">
            Missing only
        </Toggle>
    </ToggleGroupPrimitive>
);
