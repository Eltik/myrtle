import { Link } from "@tanstack/react-router";
import { LocaleSwitcher } from "#/components/LocaleSwitcher";
import { useT } from "#/lib/i18n";
import type { TypedT } from "#/lib/i18n/messages";
import type { messages } from "./Footer.messages";

export default function Footer() {
    const t: TypedT<typeof messages> = useT("common");

    return (
        <footer className="border-border border-t bg-[color-mix(in_srgb,var(--background)_84%,transparent)] py-7">
            <div className="page-gutter flex flex-col gap-2.5 [--page-max:1080px]">
                <div className="flex flex-wrap items-center justify-between gap-3">
                    <div className="inline-flex items-center gap-2.5 font-sans font-semibold text-foreground text-sm leading-none">
                        <img src="/logo/bust_transparent.png" alt="" width={22} height={22} className="h-5.5 w-5.5 shrink-0 object-contain" />
                        <span>
                            myrtle.moe <span className="font-normal text-muted-foreground">· v3</span>
                        </span>
                    </div>
                    <nav aria-label={t("footer.siteLinks")} className="inline-flex items-center gap-4">
                        <Link to="/changelog" className="font-sans text-[12.5px] text-muted-foreground leading-none transition-colors hover:text-foreground">
                            {t("footer.changelog")}
                        </Link>
                        <Link to="/terms" className="font-sans text-[12.5px] text-muted-foreground leading-none transition-colors hover:text-foreground">
                            {t("footer.terms")}
                        </Link>
                        <Link to="/privacy" className="font-sans text-[12.5px] text-muted-foreground leading-none transition-colors hover:text-foreground">
                            {t("footer.privacy")}
                        </Link>
                        <Link to="/discord" className="font-sans text-[12.5px] text-muted-foreground leading-none transition-colors hover:text-foreground">
                            {t("footer.discord")}
                        </Link>
                        <Link to="/donate" target="_blank" className="font-sans text-[12.5px] text-muted-foreground leading-none transition-colors hover:text-foreground">
                            {t("footer.donate")}
                        </Link>
                    </nav>
                </div>
                <span className="max-w-130 font-sans text-muted-foreground text-xs leading-normal">{t("footer.disclaimer")}</span>
                <div className="flex flex-wrap items-center justify-between gap-3">
                    <span className="font-medium font-mono text-[11px] text-muted-foreground leading-none tracking-wide opacity-70">{t("footer.builtOn")}</span>
                    <LocaleSwitcher />
                </div>
            </div>
        </footer>
    );
}
