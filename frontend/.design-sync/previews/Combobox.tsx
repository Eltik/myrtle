import { Combobox, ComboboxChip, ComboboxChips, ComboboxChipsInput, ComboboxEmpty, ComboboxInput, ComboboxItem, ComboboxList, ComboboxPopup, OperatorAvatar } from "frontend";
import { Search, SlidersHorizontal } from "lucide-react";

interface IOperator {
    id: string;
    name: string;
    rarity: number;
    profession: string;
}

const OPERATORS: IOperator[] = [
    { id: "char_2015_dusk", name: "Dusk", rarity: 6, profession: "Caster" },
    { id: "char_4064_mlynar", name: "Mlynar", rarity: 6, profession: "Guard" },
    { id: "char_1012_skadi2", name: "Skadi the Corrupting Heart", rarity: 6, profession: "Supporter" },
    { id: "char_2023_ling", name: "Ling", rarity: 6, profession: "Supporter" },
    { id: "char_1028_texas2", name: "Texas the Omertosa", rarity: 6, profession: "Specialist" },
    { id: "char_1020_reed2", name: "Reed the Flame Shadow", rarity: 6, profession: "Medic" },
    { id: "char_003_kalts", name: "Kal'tsit", rarity: 6, profession: "Medic" },
    { id: "char_151_myrtle", name: "Myrtle", rarity: 4, profession: "Vanguard" },
];

const Row = ({ op }: { op: IOperator }) => (
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
);

export const OperatorSearch = () => (
    <div className="w-80 space-y-2">
        <label className="block font-medium text-[12px] text-muted-foreground leading-none" htmlFor="op-search">
            Add an operator
        </label>
        <Combobox<IOperator, false> items={OPERATORS} itemToStringLabel={(op) => op.name} itemToStringValue={(op) => op.id}>
            <ComboboxInput id="op-search" placeholder="Search 231 operators..." startAddon={<Search />} />
            <ComboboxPopup>
                <ComboboxEmpty>No matching operators.</ComboboxEmpty>
                <ComboboxList>{(op: IOperator) => <ComboboxItem key={op.id} value={op}>{<Row op={op} />}</ComboboxItem>}</ComboboxList>
            </ComboboxPopup>
        </Combobox>
        <p className="text-[11px] text-muted-foreground leading-snug">Pick any operator to start. You can add the same one twice to compare builds.</p>
    </div>
);

export const OpenWithResults = () => (
    <div className="h-96 w-80 space-y-2">
        <label className="block font-medium text-[12px] text-muted-foreground leading-none" htmlFor="op-search-open">
            Add an operator
        </label>
        <Combobox<IOperator, false> defaultOpen items={OPERATORS} itemToStringLabel={(op) => op.name} itemToStringValue={(op) => op.id}>
            <ComboboxInput id="op-search-open" placeholder="Search 231 operators..." startAddon={<Search />} />
            <ComboboxPopup>
                <ComboboxEmpty>No matching operators.</ComboboxEmpty>
                <ComboboxList>{(op: IOperator) => <ComboboxItem key={op.id} value={op}>{<Row op={op} />}</ComboboxItem>}</ComboboxList>
            </ComboboxPopup>
        </Combobox>
    </div>
);

export const MultipleWithChips = () => (
    <div className="w-96 space-y-2">
        <span className="block font-medium text-[12px] text-muted-foreground leading-none">Filter by class</span>
        <Combobox<string, true> multiple defaultValue={["Guard", "Sniper", "Medic"]} items={["Vanguard", "Guard", "Defender", "Sniper", "Caster", "Medic", "Supporter", "Specialist"]}>
            <ComboboxChips startAddon={<SlidersHorizontal />}>
                <ComboboxChip>Guard</ComboboxChip>
                <ComboboxChip>Sniper</ComboboxChip>
                <ComboboxChip>Medic</ComboboxChip>
                <ComboboxChipsInput placeholder="Add class..." />
            </ComboboxChips>
            <ComboboxPopup>
                <ComboboxEmpty>No matches</ComboboxEmpty>
                <ComboboxList>{(cls: string) => <ComboboxItem key={cls} value={cls}>{cls}</ComboboxItem>}</ComboboxList>
            </ComboboxPopup>
        </Combobox>
    </div>
);

export const Disabled = () => (
    <div className="w-80 space-y-2">
        <label className="block font-medium text-[12px] text-muted-foreground leading-none" htmlFor="op-search-disabled">
            Add an operator
        </label>
        <Combobox<IOperator, false> disabled items={OPERATORS} itemToStringLabel={(op) => op.name} itemToStringValue={(op) => op.id}>
            <ComboboxInput id="op-search-disabled" placeholder="Roster locked while syncing..." startAddon={<Search />} />
        </Combobox>
        <p className="text-[11px] text-muted-foreground leading-snug">Import from Yostar is running — the picker unlocks when it finishes.</p>
    </div>
);
