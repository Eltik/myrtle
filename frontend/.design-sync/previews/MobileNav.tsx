import { MobileNav } from "frontend";
import { type ReactNode, useEffect, useRef } from "react";

// The hamburger + left drawer that replaces `MainNav` below 1024px. `items` is
// the same `navItems` model the header builds in
// `src/components/header/Header.tsx` (signed-out branch), so the drawer shows
// Navigation / External / Account exactly as the app does.

const COLLECTION = [
    { href: "/operators", label: "Operators", desc: "Every operator released in Arknights.", icon: "shield" as const },
    { href: "/enemies", label: "Enemies", desc: "Every enemy catalogued, with stats and traits.", icon: "crosshair" as const },
    { href: "/stages", label: "Stages", desc: "Every stage, mapped with an enemy-pathing simulator.", icon: "map" as const },
];

const TOOLS = [
    { href: "/tools/recruitment", label: "Recruitment calculator", desc: "Guaranteed tag combos · 1h parity", icon: "calc" as const },
    { href: "/tools/planner", label: "Operator planner", desc: "Plan promotions, skills, and modules", icon: "list-todo" as const },
    { href: "/tools/dps", label: "DPS charts", desc: "Interactive damage curves per skill", icon: "chart" as const },
    { href: "/tools/randomizer", label: "Randomizer", desc: "Pick a squad, break the meta", icon: "dice" as const },
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
    { href: "/tools", label: "Tools", items: TOOLS },
    { href: "/gacha", label: "Gacha", items: GACHA },
    { href: "/tier-lists", label: "Tier Lists" },
    { href: "/user", label: "Players", items: PLAYERS },
];

const SHORT_ITEMS = [
    { href: "/", label: "Home" },
    { href: "/operators", label: "Collection", items: COLLECTION },
    { href: "/tier-lists", label: "Tier Lists" },
];

// The drawer keeps its own open state - there is no `open` prop - so the trigger
// is clicked on mount, deferred two frames (Base UI wires the trigger after the
// first paint and an earlier click is silently dropped).
const AutoOpen = ({ children }: { children: ReactNode }) => {
    const ref = useRef<HTMLDivElement>(null);
    useEffect(() => {
        let inner = 0;
        const outer = requestAnimationFrame(() => {
            inner = requestAnimationFrame(() => ref.current?.querySelector<HTMLButtonElement>('button[aria-label="Open navigation menu"]')?.click());
        });
        return () => {
            cancelAnimationFrame(outer);
            cancelAnimationFrame(inner);
        };
    }, []);
    // The drawer dims the page behind it, so the stage carries a real page —
    // an empty stage renders as a flat grey block and reads as a broken cell.
    return (
        <div className="relative min-h-[520px] w-full" ref={ref}>
            <div className="flex items-center gap-2 border-border border-b px-3 py-3">
                {children}
                <span className="font-semibold text-[15px] tracking-tight">myrtle.moe</span>
                <span className="rounded-[5px] border border-border bg-muted px-1.5 py-0.5 font-medium font-mono text-[10.5px] text-muted-foreground leading-none">v3</span>
            </div>
            <main className="px-4 pt-6">
                <p className="m-0 mb-2 font-medium font-mono text-[10.5px] text-muted-foreground uppercase leading-none tracking-widest">Collection</p>
                <h1 className="m-0 mb-3 font-bold font-sans text-4xl text-foreground leading-tight tracking-tight">Operators</h1>
                <p className="m-0 mb-6 max-w-[52ch] font-sans text-[15px] text-muted-foreground leading-relaxed">438 operators with full stats, skills, talents and modules.</p>
                <div className="grid grid-cols-1 gap-3 sm:grid-cols-2">
                    {["Mlynar — 6★ Guard", "Skadi — 6★ Guard", "Texas — 5★ Vanguard", "Eyjafjalla — 6★ Caster", "Muelsyse — 6★ Vanguard", "Myrtle — 4★ Vanguard"].map((row) => (
                        <div key={row} className="rounded-lg border border-border bg-card px-4 py-3 font-sans text-[13.5px] text-foreground">
                            {row}
                        </div>
                    ))}
                </div>
            </main>
        </div>
    );
};

export const NavigationDrawer = () => (
    <AutoOpen>
        <MobileNav items={NAV_ITEMS} />
    </AutoOpen>
);

export const ShortNavigation = () => (
    <AutoOpen>
        <MobileNav items={SHORT_ITEMS} />
    </AutoOpen>
);

export const ClosedTrigger = () => (
    <div className="flex w-fit items-center gap-2 rounded-lg border border-border bg-background px-2 py-1.5">
        <MobileNav items={NAV_ITEMS} />
        <span className="font-semibold text-[15px] tracking-tight">myrtle.moe</span>
    </div>
);
