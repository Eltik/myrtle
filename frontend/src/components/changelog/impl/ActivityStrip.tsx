import { useMemo } from "react";
import type { IChangelogCommit } from "#/lib/api/changelog";
import { useFormatters, useT } from "#/lib/i18n";
import type { TypedT } from "#/lib/i18n/messages";
import type { messages } from "./ActivityStrip.messages";
import styles from "./ChangelogPage.module.css";

const DAY_MS = 86_400_000;

interface IBucket {
    label: string;
    count: number;
}

/** A compact commits-per-day bar strip across the active range. */
export function ActivityStrip({ commits, days }: { commits: IChangelogCommit[]; days: number }) {
    const t: TypedT<typeof messages> = useT("changelog");
    const f = useFormatters();

    const buckets = useMemo<IBucket[]>(() => {
        const today = new Date();
        today.setHours(0, 0, 0, 0);
        const start = today.getTime() - (days - 1) * DAY_MS;
        const out: IBucket[] = Array.from({ length: days }, (_, i) => {
            const d = new Date(start + i * DAY_MS);
            return { label: f.date(d, { month: "short", day: "numeric" }), count: 0 };
        });
        for (const c of commits) {
            const t = new Date(c.date).getTime();
            const idx = Math.floor((t - start) / DAY_MS);
            if (idx >= 0 && idx < out.length) out[idx].count++;
        }
        return out;
    }, [commits, days, f]);

    const max = Math.max(1, ...buckets.map((b) => b.count));
    const busiest = buckets.reduce((a, b) => (b.count > a.count ? b : a), buckets[0]);

    return (
        <div>
            <div className="mb-2 flex items-baseline justify-between">
                <span className="font-medium font-sans text-[12px] text-muted-foreground uppercase tracking-[0.14em]">{t("activity.title")}</span>
                {busiest && busiest.count > 0 ? (
                    <span className="font-sans text-[12px] text-muted-foreground">
                        {t("activity.busiest")} <span className="font-medium text-foreground/80">{busiest.label}</span> · {busiest.count}
                    </span>
                ) : null}
            </div>
            <div className="flex h-16 items-end gap-px sm:gap-0.5" role="img" aria-label={t("activity.aria", { days, label: busiest?.label ?? t("activity.aria.noDay"), count: busiest?.count ?? 0 })}>
                {buckets.map((b, i) => (
                    <div
                        // biome-ignore lint/suspicious/noArrayIndexKey: fixed-length ordered day buckets
                        key={i}
                        className="group/bar relative flex h-full flex-1 items-end"
                        title={t("activity.bar", { label: b.label, count: b.count })}
                    >
                        <div
                            className={`${styles.activityBar} w-full rounded-xs transition-colors ${b.count > 0 ? "bg-primary/35 group-hover/bar:bg-primary/70" : "bg-muted group-hover/bar:bg-muted-foreground/30"}`}
                            style={{ height: `${b.count > 0 ? Math.max(8, (b.count / max) * 100) : 6}%`, animationDelay: `${Math.min(i * 12, 360)}ms` }}
                        />
                    </div>
                ))}
            </div>
        </div>
    );
}
