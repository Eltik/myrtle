import { queryOptions } from "@tanstack/react-query";
import { createServerFn } from "@tanstack/react-start";
import { env } from "#/env";

/** Conventional-commit category, normalized into a small display-friendly set. */
export type CommitType = "feature" | "fix" | "perf" | "refactor" | "docs" | "style" | "test" | "chore" | "revert" | "other";

export interface IChangelogAuthor {
    name: string;
    login: string | null;
    avatarUrl: string | null;
    profileUrl: string | null;
}

export interface IChangelogCommit {
    sha: string;
    shortSha: string;
    /** First line of the message, with the conventional-commit prefix stripped. */
    title: string;
    /** Remaining lines of the message, with trailer lines (Co-authored-by, etc.) removed. */
    body: string;
    type: CommitType;
    scope: string | null;
    /** Whether the commit subject was flagged as a breaking change (`feat!:`). */
    breaking: boolean;
    url: string;
    /** ISO author date. */
    date: string;
    author: IChangelogAuthor;
    verified: boolean;
}

export interface IChangelogResponse {
    /** Normalized "owner/repo". */
    repo: string;
    repoUrl: string;
    /** Pinned branch, or null when the repo's default branch is used. */
    branch: string | null;
    commits: IChangelogCommit[];
    fetchedAt: string;
    /** True when the fetch window hit a cap and older commits were dropped. */
    truncated: boolean;
}

/** Time windows offered on the changelog page. Sorted ascending by span. */
export const CHANGELOG_RANGES = [
    { id: "day", label: "Today", shortLabel: "24h", days: 1 },
    { id: "week", label: "This Week", shortLabel: "7d", days: 7 },
    { id: "month", label: "This Month", shortLabel: "30d", days: 30 },
    { id: "quarter", label: "3 Months", shortLabel: "90d", days: 90 },
] as const;

export type ChangelogRangeId = (typeof CHANGELOG_RANGES)[number]["id"];

/** Widest window we fetch up front; every range filters down from this client-side. */
const WINDOW_DAYS = 90;
const MAX_COMMITS = 300;
const MAX_PAGES = 3;
const PER_PAGE = 100;
const CACHE_TTL_MS = 5 * 60 * 1000;
const DAY_MS = 86_400_000;

const TRAILER_PATTERN = /^(?:co-authored-by|signed-off-by|reviewed-by|acked-by|tested-by|reported-by|suggested-by|cc):/i;
const SUBJECT_PATTERN = /^(\w+)(?:\(([^)]*)\))?(!)?:\s*(.*)$/;
const TYPE_ALIASES: Record<string, CommitType> = {
    feat: "feature",
    feature: "feature",
    fix: "fix",
    bug: "fix",
    bugfix: "fix",
    hotfix: "fix",
    perf: "perf",
    performance: "perf",
    refactor: "refactor",
    refac: "refactor",
    docs: "docs",
    doc: "docs",
    style: "style",
    test: "test",
    tests: "test",
    chore: "chore",
    build: "chore",
    ci: "chore",
    deps: "chore",
    revert: "revert",
};

interface IGitHubCommit {
    sha: string;
    html_url: string;
    commit: {
        message: string;
        author: { name?: string; date?: string } | null;
        committer: { name?: string; date?: string } | null;
        verification?: { verified?: boolean } | null;
    };
    author: { login?: string; avatar_url?: string; html_url?: string } | null;
}

function parseRepo(input: string): { owner: string; repo: string } | null {
    const cleaned = input
        .trim()
        .replace(/^https?:\/\/github\.com\//i, "")
        .replace(/\.git$/i, "")
        .replace(/\/+$/, "");
    const [owner, repo] = cleaned.split("/");
    if (!owner || !repo) return null;
    return { owner, repo };
}

function classifySubject(subject: string): { type: CommitType; scope: string | null; title: string; breaking: boolean } {
    const match = subject.match(SUBJECT_PATTERN);
    if (!match) return { type: "other", scope: null, title: subject, breaking: false };
    const type = TYPE_ALIASES[match[1].toLowerCase()];
    if (!type) return { type: "other", scope: null, title: subject, breaking: false };
    return { type, scope: match[2]?.trim() || null, title: match[4] || subject, breaking: match[3] === "!" };
}

function cleanBody(lines: string[]): string {
    // Drop git trailers (Co-authored-by, Signed-off-by, …) and surrounding blanks.
    const kept = lines.filter((line) => !TRAILER_PATTERN.test(line.trim()));
    return kept.join("\n").trim();
}

function normalize(commit: IGitHubCommit): IChangelogCommit {
    const message = commit.commit?.message ?? "";
    const [subjectLine = "", ...rest] = message.split("\n");
    const { type, scope, title, breaking } = classifySubject(subjectLine.trim());
    const date = commit.commit?.author?.date ?? commit.commit?.committer?.date ?? new Date().toISOString();
    return {
        sha: commit.sha,
        shortSha: commit.sha.slice(0, 7),
        title,
        body: cleanBody(rest),
        type,
        scope,
        breaking,
        url: commit.html_url,
        date,
        author: {
            name: commit.commit?.author?.name ?? commit.author?.login ?? "Unknown",
            login: commit.author?.login ?? null,
            avatarUrl: commit.author?.avatar_url ?? null,
            profileUrl: commit.author?.html_url ?? null,
        },
        verified: Boolean(commit.commit?.verification?.verified),
    };
}

// Module-level cache: GitHub allows only 60 unauthenticated req/hr per IP, and
// every SSR render would otherwise hit the API. One entry covers the whole site.
let cache: { key: string; at: number; data: IChangelogResponse } | null = null;

export const getChangelogFn = createServerFn({ method: "GET" }).handler(async (): Promise<IChangelogResponse> => {
    const parsed = parseRepo(env.GITHUB_REPO ?? "");
    if (!parsed) throw new Error(`Invalid GITHUB_REPO: "${env.GITHUB_REPO}". Expected "owner/repo" or a GitHub URL.`);
    const { owner, repo } = parsed;
    const branch = env.GITHUB_BRANCH?.trim() || null;
    const cacheKey = `${owner}/${repo}@${branch ?? "default"}`;

    if (cache && cache.key === cacheKey && Date.now() - cache.at < CACHE_TTL_MS) return cache.data;

    const since = new Date(Date.now() - WINDOW_DAYS * DAY_MS).toISOString();
    const headers: Record<string, string> = {
        Accept: "application/vnd.github+json",
        "X-GitHub-Api-Version": "2022-11-28",
        "User-Agent": "myrtle.moe-changelog",
    };
    if (env.GITHUB_TOKEN) headers.Authorization = `Bearer ${env.GITHUB_TOKEN}`;

    const collected: IGitHubCommit[] = [];
    let truncated = false;
    for (let page = 1; page <= MAX_PAGES; page++) {
        const params = new URLSearchParams({ per_page: String(PER_PAGE), since, page: String(page) });
        if (branch) params.set("sha", branch);
        const res = await fetch(`https://api.github.com/repos/${owner}/${repo}/commits?${params}`, { headers });
        if (!res.ok) {
            // A later page failing shouldn't nuke commits we already have.
            if (collected.length > 0) {
                truncated = true;
                break;
            }
            const detail = (await res.text().catch(() => "")).slice(0, 200);
            const hint = res.status === 403 ? " (rate limited - set GITHUB_TOKEN)" : "";
            throw new Error(`GitHub API ${res.status} ${res.statusText}${hint}${detail ? `: ${detail}` : ""}`);
        }
        const batch = (await res.json()) as IGitHubCommit[];
        collected.push(...batch);
        if (batch.length < PER_PAGE) break;
        if (collected.length >= MAX_COMMITS) {
            truncated = true;
            break;
        }
    }

    const data: IChangelogResponse = {
        repo: `${owner}/${repo}`,
        repoUrl: `https://github.com/${owner}/${repo}`,
        branch,
        commits: collected.slice(0, MAX_COMMITS).map(normalize),
        fetchedAt: new Date().toISOString(),
        truncated,
    };
    cache = { key: cacheKey, at: Date.now(), data };
    return data;
});

export function changelogQueryOptions() {
    return queryOptions({
        queryKey: ["changelog"],
        queryFn: () => getChangelogFn(),
        staleTime: CACHE_TTL_MS,
        gcTime: 60 * 60 * 1000,
    });
}

/** Commits within the last `days`, relative to now. Input is assumed newest-first. */
export function commitsWithinDays(commits: IChangelogCommit[], days: number): IChangelogCommit[] {
    const cutoff = Date.now() - days * DAY_MS;
    return commits.filter((c) => new Date(c.date).getTime() >= cutoff);
}
