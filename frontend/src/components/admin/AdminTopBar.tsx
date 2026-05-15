import { BellIcon, ChevronRightIcon, MenuIcon, SearchIcon } from "lucide-react";
import { Fragment } from "react";
import { useIsMac } from "#/hooks/use-is-mac";
import { useCommand } from "#/lib/command-context";

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
    onOpenSidebar: () => void;
}

export function AdminTopBar({ crumbs, onOpenSidebar }: IAdminTopBarProps): React.ReactElement {
    const { open: openCmd } = useCommand();
    const isMac = useIsMac();
    const modKey = isMac ? "⌘" : "Ctrl";

    return (
        <div className="sticky top-0 z-30 border-border border-b bg-background/80 backdrop-blur-lg backdrop-saturate-150 supports-backdrop-filter:bg-background/60">
            <div className="flex h-14 items-center gap-2 px-3 sm:gap-3.5 sm:px-4 lg:px-6">
                <button type="button" aria-label="Open menu" onClick={onOpenSidebar} className="-ml-1 inline-flex size-8 shrink-0 cursor-pointer items-center justify-center rounded-lg border border-transparent text-foreground hover:bg-accent lg:hidden">
                    <MenuIcon className="size-4.5 opacity-85" strokeWidth={1.9} />
                </button>

                <div className="hidden min-w-0 flex-1 items-center gap-1.5 font-medium text-[13px] text-muted-foreground sm:flex">
                    {crumbs.map((c, i) => (
                        // biome-ignore lint/suspicious/noArrayIndexKey: breadcrumb segments are positional
                        <Fragment key={i}>
                            <span className={i === crumbs.length - 1 ? "truncate text-foreground" : "hidden truncate md:inline"}>{c}</span>
                            {i < crumbs.length - 1 ? <ChevronRightIcon className="hidden size-3 shrink-0 opacity-50 md:inline" strokeWidth={1.9} /> : null}
                        </Fragment>
                    ))}
                </div>

                {/* Mobile breadcrumb: show only current page */}
                <span className="min-w-0 flex-1 truncate font-medium text-[13.5px] text-foreground sm:hidden">{crumbs[crumbs.length - 1] ?? ""}</span>

                <button type="button" onClick={openCmd} aria-label="Open search" className="inline-flex h-8 shrink-0 cursor-pointer items-center justify-center rounded-lg border border-input bg-card px-2 font-normal text-[13px] text-muted-foreground hover:bg-accent sm:h-7.5 sm:gap-2 sm:px-2.5 md:min-w-60 lg:min-w-70">
                    <SearchIcon className="size-3.5" strokeWidth={1.9} />
                    <span className="hidden flex-1 text-left sm:inline">Search users, tier lists, operators…</span>
                    <span className="hidden h-4.25 items-center rounded border border-border border-b-[1.5px] bg-card px-1 font-medium font-mono text-[10px] text-muted-foreground sm:inline-flex">{modKey}</span>
                    <span className="hidden h-4.25 items-center rounded border border-border border-b-[1.5px] bg-card px-1 font-medium font-mono text-[10px] text-muted-foreground sm:inline-flex">K</span>
                </button>

                <div className="flex items-center gap-1 sm:gap-2">
                    <button type="button" className="inline-flex size-8 cursor-pointer items-center justify-center rounded-lg border border-transparent text-foreground hover:bg-accent sm:size-7.5" aria-label="Notifications">
                        <BellIcon className="size-4 opacity-85" strokeWidth={1.9} />
                    </button>
                    <a href="https://github.com/Eltik/myrtle" target="_blank" rel="noreferrer" className="inline-flex size-8 items-center justify-center rounded-lg border border-transparent text-foreground opacity-85 hover:bg-accent sm:size-7.5" aria-label="GitHub">
                        <GithubMark />
                    </a>
                </div>
            </div>
        </div>
    );
}
