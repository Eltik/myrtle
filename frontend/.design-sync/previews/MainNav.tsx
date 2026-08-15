import { MainNav } from "frontend";
import { type ReactNode, useEffect, useRef } from "react";

// The desktop navigation row in the site header. The nav is `hidden lg:flex` and
// the capture viewport is 900px wide, so every story passes `className="flex"` -
// tailwind-merge drops the `hidden` and the row renders as it does at >=1024px.
//
// `items` is ported verbatim from the `navItems` memo in
// `src/components/header/Header.tsx` (signed-out branch: no "My Profile" row),
// with the tool sections expanded from `getToolsByCategory()`.

const COLLECTION = [
    { href: "/operators", label: "Operators", desc: "Every operator released in Arknights.", icon: "shield" as const },
    { href: "/enemies", label: "Enemies", desc: "Every enemy catalogued, with stats and traits.", icon: "crosshair" as const },
    { href: "/stages", label: "Stages", desc: "Every stage, mapped with an enemy-pathing simulator.", icon: "map" as const },
];

const TOOL_SECTIONS = [
    {
        title: "Calculators",
        items: [
            { href: "/tools/recruitment", label: "Recruitment calculator", desc: "Guaranteed tag combos · 1h parity", icon: "calc" as const },
            { href: "/tools/planner", label: "Operator planner", desc: "Plan promotions, skills, and modules", icon: "list-todo" as const },
            { href: "/tools/dps", label: "DPS charts", desc: "Interactive damage curves per skill", icon: "chart" as const },
            { href: "/tools/hps", label: "HPS charts", desc: "Interactive healing curves per skill", icon: "heart" as const },
        ],
    },
    {
        title: "For Fun",
        items: [
            { href: "/tools/randomizer", label: "Randomizer", desc: "Pick a squad, break the meta", icon: "dice" as const },
            { href: "/tools/birthdays", label: "Birthdays", desc: "View and track operator birthdays", icon: "cake" as const },
        ],
    },
];

const GACHA = [
    { href: "/gacha/community", label: "Community", desc: "Pull rates, top operators, and timing across opted-in doctors", icon: "users" as const },
    { href: "/gacha/history", label: "History", desc: "Your synced pulls, rarity splits, and pity counters", icon: "history" as const },
];

const PLAYERS = [
    { href: "/user/search", label: "Search", desc: "Find profiles by nickname or UID", icon: "search" as const },
    { href: "/user/leaderboard", label: "Leaderboard", desc: "Top Doctors ranked by score", icon: "trophy" as const },
];

const NAV_ITEMS = [
    { href: "/", label: "Home" },
    { href: "/operators", label: "Collection", items: COLLECTION },
    { href: "/tools", label: "Tools", items: TOOL_SECTIONS.flatMap((s) => s.items), sections: TOOL_SECTIONS },
    { href: "/gacha", label: "Gacha", items: GACHA },
    { href: "/tier-lists", label: "Tier Lists" },
    { href: "/user", label: "Players", items: PLAYERS },
];

const noop = () => {};

// Each dropdown owns its open state inside `HoverDropdown`, and hover cannot be
// simulated in a still. Clicking the trigger toggles the same state, so these
// stories click it on mount - deferred two frames, because Base UI wires the
// trigger only after the first paint.
const AutoOpen = ({ label, children }: { label: string; children: ReactNode }) => {
    const ref = useRef<HTMLDivElement>(null);
    useEffect(() => {
        let inner = 0;
        const outer = requestAnimationFrame(() => {
            inner = requestAnimationFrame(() => {
                const trigger = Array.from(ref.current?.querySelectorAll<HTMLButtonElement>("button[aria-expanded]") ?? []).find((b) => b.textContent?.trim().startsWith(label));
                trigger?.click();
                // Base UI focuses the first menu row on open; its brand-red ring reads
                // as a selected/error state in a still. Drop it once the popup settles.
                requestAnimationFrame(() => requestAnimationFrame(() => (document.activeElement as HTMLElement | null)?.blur()));
            });
        });
        return () => {
            cancelAnimationFrame(outer);
            cancelAnimationFrame(inner);
        };
    }, [label]);
    return (
        <div className="min-h-[520px] w-full" ref={ref}>
            {children}
        </div>
    );
};

export const SiteNavigation = () => <MainNav className="flex" items={NAV_ITEMS} onOpenCommand={noop} />;

export const ToolsMenuOpen = () => (
    <AutoOpen label="Tools">
        <MainNav className="flex" items={NAV_ITEMS} onOpenCommand={noop} />
    </AutoOpen>
);

export const CollectionMenuOpen = () => (
    <AutoOpen label="Collection">
        <MainNav className="flex" items={NAV_ITEMS} onOpenCommand={noop} />
    </AutoOpen>
);

export const FlatLinksOnly = () => <MainNav className="flex" items={[{ href: "/", label: "Home" }, { href: "/tier-lists", label: "Tier Lists" }, { href: "/changelog", label: "Changelog" }]} />;
