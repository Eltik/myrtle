import { useQuery } from "@tanstack/react-query";
import { Link } from "@tanstack/react-router";
import { CalendarClock, ExternalLink, GitBranch, ScrollText, Users } from "lucide-react";
import { useMemo, useState } from "react";
import { Kicker } from "#/components/ui/kicker";
import { Tabs, TabsList, TabsTab } from "#/components/ui/tabs";
import { CHANGELOG_RANGES, type ChangelogRangeId, changelogQueryOptions, commitsWithinDays, type IChangelogCommit } from "#/lib/api/changelog";
import { formatRelative } from "#/lib/utils";
import { ActivityStrip } from "./impl/ActivityStrip";
import styles from "./impl/ChangelogPage.module.css";
import { CommitItem } from "./impl/CommitItem";

interface ICommitGroup {
    key: string;
    label: string;
    commits: IChangelogCommit[];
}

function startOfDay(d: Date): number {
    const x = new Date(d);
    x.setHours(0, 0, 0, 0);
    return x.getTime();
}

function groupLabel(dayStart: number, today: number): string {
    const diffDays = Math.round((today - dayStart) / 86_400_000);
    if (diffDays <= 0) return "Today";
    if (diffDays === 1) return "Yesterday";
    const d = new Date(dayStart);
    const sameYear = d.getFullYear() === new Date(today).getFullYear();
    return d.toLocaleDateString(undefined, { weekday: "short", month: "short", day: "numeric", ...(sameYear ? {} : { year: "numeric" }) });
}

function groupByDay(commits: IChangelogCommit[]): ICommitGroup[] {
    const today = startOfDay(new Date());
    const groups = new Map<number, IChangelogCommit[]>();
    for (const c of commits) {
        const key = startOfDay(new Date(c.date));
        const bucket = groups.get(key);
        if (bucket) bucket.push(c);
        else groups.set(key, [c]);
    }
    return [...groups.entries()].sort((a, b) => b[0] - a[0]).map(([key, list]) => ({ key: String(key), label: groupLabel(key, today), commits: list }));
}

function GithubMark({ className }: { className?: string }) {
    return (
        <svg viewBox="0 0 16 16" className={className} fill="currentColor" aria-hidden="true">
            <path d="M8 0C3.58 0 0 3.58 0 8c0 3.54 2.29 6.53 5.47 7.59.4.07.55-.17.55-.38 0-.19-.01-.82-.01-1.49-2.01.37-2.53-.49-2.69-.94-.09-.23-.48-.94-.82-1.13-.28-.15-.68-.52-.01-.53.63-.01 1.08.58 1.23.82.72 1.21 1.87.87 2.33.66.07-.52.28-.87.51-1.07-1.78-.2-3.64-.89-3.64-3.95 0-.87.31-1.59.82-2.15-.08-.2-.36-1.02.08-2.12 0 0 .67-.21 2.2.82.64-.18 1.32-.27 2-.27.68 0 1.36.09 2 .27 1.53-1.04 2.2-.82 2.2-.82.44 1.1.16 1.92.08 2.12.51.56.82 1.27.82 2.15 0 3.07-1.87 3.75-3.65 3.95.29.25.54.73.54 1.48 0 1.07-.01 1.93-.01 2.2 0 .21.15.46.55.38A8.01 8.01 0 0016 8c0-4.42-3.58-8-8-8z" />
        </svg>
    );
}

function StatCell({ icon, label, value }: { icon: React.ReactNode; label: string; value: React.ReactNode }) {
    return (
        <div className="flex items-center gap-2.5 px-4 py-3 max-[560px]:px-3.5">
            <span className="inline-flex size-8 shrink-0 items-center justify-center rounded-lg bg-primary/10 text-primary [&_svg]:size-4">{icon}</span>
            <div className="min-w-0">
                <div className="font-sans font-semibold text-[17px] text-foreground tabular-nums leading-none">{value}</div>
                <div className="mt-1 font-sans text-[12px] text-muted-foreground leading-none">{label}</div>
            </div>
        </div>
    );
}

export function ChangelogPage() {
    const { data, isLoading } = useQuery(changelogQueryOptions());
    const [picked, setPicked] = useState<ChangelogRangeId | null>(null);

    const commits = data?.commits ?? [];
    const counts = useMemo(() => Object.fromEntries(CHANGELOG_RANGES.map((r) => [r.id, commitsWithinDays(commits, r.days).length])) as Record<ChangelogRangeId, number>, [commits]);

    // Until the user picks a range, land on the narrowest window that actually
    // has commits, so the page never opens on an empty state when there's
    // recent history to show.
    const smartDefault = useMemo<ChangelogRangeId>(() => CHANGELOG_RANGES.find((r) => counts[r.id] > 0)?.id ?? CHANGELOG_RANGES[CHANGELOG_RANGES.length - 1].id, [counts]);
    const rangeId = picked ?? smartDefault;
    const range = CHANGELOG_RANGES.find((r) => r.id === rangeId) ?? CHANGELOG_RANGES[1];

    const filtered = useMemo(() => commitsWithinDays(commits, range.days), [commits, range.days]);
    const groups = useMemo(() => groupByDay(filtered), [filtered]);
    const contributors = useMemo(() => new Set(filtered.map((c) => c.author.login ?? c.author.name)).size, [filtered]);

    const repoLabel = data?.repo ?? "GitHub";
    const repoUrl = data?.repoUrl ?? "https://github.com";

    return (
        <main className="relative overflow-x-clip">
            <div className={styles.pageAmbient} aria-hidden="true" />
            <div className="mx-auto w-[min(820px,calc(100%-2rem))] py-12 sm:py-14">
                {/* Hero */}
                <header className="relative mb-8">
                    <div className={styles.heroGlow} aria-hidden="true" />
                    <div className="relative flex flex-col gap-4 sm:flex-row sm:items-start sm:justify-between">
                        <div className="min-w-0">
                            <Kicker className="mb-2 flex items-center gap-1.5">
                                <ScrollText className="size-3.5" strokeWidth={2.2} /> Changelog
                            </Kicker>
                            <h1 className="m-0 text-balance font-extrabold text-[clamp(30px,5vw,42px)] text-foreground leading-[1.05] tracking-[-0.02em]">What's shipped</h1>
                            <p className="m-0 mt-2.5 max-w-[52ch] font-sans text-[15.5px] text-muted-foreground leading-[1.55]">Every commit pushed to myrtle.moe, pulled live from GitHub and grouped by day.</p>
                        </div>
                        <a
                            href={repoUrl}
                            target="_blank"
                            rel="noreferrer"
                            className="inline-flex shrink-0 items-center gap-2 self-start rounded-lg border border-border bg-card px-3 py-2 font-medium font-sans text-[13.5px] text-foreground no-underline shadow-xs/5 transition-colors hover:border-border/70 hover:bg-accent/50"
                        >
                            <GithubMark className="size-4" />
                            <span className="font-mono">{repoLabel}</span>
                            <ExternalLink className="size-3.5 text-muted-foreground" strokeWidth={2} aria-hidden="true" />
                        </a>
                    </div>
                </header>

                {/* Summary + activity */}
                <section className="mb-7 overflow-hidden rounded-2xl border border-border bg-card">
                    <div className="grid grid-cols-3 divide-x divide-border max-[560px]:grid-cols-1 max-[560px]:divide-x-0 max-[560px]:divide-y">
                        <StatCell icon={<ScrollText strokeWidth={1.8} />} label={`commits · ${range.label.toLowerCase()}`} value={isLoading ? "-" : counts[range.id]} />
                        <StatCell icon={<Users strokeWidth={1.8} />} label={contributors === 1 ? "contributor" : "contributors"} value={isLoading ? "-" : contributors} />
                        <StatCell icon={data?.branch ? <GitBranch strokeWidth={1.8} /> : <CalendarClock strokeWidth={1.8} />} label={data?.branch ? "branch" : "last synced"} value={<span className="font-sans text-[14px]">{data?.branch ? data.branch : data?.fetchedAt ? formatRelative(data.fetchedAt) : "-"}</span>} />
                    </div>
                    {range.days >= 7 && filtered.length > 0 ? (
                        <div className="border-border border-t px-4 py-4 sm:px-5">
                            <ActivityStrip commits={filtered} days={range.days} />
                        </div>
                    ) : null}
                </section>

                {/* Range tabs */}
                <Tabs value={rangeId} onValueChange={(v) => setPicked(v as ChangelogRangeId)} className="mb-7">
                    <TabsList className="w-full max-[560px]:overflow-x-auto">
                        {CHANGELOG_RANGES.map((r) => (
                            <TabsTab key={r.id} value={r.id} className="gap-1.5">
                                <span className="max-[420px]:hidden">{r.label}</span>
                                <span className="min-[421px]:hidden">{r.shortLabel}</span>
                                <span className="rounded-full bg-muted-foreground/12 px-1.5 py-0.5 font-medium text-[11px] text-muted-foreground tabular-nums leading-none">{isLoading ? "·" : counts[r.id]}</span>
                            </TabsTab>
                        ))}
                    </TabsList>
                </Tabs>

                {/* Timeline */}
                {isLoading ? (
                    <TimelineSkeleton />
                ) : groups.length === 0 ? (
                    <EmptyState rangeLabel={range.label} />
                ) : (
                    <div className="flex flex-col gap-7">
                        {groups.map((group) => (
                            <section key={group.key}>
                                <div className="mb-3.5 flex items-center gap-3">
                                    <h2 className="m-0 font-sans font-semibold text-[14px] text-foreground leading-none tracking-[-0.01em]">{group.label}</h2>
                                    <span className="font-sans text-[12px] text-muted-foreground leading-none">
                                        {group.commits.length} commit{group.commits.length === 1 ? "" : "s"}
                                    </span>
                                    <span className="h-px flex-1 bg-border" aria-hidden="true" />
                                </div>
                                <ul className="m-0 list-none p-0">
                                    {group.commits.map((commit) => (
                                        <CommitItem key={commit.sha} commit={commit} />
                                    ))}
                                </ul>
                            </section>
                        ))}
                    </div>
                )}

                {data?.truncated && !isLoading ? <p className="mt-8 text-center font-sans text-[12.5px] text-muted-foreground">Older history is capped - see the full log on GitHub.</p> : null}

                <div className="mt-10 border-border border-t pt-6 text-center">
                    <Link to="/stats" className="font-sans text-[13px] text-primary no-underline hover:underline">
                        Looking for site stats instead? →
                    </Link>
                </div>
            </div>
        </main>
    );
}

function EmptyState({ rangeLabel }: { rangeLabel: string }) {
    return (
        <div className="flex flex-col items-center rounded-2xl border border-border border-dashed bg-card/40 px-6 py-14 text-center">
            <span className="mb-4 inline-flex size-12 items-center justify-center rounded-full bg-muted text-muted-foreground">
                <ScrollText className="size-6" strokeWidth={1.6} aria-hidden="true" />
            </span>
            <p className="m-0 font-sans font-semibold text-[16px] text-foreground">No commits in {rangeLabel.toLowerCase()}</p>
            <p className="m-0 mt-1.5 max-w-[40ch] font-sans text-[14px] text-muted-foreground leading-[1.55]">Nothing landed in this window yet. Try a wider range above to see recent work.</p>
        </div>
    );
}

function TimelineSkeleton() {
    return (
        <div className="flex flex-col gap-7" aria-hidden="true">
            {["a", "b"].map((g) => (
                <section key={g}>
                    <div className="mb-3.5 flex items-center gap-3">
                        <div className="h-3.5 w-20 animate-pulse rounded bg-muted" />
                        <span className="h-px flex-1 bg-border" />
                    </div>
                    <ul className="m-0 flex list-none flex-col gap-5 p-0">
                        {["x", "y", "z"].map((k) => (
                            <li key={k} className="grid grid-cols-[1.5rem_minmax(0,1fr)] gap-x-4">
                                <div className="flex justify-center pt-1">
                                    <span className="size-3.5 rounded-full bg-muted" />
                                </div>
                                <div className="rounded-xl border border-border bg-card px-4 py-3">
                                    <div className="h-4 w-16 animate-pulse rounded bg-muted" />
                                    <div className="mt-2.5 h-4 w-3/4 animate-pulse rounded bg-muted" />
                                    <div className="mt-3 h-3 w-1/3 animate-pulse rounded bg-muted" />
                                </div>
                            </li>
                        ))}
                    </ul>
                </section>
            ))}
        </div>
    );
}
