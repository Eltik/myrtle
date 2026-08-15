import { Combobox, ComboboxChip, ComboboxChips, ComboboxChipsInput, ComboboxEmpty, ComboboxItem, ComboboxList, ComboboxPopup } from "frontend";
import { Package, Users } from "lucide-react";

const OPERATORS = ["Mlynar", "Skadi the Corrupting Heart", "Muelsyse", "Texas the Omertosa", "Eyjafjalla", "Myrtle"];
const MATERIALS = ["D32 Steel", "Bipolar Nanoflake", "Orirock Cube", "Sugar Pack", "Grindstone"];

const opPopup = (
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
);

export const SquadChips = () => (
    <div className="w-96 space-y-2">
        <span className="block font-medium text-[12px] text-muted-foreground leading-none">Squad</span>
        <Combobox<string, true> multiple defaultValue={["Mlynar", "Myrtle", "Eyjafjalla"]} items={OPERATORS}>
            <ComboboxChips startAddon={<Users />}>
                <ComboboxChip>Mlynar</ComboboxChip>
                <ComboboxChip>Myrtle</ComboboxChip>
                <ComboboxChip>Eyjafjalla</ComboboxChip>
                <ComboboxChipsInput placeholder="Add operator..." />
            </ComboboxChips>
            {opPopup}
        </Combobox>
        <p className="text-[11px] text-muted-foreground leading-snug">3 of 12 deployment slots filled.</p>
    </div>
);

export const SingleChip = () => (
    <div className="w-96 space-y-2">
        <span className="block font-medium text-[12px] text-muted-foreground leading-none">Squad</span>
        <Combobox<string, true> multiple defaultValue={["Skadi the Corrupting Heart"]} items={OPERATORS}>
            <ComboboxChips startAddon={<Users />}>
                <ComboboxChip>Skadi the Corrupting Heart</ComboboxChip>
                <ComboboxChipsInput placeholder="Add operator..." />
            </ComboboxChips>
            {opPopup}
        </Combobox>
    </div>
);

export const MaterialChips = () => (
    <div className="w-96 space-y-2">
        <span className="block font-medium text-[12px] text-muted-foreground leading-none">Materials to track</span>
        <Combobox<string, true> multiple defaultValue={MATERIALS.slice(0, 4)} items={MATERIALS}>
            <ComboboxChips startAddon={<Package />}>
                {MATERIALS.slice(0, 4).map((mat) => (
                    <ComboboxChip key={mat}>{mat}</ComboboxChip>
                ))}
                <ComboboxChipsInput placeholder="Add material..." />
            </ComboboxChips>
            <ComboboxPopup>
                <ComboboxEmpty>No matching materials.</ComboboxEmpty>
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

export const NoChips = () => (
    <div className="w-80 space-y-2">
        <span className="block font-medium text-[12px] text-muted-foreground leading-none">Squad</span>
        <Combobox<string, true> multiple items={OPERATORS}>
            <ComboboxChips startAddon={<Users />}>
                <ComboboxChipsInput placeholder="Add operator..." />
            </ComboboxChips>
            {opPopup}
        </Combobox>
        <p className="text-[11px] text-muted-foreground leading-snug">0 of 12 deployment slots filled.</p>
    </div>
);
