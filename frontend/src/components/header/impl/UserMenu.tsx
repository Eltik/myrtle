import { Link } from "@tanstack/react-router";
import { ChevronDown, Cog, Heart, LayoutList, LogOut, ShieldIcon } from "lucide-react";
import { Avatar, AvatarFallback, AvatarImage } from "#/components/ui/avatar";
import { Button } from "#/components/ui/button";
import { GithubIcon } from "#/components/ui/github-icon";
import { DropdownMenu, DropdownMenuContent, DropdownMenuItem, DropdownMenuSeparator, DropdownMenuTrigger } from "#/components/ui/menu";
import { Spinner } from "#/components/ui/spinner";
import type { ISession } from "#/lib/auth/server";
import { REPO_URL } from "#/lib/constants";
import { useT } from "#/lib/i18n";
import type { TypedT } from "#/lib/i18n/messages";
import { getAvatarSkinId } from "#/lib/utils";
import { LazyAuthDialog } from "./LazyAuthDialog";
import type { messages } from "./UserMenu.messages";
import { usePlayerName } from "./use-player-name";

export default function UserMenu({ user, loading, logout }: { user: ISession | null; loading: boolean; logout: () => Promise<void> }) {
    const t: TypedT<typeof messages> = useT("nav");
    const playerName = usePlayerName();

    if (loading) {
        return <Spinner />;
    }

    if (user !== null) {
        const name = playerName(user.nickname);
        return (
            <DropdownMenu>
                {/* On a touch screen each half is at least 44px wide and its `::after`
                    reaches 44px tall: the split was 32px tall with a chevron about 24px
                    wide, an easy miss for a thumb. */}
                <div className="flex h-8 min-w-0 shrink-0 items-center rounded-md border border-border bg-transparent text-foreground text-sm">
                    <Link
                        to="/user/$id"
                        params={{ id: user.uid }}
                        aria-label={name}
                        className="relative flex h-full min-w-0 items-center gap-2 rounded-l-md pointer-coarse:px-3 px-1.5 transition-colors pointer-coarse:after:absolute pointer-coarse:after:inset-x-0 pointer-coarse:after:top-1/2 pointer-coarse:after:min-h-11 pointer-coarse:after:-translate-y-1/2 hover:bg-secondary sm:px-2"
                    >
                        <Avatar className="h-5 w-5">
                            <AvatarImage alt={t("userMenu.userAvatar")} src={getAvatarSkinId(user)} />
                            <AvatarFallback className="text-[0.625rem]">{name.slice(0, 1)}</AvatarFallback>
                        </Avatar>
                        <span className="hidden max-w-24 truncate font-medium sm:inline-block">{name}</span>
                    </Link>
                    <DropdownMenuTrigger
                        render={(triggerProps) => (
                            <button
                                {...triggerProps}
                                type="button"
                                aria-label={t("userMenu.openMenu")}
                                className="relative flex h-full cursor-pointer items-center rounded-r-md border-border border-l pointer-coarse:px-4 px-1.5 transition-colors pointer-coarse:after:absolute pointer-coarse:after:inset-x-0 pointer-coarse:after:top-1/2 pointer-coarse:after:min-h-11 pointer-coarse:after:-translate-y-1/2 hover:bg-secondary"
                            >
                                <ChevronDown className="h-3 w-3" />
                            </button>
                        )}
                    />
                </div>
                <DropdownMenuContent align="end" className="w-48">
                    {/* A menu item, not a bare Link: Base UI closes the menu
                        on item click, while a plain anchor inside the popup
                        navigated and LEFT the menu open over the profile page. */}
                    <DropdownMenuItem className="cursor-pointer flex-col items-start gap-0 py-1.5" render={<Link to="/user/$id" params={{ id: user.uid }} />}>
                        <span className="font-medium text-sm">{name}</span>
                        <span className="text-muted-foreground text-xs">{t("userMenu.level", { level: user.level })}</span>
                    </DropdownMenuItem>
                    <DropdownMenuSeparator />
                    <DropdownMenuItem className="cursor-pointer" render={<Link to="/tier-lists/my" search={{ sort: "recent", type: "all", view: "grid", q: "" }} />}>
                        <LayoutList className="h-4 w-4 text-muted-foreground" />
                        {t("userMenu.myTierLists")}
                    </DropdownMenuItem>
                    <DropdownMenuItem className="cursor-pointer" render={<Link to="/settings" />}>
                        <Cog className="h-4 w-4 text-muted-foreground" />
                        {t("userMenu.settings")}
                    </DropdownMenuItem>
                    <DropdownMenuSeparator />
                    {/* Below `lg` the mobile drawer already carries a GitHub
                        link under "External", so this one would put the same
                        destination twice on one screen. The drawer trigger is
                        `lg:hidden`, so at `lg` and up this menu is the only
                        place the repo link lives. */}
                    {/* biome-ignore lint/a11y/useAnchorContent: anchor children are slotted in by DropdownMenuItem via the render prop */}
                    <DropdownMenuItem className="cursor-pointer max-lg:hidden" render={<a href={REPO_URL} target="_blank" rel="noreferrer" />}>
                        <GithubIcon className="h-4 w-4 text-muted-foreground" />
                        {t("userMenu.github")}
                    </DropdownMenuItem>
                    <DropdownMenuItem className="cursor-pointer" render={<Link to="/donate" />}>
                        <Heart className="h-4 w-4 text-muted-foreground" />
                        {t("userMenu.support")}
                    </DropdownMenuItem>
                    <DropdownMenuSeparator />
                    {user.canAccessAdminPanel ? (
                        <DropdownMenuItem className="cursor-pointer" render={<Link to="/admin" />}>
                            <ShieldIcon className="h-4 w-4 text-primary" />
                            {t("userMenu.adminPanel")}
                        </DropdownMenuItem>
                    ) : null}
                    {/* Red stays on the icon only: red label text on the highlighted
                        row's accent fill measured 3.34:1 in light, under AA. */}
                    <DropdownMenuItem className="cursor-pointer" onClick={logout}>
                        <LogOut className="h-4 w-4 text-primary" />
                        {t("userMenu.logout")}
                    </DropdownMenuItem>
                </DropdownMenuContent>
            </DropdownMenu>
        );
    }

    return <LazyAuthDialog trigger={<Button>{t("userMenu.login")}</Button>} />;
}
