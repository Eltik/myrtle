import { ToggleGroup, ToggleGroupItem, ToggleGroupSeparator } from "frontend";
import { AlignCenterIcon, AlignLeftIcon, AlignRightIcon, BoldIcon, ItalicIcon, UnderlineIcon } from "lucide-react";

/** A hairline between two clusters of a welded outline group. */
export const BetweenClusters = () => (
    <ToggleGroup aria-label="Text formatting" defaultValue={["bold", "left"]} multiple variant="outline">
        <ToggleGroupItem aria-label="Bold" value="bold">
            <BoldIcon />
        </ToggleGroupItem>
        <ToggleGroupItem aria-label="Italic" value="italic">
            <ItalicIcon />
        </ToggleGroupItem>
        <ToggleGroupItem aria-label="Underline" value="underline">
            <UnderlineIcon />
        </ToggleGroupItem>
        <ToggleGroupSeparator />
        <ToggleGroupItem aria-label="Align left" value="left">
            <AlignLeftIcon />
        </ToggleGroupItem>
        <ToggleGroupItem aria-label="Align centre" value="center">
            <AlignCenterIcon />
        </ToggleGroupItem>
        <ToggleGroupItem aria-label="Align right" value="right">
            <AlignRightIcon />
        </ToggleGroupItem>
    </ToggleGroup>
);

/** Splitting rarity from server inside one filter control. */
export const SplittingFilters = () => (
    <ToggleGroup aria-label="Roster filters" defaultValue={["6", "en"]} multiple variant="outline">
        <ToggleGroupItem value="6">6★</ToggleGroupItem>
        <ToggleGroupItem value="5">5★</ToggleGroupItem>
        <ToggleGroupSeparator />
        <ToggleGroupItem value="en">EN</ToggleGroupItem>
        <ToggleGroupItem value="jp">JP</ToggleGroupItem>
        <ToggleGroupItem value="cn">CN</ToggleGroupItem>
    </ToggleGroup>
);

/** Vertical groups get a horizontal rule instead. */
export const Vertical = () => (
    <ToggleGroup aria-label="Depot filters" defaultValue={["t5"]} orientation="vertical" variant="outline">
        <ToggleGroupItem value="t5">T5 materials</ToggleGroupItem>
        <ToggleGroupItem value="t4">T4 materials</ToggleGroupItem>
        <ToggleGroupSeparator orientation="horizontal" />
        <ToggleGroupItem value="missing">Missing only</ToggleGroupItem>
    </ToggleGroup>
);
