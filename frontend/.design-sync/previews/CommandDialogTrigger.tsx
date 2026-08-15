import { Button, Command, CommandDialog, CommandDialogPopup, CommandDialogTrigger, CommandFooter, CommandGroup, CommandGroupLabel, CommandInput, CommandItem, CommandList, CommandPanel, Kbd, OperatorAvatar } from "frontend";
import { MapIcon, SearchIcon } from "lucide-react";

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

/** The header's search affordance: a fake input that opens the palette. */
export const SearchField = () => (
    <div className="mx-auto flex w-full max-w-2xl items-center justify-between gap-4 rounded-xl border bg-card p-3">
        <span className="font-semibold text-sm">Myrtle</span>
        <CommandDialog>
            <CommandDialogTrigger className="flex w-80 items-center gap-2 rounded-lg border bg-background px-3 py-2 text-muted-foreground text-sm shadow-xs">
                <SearchIcon className="size-4" />
                <span>Search operators, pages, tools…</span>
                <span className="ml-auto flex items-center gap-1">
                    <Kbd>⌘</Kbd>
                    <Kbd>K</Kbd>
                </span>
            </CommandDialogTrigger>
            <CommandDialogPopup>
                <PaletteBody />
            </CommandDialogPopup>
        </CommandDialog>
    </div>
);

/** Rendered as a Button through the `render` prop — the mobile header variant. */
export const AsButton = () => (
    <div className="mx-auto flex w-full max-w-2xl items-center justify-between gap-4 rounded-xl border bg-card p-3">
        <span className="font-semibold text-sm">Myrtle</span>
        <CommandDialog>
            <CommandDialogTrigger render={<Button variant="outline" />}>
                <SearchIcon />
                Search
            </CommandDialogTrigger>
            <CommandDialogPopup>
                <PaletteBody />
            </CommandDialogPopup>
        </CommandDialog>
    </div>
);

/** Trigger plus the palette it opened. */
export const Opened = () => (
    <div className="mx-auto flex w-full max-w-2xl items-center justify-between gap-4 rounded-xl border bg-card p-3">
        <span className="font-semibold text-sm">Myrtle</span>
        <CommandDialog defaultOpen>
            <CommandDialogTrigger className="flex w-80 items-center gap-2 rounded-lg border bg-background px-3 py-2 text-muted-foreground text-sm shadow-xs">
                <SearchIcon className="size-4" />
                <span>Search operators, pages, tools…</span>
                <span className="ml-auto flex items-center gap-1">
                    <Kbd>⌘</Kbd>
                    <Kbd>K</Kbd>
                </span>
            </CommandDialogTrigger>
            <CommandDialogPopup>
                <PaletteBody />
            </CommandDialogPopup>
        </CommandDialog>
    </div>
);
