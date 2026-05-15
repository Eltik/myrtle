import { BellIcon, ChevronRightIcon, SearchIcon } from "lucide-react";
import { Fragment } from "react";

function GithubMark(): React.ReactElement {
    return (
        <svg viewBox="0 0 24 24" width={16} height={16} fill="none" stroke="currentColor" strokeWidth={1.9} strokeLinecap="round" strokeLinejoin="round" aria-hidden="true">
            <title>github</title>
            <path d="M15 22v-4a4.8 4.8 0 0 0-1-3.5c3 0 6-2 6-5.5.08-1.25-.27-2.48-1-3.5.28-1.15.28-2.35 0-3.5 0 0-1 0-3 1.5-2.64-.5-5.36-.5-8 0C6 2 5 2 5 2c-.3 1.15-.3 2.35 0 3.5A5.403 5.403 0 0 0 4 9c0 3.5 3 5.5 6 5.5-.39.49-.68 1.05-.85 1.65-.17.6-.22 1.23-.15 1.85v4" />
            <path d="M9 18c-4.51 2-5-2-7-2" />
        </svg>
    );
}

interface IAdminTopBarProps {
    crumbs: string[];
}

export function AdminTopBar({ crumbs }: IAdminTopBarProps): React.ReactElement {
    return (
        <div className="sticky top-0 z-40 border-border border-b bg-background/80 backdrop-blur-lg backdrop-saturate-150 supports-backdrop-filter:bg-background/60">
            <div className="flex h-14 items-center gap-3.5 px-6">
                <div className="flex min-w-0 flex-1 items-center gap-1.5 font-medium text-[13px] text-muted-foreground">
                    {crumbs.map((c, i) => (
                        // biome-ignore lint/suspicious/noArrayIndexKey: breadcrumb segments are positional
                        <Fragment key={i}>
                            <span className={i === crumbs.length - 1 ? "text-foreground" : ""}>{c}</span>
                            {i < crumbs.length - 1 ? <ChevronRightIcon className="size-3 opacity-50" strokeWidth={1.9} /> : null}
                        </Fragment>
                    ))}
                </div>
                <button type="button" className="inline-flex h-7.5 min-w-70 cursor-pointer items-center gap-2 rounded-lg border border-input bg-card px-2.5 font-normal text-[13px] text-muted-foreground hover:bg-accent">
                    <SearchIcon className="size-3.5" strokeWidth={1.9} />
                    <span className="flex-1 text-left">Search users, tier lists, operators…</span>
                    <span className="inline-flex h-4.25 items-center rounded border border-border border-b-[1.5px] bg-card px-1 font-medium font-mono text-[10px] text-muted-foreground">⌘</span>
                    <span className="inline-flex h-4.25 items-center rounded border border-border border-b-[1.5px] bg-card px-1 font-medium font-mono text-[10px] text-muted-foreground">K</span>
                </button>
                <div className="flex items-center gap-2">
                    <button type="button" className="inline-flex size-7.5 cursor-pointer items-center justify-center rounded-lg border border-transparent text-foreground hover:bg-accent" aria-label="Notifications">
                        <BellIcon className="size-4 opacity-85" strokeWidth={1.9} />
                    </button>
                    <a href="https://github.com/Eltik/myrtle" target="_blank" rel="noreferrer" className="inline-flex size-7.5 items-center justify-center rounded-lg border border-transparent text-foreground opacity-85 hover:bg-accent" aria-label="GitHub">
                        <GithubMark />
                    </a>
                </div>
            </div>
        </div>
    );
}
