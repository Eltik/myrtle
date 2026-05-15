import { Link } from "@tanstack/react-router";
import { Kicker } from "#/components/ui/kicker";
import { formatNumber } from "#/lib/utils";

interface IHeroProps {
    total: number;
    canCreate: boolean;
}

export function Hero({ total, canCreate }: IHeroProps) {
    return (
        <section className="mx-auto w-[min(1080px,calc(100%-2rem))] pt-10 pb-6 sm:pt-14 sm:pb-8">
            <div className="flex flex-wrap items-end justify-between gap-4">
                <div className="min-w-0">
                    <Kicker>Tier Lists</Kicker>
                    <h1 className="m-0 font-bold font-sans text-3xl text-foreground leading-tight tracking-tight sm:text-4xl">Find the meta. Build your own.</h1>
                    <p className="mt-2 max-w-130 font-sans text-muted-foreground text-sm leading-relaxed">
                        {total > 0 ? (
                            <>
                                <span className="font-mono text-foreground tabular-nums">{formatNumber(total)}</span> {total === 1 ? "list" : "lists"} from the team and the community. Sort, filter, or browse what's hot right now.
                            </>
                        ) : (
                            <>Curated picks from the team alongside community-built rankings. Sort, filter, or browse what's hot right now.</>
                        )}
                    </p>
                </div>

                {canCreate ? (
                    <Link
                        to="/tier-lists/my"
                        search={{ sort: "recent", type: "all", view: "grid", q: "" }}
                        className="inline-flex items-center gap-1.5 rounded-lg border border-primary bg-primary px-3.5 py-2 font-medium font-sans text-primary-foreground text-sm leading-none no-underline shadow-xs transition-shadow hover:shadow-md"
                    >
                        <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round" className="h-4 w-4" aria-hidden="true">
                            <path d="M12 5v14" />
                            <path d="M5 12h14" />
                        </svg>
                        Create list
                    </Link>
                ) : (
                    <Link to="/login" search={{ redirect: "/tier-lists" }} className="inline-flex items-center gap-1.5 rounded-lg border border-border bg-muted px-3.5 py-2 font-medium font-sans text-foreground text-sm leading-none no-underline transition-colors hover:bg-accent">
                        Sign in to publish
                    </Link>
                )}
            </div>
        </section>
    );
}
