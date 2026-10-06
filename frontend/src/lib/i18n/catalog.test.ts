import { describe, expect, it } from "vitest";

import { pickGamedataServer } from "./catalog";

describe("pickGamedataServer", () => {
    it("trusts the locale row and ignores a pick on a backend without the loaded list", () => {
        expect(pickGamedataServer("jp", "kr", undefined)).toEqual({ server: "jp", localeServer: "jp", picked: false });
    });

    it("follows the locale row when nothing is picked", () => {
        expect(pickGamedataServer("kr", undefined, ["en", "kr"])).toEqual({ server: "kr", localeServer: "kr", picked: false });
    });

    it("lets a loaded pick win over the locale row", () => {
        expect(pickGamedataServer("en", "jp", ["en", "jp", "kr"])).toEqual({ server: "jp", localeServer: "en", picked: true });
    });

    it("ignores a pick the backend has not loaded", () => {
        expect(pickGamedataServer("en", "tw", ["en", "jp"])).toEqual({ server: "en", localeServer: "en", picked: false });
    });

    it("sends a locale whose server is not loaded to the default rather than a 404", () => {
        expect(pickGamedataServer("kr", undefined, ["en", "cn"])).toEqual({ server: "en", localeServer: "en", picked: false });
    });

    it("falls back to en with no locale row and an empty list", () => {
        expect(pickGamedataServer(undefined, "jp", [])).toEqual({ server: "en", localeServer: "en", picked: false });
    });
});
