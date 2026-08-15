import { Combobox, ComboboxChip, ComboboxChips, ComboboxChipsInput, ComboboxEmpty, ComboboxItem, ComboboxList, ComboboxPopup } from "frontend";
import { Tag, Users } from "lucide-react";

const TAGS = ["Senior Operator", "Healing", "DP-Recovery", "Fast-Redeploy", "Crowd-Control", "Nuker"];
const OPERATORS = ["Mlynar", "Skadi the Corrupting Heart", "Muelsyse", "Texas the Omertosa", "Eyjafjalla", "Myrtle"];

const tagPopup = (
    <ComboboxPopup>
        <ComboboxEmpty>No matches</ComboboxEmpty>
        <ComboboxList>
            {(tag: string) => (
                <ComboboxItem key={tag} value={tag}>
                    {tag}
                </ComboboxItem>
            )}
        </ComboboxList>
    </ComboboxPopup>
);

export const RemovableTags = () => (
    <div className="w-96 space-y-2">
        <span className="block font-medium text-[12px] text-muted-foreground leading-none">Recruitment tags</span>
        <Combobox<string, true> multiple defaultValue={["Senior Operator", "Healing", "DP-Recovery"]} items={TAGS}>
            <ComboboxChips startAddon={<Tag />}>
                <ComboboxChip>Senior Operator</ComboboxChip>
                <ComboboxChip>Healing</ComboboxChip>
                <ComboboxChip>DP-Recovery</ComboboxChip>
                <ComboboxChipsInput placeholder="Add tag..." />
            </ComboboxChips>
            {tagPopup}
        </Combobox>
        <p className="text-[11px] text-muted-foreground leading-snug">Each ✕ drops one tag from the recruitment query.</p>
    </div>
);

export const LabelledRemove = () => (
    <div className="w-96 space-y-2">
        <span className="block font-medium text-[12px] text-muted-foreground leading-none">Squad</span>
        <Combobox<string, true> multiple defaultValue={["Mlynar", "Myrtle"]} items={OPERATORS}>
            <ComboboxChips startAddon={<Users />}>
                <ComboboxChip removeProps={{ "aria-label": "Remove Mlynar from the squad" }}>Mlynar</ComboboxChip>
                <ComboboxChip removeProps={{ "aria-label": "Remove Myrtle from the squad" }}>Myrtle</ComboboxChip>
                <ComboboxChipsInput placeholder="Add operator..." />
            </ComboboxChips>
            <ComboboxPopup>
                <ComboboxEmpty>No matching operators.</ComboboxEmpty>
                <ComboboxList>
                    {(name: string) => (
                        <ComboboxItem key={name} value={name}>
                            {name}
                        </ComboboxItem>
                    )}
                </ComboboxList>
            </ComboboxPopup>
        </Combobox>
    </div>
);

export const TintedChips = () => (
    <div className="w-96 space-y-2">
        <span className="block font-medium text-[12px] text-muted-foreground leading-none">Recruitment tags</span>
        <Combobox<string, true> multiple defaultValue={["Nuker", "Crowd-Control"]} items={TAGS}>
            <ComboboxChips startAddon={<Tag />}>
                {["Nuker", "Crowd-Control"].map((tag) => (
                    <ComboboxChip
                        key={tag}
                        className="flex items-center rounded-[calc(var(--radius-md)-1px)] bg-primary/16 ps-2 font-medium text-foreground text-sm outline-none sm:text-xs/(--text-xs--line-height)"
                        removeProps={{ className: "h-full shrink-0 cursor-pointer px-1.5 text-primary opacity-90 hover:opacity-100 [&_svg:not([class*='size-'])]:size-4 sm:[&_svg:not([class*='size-'])]:size-3.5" }}
                    >
                        {tag}
                    </ComboboxChip>
                ))}
                <ComboboxChipsInput placeholder="Add tag..." />
            </ComboboxChips>
            {tagPopup}
        </Combobox>
        <p className="text-[11px] text-muted-foreground leading-snug">Brand-tinted chips keep the same remove affordance.</p>
    </div>
);
