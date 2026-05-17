import { Link } from "@tanstack/react-router";

export default function Footer() {
    return (
        <footer className="border-border border-t bg-[color-mix(in_srgb,var(--background)_84%,transparent)] py-7">
            <div className="mx-auto flex w-[min(1080px,calc(100%-2rem))] flex-col gap-2.5">
                <div className="flex flex-wrap items-center justify-between gap-3">
                    <div className="inline-flex items-center gap-2.5 font-sans font-semibold text-foreground text-sm leading-none">
                        <img src="/logo/bust_transparent.png" alt="" width={22} height={22} className="h-5.5 w-5.5 shrink-0 object-contain" />
                        <span>
                            myrtle.moe <span className="font-normal text-muted-foreground">· v3</span>
                        </span>
                    </div>
                    <nav aria-label="Legal" className="inline-flex items-center gap-4">
                        <Link to="/terms" className="font-sans text-[12.5px] text-muted-foreground leading-none transition-colors hover:text-foreground">
                            Terms
                        </Link>
                        <Link to="/privacy" className="font-sans text-[12.5px] text-muted-foreground leading-none transition-colors hover:text-foreground">
                            Privacy
                        </Link>
                    </nav>
                </div>
                <span className="max-w-130 font-sans text-muted-foreground text-xs leading-normal">Not affiliated with Hypergryph or Yostar. Game data and assets are property of their respective owners.</span>
                <span className="font-medium font-mono text-[11px] text-muted-foreground leading-none tracking-wide opacity-70">built on TanStack Start · COSS UI</span>
            </div>
        </footer>
    );
}
