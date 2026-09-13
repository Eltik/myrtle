import { ActiveFilterChips, PaginationCompact } from "frontend";

const noop = () => {};

// Chips are what `buildSharedChips` derives from `ISharedFilters`: one per
// selected value, keyed `<prefix>-<value>`, labelled through `CHIP_CONFIG`
// (rarity → "6★", class → "Guard", archetype → "Liberator Guard", nation →
// "Kazimierz", faction → "Penguin Logistics", gender/race/artist verbatim).
const chip = (key: string, label: string) => ({ key, label, onRemove: noop });

const FEW = [chip("rarity-TIER_6", "6★"), chip("class-WARRIOR", "Guard"), chip("class-SNIPER", "Sniper")];

const MANY = [
    chip("rarity-TIER_6", "6★"),
    chip("rarity-TIER_5", "5★"),
    chip("class-WARRIOR", "Guard"),
    chip("class-SNIPER", "Sniper"),
    chip("sub-librator", "Liberator Guard"),
    chip("sub-bombarder", "Flinger Sniper"),
    chip("nation-kazimierz", "Kazimierz"),
    chip("fac-penguin", "Penguin Logistics"),
    chip("gen-Female", "Female"),
    chip("race-Kuranta", "Kuranta"),
    chip("artist-Skade", "Skade"),
    chip("va-Ayane Sakura", "Ayane Sakura"),
    chip("notes-yes", "Has notes"),
];

/** A rarity and two classes: the everyday narrowing on /operators. */
export const FewChips = () => (
    <div className="w-full">
        <ActiveFilterChips chips={FEW} onClearAll={noop} />
    </div>
);

/** Every advanced field in play - the row wraps, "Clear all" trails the last chip. */
export const ManyChips = () => (
    <div className="w-full max-w-2xl">
        <ActiveFilterChips chips={MANY} onClearAll={noop} />
    </div>
);

/** A single archetype chip - the shortest row the component renders (it returns
 *  `null` for an empty list, so there is no empty state to show). */
export const SingleChip = () => (
    <div className="w-full">
        <ActiveFilterChips chips={[chip("sub-lord", "Lord Guard")]} onClearAll={noop} />
    </div>
);

/** Where the row lives: between the toolbar and the results summary in
 *  `Operators.tsx`, with the compact pager on the summary's right. */
export const AboveResults = () => (
    <div className="flex w-full flex-col gap-3.5">
        <ActiveFilterChips chips={FEW} onClearAll={noop} />
        <div className="flex flex-wrap items-center justify-between gap-3 font-medium font-sans text-[12.5px] text-muted-foreground leading-none">
            <span>
                Showing <strong className="text-foreground">1</strong> to <strong className="text-foreground">30</strong> of <strong className="text-foreground">74</strong> operators
            </span>
            <div className="ml-auto flex items-center gap-3">
                <span className="hidden font-mono text-[11px] text-muted-foreground uppercase leading-none tracking-[0.08em] md:inline">Hover for preview · Click to open</span>
                <PaginationCompact currentPage={1} totalPages={3} onPageChange={noop} />
            </div>
        </div>
    </div>
);
