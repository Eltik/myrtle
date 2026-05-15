import { ChevronDown, Search, TrendingUp, X } from "lucide-react";
import { Button } from "#/components/ui/button";
import { InputGroup, InputGroupAddon, InputGroupInput } from "#/components/ui/input-group";
import { DropdownMenu, DropdownMenuContent, DropdownMenuItem, DropdownMenuTrigger } from "#/components/ui/menu";
import { cn } from "#/lib/utils";
import { INTERVALS, type LeaderboardInterval, type LeaderboardScope, SERVERS, type ServerCode } from "../constants";

interface IToolbarProps {
    scope: LeaderboardScope;
    onScope: (next: LeaderboardScope) => void;
    server: ServerCode | "All";
    onServer: (next: ServerCode | "All") => void;
    interval: LeaderboardInterval;
    onInterval: (next: LeaderboardInterval) => void;
    movementOnly: boolean;
    onMovementOnly: (next: boolean) => void;
    query: string;
    onQuery: (next: string) => void;
}

export function Toolbar({ scope, onScope, server, onServer, interval, onInterval, movementOnly, onMovementOnly, query, onQuery }: IToolbarProps) {
    const intervalShort = INTERVALS.find((i) => i.value === interval)?.short ?? "1d";

    return (
        <div className="flex flex-col gap-2.5 sm:flex-row sm:items-center sm:justify-between">
            <div className="flex flex-wrap items-center gap-2">
                <Segmented
                    value={scope}
                    onChange={onScope}
                    options={[
                        { value: "global", label: "Global" },
                        { value: "friends", label: "Friends", disabled: true, hint: "Coming soon" },
                    ]}
                />
                <DropdownMenu>
                    <DropdownMenuTrigger className="inline-flex h-8 cursor-pointer items-center gap-2 rounded-lg border border-input bg-card px-2.5 font-medium font-sans text-foreground text-xs leading-none transition-colors hover:border-foreground/20" aria-label="Change server filter">
                        <span className="font-medium font-mono text-[10.5px] text-muted-foreground uppercase leading-none tracking-[0.14em]">Server</span>
                        <span className="font-semibold text-foreground">{server === "All" ? "All" : server}</span>
                        <ChevronDown className="size-3 opacity-70" />
                    </DropdownMenuTrigger>
                    <DropdownMenuContent align="start" className="min-w-40">
                        <DropdownMenuItem onClick={() => onServer("All")} className={cn("cursor-pointer", server === "All" && "font-semibold text-primary")}>
                            All servers
                        </DropdownMenuItem>
                        {SERVERS.map((code) => (
                            <DropdownMenuItem key={code} onClick={() => onServer(code)} className={cn("cursor-pointer", server === code && "font-semibold text-primary")}>
                                {code}
                            </DropdownMenuItem>
                        ))}
                    </DropdownMenuContent>
                </DropdownMenu>
                <DropdownMenu>
                    <DropdownMenuTrigger className="inline-flex h-8 cursor-pointer items-center gap-2 rounded-lg border border-input bg-card px-2.5 font-medium font-sans text-foreground text-xs leading-none transition-colors hover:border-foreground/20" aria-label="Change movement interval">
                        <span className="font-medium font-mono text-[10.5px] text-muted-foreground uppercase leading-none tracking-[0.14em]">Interval</span>
                        <span className="font-semibold text-foreground tabular-nums">{intervalShort}</span>
                        <ChevronDown className="size-3 opacity-70" />
                    </DropdownMenuTrigger>
                    <DropdownMenuContent align="start" className="min-w-44">
                        {INTERVALS.map((opt) => (
                            <DropdownMenuItem key={opt.value} onClick={() => onInterval(opt.value)} className={cn("cursor-pointer", interval === opt.value && "font-semibold text-primary")}>
                                {opt.label}
                            </DropdownMenuItem>
                        ))}
                    </DropdownMenuContent>
                </DropdownMenu>
                <button
                    type="button"
                    role="switch"
                    aria-checked={movementOnly}
                    onClick={() => onMovementOnly(!movementOnly)}
                    title={movementOnly ? "Showing only Doctors with movement" : "Show only Doctors with movement"}
                    className={cn(
                        "inline-flex h-8 cursor-pointer items-center gap-1.5 rounded-lg border px-2.5 font-medium font-sans text-xs leading-none transition-colors",
                        movementOnly ? "border-primary/40 bg-[color-mix(in_srgb,var(--primary)_10%,transparent)] text-primary hover:bg-[color-mix(in_srgb,var(--primary)_14%,transparent)]" : "border-input bg-card text-foreground hover:border-foreground/20",
                    )}
                >
                    <TrendingUp className={cn("size-3.5", movementOnly ? "opacity-100" : "opacity-70")} aria-hidden />
                    <span>Movement only</span>
                </button>
            </div>

            <InputGroup className="w-full max-w-80 sm:w-80">
                <InputGroupAddon>
                    <Search aria-hidden="true" />
                </InputGroupAddon>
                <InputGroupInput value={query} onChange={(e) => onQuery(e.target.value)} placeholder="Search Doctor name or UID…" aria-label="Search this page" />
                <InputGroupAddon align="inline-end">
                    {query ? (
                        <Button variant="ghost" size="icon-xs" onClick={() => onQuery("")} aria-label="Clear search">
                            <X aria-hidden="true" />
                        </Button>
                    ) : null}
                </InputGroupAddon>
            </InputGroup>
        </div>
    );
}

interface ISegmentedOption<T extends string> {
    value: T;
    label: string;
    disabled?: boolean;
    hint?: string;
}

function Segmented<T extends string>({ value, onChange, options }: { value: T; onChange: (next: T) => void; options: ISegmentedOption<T>[] }) {
    return (
        <div role="tablist" className="inline-flex items-center rounded-lg border border-border bg-muted p-0.75">
            {options.map((opt) => {
                const isActive = opt.value === value;
                return (
                    <button
                        key={opt.value}
                        type="button"
                        role="tab"
                        aria-selected={isActive}
                        disabled={opt.disabled}
                        title={opt.hint}
                        onClick={() => !opt.disabled && onChange(opt.value)}
                        className={cn("inline-flex h-6.5 items-center rounded-md px-3 font-medium font-sans text-xs leading-none transition-colors", isActive ? "bg-card text-foreground shadow-[0_1px_2px_rgb(0_0_0/0.06)]" : "text-muted-foreground hover:text-foreground", opt.disabled && "cursor-not-allowed opacity-60")}
                    >
                        {opt.label}
                    </button>
                );
            })}
        </div>
    );
}
