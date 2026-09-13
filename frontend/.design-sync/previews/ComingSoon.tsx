import { ComingSoon } from "frontend";

// ComingSoon is the placeholder `OptimizerTab` renders for a registry entry
// (`optimizers.tsx`) that has a label and blurb but no `Component` yet. The
// first def is the real one from the registry; the second is a plausible next
// entry, to show a longer blurb wrapping inside the 28rem cap.

const ACCOUNT_OPTIMIZER = {
    id: "account",
    label: "Account Optimizer",
    blurb: "Where to spend next across the account - promotions, masteries and modules, ranked by what each actually returns.",
};

const FARMING_OPTIMIZER = {
    id: "farming",
    label: "Farming Optimizer",
    blurb: "Which stages to run for the materials your queued promotions and masteries still need, ranked by sanity per drop against the current event's efficiency and your Originite Prime budget.",
};

/** The registry's own unbuilt entry. */
export const AccountOptimizer = () => <ComingSoon def={ACCOUNT_OPTIMIZER} />;

/** As `OptimizerTab` frames it: the blurb above, the placeholder below. */
export const InTabPanel = () => (
    <div className="flex flex-col gap-3">
        <p className="text-[12.5px] text-muted-foreground">{FARMING_OPTIMIZER.blurb}</p>
        <ComingSoon def={FARMING_OPTIMIZER} />
    </div>
);
