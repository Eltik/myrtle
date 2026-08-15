import { NotFound } from "frontend";

export const Default = () => <NotFound />;

export const InAppShell = () => (
    <div className="flex flex-col">
        <header className="flex h-14 items-center justify-between border-border border-b px-4">
            <div className="inline-flex items-center gap-2.5 font-sans font-semibold text-foreground text-sm leading-none">
                <span className="inline-flex size-6 items-center justify-center rounded-md bg-primary font-bold font-mono text-[11px] text-primary-foreground">M</span>
                myrtle.moe
            </div>
            <nav className="inline-flex items-center gap-4 font-sans text-[12.5px] text-muted-foreground">
                <span>Operators</span>
                <span>Stages</span>
                <span>Gacha</span>
                <span className="font-mono text-[11px] text-muted-foreground/70">/operators/char_4064_mlynarr</span>
            </nav>
        </header>
        <NotFound />
    </div>
);
