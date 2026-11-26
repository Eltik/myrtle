import { describe, expect, it } from "vitest";
import { ACQUISITION_FILTERS, matchesAcquisition, needsRecruitPool, sourceOfChannel } from "./acquisition";

describe("sourceOfChannel", () => {
    it("reads every non-headhunting channel as welfare", () => {
        expect(sourceOfChannel("headhunting")).toBe("headhunting");
        for (const channel of ["event", "voucher", "integratedStrategies", "creditStore", "giftPack", "other"] as const) expect(sourceOfChannel(channel)).toBe("welfare");
    });

    it("claims neither side for an operator with no approach", () => {
        expect(sourceOfChannel(null)).toBeNull();
    });
});

describe("matchesAcquisition", () => {
    // Channels and pool membership as the EN tree has them on 2026-10-28.
    const texas = { id: "char_102_texas", obtainChannel: "headhunting" as const };
    const yato = { id: "char_502_nblade", obtainChannel: "headhunting" as const };
    const exusiai = { id: "char_103_angel", obtainChannel: "headhunting" as const };
    const viy = { id: "char_4226_veen", obtainChannel: "event" as const };
    const highmore = { id: "char_4066_highmo", obtainChannel: "integratedStrategies" as const };
    const courier = { id: "char_198_blackd", obtainChannel: "creditStore" as const };
    const amiya = { id: "char_002_amiya", obtainChannel: "other" as const };
    const reserve = { id: "char_504_rguard", obtainChannel: null };
    const cached = { id: "char_502_nblade" };
    const recruitable = new Set([texas.id, yato.id]);

    const pass = (filter: (typeof ACQUISITION_FILTERS)[number]) => [texas, yato, exusiai, viy, highmore, courier, amiya, reserve].filter((op) => matchesAcquisition(op, filter, recruitable)).map((op) => op.id);

    it("keeps everything on any", () => {
        expect(pass("any")).toHaveLength(8);
    });

    it("welfare is the channel alone", () => {
        expect(pass("welfare")).toEqual([viy.id, highmore.id, courier.id, amiya.id]);
    });

    it("recruitment is the pool alone, whatever the channel says", () => {
        expect(pass("recruitment")).toEqual([texas.id, yato.id]);
    });

    it("gachaless is welfare plus the recruitment pool, and never a headhunting-only operator", () => {
        expect(pass("gachaless")).toEqual([texas.id, yato.id, viy.id, highmore.id, courier.id, amiya.id]);
    });

    it("an operator with no channel field matches only through the pool", () => {
        expect(matchesAcquisition(cached, "welfare", recruitable)).toBe(false);
        expect(matchesAcquisition(cached, "gachaless", recruitable)).toBe(true);
        expect(matchesAcquisition(cached, "gachaless", new Set())).toBe(false);
    });

    it("loads the pool only for the filters that read it", () => {
        expect(ACQUISITION_FILTERS.filter(needsRecruitPool)).toEqual(["gachaless", "recruitment"]);
    });
});
