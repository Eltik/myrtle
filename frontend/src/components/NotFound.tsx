import { Link } from "@tanstack/react-router";
import { Home, Search } from "lucide-react";
import { Button } from "#/components/ui/button";
import { useCommand } from "#/lib/command-context";

export function NotFound() {
    const { open: openCmd } = useCommand();

    return (
        <section className="relative flex min-h-[60vh] flex-col items-center justify-center overflow-hidden py-16 text-center">
            <div className="pointer-events-none absolute inset-0 z-0" aria-hidden="true">
                <div className="absolute top-1/2 left-1/2 h-87.5 w-150 -translate-x-1/2 -translate-y-1/2 rounded-full opacity-[0.08] blur-[80px]" style={{ background: "radial-gradient(circle, var(--primary) 0%, transparent 70%)" }} />
                <div className="absolute top-1/2 left-1/2 h-50 w-75 -translate-x-1/2 -translate-y-1/2 rounded-full opacity-[0.05] blur-2xl" style={{ background: "radial-gradient(circle, oklch(0.696 0.17 162) 0%, transparent 70%)" }} />
            </div>

            <div className="relative z-10 mx-auto flex max-w-lg flex-col items-center px-4">
                <div className="mb-6 inline-flex size-20 items-center justify-center rounded-2xl border border-border bg-muted/30 shadow-inner">
                    <span className="select-none font-bold font-mono text-[28px] text-primary tracking-tight [text-shadow:0_0_20px_var(--glow-primary)]">404</span>
                </div>

                <h1 className="m-0 mb-3 font-bold font-sans text-[32px] text-foreground leading-none tracking-tight sm:text-[40px]">Lost in the Wastes</h1>

                <p className="m-0 mb-8 font-sans text-[16px] text-muted-foreground leading-[1.6]">The page you are looking for does not exist, has been moved, or is temporarily unavailable.</p>

                <div className="flex flex-col gap-3 sm:flex-row">
                    <Button variant="default" size="lg" render={<Link to="/" />}>
                        <Home className="size-4.5" />
                        Return Home
                    </Button>
                    <Button variant="outline" size="lg" onClick={openCmd}>
                        <Search className="size-4.5" />
                        Search operators
                    </Button>
                </div>
            </div>
        </section>
    );
}
