import { Combobox, ComboboxChip, ComboboxChips, ComboboxChipsInput, ComboboxEmpty, ComboboxItem, ComboboxList, ComboboxPopup } from "frontend";
import { Tag, Users } from "lucide-react";

const FACTIONS = ["Rhodes Island", "Penguin Logistics", "Abyssal Hunters", "Karlan Trade", "Rhine Lab", "Blacksteel", "Lungmen Guard Dept.", "Elite Op"];

const Popup = () => (
    <ComboboxPopup>
        <ComboboxEmpty>No matches</ComboboxEmpty>
        <ComboboxList>
            {(faction: string) => (
                <ComboboxItem key={faction} value={faction}>
                    {faction}
                </ComboboxItem>
            )}
        </ComboboxList>
    </ComboboxPopup>
);

export const Default = () => (
    <div className="w-96 space-y-2">
        <span className="block font-medium text-[12px] text-muted-foreground leading-none">Filter by faction</span>
        <Combobox<string, true> multiple defaultValue={["Rhine Lab", "Abyssal Hunters"]} items={FACTIONS}>
            <ComboboxChips startAddon={<Users />}>
                <ComboboxChip>Rhine Lab</ComboboxChip>
                <ComboboxChip>Abyssal Hunters</ComboboxChip>
                <ComboboxChipsInput placeholder="Add faction..." />
            </ComboboxChips>
            <Popup />
        </Combobox>
    </div>
);

export const PlaceholderOnly = () => (
    <div className="w-80 space-y-2">
        <span className="block font-medium text-[12px] text-muted-foreground leading-none">Filter by faction</span>
        <Combobox<string, true> multiple items={FACTIONS}>
            <ComboboxChips startAddon={<Users />}>
                <ComboboxChipsInput placeholder="Any faction" />
            </ComboboxChips>
            <Popup />
        </Combobox>
        <p className="text-[11px] text-muted-foreground leading-snug">Leave empty to include every faction in the roster.</p>
    </div>
);

export const Sizes = () => (
    <div className="flex w-80 flex-col gap-3">
        <Combobox<string, true> multiple defaultValue={["Elite Op"]} items={FACTIONS}>
            <ComboboxChips startAddon={<Tag />}>
                <ComboboxChip>Elite Op</ComboboxChip>
                <ComboboxChipsInput size="sm" placeholder="Small" />
            </ComboboxChips>
            <Popup />
        </Combobox>
        <Combobox<string, true> multiple defaultValue={["Elite Op"]} items={FACTIONS}>
            <ComboboxChips startAddon={<Tag />}>
                <ComboboxChip>Elite Op</ComboboxChip>
                <ComboboxChipsInput placeholder="Default" />
            </ComboboxChips>
            <Popup />
        </Combobox>
        <Combobox<string, true> multiple defaultValue={["Elite Op"]} items={FACTIONS}>
            <ComboboxChips startAddon={<Tag />}>
                <ComboboxChip>Elite Op</ComboboxChip>
                <ComboboxChipsInput size="lg" placeholder="Large" />
            </ComboboxChips>
            <Popup />
        </Combobox>
    </div>
);

export const Disabled = () => (
    <div className="w-80 space-y-2">
        <span className="block font-medium text-[12px] text-muted-foreground leading-none">Filter by faction</span>
        <Combobox<string, true> disabled multiple defaultValue={["Penguin Logistics"]} items={FACTIONS}>
            <ComboboxChips startAddon={<Users />}>
                <ComboboxChip>Penguin Logistics</ComboboxChip>
                <ComboboxChipsInput placeholder="Add faction..." />
            </ComboboxChips>
        </Combobox>
        <p className="text-[11px] text-muted-foreground leading-snug">Faction filters are locked while a saved preset is applied.</p>
    </div>
);
