/**
 * The dialogs a showcase favourite opens, against a seeded query cache. No
 * profile in the local database has a faction favourite or an owned operator
 * favourite (2026-10-08: one saved layout, three unowned operators), so these
 * paths are proven here rather than in the browser.
 */
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { cleanup, fireEvent, render, screen, within } from "@testing-library/react";
import type React from "react";
import { afterEach, describe, expect, it, vi } from "vitest";
import type { ITierEntityOf } from "#/lib/api/tier-entities";
import type { IRosterEntry } from "#/lib/api/user";
import { I18nProvider } from "#/lib/i18n/context";
import type { IOperatorIndexEntry } from "#/types/operators";
import { messages as kindMessages } from "../../../../../../tier-lists/kinds.messages";
import { messages } from "./FavouriteDialogs.messages";
import type { RosterAccess } from "./favourites";

vi.mock("@tanstack/react-start", () => ({
    createServerFn: () => ({
        inputValidator: () => ({
            handler: (fn: (args: { data: unknown }) => unknown) => (opts: { data: unknown }) => fn({ data: opts.data }),
        }),
    }),
}));
vi.mock("@tanstack/react-router", () => ({
    Link: ({ children, className, params, ...rest }: { children?: React.ReactNode; className?: string; params?: { id: string }; "aria-label"?: string }) => (
        <a href={`/operators/${params?.id ?? ""}`} className={className} aria-label={rest["aria-label"]}>
            {children}
        </a>
    ),
}));
// The roster's dialog reads the full operator table; what matters here is that it is reached, with which row.
vi.mock("../Roster/OperatorDialog", () => ({
    OperatorDialog: ({ entry }: { entry: { name: string; elite: number; level: number } }) => (
        <div role="dialog" data-testid="build">
            {`${entry.name} E${entry.elite} Lv${entry.level}`}
        </div>
    ),
}));

// jsdom has no Web Animations; the dialog panel's scroll area asks for them.
if (!Element.prototype.getAnimations) Element.prototype.getAnimations = () => [];

const { FactionFavouriteDialog, OperatorFavouriteDialog, ShowcasePlayerContext } = await import("./FavouriteDialogs");

const catalog = Object.fromEntries([...Object.entries(messages).map(([k, v]) => [`user.${k}`, v.text]), ...Object.entries(kindMessages).map(([k, v]) => [`tierLists.${k}`, v.text])]);
const t = (key: keyof typeof messages, values: Record<string, string | number> = {}) => messages[key].text.replace(/\{(\w+)\}/g, (_, k) => String(values[k]));

function op(id: string, name: string, rarity: number, nationId: string, groupId: string | null = null, extra: Partial<IOperatorIndexEntry> = {}): IOperatorIndexEntry {
    return { id, name, rarity, nationId, groupId, teamId: null, profession: "WARRIOR", subProfessionId: "", artists: [], voiceActors: [], gender: "", race: "", placeOfBirth: "", isNotObtainable: false, ...extra } as unknown as IOperatorIndexEntry;
}

const INDEX = [op("char_e", "Pramanix", 6, "kjerag", "karlan"), op("char_d", "Silverash", 6, "kjerag", "karlan"), op("char_c", "Courier", 4, "kjerag", "karlan"), op("char_f", "Amiya", 5, "rhodes")];
const UPCOMING = [op("char_new", "Newcomer", 6, "kjerag")];
const ROW = { operator_id: "char_d", elite: 2, level: 90, masteries: [], modules: [] } as unknown as IRosterEntry;

const KJERAG: ITierEntityOf<"faction"> = { key: "faction:kjerag", kind: "faction", id: "kjerag", name: "Kjerag", icon: null, href: null, facets: {}, subOrder: 0, description: null, resolved: true, powerLevel: "nation" } as unknown as ITierEntityOf<"faction">;

function mount(node: React.ReactElement, access: RosterAccess) {
    const client = new QueryClient({ defaultOptions: { queries: { retry: false } } });
    client.setQueryData(["operators", "index"], INDEX);
    client.setQueryData(["upcoming"], UPCOMING);
    client.setQueryData(["operators", "detail", "char_d"], { id: "char_d" });
    return render(
        <QueryClientProvider client={client}>
            <I18nProvider locale="en" available={[]} messages={catalog}>
                <ShowcasePlayerContext.Provider value={{ uid: "1", name: "Eltik", access }}>{node}</ShowcasePlayerContext.Provider>
            </I18nProvider>
        </QueryClientProvider>,
    );
}

afterEach(cleanup);

const silverash = { id: "char_d", name: "Silverash", rarity: 6 };

describe("operator favourite", () => {
    it("opens the player's build when they own it", async () => {
        mount(<OperatorFavouriteDialog operator={silverash} open onOpenChange={() => undefined} />, { state: "ready", roster: [ROW] });
        expect((await screen.findByTestId("build")).textContent).toBe("Silverash E2 Lv90");
    });

    it("says the operator is not in the roster when they do not own it", async () => {
        mount(<OperatorFavouriteDialog operator={silverash} open onOpenChange={() => undefined} />, { state: "ready", roster: [] });
        expect(await screen.findByText(t("profile.showcase.operator.notOwned", { player: "Eltik" }))).toBeTruthy();
        expect(screen.getByRole("link", { name: /Open operator page/ }).getAttribute("href")).toBe("/operators/char_d");
    });

    it("says the roster is private and asks for nothing when the viewer may not read it", async () => {
        mount(<OperatorFavouriteDialog operator={silverash} open onOpenChange={() => undefined} />, { state: "private" });
        expect(await screen.findByText(t("profile.showcase.operator.private", { player: "Eltik" }))).toBeTruthy();
        expect(screen.queryByTestId("build")).toBeNull();
    });
});

describe("faction favourite", () => {
    it("lists every operator of the nation, rarest first, with the count and the owned count", async () => {
        mount(<FactionFavouriteDialog faction={KJERAG} open onOpenChange={() => undefined} />, { state: "ready", roster: [ROW] });
        const grid = await screen.findByRole("list", { name: t("profile.showcase.faction.gridLabel", { name: "Kjerag" }) });
        const names = [...grid.querySelectorAll("a, button")].map((el) => el.getAttribute("aria-label"));
        expect(names).toEqual([t("profile.showcase.faction.cardPage", { name: "Pramanix" }), t("profile.showcase.faction.cardBuild", { name: "Silverash", player: "Eltik" }), t("profile.showcase.faction.cardPage", { name: "Courier" })]);
        expect(screen.getByText(/3 operators/)).toBeTruthy();
        expect(screen.getByText(/1 of 3 owned/)).toBeTruthy();
        // CN-only operators are labelled under Upcoming, not hidden.
        expect(screen.getByText(t("profile.showcase.faction.upcoming", { count: 1 }))).toBeTruthy();
        expect(screen.getByRole("link", { name: t("profile.showcase.faction.upcomingAria", { name: "Newcomer" }) })).toBeTruthy();
    });

    it("narrows to the owned operators, and an owned card opens the build", async () => {
        mount(<FactionFavouriteDialog faction={KJERAG} open onOpenChange={() => undefined} />, { state: "ready", roster: [ROW] });
        fireEvent.click(await screen.findByRole("switch"));
        const grid = screen.getByRole("list", { name: t("profile.showcase.faction.gridLabel", { name: "Kjerag" }) });
        expect(grid.querySelectorAll("a, button")).toHaveLength(1);
        fireEvent.click(within(grid).getByRole("button"));
        expect((await screen.findByTestId("build")).textContent).toBe("Silverash E2 Lv90");
    });

    it("marks nothing owned and offers no toggle when the roster is private", async () => {
        mount(<FactionFavouriteDialog faction={KJERAG} open onOpenChange={() => undefined} />, { state: "private" });
        expect(await screen.findByText(t("profile.showcase.faction.private", { player: "Eltik" }))).toBeTruthy();
        expect(screen.queryByRole("switch")).toBeNull();
        expect(screen.queryByText(/owned/)).toBeNull();
    });
});
