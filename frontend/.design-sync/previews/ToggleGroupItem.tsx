import { ToggleGroup, ToggleGroupItem } from "frontend";
import { Grid3x3Icon, LayoutGridIcon, RowsIcon } from "lucide-react";

/** Items inherit variant and size from the group; only `value` is per-item. */
export const IconItems = () => (
    <ToggleGroup aria-label="View mode" defaultValue={["detailed"]} variant="outline">
        <ToggleGroupItem aria-label="Detailed view" value="detailed">
            <LayoutGridIcon />
        </ToggleGroupItem>
        <ToggleGroupItem aria-label="Compact view" value="compact">
            <Grid3x3Icon />
        </ToggleGroupItem>
        <ToggleGroupItem aria-label="List view" value="list">
            <RowsIcon />
        </ToggleGroupItem>
    </ToggleGroup>
);

/** Icon plus label — the roster's view switcher with text. */
export const LabelledItems = () => (
    <ToggleGroup aria-label="View mode" defaultValue={["compact"]} variant="outline">
        <ToggleGroupItem value="detailed">
            <LayoutGridIcon />
            Detailed
        </ToggleGroupItem>
        <ToggleGroupItem value="compact">
            <Grid3x3Icon />
            Compact
        </ToggleGroupItem>
        <ToggleGroupItem value="list">
            <RowsIcon />
            List
        </ToggleGroupItem>
    </ToggleGroup>
);

/** A disabled item: the CN-only server has no leaderboard snapshot yet. */
export const DisabledItem = () => (
    <ToggleGroup aria-label="Server" defaultValue={["en"]} variant="outline">
        <ToggleGroupItem value="en">EN</ToggleGroupItem>
        <ToggleGroupItem value="jp">JP</ToggleGroupItem>
        <ToggleGroupItem value="kr">KR</ToggleGroupItem>
        <ToggleGroupItem disabled value="cn">
            CN
        </ToggleGroupItem>
    </ToggleGroup>
);

/** Several items pressed at once when the root is `multiple`. */
export const MultiplePressed = () => (
    <ToggleGroup aria-label="Rarity filter" defaultValue={["6", "5"]} multiple variant="outline">
        <ToggleGroupItem value="6">6★</ToggleGroupItem>
        <ToggleGroupItem value="5">5★</ToggleGroupItem>
        <ToggleGroupItem value="4">4★</ToggleGroupItem>
        <ToggleGroupItem value="3">3★</ToggleGroupItem>
    </ToggleGroup>
);
