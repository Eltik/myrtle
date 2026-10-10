import { Kbd } from "#/components/ui/kbd";
import { useIsMac } from "#/hooks/use-is-mac";
import { type TypedRichT, useRichT, useT } from "#/lib/i18n";
import type { TypedT } from "#/lib/i18n/messages";
import type { messages } from "./Hero.messages";
import styles from "./Hero.module.css";
import { prefersReducedMotion, useParallax, useReveal } from "./motion";
import shared from "./shared.module.css";

/** The intro, then the key hint, reveal a beat after Myrtle. */
const INTRO_REVEAL_DELAY_MS = 80;
const HINT_REVEAL_DELAY_MS = 240;

export default function Hero({ onOpenCommand, scrollTargetId }: { onOpenCommand: () => void; scrollTargetId: string }) {
    const t: TypedT<typeof messages> = useT("home");
    const rt: TypedRichT<typeof messages> = useRichT("home");
    const isMac = useIsMac();
    const artRef = useParallax<HTMLDivElement>();
    const revealMyrtle = useReveal<HTMLDivElement>();
    const revealIntro = useReveal<HTMLDivElement>(INTRO_REVEAL_DELAY_MS);
    const revealHint = useReveal<HTMLButtonElement>(HINT_REVEAL_DELAY_MS);

    const scrollToTarget = (): void => {
        const target = document.getElementById(scrollTargetId);
        if (!target) return;
        // Measured at click time so the target lands flush under the sticky header at
        // whatever height `--site-header-height` gives it (57px narrow, 65px from 640px).
        const headerHeight = document.querySelector("header")?.offsetHeight ?? 0;
        window.scrollTo({ top: target.getBoundingClientRect().top + window.scrollY - headerHeight, behavior: prefersReducedMotion() ? "auto" : "smooth" });
    };

    return (
        <section className={styles.hero}>
            <div className={styles.art} aria-hidden="true">
                <div ref={artRef} className={styles.artLayer}>
                    <img src="/home/hero_art.jpg" alt="" width={2400} height={1350} fetchPriority="high" decoding="async" />
                </div>
                <div className={styles.artFade} />
            </div>
            <div className={styles.spacer} />

            <div className={`${shared.container} ${styles.body}`}>
                <div {...revealMyrtle} className={`${shared.reveal} ${styles.myrtle}`}>
                    <div className={styles.myrtleGlow} aria-hidden="true" />
                    <img src="/home/myrtle_sleepy.png" alt={t("hero.myrtleAlt")} width={300} height={300} className={styles.myrtleImg} />
                </div>
                <div className={styles.copy}>
                    <div {...revealIntro} className={`${shared.reveal} ${styles.intro}`}>
                        <p className={styles.eyebrow}>{t("hero.eyebrow")}</p>
                        <h1 className={styles.title}>{t("hero.title")}</h1>
                        <p className={styles.lead}>{t("hero.lead")}</p>
                    </div>
                    <button type="button" {...revealHint} className={`${shared.reveal} ${styles.hint}`} onClick={onOpenCommand}>
                        {rt("hero.hint", {
                            keys: (
                                <span className="inline-flex items-center gap-1.5">
                                    <Kbd>{isMac ? "⌘" : "Ctrl"}</Kbd>
                                    <Kbd>K</Kbd>
                                </span>
                            ),
                        })}
                    </button>
                </div>
            </div>

            <button type="button" className={styles.cue} onClick={scrollToTarget}>
                <img src="/home/myrtle_happy.png" alt="" width={52} height={52} className={styles.cueImg} />
                <span className={styles.cueText}>
                    <span className={styles.cueTitle}>{t("hero.cue.title")}</span>
                    <span className={styles.cueSub}>
                        {t("hero.cue.subtitle")}
                        <svg className={styles.cueChevron} width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round" aria-hidden="true">
                            <path d="M6 9l6 6 6-6" />
                        </svg>
                    </span>
                </span>
            </button>
        </section>
    );
}
