import { Link, useRouterState } from "@tanstack/react-router";
import { ChevronDown, Cog, ExternalLinkIcon, Heart, LayoutList, LogOut, MenuIcon, UserIcon } from "lucide-react";
import { Fragment, useId } from "react";
import { useGamedataServerChoice, useGamedataServerOptions, useGamedataServerSwitch } from "#/components/GamedataServerSwitcher";
import type { messages as gamedataServerMessages } from "#/components/GamedataServerSwitcher.messages";
import { LocaleOptionLabel, useLocaleSwitch } from "#/components/LocaleSwitcher";
import type { messages as localeSwitcherMessages } from "#/components/LocaleSwitcher.messages";
import { Avatar, AvatarFallback, AvatarImage } from "#/components/ui/avatar";
import { Button } from "#/components/ui/button";
import { Collapsible, CollapsiblePanel, CollapsibleTrigger } from "#/components/ui/collapsible";
import { Drawer, DrawerClose, DrawerHeader, DrawerMenu, DrawerMenuGroup, DrawerMenuGroupLabel, DrawerMenuItem, DrawerMenuRadioGroup, DrawerMenuRadioItem, DrawerMenuSeparator, DrawerPanel, DrawerPopup, DrawerTitle, DrawerTrigger } from "#/components/ui/drawer";
import { GithubIcon } from "#/components/ui/github-icon";
import { Spinner } from "#/components/ui/spinner";
import { useAuth } from "#/hooks/use-auth";
import { useTheme } from "#/hooks/use-theme";
import { REPO_URL } from "#/lib/constants";
import { useI18n, useT } from "#/lib/i18n";
import type { TypedT } from "#/lib/i18n/messages";
import { cn, getAvatarSkinId } from "#/lib/utils";
import { ActiveIndicator } from "./ActiveIndicator";
import { LazyAuthDialog } from "./LazyAuthDialog";
import type { INavItem } from "./MainNav";
import type { messages } from "./MobileNav.messages";
import type { messages as themeMessages } from "./ThemeToggle.messages";

interface IMobileNavProps {
    items: INavItem[];
    /** The desktop nav did not fit, so the hamburger shows at every width. */
    forceVisible?: boolean;
}

export function MobileNav({ items, forceVisible = false }: IMobileNavProps) {
    const { user, loading, logout } = useAuth();
    const router = useRouterState();
    const pathname = router.location.pathname;
    const t: TypedT<typeof messages> = useT("nav");
    const tCommon: TypedT<typeof localeSwitcherMessages & typeof gamedataServerMessages> = useT("common");
    const { locale, available } = useI18n();
    const switchLocale = useLocaleSwitch();
    const serverOptions = useGamedataServerOptions();
    const serverChoice = useGamedataServerChoice();
    const switchServer = useGamedataServerSwitch();
    const { mode, setMode } = useTheme();
    const tTheme: TypedT<typeof themeMessages> = useT("nav");
    const groupIdBase = useId();

    const renderSubItem = (subItem: INavItem) => (
        <DrawerMenuItem key={subItem.href} className="h-9 text-muted-foreground text-sm hover:text-foreground" render={<DrawerClose nativeButton={false} render={<Link to={subItem.href} />}></DrawerClose>}>
            {subItem.label}
        </DrawerMenuItem>
    );

    const renderNavItem = (item: INavItem) => {
        const isActive = pathname === item.href || (item.href !== "/" && pathname.startsWith(item.href));

        if (item.items && item.items.length > 0) {
            return (
                <Collapsible key={item.href} className="flex flex-col">
                    <CollapsibleTrigger className="group" render={<DrawerMenuItem className="justify-between" />}>
                        {item.label}
                        <ChevronDown className="h-4 w-4 opacity-50 transition-transform duration-200 group-data-panel-open:rotate-180" />
                    </CollapsibleTrigger>
                    <CollapsiblePanel className="mt-1 ml-4 flex flex-col gap-1 border-border border-l">
                        {/* The same sections the desktop dropdown shows, with the
                            drawer's own group label and separator. */}
                        {item.sections && item.sections.length > 1
                            ? item.sections.map((section, i) => {
                                  const labelId = `${groupIdBase}-${item.href}-${i}`;
                                  return (
                                      <Fragment key={section.title}>
                                          {i > 0 && <DrawerMenuSeparator className="my-0" />}
                                          <DrawerMenuGroup role="group" aria-labelledby={labelId} className="gap-1">
                                              <DrawerMenuGroupLabel id={labelId}>{section.title}</DrawerMenuGroupLabel>
                                              {section.items.map(renderSubItem)}
                                          </DrawerMenuGroup>
                                      </Fragment>
                                  );
                              })
                            : item.items.map(renderSubItem)}
                    </CollapsiblePanel>
                </Collapsible>
            );
        }

        return (
            <DrawerMenuItem key={item.href} className={cn(isActive && "bg-accent text-accent-foreground")} render={<DrawerClose nativeButton={false} render={<Link to={item.href} />} />}>
                {item.label}
                {isActive && <ActiveIndicator />}
            </DrawerMenuItem>
        );
    };

    return (
        <Drawer position="left">
            <DrawerTrigger render={<Button variant="ghost" size="icon" className={cn(!forceVisible && "lg:hidden")} aria-label={t("mobileNav.openMenu")} />}>
                <MenuIcon className="h-5 w-5" />
            </DrawerTrigger>
            <DrawerPopup showCloseButton className="w-70 max-w-[calc(100vw-3rem)]">
                <DrawerHeader>
                    <DrawerTitle className="flex items-center gap-2">
                        <img src="/logo/bust_128.png" alt="" width={28} height={28} className="h-7 w-7 shrink-0 object-contain" />
                        myrtle.moe
                    </DrawerTitle>
                </DrawerHeader>
                <DrawerPanel scrollFade={false}>
                    <DrawerMenu>
                        <DrawerMenuGroup>
                            <DrawerMenuGroupLabel>{t("mobileNav.navigation")}</DrawerMenuGroupLabel>
                            {items.map(renderNavItem)}
                        </DrawerMenuGroup>
                        <DrawerMenuSeparator />
                        <DrawerMenuGroup>
                            <DrawerMenuGroupLabel>{t("mobileNav.external")}</DrawerMenuGroupLabel>
                            <DrawerMenuItem
                                render={
                                    <a href={REPO_URL} target="_blank" rel="noreferrer" className="cursor-pointer">
                                        {t("mobileNav.github")}
                                        <GithubIcon className="h-4 w-4" />
                                        <ExternalLinkIcon className="ml-auto h-3 w-3 opacity-50" />
                                    </a>
                                }
                            />
                            <DrawerMenuItem render={<DrawerClose nativeButton={false} render={<Link to="/donate" target="_blank" />} />}>
                                {t("mobileNav.donate")}
                                <Heart className="mr-2 h-4 w-4" />
                                <ExternalLinkIcon className="ml-auto h-3 w-3 opacity-50" />
                            </DrawerMenuItem>
                        </DrawerMenuGroup>

                        {/* Below `sm` this is the only path: the header drops its
                            language and theme toggles there. From `sm` up it is a
                            second path that spells the group out ("Language")
                            where the bar has room for a two-letter code. */}
                        {available.length > 1 ? (
                            <>
                                <DrawerMenuSeparator />
                                <DrawerMenuGroup>
                                    <DrawerMenuGroupLabel>{tCommon("localeSwitcher.language")}</DrawerMenuGroupLabel>
                                    <DrawerMenuRadioGroup
                                        value={locale}
                                        onValueChange={(next: string) => {
                                            if (next !== locale) switchLocale(next);
                                        }}
                                    >
                                        {available.map((entry) => (
                                            <DrawerMenuRadioItem key={entry.code} value={entry.code} lang={entry.code}>
                                                <LocaleOptionLabel entry={entry} />
                                            </DrawerMenuRadioItem>
                                        ))}
                                    </DrawerMenuRadioGroup>
                                </DrawerMenuGroup>
                            </>
                        ) : null}

                        {serverOptions.length > 0 ? (
                            <>
                                <DrawerMenuSeparator />
                                <DrawerMenuGroup>
                                    <DrawerMenuGroupLabel>{tCommon("gamedataServer.label")}</DrawerMenuGroupLabel>
                                    <DrawerMenuRadioGroup
                                        value={serverChoice}
                                        onValueChange={(next: string) => {
                                            if (next !== serverChoice) switchServer(next);
                                        }}
                                    >
                                        {serverOptions.map((option) => (
                                            <DrawerMenuRadioItem key={option.value} value={option.value}>
                                                {option.label}
                                            </DrawerMenuRadioItem>
                                        ))}
                                    </DrawerMenuRadioGroup>
                                </DrawerMenuGroup>
                            </>
                        ) : null}

                        {/* Mode only; accent and dynamic art stay in Settings > Appearance. */}
                        <DrawerMenuSeparator />
                        <DrawerMenuGroup>
                            <DrawerMenuGroupLabel>{tTheme("themeToggle.appearance")}</DrawerMenuGroupLabel>
                            <DrawerMenuRadioGroup
                                value={mode}
                                onValueChange={(next: string) => {
                                    if (next === "light" || next === "dark" || next === "auto") setMode(next);
                                }}
                            >
                                <DrawerMenuRadioItem value="light">{tTheme("themeToggle.light")}</DrawerMenuRadioItem>
                                <DrawerMenuRadioItem value="dark">{tTheme("themeToggle.dark")}</DrawerMenuRadioItem>
                                <DrawerMenuRadioItem value="auto">{tTheme("themeToggle.auto")}</DrawerMenuRadioItem>
                            </DrawerMenuRadioGroup>
                        </DrawerMenuGroup>

                        <DrawerMenuSeparator />
                        <DrawerMenuGroup>
                            <DrawerMenuGroupLabel>{t("mobileNav.account")}</DrawerMenuGroupLabel>
                            {loading ? (
                                <div className="flex h-12 items-center justify-center">
                                    <Spinner />
                                </div>
                            ) : user ? (
                                <>
                                    <div className="mb-2 flex items-center gap-3 px-2 py-2">
                                        <Avatar className="h-8 w-8">
                                            <AvatarImage alt={t("mobileNav.userAvatar")} src={getAvatarSkinId(user)} />
                                            <AvatarFallback className="text-[0.625rem]">{(user.nickname ?? "Player").slice(0, 1)}</AvatarFallback>
                                        </Avatar>
                                        <div className="flex flex-col">
                                            <span className="font-medium text-sm">{user.nickname ?? "Player"}</span>
                                            <span className="text-muted-foreground text-xs">{t("mobileNav.level", { level: user.level })}</span>
                                        </div>
                                    </div>
                                    <DrawerMenuItem render={<DrawerClose nativeButton={false} render={<Link to="/tier-lists/my" search={{ sort: "recent", type: "all", view: "grid", q: "" }} />} />}>
                                        <LayoutList className="mr-2 h-4 w-4 text-muted-foreground" />
                                        {t("mobileNav.myTierLists")}
                                    </DrawerMenuItem>
                                    <DrawerMenuItem render={<DrawerClose nativeButton={false} render={<Link to="/settings" />} />}>
                                        <Cog className="mr-2 h-4 w-4 text-muted-foreground" />
                                        {t("mobileNav.settings")}
                                    </DrawerMenuItem>
                                    <DrawerMenuItem variant="destructive" onClick={logout}>
                                        <LogOut className="mr-2 h-4 w-4" />
                                        {t("mobileNav.logout")}
                                    </DrawerMenuItem>
                                </>
                            ) : (
                                <>
                                    <DrawerMenuItem render={<DrawerClose nativeButton={false} render={<Link to="/settings" />} />}>
                                        <Cog className="mr-2 h-4 w-4 text-muted-foreground" />
                                        {t("mobileNav.settings")}
                                    </DrawerMenuItem>
                                    <LazyAuthDialog
                                        trigger={
                                            <DrawerMenuItem>
                                                <UserIcon className="mr-2 h-4 w-4 text-muted-foreground" />
                                                {t("mobileNav.login")}
                                            </DrawerMenuItem>
                                        }
                                    />
                                </>
                            )}
                        </DrawerMenuGroup>
                    </DrawerMenu>
                </DrawerPanel>
            </DrawerPopup>
        </Drawer>
    );
}
