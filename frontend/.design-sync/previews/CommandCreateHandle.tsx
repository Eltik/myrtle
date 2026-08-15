import { Button, Command, CommandCreateHandle, CommandDialog, CommandDialogPopup, CommandDialogTrigger, CommandFooter, CommandGroup, CommandGroupLabel, CommandInput, CommandItem, CommandList, CommandPanel, Kbd, OperatorAvatar } from "frontend";
import { MapIcon, SearchIcon } from "lucide-react";

/** One handle, created once at module scope, shared by trigger and dialog. */
const searchPalette = CommandCreateHandle();

const OPERATORS = [
    { id: "char_4064_mlynar", name: "Młynar", meta: "6★ · Guard" },
    { id: "char_1012_skadi2", name: "Skadi the Corrupting Heart", meta: "6★ · Supporter" },
    { id: "char_249_mlyss", name: "Muelsyse", meta: "6★ · Vanguard" },
];

const PaletteBody = () => (
    <Command mode="none">
        <CommandInput placeholder="Search operators, pages, tools…" />
        <CommandPanel>
            <CommandList>
                <CommandGroup>
                    <CommandGroupLabel>Operators</CommandGroupLabel>
                    {OPERATORS.map((op) => (
                        <CommandItem className="flex flex-row gap-2" key={op.id} value={`operator:${op.id}`}>
                            <span aria-hidden="true" className="op-chip">
                                <OperatorAvatar charId={op.id} name={op.name} />
                            </span>
                            <span className="flex-1 font-medium">{op.name}</span>
                            <span className="text-muted-foreground text-xs">{op.meta}</span>
                        </CommandItem>
                    ))}
                </CommandGroup>
                <CommandGroup>
                    <CommandGroupLabel>Pages</CommandGroupLabel>
                    <CommandItem className="flex flex-row gap-2" value="page:stages">
                        <MapIcon className="size-4 text-muted-foreground" />
                        <span className="flex-1">Stages</span>
                        <span className="text-muted-foreground text-xs">Enemy-pathing simulator</span>
                    </CommandItem>
                </CommandGroup>
            </CommandList>
        </CommandPanel>
        <CommandFooter>
            <span className="flex items-center gap-1">
                <Kbd>↵</Kbd> to select
            </span>
            <span className="ml-auto font-mono text-muted-foreground/60">powered by COSS UI</span>
        </CommandFooter>
    </Command>
);

/**
 * The handle decouples the two: the header trigger lives in the site chrome,
 * the dialog is mounted once at the app root.
 */
export const RemoteTrigger = () => (
    <div className="mx-auto w-full max-w-2xl">
        <div className="flex items-center justify-between gap-4 rounded-xl border bg-card p-3">
            <span className="font-semibold text-sm">Myrtle</span>
            <CommandDialogTrigger className="flex w-80 items-center gap-2 rounded-lg border bg-background px-3 py-2 text-muted-foreground text-sm shadow-xs" handle={searchPalette}>
                <SearchIcon className="size-4" />
                <span>Search operators, pages, tools…</span>
                <span className="ml-auto flex items-center gap-1">
                    <Kbd>⌘</Kbd>
                    <Kbd>K</Kbd>
                </span>
            </CommandDialogTrigger>
        </div>
        <div className="mt-4 rounded-xl border bg-card p-3 text-muted-foreground text-sm">
            Chapter 8 — Roaring Flare · 14 stages · recommended E2 40
        </div>
        <CommandDialog handle={searchPalette}>
            <CommandDialogPopup>
                <PaletteBody />
            </CommandDialogPopup>
        </CommandDialog>
    </div>
);

/** Opened through the same handle from a secondary entry point. */
export const OpenedByHandle = () => (
    <div className="mx-auto w-full max-w-2xl">
        <div className="flex items-center justify-between gap-4 rounded-xl border bg-card p-3">
            <span className="font-semibold text-sm">Myrtle</span>
            <CommandDialogTrigger handle={searchPalette} render={<Button variant="outline" />}>
                <SearchIcon />
                Search
            </CommandDialogTrigger>
        </div>
        <CommandDialog defaultOpen handle={searchPalette}>
            <CommandDialogPopup>
                <PaletteBody />
            </CommandDialogPopup>
        </CommandDialog>
    </div>
);
