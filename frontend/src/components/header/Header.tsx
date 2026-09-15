import { Link } from "@tanstack/react-router";
import { Heart, Settings } from "lucide-react";
import { useMemo } from "react";
import { WhatsNewButton } from "#/components/changelog/WhatsNewButton";
import { Button } from "#/components/ui/button";
import { Separator } from "#/components/ui/separator";
import { useAuth } from "#/hooks/use-auth";
import { useIsMac } from "#/hooks/use-is-mac";
import { useCommand } from "#/lib/command-context";
import { useT } from "#/lib/i18n";
import type { TypedT } from "#/lib/i18n/messages";
import { getToolsByCategory, modKey } from "#/lib/registry/tools";
import type { messages as toolMessages } from "#/lib/registry/tools.messages";
import { Kbd } from "../ui/kbd";
import type { messages } from "./Header.messages";
import styles from "./impl/Header.module.css";
import LanguageToggle from "./impl/LanguageToggle";
import { type INavItem, type INavSection, MainNav } from "./impl/MainNav";
import { MobileNav } from "./impl/MobileNav";
import ThemeToggle from "./impl/ThemeToggle";
import UserMenu from "./impl/UserMenu";

export default function Header() {
    const { user, loading, logout } = useAuth();
    const { open: openCmd } = useCommand();
    const isMac = useIsMac();
    // The tool registry is a plain module, so its entries carry message keys;
    // this is the component that resolves them.
    const t: TypedT<typeof messages & typeof toolMessages> = useT("nav");

    const navItems = useMemo<INavItem[]>(() => {
        const toolSections: INavSection[] = getToolsByCategory().map(({ category, tools }) => ({
            title: t(category.labelKey),
            items: tools.map((tool) => ({ href: tool.href, label: t(tool.labelKey), desc: t(tool.descKey), icon: tool.icon })),
        }));
        const toolItems: INavItem[] = toolSections.flatMap((s) => s.items);
        const collectionItems: INavItem[] = [
            { href: "/operators", label: t("header.collection.operators.label"), desc: t("header.collection.operators.desc"), icon: "shield" },
            { href: "/enemies", label: t("header.collection.enemies.label"), desc: t("header.collection.enemies.desc"), icon: "crosshair" },
            { href: "/stages", label: t("header.collection.stages.label"), desc: t("header.collection.stages.desc"), icon: "map" },
        ];
        const gachaItems: INavItem[] = [
            { href: "/gacha/community", label: t("header.gacha.community.label"), desc: t("header.gacha.community.desc"), icon: "users" },
            { href: "/gacha/history", label: t("header.gacha.history.label"), desc: t("header.gacha.history.desc"), icon: "history" },
        ];
        const playerItems: INavItem[] = [
            ...(user ? [{ href: `/user/${user.uid}`, label: t("header.players.myProfile.label"), desc: t("header.players.myProfile.desc"), icon: "user" as const }] : []),
            { href: "/user/search", label: t("header.players.search.label"), desc: t("header.players.search.desc"), icon: "search" },
            { href: "/user/leaderboard", label: t("header.players.leaderboard.label"), desc: t("header.players.leaderboard.desc"), icon: "trophy" },
        ];
        return [
            { href: "/", label: t("header.home") },
            { href: "/operators", label: t("header.collection"), items: collectionItems },
            { href: "/tools", label: t("header.tools"), items: toolItems, sections: toolSections },
            { href: "/gacha", label: t("header.gacha"), items: gachaItems },
            { href: "/tier-lists", label: t("header.tierLists") },
            { href: "/user", label: t("header.players"), items: playerItems },
        ];
    }, [t, user]);

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
                        <button type="button" className={`${styles.headerSearch} hidden! xl:inline-flex!`} onClick={openCmd} aria-label={t("header.searchOperators")}>
                            <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round" aria-hidden="true">
                                <circle cx="11" cy="11" r="8" />
                                <path d="m21 21-4.35-4.35" />
                            </svg>
                            <span>{t("header.searchOperatorsPlaceholder")}</span>
                            <span className="kbd-inline ml-auto flex gap-1">
                                <Kbd>{modKey(isMac)}</Kbd>
                                <Kbd>K</Kbd>
                            </span>
                        </button>
                        <Button variant="ghost" size="icon" className="xl:hidden" onClick={openCmd} aria-label={t("header.search")}>
                            <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round" aria-hidden="true">
                                <circle cx="11" cy="11" r="8" />
                                <path d="m21 21-4.35-4.35" />
                            </svg>
                        </Button>

                        <Separator orientation="vertical" className="mx-1 hidden h-5 sm:block" />

                        <LanguageToggle />
                        <ThemeToggle />
                        <WhatsNewButton />
                        {/* Signed-out visitors have no avatar menu, so the two
                            outbound links stay in the bar for them. Signed-in
                            visitors find both inside UserMenu instead. */}
                        {user === null ? (
                            <>
                                <Button
                                    variant="ghost"
                                    size="icon"
                                    className="hidden sm:inline-flex"
                                    render={
                                        <Link to="/donate" target="_blank" aria-label={t("header.support")}>
                                            <Heart className="h-4 w-4" aria-hidden="true" />
                                            <span className="sr-only">{t("header.support")}</span>
                                        </Link>
                                    }
                                />
                                <Button
                                    variant="ghost"
                                    size="icon"
                                    className="hidden sm:inline-flex"
                                    render={
                                        <Link to="/settings" aria-label={t("header.settings")}>
                                            <Settings className="h-4 w-4" />
                                            <span className="sr-only">{t("header.settings")}</span>
                                        </Link>
                                    }
                                />
                            </>
                        ) : null}

                        <UserMenu loading={loading} user={user} logout={logout} />
                    </div>
                </div>
            </div>
        </header>
    );
}
