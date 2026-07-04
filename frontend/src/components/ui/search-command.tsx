"use client";

import { useQuery } from "@tanstack/react-query";
import { useNavigate } from "@tanstack/react-router";
import * as React from "react";
import { Command, CommandDialog, CommandDialogPopup, CommandEmpty, CommandFooter, CommandGroup, CommandGroupLabel, CommandInput, CommandItem, CommandList, CommandPanel, CommandSeparator } from "#/components/ui/command";
import { Kbd } from "#/components/ui/kbd";
import { OperatorAvatar } from "#/components/ui/operator-avatar";
import { Skeleton } from "#/components/ui/skeleton";
import { operatorsIndexQueryOptions } from "#/lib/api/operators";
import { hasMod, isEditableTarget } from "#/lib/hotkeys";
import { professionClass, professionLabel } from "#/lib/registry/operator-display";
import { type IPage, PAGES } from "#/lib/registry/pages";
import { ToolIcon } from "#/lib/registry/ToolIcon";
import { type ITool, TOOLS } from "#/lib/registry/tools";
import { searchAndRank } from "#/lib/search/fuzzy";
import type { IOperatorIndexEntry } from "#/types/operators";

interface ISearchCommandProps {
    open: boolean;
    onOpenChange: (open: boolean) => void;
}

const MAX_PAGES = 6;
const MAX_TOOLS = 6;
const MAX_OPERATORS = 8;

export function SearchCommand({ open, onOpenChange }: ISearchCommandProps): React.ReactElement {
    const [query, setQuery] = React.useState("");
    const navigate = useNavigate();

    React.useEffect(() => {
        const down = (e: KeyboardEvent) => {
            if (e.key === "Escape") {
                onOpenChange(false);
                return;
            }
            if (e.key.toLowerCase() === "k" && hasMod(e)) {
                e.preventDefault();
                onOpenChange(!open);
                return;
            }
            if (e.key === "/" && !hasMod(e) && !e.altKey && !e.shiftKey && !isEditableTarget(e.target)) {
                e.preventDefault();
                onOpenChange(!open);
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

    const closeAndGo = (href: string) => {
        onOpenChange(false);
        void navigate({ to: href });
    };

    const hasResults = pageResults.length > 0 || toolResults.length > 0 || operatorResults.length > 0;
    const anyLoading = operatorsQuery.isLoading;

    return (
        <CommandDialog open={open} onOpenChange={onOpenChange}>
            <CommandDialogPopup>
                <Command value={query} onValueChange={setQuery} mode="none">
                    <CommandInput placeholder="Search operators, pages, tools…" />
                    <CommandPanel>
                        <CommandList>
                            {!hasResults && !anyLoading && <CommandEmpty>No results found.</CommandEmpty>}

                            <CommandGroup>
                                <CommandGroupLabel>Operators</CommandGroupLabel>
                                {operatorsQuery.isLoading ? (
                                    <OperatorSkeletons />
                                ) : operatorsQuery.isError ? (
                                    <div className="px-2 py-3 text-muted-foreground text-xs">Failed to load operators. Try reopening the palette.</div>
                                ) : operatorResults.length === 0 ? (
                                    query.trim().length > 0 ? (
                                        <div className="px-2 py-3 text-muted-foreground text-xs">No operators match "{query}".</div>
                                    ) : null
                                ) : (
                                    operatorResults.map(({ item: op }) => <OperatorRow key={op.id} op={op} onClick={() => closeAndGo(`/operators/${op.id}`)} />)
                                )}
                            </CommandGroup>

                            {(operatorResults.length > 0 || operatorsQuery.isLoading) && pageResults.length > 0 && <CommandSeparator />}

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
                        </CommandList>
                    </CommandPanel>
                    <CommandFooter>
                        <span className="flex pointer-coarse:hidden items-center gap-1">
                            <Kbd>↑</Kbd>
                            <Kbd>↓</Kbd>
                            to navigate
                        </span>
                        <span className="flex pointer-coarse:hidden items-center gap-1">
                            <Kbd>↵</Kbd> to select
                        </span>
                        <span className="ml-auto font-mono text-[10.5px] text-muted-foreground/60 tracking-[0.04em]">powered by COSS UI</span>
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
            <span className="hidden text-muted-foreground text-xs sm:inline">{page.desc}</span>
        </CommandItem>
    );
}

function ToolRow({ tool, onClick }: { tool: ITool; onClick: () => void }): React.ReactElement {
    return (
        <CommandItem value={`tool:${tool.id}`} onClick={onClick} className="flex cursor-pointer flex-row gap-2">
            <ToolIcon name={tool.icon} className="size-4 text-muted-foreground" />
            <span className="flex-1">{tool.label}</span>
            <span className="hidden text-muted-foreground text-xs sm:inline">{tool.desc}</span>
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
            <span className="text-muted-foreground text-xs">
                {op.rarity}★ · {professionLabel(op.profession)}
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
                    <Skeleton className="h-3.5 max-w-35 flex-1" />
                    <Skeleton className="h-3 w-16" />
                </div>
            ))}
        </div>
    );
}
