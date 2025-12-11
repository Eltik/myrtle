import Link from "next/link";
import { Button } from "~/components/ui/button";
import { SidebarTrigger } from "~/components/ui/sidebar";
import { Dialog, DialogTrigger } from "../ui/dialog";
import { LoginDialogue } from "./login-dialogue";
import { useCookies } from "react-cookie";
import { useStore } from "zustand";
import { usePlayer } from "~/store";
import { Avatar, AvatarFallback, AvatarImage } from "../ui/avatar";
import type { StoredUser } from "~/types/impl/api";
import { DropdownMenu, DropdownMenuContent, DropdownMenuGroup, DropdownMenuItem, DropdownMenuLabel, DropdownMenuSeparator, DropdownMenuShortcut, DropdownMenuTrigger } from "../ui/dropdown-menu";
import { ChevronDown } from "lucide-react";
import { getAvatarSkinId } from "~/helper";

export function Navbar() {
    const [cookies] = useCookies(["login"]);
    const playerData = useStore(usePlayer, (state) => (state as { playerData: StoredUser })?.playerData);

    return (
        <>
            <Dialog>
                <header className="bg-background-900/95 fixed top-0 z-30 w-full border-b shadow backdrop-blur supports-[backdrop-filter]:bg-background/60 dark:shadow-none">
                    <div className="mx-4 flex h-14 items-center sm:mx-4 md:mx-8 md:ml-32 lg:ml-32">
                        <div className="flex items-center">
                            <SidebarTrigger className="block md:hidden" />
                            <Link href={"/"} className="text-base font-bold">
                                myrtle.moe
                            </Link>
                        </div>
                        <div className="flex flex-1 items-center justify-end space-x-2 lg:mr-14">
                            {cookies.login && playerData && playerData.status !== null && playerData.status !== undefined ? (
                                <>
                                    <DropdownMenu>
                                        <DropdownMenuTrigger>
                                            <div className="flex flex-row rounded-md border px-2 transition-all duration-150 hover:bg-secondary">
                                                <Avatar className="h-12 w-12">
                                                    <AvatarImage src={getAvatarSkinId(playerData)} alt="@shadcn" />
                                                    <AvatarFallback>{playerData.status?.nickName?.slice(0, 1) ?? "E"}</AvatarFallback>
                                                </Avatar>
                                                <div className="flex flex-row items-center gap-2">
                                                    <div className="ml-2 hidden flex-col justify-center md:flex">
                                                        <div className="text-sm font-semibold">{playerData.status?.nickName && playerData.status?.nickName.length > 0 ? playerData.status?.nickName : "Unknown"}</div>
                                                        <div className="text-xs text-muted-foreground">#{playerData.status?.nickNumber ?? "-1"}</div>
                                                    </div>
                                                    <ChevronDown size={15} />
                                                </div>
                                            </div>
                                        </DropdownMenuTrigger>
                                        <DropdownMenuContent className="w-56">
                                            <DropdownMenuLabel>My Account</DropdownMenuLabel>
                                            <DropdownMenuSeparator />
                                            <DropdownMenuGroup>
                                                <Link href={`/user/${playerData.status.uid}`}>
                                                    <DropdownMenuItem className="cursor-pointer">
                                                        Profile
                                                        <DropdownMenuShortcut>⇧⌘P</DropdownMenuShortcut>
                                                    </DropdownMenuItem>
                                                </Link>
                                                <Link href={"/statistics"}>
                                                    <DropdownMenuItem className="cursor-pointer">
                                                        Statistics
                                                        <DropdownMenuShortcut>⇧⌘⌥S</DropdownMenuShortcut>
                                                    </DropdownMenuItem>
                                                </Link>
                                                <Link href={"/settings"}>
                                                    <DropdownMenuItem className="cursor-pointer">
                                                        Settings
                                                        <DropdownMenuShortcut>⌘S</DropdownMenuShortcut>
                                                    </DropdownMenuItem>
                                                </Link>
                                            </DropdownMenuGroup>
                                            <DropdownMenuSeparator />
                                            <DropdownMenuGroup>
                                                <Link href={"https://github.com/Eltik/myrtle"} target="_blank">
                                                    <DropdownMenuItem className="cursor-pointer">GitHub</DropdownMenuItem>
                                                </Link>
                                                <Link href={"#"}>
                                                    <DropdownMenuItem className="cursor-pointer">Discord</DropdownMenuItem>
                                                </Link>
                                                <DropdownMenuItem disabled>API</DropdownMenuItem>
                                            </DropdownMenuGroup>
                                            <DropdownMenuSeparator />
                                            <Link href={"/logout"}>
                                                <DropdownMenuItem className="cursor-pointer">
                                                    Log Out
                                                    <DropdownMenuShortcut>⇧⌘Q</DropdownMenuShortcut>
                                                </DropdownMenuItem>
                                            </Link>
                                        </DropdownMenuContent>
                                    </DropdownMenu>
                                </>
                            ) : (
                                <>
                                    <DialogTrigger asChild>
                                        <Button>Login</Button>
                                    </DialogTrigger>
                                    <LoginDialogue />
                                </>
                            )}
                        </div>
                    </div>
                </header>
            </Dialog>
        </>
    );
}
