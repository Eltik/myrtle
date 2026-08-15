import { Combobox, ComboboxEmpty, ComboboxInput, ComboboxItem, ComboboxList, ComboboxPopup, ComboboxRow } from "frontend";
import { Search } from "lucide-react";

// Base UI turns the listbox into a grid when `grid` is set on the root; each
// DOM row is a `ComboboxRow`, so `items` is an array of rows.
const TAG_ROWS: string[][] = [
    ["Starter", "Senior Operator", "Top Operator"],
    ["Melee", "Ranged", "DP-Recovery"],
    ["Healing", "Support", "DPS"],
    ["Survival", "Defense", "AoE"],
    ["Slow", "Debuff", "Shift"],
    ["Nuker", "Summon", "Fast-Redeploy"],
];

const SQUAD_ROWS: string[][] = [
    ["Mlynar", "Skadi", "Thorns"],
    ["Exusiai", "Ash", "Pozemka"],
    ["Kal'tsit", "Shining", "Nightingale"],
];

export const RecruitmentTags = () => (
    <div className="h-96 w-80 space-y-2">
        <label className="block font-medium text-[12px] text-muted-foreground leading-none" htmlFor="tag-grid">
            Recruitment tags
        </label>
        <Combobox<string, true> defaultOpen grid multiple items={TAG_ROWS} defaultValue={["Senior Operator", "Healing"]}>
            <ComboboxInput id="tag-grid" placeholder="2 tags selected" startAddon={<Search />} />
            <ComboboxPopup className="w-84">
                <ComboboxEmpty>No tags match.</ComboboxEmpty>
                <ComboboxList>
                    {(row: string[]) => (
                        <ComboboxRow key={row.join()} className="grid grid-cols-3 gap-1">
                            {row.map((tag) => (
                                <ComboboxItem key={tag} value={tag} className="text-[12px]">
                                    {tag}
                                </ComboboxItem>
                            ))}
                        </ComboboxRow>
                    )}
                </ComboboxList>
            </ComboboxPopup>
        </Combobox>
    </div>
);

export const SquadGrid = () => (
    <div className="h-80 w-80 space-y-2">
        <label className="block font-medium text-[12px] text-muted-foreground leading-none" htmlFor="squad-grid">
            Squad slot 4
        </label>
        <Combobox<string, false> defaultOpen grid items={SQUAD_ROWS}>
            <ComboboxInput id="squad-grid" placeholder="Search your roster..." startAddon={<Search />} />
            <ComboboxPopup className="w-84">
                <ComboboxEmpty>No operators match.</ComboboxEmpty>
                <ComboboxList>
                    {(row: string[]) => (
                        <ComboboxRow key={row.join()} className="grid grid-cols-3 gap-1">
                            {row.map((name) => (
                                <ComboboxItem key={name} value={name} className="text-[12px]">
                                    {name}
                                </ComboboxItem>
                            ))}
                        </ComboboxRow>
                    )}
                </ComboboxList>
            </ComboboxPopup>
        </Combobox>
    </div>
);

export const ClosedGridField = () => (
    <div className="w-80 space-y-2">
        <label className="block font-medium text-[12px] text-muted-foreground leading-none" htmlFor="tag-grid-closed">
            Recruitment tags
        </label>
        <Combobox<string, true> grid multiple items={TAG_ROWS} defaultValue={["Senior Operator", "Healing"]}>
            <ComboboxInput id="tag-grid-closed" placeholder="2 tags selected" startAddon={<Search />} />
            <ComboboxPopup>
                <ComboboxEmpty>No tags match.</ComboboxEmpty>
                <ComboboxList>
                    {(row: string[]) => (
                        <ComboboxRow key={row.join()} className="grid grid-cols-3 gap-1">
                            {row.map((tag) => (
                                <ComboboxItem key={tag} value={tag} className="text-[12px]">
                                    {tag}
                                </ComboboxItem>
                            ))}
                        </ComboboxRow>
                    )}
                </ComboboxList>
            </ComboboxPopup>
        </Combobox>
        <p className="text-[11px] text-muted-foreground leading-snug">Pick up to 5 tags from a recruitment banner to see guaranteed combinations.</p>
    </div>
);
