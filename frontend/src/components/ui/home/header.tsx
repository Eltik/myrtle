"use client";

import type React from "react";

import Link from "next/link";
import { Button } from "~/components/ui/button";
import { Github, User } from "lucide-react";
import { useState, useRef } from "react";

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
    const navRef = useRef<HTMLElement>(null);

    const handleMouseEnter = (e: React.MouseEvent<HTMLAnchorElement>, index: number) => {
        const target = e.currentTarget;
        const navRect = navRef.current?.getBoundingClientRect();
        const targetRect = target.getBoundingClientRect();

        if (navRect) {
            setHoverStyle({
                left: targetRect.left - navRect.left,
                width: targetRect.width,
                opacity: 1,
            });
        }
        setHoveredIndex(index);
    };

    const handleMouseLeave = () => {
        setHoveredIndex(null);
        setHoverStyle((prev) => ({ ...prev, opacity: 0 }));
    };

    return (
        <header className="sticky top-0 z-50 border-b border-border bg-background/80 backdrop-blur-sm">
            <div className="flex h-14 items-center justify-between px-4">
                {/* Left: Logo */}
                <Link href="/" className="flex items-center gap-2">
                    <div className="flex h-8 w-8 items-center justify-center rounded-lg bg-primary">
                        {/* logo here */}
                    </div>
                    <span className="text-base font-semibold text-foreground">myrtle.moe</span>
                </Link>

                {/* Center: Pill-shaped navigation with sliding highlight */}
                <nav ref={navRef} className="relative hidden md:flex items-center gap-1 rounded-full border border-border bg-card/50 px-1.5 py-1" onMouseLeave={handleMouseLeave}>
                    <div
                        className="absolute top-1 bottom-1 rounded-full bg-secondary transition-all duration-200 ease-out"
                        style={{
                            left: hoverStyle.left,
                            width: hoverStyle.width,
                            opacity: hoverStyle.opacity,
                        }}
                    />

                    {navItems.map((item, index) => (
                        <Link
                            key={item.label}
                            href={item.href}
                            onMouseEnter={(e) => handleMouseEnter(e, index)}
                            className={`relative z-10 flex items-center gap-1.5 rounded-full px-3 py-1.5 text-sm font-medium transition-colors ${
                                item.isActive && hoveredIndex === null ? "bg-secondary text-foreground" : hoveredIndex === index ? "text-foreground" : "text-muted-foreground hover:text-foreground"
                            } ${item.isActive && hoveredIndex !== null ? "bg-transparent" : ""}`}
                        >
                            {item.icon && <span className="text-xs">{item.icon}</span>}
                            {item.label}
                        </Link>
                    ))}
                </nav>

                {/* Right: Actions */}
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
        </header>
    );
}
