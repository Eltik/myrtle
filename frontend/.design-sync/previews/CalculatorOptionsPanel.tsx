import { CalculatorOptionsPanel } from "frontend";

// The "Options" card of the recruitment calculator: three inclusion switches and
// the operator sort order. The Select trigger is `w-fit`, so it hugs the label.

const noop = () => {};

const options = (over: Record<string, unknown> = {}) => ({
    includeRobots: true,
    includeTwoStars: true,
    includeThreeStars: true,
    operatorSortMode: "rarity-desc",
    ...over,
});

export const Defaults = () => (
    <div className="max-w-sm">
        <CalculatorOptionsPanel onChangeIncludeRobots={noop} onChangeIncludeThreeStars={noop} onChangeIncludeTwoStars={noop} onChangeSortMode={noop} options={options()} />
    </div>
);

// Chasing 4★+ only: the low-rarity fillers are switched off.
export const HighRarityOnly = () => (
    <div className="max-w-sm">
        <CalculatorOptionsPanel onChangeIncludeRobots={noop} onChangeIncludeThreeStars={noop} onChangeIncludeTwoStars={noop} onChangeSortMode={noop} options={options({ includeRobots: false, includeTwoStars: false, includeThreeStars: false })} />
    </div>
);

// Robot-hunting: 2★/3★ stay in, and results lead with the most common pulls.
export const CommonFirst = () => (
    <div className="max-w-sm">
        <CalculatorOptionsPanel onChangeIncludeRobots={noop} onChangeIncludeThreeStars={noop} onChangeIncludeTwoStars={noop} onChangeSortMode={noop} options={options({ operatorSortMode: "common-first" })} />
    </div>
);
