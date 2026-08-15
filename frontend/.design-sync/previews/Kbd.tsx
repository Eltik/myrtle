import { Kbd } from "frontend";

export const SearchHint = () => (
    <p className="m-0 flex max-w-[48ch] flex-wrap items-center gap-1 font-sans text-[17px] text-muted-foreground leading-[1.55]">
        400+ operators, complete stats, community tier lists, and live roster sync.
        <span className="flex flex-row items-center gap-2">
            Hit{" "}
            <span>
                <Kbd>⌘</Kbd> <Kbd>K</Kbd>
            </span>{" "}
            to jump anywhere.
        </span>
    </p>
);

export const CommandPaletteFooter = () => (
    <div className="w-full max-w-md rounded-lg border border-border bg-card">
        <div className="flex items-center gap-2 border-border border-b px-3 py-2.5">
            <span className="font-sans text-muted-foreground text-sm">Search operators, stages, tools…</span>
            <span className="ml-auto">
                <Kbd>esc</Kbd>
            </span>
        </div>
        <div className="flex flex-col gap-1 px-3 py-2 font-sans text-foreground text-sm">
            <span>Mlynar — 6★ Guard</span>
            <span>Skadi the Corrupting Heart — 6★ Guard</span>
            <span>S4-1 — Chernobog</span>
        </div>
        <div className="flex items-center gap-3 border-border border-t px-3 py-2 font-sans text-muted-foreground text-xs">
            <span className="flex items-center gap-1">
                <Kbd>↑</Kbd>
                <Kbd>↓</Kbd> to navigate
            </span>
            <span className="flex items-center gap-1">
                <Kbd>↵</Kbd> to select
            </span>
        </div>
    </div>
);

export const ShortcutRows = () => (
    <div className="w-full max-w-sm rounded-lg border border-border bg-card px-4 py-3">
        <p className="m-0 mb-2 font-medium font-sans text-foreground text-sm">Keyboard shortcuts</p>
        <ul className="m-0 flex list-none flex-col gap-2 p-0 font-sans text-muted-foreground text-sm">
            <li className="flex items-center justify-between gap-4">
                Open command palette
                <span className="flex items-center gap-1">
                    <Kbd>⌘</Kbd>
                    <Kbd>K</Kbd>
                </span>
            </li>
            <li className="flex items-center justify-between gap-4">
                Focus operator filters
                <span className="flex items-center gap-1">
                    <Kbd>F</Kbd>
                </span>
            </li>
            <li className="flex items-center justify-between gap-4">
                Next roster page
                <span className="flex items-center gap-1">
                    <Kbd>→</Kbd>
                </span>
            </li>
            <li className="flex items-center justify-between gap-4">
                Toggle dark mode
                <span className="flex items-center gap-1">
                    <Kbd>⌘</Kbd>
                    <Kbd>⇧</Kbd>
                    <Kbd>L</Kbd>
                </span>
            </li>
        </ul>
    </div>
);
