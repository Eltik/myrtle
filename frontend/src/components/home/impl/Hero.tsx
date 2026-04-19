import { AuthDialog } from "#/components/header/impl/AuthDialog";
import { Button } from "#/components/ui/button";
import { Kbd } from "#/components/ui/kbd";
import { Separator } from "#/components/ui/separator";
import CommandPreview from "./CommandPreview";
import { useHeroTilt } from "./useHeroTilt";

export default function Hero({ onOpenCommand }: { onOpenCommand: () => void }) {
    const tiltRef = useHeroTilt();
    return (
        <section className="hero">
            <div className="hero-bg" aria-hidden="true" />
            <div className="hero-glow" aria-hidden="true" />
            <div className="hero-grid" aria-hidden="true" />

            <div className="relative z-[3] mx-auto grid w-[min(1080px,calc(100%-2rem))] grid-cols-1 items-center gap-10 md:grid-cols-2 md:gap-12">
                <div className="flex flex-col">
                    <div className="mb-5 inline-flex w-max items-center gap-2 rounded-full border border-border bg-muted/60 px-3 py-1.5 pl-2.5">
                        <span className="dot-pulse" aria-hidden="true" />
                        <span className="font-mono text-[11.5px] font-medium leading-none text-muted-foreground">v3 · live backend sync</span>
                        <Separator orientation="vertical" className="h-3.5 bg-white/10" />
                        <a className="cursor-pointer font-sans text-[11.5px] font-medium leading-none text-primary transition-colors hover:text-[oklch(0.85_0.12_25)]" href="https://github.com/Eltik/myrtle" target="_blank" rel="noreferrer">
                            changelog →
                        </a>
                    </div>

                    <h1 className="m-0 mb-[18px] max-w-[14ch] font-sans text-[42px] font-bold leading-[1.04] tracking-[-0.03em] text-foreground md:text-[62px]">
                        The <span className="text-primary [text-shadow:0_0_30px_oklch(0.75_0.15_25/0.35)]">Arknights</span> companion,
                        <br />
                        rebuilt for speed.
                    </h1>

                    <p className="m-0 mb-7 flex max-w-[48ch] flex-wrap items-center gap-1 font-sans text-[17px] leading-[1.55] text-muted-foreground">
                        400+ operators, complete stats, community tier lists, and live roster sync. Hit <Kbd>⌘</Kbd> <Kbd>K</Kbd> to jump anywhere.
                    </p>

                    <div className="mb-8 flex flex-wrap gap-2.5">
                        <Button variant="default" size="lg" onClick={onOpenCommand}>
                            <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
                                <circle cx="11" cy="11" r="8" />
                                <path d="m21 21-4.35-4.35" />
                            </svg>
                            Search operators
                        </Button>
                        <AuthDialog
                            trigger={
                                <Button variant="outline" size="lg">
                                    Link Yostar account
                                    <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
                                        <path d="M5 12h14" />
                                        <path d="m12 5 7 7-7 7" />
                                    </svg>
                                </Button>
                            }
                        />
                    </div>

                    <div className="flex gap-7 border-t border-border pt-5">
                        {[
                            { v: "423", l: "operators" },
                            { v: "58", l: "tier lists" },
                            { v: "1.2M", l: "rosters synced" },
                        ].map((s) => (
                            <div key={s.l} className="flex flex-col gap-1">
                                <b className="font-sans text-2xl font-bold leading-none tracking-tight text-foreground">{s.v}</b>
                                <span className="font-mono text-[11px] font-medium uppercase leading-none tracking-wider text-muted-foreground">{s.l}</span>
                            </div>
                        ))}
                    </div>
                </div>

                <div className="hero-panel" ref={tiltRef} onClick={onOpenCommand} role="button" tabIndex={0} onKeyDown={(e) => e.key === "Enter" && onOpenCommand()}>
                    <CommandPreview />
                </div>
            </div>
        </section>
    );
}
