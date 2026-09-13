import { TagRow } from "frontend";

const noop = () => {};

// A single-select segmented row of small tags. Three rows ship: Availability
// and Notes on /operators (the `basicLeading` / `basicTrailing` slots of
// `OperatorFilterFields`), Ownership on the profile roster.
const Field = ({ children }: { children: React.ReactNode }) => <div className="w-64">{children}</div>;

const AVAILABILITY = [
    { value: "global", label: "Global" },
    { value: "upcoming", label: "Upcoming (CN)" },
] as const;

const NOTES = [
    { value: "any", label: "Any" },
    { value: "yes", label: "Has notes" },
    { value: "no", label: "No notes" },
] as const;

const OWNERSHIP = [
    { value: "owned", label: "Owned" },
    { value: "unowned", label: "Unowned" },
    { value: "all", label: "All" },
] as const;

/** Global is the default on /operators; "Upcoming (CN)" swaps the list for the CN-only roster. */
export const Availability = () => (
    <Field>
        <TagRow label="Availability" options={AVAILABILITY} value="global" onChange={noop} />
    </Field>
);

/** The Notes row with the non-default "Has notes" pressed. */
export const Notes = () => (
    <Field>
        <TagRow label="Notes" options={NOTES} value="yes" onChange={noop} />
    </Field>
);

/** The profile roster's leading row - "Owned" is its default. */
export const Ownership = () => (
    <Field>
        <TagRow label="Ownership" options={OWNERSHIP} value="owned" onChange={noop} />
    </Field>
);

/** Both /operators rows as they stack inside the Basic section. */
export const Stacked = () => (
    <Field>
        <div className="flex flex-col gap-4">
            <TagRow label="Availability" options={AVAILABILITY} value="upcoming" onChange={noop} />
            <TagRow label="Notes" options={NOTES} value="any" onChange={noop} />
        </div>
    </Field>
);
