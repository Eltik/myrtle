import { ChevronDown, Search, X } from "lucide-react";
import { Button } from "#/components/ui/button";
import { InputGroup, InputGroupAddon, InputGroupInput } from "#/components/ui/input-group";
import { DropdownMenu, DropdownMenuContent, DropdownMenuItem, DropdownMenuTrigger } from "#/components/ui/menu";
import { cn } from "#/lib/utils";
import { type LeaderboardScope, SERVERS, type ServerCode } from "../constants";

interface IToolbarProps {
    scope: LeaderboardScope;
    onScope: (next: LeaderboardScope) => void;
    server: ServerCode | "All";
    onServer: (next: ServerCode | "All") => void;
    query: string;
    onQuery: (next: string) => void;
}

export function Toolbar({ scope, onScope, server, onServer, query, onQuery }: IToolbarProps) {
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
                    <DropdownMenuTrigger className="inline-flex h-8 cursor-pointer items-center gap-2 rounded-lg border border-input bg-card px-2.5 font-sans text-xs font-medium leading-none text-foreground transition-colors hover:border-foreground/20" aria-label="Change server filter">
                        <span className="font-mono text-[10.5px] font-medium uppercase leading-none tracking-[0.14em] text-muted-foreground">Server</span>
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
                        className={cn(
                            "inline-flex h-6.5 items-center rounded-[6px] px-3 font-sans text-xs font-medium leading-none transition-colors",
                            isActive ? "bg-card text-foreground shadow-[0_1px_2px_rgb(0_0_0/0.06)]" : "text-muted-foreground hover:text-foreground",
                            opt.disabled && "cursor-not-allowed opacity-60",
                        )}
                    >
                        {opt.label}
                    </button>
                );
            })}
        </div>
    );
}
