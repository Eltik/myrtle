import { describe, expect, it } from "vitest";
import { teamBuffsGate } from "./teamBuffsGate";

const ok = { isPending: false, error: null };
const pending = { isPending: true, error: null };
const failed = { isPending: false, error: new Error("503") };

describe("teamBuffsGate", () => {
    it("never holds requests when no supporter is on", () => {
        expect(teamBuffsGate({ enabled: false, supporters: pending, scoped: false, index: pending })).toEqual({ waiting: false, error: null });
    });

    it("waits for the supporter values, and fails instead of computing without them", () => {
        expect(teamBuffsGate({ enabled: true, supporters: pending, scoped: false, index: ok })).toEqual({ waiting: true, error: null });
        expect(teamBuffsGate({ enabled: true, supporters: failed, scoped: false, index: ok })).toEqual({ waiting: false, error: failed.error });
        expect(teamBuffsGate({ enabled: true, supporters: ok, scoped: false, index: pending })).toEqual({ waiting: false, error: null });
    });

    it("waits for the operator index only when a class-limited supporter is on, and fails if it fails", () => {
        expect(teamBuffsGate({ enabled: true, supporters: ok, scoped: true, index: pending })).toEqual({ waiting: true, error: null });
        expect(teamBuffsGate({ enabled: true, supporters: ok, scoped: true, index: failed })).toEqual({ waiting: false, error: failed.error });
        expect(teamBuffsGate({ enabled: true, supporters: ok, scoped: true, index: ok })).toEqual({ waiting: false, error: null });
    });
});
