"use client";

import Link from "next/link";
import { Button } from "~/components/ui/button";
import { Github, User } from "lucide-react";
import { useState, useRef, useLayoutEffect, useCallback } from "react";

const navItems = [
    { label: "Home", href: "/", icon: "◎", isActive: true },
    { label: "Operators", href: "#" },
    { label: "Planner", href: "#" },
    { label: "Events", href: "#" },
    { label: "More", href: "#" },
];

export function Header() {
    const [hoveredIndex, setHoveredIndex] = useState<number | null>(null);
    const [hoverStyle, setHoverStyle] = useState({ left: 0, width: 0, opacity: 0 });
    const [mousePosition, setMousePosition] = useState({ x: 0, y: 0 });
    const [isNavHovered, setIsNavHovered] = useState(false);
    const navRef = useRef<HTMLElement>(null);
    const itemRefs = useRef<(HTMLAnchorElement | null)[]>([]);

    const activeIndex = navItems.findIndex((item) => item.isActive);

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
        }
    }, []);

    useLayoutEffect(() => {
        if (activeIndex >= 0 && hoveredIndex === null) {
            updateIndicator(activeIndex, 1);
        }
    }, [activeIndex, hoveredIndex, updateIndicator]);

    const handleMouseEnter = (index: number) => {
        updateIndicator(index, 1);
        setHoveredIndex(index);
    };

    const handleMouseLeave = () => {
        setHoveredIndex(null);
        setIsNavHovered(false);
        if (activeIndex >= 0) {
            updateIndicator(activeIndex, 1);
        } else {
            setHoverStyle((prev) => ({ ...prev, opacity: 0 }));
        }
    };

    const handleNavMouseMove = (e: React.MouseEvent<HTMLElement>) => {
        const rect = navRef.current?.getBoundingClientRect();
        if (rect) {
            setMousePosition({
                x: e.clientX - rect.left,
                y: e.clientY - rect.top,
            });
        }
        setIsNavHovered(true);
    };

    return (
        <header className="fixed top-0 z-50 w-full">
            <div className="relative">
                <div
                    className="absolute inset-x-0 -bottom-px h-px"
                    style={{
                        background: "linear-gradient(90deg, transparent, oklch(0.75 0.15 25 / 0.5), oklch(0.85 0.12 40 / 0.3), oklch(0.75 0.15 25 / 0.5), transparent)",
                    }}
                />
                <div
                    className="absolute inset-x-0 -bottom-1 h-4 blur-md pointer-events-none"
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
                    <Link href="/" className="flex items-center gap-2">
                        <div className="flex h-8 w-8 items-center justify-center rounded-lg bg-primary">{/* tbd logo here */}</div>
                        <span className="text-base font-semibold text-foreground">myrtle.moe</span>
                    </Link>

                    <nav
                        ref={navRef}
                        className="group relative hidden md:flex items-center gap-0.5 rounded-full border border-border px-1 py-1 overflow-hidden"
                        style={{
                            background: "linear-gradient(180deg, oklch(0.18 0.008 285 / 0.5) 0%, oklch(0.14 0.006 285 / 0.4) 100%)",
                            boxShadow: "inset 0 1px 1px oklch(1 0 0 / 0.04), inset 0 -1px 2px oklch(0 0 0 / 0.15), 0 1px 3px oklch(0 0 0 / 0.1), 0 0 1px oklch(0 0 0 / 0.1)",
                        }}
                        onMouseMove={handleNavMouseMove}
                        onMouseLeave={handleMouseLeave}
                    >
                        <div
                            className="pointer-events-none absolute inset-0 z-0 rounded-full transition-opacity duration-300"
                            style={{
                                background: `radial-gradient(120px circle at ${mousePosition.x}px ${mousePosition.y}px, oklch(0.75 0.15 25 / 0.25) 0%, transparent 65%)`,
                                opacity: isNavHovered ? 1 : 0,
                            }}
                        />
                        <div
                            className="absolute top-1 bottom-1 rounded-full pointer-events-none z-1"
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
                            className="absolute bottom-[3px] h-px rounded-full pointer-events-none z-2"
                            style={{
                                left: hoverStyle.left + 4,
                                width: Math.max(0, hoverStyle.width - 8),
                                opacity: hoverStyle.opacity,
                                transition: "all 300ms cubic-bezier(0.4, 0, 0.2, 1), opacity 200ms ease-out",
                                background: "linear-gradient(90deg, oklch(0.75 0.15 25 / 0) 0%, oklch(0.75 0.15 25 / 0.8) 50%, oklch(0.75 0.15 25 / 0) 100%)",
                                boxShadow: "0 0 6px oklch(0.75 0.15 25 / 0.6), 0 0 2px oklch(0.75 0.15 25 / 0.4)",
                            }}
                        />

                        {navItems.map((item, index) => (
                            <Link
                                key={item.label}
                                ref={(el) => {
                                    itemRefs.current[index] = el;
                                }}
                                href={item.href}
                                onMouseEnter={() => handleMouseEnter(index)}
                                className={`relative z-10 flex items-center gap-1.5 rounded-full px-3.5 py-1.5 text-sm font-medium transition-colors duration-200 ${hoveredIndex === index || (item.isActive && hoveredIndex === null) ? "text-foreground" : "text-muted-foreground"}`}
                            >
                                {item.icon && <span className="text-xs opacity-60">{item.icon}</span>}
                                {item.label}
                            </Link>
                        ))}
                    </nav>
                    <div className="flex items-center gap-2">
                        <Button variant="ghost" size="icon" className="h-8 w-8" asChild>
                            <Link href="https://github.com" target="_blank" rel="noopener noreferrer">
                                <Github className="h-4 w-4" />
                                <span className="sr-only">GitHub</span>
                            </Link>
                        </Button>
                        <Button variant="outline" size="sm" className="h-8 gap-2 bg-transparent">
                            <User className="h-3.5 w-3.5" />
                            Login
                        </Button>
                    </div>
                </div>
            </div>
        </header>
    );
}
