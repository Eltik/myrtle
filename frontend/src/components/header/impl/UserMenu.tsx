import { Link } from "@tanstack/react-router";
import { ChevronDown, Cog, LayoutList, LogOut, ShieldIcon } from "lucide-react";
import { Avatar, AvatarFallback, AvatarImage } from "#/components/ui/avatar";
import { Button } from "#/components/ui/button";
import { DropdownMenu, DropdownMenuContent, DropdownMenuItem, DropdownMenuSeparator, DropdownMenuTrigger } from "#/components/ui/menu";
import { Spinner } from "#/components/ui/spinner";
import { isAnyAdminRole } from "#/lib/api/admin";
import { getAvatarSkinId } from "#/lib/utils";
import type { IUserProfile } from "#/types/user";
import { AuthDialog } from "./AuthDialog";

export default function UserMenu({ user, loading, logout }: { user: IUserProfile | null; loading: boolean; logout: () => Promise<void> }) {
    if (loading) {
        return <Spinner />;
    }

    if (user !== null) {
        return (
            <DropdownMenu>
                <div className="flex h-8 min-w-0 shrink-0 items-center rounded-md border border-border bg-transparent text-foreground text-sm">
                    <Link to="/user/$id" params={{ id: user.uid }} aria-label={user.nickname ?? "Doctor"} className="flex h-full min-w-0 items-center gap-2 rounded-l-md px-1.5 transition-colors hover:bg-secondary sm:px-2">
                        <Avatar className="h-5 w-5">
                            <AvatarImage alt="User avatar" src={getAvatarSkinId(user)} />
                            <AvatarFallback className="text-[0.625rem]">{(user.nickname ?? "Doctor").slice(0, 1) ?? "E"}</AvatarFallback>
                        </Avatar>
                        <span className="hidden max-w-24 truncate font-medium sm:inline-block">{user.nickname ?? "Doctor"}</span>
                    </Link>
                    <DropdownMenuTrigger
                        render={(triggerProps) => (
                            <button {...triggerProps} type="button" aria-label="Open user menu" className="flex h-full cursor-pointer items-center rounded-r-md border-border border-l px-1.5 transition-colors hover:bg-secondary">
                                <ChevronDown className="h-3 w-3" />
                            </button>
                        )}
                    />
                </div>
                <DropdownMenuContent align="end" className="w-48">
                    <div className="px-2 pb-1.5">
                        <Link className="font-medium text-sm hover:underline" to="/user/$id" params={{ id: user.uid }}>
                            {user.nickname ?? "Doctor"}
                        </Link>
                        <p className="text-muted-foreground text-xs">Level {user.level}</p>
                    </div>
                    <DropdownMenuSeparator />
                    <DropdownMenuItem className="cursor-pointer">
                        <Link to="/tier-lists/my" search={{ sort: "recent", type: "all", view: "grid", q: "" }} className="flex flex-row items-center gap-2">
                            <LayoutList className="h-4 w-4 text-muted-foreground" />
                            My Tier Lists
                        </Link>
                    </DropdownMenuItem>
                    <DropdownMenuItem className="cursor-pointer">
                        <Link to="/settings" className="flex flex-row items-center gap-2">
                            <Cog className="h-4 w-4 text-muted-foreground" />
                            Settings
                        </Link>
                    </DropdownMenuItem>
                    {isAnyAdminRole(user.role) ? (
                        <DropdownMenuItem className="cursor-pointer">
                            <Link to="/admin" className="flex flex-row items-center gap-2">
                                <ShieldIcon className="h-4 w-4 text-primary" />
                                Admin panel
                            </Link>
                        </DropdownMenuItem>
                    ) : null}
                    <DropdownMenuItem className="cursor-pointer pl-3 text-primary transition-colors focus:text-primary/80" onClick={logout}>
                        <LogOut className="mr-2 h-4 w-4" />
                        Logout
                    </DropdownMenuItem>
                </DropdownMenuContent>
            </DropdownMenu>
        );
    }

    return <AuthDialog trigger={<Button>Login</Button>} />;
}
