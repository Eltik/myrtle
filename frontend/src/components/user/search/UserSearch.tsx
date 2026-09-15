import { useQuery } from "@tanstack/react-query";
import { useNavigate } from "@tanstack/react-router";
import { ChevronRight, Search, Trophy, Users, X } from "lucide-react";
import { useEffect, useRef, useState } from "react";
import { Pagination } from "#/components/operators/list/impl/components/Pagination";
import { Button } from "#/components/ui/button";
import { Empty, EmptyContent, EmptyDescription, EmptyHeader, EmptyMedia, EmptyTitle } from "#/components/ui/empty";
import { InputGroup, InputGroupAddon, InputGroupInput } from "#/components/ui/input-group";
import { useDebounce } from "#/hooks/use-debounce";
import { searchUsersQueryOptions } from "#/lib/api/user";
import { isEditableTarget, isPlainKey } from "#/lib/hotkeys";
import { type TypedRichT, useFormatters, useLocale, useRichT, useT } from "#/lib/i18n";
import type { TypedT } from "#/lib/i18n/messages";
import { Route } from "#/routes/user.search";
import { UserCard } from "./impl/components/UserCard";
import { UserGridSkeleton } from "./impl/components/UserGridSkeleton";
import { PAGE_SIZE } from "./impl/constants";
import { sortBySubstringMatch } from "./impl/sortBySubstringMatch";
import type { DisplayUser } from "./impl/types";
import type { messages } from "./UserSearch.messages";

export function UserSearch() {
    const t: TypedT<typeof messages> = useT("user");
    const rt: TypedRichT<typeof messages> = useRichT("user");
    const f = useFormatters();
    const locale = useLocale();
    const { q: initialQ, page: initialPage } = Route.useSearch();
    const navigate = useNavigate({ from: "/user/search" });

    const [inputValue, setInputValue] = useState(initialQ);
    const inputRef = useRef<HTMLInputElement>(null);

    const debouncedQuery = useDebounce(inputValue.trim(), 350);
    const isSearching = debouncedQuery.length > 0;

    const [currentPage, setCurrentPage] = useState(initialPage);

    const prevDebouncedRef = useRef(debouncedQuery);
    useEffect(() => {
        if (prevDebouncedRef.current !== debouncedQuery) {
            prevDebouncedRef.current = debouncedQuery;
            setCurrentPage(1);
        }
    }, [debouncedQuery]);

    useEffect(() => {
        navigate({ search: { q: inputValue.trim(), page: currentPage }, replace: true, resetScroll: false });
    }, [inputValue, currentPage, navigate]);

    useEffect(() => {
        const onKeyDown = (e: KeyboardEvent) => {
            if (e.key !== "/" || !isPlainKey(e) || e.shiftKey) return;
            if (isEditableTarget(e.target)) return;
            e.preventDefault();
            e.stopImmediatePropagation();
            inputRef.current?.focus();
            inputRef.current?.select();
        };
        document.addEventListener("keydown", onKeyDown, { capture: true });
        return () => document.removeEventListener("keydown", onKeyDown, { capture: true });
    }, []);

    const offset = (currentPage - 1) * PAGE_SIZE;
    const searchQuery = useQuery(searchUsersQueryOptions({ q: debouncedQuery || undefined, limit: PAGE_SIZE, offset }));

    const isLoading = searchQuery.isLoading || searchQuery.isFetching;
    const rawUsers: DisplayUser[] = searchQuery.data?.entries ?? [];
    const users: DisplayUser[] = isSearching ? sortBySubstringMatch(rawUsers, debouncedQuery, locale) : rawUsers;
    const totalCount = searchQuery.data?.total ?? null;
    const totalPages = Math.max(1, Math.ceil((totalCount ?? 0) / PAGE_SIZE));
    const showResults = !isLoading && users.length > 0;
    const showEmpty = !isLoading && users.length === 0;

    const handlePageChange = (page: number) => {
        setCurrentPage(page);
        if (typeof window !== "undefined") {
            window.scrollTo({ top: 0, behavior: "smooth" });
        }
    };

    const handleClear = () => {
        setInputValue("");
        inputRef.current?.focus();
    };

    return (
        <div className="relative z-1 mx-auto w-[min(1400px,calc(100%-2rem))] pb-20">
            <div className="pt-7 pb-1.5">
                <nav className="mb-2.5 flex items-center gap-1.5 font-medium font-sans text-[12px] text-muted-foreground leading-none" aria-label={t("search.breadcrumb.label")}>
                    <span>{t("search.breadcrumb.doctors")}</span>
                    <ChevronRight className="h-2.5 w-2.5" aria-hidden="true" />
                    <span className="text-foreground">{t("search.breadcrumb.current")}</span>
                </nav>
                <h1 className="m-0 font-bold font-sans text-[30px] text-foreground leading-[1.1] tracking-tight">{t("search.title")}</h1>
                <p className="mt-1.5 font-sans text-[13.5px] text-muted-foreground leading-normal">{t("search.subtitle")}</p>
            </div>

            <div className="flex flex-col gap-4 pt-5">
                <InputGroup className="max-w-xl">
                    <InputGroupAddon>
                        <Search aria-hidden="true" />
                    </InputGroupAddon>
                    <InputGroupInput ref={inputRef} value={inputValue} onChange={(e) => setInputValue(e.target.value)} placeholder={t("search.input.placeholder")} aria-label={t("search.input.label")} />
                    <InputGroupAddon align="inline-end">
                        {inputValue ? (
                            <Button variant="ghost" size="icon-xs" onClick={handleClear} aria-label={t("search.clearSearch")}>
                                <X aria-hidden="true" />
                            </Button>
                        ) : null}
                    </InputGroupAddon>
                </InputGroup>

                <div className="flex items-center justify-between gap-3 font-sans text-[12.5px] text-muted-foreground leading-none">
                    <span className="inline-flex items-center gap-1.5">
                        {isSearching ? (
                            <>
                                <Search className="h-3 w-3" aria-hidden="true" />
                                {t("search.resultsFor")} <strong className="text-foreground">{t("search.quotedQuery", { query: debouncedQuery })}</strong>
                            </>
                        ) : (
                            <>
                                <Trophy className="h-3 w-3" aria-hidden="true" />
                                {t("search.browsing")}
                            </>
                        )}
                    </span>
                    {totalCount !== null && (
                        <span className="hidden font-mono text-[11px] uppercase leading-none tracking-[0.08em] sm:inline">
                            <strong className="text-foreground">{f.number(totalCount)}</strong> {t("search.count.unit", { count: totalCount })}
                        </span>
                    )}
                </div>

                {isLoading ? (
                    <UserGridSkeleton />
                ) : showResults ? (
                    <div className="grid grid-cols-1 gap-3 sm:grid-cols-2 lg:grid-cols-3 xl:grid-cols-4">
                        {users.map((user) => (
                            <UserCard key={`${user.uid}-${user.server}`} user={user} />
                        ))}
                    </div>
                ) : showEmpty ? (
                    isSearching ? (
                        <Empty>
                            <EmptyHeader>
                                <EmptyMedia variant="icon">
                                    <Search aria-hidden="true" />
                                </EmptyMedia>
                                <EmptyTitle>{t("search.empty.noResults.title")}</EmptyTitle>
                                <EmptyDescription>{rt("search.empty.noResults.desc", { query: <span className="font-medium text-foreground">{t("search.quotedQuery", { query: debouncedQuery })}</span> })}</EmptyDescription>
                            </EmptyHeader>
                            <EmptyContent>
                                <Button variant="outline" size="sm" onClick={handleClear}>
                                    {t("search.clearSearch")}
                                </Button>
                            </EmptyContent>
                        </Empty>
                    ) : (
                        <Empty>
                            <EmptyHeader>
                                <EmptyMedia variant="icon">
                                    <Users aria-hidden="true" />
                                </EmptyMedia>
                                <EmptyTitle>{t("search.empty.none.title")}</EmptyTitle>
                                <EmptyDescription>{t("search.empty.none.desc")}</EmptyDescription>
                            </EmptyHeader>
                        </Empty>
                    )
                ) : null}

                {!isLoading && users.length > 0 && <Pagination currentPage={currentPage} totalPages={totalPages} onPageChange={handlePageChange} />}
            </div>
        </div>
    );
}
