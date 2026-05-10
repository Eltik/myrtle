"use client";

import { keepPreviousData, useQuery } from "@tanstack/react-query";
import { useNavigate } from "@tanstack/react-router";
import * as React from "react";
import { Avatar, AvatarFallback, AvatarImage } from "#/components/ui/avatar";
import { Command, CommandDialog, CommandDialogPopup, CommandEmpty, CommandFooter, CommandGroup, CommandGroupLabel, CommandInput, CommandItem, CommandList, CommandPanel, CommandSeparator, CommandShortcut } from "#/components/ui/command";
import { Kbd } from "#/components/ui/kbd";
import { OperatorAvatar } from "#/components/ui/operator-avatar";
import { Skeleton } from "#/components/ui/skeleton";
import { useDebounce } from "#/hooks/use-debounce";
import { useIsMac } from "#/hooks/use-is-mac";
import { operatorsIndexQueryOptions } from "#/lib/api/operators";
import { searchUsersQueryOptions } from "#/lib/api/user";
import { professionClass, professionLabel } from "#/lib/registry/operator-display";
import { type IPage, PAGES } from "#/lib/registry/pages";
import { ToolIcon } from "#/lib/registry/ToolIcon";
import { type ITool, TOOLS, toolShortcut } from "#/lib/registry/tools";
import { searchAndRank } from "#/lib/search/fuzzy";
import { formatNumber, getAvatarById } from "#/lib/utils";
import type { IOperatorIndexEntry } from "#/types/operators";
import type { IUserProfile } from "#/types/user";

interface SearchCommandProps {
    open: boolean;
    onOpenChange: (open: boolean) => void;
}

const MAX_PAGES = 6;
const MAX_TOOLS = 6;
const MAX_OPERATORS = 8;
const MAX_PLAYERS = 6;
const PLAYER_QUERY_MIN = 2;
const PLAYER_DEBOUNCE_MS = 220;

export function SearchCommand({ open, onOpenChange }: SearchCommandProps): React.ReactElement {
    const [query, setQuery] = React.useState("");
    const navigate = useNavigate();
    const debouncedQuery = useDebounce(query.trim(), PLAYER_DEBOUNCE_MS);

    React.useEffect(() => {
        const down = (e: KeyboardEvent) => {
            if (e.key === "k" && (e.metaKey || e.ctrlKey)) {
                e.preventDefault();
                onOpenChange(!open);
            }
            if (e.key === "Escape") {
                onOpenChange(false);
            }
        };
        document.addEventListener("keydown", down);
        return () => document.removeEventListener("keydown", down);
    }, [open, onOpenChange]);

    React.useEffect(() => {
        if (!open) setQuery("");
    }, [open]);

    const operatorsQuery = useQuery({
        ...operatorsIndexQueryOptions(),
        enabled: open,
    });
    const operators = operatorsQuery.data;

    const playersEnabled = open && debouncedQuery.length >= PLAYER_QUERY_MIN;
    const playersQuery = useQuery({
        ...searchUsersQueryOptions({ q: debouncedQuery, limit: MAX_PLAYERS, offset: 0 }),
        enabled: playersEnabled,
        placeholderData: keepPreviousData,
    });

    const pageResults = React.useMemo(
        () =>
            searchAndRank(
                query,
                PAGES,
                (p) => ({
                    name: p.label,
                    extra: `${p.desc} ${p.keywords.join(" ")} ${p.id}`,
                }),
                MAX_PAGES,
            ),
        [query],
    );

    const toolResults = React.useMemo(
        () =>
            searchAndRank(
                query,
                TOOLS,
                (t) => ({
                    name: t.label,
                    extra: `${t.desc} ${t.keywords.join(" ")} ${t.id}`,
                }),
                MAX_TOOLS,
            ),
        [query],
    );

    const operatorResults = React.useMemo(() => {
        if (!operators) return [];
        return searchAndRank(
            query,
            operators,
            (op) => ({
                name: op.name,
                extra: `${op.appellation} ${professionLabel(op.profession)} ${op.subProfessionId} ${op.rarity}★ ${op.tagList.join(" ")} ${op.nationId}`,
            }),
            MAX_OPERATORS,
        );
    }, [operators, query]);

    const playerResults: IUserProfile[] = playersEnabled ? (playersQuery.data?.entries ?? []) : [];
    const playersLoading = playersEnabled && playersQuery.isLoading;
    const showPlayersGroup = playersEnabled;

    const closeAndGo = (href: string) => {
        onOpenChange(false);
        void navigate({ to: href });
    };

    const hasResults = pageResults.length > 0 || toolResults.length > 0 || operatorResults.length > 0 || playerResults.length > 0;
    const anyLoading = operatorsQuery.isLoading || playersLoading;

    return (
        <CommandDialog open={open} onOpenChange={onOpenChange}>
            <CommandDialogPopup>
                <Command value={query} onValueChange={setQuery} mode="none">
                    <CommandInput placeholder="Search players, operators, pages, tools…" />
                    <CommandPanel>
                        <CommandList>
                            {!hasResults && !anyLoading && <CommandEmpty>No results found.</CommandEmpty>}

                            {pageResults.length > 0 && (
                                <CommandGroup>
                                    <CommandGroupLabel>Pages</CommandGroupLabel>
                                    {pageResults.map(({ item: page }) => (
                                        <PageRow key={page.id} page={page} onClick={() => closeAndGo(page.href)} />
                                    ))}
                                </CommandGroup>
                            )}

                            {pageResults.length > 0 && toolResults.length > 0 && <CommandSeparator />}

                            {toolResults.length > 0 && (
                                <CommandGroup>
                                    <CommandGroupLabel>Tools</CommandGroupLabel>
                                    {toolResults.map(({ item: tool }) => (
                                        <ToolRow key={tool.id} tool={tool} onClick={() => closeAndGo(tool.href)} />
                                    ))}
                                </CommandGroup>
                            )}

                            {(pageResults.length > 0 || toolResults.length > 0) && showPlayersGroup && <CommandSeparator />}

                            {showPlayersGroup && (
                                <CommandGroup>
                                    <CommandGroupLabel>Players</CommandGroupLabel>
                                    {playersLoading && playerResults.length === 0 ? (
                                        <PlayerSkeletons />
                                    ) : playersQuery.isError ? (
                                        <div className="px-2 py-3 text-xs text-muted-foreground">Failed to load players.</div>
                                    ) : playerResults.length === 0 ? (
                                        <div className="px-2 py-3 text-xs text-muted-foreground">No doctors match "{debouncedQuery}".</div>
                                    ) : (
                                        playerResults.map((player) => <PlayerRow key={`${player.uid}-${player.server}`} player={player} onClick={() => closeAndGo(`/user/${player.uid}`)} />)
                                    )}
                                </CommandGroup>
                            )}

                            {(pageResults.length > 0 || toolResults.length > 0 || showPlayersGroup) && (operatorResults.length > 0 || operatorsQuery.isLoading) && <CommandSeparator />}

                            <CommandGroup>
                                <CommandGroupLabel>Operators</CommandGroupLabel>
                                {operatorsQuery.isLoading ? (
                                    <OperatorSkeletons />
                                ) : operatorsQuery.isError ? (
                                    <div className="px-2 py-3 text-xs text-muted-foreground">Failed to load operators. Try reopening the palette.</div>
                                ) : operatorResults.length === 0 ? (
                                    query.trim().length > 0 ? (
                                        <div className="px-2 py-3 text-xs text-muted-foreground">No operators match "{query}".</div>
                                    ) : null
                                ) : (
                                    operatorResults.map(({ item: op }) => <OperatorRow key={op.id} op={op} onClick={() => closeAndGo(`/operators/${op.id}`)} />)
                                )}
                            </CommandGroup>
                        </CommandList>
                    </CommandPanel>
                    <CommandFooter>
                        <span className="flex items-center gap-1">
                            <Kbd>↑</Kbd>
                            <Kbd>↓</Kbd>
                            to navigate
                        </span>
                        <span className="flex items-center gap-1">
                            <Kbd>↵</Kbd> to select
                        </span>
                        <span className="ml-auto font-mono text-[10.5px] tracking-[0.04em] text-muted-foreground/60">powered by COSS UI</span>
                    </CommandFooter>
                </Command>
            </CommandDialogPopup>
        </CommandDialog>
    );
}

function PageRow({ page, onClick }: { page: IPage; onClick: () => void }): React.ReactElement {
    return (
        <CommandItem value={`page:${page.id}`} onClick={onClick} className="flex cursor-pointer flex-row gap-2">
            <ToolIcon name={page.icon} className="size-4 text-muted-foreground" />
            <span className="flex-1">{page.label}</span>
            <span className="hidden text-xs text-muted-foreground sm:inline">{page.desc}</span>
        </CommandItem>
    );
}

function ToolRow({ tool, onClick }: { tool: ITool; onClick: () => void }): React.ReactElement {
    const isMac = useIsMac();
    const shortcut = toolShortcut(tool, isMac);
    return (
        <CommandItem value={`tool:${tool.id}`} onClick={onClick} className="flex cursor-pointer flex-row gap-2">
            <ToolIcon name={tool.icon} className="size-4 text-muted-foreground" />
            <span className="flex-1">{tool.label}</span>
            {shortcut && <CommandShortcut>{shortcut}</CommandShortcut>}
        </CommandItem>
    );
}

function OperatorRow({ op, onClick }: { op: IOperatorIndexEntry; onClick: () => void }): React.ReactElement {
    const cls = professionClass(op.profession);
    return (
        <CommandItem value={`operator:${op.id}`} onClick={onClick} className="flex cursor-pointer flex-row gap-2">
            <span className={`op-chip ${cls}`} aria-hidden="true">
                <OperatorAvatar charId={op.id} name={op.name} />
            </span>
            <span className="flex-1 font-medium">{op.name}</span>
            <span className="text-xs text-muted-foreground">
                {op.rarity}★ · {professionLabel(op.profession)}
            </span>
        </CommandItem>
    );
}

function PlayerRow({ player, onClick }: { player: IUserProfile; onClick: () => void }): React.ReactElement {
    const nickname = player.nickname ?? `Doctor ${player.uid}`;
    const initials = (player.nickname ?? player.uid).slice(0, 2).toUpperCase();
    const avatarSrc = player.avatar_id ? getAvatarById(player.avatar_id) : null;
    return (
        <CommandItem value={`player:${player.uid}-${player.server}`} onClick={onClick} className="flex cursor-pointer flex-row items-center gap-2">
            <Avatar className="size-6 rounded-md">
                {avatarSrc && <AvatarImage src={avatarSrc} alt="" />}
                <AvatarFallback className="rounded-md text-[10px]">{initials}</AvatarFallback>
            </Avatar>
            <span className="flex-1 truncate font-medium">{nickname}</span>
            <span className="flex shrink-0 items-center gap-1.5 text-xs text-muted-foreground">
                <span className="font-mono uppercase">{player.server}</span>
                {player.level != null && <span>Lv {player.level}</span>}
                {player.total_score != null && <span className="tabular-nums">{formatNumber(player.total_score)}</span>}
            </span>
        </CommandItem>
    );
}

function OperatorSkeletons(): React.ReactElement {
    return (
        <div className="space-y-1 py-1">
            {Array.from({ length: 4 }).map((_, i) => (
                // biome-ignore lint/suspicious/noArrayIndexKey: static placeholder list
                <div key={i} className="flex items-center gap-2 px-2 py-1.5">
                    <Skeleton className="size-6 rounded-md" />
                    <Skeleton className="h-3.5 flex-1 max-w-35" />
                    <Skeleton className="h-3 w-16" />
                </div>
            ))}
        </div>
    );
}

function PlayerSkeletons(): React.ReactElement {
    return (
        <div className="space-y-1 py-1">
            {Array.from({ length: 3 }).map((_, i) => (
                // biome-ignore lint/suspicious/noArrayIndexKey: static placeholder list
                <div key={i} className="flex items-center gap-2 px-2 py-1.5">
                    <Skeleton className="size-6 rounded-md" />
                    <Skeleton className="h-3.5 flex-1 max-w-40" />
                    <Skeleton className="h-3 w-20" />
                </div>
            ))}
        </div>
    );
}
