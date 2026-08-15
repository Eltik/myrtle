import { Command, CommandEmpty, CommandFooter, CommandInput, CommandList, CommandPanel, Kbd } from "frontend";
import { SearchXIcon } from "lucide-react";

const Shell = ({ children }: { children?: React.ReactNode }) => <div className="mx-auto flex w-full max-w-xl flex-col rounded-2xl border bg-popover text-popover-foreground shadow-lg">{children}</div>;

const Hints = () => (
    <CommandFooter>
        <span className="flex items-center gap-1">
            <Kbd>esc</Kbd> to close
        </span>
        <span className="ml-auto font-mono text-muted-foreground/60">powered by COSS UI</span>
    </CommandFooter>
);

/** Nothing matched the query — the palette's only body content. */
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

/** A richer empty body — icon, headline and a hint about what is searchable. */
export const WithHint = () => (
    <Shell>
        <Command defaultValue="annihilation 12" items={[]} mode="none">
            <CommandInput placeholder="Search operators, pages, tools…" />
            <CommandPanel>
                <CommandList>
                    <CommandEmpty>
                        <span className="flex flex-col items-center gap-2">
                            <SearchXIcon className="size-5 text-muted-foreground" />
                            <span className="font-medium text-foreground text-sm">No results for “annihilation 12”</span>
                            <span className="text-muted-foreground text-xs">Try an operator name, a stage code like S4-1, or a tool.</span>
                        </span>
                    </CommandEmpty>
                </CommandList>
            </CommandPanel>
            <Hints />
        </Command>
    </Shell>
);

/** Scoped picker: the roster is exhausted rather than the query unmatched. */
export const AllPlanned = () => (
    <Shell>
        <Command items={[]} mode="none">
            <CommandInput placeholder="Add an operator to the planner…" />
            <CommandPanel>
                <CommandList>
                    <CommandEmpty>Every owned operator is already in the plan.</CommandEmpty>
                </CommandList>
            </CommandPanel>
            <Hints />
        </Command>
    </Shell>
);
