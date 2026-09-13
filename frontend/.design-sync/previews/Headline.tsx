import { Headline } from "frontend";

// The base optimizer's headline figures. Each Headline is one kicker + a
// monospaced value + an optional hint; BasePanel lines four of them up inside
// the bordered strip above the RIIC board, next to the Optimize button.
//
// The numbers are a 2-4-3 base (2 trading posts, 4 factories, 3 power plants)
// at L3: efficiency is the SUM of every production room's speed bonus, power
// is 3 × 270 generated against 750 drawn.

/** The canonical row - what BasePanel renders once the layout has been scored. */
export const HeadlineRow = () => (
    <div className="flex flex-wrap items-end justify-between gap-3 rounded-xl border border-border bg-card px-4 py-3">
        <div className="flex flex-wrap gap-6">
            <Headline hint="Production rooms" label="Efficiency" value="412%" />
            <Headline label="LMD / day" value="33,480" />
            <Headline label="EXP / day" value="21,600" />
            <Headline hint="810 generated · 750 drawn" label="Power" value="+60" />
        </div>
    </div>
);

/** One figure with its hint line under the value. */
export const WithHint = () => (
    <div className="inline-flex rounded-xl border border-border bg-card px-4 py-3">
        <Headline hint="810 generated · 750 drawn" label="Power" value="+60" />
    </div>
);

/** Label and value only - the hint is optional and the layout does not reserve space for it. */
export const ValueOnly = () => (
    <div className="inline-flex rounded-xl border border-border bg-card px-4 py-3">
        <Headline label="LMD / day" value="33,480" />
    </div>
);

/** Before the evaluation resolves every value is a dash; the hints that depend on data drop out. */
export const Unscored = () => (
    <div className="flex flex-wrap items-end justify-between gap-3 rounded-xl border border-border bg-card px-4 py-3">
        <div className="flex flex-wrap gap-6">
            <Headline hint="Production rooms" label="Efficiency" value="-" />
            <Headline label="LMD / day" value="-" />
            <Headline label="EXP / day" value="-" />
            <Headline label="Power" value="-" />
        </div>
    </div>
);

/** A power deficit reads with its sign - the same tabular figure, so the row does not shift. */
export const PowerDeficit = () => (
    <div className="inline-flex rounded-xl border border-border bg-card px-4 py-3">
        <Headline hint="540 generated · 690 drawn" label="Power" value="-150" />
    </div>
);
