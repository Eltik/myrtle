import { BriefingHero } from "frontend";

// The randomizer page header: title, blurb, a live counter strip, and the roll
// controls. `hasResult` is what adds the Reset button; `canRoll` disables the
// primary action; a zero count turns its counter amber.

const noop = () => {};

export const ReadyToRoll = () => <BriefingHero canRoll hasResult={false} onOpenSettings={noop} onReset={noop} onRollAll={noop} operatorsAvailable={287} operatorsRoster={438} stagesAvailable={612} />;

export const AfterARoll = () => <BriefingHero canRoll hasResult onOpenSettings={noop} onReset={noop} onRollAll={noop} operatorsAvailable={287} operatorsRoster={438} stagesAvailable={612} />;

// Roster pruned to owned E2 operators only, and the stage pool cut to cleared
// stages — a plausible signed-in configuration.
export const NarrowPool = () => <BriefingHero canRoll hasResult onOpenSettings={noop} onReset={noop} onRollAll={noop} operatorsAvailable={41} operatorsRoster={192} stagesAvailable={58} />;

// Every class toggled off: nothing is drawable, both counters go amber and
// "Roll squad" is disabled.
export const NothingDrawable = () => <BriefingHero canRoll={false} hasResult={false} onOpenSettings={noop} onReset={noop} onRollAll={noop} operatorsAvailable={0} operatorsRoster={438} stagesAvailable={0} />;
