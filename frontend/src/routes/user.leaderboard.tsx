import { createFileRoute, stripSearchParams } from "@tanstack/react-router";
import { INTERVALS, type LeaderboardInterval, type LeaderboardScope, SERVERS, type ServerCode } from "#/components/user/leaderboard/impl/constants";
import { Leaderboard } from "#/components/user/leaderboard/Leaderboard";
import { defaultOgURL } from "#/lib/og";
import { seo } from "#/lib/seo";

interface ILeaderboardSearch {
    scope: LeaderboardScope;
    server: ServerCode | "All";
    interval: LeaderboardInterval;
    movement: boolean;
    q: string;
    page: number;
}

const LEADERBOARD_DEFAULTS: ILeaderboardSearch = { scope: "global", server: "All", interval: "1 day", movement: false, q: "", page: 1 };

const VALID_SERVERS = new Set<string>(["All", ...SERVERS]);
const VALID_INTERVALS = new Set<string>(INTERVALS.map((i) => i.value));

export const Route = createFileRoute("/user/leaderboard")({
    component: RouteComponent,
    validateSearch: (search: Record<string, unknown>): ILeaderboardSearch => {
        const scopeRaw = typeof search.scope === "string" ? search.scope : "";
        const scope: LeaderboardScope = scopeRaw === "friends" ? "friends" : "global";
        const serverRaw = typeof search.server === "string" ? search.server.toUpperCase() : "All";
        const server = (VALID_SERVERS.has(serverRaw) ? serverRaw : "All") as ServerCode | "All";
        const intervalRaw = typeof search.interval === "string" ? search.interval : "1 day";
        const interval = (VALID_INTERVALS.has(intervalRaw) ? intervalRaw : "1 day") as LeaderboardInterval;
        const movement = search.movement === true || search.movement === "true";
        const rawPage = typeof search.page === "number" ? search.page : typeof search.page === "string" ? Number(search.page) : 1;
        const page = Number.isFinite(rawPage) && rawPage >= 1 ? Math.floor(rawPage) : 1;
        const q = typeof search.q === "string" ? search.q : "";
        return { scope, server, interval, movement, q, page };
    },
    search: { middlewares: [stripSearchParams(LEADERBOARD_DEFAULTS)] },
    head: () => {
        const { meta, links } = seo({
            title: "Leaderboard",
            description: "Top Doctors ranked by score across servers.",
            path: "/user/leaderboard",
            image: defaultOgURL("user-leaderboard"),
        });
        return {
            meta: [{ charSet: "utf-8" }, { name: "viewport", content: "width=device-width, initial-scale=1" }, ...meta],
            links,
        };
    },
});

function RouteComponent() {
    return <Leaderboard />;
}
