import { Link } from "@tanstack/react-router";
import { Kicker } from "#/components/ui/kicker";
import { authActions } from "#/lib/auth/store";
import { useFormatters, useT } from "#/lib/i18n";
import type { TypedT } from "#/lib/i18n/messages";
import type { messages } from "./Hero.messages";

interface IHeroProps {
    total: number;
    canCreate: boolean;
}

export function Hero({ total, canCreate }: IHeroProps) {
    const t: TypedT<typeof messages> = useT("tierLists");
    const f = useFormatters();

    return (
        <section className="page-gutter pt-10 pb-6 [--page-max:1080px] sm:pt-14 sm:pb-8">
            <div className="flex flex-wrap items-end justify-between gap-4">
                <div className="min-w-0">
                    <Kicker>{t("browse.hero.kicker")}</Kicker>
                    <h1 className="m-0 font-bold font-sans text-3xl text-foreground leading-tight tracking-tight sm:text-4xl">{t("browse.hero.title")}</h1>
                    <p className="mt-2 max-w-130 font-sans text-muted-foreground text-sm leading-relaxed">
                        {total > 0 ? (
                            <>
                                <span className="font-mono text-foreground tabular-nums">{f.number(total)}</span> {t("browse.hero.blurb", { count: total })}
                            </>
                        ) : (
                            t("browse.hero.blurbEmpty")
                        )}
                    </p>
                </div>

                {canCreate ? (
                    <Link
                        to="/tier-lists/my"
                        search={{ sort: "recent", type: "all", view: "grid", q: "" }}
                        className="inline-flex items-center gap-1.5 rounded-lg border border-primary bg-primary px-3.5 py-2 font-medium font-sans text-primary-foreground text-sm leading-none no-underline shadow-xs transition-shadow hover:shadow-md"
                    >
                        <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round" className="h-4 w-4" aria-hidden="true">
                            <path d="M12 5v14" />
                            <path d="M5 12h14" />
                        </svg>
                        {t("browse.hero.create")}
                    </Link>
                ) : (
                    <button type="button" onClick={() => authActions.openLoginDialog()} className="inline-flex cursor-pointer items-center gap-1.5 rounded-lg border border-border bg-muted px-3.5 py-2 font-medium font-sans text-foreground text-sm leading-none transition-colors hover:bg-accent">
                        {t("browse.hero.signIn")}
                    </button>
                )}
            </div>
        </section>
    );
}
