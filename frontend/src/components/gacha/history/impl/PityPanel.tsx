import type React from "react";
import { Badge } from "#/components/ui/badge";
import { Kicker } from "#/components/ui/kicker";
import { Skeleton } from "#/components/ui/skeleton";
import type { ClientGachaGroup, IBanner, IClientGachaRecords } from "#/lib/api/gacha";
import { useT } from "#/lib/i18n";
import type { TypedT } from "#/lib/i18n/messages";
import { BANNER_GROUP_LABEL_KEYS, type GachaMessageKey } from "../../constants";
import type { messages as gachaConstantsMessages } from "../../constants.messages";
import type { messages } from "./PityPanel.messages";
import { pityWithLimitedReset, sharedPity } from "./pity";

/** This panel renders its own chrome plus the shared banner-bucket labels. */
type PityT = TypedT<typeof messages & typeof gachaConstantsMessages>;

interface IPityPanelProps {
    records: IClientGachaRecords | null;
    bannersById: Map<string, IBanner>;
    isLoading: boolean;
}

interface IBannerPity {
    key: ClientGachaGroup;
    labelKey: GachaMessageKey;
    pity: number;
    softPityAt: number;
    hardPityAt: number;
    color: string;
    /** True if pity was reset to 0 because the user's last pull was on an ended Limited/Collab banner. */
    reset?: boolean;
    /** Display name of the now-ended banner, when `reset` is true. */
    resetReason?: string;
}

const BANNER_CONFIGS: { key: ClientGachaGroup; labelKey: GachaMessageKey; softPityAt: number; hardPityAt: number; color: string }[] = [
    { key: "limited", labelKey: BANNER_GROUP_LABEL_KEYS.limited, softPityAt: 50, hardPityAt: 99, color: "oklch(0.85 0.18 80)" },
    // Collab banners: their own pity, same 50/99 curve. The "within 120 attempts" in their
    // text is a once-per-banner guarantee of the featured operator, not 6* pity.
    { key: "linkage", labelKey: BANNER_GROUP_LABEL_KEYS.linkage, softPityAt: 50, hardPityAt: 99, color: "oklch(0.78 0.16 320)" },
    { key: "regular", labelKey: BANNER_GROUP_LABEL_KEYS.regular, softPityAt: 50, hardPityAt: 99, color: "#bcabdb" },
    { key: "special", labelKey: BANNER_GROUP_LABEL_KEYS.special, softPityAt: 50, hardPityAt: 99, color: "#88c8e3" },
    // No Starter Headhunting card: its 21 pulls end on their own guaranteed 6*, so there is no counter to carry.
];

function PityMeter({ pity, softPityAt, hardPityAt, color }: { pity: number; softPityAt: number; hardPityAt: number; color: string }) {
    const t: PityT = useT("gacha");
    const pct = Math.min((pity / hardPityAt) * 100, 100);
    const softPct = (softPityAt / hardPityAt) * 100;
    const isSoftPity = pity >= softPityAt;
    const isNearHard = pity >= hardPityAt - 5;

    return (
        <div className="flex flex-col gap-1.5">
            <div className="relative h-3 w-full overflow-visible rounded-full bg-muted">
                <div className="h-full overflow-hidden rounded-full">
                    <div
                        className="h-full rounded-full transition-all duration-500"
                        style={{
                            width: `${pct}%`,
                            background: isNearHard ? "linear-gradient(90deg, oklch(0.78 0.18 25), oklch(0.65 0.22 15))" : isSoftPity ? `linear-gradient(90deg, ${color}, oklch(0.78 0.18 25))` : `linear-gradient(90deg, color-mix(in oklch, ${color} 60%, transparent), ${color})`,
                        }}
                    />
                </div>
                {/* No title: the tick takes no pointer events, so a tooltip on it never showed. The
                    "soft pity at N" caption under the bar is its label. */}
                <div className="pointer-events-none absolute top-0 bottom-0 w-px bg-foreground/30" style={{ left: `${softPct}%` }} aria-hidden />
            </div>
            <div className="flex items-center justify-between font-mono text-[10px] text-muted-foreground">
                <span>{t("history.pity.softPityAt", { count: softPityAt })}</span>
                <span>{t("history.pity.guaranteedAt", { count: hardPityAt })}</span>
            </div>
        </div>
    );
}

function PityCard({ labelKey, pity, softPityAt, hardPityAt, color, total, reset, resetReason }: { labelKey: GachaMessageKey; pity: number; softPityAt: number; hardPityAt: number; color: string; total: number; reset?: boolean; resetReason?: string }) {
    const t: PityT = useT("gacha");
    const isSoftPity = pity >= softPityAt;
    const isNearHard = pity >= hardPityAt - 5;
    // Status tokens, not the old oklch(0.78) literals that had no light variant (1.93 / 2.28).
    // On muted/30 over the card: warning 4.80 light / 10.85 dark, destructive 6.10 / 6.44.
    const statusColor = reset ? "text-muted-foreground" : isNearHard ? "text-destructive-foreground" : isSoftPity ? "text-warning-foreground" : "text-foreground";

    return (
        <div className="flex flex-col gap-3 rounded-2xl border border-border bg-muted/30 p-4">
            <div className="flex items-start justify-between gap-3">
                <div>
                    <Kicker className="mb-1 block">{t("history.pity.cardLabel", { type: t(labelKey) })}</Kicker>
                    <div className={`font-bold font-sans text-[32px] tabular-nums leading-none tracking-[-0.04em] ${statusColor}`}>
                        {pity}
                        <span className="ml-1 font-medium font-mono text-[13px] text-muted-foreground">{t("history.pity.pullsUnit")}</span>
                    </div>
                </div>
                <div className="flex flex-col items-end gap-1 text-right">
                    {reset ? (
                        <Badge variant="outline">{t("history.pity.bannerEnded")}</Badge>
                    ) : (
                        <>
                            {isSoftPity ? (
                                // Tone on the fill and border, foreground text: 17.20 light / 14.10 dark.
                                <Badge style={{ "--tone": "var(--warning)" } as React.CSSProperties} variant="tint">
                                    {t("history.pity.softPity")}
                                </Badge>
                            ) : null}
                            {isNearHard ? (
                                <Badge style={{ "--tone": "var(--destructive)" } as React.CSSProperties} variant="tint">
                                    {t("history.pity.nearGuaranteed")}
                                </Badge>
                            ) : null}
                        </>
                    )}
                    <span className="font-mono text-[10px] text-muted-foreground tabular-nums">{t("history.pity.total", { count: total })}</span>
                </div>
            </div>
            <PityMeter pity={pity} softPityAt={softPityAt} hardPityAt={hardPityAt} color={color} />
            {reset && resetReason ? <div className="font-mono text-[10px] text-muted-foreground leading-snug">{t("history.pity.resetNote", { type: t(labelKey), banner: resetReason })}</div> : null}
        </div>
    );
}

export function PityPanel({ records, bannersById, isLoading }: IPityPanelProps) {
    const t: PityT = useT("gacha");

    if (isLoading) {
        return (
            <section className="flex flex-col gap-4 rounded-2xl border border-border bg-card p-4.5 sm:p-[22px_24px]">
                <Skeleton className="h-4 w-28 rounded" />
                <div className="grid grid-cols-1 gap-3 sm:grid-cols-2 lg:grid-cols-4">
                    {Array.from({ length: 4 }).map((_, i) => (
                        // biome-ignore lint/suspicious/noArrayIndexKey: skeleton
                        <Skeleton key={i} className="h-32 rounded-2xl" />
                    ))}
                </div>
            </section>
        );
    }

    if (!records) return null;

    const pities: IBannerPity[] = BANNER_CONFIGS.map((cfg) => {
        const items = records[cfg.key].records;
        if (cfg.key === "limited" || cfg.key === "linkage") {
            const { pity, reset, resetReason } = pityWithLimitedReset(items, bannersById, Date.now() / 1000);
            return { ...cfg, pity, reset, resetReason };
        }
        return { ...cfg, pity: sharedPity(items, bannersById) };
    });

    return (
        <section className="flex flex-col gap-4 rounded-2xl border border-border bg-card p-4.5 sm:p-[22px_24px]">
            <header>
                <Kicker className="mb-1.5">{t("history.pity.kicker")}</Kicker>
                <h2 className="m-0 font-sans font-semibold text-[20px] text-foreground leading-[1.15] tracking-[-0.02em] sm:text-[22px]">{t("history.pity.title")}</h2>
            </header>
            <div className="grid grid-cols-1 gap-3 sm:grid-cols-2 lg:grid-cols-4">
                {pities.map((p) => (
                    <PityCard key={p.key} labelKey={p.labelKey} pity={p.pity} softPityAt={p.softPityAt} hardPityAt={p.hardPityAt} color={p.color} total={records[p.key].total} reset={p.reset} resetReason={p.resetReason} />
                ))}
            </div>
        </section>
    );
}
