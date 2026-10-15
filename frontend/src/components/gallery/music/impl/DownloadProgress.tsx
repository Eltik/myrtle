import type React from "react";
import { useLocale, useT } from "#/lib/i18n";
import type { TypedT } from "#/lib/i18n/messages";
import type { messages } from "../MusicGallery.messages";
import { useDownload } from "./download";

type MusicT = TypedT<typeof messages>;

/** How coarsely a screen reader hears the progress: a quarter of a known size, or every 10 MB of an unknown one. */
const SPOKEN_STEP = 0.25;
const SPOKEN_MB = 10;

/**
 * A download toast's progress line. It reads the download store itself, so
 * the toast is added once and never updated while the bytes arrive: every
 * `toastManager.update` bumps the toast's `updateKey`, which replays the
 * upsert pulse. The visible text follows the store (at most four writes a
 * second, see `counted`) in whole percents or tenths of a MB with tabular
 * figures, so it does not jitter; the toast viewport is a polite live region,
 * so the spoken copy is hidden from sight and changes only in coarse steps.
 */
export function DownloadProgress({ trackId }: { trackId: string }): React.ReactElement {
    const t: MusicT = useT("gallery");
    const locale = useLocale();
    const d = useDownload(trackId);
    if (!d || (d.received === 0 && !d.total)) return <>{t("music.download.starting")}</>;
    let shown: string;
    let spoken: string;
    if (d.total) {
        const fraction = Math.min(1, d.received / d.total);
        const pct = (f: number) => new Intl.NumberFormat(locale, { style: "percent", maximumFractionDigits: 0 }).format(f);
        shown = t("music.download.progress", { percent: pct(Math.floor(fraction * 100) / 100) });
        spoken = t("music.download.progress", { percent: pct(Math.floor(fraction / SPOKEN_STEP) * SPOKEN_STEP) });
    } else {
        const mb = (n: number) => new Intl.NumberFormat(locale, { minimumFractionDigits: 1, maximumFractionDigits: 1 }).format(n);
        const received = d.received / 1_000_000;
        shown = t("music.download.received", { size: mb(Math.floor(received * 10) / 10) });
        spoken = t("music.download.received", { size: mb(Math.floor(received / SPOKEN_MB) * SPOKEN_MB) });
    }
    return (
        <>
            <span aria-hidden="true" className="tabular-nums">
                {shown}
            </span>
            <span className="sr-only">{spoken}</span>
        </>
    );
}
