import { useQuery } from "@tanstack/react-query";
import { AuthDialog } from "#/components/header/impl/AuthDialog";
import { Button } from "#/components/ui/button";
import { Kbd } from "#/components/ui/kbd";
import { Separator } from "#/components/ui/separator";
import { useIsMac } from "#/hooks/use-is-mac";
import { statsQueryOptions } from "#/lib/api/stats";
import { formatNumberCompact } from "#/lib/utils";
import CommandPreview from "./CommandPreview";
import styles from "./Hero.module.css";
import { useHeroTilt } from "./useHeroTilt";

export default function Hero({ onOpenCommand }: { onOpenCommand: () => void }) {
    const tiltRef = useHeroTilt();
    const { data: stats } = useQuery(statsQueryOptions());

    const heroStats = [
        { v: stats ? stats.gameData.operators.toString() : "-", l: "operators" },
        { v: stats ? stats.tierLists.active.toString() : "-", l: "tier lists" },
        { v: stats ? formatNumberCompact(stats.rosters.total) : "-", l: "rosters synced" },
    ];

    const isMac = useIsMac();
    return (
        <section className={styles.hero}>
            <div className={styles.heroAmbient} aria-hidden="true" />
            <div className={styles.heroBg} aria-hidden="true" />
            <div className={styles.heroGlow} aria-hidden="true" />
            <div className={styles.heroGrid} aria-hidden="true" />

            <div className="relative z-3 mx-auto grid w-[min(1080px,calc(100%-2rem))] grid-cols-1 items-center gap-10 md:grid-cols-2 md:gap-12">
                <div className="flex flex-col">
                    <div className="mb-5 inline-flex w-max items-center gap-2 rounded-full border border-border bg-muted/60 px-3 py-1.5 pl-2.5">
                        <span className={styles.dotPulse} aria-hidden="true" />
                        <span className="font-medium font-mono text-[11.5px] text-muted-foreground leading-none">v3</span>
                        <Separator orientation="vertical" className="h-3.5 bg-border" />
                        <a href="/changelog" className="cursor-pointer font-medium font-sans text-[11.5px] text-primary leading-none transition-colors hover:text-[color-mix(in_oklab,var(--primary),white_40%)]" rel="noreferrer">
                            changelog →
                        </a>
                    </div>

                    <h1 className="m-0 mb-4.5 max-w-[14ch] font-bold font-sans text-[42px] text-foreground leading-[1.04] tracking-[-0.03em] md:text-[62px]">
                        The <span className="text-primary [text-shadow:0_0_30px_var(--glow-primary)]">Arknights</span> companion.
                    </h1>

                    <p className="m-0 mb-7 flex max-w-[48ch] flex-wrap items-center gap-1 font-sans text-[17px] text-muted-foreground leading-[1.55]">
                        400+ operators, complete stats, community tier lists, and live roster sync. <br />
                        <span className="flex flex-row items-center gap-2">
                            Hit{" "}
                            <span>
                                <Kbd>{isMac ? "⌘" : "Ctrl"}</Kbd> <Kbd>K</Kbd>
                            </span>{" "}
                            to jump anywhere.
                        </span>
                    </p>

                    <div className="mb-8 flex flex-wrap gap-2.5">
                        <Button variant="default" size="lg" onClick={onOpenCommand}>
                            <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round" role="image" aria-label="Search">
                                <circle cx="11" cy="11" r="8" />
                                <path d="m21 21-4.35-4.35" />
                            </svg>
                            Search operators
                        </Button>
                        <AuthDialog
                            trigger={
                                <Button variant="outline" size="lg">
                                    Link Yostar account
                                    <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round" role="image" aria-label="Right arrow">
                                        <path d="M5 12h14" />
                                        <path d="m12 5 7 7-7 7" />
                                    </svg>
                                </Button>
                            }
                        />
                    </div>

                    <div className="flex gap-7 border-border border-t pt-5">
                        {heroStats.map((s) => (
                            <div key={s.l} className="flex flex-col gap-1">
                                <b className="font-bold font-sans text-2xl text-foreground leading-none tracking-tight">{s.v}</b>
                                <span className="font-medium font-mono text-[11px] text-muted-foreground uppercase leading-none tracking-wider">{s.l}</span>
                            </div>
                        ))}
                    </div>
                </div>

                <div className={styles.heroPanel} ref={tiltRef}>
                    <CommandPreview onOpenCommand={onOpenCommand} />
                </div>
            </div>
        </section>
    );
}
