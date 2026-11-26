import { Link } from "@tanstack/react-router";
import { Button } from "#/components/ui/button";
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
                    <Button render={<Link to="/tier-lists/my" search={{ sort: "recent", type: "all", view: "grid", q: "" }} />}>
                        <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round" className="size-4" aria-hidden="true">
                            <path d="M12 5v14" />
                            <path d="M5 12h14" />
                        </svg>
                        {t("browse.hero.create")}
                    </Button>
                ) : (
                    <Button variant="secondary" onClick={() => authActions.openLoginDialog()}>
                        {t("browse.hero.signIn")}
                    </Button>
                )}
            </div>
        </section>
    );
}
