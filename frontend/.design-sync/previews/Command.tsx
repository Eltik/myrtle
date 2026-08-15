import { Command, CommandEmpty, CommandFooter, CommandGroup, CommandGroupLabel, CommandInput, CommandItem, CommandList, CommandPanel, CommandSeparator, Kbd, OperatorAvatar } from "frontend";
import { CalculatorIcon, ListTodoIcon, MapIcon, PackageIcon, TrophyIcon } from "lucide-react";

interface IOperator {
    id: string;
    name: string;
    meta: string;
}

const OPERATORS: IOperator[] = [
    { id: "char_4064_mlynar", name: "Młynar", meta: "6★ · Guard" },
    { id: "char_1012_skadi2", name: "Skadi the Corrupting Heart", meta: "6★ · Supporter" },
    { id: "char_249_mlyss", name: "Muelsyse", meta: "6★ · Vanguard" },
    { id: "char_1028_texas2", name: "Texas the Omertosa", meta: "6★ · Specialist" },
];

const Shell = ({ children }: { children?: React.ReactNode }) => <div className="mx-auto flex w-full max-w-xl flex-col rounded-2xl border bg-popover text-popover-foreground shadow-lg">{children}</div>;

const OperatorRow = ({ op }: { op: IOperator }) => (
    <CommandItem className="flex flex-row gap-2" value={`operator:${op.id}`}>
        <span aria-hidden="true" className="op-chip">
            <OperatorAvatar charId={op.id} name={op.name} />
        </span>
        <span className="flex-1 font-medium">{op.name}</span>
        <span className="text-muted-foreground text-xs">{op.meta}</span>
    </CommandItem>
);

const LinkRow = ({ icon: Icon, label, desc, value }: { icon: typeof MapIcon; label: string; desc: string; value: string }) => (
    <CommandItem className="flex flex-row gap-2" value={value}>
        <Icon className="size-4 text-muted-foreground" />
        <span className="flex-1">{label}</span>
        <span className="text-muted-foreground text-xs">{desc}</span>
    </CommandItem>
);

const Hints = () => (
    <CommandFooter>
        <span className="flex items-center gap-1">
            <Kbd>↑</Kbd>
            <Kbd>↓</Kbd> to navigate
        </span>
        <span className="flex items-center gap-1">
            <Kbd>↵</Kbd> to select
        </span>
        <span className="ml-auto font-mono text-muted-foreground/60">powered by COSS UI</span>
    </CommandFooter>
);

export const Palette = () => (
    <Shell>
        <Command mode="none">
            <CommandInput placeholder="Search operators, pages, tools…" />
            <CommandPanel>
                <CommandList>
                    <CommandGroup>
                        <CommandGroupLabel>Operators</CommandGroupLabel>
                        {OPERATORS.map((op) => (
                            <OperatorRow key={op.id} op={op} />
                        ))}
                    </CommandGroup>
                    <CommandSeparator />
                    <CommandGroup>
                        <CommandGroupLabel>Pages</CommandGroupLabel>
                        <LinkRow desc="Stats, skills, modules" icon={PackageIcon} label="Operators" value="page:operators" />
                        <LinkRow desc="Enemy-pathing simulator" icon={MapIcon} label="Stages" value="page:stages" />
                        <LinkRow desc="Top Doctors by score" icon={TrophyIcon} label="Leaderboard" value="page:leaderboard" />
                    </CommandGroup>
                </CommandList>
            </CommandPanel>
            <Hints />
        </Command>
    </Shell>
);

export const Filtered = () => (
    <Shell>
        <Command defaultValue="plan" mode="none">
            <CommandInput placeholder="Search operators, pages, tools…" />
            <CommandPanel>
                <CommandList>
                    <CommandGroup>
                        <CommandGroupLabel>Tools</CommandGroupLabel>
                        <LinkRow desc="Promotions, skills, modules" icon={ListTodoIcon} label="Operator planner" value="tool:planner" />
                        <LinkRow desc="Guaranteed tag combos" icon={CalculatorIcon} label="Recruitment calculator" value="tool:recruitment" />
                    </CommandGroup>
                </CommandList>
            </CommandPanel>
            <Hints />
        </Command>
    </Shell>
);

export const NoResults = () => (
    <Shell>
        <Command defaultValue="wisadel e3" items={[]} mode="none">
            <CommandInput placeholder="Search operators, pages, tools…" />
            <CommandPanel>
                <CommandList>
                    <CommandEmpty>No results found.</CommandEmpty>
                </CommandList>
            </CommandPanel>
            <Hints />
        </Command>
    </Shell>
);
