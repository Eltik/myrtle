import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { RefreshCw } from "lucide-react";
import * as React from "react";
import { CartesianGrid, Line, LineChart, ResponsiveContainer, Tooltip, XAxis, YAxis } from "recharts";
import { Button } from "#/components/ui/button";
import { Card } from "#/components/ui/card";
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from "#/components/ui/select";
import { useAuth } from "#/hooks/use-auth";
import { refreshRosterFn } from "#/lib/api/auth";
import { userQueryOptions } from "#/lib/api/user";
import { type TypedRichT, useFormatters, useLocale, useRichT, useT } from "#/lib/i18n";
import { cn } from "#/lib/utils";
import { formatDate } from "../helpers";
import { ANNIHILATION_CAPS, DAILY_MISSION_ORUNDUM, dayAt, type GreenCertShop, type IProjectedDay, MONTHLY_CARD_ORUNDUM, originiteWarning, WEEKLY_MISSION_ORUNDUM } from "../pulls/income";
import { ORUNDUM_PER_PULL } from "../pulls/rates";
import type { ISkinCommitment } from "../pulls/skins";
import type { IPullsSettings } from "../pulls/store";
import type { messages } from "./PullsPlannerTab.messages";
import { CurrencyIcon, CurrencyLabel, FIELD_LABEL, HINT_TEXT, InfoHint, PullsNumber, type PullsT, Stat } from "./PullsShared";
import { SectionTitle, ToggleField } from "./shared";

const HORIZONS = [30, 90, 180, 365] as const;

const CHART_PULLS = "#bcabdb";
const CHART_ORIGINITE = "oklch(0.85 0.18 80)";
const CHART_NET = "oklch(0.72 0.14 160)";

/** One entry in the chart's key: the series' own stroke, drawn small, plus its name. */
function ChartKey({ color, label, icon, dashed }: { color: string; label: string; icon?: "originite"; dashed?: boolean }): React.ReactElement {
    return (
        <span className="inline-flex items-center gap-1.5 font-sans text-[12px] text-muted-foreground">
            <svg width="14" height="2" aria-hidden="true" className="shrink-0">
                <title>{label}</title>
                <line x1="0" y1="1" x2="14" y2="1" stroke={color} strokeWidth="2" strokeDasharray={dashed ? "4 3" : undefined} />
            </svg>
            {icon && <CurrencyIcon name={icon} className="size-4" />}
            {label}
        </span>
    );
}

/** The "N with Originite Prime" line under a pull figure, with the currency's own icon. */
function OriginiteSub({ children }: { children: React.ReactNode }): React.ReactElement {
    return (
        <span className="inline-flex items-center gap-1">
            <CurrencyIcon name="originite" className="size-3.5" />
            {children}
        </span>
    );
}

/** One withdrawal the plan makes: rolls leaving the bank on the day a banner opens. */
export interface IPlanSpend {
    /** Unix seconds of the banner's EN start. */
    at: number;
    pulls: number;
}

interface IPullsBudgetProps {
    settings: IPullsSettings;
    setSettings: React.Dispatch<React.SetStateAction<IPullsSettings>>;
    days: IProjectedDay[];
    /** Pulls the plan below has already committed, subtracted from the headline. */
    committed: number;
    /** Free pulls the banners hand out, shown separately because they cannot be saved. */
    freePulls: number;
    /** Where the plan spends, so the chart can show the balance falling. */
    spend: IPlanSpend[];
    /** What the skins planner has committed to outfits. */
    skins: ISkinCommitment;
}

export function PullsBudget({ settings, setSettings, days, committed, freePulls, spend, skins }: IPullsBudgetProps): React.ReactElement {
    const t: PullsT = useT("tools");
    const rt: TypedRichT<typeof messages> = useRichT("tools");
    const f = useFormatters();
    const locale = useLocale();
    const { user } = useAuth();
    const uid = user?.uid ?? null;
    const profile = useQuery({ ...userQueryOptions(uid ?? ""), enabled: !!uid });
    const queryClient = useQueryClient();
    const resync = useMutation({
        mutationFn: () => refreshRosterFn(),
        onSuccess: () => queryClient.invalidateQueries({ queryKey: ["user"] }),
    });

    const first = days[0];
    const last = days[days.length - 1];
    // Base UI renders the raw value unless SelectValue is given a render function,
    // so every label the trigger shows is resolved here.
    const greenShopLabel = settings.greenCertShop === "phase1" ? t("release.pulls.income.greenShop.phase1") : settings.greenCertShop === "phase2" ? t("release.pulls.income.greenShop.phase2") : t("release.pulls.income.greenShop.off");
    // The projection already holds the outfit cost back from conversion, so the only
    // warning left is the outfits costing more than the player will hold at all.
    const warning = originiteWarning(skins.originite, last?.originite ?? 0);
    const set = <K extends keyof IPullsSettings>(key: K, value: IPullsSettings[K]) => setSettings((s) => ({ ...s, [key]: value }));

    /**
     * Recurring income as pulls per week. Annihilation and weekly missions land once
     * a week; the daily sources are multiplied up. The monthly store and the card's
     * Originite Prime are monthly, so they are deliberately left out of a figure
     * labelled "per week" rather than smeared across it.
     */
    const weekly = (DAILY_MISSION_ORUNDUM + settings.extraPerDay + (settings.monthlyCard ? MONTHLY_CARD_ORUNDUM : 0)) * 7 + WEEKLY_MISSION_ORUNDUM + settings.annihilation;

    const canSync = !!uid && (profile.data?.orundum ?? null) !== null;
    /**
     * Which boxes the fill writes, shown rather than described.
     *
     * Reported in #ui-ux: it was not obvious that "Pulls since last 6*" is NOT filled
     * from the account, so a player who pressed the button read a stale zero there as
     * their real counter. The four boxes it does write ring for a moment on the press,
     * which leaves the fifth visibly untouched.
     */
    const [flash, setFlash] = React.useState(false);
    React.useEffect(() => {
        if (!flash) return;
        const timer = window.setTimeout(() => setFlash(false), 1400);
        return () => window.clearTimeout(timer);
    }, [flash]);

    const fillFromAccount = () => {
        const p = profile.data;
        if (!p) return;
        setFlash(true);
        setSettings((s) => ({
            ...s,
            orundum: p.orundum ?? 0,
            permits: p.gacha_tickets ?? 0,
            tenPermits: p.ten_pull_tickets ?? 0,
            originite: p.originite ?? 0,
            monthlyCard: (p.monthly_sub_end ?? 0) * 1000 > Date.now(),
            manual: false,
        }));
    };

    /**
     * The projection, plus what the plan does to it.
     *
     * The two income lines only ever rise, which is an honest answer to "what will I
     * have banked" and a misleading one to "what will I be holding": a player who
     * commits four hundred rolls in December is not richer in January for having
     * earned since. The third line is the balance after the plan, so every banner the
     * user commits to shows up as the step DOWN that it is, and a plan that outruns
     * the income is visible as a line that keeps bottoming out.
     */
    const chartRows = React.useMemo(() => {
        if (days.length === 0) return [];
        // A commitment lands on the day its banner opens, so it is booked against that
        // day rather than smeared across the sample it happens to fall in.
        const drops = new Map<number, number>();
        for (const s of spend) {
            if (s.pulls <= 0) continue;
            const day = dayAt(days, s.at);
            if (!day) continue;
            drops.set(day.at, (drops.get(day.at) ?? 0) + s.pulls);
        }
        // The plan draws from whichever bank it was built against, so the net line
        // falls away from the same series the plan itself was priced on.
        let taken = 0;
        const rows = days.map((d) => {
            taken += drops.get(d.at) ?? 0;
            const bank = settings.spendOriginite ? d.pullsWithOriginite : d.pulls;
            return { at: d.at * 1000, pulls: d.pulls, withOp: d.pullsWithOriginite, net: Math.max(0, bank - taken) };
        });

        // One point per day is more than a 180-day axis can show; sample it down so
        // the line stays the same shape without handing Recharts 365 nodes. A drop day
        // and the day before it are always kept, because dropping either is exactly
        // what turns a one-day cliff into a gentle slope.
        const stride = Math.max(1, Math.ceil(days.length / 120));
        return rows.filter((_, i) => i % stride === 0 || i === rows.length - 1 || drops.has(days[i].at) || (i + 1 < days.length && drops.has(days[i + 1].at)));
    }, [days, spend, settings.spendOriginite]);

    const planned = chartRows.length > 0 && spend.some((s) => s.pulls > 0);

    return (
        <div className="flex flex-col gap-3">
            <Card className="flex flex-col gap-4 p-4">
                <div className="flex flex-wrap items-baseline justify-between gap-2">
                    <SectionTitle>{t("release.pulls.resources.title")}</SectionTitle>
                    {uid ? (
                        <div className="flex items-center gap-2">
                            <span className={HINT_TEXT}>{settings.manual ? "" : t("release.pulls.resources.synced")}</span>
                            <Button size="sm" variant="outline" disabled={!canSync} onClick={fillFromAccount}>
                                {t("release.pulls.resources.sync")}
                            </Button>
                            {/* An unlabelled circular arrow beside a labelled button is a
                                second unexplained control; it re-reads the account from the
                                game rather than re-reading the page's copy of it. */}
                            <Button size="sm" variant="ghost" disabled={resync.isPending} onClick={() => resync.mutate()} aria-label={t("release.pulls.resources.resync")} title={t("release.pulls.resources.resync")}>
                                <RefreshCw className={resync.isPending ? "size-4 animate-spin" : "size-4"} />
                            </Button>
                        </div>
                    ) : (
                        <span className={HINT_TEXT}>{t("release.pulls.resources.signedOut")}</span>
                    )}
                </div>

                <div className="flex flex-wrap gap-x-4 gap-y-3">
                    <PullsNumber id="pulls-orundum" label={<CurrencyLabel name="orundum">{t("release.pulls.resources.orundum")}</CurrencyLabel>} flash={flash} value={settings.orundum} onChange={(v) => setSettings((s) => ({ ...s, orundum: v, manual: true }))} step={600} />
                    <PullsNumber id="pulls-permits" label={<CurrencyLabel name="permit">{t("release.pulls.resources.permits")}</CurrencyLabel>} flash={flash} value={settings.permits} onChange={(v) => setSettings((s) => ({ ...s, permits: v, manual: true }))} className="w-20" />
                    <PullsNumber id="pulls-ten" label={<CurrencyLabel name="tenPermit">{t("release.pulls.resources.tenPermits")}</CurrencyLabel>} flash={flash} value={settings.tenPermits} onChange={(v) => setSettings((s) => ({ ...s, tenPermits: v, manual: true }))} className="w-20" />
                    <PullsNumber id="pulls-op" label={<CurrencyLabel name="originite">{t("release.pulls.resources.originite")}</CurrencyLabel>} flash={flash} value={settings.originite} onChange={(v) => setSettings((s) => ({ ...s, originite: v, manual: true }))} className="w-24" />
                    {/* The note that used to sit at the bottom of this card hangs off the
                        field it is about. It was read as a note about the whole card. */}
                    <PullsNumber
                        id="pulls-pity"
                        label={
                            <span className="inline-flex items-center gap-1.5">
                                {t("release.pulls.resources.pity")}
                                <InfoHint label={t("release.pulls.resources.pityHint.aria")}>{t("release.pulls.resources.pityHint")}</InfoHint>
                            </span>
                        }
                        value={settings.pity}
                        onChange={(v) => set("pity", v)}
                        max={98}
                        className="w-20"
                    />
                </div>
                <p className={cn("m-0", HINT_TEXT)}>{t("release.pulls.resources.pityManual")}</p>
            </Card>

            <Card className="flex flex-col gap-4 p-4">
                <SectionTitle>{t("release.pulls.income.title")}</SectionTitle>
                <div className="flex flex-wrap items-center gap-x-5 gap-y-2">
                    <ToggleField id="pulls-card" label={<CurrencyLabel name="monthlyCard">{t("release.pulls.income.monthlyCard")}</CurrencyLabel>} checked={settings.monthlyCard} onChange={(v) => set("monthlyCard", v)} />
                    <ToggleField id="pulls-op-spend" label={<CurrencyLabel name="originite">{t("release.pulls.income.spendOriginite")}</CurrencyLabel>} checked={settings.spendOriginite} onChange={(v) => set("spendOriginite", v)} />
                    <ToggleField
                        id="pulls-store"
                        label={
                            <span className="inline-flex items-center gap-1.5">
                                <CurrencyLabel name="orundum">{t("release.pulls.income.store")}</CurrencyLabel>
                                <InfoHint label={t("release.pulls.income.store.hint.aria")}>{t("release.pulls.income.store.hint")}</InfoHint>
                            </span>
                        }
                        ariaLabel={t("release.pulls.income.store")}
                        checked={settings.store}
                        onChange={(v) => set("store", v)}
                    />
                </div>
                <div className="flex flex-wrap items-center gap-x-4 gap-y-3">
                    <div className="flex flex-col gap-1">
                        <span className={FIELD_LABEL}>{t("release.pulls.income.annihilation")}</span>
                        <Select value={String(settings.annihilation)} onValueChange={(v) => v !== null && set("annihilation", Number(v))}>
                            <SelectTrigger size="sm" className="w-28 font-mono tabular-nums" aria-label={t("release.pulls.income.annihilation")}>
                                <SelectValue>{() => f.number(settings.annihilation)}</SelectValue>
                            </SelectTrigger>
                            <SelectContent>
                                {ANNIHILATION_CAPS.map((cap) => (
                                    <SelectItem key={cap} value={String(cap)}>
                                        {f.number(cap)}
                                    </SelectItem>
                                ))}
                            </SelectContent>
                        </Select>
                    </div>
                    <PullsNumber id="pulls-extra" label={t("release.pulls.income.extra")} value={settings.extraPerDay} onChange={(v) => set("extraPerDay", v)} step={50} className="w-24" />
                    <div className="flex flex-col gap-1">
                        <span className={FIELD_LABEL}>{t("release.pulls.income.horizon")}</span>
                        <Select value={String(settings.horizonDays)} onValueChange={(v) => v !== null && set("horizonDays", Number(v))}>
                            <SelectTrigger size="sm" className="w-28 font-mono tabular-nums" aria-label={t("release.pulls.a11y.horizon")}>
                                <SelectValue>{() => t("release.pulls.income.horizonDays", { count: settings.horizonDays })}</SelectValue>
                            </SelectTrigger>
                            <SelectContent>
                                {HORIZONS.map((d) => (
                                    <SelectItem key={d} value={String(d)}>
                                        {t("release.pulls.income.horizonDays", { count: d })}
                                    </SelectItem>
                                ))}
                            </SelectContent>
                        </Select>
                    </div>
                </div>
                <p className={cn("m-0", HINT_TEXT)}>{t("release.pulls.income.extraHint")}</p>

                <div className="flex flex-col gap-2 border-border border-t pt-3">
                    <div className="flex flex-wrap items-center gap-x-4 gap-y-3">
                        <PullsNumber id="pulls-gold-certs" label={<CurrencyLabel name="goldCert">{t("release.pulls.income.goldCerts")}</CurrencyLabel>} value={settings.goldCertsPerDay} onChange={(v) => set("goldCertsPerDay", v)} step={0.1} max={500} className="w-24" />
                        {/* A hand-tuned offset faking baseline alignment desynchronises
                            the moment a neighbouring label wraps, which it does below sm. */}
                        {/* "buy <hh permit icon> with <gold cert icon>", as asked for in #ui-ux:
                            the two nouns are things the player recognises by their picture. */}
                        <ToggleField
                            id="pulls-gold-shop"
                            label={rt("release.pulls.income.goldCertShop.rich", {
                                permits: <CurrencyLabel name="permit">{t("release.pulls.income.goldCertShop.permits")}</CurrencyLabel>,
                                certs: <CurrencyLabel name="goldCert">{t("release.pulls.income.goldCertShop.certs")}</CurrencyLabel>,
                            })}
                            ariaLabel={t("release.pulls.income.goldCertShop")}
                            checked={settings.goldCertShop}
                            onChange={(v) => set("goldCertShop", v)}
                        />
                    </div>
                    <p className={cn("m-0", HINT_TEXT)}>{t("release.pulls.income.goldCertHint")}</p>
                </div>

                <div className="flex flex-col gap-2 border-border border-t pt-3">
                    <div className="flex flex-wrap items-center gap-x-4 gap-y-3">
                        <PullsNumber id="pulls-green-certs" label={<CurrencyLabel name="greenCert">{t("release.pulls.income.greenCerts")}</CurrencyLabel>} value={settings.greenCertsPerWeek} onChange={(v) => set("greenCertsPerWeek", v)} step={5} max={5000} className="w-24" />
                        <div className="flex flex-col gap-1">
                            <span className={FIELD_LABEL}>
                                <CurrencyLabel name="greenCert">{t("release.pulls.income.greenCertShop")}</CurrencyLabel>
                            </span>
                            <Select value={settings.greenCertShop} onValueChange={(v) => v !== null && set("greenCertShop", v as GreenCertShop)}>
                                <SelectTrigger size="sm" className="w-40" aria-label={t("release.pulls.income.greenCertShop")}>
                                    <SelectValue>{() => greenShopLabel}</SelectValue>
                                </SelectTrigger>
                                <SelectContent>
                                    <SelectItem value="off">{t("release.pulls.income.greenShop.off")}</SelectItem>
                                    <SelectItem value="phase1">{t("release.pulls.income.greenShop.phase1")}</SelectItem>
                                    <SelectItem value="phase2">{t("release.pulls.income.greenShop.phase2")}</SelectItem>
                                </SelectContent>
                            </Select>
                        </div>
                    </div>
                    <p className={cn("m-0", HINT_TEXT)}>{t("release.pulls.income.greenCertHint")}</p>
                </div>

                <div className="flex flex-col gap-2 border-border border-t pt-3">
                    <ToggleField id="pulls-free" label={t("release.pulls.income.freePulls")} checked={settings.countFreePulls} onChange={(v) => set("countFreePulls", v)} />
                    <p className={cn("m-0", HINT_TEXT)}>{t("release.pulls.income.freePullsHint")}</p>
                    <p className={cn("m-0", HINT_TEXT)}>
                        {t("release.pulls.income.skinOriginiteHint", { count: skins.count, op: f.number(skins.originite) })}
                        {skins.unpriced > 0 && ` ${t("release.pulls.income.skinUnpriced", { count: skins.unpriced })}`}
                    </p>
                    {warning && <output className="m-0 block rounded-md border border-amber-500/30 bg-amber-500/10 px-3 py-2 font-sans text-[12px] text-amber-500 leading-normal">{t("release.pulls.income.originiteShort", { needed: f.number(warning.needed), available: f.number(warning.available) })}</output>}
                </div>
            </Card>

            <Card className="flex flex-col gap-4 p-4">
                <div className="flex flex-wrap items-start gap-x-8 gap-y-4">
                    {/* The "with Originite Prime" sub-lines and the weekly Orundum figure
                        were marked for icons; the currency sits in front of the number it
                        belongs to rather than being spelled out again in words. */}
                    <Stat
                        label={t("release.pulls.summary.now")}
                        value={f.number(first?.pulls ?? 0)}
                        sub={settings.spendOriginite && (first?.pullsWithOriginite ?? 0) > (first?.pulls ?? 0) ? <OriginiteSub>{t("release.pulls.summary.withOriginite", { count: f.number(first?.pullsWithOriginite ?? 0) })}</OriginiteSub> : undefined}
                    />
                    <Stat
                        label={t("release.pulls.summary.horizon", { date: last ? formatDate(last.at, locale) : "" })}
                        value={f.number(Math.max(0, (last?.pulls ?? 0) - committed))}
                        sub={settings.spendOriginite && (last?.pullsWithOriginite ?? 0) > (last?.pulls ?? 0) ? <OriginiteSub>{t("release.pulls.summary.withOriginite", { count: f.number(Math.max(0, (last?.pullsWithOriginite ?? 0) - committed)) })}</OriginiteSub> : undefined}
                    />
                    <Stat label={t("release.pulls.summary.perWeek")} value={f.number(weekly)} icon="orundum" sub={t("release.pulls.income.weekly", { count: Math.floor(weekly / ORUNDUM_PER_PULL) })} />
                    {freePulls > 0 && <Stat label={t("release.pulls.summary.free")} value={f.number(freePulls)} hint={<InfoHint label={t("release.pulls.summary.free.hint.aria")}>{t("release.pulls.summary.free.hint")}</InfoHint>} />}
                </div>

                <div className="flex flex-wrap items-baseline gap-x-4 gap-y-1">
                    <span className="font-sans font-semibold text-[13px] text-foreground">{t("release.pulls.chart.title")}</span>
                    {/* Three unlabelled lines are a puzzle. Recharts' own legend brings
                        its own type scale and its own spacing; this is the same key in
                        the type the rest of the card uses. */}
                    <div className="flex flex-wrap items-center gap-x-3 gap-y-1">
                        <ChartKey color={CHART_PULLS} label={t("release.pulls.chart.pulls")} />
                        {settings.spendOriginite && <ChartKey color={CHART_ORIGINITE} label={t("release.pulls.chart.withOriginite")} icon="originite" dashed />}
                        {planned && <ChartKey color={CHART_NET} label={t("release.pulls.chart.net")} />}
                    </div>
                </div>

                <div className="h-56 w-full sm:h-64" role="img" aria-label={planned ? t("release.pulls.chart.ariaPlanned", { date: last ? formatDate(last.at, locale) : "", count: spend.filter((s) => s.pulls > 0).length }) : t("release.pulls.chart.aria", { date: last ? formatDate(last.at, locale) : "" })}>
                    <ResponsiveContainer width="100%" height="100%">
                        <LineChart data={chartRows} margin={{ top: 8, right: 8, bottom: 0, left: -18 }}>
                            <CartesianGrid stroke="var(--border)" strokeDasharray="3 3" vertical={false} />
                            <XAxis dataKey="at" type="number" scale="time" domain={["dataMin", "dataMax"]} tickFormatter={(v: number) => formatDate(Math.floor(v / 1000), locale)} tick={{ fill: "var(--muted-foreground)", fontSize: 11 }} stroke="var(--border)" minTickGap={40} />
                            <YAxis tick={{ fill: "var(--muted-foreground)", fontSize: 11 }} stroke="var(--border)" width={48} allowDecimals={false} />
                            <Tooltip
                                contentStyle={{ background: "var(--popover)", border: "1px solid var(--border)", borderRadius: 8, fontSize: 12 }}
                                labelStyle={{ color: "var(--muted-foreground)" }}
                                labelFormatter={(v) => formatDate(Math.floor(Number(v) / 1000), locale)}
                                formatter={(value, name) => [f.number(Number(value)), String(name)]}
                            />
                            <Line type="monotone" dataKey="pulls" name={t("release.pulls.chart.pulls")} stroke={CHART_PULLS} strokeWidth={2} dot={false} isAnimationActive={false} />
                            {settings.spendOriginite && <Line type="monotone" dataKey="withOp" name={t("release.pulls.chart.withOriginite")} stroke={CHART_ORIGINITE} strokeWidth={2} strokeDasharray="4 3" dot={false} isAnimationActive={false} />}
                            {/* Linear, unlike its neighbours. A monotone spline rounds the
                                one-day step into a slope, and the step is the only reason
                                this line is drawn. */}
                            {planned && <Line type="linear" dataKey="net" name={t("release.pulls.chart.net")} stroke={CHART_NET} strokeWidth={2} dot={false} isAnimationActive={false} />}
                        </LineChart>
                    </ResponsiveContainer>
                </div>
            </Card>
        </div>
    );
}
