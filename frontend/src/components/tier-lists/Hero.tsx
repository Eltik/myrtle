import { Link } from "@tanstack/react-router";
import { Kicker } from "#/components/ui/kicker";
import { formatNumber } from "#/lib/utils";

interface Props {
    total: number;
    canCreate: boolean;
}

export function Hero({ total, canCreate }: Props) {
    return (
        <section className="mx-auto w-[min(1080px,calc(100%-2rem))] pt-10 pb-6 sm:pt-14 sm:pb-8">
            <div className="flex flex-wrap items-end justify-between gap-4">
                <div className="min-w-0">
                    <Kicker>Tier Lists</Kicker>
                    <h1 className="m-0 font-sans text-3xl font-bold leading-tight tracking-tight text-foreground sm:text-4xl">Find the meta. Build your own.</h1>
                    <p className="mt-2 max-w-130 font-sans text-sm leading-relaxed text-muted-foreground">
                        {total > 0 ? (
                            <>
                                <span className="font-mono tabular-nums text-foreground">{formatNumber(total)}</span> {total === 1 ? "list" : "lists"} from the team and the community. Sort, filter, or browse what's hot right now.
                            </>
                        ) : (
                            <>Curated picks from the team alongside community-built rankings. Sort, filter, or browse what's hot right now.</>
                        )}
                    </p>
                </div>

                {canCreate ? (
                    <Link to="/" className="inline-flex items-center gap-1.5 rounded-lg border border-primary bg-primary px-3.5 py-2 font-sans text-sm font-medium leading-none text-primary-foreground no-underline shadow-xs transition-shadow hover:shadow-md">
                        <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round" className="h-4 w-4" aria-hidden="true">
                            <path d="M12 5v14" />
                            <path d="M5 12h14" />
                        </svg>
                        Create list
                    </Link>
                ) : (
                    <Link to="/login" search={{ redirect: "/tier-lists" }} className="inline-flex items-center gap-1.5 rounded-lg border border-border bg-muted px-3.5 py-2 font-sans text-sm font-medium leading-none text-foreground no-underline transition-colors hover:bg-accent">
                        Sign in to publish
                    </Link>
                )}
            </div>
        </section>
    );
}
