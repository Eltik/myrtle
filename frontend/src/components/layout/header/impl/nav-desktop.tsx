"use client";

import { ChevronDown, ExternalLink } from "lucide-react";
import Link from "next/link";
import { useCallback, useLayoutEffect, useMemo, useRef, useState } from "react";
import { navItems } from "./constants";
import { isNavItemActive } from "./helpers";

interface NavDesktopProps {
    pathname: string;
}

export function NavDesktop({ pathname }: NavDesktopProps) {
    const [hoveredIndex, setHoveredIndex] = useState<number | null>(null);
    const [hoverStyle, setHoverStyle] = useState({ left: 0, width: 0, opacity: 0 });
    const [activeIconStyle, setActiveIconStyle] = useState({ left: 0, opacity: 0 });
    const [isNavHovered, setIsNavHovered] = useState(false);
    const [activeDropdown, setActiveDropdown] = useState<number | null>(null);
    const [dropdownStyle, setDropdownStyle] = useState({ left: 0, width: 220 });
    const navRef = useRef<HTMLElement>(null);
    const itemRefs = useRef<(HTMLAnchorElement | HTMLButtonElement | null)[]>([]);
    const timeoutRef = useRef<NodeJS.Timeout | null>(null);
    const mousePosRef = useRef({ x: 0, y: 0 });
    const glowRef = useRef<HTMLDivElement>(null);

    const activeIndex = useMemo(() => {
        return navItems.findIndex((item) => isNavItemActive(item, pathname));
    }, [pathname]);

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

    useLayoutEffect(() => {
        if (activeIndex >= 0) {
            updateActiveIcon(activeIndex);
        } else {
            setActiveIconStyle((prev) => ({ ...prev, opacity: 0 }));
        }
    }, [activeIndex, updateActiveIcon]);

    const handleMouseEnter = useCallback(
        (index: number) => {
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

    const handleNavMouseMove = useCallback(
        (e: React.MouseEvent<HTMLElement>) => {
            const rect = navRef.current?.getBoundingClientRect();
            if (rect) {
                mousePosRef.current = {
                    x: e.clientX - rect.left,
                    y: e.clientY - rect.top,
                };
                if (glowRef.current) {
                    glowRef.current.style.background = `radial-gradient(120px circle at ${mousePosRef.current.x}px ${mousePosRef.current.y}px, var(--glow-nav) 0%, transparent 65%)`;
                }
            }
            if (!isNavHovered) {
                setIsNavHovered(true);
            }
        },
        [isNavHovered],
    );

    return (
        <div className="relative">
            <nav className="nav-pill-bg group relative hidden items-center gap-0.5 overflow-hidden rounded-full border border-border px-1 py-1 md:flex" onMouseLeave={handleNavMouseLeave} onMouseMove={handleNavMouseMove} ref={navRef}>
                <div
                    className="pointer-events-none absolute inset-0 z-0 rounded-full transition-opacity duration-300"
                    ref={glowRef}
                    style={{
                        background: "radial-gradient(120px circle at 0px 0px, var(--glow-nav) 0%, transparent 65%)",
                        opacity: isNavHovered ? 1 : 0,
                    }}
                />
                <div
                    className="nav-pill-hover pointer-events-none absolute top-1 bottom-1 z-1 rounded-full"
                    style={{
                        left: hoverStyle.left,
                        width: hoverStyle.width,
                        opacity: hoverStyle.opacity,
                        transform: hoverStyle.opacity ? "scale(1)" : "scale(0.95)",
                        transition: "all 300ms cubic-bezier(0.4, 0, 0.2, 1), opacity 200ms ease-out",
                    }}
                />
                <div
                    className="pointer-events-none absolute bottom-0.75 z-2 h-px rounded-full"
                    style={{
                        left: hoverStyle.left + 4,
                        width: Math.max(0, hoverStyle.width - 8),
                        opacity: hoverStyle.opacity,
                        transition: "all 300ms cubic-bezier(0.4, 0, 0.2, 1), opacity 200ms ease-out",
                        background: "linear-gradient(90deg, transparent 0%, var(--glow-nav-line-center) 50%, transparent 100%)",
                        boxShadow: "0 0 6px var(--glow-nav-line-shadow), 0 0 2px var(--glow-nav-line-shadow-soft)",
                    }}
                />

                <span
                    className="pointer-events-none absolute top-1/2 z-20 mt-px text-primary/80 text-xs"
                    style={{
                        left: activeIconStyle.left,
                        opacity: activeIconStyle.opacity,
                        transform: "translateY(-50%)",
                        transition: "left 400ms cubic-bezier(0.4, 0, 0.2, 1), opacity 200ms ease-out",
                        textShadow: "0 0 8px var(--glow-text-icon)",
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

            {/* Dropdown container */}
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
                <div className="nav-dropdown-bg overflow-hidden rounded-lg border border-border shadow-lg backdrop-blur-sm">
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
                                                <span className="flex items-center gap-1.5 font-medium text-foreground text-sm">
                                                    {dropdownItem.label}
                                                    {dropdownItem.external && <ExternalLink className="h-3 w-3 text-muted-foreground" />}
                                                </span>
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
    );
}
