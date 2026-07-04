import { Link } from "@tanstack/react-router";
import { BellIcon, ChevronRightIcon, MenuIcon, SearchIcon } from "lucide-react";
import { Fragment } from "react";
import { GithubIcon } from "#/components/ui/github-icon";
import { useIsMac } from "#/hooks/use-is-mac";
import { useCommand } from "#/lib/command-context";
import { REPO_URL } from "#/lib/constants";

export interface IAdminCrumb {
    label: string;
    to?: string;
    href?: string;
}

interface IAdminTopBarProps {
    crumbs: IAdminCrumb[];
    onOpenSidebar: () => void;
}

function CrumbLabel({ crumb, isLast }: { crumb: IAdminCrumb; isLast: boolean }): React.ReactElement {
    const className = isLast ? "truncate text-foreground" : "hidden truncate text-muted-foreground hover:text-foreground md:inline";
    if (!isLast && crumb.to) {
        return (
            <Link to={crumb.to} className={`${className} transition-colors`}>
                {crumb.label}
            </Link>
        );
    }
    if (!isLast && crumb.href) {
        return (
            <a href={crumb.href} className={`${className} transition-colors`}>
                {crumb.label}
            </a>
        );
    }
    return <span className={className}>{crumb.label}</span>;
}

export function AdminTopBar({ crumbs, onOpenSidebar }: IAdminTopBarProps): React.ReactElement {
    const { open: openCmd } = useCommand();
    const isMac = useIsMac();
    const modKey = isMac ? "⌘" : "Ctrl";
    const lastCrumb = crumbs[crumbs.length - 1];

    return (
        <div className="sticky top-0 z-30 border-border border-b bg-background/80 backdrop-blur-lg backdrop-saturate-150 supports-backdrop-filter:bg-background/60">
            <div className="flex h-14 items-center gap-2 px-3 sm:gap-3.5 sm:px-4 lg:px-6">
                <button type="button" aria-label="Open menu" onClick={onOpenSidebar} className="-ml-1 inline-flex size-8 shrink-0 cursor-pointer items-center justify-center rounded-lg border border-transparent text-foreground hover:bg-accent lg:hidden">
                    <MenuIcon className="size-4.5 opacity-85" strokeWidth={1.9} />
                </button>

                <nav aria-label="Breadcrumb" className="hidden min-w-0 flex-1 items-center gap-1.5 font-medium text-[13px] sm:flex">
                    {crumbs.map((c, i) => (
                        // biome-ignore lint/suspicious/noArrayIndexKey: breadcrumb segments are positional
                        <Fragment key={i}>
                            <CrumbLabel crumb={c} isLast={i === crumbs.length - 1} />
                            {i < crumbs.length - 1 ? <ChevronRightIcon className="hidden size-3 shrink-0 opacity-50 md:inline" strokeWidth={1.9} /> : null}
                        </Fragment>
                    ))}
                </nav>

                {/* Mobile breadcrumb: show only current page */}
                <span className="min-w-0 flex-1 truncate font-medium text-[13.5px] text-foreground sm:hidden">{lastCrumb?.label ?? ""}</span>

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
                    <a href={REPO_URL} target="_blank" rel="noreferrer" className="inline-flex size-8 items-center justify-center rounded-lg border border-transparent text-foreground opacity-85 hover:bg-accent sm:size-7.5" aria-label="GitHub">
                        <GithubIcon className="size-4" strokeWidth={1.9} />
                    </a>
                </div>
            </div>
        </div>
    );
}
