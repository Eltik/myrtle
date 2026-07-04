import { Link, useRouterState } from "@tanstack/react-router";
import { ChevronDown, Cog, ExternalLinkIcon, Heart, LayoutList, LogOut, MenuIcon, UserIcon } from "lucide-react";
import { Avatar, AvatarFallback, AvatarImage } from "#/components/ui/avatar";
import { Button } from "#/components/ui/button";
import { Collapsible, CollapsiblePanel, CollapsibleTrigger } from "#/components/ui/collapsible";
import { Drawer, DrawerClose, DrawerHeader, DrawerMenu, DrawerMenuGroup, DrawerMenuGroupLabel, DrawerMenuItem, DrawerMenuSeparator, DrawerPanel, DrawerPopup, DrawerTitle, DrawerTrigger } from "#/components/ui/drawer";
import { GithubIcon } from "#/components/ui/github-icon";
import { Spinner } from "#/components/ui/spinner";
import { useAuth } from "#/hooks/use-auth";
import { REPO_URL } from "#/lib/constants";
import { cn, getAvatarSkinId } from "#/lib/utils";
import { ActiveIndicator } from "./ActiveIndicator";
import { AuthDialog } from "./AuthDialog";
import type { INavItem } from "./MainNav";

interface IMobileNavProps {
    items: INavItem[];
}

export function MobileNav({ items }: IMobileNavProps) {
    const { user, loading, logout } = useAuth();
    const router = useRouterState();
    const pathname = router.location.pathname;

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
                        {item.items.map((subItem) => (
                            <DrawerMenuItem key={subItem.href} className="h-9 text-muted-foreground text-sm hover:text-foreground" render={<DrawerClose nativeButton={false} render={<Link to={subItem.href} />}></DrawerClose>}>
                                {subItem.label}
                            </DrawerMenuItem>
                        ))}
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
            <DrawerTrigger render={<Button variant="ghost" size="icon" className="lg:hidden" aria-label="Open navigation menu" />}>
                <MenuIcon className="h-5 w-5" />
            </DrawerTrigger>
            <DrawerPopup showCloseButton className="w-70 max-w-[calc(100vw-3rem)]">
                <DrawerHeader>
                    <DrawerTitle className="flex items-center gap-2">
                        <img src="/logo/bust_transparent.png" alt="" width={28} height={28} className="h-7 w-7 shrink-0 object-contain" />
                        myrtle.moe
                    </DrawerTitle>
                </DrawerHeader>
                <DrawerPanel scrollFade={false}>
                    <DrawerMenu>
                        <DrawerMenuGroup>
                            <DrawerMenuGroupLabel>Navigation</DrawerMenuGroupLabel>
                            {items.map(renderNavItem)}
                        </DrawerMenuGroup>
                        <DrawerMenuSeparator />
                        <DrawerMenuGroup>
                            <DrawerMenuGroupLabel>External</DrawerMenuGroupLabel>
                            <DrawerMenuItem
                                render={
                                    <a href={REPO_URL} target="_blank" rel="noreferrer" className="cursor-pointer">
                                        GitHub
                                        <GithubIcon className="h-4 w-4" />
                                        <ExternalLinkIcon className="ml-auto h-3 w-3 opacity-50" />
                                    </a>
                                }
                            />
                            <DrawerMenuItem render={<DrawerClose nativeButton={false} render={<Link to="/donate" target="_blank" />} />}>
                                Donate
                                <Heart className="mr-2 h-4 w-4" />
                                <ExternalLinkIcon className="ml-auto h-3 w-3 opacity-50" />
                            </DrawerMenuItem>
                        </DrawerMenuGroup>

                        <DrawerMenuSeparator />
                        <DrawerMenuGroup>
                            <DrawerMenuGroupLabel>Account</DrawerMenuGroupLabel>
                            {loading ? (
                                <div className="flex h-12 items-center justify-center">
                                    <Spinner />
                                </div>
                            ) : user ? (
                                <>
                                    <div className="mb-2 flex items-center gap-3 px-2 py-2">
                                        <Avatar className="h-8 w-8">
                                            <AvatarImage alt="User avatar" src={getAvatarSkinId(user)} />
                                            <AvatarFallback className="text-[0.625rem]">{(user.nickname ?? "Doctor").slice(0, 1)}</AvatarFallback>
                                        </Avatar>
                                        <div className="flex flex-col">
                                            <span className="font-medium text-sm">{user.nickname ?? "Doctor"}</span>
                                            <span className="text-muted-foreground text-xs">Level {user.level}</span>
                                        </div>
                                    </div>
                                    <DrawerMenuItem render={<DrawerClose nativeButton={false} render={<Link to="/tier-lists/my" search={{ sort: "recent", type: "all", view: "grid", q: "" }} />} />}>
                                        <LayoutList className="mr-2 h-4 w-4 text-muted-foreground" />
                                        My Tier Lists
                                    </DrawerMenuItem>
                                    <DrawerMenuItem render={<DrawerClose nativeButton={false} render={<Link to="/settings" />} />}>
                                        <Cog className="mr-2 h-4 w-4 text-muted-foreground" />
                                        Settings
                                    </DrawerMenuItem>
                                    <DrawerMenuItem variant="destructive" onClick={logout}>
                                        <LogOut className="mr-2 h-4 w-4" />
                                        Logout
                                    </DrawerMenuItem>
                                </>
                            ) : (
                                <AuthDialog
                                    trigger={
                                        <DrawerMenuItem>
                                            <UserIcon className="mr-2 h-4 w-4 text-muted-foreground" />
                                            Login
                                        </DrawerMenuItem>
                                    }
                                />
                            )}
                        </DrawerMenuGroup>
                    </DrawerMenu>
                </DrawerPanel>
            </DrawerPopup>
        </Drawer>
    );
}
