import { CurrencyLabel, PullsNumber, ReleaseInfoHint } from "frontend";
import { useState } from "react";

function Field({ id, label, initial, step, className, flash }: { id: string; label: React.ReactNode; initial: number; step?: number; className?: string; flash?: boolean }) {
    const [v, setV] = useState(initial);
    return <PullsNumber id={id} label={label} value={v} onChange={setV} step={step} className={className} flash={flash} />;
}

// One numeric field from the pull planner's resources card.
export const Default = () => (
    <div className="p-4">
        <Field id="pn-orundum" label={<CurrencyLabel name="orundum">Orundum</CurrencyLabel>} initial={45600} step={600} />
    </div>
);

// The resources row: 45,600 Orundum, 38 permits, 2 ten-roll permits, 120
// Originite Prime and a pity count with its hint.
export const ResourcesRow = () => (
    <div className="flex flex-wrap items-end gap-x-4 gap-y-3 p-4">
        <Field id="pn-r-orundum" label={<CurrencyLabel name="orundum">Orundum</CurrencyLabel>} initial={45600} step={600} />
        <Field id="pn-r-permits" label={<CurrencyLabel name="permit">Permits</CurrencyLabel>} initial={38} className="w-20" />
        <Field id="pn-r-ten" label={<CurrencyLabel name="tenPermit">Ten-roll permits</CurrencyLabel>} initial={2} className="w-20" />
        <Field id="pn-r-op" label={<CurrencyLabel name="originite">Originite Prime</CurrencyLabel>} initial={120} className="w-24" />
        <Field
            id="pn-r-pity"
            label={
                <span className="inline-flex items-center gap-1">
                    Pulls since last 6★
                    <ReleaseInfoHint label="About the pity counter">Counts on Standard and Kernel banners, which share a counter. Limited and collab banners always start you at zero.</ReleaseInfoHint>
                </span>
            }
            initial={23}
            className="w-20"
        />
    </div>
);

// Flashed: just filled from the account's data, ringed for a moment.
export const Flashed = () => (
    <div className="flex flex-wrap items-end gap-4 p-4">
        <Field id="pn-f-orundum" label={<CurrencyLabel name="orundum">Orundum</CurrencyLabel>} initial={45600} step={600} flash />
        <Field id="pn-f-permits" label={<CurrencyLabel name="permit">Permits</CurrencyLabel>} initial={38} className="w-20" flash />
    </div>
);
