"use client";

import { useQuery } from "@tanstack/react-query";
import { useNavigate } from "@tanstack/react-router";
import * as React from "react";
import { Command, CommandDialog, CommandDialogPopup, CommandEmpty, CommandFooter, CommandGroup, CommandGroupLabel, CommandInput, CommandItem, CommandList, CommandPanel, CommandSeparator } from "#/components/ui/command";
import { Kbd } from "#/components/ui/kbd";
import { OperatorAvatar } from "#/components/ui/operator-avatar";
import { Skeleton } from "#/components/ui/skeleton";
import { useOperatorName } from "#/hooks/use-operator-name";
import { operatorsIndexQueryOptions } from "#/lib/api/operators";
import { upcomingQueryOptions } from "#/lib/api/upcoming";
import { hasMod, isEditableTarget } from "#/lib/hotkeys";
import { type TFunction, useGamedataServer, useT } from "#/lib/i18n";
import type { TypedT } from "#/lib/i18n/messages";
import { professionClass, professionLabel } from "#/lib/registry/operator-display";
import { type IPage, PAGES } from "#/lib/registry/pages";
import { ToolIcon } from "#/lib/registry/ToolIcon";
import { type ITool, TOOLS } from "#/lib/registry/tools";
import { searchAndRank } from "#/lib/search/fuzzy";
import type { IOperatorIndexEntry } from "#/types/operators";
import type { messages } from "./search-command.messages";

interface ISearchCommandProps {
    open: boolean;
    onOpenChange: (open: boolean) => void;
}

const MAX_PAGES = 6;
const MAX_TOOLS = 6;
const MAX_OPERATORS = 8;

interface ISearchOperator {
    op: IOperatorIndexEntry;
    /** CN-only: the avatar comes from the CN tree and the row says so. */
    upcoming: boolean;
}

export function SearchCommand({ open, onOpenChange }: ISearchCommandProps): React.ReactElement {
    const [query, setQuery] = React.useState("");
    const navigate = useNavigate();
    const t: TypedT<typeof messages> = useT("common");
    // The page and tool registries are plain modules, so they carry message
    // keys in the `nav` namespace rather than text; this resolves them.
    const tNav: TFunction = useT("nav");

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

    const gamedataServer = useGamedataServer();
    const operatorsQuery = useQuery({
        ...operatorsIndexQueryOptions(gamedataServer),
        enabled: open,
    });
    // CN-only operators live in a separate list. They are appended so that a
    // released operator always wins a tie, and searched by appellation too,
    // since nobody types the Han name to find one.
    const upcomingQuery = useQuery({
        ...upcomingQueryOptions(gamedataServer),
        enabled: open,
    });
    const operators = React.useMemo<ISearchOperator[] | undefined>(() => {
        if (!operatorsQuery.data) return undefined;
        const released: ISearchOperator[] = operatorsQuery.data.map((op) => ({ op, upcoming: false }));
        const upcoming: ISearchOperator[] = (upcomingQuery.data ?? []).map((op) => ({ op, upcoming: true }));
        return released.concat(upcoming);
    }, [operatorsQuery.data, upcomingQuery.data]);

    const pageResults = React.useMemo(
        () =>
            searchAndRank(
                query,
                PAGES,
                (p) => ({
                    name: tNav(p.labelKey),
                    extra: `${tNav(p.descKey)} ${p.keywords.join(" ")} ${p.id}`,
                }),
                MAX_PAGES,
            ),
        [query, tNav],
    );

    const toolResults = React.useMemo(
        () =>
            searchAndRank(
                query,
                TOOLS,
                (tool) => ({
                    name: tNav(tool.labelKey),
                    extra: `${tNav(tool.descKey)} ${tool.keywords.join(" ")} ${tool.id}`,
                }),
                MAX_TOOLS,
            ),
        [query, tNav],
    );

    const operatorResults = React.useMemo(() => {
        if (!operators) return [];
        return searchAndRank(
            query,
            operators,
            ({ op }) => ({
                name: op.name,
                aliases: [op.appellation],
                extra: `${professionLabel(op.profession)} ${op.subProfessionId} ${op.rarity}★ ${op.tagList.join(" ")} ${op.nationId}`,
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
                    <CommandInput placeholder={t("searchCommand.placeholder")} />
                    <CommandPanel>
                        <CommandList>
                            {!hasResults && !anyLoading && <CommandEmpty>{t("searchCommand.noResults")}</CommandEmpty>}

                            <CommandGroup>
                                <CommandGroupLabel>{t("searchCommand.operators")}</CommandGroupLabel>
                                {operatorsQuery.isLoading ? (
                                    <OperatorSkeletons />
                                ) : operatorsQuery.isError ? (
                                    <div className="px-2 py-3 text-muted-foreground text-xs">{t("searchCommand.operatorsFailed")}</div>
                                ) : operatorResults.length === 0 ? (
                                    query.trim().length > 0 ? (
                                        <div className="px-2 py-3 text-muted-foreground text-xs">{t("searchCommand.noOperatorMatch", { query })}</div>
                                    ) : null
                                ) : (
                                    operatorResults.map(({ item: { op, upcoming } }) => <OperatorRow key={op.id} op={op} upcoming={upcoming} onClick={() => closeAndGo(`/operators/${op.id}`)} />)
                                )}
                            </CommandGroup>

                            {(operatorResults.length > 0 || operatorsQuery.isLoading) && pageResults.length > 0 && <CommandSeparator />}

                            {pageResults.length > 0 && (
                                <CommandGroup>
                                    <CommandGroupLabel>{t("searchCommand.pages")}</CommandGroupLabel>
                                    {pageResults.map(({ item: page }) => (
                                        <PageRow key={page.id} page={page} onClick={() => closeAndGo(page.href)} />
                                    ))}
                                </CommandGroup>
                            )}

                            {pageResults.length > 0 && toolResults.length > 0 && <CommandSeparator />}

                            {toolResults.length > 0 && (
                                <CommandGroup>
                                    <CommandGroupLabel>{t("searchCommand.tools")}</CommandGroupLabel>
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
                            {t("searchCommand.toNavigate")}
                        </span>
                        <span className="flex pointer-coarse:hidden items-center gap-1">
                            <Kbd>↵</Kbd> {t("searchCommand.toSelect")}
                        </span>
                        <span className="ml-auto font-mono text-[10.5px] text-muted-foreground/60 tracking-[0.04em]">{t("searchCommand.poweredBy")}</span>
                    </CommandFooter>
                </Command>
            </CommandDialogPopup>
        </CommandDialog>
    );
}

function PageRow({ page, onClick }: { page: IPage; onClick: () => void }): React.ReactElement {
    const tNav: TFunction = useT("nav");

    return (
        <CommandItem value={`page:${page.id}`} onClick={onClick} className="flex cursor-pointer flex-row gap-2">
            <ToolIcon name={page.icon} className="size-4 text-muted-foreground" />
            <span className="flex-1">{tNav(page.labelKey)}</span>
            <span className="hidden text-muted-foreground text-xs sm:inline">{tNav(page.descKey)}</span>
        </CommandItem>
    );
}

function ToolRow({ tool, onClick }: { tool: ITool; onClick: () => void }): React.ReactElement {
    const tNav: TFunction = useT("nav");

    return (
        <CommandItem value={`tool:${tool.id}`} onClick={onClick} className="flex cursor-pointer flex-row gap-2">
            <ToolIcon name={tool.icon} className="size-4 text-muted-foreground" />
            <span className="flex-1">{tNav(tool.labelKey)}</span>
            <span className="hidden text-muted-foreground text-xs sm:inline">{tNav(tool.descKey)}</span>
        </CommandItem>
    );
}

function OperatorRow({ op, upcoming, onClick }: { op: IOperatorIndexEntry; upcoming: boolean; onClick: () => void }): React.ReactElement {
    const t: TypedT<typeof messages> = useT("common");
    const operatorName = useOperatorName();
    const cls = professionClass(op.profession);
    const name = operatorName(op);
    return (
        <CommandItem value={`operator:${op.id}`} onClick={onClick} className="flex cursor-pointer flex-row gap-2">
            <span className={`op-chip ${cls}`} aria-hidden="true">
                <OperatorAvatar charId={op.id} name={name} server={upcoming ? "cn" : undefined} />
            </span>
            <span className="flex-1 font-medium">{name}</span>
            <span className="text-muted-foreground text-xs">
                {op.rarity}★ · {professionLabel(op.profession)}
                {upcoming ? ` · ${t("searchCommand.cnOnly")}` : null}
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
