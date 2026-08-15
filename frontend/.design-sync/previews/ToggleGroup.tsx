import { ToggleGroup, ToggleGroupItem } from "frontend";
import { Grid3x3Icon, LayoutGridIcon, LayoutListIcon, ListIcon, UsersIcon } from "lucide-react";

/** Ported from the planner's RequirementsPanel — outline variant, icon-only, single select. */
export const RequirementsView = () => (
    <ToggleGroup aria-label="Requirements view" defaultValue={["grouped"]} variant="outline">
        <ToggleGroupItem aria-label="Grouped view" value="grouped">
            <LayoutListIcon />
        </ToggleGroupItem>
        <ToggleGroupItem aria-label="Flat view" value="flat">
            <ListIcon />
        </ToggleGroupItem>
        <ToggleGroupItem aria-label="By operator view" value="by-operator">
            <UsersIcon />
        </ToggleGroupItem>
    </ToggleGroup>
);

/** The two variants: spaced pills by default, a welded segmented control with `outline`. */
export const Variants = () => (
    <div className="flex flex-col items-start gap-4">
        <ToggleGroup aria-label="Roster view (default)" defaultValue={["detailed"]}>
            <ToggleGroupItem value="detailed">
                <LayoutGridIcon />
                Detailed
            </ToggleGroupItem>
            <ToggleGroupItem value="compact">
                <Grid3x3Icon />
                Compact
            </ToggleGroupItem>
        </ToggleGroup>
        <ToggleGroup aria-label="Roster view (outline)" defaultValue={["compact"]} variant="outline">
            <ToggleGroupItem value="detailed">
                <LayoutGridIcon />
                Detailed
            </ToggleGroupItem>
            <ToggleGroupItem value="compact">
                <Grid3x3Icon />
                Compact
            </ToggleGroupItem>
        </ToggleGroup>
    </div>
);

/** `multiple` on the root lets several server filters stay pressed at once. */
export const MultiSelect = () => (
    <ToggleGroup aria-label="Servers" defaultValue={["en", "jp"]} multiple variant="outline">
        <ToggleGroupItem value="en">EN</ToggleGroupItem>
        <ToggleGroupItem value="cn">CN</ToggleGroupItem>
        <ToggleGroupItem value="jp">JP</ToggleGroupItem>
        <ToggleGroupItem value="kr">KR</ToggleGroupItem>
    </ToggleGroup>
);

/** Sizes, and a vertical group for a narrow filter rail. */
export const SizesAndOrientation = () => (
    <div className="flex flex-wrap items-start gap-8">
        <div className="flex flex-col items-start gap-3">
            <ToggleGroup aria-label="Interval (small)" defaultValue={["7d"]} size="sm" variant="outline">
                <ToggleGroupItem value="24h">24h</ToggleGroupItem>
                <ToggleGroupItem value="7d">7d</ToggleGroupItem>
                <ToggleGroupItem value="30d">30d</ToggleGroupItem>
            </ToggleGroup>
            <ToggleGroup aria-label="Interval (large)" defaultValue={["30d"]} size="lg" variant="outline">
                <ToggleGroupItem value="24h">24h</ToggleGroupItem>
                <ToggleGroupItem value="7d">7d</ToggleGroupItem>
                <ToggleGroupItem value="30d">30d</ToggleGroupItem>
            </ToggleGroup>
        </div>
        <ToggleGroup aria-label="Rarity" defaultValue={["6"]} orientation="vertical" variant="outline">
            <ToggleGroupItem value="6">6★</ToggleGroupItem>
            <ToggleGroupItem value="5">5★</ToggleGroupItem>
            <ToggleGroupItem value="4">4★</ToggleGroupItem>
        </ToggleGroup>
    </div>
);
