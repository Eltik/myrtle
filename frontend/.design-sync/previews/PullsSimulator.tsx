import { PullsSimulator } from "frontend";

// The pull simulator card: roll a banner one pull or ten at a time, and the
// sampled distribution of pulls needed for the target copies. Inputs are the
// planner's budget (134 pulls) and carried pity (23); everything shown is the
// component's own simulation.
export const LimitedBudget = () => (
    <div className="w-full max-w-3xl p-4">
        <PullsSimulator budget={134} pity={23} />
    </div>
);

export const NoBudget = () => (
    <div className="w-full max-w-3xl p-4">
        <PullsSimulator budget={0} pity={0} />
    </div>
);
