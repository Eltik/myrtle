"use client";

import { LogOut, Menu } from "lucide-react";
import Link from "next/link";
import { Accordion, AccordionContent, AccordionItem, AccordionTrigger } from "~/components/ui/shadcn/accordion";
import { Avatar, AvatarFallback, AvatarImage } from "~/components/ui/shadcn/avatar";
import { Button } from "~/components/ui/shadcn/button";
import { Sheet, SheetContent, SheetHeader, SheetTitle, SheetTrigger } from "~/components/ui/shadcn/sheet";
import type { AuthUser } from "~/hooks/use-auth";
import { getAvatarSkinId } from "~/lib/utils";
import { Login } from "../../login";
import { navItems } from "./constants";
import { isNavItemActive } from "./helpers";

interface NavMobileProps {
    pathname: string;
    user: AuthUser | null;
    loading: boolean;
    logout: () => void;
    mobileMenuOpen: boolean;
    setMobileMenuOpen: (open: boolean) => void;
}

export function NavMobile({ pathname, user, loading, logout, mobileMenuOpen, setMobileMenuOpen }: NavMobileProps) {
    return (
        <Sheet onOpenChange={setMobileMenuOpen} open={mobileMenuOpen}>
            <SheetTrigger asChild>
                <Button className="h-8 w-8 md:hidden" size="icon" variant="ghost">
                    <Menu className="h-4 w-4" />
                    <span className="sr-only">Open menu</span>
                </Button>
            </SheetTrigger>
            <SheetContent
                className="w-[280px] overflow-y-auto border-border sm:w-[320px]"
                side="left"
                style={{
                    background: "linear-gradient(180deg, oklch(0.15 0.005 285) 0%, oklch(0.12 0.005 285) 100%)",
                }}
            >
                <SheetHeader className="border-border border-b pb-4">
                    <SheetTitle className="flex items-center gap-2">
                        <div className="flex h-7 w-7 items-center justify-center rounded-lg bg-primary" />
                        <span className="font-semibold text-foreground">myrtle.moe</span>
                    </SheetTitle>
                </SheetHeader>
                <nav className="flex flex-col gap-1 py-4">
                    {navItems.map((item, index) =>
                        item.dropdown ? (
                            <Accordion collapsible key={typeof item.label === "string" ? item.label : index} type="single">
                                <AccordionItem className="border-b-0" value={`item-${index}`}>
                                    <AccordionTrigger className={`rounded-md px-3 py-2.5 hover:no-underline ${isNavItemActive(item, pathname) ? "bg-primary/10 text-foreground" : "text-muted-foreground hover:bg-secondary hover:text-foreground"}`}>
                                        <span className="font-medium">{item.label}</span>
                                    </AccordionTrigger>
                                    <AccordionContent className="pt-1 pb-1">
                                        <div className="ml-3 flex flex-col gap-0.5 border-border border-l pl-3">
                                            {item.dropdown.map((dropdownItem) => (
                                                <Link
                                                    className={`flex flex-col gap-0.5 rounded-md px-3 py-2 transition-colors ${pathname === dropdownItem.href ? "bg-primary/10 text-foreground" : "text-muted-foreground hover:bg-secondary hover:text-foreground"}`}
                                                    href={dropdownItem.href}
                                                    key={dropdownItem.label}
                                                    onClick={() => setMobileMenuOpen(false)}
                                                >
                                                    <span className="font-medium text-sm">{dropdownItem.label}</span>
                                                    <span className="text-muted-foreground text-xs">{dropdownItem.description}</span>
                                                </Link>
                                            ))}
                                        </div>
                                    </AccordionContent>
                                </AccordionItem>
                            </Accordion>
                        ) : (
                            <Link
                                className={`rounded-md px-3 py-2.5 font-medium transition-colors ${isNavItemActive(item, pathname) ? "bg-primary/10 text-foreground" : "text-muted-foreground hover:bg-secondary hover:text-foreground"}`}
                                href={item.href}
                                key={typeof item.label === "string" ? item.label : index}
                                onClick={() => setMobileMenuOpen(false)}
                            >
                                {item.label}
                            </Link>
                        ),
                    )}
                </nav>
                <div className="mt-auto border-border border-t px-2 py-4">
                    {loading ? (
                        <div className="flex h-10 w-full items-center justify-center">
                            <div className="h-4 w-4 animate-spin rounded-full border-2 border-muted-foreground border-t-transparent" />
                        </div>
                    ) : user?.status ? (
                        <div className="space-y-3">
                            <div className="flex items-center gap-3 rounded-md bg-secondary/50 px-3 py-2">
                                <Avatar className="h-8 w-8">
                                    <AvatarImage alt="User avatar" src={getAvatarSkinId(user)} />
                                    <AvatarFallback className="bg-primary/20 text-primary text-xs">{user.status.nickName.slice(0, 1) ?? "E"}</AvatarFallback>
                                </Avatar>
                                <div className="flex-1 overflow-hidden">
                                    <p className="truncate font-medium text-foreground text-sm">{user.status.nickName}</p>
                                    <p className="text-muted-foreground text-xs">Level {user.status.level}</p>
                                </div>
                            </div>
                            <Button
                                className="w-full justify-start text-destructive hover:bg-destructive/10 hover:text-destructive"
                                onClick={() => {
                                    logout();
                                    setMobileMenuOpen(false);
                                }}
                                variant="outline"
                            >
                                <LogOut className="mr-2 h-4 w-4" />
                                Logout
                            </Button>
                        </div>
                    ) : (
                        <Login />
                    )}
                </div>
            </SheetContent>
        </Sheet>
    );
}
