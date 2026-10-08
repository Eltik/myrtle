import { FieldGroup, RandomizerSwitchRow, ToggleGroup, ToggleGroupItem } from "frontend";
import { useState } from "react";

// A labelled block of controls inside a randomizer settings tab: a mono,
// letter-spaced uppercase eyebrow over whatever controls the block holds.
// The operator panel stacks four of them (Class, Rarity, Squad size, Rules).

// The operator panel dims unpressed chips with exactly this class string; it
// compiles because OperatorFiltersPanel uses it verbatim.
const DIM_UNPRESSED = "[&:not([data-pressed])>span]:opacity-35 [&:not([data-pressed])]:bg-input/64 [&:not([data-pressed])]:before:shadow-none! dark:[&:not([data-pressed])]:bg-input";

/** Rules: a stack of switch rows under the eyebrow. */
export const WithSwitchRows = () => {
    const [dupes, setDupes] = useState(false);
    const [hide, setHide] = useState(true);
    return (
        <div className="max-w-sm">
            <FieldGroup label="Rules">
                <RandomizerSwitchRow label="Allow duplicates" description="Same operator can appear twice in a squad." checked={dupes} onChange={setDupes} />
                <RandomizerSwitchRow label="Hide unplayable operators" description="Exclude tokens, support-only, and reserve operators." checked={hide} onChange={setHide} />
            </FieldGroup>
        </div>
    );
};

/** Rarity: an outline toggle group of star chips, 4★ to 6★ allowed. */
export const WithToggleGroup = () => {
    const [value, setValue] = useState<string[]>(["4", "5", "6"]);
    return (
        <div className="max-w-sm">
            <FieldGroup label="Rarity">
                <ToggleGroup aria-label="Allowed rarities" multiple value={value} onValueChange={(next: string[]) => setValue(next)} variant="outline">
                    {["1", "2", "3", "4", "5", "6"].map((r) => (
                        <ToggleGroupItem key={r} value={r} aria-label={`${r} star`} className={DIM_UNPRESSED}>
                            <span>{r}★</span>
                        </ToggleGroupItem>
                    ))}
                </ToggleGroup>
            </FieldGroup>
        </div>
    );
};

/** Two groups stacked the way the panel spaces them. */
export const Stacked = () => {
    const [value, setValue] = useState<string[]>(["5", "6"]);
    const [dupes, setDupes] = useState(true);
    return (
        <div className="flex max-w-sm flex-col gap-6">
            <FieldGroup label="Rarity">
                <ToggleGroup aria-label="Allowed rarities" multiple value={value} onValueChange={(next: string[]) => setValue(next)} variant="outline">
                    {["1", "2", "3", "4", "5", "6"].map((r) => (
                        <ToggleGroupItem key={r} value={r} aria-label={`${r} star`} className={DIM_UNPRESSED}>
                            <span>{r}★</span>
                        </ToggleGroupItem>
                    ))}
                </ToggleGroup>
            </FieldGroup>
            <FieldGroup label="Rules">
                <RandomizerSwitchRow label="Allow duplicates" description="Same operator can appear twice in a squad." checked={dupes} onChange={setDupes} />
            </FieldGroup>
        </div>
    );
};
