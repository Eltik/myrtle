import { Combobox, ComboboxEmpty, ComboboxItem, ComboboxList, ComboboxPopup, ComboboxTrigger, ComboboxValue } from "frontend";
import { ChevronsUpDown } from "lucide-react";
import type { ReactNode } from "react";

const SORTS = ["Rarity — high to low", "Rarity — low to high", "Name (A–Z)", "Recently updated", "Tier list rank"];
const CLASSES = ["Vanguard", "Guard", "Defender", "Sniper", "Caster", "Medic", "Supporter", "Specialist"];

const TriggerShell = ({ children }: { children: ReactNode }) => (
    <ComboboxTrigger className="flex h-9 w-full cursor-pointer items-center justify-between gap-2 rounded-lg border border-input bg-background px-3 text-left text-sm shadow-xs/5 outline-none hover:bg-accent/40">
        {children}
        <ChevronsUpDown className="size-4 shrink-0 opacity-60" />
    </ComboboxTrigger>
);

export const SelectedValue = () => (
    <div className="w-72 space-y-2">
        <span className="block font-medium text-[12px] text-muted-foreground leading-none">Sort operators by</span>
        <Combobox<string, false> defaultValue="Rarity — high to low" items={SORTS}>
            <TriggerShell>
                <ComboboxValue placeholder="Choose a sort" />
            </TriggerShell>
            <ComboboxPopup>
                <ComboboxEmpty>No matching sorts.</ComboboxEmpty>
                <ComboboxList>
                    {(sort: string) => (
                        <ComboboxItem key={sort} value={sort}>
                            {sort}
                        </ComboboxItem>
                    )}
                </ComboboxList>
            </ComboboxPopup>
        </Combobox>
    </div>
);

export const Placeholder = () => (
    <div className="w-72 space-y-2">
        <span className="block font-medium text-[12px] text-muted-foreground leading-none">Sort operators by</span>
        {/* No children render fn: it would shadow Base UI's placeholder and leave the trigger blank. */}
        <Combobox<string, false> items={SORTS}>
            <TriggerShell>
                <ComboboxValue placeholder="Choose a sort" />
            </TriggerShell>
            <ComboboxPopup>
                <ComboboxEmpty>No matching sorts.</ComboboxEmpty>
                <ComboboxList>
                    {(sort: string) => (
                        <ComboboxItem key={sort} value={sort}>
                            {sort}
                        </ComboboxItem>
                    )}
                </ComboboxList>
            </ComboboxPopup>
        </Combobox>
    </div>
);

export const FormattedValue = () => (
    <div className="w-72 space-y-2">
        <span className="block font-medium text-[12px] text-muted-foreground leading-none">Filter by class</span>
        <Combobox<string, true> multiple defaultValue={["Guard", "Sniper", "Medic"]} items={CLASSES}>
            <TriggerShell>
                <ComboboxValue>{(selected: string[]) => (selected.length === 0 ? "Any class" : `${selected.length} classes — ${selected.join(", ")}`)}</ComboboxValue>
            </TriggerShell>
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

export const FilterBar = () => (
    <div className="flex w-[34rem] items-end gap-3">
        <div className="flex-1 space-y-2">
            <span className="block font-medium text-[12px] text-muted-foreground leading-none">Sort operators by</span>
            <Combobox<string, false> defaultValue="Recently updated" items={SORTS}>
                <TriggerShell>
                    <ComboboxValue placeholder="Choose a sort" />
                </TriggerShell>
                <ComboboxPopup>
                    <ComboboxEmpty>No matching sorts.</ComboboxEmpty>
                    <ComboboxList>
                        {(sort: string) => (
                            <ComboboxItem key={sort} value={sort}>
                                {sort}
                            </ComboboxItem>
                        )}
                    </ComboboxList>
                </ComboboxPopup>
            </Combobox>
        </div>
        <div className="flex-1 space-y-2">
            <span className="block font-medium text-[12px] text-muted-foreground leading-none">Filter by class</span>
            <Combobox<string, true> multiple defaultValue={["Defender"]} items={CLASSES}>
                <TriggerShell>
                    <ComboboxValue>{(selected: string[]) => (selected.length === 0 ? "Any class" : selected.join(", "))}</ComboboxValue>
                </TriggerShell>
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
    </div>
);
