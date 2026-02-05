"use client";

import { ChevronDown, Cog, LayoutList, LogOut } from "lucide-react";
import Link from "next/link";
import { Avatar, AvatarFallback, AvatarImage } from "~/components/ui/shadcn/avatar";
import { Button } from "~/components/ui/shadcn/button";
import { DropdownMenu, DropdownMenuContent, DropdownMenuItem, DropdownMenuSeparator, DropdownMenuTrigger } from "~/components/ui/shadcn/dropdown-menu";
import type { AuthUser } from "~/hooks/use-auth";
import { getAvatarSkinId } from "~/lib/utils";
import { Login } from "../../login";

interface UserMenuProps {
    user: AuthUser | null;
    loading: boolean;
    logout: () => void;
}

export function UserMenu({ user, loading, logout }: UserMenuProps) {
    if (loading) {
        return (
            <div className="flex h-8 w-8 items-center justify-center">
                <div className="h-4 w-4 animate-spin rounded-full border-2 border-muted-foreground border-t-transparent" />
            </div>
        );
    }

    if (user?.status) {
        return (
            <DropdownMenu>
                <DropdownMenuTrigger asChild>
                    <Button className="flex h-8 items-center gap-2 rounded-md border border-border bg-transparent px-2 text-foreground text-sm transition-colors hover:bg-secondary" variant="ghost">
                        <Avatar className="h-5 w-5">
                            <AvatarImage alt="User avatar" src={getAvatarSkinId(user)} />
                            <AvatarFallback className="text-2.5">{user.status.nickName.slice(0, 1) ?? "E"}</AvatarFallback>
                        </Avatar>
                        <span className="max-w-24 truncate font-medium">{user.status.nickName}</span>
                        <ChevronDown className="h-3 w-3" />
                    </Button>
                </DropdownMenuTrigger>
                <DropdownMenuContent align="end" className="w-48">
                    <div className="px-2 pb-1.5">
                        <Link className="font-medium text-sm hover:underline" href={`/user/${user.status.uid}`}>
                            {user.status.nickName}
                        </Link>
                        <p className="text-muted-foreground text-xs">Level {user.status.level}</p>
                    </div>
                    <DropdownMenuSeparator />
                    <DropdownMenuItem asChild className="cursor-pointer">
                        <Link href="/my/tier-lists">
                            <LayoutList className="h-4 w-4" />
                            My Tier Lists
                        </Link>
                    </DropdownMenuItem>
                    <DropdownMenuItem asChild className="cursor-pointer">
                        <Link href="/my/settings">
                            <Cog className="h-4 w-4" />
                            Settings
                        </Link>
                    </DropdownMenuItem>
                    <DropdownMenuItem className="text-primary transition-colors focus:text-primary/80" onClick={logout}>
                        <LogOut className="mr-2 h-4 w-4" />
                        Logout
                    </DropdownMenuItem>
                </DropdownMenuContent>
            </DropdownMenu>
        );
    }

    return <Login variant="header" />;
}
