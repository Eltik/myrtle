import { Combobox, ComboboxEmpty, ComboboxInput, ComboboxItem, ComboboxList, ComboboxPopup, OperatorAvatar } from "frontend";
import { Search } from "lucide-react";

interface IOperator {
    id: string;
    name: string;
    rarity: number;
    profession: string;
}

const OPERATORS: IOperator[] = [
    { id: "char_4064_mlynar", name: "Mlynar", rarity: 6, profession: "Guard" },
    { id: "char_1012_skadi2", name: "Skadi the Corrupting Heart", rarity: 6, profession: "Supporter" },
    { id: "char_2024_chyue", name: "Chongyue", rarity: 6, profession: "Guard" },
    { id: "char_2013_cerber", name: "Cerberus", rarity: 5, profession: "Guard" },
    { id: "char_293_thorns", name: "Thorns", rarity: 6, profession: "Guard" },
];

const MODULES = ["No module", "GUA-X (Stage 3)", "GUA-Y (Stage 2)", "GUA-Delta (Stage 1)"];

export const OperatorRows = () => (
    <div className="h-96 w-80 space-y-2">
        <label className="block font-medium text-[12px] text-muted-foreground leading-none" htmlFor="item-ops">
            Add an operator
        </label>
        <Combobox<IOperator, false> defaultOpen items={OPERATORS} itemToStringLabel={(op) => op.name} itemToStringValue={(op) => op.id}>
            <ComboboxInput id="item-ops" placeholder="Search 231 operators..." startAddon={<Search />} />
            <ComboboxPopup>
                <ComboboxEmpty>No matching operators.</ComboboxEmpty>
                <ComboboxList>
                    {(op: IOperator) => (
                        <ComboboxItem key={op.id} value={op}>
                            <span className="flex items-center gap-2">
                                <span aria-hidden="true" className="inline-flex size-7 shrink-0 items-center justify-center overflow-hidden rounded-md bg-muted font-semibold text-[10px] text-muted-foreground">
                                    <OperatorAvatar charId={op.id} name={op.name} />
                                </span>
                                <span className="flex min-w-0 flex-1 flex-col">
                                    <span className="truncate font-medium text-[13px]">{op.name}</span>
                                    <span className="truncate text-[10.5px] text-muted-foreground">
                                        {op.rarity}★ · {op.profession}
                                    </span>
                                </span>
                            </span>
                        </ComboboxItem>
                    )}
                </ComboboxList>
            </ComboboxPopup>
        </Combobox>
    </div>
);

export const SelectedIndicator = () => (
    <div className="h-72 w-72 space-y-2">
        <label className="block font-medium text-[12px] text-muted-foreground leading-none" htmlFor="item-module">
            Module
        </label>
        <Combobox<string, false> defaultOpen defaultValue="GUA-X (Stage 3)" defaultInputValue="GUA-X (Stage 3)" items={MODULES}>
            <ComboboxInput id="item-module" placeholder="No module" />
            <ComboboxPopup>
                <ComboboxEmpty>No modules unlocked.</ComboboxEmpty>
                <ComboboxList>
                    {(mod: string) => (
                        <ComboboxItem key={mod} value={mod}>
                            {mod}
                        </ComboboxItem>
                    )}
                </ComboboxList>
            </ComboboxPopup>
        </Combobox>
    </div>
);

export const MultiSelected = () => (
    <div className="h-80 w-72 space-y-2">
        <span className="block font-medium text-[12px] text-muted-foreground leading-none">Filter by class</span>
        <Combobox<string, true> defaultOpen multiple defaultValue={["Guard", "Sniper"]} items={["Vanguard", "Guard", "Defender", "Sniper", "Caster", "Medic"]}>
            <ComboboxInput placeholder="2 selected" />
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

export const DisabledItem = () => (
    <div className="h-72 w-72 space-y-2">
        <label className="block font-medium text-[12px] text-muted-foreground leading-none" htmlFor="item-disabled">
            Module
        </label>
        <Combobox<string, false> defaultOpen items={MODULES}>
            <ComboboxInput id="item-disabled" placeholder="No module" />
            <ComboboxPopup>
                <ComboboxEmpty>No modules unlocked.</ComboboxEmpty>
                <ComboboxList>
                    {(mod: string) => (
                        <ComboboxItem key={mod} value={mod} disabled={mod === "GUA-Delta (Stage 1)"}>
                            {mod}
                            {mod === "GUA-Delta (Stage 1)" && <span className="ms-2 text-[10.5px] text-muted-foreground">— E2 lv.40 required</span>}
                        </ComboboxItem>
                    )}
                </ComboboxList>
            </ComboboxPopup>
        </Combobox>
    </div>
);
