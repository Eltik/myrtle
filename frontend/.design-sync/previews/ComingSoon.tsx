import { ComingSoon } from "frontend";

// ComingSoon is the placeholder `OptimizerTab` renders for a registry entry
// (`optimizers.tsx`) that has a label and blurb but no `Component` yet. Since
// the i18n migration an entry carries message KEYS (`labelKey`, `blurbKey`,
// declared in `optimizers.messages.ts`), so only the registry's real entries
// resolve to text; the Account Optimizer is the one without a Component.

const ACCOUNT_OPTIMIZER = {
    id: "account",
    labelKey: "profile.optimizer.account.label",
    blurbKey: "profile.optimizer.account.blurb",
} as const;

/** The registry's own unbuilt entry. */
export const AccountOptimizer = () => <ComingSoon def={ACCOUNT_OPTIMIZER} />;

/** As `OptimizerTab` frames it: the tab strip above, the placeholder filling the panel. */
export const InTabPanel = () => (
    <div className="flex flex-col gap-3">
        <div className="flex items-center gap-1 border-border border-b">
            <span className="px-3 py-2 text-[13px] text-muted-foreground">Base Optimizer</span>
            <span className="-mb-px border-primary border-b-2 px-3 py-2 font-medium text-[13px] text-foreground">Account Optimizer</span>
        </div>
        <ComingSoon def={ACCOUNT_OPTIMIZER} />
    </div>
);
