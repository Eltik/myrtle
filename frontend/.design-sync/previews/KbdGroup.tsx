import { Kbd, KbdGroup } from "frontend";

export const KeyCombos = () => (
    <div className="w-full max-w-sm rounded-lg border border-border bg-card px-4 py-3">
        <p className="m-0 mb-2 font-medium font-sans text-foreground text-sm">Roster shortcuts</p>
        <ul className="m-0 flex list-none flex-col gap-2 p-0 font-sans text-muted-foreground text-sm">
            <li className="flex items-center justify-between gap-4">
                Command palette
                <KbdGroup>
                    <Kbd>⌘</Kbd>
                    <Kbd>K</Kbd>
                </KbdGroup>
            </li>
            <li className="flex items-center justify-between gap-4">
                Compare selected operators
                <KbdGroup>
                    <Kbd>⌘</Kbd>
                    <Kbd>⇧</Kbd>
                    <Kbd>C</Kbd>
                </KbdGroup>
            </li>
            <li className="flex items-center justify-between gap-4">
                Clear all filters
                <KbdGroup>
                    <Kbd>⌥</Kbd>
                    <Kbd>⌫</Kbd>
                </KbdGroup>
            </li>
        </ul>
    </div>
);

export const PlatformVariants = () => (
    <div className="flex flex-col gap-3">
        <div className="flex items-center gap-2 font-sans text-muted-foreground text-sm">
            <span className="w-20 text-foreground">macOS</span>
            <KbdGroup>
                <Kbd>⌘</Kbd>
                <Kbd>K</Kbd>
            </KbdGroup>
        </div>
        <div className="flex items-center gap-2 font-sans text-muted-foreground text-sm">
            <span className="w-20 text-foreground">Windows</span>
            <KbdGroup>
                <Kbd>Ctrl</Kbd>
                <Kbd>K</Kbd>
            </KbdGroup>
        </div>
        <div className="flex items-center gap-2 font-sans text-muted-foreground text-sm">
            <span className="w-20 text-foreground">Either</span>
            <KbdGroup>
                <Kbd>↑</Kbd>
                <Kbd>↓</Kbd>
            </KbdGroup>
            <span>to move through results</span>
        </div>
    </div>
);

export const InlineInSentence = () => (
    <p className="m-0 max-w-[52ch] font-sans text-muted-foreground text-sm leading-[1.6]">
        Press{" "}
        <KbdGroup>
            <Kbd>⌘</Kbd>
            <Kbd>K</Kbd>
        </KbdGroup>{" "}
        for all commands, or{" "}
        <KbdGroup>
            <Kbd>G</Kbd>
            <Kbd>S</Kbd>
        </KbdGroup>{" "}
        to jump straight to the stage browser.
    </p>
);
