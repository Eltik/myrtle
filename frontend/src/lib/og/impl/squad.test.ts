import { describe, expect, it } from "vitest";
import { ogHashers } from "./hashers";
import { buildSquadOgData, parseSquadOgId, SQUAD_OG_ID_MAX, squadOgId } from "./squad";

const S = "char_332_archet.2.003_archet|||||char_222_bpipe.2.-||||||";

describe("squad card id", () => {
    it("round-trips a share link, keeping seats, support, name and stage", () => {
        const id = squadOgId({ s: S, sup: "char_479_sleach.2.-", n: "Roof Duty", st: "main_04-04" });
        if (!id) throw new Error("expected an id");
        expect(id).toMatch(/^[A-Za-z0-9_-]+$/);
        const parsed = parseSquadOgId(id);
        expect(parsed?.server).toBe("en");
        expect(parsed?.draft.members[0]?.char_id).toBe("char_332_archet");
        expect(parsed?.draft.members[0]?.module_id).toBe("uniequip_003_archet");
        expect(parsed?.draft.members[5]?.char_id).toBe("char_222_bpipe");
        expect(parsed?.draft.support?.char_id).toBe("char_479_sleach");
        expect(parsed?.draft.name).toBe("Roof Duty");
        expect(parsed?.draft.stageId).toBe("main_04-04");
    });

    it("is canonical: links that decode to one squad share one id", () => {
        // A trailing seat list the decoder pads, and a junk seat it drops, are the same squad.
        expect(squadOgId({ s: "char_332_archet.2.003_archet" })).toBe(squadOgId({ s: "char_332_archet.2.003_archet|||||||||||" }));
        expect(squadOgId({ s: "char_332_archet.2.003_archet|nonsense" })).toBe(squadOgId({ s: "char_332_archet.2.003_archet" }));
    });

    it("carries the server off the default only", () => {
        const id = squadOgId({ s: S }, "cn");
        expect(id?.startsWith("cn:")).toBe(true);
        expect(parseSquadOgId(id ?? "")?.server).toBe("cn");
    });

    it("is null for a link with no seat, and parse rejects junk", () => {
        expect(squadOgId({})).toBeNull();
        expect(squadOgId({ s: "|||" })).toBeNull();
        expect(squadOgId({ s: "nonsense|also.bad" })).toBeNull();
        expect(parseSquadOgId("!!notbase64")).toBeNull();
        expect(parseSquadOgId("")).toBeNull();
    });

    it("caps the id length: drops builds first, then gives up", () => {
        const build = "char_002_amiya.1.002_amiya.2.80.5.7.3.3";
        const full = squadOgId({ s: Array(12).fill(build).join("|"), sup: build, n: "x".repeat(40) });
        expect(full).not.toBeNull();
        expect((full ?? "").length).toBeLessThanOrEqual(SQUAD_OG_ID_MAX);
        expect(parseSquadOgId("a".repeat(SQUAD_OG_ID_MAX + 1))).toBeNull();
    });

    it("hashes the squad: a changed seat changes the hash", () => {
        const a = parseSquadOgId(squadOgId({ s: S }) ?? "");
        const b = parseSquadOgId(squadOgId({ s: S.replace("2.003_archet", "1.003_archet") }) ?? "");
        if (!a || !b) throw new Error("expected squads");
        expect(ogHashers.squad.hash(buildSquadOgData(a.draft, "en"))).not.toBe(ogHashers.squad.hash(buildSquadOgData(b.draft, "en")));
        expect(ogHashers.squad.hash(buildSquadOgData(a.draft, "en"))).toBe(ogHashers.squad.hash(buildSquadOgData(a.draft, "en")));
    });
});
