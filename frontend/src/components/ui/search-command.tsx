"use client";

import { useQuery } from "@tanstack/react-query";
import { useNavigate } from "@tanstack/react-router";
import * as React from "react";
import { Command, CommandDialog, CommandDialogPopup, CommandEmpty, CommandFooter, CommandGroup, CommandGroupLabel, CommandInput, CommandItem, CommandList, CommandPanel, CommandSeparator, CommandShortcut } from "#/components/ui/command";
import { Kbd } from "#/components/ui/kbd";
import { OperatorAvatar } from "#/components/ui/operator-avatar";
import { Skeleton } from "#/components/ui/skeleton";
import { operatorsIndexQueryOptions } from "#/lib/api/operators";
import type { IOperatorIndexEntry } from "#/types/operators";
import { professionClass, professionLabel } from "#/lib/registry/operator-display";
import { ToolIcon } from "#/lib/registry/ToolIcon";
import { TOOLS, type Tool } from "#/lib/registry/tools";
import { searchAndRank } from "#/lib/search/fuzzy";

interface SearchCommandProps {
    open: boolean;
    onOpenChange: (open: boolean) => void;
}

const MAX_OPERATORS = 8;
const MAX_TOOLS = 6;

export function SearchCommand({ open, onOpenChange }: SearchCommandProps): React.ReactElement {
    const [query, setQuery] = React.useState("");
    const navigate = useNavigate();

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

    const hasResults = toolResults.length > 0 || operatorResults.length > 0;

    return (
        <CommandDialog open={open} onOpenChange={onOpenChange}>
            <CommandDialogPopup>
                <Command value={query} onValueChange={setQuery} mode="none">
                    <CommandInput placeholder="Search operators, tools, actions…" />
                    <CommandPanel>
                        <CommandList>
                            {!hasResults && !operatorsQuery.isLoading && <CommandEmpty>No results found.</CommandEmpty>}

                            {toolResults.length > 0 && (
                                <CommandGroup>
                                    <CommandGroupLabel>Tools</CommandGroupLabel>
                                    {toolResults.map(({ item: tool }) => (
                                        <ToolRow key={tool.id} tool={tool} onSelect={() => closeAndGo(tool.href)} />
                                    ))}
                                </CommandGroup>
                            )}

                            {toolResults.length > 0 && (operatorResults.length > 0 || operatorsQuery.isLoading) && <CommandSeparator />}

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
                                    operatorResults.map(({ item: op }) => <OperatorRow key={op.id} op={op} onSelect={() => closeAndGo(`/operators/${op.id}`)} />)
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

function ToolRow({ tool, onSelect }: { tool: Tool; onSelect: () => void }): React.ReactElement {
    return (
        <CommandItem value={`tool:${tool.id}`} onSelect={onSelect} className="flex flex-row gap-2">
            <ToolIcon name={tool.icon} className="size-4 text-muted-foreground" />
            <span className="flex-1">{tool.label}</span>
            {tool.shortcut && <CommandShortcut>{tool.shortcut}</CommandShortcut>}
        </CommandItem>
    );
}

function OperatorRow({ op, onSelect }: { op: IOperatorIndexEntry; onSelect: () => void }): React.ReactElement {
    const cls = professionClass(op.profession);
    return (
        <CommandItem value={`operator:${op.id}`} onSelect={onSelect} className="flex flex-row gap-2">
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
