import { SelectButton } from "frontend";

/**
 * `SelectButton` is the select-shaped trigger without a Select root — a plain
 * button that carries `selectTriggerVariants`, used where the popup is a menu,
 * dialog or command palette rather than a Base UI Select.
 */

export const Sizes = () => (
    <div className="flex w-72 flex-col gap-3">
        <div className="space-y-1.5">
            <span className="block font-medium text-[12px] text-muted-foreground leading-none">sm</span>
            <SelectButton size="sm">Rarity — 6★</SelectButton>
        </div>
        <div className="space-y-1.5">
            <span className="block font-medium text-[12px] text-muted-foreground leading-none">default</span>
            <SelectButton>EN (Yostar)</SelectButton>
        </div>
        <div className="space-y-1.5">
            <span className="block font-medium text-[12px] text-muted-foreground leading-none">lg</span>
            <SelectButton size="lg">Chapter 8 — Roaring Flare</SelectButton>
        </div>
    </div>
);

export const FilterBar = () => (
    <div className="w-full max-w-2xl rounded-lg border border-border bg-card p-4">
        <div className="mb-3 flex items-center justify-between">
            <span className="font-medium text-foreground text-sm">Operator roster</span>
            <span className="font-mono text-muted-foreground text-xs tabular-nums">231 owned</span>
        </div>
        <div className="flex flex-wrap items-center gap-2">
            <SelectButton className="w-36" size="sm">
                All classes
            </SelectButton>
            <SelectButton className="w-36" size="sm">
                Rarity — 6★
            </SelectButton>
            <SelectButton className="w-36" size="sm">
                Sort: Trust
            </SelectButton>
        </div>
    </div>
);

export const FullWidthField = () => (
    <div className="w-72 space-y-1.5">
        <span className="block font-medium text-[12px] text-muted-foreground leading-none">Farming target</span>
        <SelectButton>10-8 — Ancient Memories</SelectButton>
        <p className="text-muted-foreground text-xs">Sanity cost 21 · drops Bipolar Nanoflake at 8.4%.</p>
    </div>
);

export const Truncated = () => (
    <div className="w-44 space-y-1.5">
        <span className="block font-medium text-[12px] text-muted-foreground leading-none">Module</span>
        <SelectButton size="sm">Mlynar — SWD-Y (Stage 3)</SelectButton>
    </div>
);
