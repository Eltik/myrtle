import { describe, expect, it } from "vitest";

import { metOperatorIds, shouldShowCompletedNotice } from "./completedPlans";

describe("metOperatorIds", () => {
    it("keeps the met plans, sorted", () => {
        const plans = [
            { operator_id: "char_c", met: true },
            { operator_id: "char_a", met: false },
            { operator_id: "char_b", met: true },
        ];
        expect(metOperatorIds(plans)).toEqual(["char_b", "char_c"]);
    });
});

describe("shouldShowCompletedNotice", () => {
    it("stays hidden when no plan is met", () => {
        expect(shouldShowCompletedNotice([], [])).toBe(false);
        expect(shouldShowCompletedNotice([], ["char_a"])).toBe(false);
    });

    it("shows met plans nobody has dismissed", () => {
        expect(shouldShowCompletedNotice(["char_a"], [])).toBe(true);
    });

    it("stays hidden for the set that was dismissed", () => {
        expect(shouldShowCompletedNotice(["char_a", "char_b"], ["char_a", "char_b"])).toBe(false);
    });

    it("comes back when a sync completes another plan", () => {
        expect(shouldShowCompletedNotice(["char_a", "char_b", "char_c"], ["char_a", "char_b"])).toBe(true);
    });

    it("stays hidden when a dismissed plan is deleted elsewhere", () => {
        expect(shouldShowCompletedNotice(["char_b"], ["char_a", "char_b"])).toBe(false);
    });
});
