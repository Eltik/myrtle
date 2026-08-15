import { Command, CommandDialog, CommandDialogPopup, CommandFooter, CommandGroup, CommandGroupLabel, CommandInput, CommandItem, CommandList, CommandPanel, Kbd, OperatorAvatar } from "frontend";
import { MapIcon, TrophyIcon } from "lucide-react";

const OPERATORS = [
    { id: "char_4064_mlynar", name: "Młynar", meta: "6★ · Guard" },
    { id: "char_1012_skadi2", name: "Skadi the Corrupting Heart", meta: "6★ · Supporter" },
    { id: "char_249_mlyss", name: "Muelsyse", meta: "6★ · Vanguard" },
    { id: "char_1028_texas2", name: "Texas the Omertosa", meta: "6★ · Specialist" },
];

const LEADERBOARD = [
    { rank: 1, doctor: "Kal'tsit", score: "18,412" },
    { rank: 2, doctor: "Doctor Amiya", score: "17,980" },
    { rank: 3, doctor: "Rhodes Ops", score: "17,344" },
    { rank: 4, doctor: "Babel Vet", score: "16,905" },
];

const Hints = () => (
    <CommandFooter>
        <span className="flex items-center gap-1">
            <Kbd>esc</Kbd> to dismiss
        </span>
        <span className="ml-auto font-mono text-muted-foreground/60">powered by COSS UI</span>
    </CommandFooter>
);

const Palette = ({ children }: { children?: React.ReactNode }) => (
    <CommandDialog defaultOpen>
        <CommandDialogPopup>
            <Command mode="none">
                <CommandInput placeholder="Search operators, pages, tools…" />
                <CommandPanel>
                    <CommandList>{children}</CommandList>
                </CommandPanel>
                <Hints />
            </Command>
        </CommandDialogPopup>
    </CommandDialog>
);

/** The backdrop dims and blurs the operator grid underneath. */
export const OverOperatorGrid = () => (
    <div>
        <div className="mx-auto w-full max-w-2xl">
            <div className="flex items-center justify-between border-b pb-3">
                <span className="font-semibold text-lg">Operators</span>
                <span className="text-muted-foreground text-sm">312 operators · EN server</span>
            </div>
            <div className="mt-4 grid grid-cols-4 gap-3">
                {OPERATORS.map((op) => (
                    <div className="rounded-lg border bg-card p-3" key={op.id}>
                        <span aria-hidden="true" className="op-chip">
                            <OperatorAvatar charId={op.id} name={op.name} />
                        </span>
                        <div className="mt-2 truncate font-medium text-sm">{op.name}</div>
                        <div className="text-muted-foreground text-xs">{op.meta}</div>
                    </div>
                ))}
            </div>
        </div>
        <Palette>
            <CommandGroup>
                <CommandGroupLabel>Operators</CommandGroupLabel>
                {OPERATORS.slice(0, 3).map((op) => (
                    <CommandItem className="flex flex-row gap-2" key={op.id} value={`operator:${op.id}`}>
                        <span aria-hidden="true" className="op-chip">
                            <OperatorAvatar charId={op.id} name={op.name} />
                        </span>
                        <span className="flex-1 font-medium">{op.name}</span>
                        <span className="text-muted-foreground text-xs">{op.meta}</span>
                    </CommandItem>
                ))}
            </CommandGroup>
        </Palette>
    </div>
);

/** Same backdrop over a denser page — the leaderboard table. */
export const OverLeaderboard = () => (
    <div>
        <div className="mx-auto w-full max-w-2xl">
            <div className="flex items-center justify-between border-b pb-3">
                <span className="font-semibold text-lg">Leaderboard</span>
                <span className="text-muted-foreground text-sm">Top Doctors by score</span>
            </div>
            <div className="mt-4 flex flex-col gap-2">
                {LEADERBOARD.map((row) => (
                    <div className="flex items-center justify-between rounded-lg border bg-card px-3 py-2" key={row.rank}>
                        <span className="font-mono text-muted-foreground text-xs">#{row.rank}</span>
                        <span className="flex-1 px-3 font-medium text-sm">{row.doctor}</span>
                        <span className="font-mono text-sm tabular-nums">{row.score}</span>
                    </div>
                ))}
            </div>
        </div>
        <Palette>
            <CommandGroup>
                <CommandGroupLabel>Pages</CommandGroupLabel>
                <CommandItem className="flex flex-row gap-2" value="page:leaderboard">
                    <TrophyIcon className="size-4 text-muted-foreground" />
                    <span className="flex-1">Leaderboard</span>
                    <span className="text-muted-foreground text-xs">Top Doctors by score</span>
                </CommandItem>
                <CommandItem className="flex flex-row gap-2" value="page:stages">
                    <MapIcon className="size-4 text-muted-foreground" />
                    <span className="flex-1">Stages</span>
                    <span className="text-muted-foreground text-xs">Enemy-pathing simulator</span>
                </CommandItem>
            </CommandGroup>
        </Palette>
    </div>
);
