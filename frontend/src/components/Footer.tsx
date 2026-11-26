import { Link } from "@tanstack/react-router";
import { useT } from "#/lib/i18n";
import type { TypedT } from "#/lib/i18n/messages";
import { useRichT } from "#/lib/i18n/rich";
import type { messages } from "./Footer.messages";

/** A footer nav link. On a touch screen it grows to 44px: at `leading-none` it was about 13px tall. */
const FOOTER_LINK = "inline-flex items-center justify-center font-sans text-[12.5px] text-muted-foreground leading-none transition-colors hover:text-foreground pointer-coarse:min-h-11 pointer-coarse:min-w-11";

/** The font credit's six links. The sentence itself is ONE message, so a translator can move them.
    They sit inline in running text, so on a touch screen the 44px target is a centred `::after`
    rather than a taller box that would break the sentence's line height. */
const FONT_LINK = "relative underline underline-offset-2 transition-colors hover:text-foreground pointer-coarse:after:absolute pointer-coarse:after:inset-x-0 pointer-coarse:after:top-1/2 pointer-coarse:after:h-11 pointer-coarse:after:-translate-y-1/2";

export default function Footer() {
    const t: TypedT<typeof messages> = useT("common");
    const rt = useRichT("common");

    return (
        <footer className="border-border border-t bg-[color-mix(in_srgb,var(--background)_84%,transparent)] py-7">
            <div className="page-gutter flex flex-col gap-2.5 [--page-max:1080px]">
                <div className="flex flex-wrap items-center justify-between gap-3">
                    <div className="inline-flex items-center gap-2.5 font-sans font-semibold text-foreground text-sm leading-none">
                        <img src="/logo/bust_128.png" alt="" width={22} height={22} className="h-5.5 w-5.5 shrink-0 object-contain" />
                        <span>
                            myrtle.moe <span className="font-normal text-muted-foreground">· v3</span>
                        </span>
                    </div>
                    <nav aria-label={t("footer.siteLinks")} className="inline-flex flex-wrap items-center gap-x-4">
                        <Link to="/changelog" className={FOOTER_LINK}>
                            {t("footer.changelog")}
                        </Link>
                        <Link to="/terms" className={FOOTER_LINK}>
                            {t("footer.terms")}
                        </Link>
                        <Link to="/privacy" className={FOOTER_LINK}>
                            {t("footer.privacy")}
                        </Link>
                        <Link to="/discord" className={FOOTER_LINK}>
                            {t("footer.discord")}
                        </Link>
                        <Link to="/donate" className={FOOTER_LINK}>
                            {t("footer.donate")}
                        </Link>
                    </nav>
                </div>
                <span className="max-w-130 font-sans text-muted-foreground text-xs leading-normal">{t("footer.disclaimer")}</span>
                {/* One line, and it is REQUIRED: the Sarkaz face is CC BY-NC 4.0,
                    which asks for attribution by name, the two Terra faces are OFL
                    derivatives renamed because OFL reserves "Samigirian", and
                    OpenDyslexic is shipped unmodified under OFL 1.1, which asks
                    for the copyright notice to travel with the font. */}
                <span className="max-w-170 font-sans text-[11px] text-muted-foreground leading-normal">
                    {rt("footer.fontCredit", {
                        endfield: (
                            <a href="https://github.com/lhclbt/Endfield_Font" target="_blank" rel="noreferrer" className={FONT_LINK}>
                                Endfield_Font
                            </a>
                        ),
                        ccLicence: (
                            <a href="https://creativecommons.org/licenses/by-nc/4.0/" target="_blank" rel="noreferrer" className={FONT_LINK}>
                                CC BY-NC 4.0
                            </a>
                        ),
                        samigirian: (
                            <a href="https://github.com/Siphercase/Samigirian" target="_blank" rel="noreferrer" className={FONT_LINK}>
                                Samigirian
                            </a>
                        ),
                        oflLicence: (
                            <a href="/terra-fonts/OFL.txt" target="_blank" rel="noreferrer" className={FONT_LINK}>
                                OFL 1.1
                            </a>
                        ),
                        openDyslexic: (
                            <a href="https://github.com/antijingoist/opendyslexic" target="_blank" rel="noreferrer" className={FONT_LINK}>
                                OpenDyslexic
                            </a>
                        ),
                        odLicence: (
                            <a href="/opendyslexic/OFL.txt" target="_blank" rel="noreferrer" className={FONT_LINK}>
                                OFL 1.1
                            </a>
                        ),
                    })}
                </span>
                {/* No language picker here: it would be the third copy of one control, after the
                    header trigger and the mobile drawer, and the footer is the least likely
                    place anyone looks for it. */}
                <div className="flex flex-wrap items-center justify-between gap-3">
                    <span className="font-medium font-mono text-[11px] text-muted-foreground leading-none tracking-wide">{t("footer.builtOn")}</span>
                </div>
            </div>
        </footer>
    );
}
