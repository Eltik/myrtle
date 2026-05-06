import { useQuery } from "@tanstack/react-query";
import { Link, useNavigate } from "@tanstack/react-router";
import { ChevronRight, Search, Trophy, Users, X } from "lucide-react";
import { useEffect, useRef, useState } from "react";
import { Pagination } from "#/components/operators/list/impl/components/Pagination";
import { Avatar, AvatarFallback, AvatarImage } from "#/components/ui/avatar";
import { Badge } from "#/components/ui/badge";
import { Button } from "#/components/ui/button";
import { Card } from "#/components/ui/card";
import { Empty, EmptyContent, EmptyDescription, EmptyHeader, EmptyMedia, EmptyTitle } from "#/components/ui/empty";
import { InputGroup, InputGroupAddon, InputGroupInput } from "#/components/ui/input-group";
import { Skeleton } from "#/components/ui/skeleton";
import { searchUsersQueryOptions } from "#/lib/api/user";
import { formatNumber, getAvatarById } from "#/lib/utils";
import { Route } from "#/routes/user.search";
import type { IUserProfile } from "#/types/user";

const PAGE_SIZE = 24;
const DEFAULT_AVATAR_ID = "char_002_amiya";

type DisplayUser = Pick<IUserProfile, "uid" | "nickname" | "level" | "avatar_id" | "server" | "grade" | "total_score" | "operator_count" | "skin_count">;

function useDebounce<T>(value: T, delay: number): T {
    const [debounced, setDebounced] = useState(value);
    useEffect(() => {
        const id = setTimeout(() => setDebounced(value), delay);
        return () => clearTimeout(id);
    }, [value, delay]);
    return debounced;
}

export function UserSearch() {
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
        navigate({ search: { q: inputValue.trim(), page: currentPage }, replace: true });
    }, [inputValue, currentPage, navigate]);

    useEffect(() => {
        const onKeyDown = (e: KeyboardEvent) => {
            if ((e.metaKey || e.ctrlKey) && e.key.toLowerCase() === "k") {
                e.preventDefault();
                inputRef.current?.focus();
                inputRef.current?.select();
            }
        };
        window.addEventListener("keydown", onKeyDown);
        return () => window.removeEventListener("keydown", onKeyDown);
    }, []);

    const offset = (currentPage - 1) * PAGE_SIZE;
    const searchQuery = useQuery(searchUsersQueryOptions({ q: debouncedQuery || undefined, limit: PAGE_SIZE, offset }));

    const isLoading = searchQuery.isLoading || searchQuery.isFetching;
    const users: DisplayUser[] = searchQuery.data?.entries ?? [];
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
                <nav className="mb-2.5 flex items-center gap-1.5 font-sans text-[12px] font-medium leading-none text-muted-foreground" aria-label="Breadcrumb">
                    <span>Doctors</span>
                    <ChevronRight className="h-2.5 w-2.5" aria-hidden="true" />
                    <span className="text-foreground">Search</span>
                </nav>
                <h1 className="m-0 font-sans text-[30px] font-bold leading-[1.1] tracking-tight text-foreground">Search Doctors</h1>
                <p className="mt-1.5 font-sans text-[13.5px] leading-normal text-muted-foreground">Find Doctor profiles by nickname or browse public profiles ranked by score.</p>
            </div>

            <div className="flex flex-col gap-4 pt-5">
                <InputGroup className="max-w-xl">
                    <InputGroupAddon>
                        <Search aria-hidden="true" />
                    </InputGroupAddon>
                    <InputGroupInput ref={inputRef} value={inputValue} onChange={(e) => setInputValue(e.target.value)} placeholder="Search by nickname…" aria-label="Search doctors" />
                    <InputGroupAddon align="inline-end">
                        {inputValue ? (
                            <Button variant="ghost" size="icon-xs" onClick={handleClear} aria-label="Clear search">
                                <X aria-hidden="true" />
                            </Button>
                        ) : null}
                    </InputGroupAddon>
                </InputGroup>

                <div className="flex items-center justify-between gap-3 font-sans text-[12.5px] leading-none text-muted-foreground">
                    <span className="inline-flex items-center gap-1.5">
                        {isSearching ? (
                            <>
                                <Search className="h-3 w-3" aria-hidden="true" />
                                Results for <strong className="text-foreground">"{debouncedQuery}"</strong>
                            </>
                        ) : (
                            <>
                                <Trophy className="h-3 w-3" aria-hidden="true" />
                                Browsing public profiles by total score
                            </>
                        )}
                    </span>
                    {totalCount !== null && (
                        <span className="hidden font-mono text-[11px] uppercase leading-none tracking-[0.08em] sm:inline">
                            <strong className="text-foreground">{formatNumber(totalCount)}</strong> {totalCount === 1 ? "doctor" : "doctors"}
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
                                <EmptyTitle>No doctors found</EmptyTitle>
                                <EmptyDescription>
                                    No public profiles match <span className="font-medium text-foreground">"{debouncedQuery}"</span>. Try a different nickname.
                                </EmptyDescription>
                            </EmptyHeader>
                            <EmptyContent>
                                <Button variant="outline" size="sm" onClick={handleClear}>
                                    Clear search
                                </Button>
                            </EmptyContent>
                        </Empty>
                    ) : (
                        <Empty>
                            <EmptyHeader>
                                <EmptyMedia variant="icon">
                                    <Users aria-hidden="true" />
                                </EmptyMedia>
                                <EmptyTitle>No public profiles yet</EmptyTitle>
                                <EmptyDescription>Public Doctor profiles will appear here as players opt in.</EmptyDescription>
                            </EmptyHeader>
                        </Empty>
                    )
                ) : null}

                {!isLoading && users.length > 0 && <Pagination currentPage={currentPage} totalPages={totalPages} onPageChange={handlePageChange} />}
            </div>
        </div>
    );
}

function UserCard({ user }: { user: DisplayUser }) {
    const nickname = user.nickname ?? `Doctor ${user.uid}`;
    const initials = (user.nickname ?? user.uid).slice(0, 2).toUpperCase();
    const avatarSrc = getAvatarById(user.avatar_id ?? DEFAULT_AVATAR_ID);

    return (
        <Card className="group transition-shadow duration-150 hover:shadow-md">
            <Link to="/user/$id" params={{ id: user.uid }} className="flex items-center gap-3.5 px-4 py-3.5 no-underline">
                <Avatar className="h-14 w-14 shrink-0 rounded-xl border border-border transition-transform duration-200 group-hover:scale-105">
                    <AvatarImage src={avatarSrc} alt={nickname} />
                    <AvatarFallback className="rounded-xl text-sm">{initials}</AvatarFallback>
                </Avatar>

                <div className="min-w-0 flex-1">
                    <div className="flex items-center gap-1.5 truncate">
                        <span className="truncate font-sans text-[14px] font-semibold leading-snug text-foreground transition-colors duration-150 group-hover:text-primary">{nickname}</span>
                        {user.grade && (
                            <Badge variant="outline" size="sm" className="font-mono">
                                {user.grade}
                            </Badge>
                        )}
                    </div>

                    <div className="mt-1 flex flex-wrap items-center gap-x-2 gap-y-0.5">
                        <span className="font-mono text-[11px] leading-none text-muted-foreground tabular-nums">{user.uid}</span>
                        <Badge variant="secondary" size="sm" className="font-mono uppercase">
                            {user.server}
                        </Badge>
                    </div>

                    <div className="mt-1.5 flex items-center gap-2.5 font-sans text-[11.5px] leading-none text-muted-foreground">
                        {user.level != null && (
                            <span>
                                <span className="font-medium text-foreground">Lv {user.level}</span>
                            </span>
                        )}
                        {user.total_score != null && (
                            <span>
                                <span className="font-medium text-foreground">{formatNumber(user.total_score)}</span> pts
                            </span>
                        )}
                        {user.operator_count != null && (
                            <span>
                                <span className="font-medium text-foreground">{formatNumber(user.operator_count)}</span> ops
                            </span>
                        )}
                        {user.skin_count != null && (
                            <span>
                                <span className="font-medium text-foreground">{formatNumber(user.skin_count)}</span> skins
                            </span>
                        )}
                    </div>
                </div>

                <ChevronRight className="h-3.5 w-3.5 shrink-0 text-muted-foreground/50 transition-transform duration-150 group-hover:translate-x-0.5" aria-hidden="true" />
            </Link>
        </Card>
    );
}

function UserGridSkeleton() {
    return (
        <div className="grid grid-cols-1 gap-3 sm:grid-cols-2 lg:grid-cols-3 xl:grid-cols-4">
            {Array.from({ length: PAGE_SIZE }, (_, i) => `skel-${i}`).map((id) => (
                <UserCardSkeleton key={id} />
            ))}
        </div>
    );
}

function UserCardSkeleton() {
    return (
        <Card>
            <div className="flex items-center gap-3.5 px-4 py-3.5">
                <Skeleton className="h-14 w-14 shrink-0 rounded-xl" />
                <div className="min-w-0 flex-1 flex flex-col gap-2">
                    <div className="flex items-center gap-1.5">
                        <Skeleton className="h-3.5 w-28 rounded-md" />
                        <Skeleton className="h-4 w-8 rounded-full" />
                    </div>
                    <div className="flex items-center gap-1.5">
                        <Skeleton className="h-2.5 w-16 rounded" />
                        <Skeleton className="h-3.5 w-10 rounded-sm" />
                    </div>
                    <div className="flex items-center gap-2">
                        <Skeleton className="h-2.5 w-12 rounded" />
                        <Skeleton className="h-2.5 w-20 rounded" />
                    </div>
                </div>
            </div>
        </Card>
    );
}
