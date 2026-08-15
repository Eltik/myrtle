import { Combobox, ComboboxChip, ComboboxChips, ComboboxChipsInput, ComboboxEmpty, ComboboxItem, ComboboxList, ComboboxPopup } from "frontend";
import { SlidersHorizontal, Tag } from "lucide-react";

const CLASSES = ["Vanguard", "Guard", "Defender", "Sniper", "Caster", "Medic", "Supporter", "Specialist"];
const TAGS = ["Senior Operator", "Healing", "DP-Recovery", "Fast-Redeploy", "Crowd-Control", "Nuker", "Summon", "Shift"];

export const ClassFilter = () => (
    <div className="w-80 space-y-2">
        <span className="block font-medium text-[12px] text-muted-foreground leading-none">Filter by class</span>
        <Combobox<string, true> multiple defaultValue={["Guard", "Sniper", "Medic"]} items={CLASSES}>
            <ComboboxChips>
                <ComboboxChip>Guard</ComboboxChip>
                <ComboboxChip>Sniper</ComboboxChip>
                <ComboboxChip>Medic</ComboboxChip>
                <ComboboxChipsInput placeholder="Add class..." />
            </ComboboxChips>
            <ComboboxPopup>
                <ComboboxEmpty>No matches</ComboboxEmpty>
                <ComboboxList>
                    {(cls: string) => (
                        <ComboboxItem key={cls} value={cls}>
                            {cls}
                        </ComboboxItem>
                    )}
                </ComboboxList>
            </ComboboxPopup>
        </Combobox>
    </div>
);

export const WithStartAddon = () => (
    <div className="w-80 space-y-2">
        <span className="block font-medium text-[12px] text-muted-foreground leading-none">Recruitment tags</span>
        <Combobox<string, true> multiple defaultValue={["Senior Operator", "Healing"]} items={TAGS}>
            <ComboboxChips startAddon={<Tag />}>
                <ComboboxChip>Senior Operator</ComboboxChip>
                <ComboboxChip>Healing</ComboboxChip>
                <ComboboxChipsInput placeholder="Add tag..." />
            </ComboboxChips>
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
        </Combobox>
    </div>
);

export const EmptyField = () => (
    <div className="w-80 space-y-2">
        <span className="block font-medium text-[12px] text-muted-foreground leading-none">Filter by class</span>
        <Combobox<string, true> multiple items={CLASSES}>
            <ComboboxChips startAddon={<SlidersHorizontal />}>
                <ComboboxChipsInput placeholder="Any class" />
            </ComboboxChips>
            <ComboboxPopup>
                <ComboboxEmpty>No matches</ComboboxEmpty>
                <ComboboxList>
                    {(cls: string) => (
                        <ComboboxItem key={cls} value={cls}>
                            {cls}
                        </ComboboxItem>
                    )}
                </ComboboxList>
            </ComboboxPopup>
        </Combobox>
        <p className="text-[11px] text-muted-foreground leading-snug">Showing all 231 operators.</p>
    </div>
);

export const Overflowing = () => (
    <div className="w-80 space-y-2">
        <span className="block font-medium text-[12px] text-muted-foreground leading-none">Recruitment tags</span>
        <Combobox<string, true> multiple defaultValue={TAGS.slice(0, 6)} items={TAGS}>
            <ComboboxChips startAddon={<Tag />}>
                {TAGS.slice(0, 6).map((tag) => (
                    <ComboboxChip key={tag}>{tag}</ComboboxChip>
                ))}
                <ComboboxChipsInput placeholder="Add tag..." />
            </ComboboxChips>
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
        </Combobox>
    </div>
);
