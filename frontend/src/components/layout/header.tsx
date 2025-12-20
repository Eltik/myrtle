"use client";

import { ChevronDown, Ellipsis, Github, LogOut, Menu, User } from "lucide-react";
import Link from "next/link";
import { useRouter } from "next/router";
import { useCallback, useLayoutEffect, useMemo, useRef, useState } from "react";
import { Accordion, AccordionContent, AccordionItem, AccordionTrigger } from "~/components/ui/shadcn/accordion";
import { Button } from "~/components/ui/shadcn/button";
import { DropdownMenu, DropdownMenuContent, DropdownMenuItem, DropdownMenuSeparator, DropdownMenuTrigger } from "~/components/ui/shadcn/dropdown-menu";
import { Sheet, SheetContent, SheetHeader, SheetTitle, SheetTrigger } from "~/components/ui/shadcn/sheet";
import { useAuth } from "~/hooks/use-auth";
import { Login } from "./login";

type NavItem = {
    label: string | React.ReactNode;
    href: string;
    dropdown?: { label: string; href: string; description: string }[];
};

const navItems: NavItem[] = [
    { label: "Home", href: "/" },
    {
        label: "Operators",
        href: "#",
        dropdown: [
            { label: "Collection", href: "/operators/list", description: "List of all released operators" },
            { label: "Tier List", href: "/operators/tier-list", description: "Operator rankings by class" },
            { label: "Chibi Viewer", href: "/operators/chibis", description: "Arknights chibi viewer" },
        ],
    },
    {
        label: "Tools",
        href: "#",
        dropdown: [
            { label: "Recruitment Calculator", href: "/tools/recruitment", description: "Calculate recruitment probabilities" },
            { label: "DPS Charts", href: "/tools/dps", description: "Display DPS to compare multiple operators" },
            { label: "Squad Randomizer", href: "/tools/squad", description: "Randomize a 12-operator squad" },
        ],
    },
    {
        label: "Events",
        href: "#",
        dropdown: [
            { label: "Current Events", href: "/events/current", description: "Active event guides" },
            { label: "Event Archive", href: "/events/archive", description: "Past event information" },
            { label: "CN Preview", href: "/events/cn-preview", description: "Upcoming content from CN" },
        ],
    },
    {
        label: <Ellipsis />,
        href: "#",
        dropdown: [
            { label: "About", href: "/about", description: "About this project" },
            { label: "Changelog", href: "/changelog", description: "Recent updates" },
            { label: "Terms", href: "/terms", description: "Terms of use" },
        ],
    },
];

function isNavItemActive(item: NavItem, pathname: string): boolean {
    // For direct links (no dropdown), check exact match
    if (!item.dropdown) {
        return item.href === pathname;
    }
    // For dropdown items, check if current path matches any dropdown href
    // Also check if pathname matches the base path (e.g., /operators for /operators?id=...)
    return item.dropdown.some((dropdownItem) => {
        const basePath = dropdownItem.href.split("/").slice(0, 2).join("/"); // e.g., "/operators" from "/operators/list"
        return pathname === dropdownItem.href || pathname.startsWith(`${dropdownItem.href}/`) || pathname === basePath;
    });
}

export function Header() {
    const router = useRouter();
    const { user, loading, logout } = useAuth();
    const [hoveredIndex, setHoveredIndex] = useState<number | null>(null);
    const [hoverStyle, setHoverStyle] = useState({ left: 0, width: 0, opacity: 0 });
    const [activeIconStyle, setActiveIconStyle] = useState({ left: 0, opacity: 0 });
    const [isNavHovered, setIsNavHovered] = useState(false);
    const [activeDropdown, setActiveDropdown] = useState<number | null>(null);
    const [dropdownStyle, setDropdownStyle] = useState({ left: 0, width: 220 });
    const [mobileMenuOpen, setMobileMenuOpen] = useState(false);
    const navRef = useRef<HTMLElement>(null);
    const itemRefs = useRef<(HTMLAnchorElement | HTMLButtonElement | null)[]>([]);
    const timeoutRef = useRef<NodeJS.Timeout | null>(null);

    const activeIndex = useMemo(() => {
        return navItems.findIndex((item) => isNavItemActive(item, router.pathname));
    }, [router.pathname]);

    const updateIndicator = useCallback((index: number, opacity: number) => {
        const target = itemRefs.current[index];
        const navRect = navRef.current?.getBoundingClientRect();
        const targetRect = target?.getBoundingClientRect();

        if (navRect && targetRect) {
            setHoverStyle({
                left: targetRect.left - navRect.left,
                width: targetRect.width,
                opacity,
            });

            // Update dropdown position
            const item = navItems[index];
            if (item?.dropdown) {
                const dropdownWidth = 220;
                setDropdownStyle({
                    left: targetRect.left - navRect.left + targetRect.width / 2 - dropdownWidth / 2,
                    width: dropdownWidth,
                });
            }
        }
    }, []);

    const updateActiveIcon = useCallback((index: number) => {
        const target = itemRefs.current[index];
        const navRect = navRef.current?.getBoundingClientRect();
        const targetRect = target?.getBoundingClientRect();

        if (navRect && targetRect) {
            // Position the icon at the left edge of the nav item, with some padding
            setActiveIconStyle({
                left: targetRect.left - navRect.left + 12,
                opacity: 1,
            });
        }
    }, []);

    useLayoutEffect(() => {
        if (activeIndex >= 0 && hoveredIndex === null) {
            updateIndicator(activeIndex, 1);
        }
    }, [activeIndex, hoveredIndex, updateIndicator]);

    // Update active icon position when active index changes or on initial render
    useLayoutEffect(() => {
        if (activeIndex >= 0) {
            updateActiveIcon(activeIndex);
        } else {
            setActiveIconStyle((prev) => ({ ...prev, opacity: 0 }));
        }
    }, [activeIndex, updateActiveIcon]);

    const handleMouseEnter = useCallback(
        (index: number) => {
            // Clear any pending close timeout
            if (timeoutRef.current) {
                clearTimeout(timeoutRef.current);
                timeoutRef.current = null;
            }

            updateIndicator(index, 1);
            updateActiveIcon(index);
            setHoveredIndex(index);

            if (navItems[index]?.dropdown) {
                setActiveDropdown(index);
            } else {
                setActiveDropdown(null);
            }
        },
        [updateIndicator, updateActiveIcon],
    );

    const handleNavMouseLeave = useCallback(() => {
        timeoutRef.current = setTimeout(() => {
            setHoveredIndex(null);
            setIsNavHovered(false);
            setActiveDropdown(null);
            if (activeIndex >= 0) {
                updateIndicator(activeIndex, 1);
                updateActiveIcon(activeIndex);
            } else {
                setHoverStyle((prev) => ({ ...prev, opacity: 0 }));
                setActiveIconStyle((prev) => ({ ...prev, opacity: 0 }));
            }
        }, 100);
    }, [activeIndex, updateIndicator, updateActiveIcon]);

    const handleDropdownMouseEnter = useCallback(() => {
        if (timeoutRef.current) {
            clearTimeout(timeoutRef.current);
            timeoutRef.current = null;
        }
    }, []);

    const handleDropdownMouseLeave = useCallback(() => {
        timeoutRef.current = setTimeout(() => {
            setHoveredIndex(null);
            setIsNavHovered(false);
            setActiveDropdown(null);
            if (activeIndex >= 0) {
                updateIndicator(activeIndex, 1);
                updateActiveIcon(activeIndex);
            } else {
                setHoverStyle((prev) => ({ ...prev, opacity: 0 }));
                setActiveIconStyle((prev) => ({ ...prev, opacity: 0 }));
            }
        }, 100);
    }, [activeIndex, updateIndicator, updateActiveIcon]);

    const mousePosRef = useRef({ x: 0, y: 0 });
    const glowRef = useRef<HTMLDivElement>(null);

    const handleNavMouseMove = useCallback(
        (e: React.MouseEvent<HTMLElement>) => {
            const rect = navRef.current?.getBoundingClientRect();
            if (rect) {
                mousePosRef.current = {
                    x: e.clientX - rect.left,
                    y: e.clientY - rect.top,
                };
                if (glowRef.current) {
                    glowRef.current.style.background = `radial-gradient(120px circle at ${mousePosRef.current.x}px ${mousePosRef.current.y}px, oklch(0.75 0.15 25 / 0.25) 0%, transparent 65%)`;
                }
            }
            if (!isNavHovered) {
                setIsNavHovered(true);
            }
        },
        [isNavHovered],
    );

    return (
        <header className="fixed top-0 z-50 w-full">
            <div className="relative">
                <div
                    className="-bottom-px absolute inset-x-0 h-px"
                    style={{
                        background: "linear-gradient(90deg, transparent, oklch(0.75 0.15 25 / 0.5), oklch(0.85 0.12 40 / 0.3), oklch(0.75 0.15 25 / 0.5), transparent)",
                    }}
                />
                <div
                    className="-bottom-1 pointer-events-none absolute inset-x-0 h-4 blur-md"
                    style={{
                        background: "linear-gradient(90deg, transparent 10%, oklch(0.75 0.15 25 / 0.15), oklch(0.80 0.10 35 / 0.1), oklch(0.75 0.15 25 / 0.15), transparent 90%)",
                    }}
                />
                <div
                    className="absolute inset-0 backdrop-blur"
                    style={{
                        background: "linear-gradient(180deg, oklch(0.15 0.005 285 / 0.8) 0%, oklch(0.13 0.005 285 / 0.6) 100%)",
                    }}
                />
                <div
                    className="absolute inset-x-0 top-0 h-px"
                    style={{
                        background: "linear-gradient(90deg, transparent, oklch(1 0 0 / 0.05), transparent)",
                    }}
                />
                <div className="relative flex h-14 items-center justify-between px-4">
                    <Link className="flex items-center gap-2" href="/">
                        <div className="flex h-8 w-8 items-center justify-center rounded-lg bg-primary">{/* tbd logo here */}</div>
                        <span className="font-semibold text-base text-foreground">myrtle.moe</span>
                    </Link>

                    <div className="relative">
                        <nav
                            className="group relative hidden items-center gap-0.5 overflow-hidden rounded-full border border-border px-1 py-1 md:flex"
                            onMouseLeave={handleNavMouseLeave}
                            onMouseMove={handleNavMouseMove}
                            ref={navRef}
                            style={{
                                background: "linear-gradient(180deg, oklch(0.18 0.008 285 / 0.5) 0%, oklch(0.14 0.006 285 / 0.4) 100%)",
                                boxShadow: "inset 0 1px 1px oklch(1 0 0 / 0.04), inset 0 -1px 2px oklch(0 0 0 / 0.15), 0 1px 3px oklch(0 0 0 / 0.1), 0 0 1px oklch(0 0 0 / 0.1)",
                            }}
                        >
                            <div
                                className="pointer-events-none absolute inset-0 z-0 rounded-full transition-opacity duration-300"
                                ref={glowRef}
                                style={{
                                    background: "radial-gradient(120px circle at 0px 0px, oklch(0.75 0.15 25 / 0.25) 0%, transparent 65%)",
                                    opacity: isNavHovered ? 1 : 0,
                                }}
                            />
                            <div
                                className="pointer-events-none absolute top-1 bottom-1 z-1 rounded-full"
                                style={{
                                    left: hoverStyle.left,
                                    width: hoverStyle.width,
                                    opacity: hoverStyle.opacity,
                                    transform: hoverStyle.opacity ? "scale(1)" : "scale(0.95)",
                                    transition: "all 300ms cubic-bezier(0.4, 0, 0.2, 1), opacity 200ms ease-out",
                                    background: "linear-gradient(180deg, oklch(0.24 0.025 25 / 0.5) 0%, oklch(0.20 0.02 25 / 0.35) 100%)",
                                    boxShadow: "inset 0 1px 1px oklch(1 0 0 / 0.05), inset 0 -1px 1px oklch(0 0 0 / 0.1)",
                                }}
                            />
                            <div
                                className="pointer-events-none absolute bottom-[3px] z-2 h-px rounded-full"
                                style={{
                                    left: hoverStyle.left + 4,
                                    width: Math.max(0, hoverStyle.width - 8),
                                    opacity: hoverStyle.opacity,
                                    transition: "all 300ms cubic-bezier(0.4, 0, 0.2, 1), opacity 200ms ease-out",
                                    background: "linear-gradient(90deg, oklch(0.75 0.15 25 / 0) 0%, oklch(0.75 0.15 25 / 0.8) 50%, oklch(0.75 0.15 25 / 0) 100%)",
                                    boxShadow: "0 0 6px oklch(0.75 0.15 25 / 0.6), 0 0 2px oklch(0.75 0.15 25 / 0.4)",
                                }}
                            />

                            {/* Active indicator icon - slides to active nav item */}
                            <span
                                className="pointer-events-none absolute top-1/2 z-20 mt-px text-primary/80 text-xs"
                                style={{
                                    left: activeIconStyle.left,
                                    opacity: activeIconStyle.opacity,
                                    transform: "translateY(-50%)",
                                    transition: "left 400ms cubic-bezier(0.4, 0, 0.2, 1), opacity 200ms ease-out",
                                    textShadow: "0 0 8px oklch(0.75 0.15 25 / 0.6)",
                                }}
                            >
                                ◎
                            </span>

                            {navItems.map((item, index) =>
                                item.dropdown ? (
                                    <button
                                        className={`relative z-10 flex items-center gap-1 rounded-full py-1.5 pr-3.5 pl-7 font-medium text-sm transition-colors duration-200 ${hoveredIndex === index || (activeIndex === index && hoveredIndex === null) ? "text-foreground" : "text-muted-foreground"}`}
                                        key={typeof item.label === "string" ? item.label : index}
                                        onMouseEnter={() => handleMouseEnter(index)}
                                        ref={(el) => {
                                            itemRefs.current[index] = el;
                                        }}
                                        type="button"
                                    >
                                        {item.label}
                                        <ChevronDown
                                            className="h-3 w-3 will-change-transform"
                                            style={{
                                                transform: activeDropdown === index ? "rotate(180deg)" : "rotate(0deg)",
                                                transition: "transform 200ms ease-out",
                                            }}
                                        />
                                    </button>
                                ) : (
                                    <Link
                                        className={`relative z-10 flex items-center gap-1.5 rounded-full py-1.5 pr-3.5 pl-7 font-medium text-sm transition-colors duration-200 ${hoveredIndex === index || (activeIndex === index && hoveredIndex === null) ? "text-foreground" : "text-muted-foreground"}`}
                                        href={item.href}
                                        key={typeof item.label === "string" ? item.label : index}
                                        onMouseEnter={() => handleMouseEnter(index)}
                                        ref={(el) => {
                                            itemRefs.current[index] = el;
                                        }}
                                    >
                                        {item.label}
                                    </Link>
                                ),
                            )}
                        </nav>

                        {/* Dropdown container - GPU accelerated */}
                        <div
                            className="absolute top-full pt-2 will-change-transform"
                            onMouseEnter={handleDropdownMouseEnter}
                            onMouseLeave={handleDropdownMouseLeave}
                            role="menu"
                            style={{
                                transform: `translateX(${dropdownStyle.left}px) ${activeDropdown !== null ? "translateY(0)" : "translateY(-8px)"}`,
                                width: dropdownStyle.width,
                                opacity: activeDropdown !== null ? 1 : 0,
                                pointerEvents: activeDropdown !== null ? "auto" : "none",
                                transition: "transform 200ms ease-out, opacity 150ms ease-out",
                            }}
                        >
                            <div
                                className="overflow-hidden rounded-lg border border-border shadow-lg backdrop-blur-sm"
                                style={{
                                    background: "linear-gradient(180deg, oklch(0.18 0.008 285 / 0.95) 0%, oklch(0.14 0.006 285 / 0.95) 100%)",
                                }}
                            >
                                {navItems.map(
                                    (item, index) =>
                                        item.dropdown && (
                                            <div
                                                className="will-change-transform"
                                                key={typeof item.label === "string" ? item.label : index}
                                                style={{
                                                    display: activeDropdown === index ? "block" : "none",
                                                }}
                                            >
                                                <div className="p-1.5">
                                                    {item.dropdown.map((dropdownItem) => (
                                                        <Link className="group/item flex flex-col gap-0.5 rounded-md px-3 py-2 transition-colors hover:bg-secondary" href={dropdownItem.href} key={dropdownItem.label}>
                                                            <span className="font-medium text-foreground text-sm">{dropdownItem.label}</span>
                                                            <span className="text-muted-foreground text-xs">{dropdownItem.description}</span>
                                                        </Link>
                                                    ))}
                                                </div>
                                            </div>
                                        ),
                                )}
                            </div>
                        </div>
                    </div>
                    <div className="flex items-center gap-2">
                        {/* Mobile Menu Button - Only visible on mobile */}
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
                                                    <AccordionTrigger className={`rounded-md px-3 py-2.5 hover:no-underline ${isNavItemActive(item, router.pathname) ? "bg-primary/10 text-foreground" : "text-muted-foreground hover:bg-secondary hover:text-foreground"}`}>
                                                        <span className="font-medium">{item.label}</span>
                                                    </AccordionTrigger>
                                                    <AccordionContent className="pt-1 pb-1">
                                                        <div className="ml-3 flex flex-col gap-0.5 border-border border-l pl-3">
                                                            {item.dropdown.map((dropdownItem) => (
                                                                <Link
                                                                    className={`flex flex-col gap-0.5 rounded-md px-3 py-2 transition-colors ${router.pathname === dropdownItem.href ? "bg-primary/10 text-foreground" : "text-muted-foreground hover:bg-secondary hover:text-foreground"}`}
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
                                                className={`rounded-md px-3 py-2.5 font-medium transition-colors ${isNavItemActive(item, router.pathname) ? "bg-primary/10 text-foreground" : "text-muted-foreground hover:bg-secondary hover:text-foreground"}`}
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
                                                <div className="flex h-8 w-8 items-center justify-center rounded-full bg-primary/20">
                                                    <User className="h-4 w-4 text-primary" />
                                                </div>
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

                        <Button asChild className="h-8 w-8" size="icon" variant="ghost">
                            <Link href="https://github.com" rel="noopener noreferrer" target="_blank">
                                <Github className="h-4 w-4" />
                                <span className="sr-only">GitHub</span>
                            </Link>
                        </Button>
                        <div className="hidden md:block">
                            {loading ? (
                                <div className="flex h-8 w-8 items-center justify-center">
                                    <div className="h-4 w-4 animate-spin rounded-full border-2 border-muted-foreground border-t-transparent" />
                                </div>
                            ) : user?.status ? (
                                <DropdownMenu>
                                    <DropdownMenuTrigger asChild>
                                        <Button className="flex h-8 items-center gap-2 rounded-md border border-border bg-transparent px-3 text-foreground text-sm transition-colors hover:bg-secondary" variant="ghost">
                                            <User className="h-3.5 w-3.5" />
                                            <span className="max-w-24 truncate font-medium">{user.status.nickName}</span>
                                            <ChevronDown className="h-3 w-3" />
                                        </Button>
                                    </DropdownMenuTrigger>
                                    <DropdownMenuContent align="end" className="w-48">
                                        <div className="px-2 py-1.5">
                                            <p className="font-medium text-sm">{user.status.nickName}</p>
                                            <p className="text-muted-foreground text-xs">Level {user.status.level}</p>
                                        </div>
                                        <DropdownMenuSeparator />
                                        <DropdownMenuItem className="text-primary transition-colors focus:text-primary/80" onClick={logout}>
                                            <LogOut className="mr-2 h-4 w-4" />
                                            Logout
                                        </DropdownMenuItem>
                                    </DropdownMenuContent>
                                </DropdownMenu>
                            ) : (
                                <Login variant="header" />
                            )}
                        </div>
                    </div>
                </div>
            </div>
        </header>
    );
}
