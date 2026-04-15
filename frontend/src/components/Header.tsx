import { Link } from "@tanstack/react-router";
import { Button } from "#/components/ui/button";
import { Separator } from "#/components/ui/separator";
import { MainNav, type NavItem } from "./MainNav";
import { MobileNav } from "./MobileNav";
import ThemeToggle from "./ThemeToggle";

const navItems: NavItem[] = [
    { href: "/", label: "Home" },
    { href: "/operators", label: "Operators" },
    { href: "/tools", label: "Tools" },
    { href: "/about", label: "About" },
];

const toolItems: NavItem[] = [
    { href: "/recruitment", label: "Recruitment Calculator" },
    { href: "/tier-lists", label: "Tier Lists" },
    { href: "/dps", label: "DPS Charts" },
    { href: "/randomizer", label: "Randomizer" },
];

export default function Header() {
    return (
        <header className="sticky top-0 z-50 w-full border-b border-border bg-background/95 backdrop-blur-md supports-[backdrop-filter]:bg-background/60">
            <div className="page-wrap flex h-14 items-center gap-4 px-4 sm:h-16">
                <MobileNav items={navItems} toolItems={toolItems} />
                <Link to="/" className="flex items-center gap-2 text-foreground no-underline">
                    <span className="flex h-7 w-7 items-center justify-center rounded-full bg-gradient-to-br from-primary to-[var(--lagoon)]">
                        <svg viewBox="0 0 24 24" className="h-3.5 w-3.5 text-white" fill="currentColor">
                            <title>myrtle.moe logo icon</title>
                            <path d="M12 2L2 7l10 5 10-5-10-5zM2 17l10 5 10-5M2 12l10 5 10-5" />
                        </svg>
                    </span>
                    <span className="font-semibold tracking-tight">myrtle.moe</span>
                </Link>
                <MainNav items={navItems} className="ml-6" />
                <div className="flex-1" />
                <div className="flex items-center gap-2">
                    <ThemeToggle />

                    <Separator orientation="vertical" className="mx-1 hidden h-5 sm:block" />

                    <Button
                        variant="ghost"
                        size="icon"
                        className="hidden sm:inline-flex"
                        render={
                            <a href="https://github.com/myrtle-moe" target="_blank" rel="noreferrer" aria-label="GitHub">
                                <span className="sr-only">GitHub</span>
                                <svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round" className="lucide lucide-github h-4 w-4" aria-hidden="true">
                                    <path d="M15 22v-4a4.8 4.8 0 0 0-1-3.5c3 0 6-2 6-5.5.08-1.25-.27-2.48-1-3.5.28-1.15.28-2.35 0-3.5 0 0-1 0-3 1.5-2.64-.5-5.36-.5-8 0C6 2 5 2 5 2c-.3 1.15-.3 2.35 0 3.5A5.403 5.403 0 0 0 4 9c0 3.5 3 5.5 6 5.5-.39.49-.68 1.05-.85 1.65-.17.6-.22 1.23-.15 1.85v4" />
                                    <path d="M9 18c-4.51 2-5-2-7-2" />
                                </svg>
                            </a>
                        }
                    />

                    <Button size="sm" className="hidden sm:inline-flex">
                        <Link to="/login" className="text-inherit no-underline">
                            Sign In
                        </Link>
                    </Button>
                </div>
            </div>
        </header>
    );
}
