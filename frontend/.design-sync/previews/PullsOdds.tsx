import { PullsOdds } from "frontend";

// The pull planner's odds card. It takes only a pull budget and the carried pity;
// the banner archetype, the rate-up shares, the per-roll pity schedule and every
// figure on the card are computed by the component from the game's declared rates.

// The budget from the planner's resources (134 pulls) with 23 pulls of pity; the
// default Limited banner resets pity, so the 23 do not carry.
export const LimitedBudget = () => (
    <div className="w-full max-w-3xl p-4">
        <PullsOdds budget={134} pity={23} />
    </div>
);

// Nothing banked: the field falls back to 100 pulls.
export const NoBudget = () => (
    <div className="w-full max-w-3xl p-4">
        <PullsOdds budget={0} pity={0} />
    </div>
);
