"use client";

import { Github } from "lucide-react";
import Image from "next/image";
import Link from "next/link";
import { useRouter } from "next/router";
import { useState } from "react";
import { Button } from "~/components/ui/shadcn/button";
import { useAuth } from "~/hooks/use-auth";
import { NavDesktop } from "./impl/nav-desktop";
import { NavMobile } from "./impl/nav-mobile";
import { ThemeToggle } from "./impl/theme-toggle";
import { UserMenu } from "./impl/user-menu";

export function Header() {
    const router = useRouter();
    const { user, loading, logout } = useAuth();
    const [mobileMenuOpen, setMobileMenuOpen] = useState(false);

    return (
        <header className="fixed top-0 z-50 w-full">
            <div className="relative">
                <div
                    className="absolute inset-x-0 -bottom-px h-px"
                    style={{
                        background: "linear-gradient(90deg, transparent, var(--glow-header-border), var(--glow-header-border-secondary), var(--glow-header-border), transparent)",
                    }}
                />
                <div
                    className="pointer-events-none absolute inset-x-0 -bottom-1 h-4 blur-md"
                    style={{
                        background: "linear-gradient(90deg, transparent 10%, var(--glow-header), var(--glow-header-secondary), var(--glow-header), transparent 90%)",
                    }}
                />
                <div className="header-backdrop absolute inset-0 backdrop-blur" />
                <div className="header-top-line absolute inset-x-0 top-0 h-px" />
                <div className="relative flex h-14 items-center justify-between px-4">
                    <Link className="flex items-center gap-2" href="/">
                        <div className="flex h-8 w-8 items-center justify-center rounded-lg bg-primary">
                            <Image alt="Logo" height={32} src="/logo/bust_transparent.png" width={32} />
                        </div>
                        <span className="font-semibold text-base text-foreground">myrtle.moe</span>
                    </Link>

                    <NavDesktop pathname={router.pathname} />

                    <div className="flex items-center gap-2">
                        <NavMobile loading={loading} logout={logout} mobileMenuOpen={mobileMenuOpen} pathname={router.pathname} setMobileMenuOpen={setMobileMenuOpen} user={user} />

                        <Button asChild className="h-8 w-8" size="icon" variant="ghost">
                            <Link href="https://github.com" rel="noopener noreferrer" target="_blank">
                                <Github className="h-4 w-4" />
                                <span className="sr-only">GitHub</span>
                            </Link>
                        </Button>
                        <ThemeToggle />
                        <div className="hidden md:block">
                            <UserMenu loading={loading} logout={logout} user={user} />
                        </div>
                    </div>
                </div>
            </div>
        </header>
    );
}
