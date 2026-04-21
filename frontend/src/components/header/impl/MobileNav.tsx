import { Link, useRouterState } from "@tanstack/react-router";
import { ChevronDown, Cog, ExternalLinkIcon, LayoutList, LogOut, MenuIcon, UserIcon } from "lucide-react";
import { Avatar, AvatarFallback, AvatarImage } from "#/components/ui/avatar";
import { Button } from "#/components/ui/button";
import { Collapsible, CollapsiblePanel, CollapsibleTrigger } from "#/components/ui/collapsible";
import { Drawer, DrawerClose, DrawerHeader, DrawerMenu, DrawerMenuGroup, DrawerMenuGroupLabel, DrawerMenuItem, DrawerMenuSeparator, DrawerPanel, DrawerPopup, DrawerTitle, DrawerTrigger } from "#/components/ui/drawer";
import { Spinner } from "#/components/ui/spinner";
import { useAuth } from "#/hooks/use-auth";
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
                    <CollapsibleTrigger>
                        <DrawerMenuItem className="justify-between">
                            {item.label}
                            <ChevronDown className="h-4 w-4 opacity-50 transition-transform duration-200 group-data-[state=open]:rotate-180" />
                        </DrawerMenuItem>
                    </CollapsibleTrigger>
                    <CollapsiblePanel className="flex flex-col border-l border-border ml-4 mt-1 gap-1">
                        {item.items.map((subItem) => (
                            <DrawerMenuItem key={subItem.href} className="h-9 text-sm text-muted-foreground hover:text-foreground" render={<DrawerClose render={<Link to={subItem.href} />}></DrawerClose>}>
                                {subItem.label}
                            </DrawerMenuItem>
                        ))}
                    </CollapsiblePanel>
                </Collapsible>
            );
        }

        return (
            <DrawerMenuItem key={item.href} className={cn(isActive && "bg-accent text-accent-foreground")} render={<DrawerClose render={<Link to={item.href} />} />}>
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
                        <span className="flex h-6 w-6 items-center justify-center rounded-full bg-linear-to-br from-primary to-(--lagoon)">
                            <svg viewBox="0 0 24 24" className="h-3.5 w-3.5 text-white" fill="currentColor">
                                <title>myrtle.moe logo icon</title>
                                <path d="M12 2L2 7l10 5 10-5-10-5zM2 17l10 5 10-5M2 12l10 5 10-5" />
                            </svg>
                        </span>
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
                                    <a href="https://github.com/myrtle-moe" target="_blank" rel="noreferrer" className="cursor-pointer">
                                        GitHub
                                        <svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round" className="lucide lucide-github h-4 w-4" aria-hidden="true">
                                            <path d="M15 22v-4a4.8 4.8 0 0 0-1-3.5c3 0 6-2 6-5.5.08-1.25-.27-2.48-1-3.5.28-1.15.28-2.35 0-3.5 0 0-1 0-3 1.5-2.64-.5-5.36-.5-8 0C6 2 5 2 5 2c-.3 1.15-.3 2.35 0 3.5A5.403 5.403 0 0 0 4 9c0 3.5 3 5.5 6 5.5-.39.49-.68 1.05-.85 1.65-.17.6-.22 1.23-.15 1.85v4" />
                                            <path d="M9 18c-4.51 2-5-2-7-2" />
                                        </svg>
                                        <ExternalLinkIcon className="ml-auto h-3 w-3 opacity-50" />
                                    </a>
                                }
                            />
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
                                    <div className="flex items-center gap-3 px-2 py-2 mb-2">
                                        <Avatar className="h-8 w-8">
                                            <AvatarImage alt="User avatar" src={getAvatarSkinId(user)} />
                                            <AvatarFallback className="text-[0.625rem]">{(user.nickname ?? "Doctor").slice(0, 1) ?? "E"}</AvatarFallback>
                                        </Avatar>
                                        <div className="flex flex-col">
                                            <span className="text-sm font-medium">{user.nickname ?? "Doctor"}</span>
                                            <span className="text-xs text-muted-foreground">Level {user.level}</span>
                                        </div>
                                    </div>
                                    <DrawerMenuItem render={<DrawerClose render={<Link to="/my/tier-lists" />} />}>
                                        <LayoutList className="mr-2 h-4 w-4 text-muted-foreground" />
                                        My Tier Lists
                                    </DrawerMenuItem>
                                    <DrawerMenuItem render={<DrawerClose render={<Link to="/my/settings" />} />}>
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
