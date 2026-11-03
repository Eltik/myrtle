import { Kbd } from "#/components/ui/kbd";
import { useIsMac } from "#/hooks/use-is-mac";
import { type TypedRichT, useRichT, useT } from "#/lib/i18n";
import type { TypedT } from "#/lib/i18n/messages";
import type { messages } from "./Hero.messages";
import styles from "./Hero.module.css";
import shared from "./shared.module.css";

const HERO_WIDTHS = [800, 1200, 1600, 2400];
const HERO_WEBP_SRCSET = HERO_WIDTHS.map((w) => `/home/hero_art-${w}w.webp ${w}w`).join(", ");
const HERO_JPEG_SRCSET = HERO_WIDTHS.map((w) => `/home/${w === 2400 ? "hero_art.jpg" : `hero_art-${w}w.jpg`} ${w}w`).join(", ");
// The art is cropped with object-fit: cover inside a box up to 116% of max(78svh, 52vw)
// tall, so the 16:9 image renders wider than the viewport: up to 161vh when the viewport
// is narrower than 3:2, up to 108vw otherwise. A 375px phone draws it about 1110px wide,
// so 100vw would hand it a file a third of the width it shows. The box also stops at the
// hero's height, so these are ceilings and err towards the sharper file.
const HERO_SIZES = "(max-aspect-ratio: 3/2) 161vh, 108vw";

export default function Hero({ onOpenCommand, scrollTargetId }: { onOpenCommand: () => void; scrollTargetId: string }) {
    const t: TypedT<typeof messages> = useT("home");
    const rt: TypedRichT<typeof messages> = useRichT("home");
    const isMac = useIsMac();

    return (
        <section className={styles.hero}>
            <div className={styles.art} aria-hidden="true">
                <div className={styles.artLayer}>
                    <picture>
                        <source type="image/webp" srcSet={HERO_WEBP_SRCSET} sizes={HERO_SIZES} />
                        <img src="/home/hero_art.jpg" srcSet={HERO_JPEG_SRCSET} sizes={HERO_SIZES} alt="" width={2400} height={1350} fetchPriority="high" />
                    </picture>
                </div>
                <div className={styles.artFade} />
            </div>
            <div className={styles.spacer} />

            <div className={`${shared.container} ${styles.body}`}>
                <div className={styles.myrtle}>
                    <div className={styles.myrtleGlow} aria-hidden="true" />
                    <img src="/home/myrtle_sleepy.png" alt={t("hero.myrtleAlt")} width={300} height={300} className={styles.myrtleImg} />
                </div>
                <div className={styles.copy}>
                    <div className={styles.intro}>
                        <h1 className={styles.title}>{t("hero.title")}</h1>
                        <p className={styles.lead}>{t("hero.lead")}</p>
                    </div>
                    <button type="button" className={styles.hint} onClick={onOpenCommand}>
                        {rt("hero.hint", {
                            keys: (
                                <span className="inline-flex items-center gap-1.5">
                                    <Kbd>{isMac ? "⌘" : t("hero.keyCtrl")}</Kbd>
                                    <Kbd>K</Kbd>
                                </span>
                            ),
                        })}
                    </button>
                </div>
            </div>

            <a href={`#${scrollTargetId}`} className={styles.cue}>
                <img src="/home/myrtle_happy.png" alt="" width={52} height={52} className={styles.cueImg} />
                <span className={styles.cueText}>
                    <span className={styles.cueTitle}>{t("hero.cue.title")}</span>
                    <span className={styles.cueSub}>
                        {t("hero.cue.subtitle")}
                        <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round" aria-hidden="true">
                            <path d="M6 9l6 6 6-6" />
                        </svg>
                    </span>
                </span>
            </a>
        </section>
    );
}
