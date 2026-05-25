import { Link } from "@tanstack/react-router";
import { useMemo } from "react";
import { Button } from "#/components/ui/button";
import { Separator } from "#/components/ui/separator";
import { useAuth } from "#/hooks/use-auth";
import { useIsMac } from "#/hooks/use-is-mac";
import { useCommand } from "#/lib/command-context";
import { getToolsByCategory, modKey } from "#/lib/registry/tools";
import { Kbd } from "../ui/kbd";
import styles from "./impl/Header.module.css";
import { type INavItem, type INavSection, MainNav } from "./impl/MainNav";
import { MobileNav } from "./impl/MobileNav";
import ThemeToggle from "./impl/ThemeToggle";
import UserMenu from "./impl/UserMenu";

export default function Header() {
    const { user, loading, logout } = useAuth();
    const { open: openCmd } = useCommand();
    const isMac = useIsMac();

    const navItems = useMemo<INavItem[]>(() => {
        const toolSections: INavSection[] = getToolsByCategory().map(({ category, tools }) => ({
            title: category.label,
            items: tools.map((t) => ({ href: t.href, label: t.label, desc: t.desc, icon: t.icon })),
        }));
        const toolItems: INavItem[] = toolSections.flatMap((s) => s.items);
        return [
            { href: "/", label: "Home" },
            {
                href: "/operators",
                label: "Collection",
                items: [
                    { href: "/operators", label: "Operators", desc: "Every operator released in Arknights.", icon: "shield" },
                    { href: "/enemies", label: "Enemies", desc: "Every enemy catalogued, with stats and traits.", icon: "crosshair" },
                ],
            },
            { href: "/tier-lists", label: "Tier Lists" },
            {
                href: "/user",
                label: "Players",
                items: [
                    ...(user ? [{ href: `/user/${user.uid}`, label: "My Profile", desc: "Open your profile", icon: "user" }] : []),
                    { href: "/user/search", label: "Search", desc: "Find profiles by nickname or UID", icon: "search" },
                    { href: "/user/leaderboard", label: "Leaderboard", desc: "Top Doctors ranked by score", icon: "trophy" },
                ],
            },
            {
                href: "/gacha",
                label: "Gacha",
                items: [
                    { href: "/gacha/community", label: "Community", desc: "Pull rates, top operators, and timing across opted-in doctors", icon: "users" },
                    { href: "/gacha/history", label: "History", desc: "Your synced pulls, rarity splits, and pity counters", icon: "history" },
                ],
            },
            { href: "/tools", label: "Tools", items: toolItems, sections: toolSections },
        ];
    }, [user]);

    return (
        <header className="sticky top-0 z-50 w-full border-border border-b bg-background/80 backdrop-blur-lg backdrop-saturate-150 supports-backdrop-filter:bg-background/60">
            <div className="flex h-14 items-center gap-3.5 px-3 sm:h-16 sm:px-4">
                <div className="flex flex-1 items-center gap-2 sm:gap-4">
                    <MobileNav items={navItems} />
                    <Link to="/" className="flex shrink-0 items-center gap-2 text-foreground no-underline">
                        <img src="/logo/bust_transparent.png" alt="" width={28} height={28} className="h-7 w-7 shrink-0 object-contain" />
                        <span className="font-semibold text-[15px] tracking-tight">myrtle.moe</span>
                        <span className="hidden rounded-[5px] border border-border bg-muted/60 px-1.5 py-0.75 font-medium font-mono text-[10.5px] text-muted-foreground leading-none sm:inline-block">v3</span>
                    </Link>

                    <MainNav items={navItems} onOpenCommand={openCmd} />

                    <div className="ml-auto flex items-center gap-1 sm:gap-1.5">
                        <button type="button" className={`${styles.headerSearch} hidden! xl:inline-flex!`} onClick={openCmd} aria-label="Search operators">
                            <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round" aria-hidden="true">
                                <circle cx="11" cy="11" r="8" />
                                <path d="m21 21-4.35-4.35" />
                            </svg>
                            <span>Search operators…</span>
                            <span className="kbd-inline ml-auto flex gap-1">
                                <Kbd>{modKey(isMac)}</Kbd>
                                <Kbd>K</Kbd>
                            </span>
                        </button>
                        <Button variant="ghost" size="icon" className="xl:hidden" onClick={openCmd} aria-label="Search">
                            <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round" aria-hidden="true">
                                <circle cx="11" cy="11" r="8" />
                                <path d="m21 21-4.35-4.35" />
                            </svg>
                        </Button>

                        <Separator orientation="vertical" className="mx-1 hidden h-5 sm:block" />

                        <ThemeToggle />
                        <Button
                            variant="ghost"
                            size="icon"
                            className="hidden sm:inline-flex"
                            render={
                                <a href="https://github.com/Eltik/myrtle" target="_blank" rel="noreferrer" aria-label="GitHub">
                                    <svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round" className="h-4 w-4" aria-hidden="true">
                                        <path d="M15 22v-4a4.8 4.8 0 0 0-1-3.5c3 0 6-2 6-5.5.08-1.25-.27-2.48-1-3.5.28-1.15.28-2.35 0-3.5 0 0-1 0-3 1.5-2.64-.5-5.36-.5-8 0C6 2 5 2 5 2c-.3 1.15-.3 2.35 0 3.5A5.403 5.403 0 0 0 4 9c0 3.5 3 5.5 6 5.5-.39.49-.68 1.05-.85 1.65-.17.6-.22 1.23-.15 1.85v4" />
                                        <path d="M9 18c-4.51 2-5-2-7-2" />
                                    </svg>
                                    <span className="sr-only">GitHub</span>
                                </a>
                            }
                        />

                        <UserMenu loading={loading} user={user} logout={logout} />
                    </div>
                </div>
            </div>
        </header>
    );
}
