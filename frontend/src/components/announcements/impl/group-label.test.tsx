import { renderHook } from "@testing-library/react";
import { describe, expect, it } from "vitest";
import { useGroupLabel } from "./group-label";

describe("useGroupLabel", () => {
    it("names the two known groups and title-cases any other", () => {
        const { result } = renderHook(() => useGroupLabel());
        const label = result.current;
        expect(["ACTIVITY", "SYSTEM", "EVENT", "mAINTENANCE", "x", ""].map(label)).toEqual(["Events", "Notices", "Event", "Maintenance", "X", ""]);
    });
});
