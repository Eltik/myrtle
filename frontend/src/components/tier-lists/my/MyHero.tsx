import { PlusIcon } from "lucide-react";
import { Button } from "#/components/ui/button";
import { Kicker } from "#/components/ui/kicker";
import { type TypedRichT, useFormatters, useRichT, useT } from "#/lib/i18n";
import type { TypedT } from "#/lib/i18n/messages";
import type { messages } from "./MyHero.messages";

interface IMyHeroProps {
    total: number;
    communityCount: number;
    communityQuota: number;
    officialCount: number;
    totalViews: number;
    totalFavorites: number;
    onCreate: () => void;
}

interface IStatPillProps {
    label: string;
    value: string;
}

function StatPill({ label, value }: IStatPillProps) {
    return (
        <div className="flex min-w-0 items-center gap-2 rounded-lg border border-border bg-card px-3 py-2 shadow-[0_1px_2px_oklch(0_0_0/0.04)]">
            <div className="flex flex-col gap-1">
                <span className="font-medium font-mono text-[10px] text-muted-foreground uppercase leading-none tracking-wider">{label}</span>
                <span className="font-sans font-semibold text-[15px] text-foreground tabular-nums leading-none">{value}</span>
            </div>
        </div>
    );
}

export function MyHero({ total, communityCount, communityQuota, officialCount, totalViews, totalFavorites, onCreate }: IMyHeroProps) {
    const t: TypedT<typeof messages> = useT("tierLists");
    const rt: TypedRichT<typeof messages> = useRichT("tierLists");
    const f = useFormatters();
    const atQuota = communityCount >= communityQuota;
    const quotaPct = Math.min(100, Math.round((communityCount / communityQuota) * 100));

    return (
        <section className="mx-auto w-[min(1080px,calc(100%-2rem))] pt-10 pb-6 sm:pt-14 sm:pb-8">
            <div className="flex flex-wrap items-end justify-between gap-4">
                <div className="min-w-0 flex-1">
                    <Kicker>{t("my.hero.kicker")}</Kicker>
                    <h1 className="m-0 font-bold font-sans text-3xl text-foreground leading-tight tracking-tight sm:text-4xl">{t("my.hero.title")}</h1>
                    <p className="mt-2 max-w-130 font-sans text-muted-foreground text-sm leading-relaxed">{total > 0 ? rt("my.hero.blurb", { total: <span className="font-mono text-foreground tabular-nums">{f.number(total)}</span>, count: total }) : t("my.hero.blurbEmpty")}</p>
                </div>

                <Button onClick={onCreate} disabled={atQuota} title={atQuota ? t("my.hero.quotaReached", { max: communityQuota }) : undefined}>
                    <PlusIcon />
                    <span>{t("my.hero.create")}</span>
                </Button>
            </div>

            <div className="mt-6 grid grid-cols-2 gap-2 sm:grid-cols-4">
                <StatPill label={t("my.hero.stat.total")} value={f.number(total)} />
                <div className="relative overflow-hidden rounded-lg border border-border bg-card px-3 py-2 shadow-[0_1px_2px_oklch(0_0_0/0.04)]">
                    <div className="flex flex-col gap-1">
                        <span className="font-medium font-mono text-[10px] text-muted-foreground uppercase leading-none tracking-wider">{t("my.hero.stat.quota")}</span>
                        <span className="font-sans font-semibold text-[15px] text-foreground tabular-nums leading-none">
                            {communityCount}
                            <span className="text-muted-foreground"> / {communityQuota}</span>
                        </span>
                    </div>
                    <div aria-hidden="true" className="mt-2 h-1 w-full overflow-hidden rounded-full bg-muted">
                        <div
                            className="h-full rounded-full transition-[width] duration-300"
                            style={{
                                width: `${quotaPct}%`,
                                background: atQuota ? "var(--destructive)" : "var(--primary)",
                                boxShadow: atQuota ? "none" : "0 0 8px color-mix(in srgb, var(--primary) 50%, transparent)",
                            }}
                        />
                    </div>
                </div>
                <StatPill label={t("my.hero.stat.views")} value={f.compact(totalViews)} />
                <StatPill label={t("my.hero.stat.favorites")} value={f.compact(totalFavorites)} />
            </div>

            {officialCount > 0 && (
                <p className="mt-3 inline-flex items-center gap-1.5 font-mono text-[10.5px] text-muted-foreground uppercase leading-none tracking-wider">
                    <span className="inline-block h-1.5 w-1.5 rounded-full bg-warning" aria-hidden="true" />
                    {t("my.hero.officialIncluded", { count: officialCount })}
                </p>
            )}
        </section>
    );
}
