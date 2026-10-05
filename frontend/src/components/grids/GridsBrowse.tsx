import { keepPreviousData, useQuery } from "@tanstack/react-query";
import { Link, useNavigate } from "@tanstack/react-router";
import { ChevronLeftIcon, ChevronRightIcon, SearchIcon } from "lucide-react";
import { useEffect, useState } from "react";
import { Button } from "#/components/ui/button";
import { InputGroup, InputGroupAddon, InputGroupInput } from "#/components/ui/input-group";
import { Kicker } from "#/components/ui/kicker";
import { Skeleton } from "#/components/ui/skeleton";
import { useAuth } from "#/hooks/use-auth";
import { useDebounce } from "#/hooks/use-debounce";
import { browseGridsQueryOptions, type GridSort } from "#/lib/api/grids";
import { useFormatters, useGamedataServer, useT } from "#/lib/i18n";
import type { TypedT } from "#/lib/i18n/messages";
import { cn } from "#/lib/utils";
import { GridCard } from "./GridCard";
import type { messages } from "./GridsBrowse.messages";
import { NewGridButton } from "./NewGridButton";
import type { IGridsSearch } from "./shared";

const SORTS: GridSort[] = ["recent", "popular"];

export function GridsBrowse({ search }: { search: IGridsSearch }) {
    const t: TypedT<typeof messages> = useT("grids");
    const f = useFormatters();
    const navigate = useNavigate({ from: "/grids" });
    const server = useGamedataServer();
    const { user } = useAuth();
    const [input, setInput] = useState(search.q);
    const debounced = useDebounce(input.trim(), 250);

    const { data, isPending, isError, refetch, isFetching } = useQuery({ ...browseGridsQueryOptions({ ...search, server }), placeholderData: keepPreviousData });

    useEffect(() => {
        if (debounced === search.q) return;
        navigate({ search: (prev) => ({ ...prev, q: debounced, page: 1 }), replace: true, resetScroll: false });
    }, [debounced, navigate, search.q]);

    const totalPages = data ? Math.max(1, Math.ceil(Number(data.total) / Math.max(1, data.per_page))) : 1;
    const setSort = (sort: GridSort) => navigate({ search: (prev) => ({ ...prev, sort, page: 1 }), replace: true, resetScroll: false });
    const setPage = (page: number) => navigate({ search: (prev) => ({ ...prev, page }), resetScroll: true });

    return (
        <main className="min-h-dvh pb-24">
            <section className="page-gutter pt-10 pb-6 [--page-max:1080px] sm:pt-14 sm:pb-8">
                <div className="flex flex-wrap items-end justify-between gap-4">
                    <div className="min-w-0">
                        <Kicker>{t("browse.kicker")}</Kicker>
                        <h1 className="m-0 font-bold font-sans text-3xl text-foreground leading-tight tracking-tight sm:text-4xl">{t("browse.title")}</h1>
                        <p className="mt-2 max-w-130 font-sans text-muted-foreground text-sm leading-relaxed">{t("browse.blurb")}</p>
                    </div>
                    <div className="flex flex-wrap items-center gap-2">
                        {user && (
                            <Button render={<Link to="/grids/my" />} variant="outline">
                                {t("browse.mine")}
                            </Button>
                        )}
                        <NewGridButton />
                    </div>
                </div>
            </section>

            <div className="page-gutter [--page-max:1080px]">
                <div className="flex flex-col gap-3 sm:flex-row sm:items-center sm:justify-between">
                    <InputGroup className="sm:max-w-80">
                        <InputGroupAddon>
                            <SearchIcon aria-hidden="true" />
                        </InputGroupAddon>
                        <InputGroupInput value={input} onChange={(e) => setInput((e.target as HTMLInputElement).value)} placeholder={t("browse.searchPlaceholder")} type="search" aria-label={t("browse.searchLabel")} />
                    </InputGroup>
                    <fieldset className="m-0 inline-flex rounded-lg border-0 bg-muted p-0.5">
                        <legend className="sr-only">{t("browse.sortLabel")}</legend>
                        {SORTS.map((s) => (
                            <button
                                key={s}
                                type="button"
                                aria-pressed={search.sort === s}
                                onClick={() => setSort(s)}
                                className={cn(
                                    "relative inline-flex h-8 cursor-pointer items-center justify-center rounded-md px-3 font-medium font-sans text-sm transition-colors pointer-coarse:after:absolute pointer-coarse:after:size-full pointer-coarse:after:min-h-11",
                                    search.sort === s ? "bg-background text-foreground shadow-sm/5" : "text-muted-foreground hover:text-foreground",
                                )}
                            >
                                {s === "recent" ? t("browse.sort.recent") : t("browse.sort.popular")}
                            </button>
                        ))}
                    </fieldset>
                </div>

                <div className="mt-5">
                    {isPending ? (
                        <div className="grid grid-cols-1 gap-3 sm:grid-cols-2 lg:grid-cols-3">
                            {["s1", "s2", "s3", "s4", "s5", "s6"].map((k) => (
                                <Skeleton key={k} className="h-56 rounded-xl" />
                            ))}
                        </div>
                    ) : isError ? (
                        <div className="rounded-lg border border-border border-dashed bg-muted/20 px-5 py-10 text-center">
                            <p className="m-0 font-sans text-muted-foreground text-sm">{t("browse.error")}</p>
                            <Button type="button" variant="outline" size="sm" className="mt-3" onClick={() => void refetch()} loading={isFetching}>
                                {t("browse.retry")}
                            </Button>
                        </div>
                    ) : data.items.length === 0 ? (
                        <div className="rounded-lg border border-border border-dashed bg-muted/20 px-5 py-12 text-center">
                            <p className="m-0 font-medium font-sans text-foreground text-sm">{search.q ? t("browse.emptySearch") : t("browse.empty")}</p>
                            <p className="mt-1 font-sans text-[12.5px] text-muted-foreground">{t("browse.emptyHint")}</p>
                        </div>
                    ) : (
                        <>
                            <div className={cn("grid grid-cols-1 gap-3 sm:grid-cols-2 lg:grid-cols-3", isFetching && "opacity-70 transition-opacity")}>
                                {data.items.map((grid) => (
                                    <GridCard key={grid.slug} grid={grid} />
                                ))}
                            </div>
                            {totalPages > 1 && (
                                <nav className="mt-6 flex items-center justify-center gap-3" aria-label={t("browse.pagination")}>
                                    <Button type="button" variant="outline" size="sm" disabled={search.page <= 1} onClick={() => setPage(search.page - 1)}>
                                        <ChevronLeftIcon />
                                        {t("browse.prev")}
                                    </Button>
                                    <span className="font-mono text-muted-foreground text-xs tabular-nums">{t("browse.pageOf", { page: f.number(search.page), total: f.number(totalPages) })}</span>
                                    <Button type="button" variant="outline" size="sm" disabled={search.page >= totalPages} onClick={() => setPage(search.page + 1)}>
                                        {t("browse.next")}
                                        <ChevronRightIcon />
                                    </Button>
                                </nav>
                            )}
                        </>
                    )}
                </div>
            </div>
        </main>
    );
}
