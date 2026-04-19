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

            <div className="hero-inner">
                <div className="hero-copy">
                    <div className="status-pill">
                        <span className="dot-pulse" aria-hidden="true" />
                        <span className="small-mono">v3 · live backend sync</span>
                        <Separator orientation="vertical" className="h-3.5 bg-white/10" />
                        <a className="pill-link" href="https://github.com/Eltik/myrtle" target="_blank" rel="noreferrer">
                            changelog →
                        </a>
                    </div>

                    <h1 className="hero-display">
                        The <span className="hero-accent">Arknights</span> companion,
                        <br />
                        rebuilt for speed.
                    </h1>

                    <p className="hero-lead">
                        400+ operators, complete stats, community tier lists, and live roster sync. Hit <Kbd>⌘</Kbd> <Kbd>K</Kbd> to jump anywhere.
                    </p>

                    <div className="hero-actions">
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

                    <div className="hero-stats">
                        <div>
                            <b>423</b>
                            <span>operators</span>
                        </div>
                        <div>
                            <b>58</b>
                            <span>tier lists</span>
                        </div>
                        <div>
                            <b>1.2M</b>
                            <span>rosters synced</span>
                        </div>
                    </div>
                </div>

                <div className="hero-panel" ref={tiltRef} onClick={onOpenCommand} role="button" tabIndex={0} onKeyDown={(e) => e.key === "Enter" && onOpenCommand()}>
                    <CommandPreview />
                </div>
            </div>
        </section>
    );
}
