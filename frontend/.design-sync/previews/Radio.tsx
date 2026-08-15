import { Radio, RadioGroup } from "frontend";

/** Checked vs unchecked inside one group. */
export const Checked = () => (
    <RadioGroup className="flex w-72 flex-col gap-3" defaultValue="e2">
        {[
            { value: "e0", label: "Elite 0" },
            { value: "e1", label: "Elite 1" },
            { value: "e2", label: "Elite 2 — max promotion" },
        ].map((elite) => (
            // biome-ignore lint/a11y/noLabelWithoutControl: Radio renders the input
            <label className="flex cursor-pointer items-center gap-2.5" key={elite.value}>
                <Radio value={elite.value} />
                <span className="text-foreground text-sm leading-none">{elite.label}</span>
            </label>
        ))}
    </RadioGroup>
);

/** Per-item `disabled` — masteries are unavailable below Elite 2. */
export const ItemDisabled = () => (
    <RadioGroup className="flex w-72 flex-col gap-3" defaultValue="m0">
        {[
            { value: "m0", label: "No mastery", disabled: false },
            { value: "m1", label: "Mastery 1", disabled: false },
            { value: "m3", label: "Mastery 3 — needs Elite 2", disabled: true },
        ].map((mastery) => (
            // biome-ignore lint/a11y/noLabelWithoutControl: Radio renders the input
            <label className="flex items-center gap-2.5" key={mastery.value}>
                <Radio disabled={mastery.disabled} value={mastery.value} />
                <span className="text-foreground text-sm leading-none">{mastery.label}</span>
            </label>
        ))}
    </RadioGroup>
);

/** `aria-invalid` — the destructive treatment when a required choice is missing. */
export const Invalid = () => (
    <div className="flex w-72 flex-col gap-2">
        <span className="font-medium text-foreground text-sm">Recruitment slot</span>
        <RadioGroup className="flex flex-col gap-3">
            {[
                { value: "slot1", label: "Slot 1 — 9:00 tag reset" },
                { value: "slot2", label: "Slot 2 — 4:00 expedited" },
            ].map((slot) => (
                // biome-ignore lint/a11y/noLabelWithoutControl: Radio renders the input
                <label className="flex cursor-pointer items-center gap-2.5" key={slot.value}>
                    <Radio aria-invalid="true" value={slot.value} />
                    <span className="text-foreground text-sm leading-none">{slot.label}</span>
                </label>
            ))}
        </RadioGroup>
        <span className="text-destructive text-xs">Pick a slot before running the calculator.</span>
    </div>
);

/** Read-only — a shared tier list shows the author's choice without allowing edits. */
export const ReadOnly = () => (
    <RadioGroup className="flex w-72 flex-col gap-3" defaultValue="global" readOnly>
        {[
            { value: "global", label: "Global meta" },
            { value: "cn", label: "CN meta" },
        ].map((scope) => (
            // biome-ignore lint/a11y/noLabelWithoutControl: Radio renders the input
            <label className="flex items-center gap-2.5" key={scope.value}>
                <Radio value={scope.value} />
                <span className="text-foreground text-sm leading-none">{scope.label}</span>
            </label>
        ))}
    </RadioGroup>
);
