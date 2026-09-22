import { ChevronDown, Search, TrendingUp, X } from "lucide-react";
import { Button } from "#/components/ui/button";
import { InputGroup, InputGroupAddon, InputGroupInput } from "#/components/ui/input-group";
import { DropdownMenu, DropdownMenuContent, DropdownMenuItem, DropdownMenuTrigger } from "#/components/ui/menu";
import type { IMaterials } from "#/lib/api/materials";
import { useT } from "#/lib/i18n";
import type { TypedT } from "#/lib/i18n/messages";
import { cn } from "#/lib/utils";
import { INTERVALS, type LeaderboardInterval, type LeaderboardScope, type Ranking, SERVERS, type ServerCode } from "../constants";
import type { messages as constantsMessages } from "../constants.messages";
import type { ICatalog } from "../inventory.types";
import { RankByPicker } from "./RankByPicker";
import type { messages } from "./Toolbar.messages";

/** The interval labels in `constants.messages.ts` are rendered here too. */
type ToolbarT = TypedT<typeof messages & typeof constantsMessages>;

interface IToolbarProps {
    ranking: Ranking;
    onRanking: (next: Ranking) => void;
    catalog: ICatalog;
    materials: IMaterials | undefined;
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

/** Filter trigger: a 36px tap target on touch widths, the site's 32px on desktop. */
const FILTER = "inline-flex h-9 cursor-pointer items-center gap-2 rounded-lg border border-input bg-card px-2.5 font-medium font-sans text-foreground text-xs leading-none transition-colors hover:border-foreground/20 disabled:cursor-not-allowed disabled:opacity-50 disabled:hover:border-input sm:h-8";

export function Toolbar({ ranking, onRanking, catalog, materials, scope, onScope, server, onServer, interval, onInterval, movementOnly, onMovementOnly, query, onQuery }: IToolbarProps) {
    const t: ToolbarT = useT("user");
    const intervalShortKey = INTERVALS.find((i) => i.value === interval)?.shortKey ?? "leaderboard.interval.day.short";
    // Rank movement comes from score snapshots; item holdings are not
    // snapshotted, so the two movement controls have nothing to show and
    // say so rather than vanish.
    const movementAvailable = ranking.kind === "score";
    const movementHint = movementAvailable ? undefined : t("leaderboard.toolbar.movement.scoreOnly");

    return (
        <div className="flex flex-col gap-2.5">
            <div className="flex flex-col gap-2 sm:flex-row sm:items-center">
                <RankByPicker ranking={ranking} onRanking={onRanking} catalog={catalog} materials={materials} variant="toolbar" className="w-full sm:w-auto sm:max-w-80" />
                <InputGroup className="w-full sm:ml-auto sm:w-72">
                    <InputGroupAddon>
                        <Search aria-hidden="true" />
                    </InputGroupAddon>
                    <InputGroupInput value={query} onChange={(e) => onQuery(e.target.value)} placeholder={t("leaderboard.toolbar.search.placeholder")} aria-label={t("leaderboard.toolbar.search.label")} />
                    <InputGroupAddon align="inline-end">
                        {query ? (
                            <Button variant="ghost" size="icon-xs" onClick={() => onQuery("")} aria-label={t("leaderboard.toolbar.search.clear")}>
                                <X aria-hidden="true" />
                            </Button>
                        ) : null}
                    </InputGroupAddon>
                </InputGroup>
            </div>

            <div className="flex flex-wrap items-center gap-2">
                <Segmented
                    value={scope}
                    onChange={onScope}
                    options={[
                        { value: "global", label: t("leaderboard.toolbar.scope.global") },
                        { value: "friends", label: t("leaderboard.toolbar.scope.friends"), disabled: true, hint: t("leaderboard.toolbar.comingSoon") },
                    ]}
                />
                <DropdownMenu>
                    <DropdownMenuTrigger className={FILTER} aria-label={t("leaderboard.toolbar.serverLabel")}>
                        <span className="font-medium font-mono text-[10.5px] text-muted-foreground uppercase leading-none tracking-[0.14em]">{t("leaderboard.toolbar.server")}</span>
                        <span className="font-semibold text-foreground">{server === "All" ? t("leaderboard.toolbar.all") : server}</span>
                        <ChevronDown className="size-3 opacity-70" />
                    </DropdownMenuTrigger>
                    <DropdownMenuContent align="start" className="min-w-40">
                        <DropdownMenuItem onClick={() => onServer("All")} className={cn("cursor-pointer", server === "All" && "font-semibold text-primary")}>
                            {t("leaderboard.toolbar.allServers")}
                        </DropdownMenuItem>
                        {SERVERS.map((code) => (
                            <DropdownMenuItem key={code} onClick={() => onServer(code)} className={cn("cursor-pointer", server === code && "font-semibold text-primary")}>
                                {code}
                            </DropdownMenuItem>
                        ))}
                    </DropdownMenuContent>
                </DropdownMenu>
                <DropdownMenu>
                    <DropdownMenuTrigger className={FILTER} aria-label={t("leaderboard.toolbar.intervalLabel")} disabled={!movementAvailable} title={movementHint}>
                        <span className="font-medium font-mono text-[10.5px] text-muted-foreground uppercase leading-none tracking-[0.14em]">{t("leaderboard.toolbar.interval")}</span>
                        <span className="font-semibold text-foreground tabular-nums">{t(intervalShortKey)}</span>
                        <ChevronDown className="size-3 opacity-70" />
                    </DropdownMenuTrigger>
                    <DropdownMenuContent align="start" className="min-w-44">
                        {INTERVALS.map((opt) => (
                            <DropdownMenuItem key={opt.value} onClick={() => onInterval(opt.value)} className={cn("cursor-pointer", interval === opt.value && "font-semibold text-primary")}>
                                {t(opt.labelKey)}
                            </DropdownMenuItem>
                        ))}
                    </DropdownMenuContent>
                </DropdownMenu>
                <button
                    type="button"
                    role="switch"
                    aria-checked={movementOnly}
                    disabled={!movementAvailable}
                    onClick={() => onMovementOnly(!movementOnly)}
                    title={movementHint ?? (movementOnly ? t("leaderboard.toolbar.movementOnly.on") : t("leaderboard.toolbar.movementOnly.off"))}
                    className={cn(
                        "inline-flex h-9 cursor-pointer items-center gap-1.5 rounded-lg border px-2.5 font-medium font-sans text-xs leading-none transition-colors disabled:cursor-not-allowed disabled:opacity-50 sm:h-8",
                        movementOnly && movementAvailable ? "border-primary/40 bg-[color-mix(in_srgb,var(--primary)_10%,transparent)] text-primary hover:bg-[color-mix(in_srgb,var(--primary)_14%,transparent)]" : "border-input bg-card text-foreground hover:border-foreground/20 disabled:hover:border-input",
                    )}
                >
                    <TrendingUp className={cn("size-3.5", movementOnly && movementAvailable ? "opacity-100" : "opacity-70")} aria-hidden />
                    <span>{t("leaderboard.toolbar.movementOnly")}</span>
                </button>
            </div>
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
        <div role="tablist" className="inline-flex h-9 items-center rounded-lg border border-border bg-muted p-0.75 sm:h-8">
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
                        className={cn("inline-flex h-full items-center rounded-md px-3 font-medium font-sans text-xs leading-none transition-colors", isActive ? "bg-card text-foreground shadow-[0_1px_2px_rgb(0_0_0/0.06)]" : "text-muted-foreground hover:text-foreground", opt.disabled && "cursor-not-allowed opacity-60")}
                    >
                        {opt.label}
                    </button>
                );
            })}
        </div>
    );
}
