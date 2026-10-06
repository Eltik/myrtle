import { describe, expect, it } from "vitest";
import { parseBirthday } from "./calculate";

describe("parseBirthday", () => {
    it("parses the English forms", () => {
        expect(parseBirthday("Jan 7")).toEqual({ month: 1, day: 7 });
        expect(parseBirthday("Mar. 7")).toEqual({ month: 3, day: 7 });
        expect(parseBirthday("December 25")).toEqual({ month: 12, day: 25 });
    });

    it("parses the Korean form", () => {
        expect(parseBirthday("1월 7일")).toEqual({ month: 1, day: 7 });
        expect(parseBirthday("12월 31일")).toEqual({ month: 12, day: 31 });
    });

    it("parses the Japanese and Chinese form", () => {
        expect(parseBirthday("1月7日")).toEqual({ month: 1, day: 7 });
        expect(parseBirthday("11月30日")).toEqual({ month: 11, day: 30 });
    });

    it("rejects garbage and out-of-range dates", () => {
        expect(parseBirthday("")).toBeNull();
        expect(parseBirthday("Unknown")).toBeNull();
        expect(parseBirthday("13월 1일")).toBeNull();
        expect(parseBirthday("1月32日")).toBeNull();
        expect(parseBirthday("0月5日")).toBeNull();
        expect(parseBirthday("Foo 7")).toBeNull();
    });
});
