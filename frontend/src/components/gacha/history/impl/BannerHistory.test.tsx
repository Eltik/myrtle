import { cleanup, fireEvent, render, screen, within } from "@testing-library/react";
import { afterEach, describe, expect, it } from "vitest";
import { deriveClientGachaRecords, type IGachaItem, type IGachaRecords } from "#/lib/api/gacha";
import { I18nProvider } from "#/lib/i18n/context";
import { messages as constantsMessages } from "../../constants.messages";
import { BannerHistory } from "./BannerHistory";
import { messages } from "./BannerHistory.messages";

const MESSAGES = Object.fromEntries(Object.entries({ ...messages, ...constantsMessages }).map(([key, m]) => [`gacha.${key}`, m.text]));

function pull(poolId: string, at: number, charName: string, star: "3" | "4" | "5" | "6" = "3"): IGachaItem {
    return { charId: `char_${charName}`, charName, star, color: "", poolId, poolName: poolId, typeName: "", at, atStr: "" } as IGachaItem;
}

const PULLS = [pull("LIMITED_EN_1", 3000, "Limited Six", "6"), pull("NORM_EN_1", 1000, "Standard Three"), pull("LINKAGE_EN_1", 2000, "Collab Four", "4"), pull("CLASSIC_EN_1", 4000, "Kernel Five", "5")];

function mount(items: IGachaItem[]) {
    const bucket = (list: IGachaItem[]) => ({ records: list, total: list.length });
    const records = deriveClientGachaRecords({ limited: bucket(items), regular: bucket([]), special: bucket([]) } as unknown as IGachaRecords);
    return render(
        <I18nProvider locale="en" available={[{ code: "en", nativeName: "English" }]} messages={MESSAGES}>
            <BannerHistory records={records} operatorsById={new Map()} bannersById={new Map()} isLoading={false} />
        </I18nProvider>,
    );
}

/** Operator names in table order. */
function rowNames(): string[] {
    return within(screen.getByRole("table"))
        .getAllByRole("row")
        .slice(1)
        .map((row) => row.querySelector("td:nth-child(2) .truncate")?.textContent ?? "");
}

const tab = (label: string) => screen.getByRole("button", { name: new RegExp(`^${label}\\s*\\d`) });

afterEach(cleanup);

describe("BannerHistory All tab", () => {
    it("opens on Limited, as before", () => {
        mount(PULLS);
        expect(rowNames()).toEqual(["Limited Six"]);
    });

    it("merges every group newest first, with the All badge counting them all", () => {
        mount(PULLS);
        const all = tab(messages["history.banners.tab.all"].text);
        expect(all.textContent).toContain("4");
        fireEvent.click(all);
        expect(rowNames()).toEqual(["Kernel Five", "Limited Six", "Collab Four", "Standard Three"]);
    });

    it("applies the rarity filter across the merged list and the All badge follows it", () => {
        mount(PULLS);
        fireEvent.click(tab(messages["history.banners.tab.all"].text));
        // Turn 3* and 4* off.
        fireEvent.click(screen.getByRole("button", { name: /^3 star/ }));
        fireEvent.click(screen.getByRole("button", { name: /^4 star/ }));
        expect(rowNames()).toEqual(["Kernel Five", "Limited Six"]);
        expect(tab(messages["history.banners.tab.all"].text).textContent).toContain("2");
    });

    it("names the banner under each row on All, whose rows mix groups", () => {
        mount(PULLS);
        fireEvent.click(tab(messages["history.banners.tab.all"].text));
        // The inline copy (shown below `sm`) sits beside the Banner column's.
        expect(screen.getAllByText("LINKAGE_EN_1")).toHaveLength(2);
        fireEvent.click(tab(constantsMessages["banner.group.linkage"].text));
        expect(screen.getAllByText("LINKAGE_EN_1")).toHaveLength(1);
    });
});
