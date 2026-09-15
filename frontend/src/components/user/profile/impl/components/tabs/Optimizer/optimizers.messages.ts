import { defineMessages, type MessageMap } from "#/lib/i18n/messages";

/**
 * The optimizer registry is a plain table, so each entry carries message KEYS
 * and the tab strip resolves them with `t()`.
 */
export const namespace = "user";

export const messages = {
    "profile.optimizer.base.label": {
        text: "Base Optimizer",
        description: "Name of the optimizer that staffs the player's base. Also the tab's label.",
    },
    "profile.optimizer.base.blurb": {
        text: "Solve the best peak staffing for the RIIC, then plans a shift rotation.",
        description: "One-line blurb under the Base Optimizer tab. 'RIIC' is the game's own name for the base system.",
    },
    "profile.optimizer.account.label": {
        text: "Account Optimizer",
        description: "Name of the optimizer that ranks account-wide investments. Also the tab's label.",
    },
    "profile.optimizer.account.blurb": {
        text: "Where to spend next across the account - promotions, masteries and modules, ranked by what each actually returns.",
        description: "One-line blurb under the Account Optimizer tab. Promotion, mastery and module are the game's progression systems.",
    },
} satisfies MessageMap;

// `dynamic`: these keys are stored on the OPTIMIZERS entries and resolved as
// `t(optimizer.labelKey)`, so the extractor has no literal call site to match
// them against.
export const { keys } = defineMessages({ namespace, messages, dynamic: true });
