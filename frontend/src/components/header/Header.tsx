import { Link } from "@tanstack/react-router";
import { Heart } from "lucide-react";
import { useMemo } from "react";
import { Button } from "#/components/ui/button";
import { GithubIcon } from "#/components/ui/github-icon";
import { Separator } from "#/components/ui/separator";
import { useAuth } from "#/hooks/use-auth";
import { useIsMac } from "#/hooks/use-is-mac";
import { useCommand } from "#/lib/command-context";
import { REPO_URL } from "#/lib/constants";
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
        const collectionItems: INavItem[] = [
            { href: "/operators", label: "Operators", desc: "Every operator released in Arknights.", icon: "shield" },
            { href: "/enemies", label: "Enemies", desc: "Every enemy catalogued, with stats and traits.", icon: "crosshair" },
            { href: "/stages", label: "Stages", desc: "Every stage, mapped with an enemy-pathing simulator.", icon: "map" },
        ];
        const gachaItems: INavItem[] = [
            { href: "/gacha/community", label: "Community", desc: "Pull rates, top operators, and timing across opted-in doctors", icon: "users" },
            { href: "/gacha/history", label: "History", desc: "Your synced pulls, rarity splits, and pity counters", icon: "history" },
        ];
        const playerItems: INavItem[] = [
            ...(user ? [{ href: `/user/${user.uid}`, label: "My Profile", desc: "Open your profile", icon: "user" as const }] : []),
            { href: "/user/search", label: "Search", desc: "Find profiles by nickname or UID", icon: "search" },
            { href: "/user/leaderboard", label: "Leaderboard", desc: "Top Doctors ranked by score", icon: "trophy" },
        ];
        return [
            { href: "/", label: "Home" },
            { href: "/operators", label: "Collection", items: collectionItems },
            { href: "/tools", label: "Tools", items: toolItems, sections: toolSections },
            { href: "/gacha", label: "Gacha", items: gachaItems },
            { href: "/tier-lists", label: "Tier Lists" },
            { href: "/user", label: "Players", items: playerItems },
        ];
    }, [user]);

    return (
        <header className="sticky top-0 z-50 w-full border-border border-b bg-background/80 backdrop-blur-lg backdrop-saturate-150 supports-backdrop-filter:bg-background/60">
            <div className="flex h-14 items-center gap-2 px-3 sm:h-16 sm:gap-3.5 sm:px-4">
                <div className="flex flex-1 items-center gap-2 sm:gap-4">
                    <MobileNav items={navItems} />
                    <Link to="/" className="flex shrink-0 items-center gap-2 text-foreground no-underline">
                        <img src="/logo/bust_transparent.png" alt="" width={28} height={28} className="h-7 w-7 shrink-0 object-contain" />
                        <span className="hidden font-semibold text-[15px] tracking-tight min-[340px]:inline">myrtle.moe</span>
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
                                <Link to="/donate" target="_blank" aria-label="Support myrtle.moe">
                                    <Heart className="h-4 w-4" aria-hidden="true" />
                                    <span className="sr-only">Support myrtle.moe</span>
                                </Link>
                            }
                        />
                        <Button
                            variant="ghost"
                            size="icon"
                            className="hidden sm:inline-flex"
                            render={
                                <a href={REPO_URL} target="_blank" rel="noreferrer" aria-label="GitHub">
                                    <GithubIcon className="h-4 w-4" />
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
