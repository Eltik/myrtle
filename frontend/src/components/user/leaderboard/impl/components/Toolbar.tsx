import { Check, ChevronDown, Search, TrendingUp, X } from "lucide-react";
import type React from "react";
import { Button } from "#/components/ui/button";
import { InputGroup, InputGroupAddon, InputGroupInput } from "#/components/ui/input-group";
import { Kicker } from "#/components/ui/kicker";
import { DropdownMenu, DropdownMenuContent, DropdownMenuItem, DropdownMenuTrigger } from "#/components/ui/menu";
import { ToggleGroup, ToggleGroupItem } from "#/components/ui/toggle-group";
import { Tooltip, TooltipPopup, TooltipTrigger } from "#/components/ui/tooltip";
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

/** Filter trigger: 44px under a coarse pointer, 36px on narrow screens, the site's 32px on desktop. */
const FILTER = "inline-flex h-9 cursor-pointer items-center gap-2 rounded-lg border border-input bg-card px-2.5 font-medium font-sans text-foreground text-xs leading-none transition-colors hover:border-foreground/20 disabled:cursor-not-allowed disabled:opacity-50 disabled:hover:border-input sm:h-8 pointer-coarse:h-11";

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
                    label={t("leaderboard.toolbar.scope.label")}
                    value={scope}
                    onChange={onScope}
                    options={[
                        { value: "global", label: t("leaderboard.toolbar.scope.global") },
                        { value: "friends", label: t("leaderboard.toolbar.scope.friends"), disabled: true, hint: t("leaderboard.toolbar.comingSoon") },
                    ]}
                />
                <DropdownMenu>
                    <DropdownMenuTrigger className={FILTER} aria-label={t("leaderboard.toolbar.serverLabel")}>
                        <Kicker className="mb-0">{t("leaderboard.toolbar.server")}</Kicker>
                        <span className="font-semibold text-foreground">{server === "All" ? t("leaderboard.toolbar.all") : server}</span>
                        <ChevronDown className="size-3 opacity-70" />
                    </DropdownMenuTrigger>
                    <DropdownMenuContent align="start" className="min-w-40">
                        <DropdownMenuItem onClick={() => onServer("All")} className={cn("cursor-pointer justify-between", server === "All" && "font-semibold")}>
                            {t("leaderboard.toolbar.allServers")}
                            <MenuCheck on={server === "All"} />
                        </DropdownMenuItem>
                        {SERVERS.map((code) => (
                            <DropdownMenuItem key={code} onClick={() => onServer(code)} className={cn("cursor-pointer justify-between", server === code && "font-semibold")}>
                                {code}
                                <MenuCheck on={server === code} />
                            </DropdownMenuItem>
                        ))}
                    </DropdownMenuContent>
                </DropdownMenu>
                <DropdownMenu>
                    <HintWrap hint={movementHint}>
                        <DropdownMenuTrigger className={FILTER} aria-label={t("leaderboard.toolbar.intervalLabel")} disabled={!movementAvailable}>
                            <Kicker className="mb-0">{t("leaderboard.toolbar.interval")}</Kicker>
                            <span className="font-semibold text-foreground tabular-nums">{t(intervalShortKey)}</span>
                            <ChevronDown className="size-3 opacity-70" />
                        </DropdownMenuTrigger>
                    </HintWrap>
                    <DropdownMenuContent align="start" className="min-w-44">
                        {INTERVALS.map((opt) => (
                            <DropdownMenuItem key={opt.value} onClick={() => onInterval(opt.value)} className={cn("cursor-pointer justify-between", interval === opt.value && "font-semibold")}>
                                {t(opt.labelKey)}
                                <MenuCheck on={interval === opt.value} />
                            </DropdownMenuItem>
                        ))}
                    </DropdownMenuContent>
                </DropdownMenu>
                <HintWrap hint={movementHint ?? (movementOnly ? t("leaderboard.toolbar.movementOnly.on") : t("leaderboard.toolbar.movementOnly.off"))}>
                    <button
                        type="button"
                        role="switch"
                        aria-checked={movementOnly}
                        disabled={!movementAvailable}
                        onClick={() => onMovementOnly(!movementOnly)}
                        className={cn(
                            "inline-flex h-9 pointer-coarse:h-11 cursor-pointer items-center gap-1.5 rounded-lg border px-2.5 font-medium font-sans text-xs leading-none transition-colors disabled:cursor-not-allowed disabled:opacity-50 sm:h-8",
                            movementOnly && movementAvailable
                                ? "border-primary/40 bg-[color-mix(in_srgb,var(--primary)_10%,transparent)] text-foreground hover:bg-[color-mix(in_srgb,var(--primary)_14%,transparent)] [&>svg]:text-primary"
                                : "border-input bg-card text-foreground hover:border-foreground/20 disabled:hover:border-input",
                        )}
                    >
                        <TrendingUp className={cn("size-3.5", movementOnly && movementAvailable ? "opacity-100" : "opacity-70")} aria-hidden />
                        <span>{t("leaderboard.toolbar.movementOnly")}</span>
                    </button>
                </HintWrap>
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

/** Single-choice filter: a kit ToggleGroup, so it carries pressed state and a 44px coarse target, not tab roles without panels. */
function Segmented<T extends string>({ label, value, onChange, options }: { label: string; value: T; onChange: (next: T) => void; options: ISegmentedOption<T>[] }) {
    return (
        <ToggleGroup aria-label={label} variant="outline" value={[value]} onValueChange={(next) => next[0] && onChange(next[0] as T)}>
            {options.map((opt) => (
                <HintWrap key={opt.value} hint={opt.hint}>
                    <ToggleGroupItem value={opt.value} disabled={opt.disabled} className="px-3 text-xs sm:text-xs">
                        {opt.label}
                    </ToggleGroupItem>
                </HintWrap>
            ))}
        </ToggleGroup>
    );
}

/** The chosen menu row: weight and a check mark, text stays foreground (primary text on the highlighted row measured 4.01 light). */
function MenuCheck({ on }: { on: boolean }) {
    return on ? <Check className="size-3.5 shrink-0" aria-hidden /> : <span className="size-3.5 shrink-0" aria-hidden />;
}

/**
 * Shows `hint` as a tooltip over a control that may be disabled. A disabled button never fires
 * hover, so the trigger is an inline wrapper around it; with no hint the control renders bare.
 */
function HintWrap({ hint, className, children }: { hint: string | undefined; className?: string; children: React.ReactElement }) {
    if (!hint) return children;
    return (
        <Tooltip>
            <TooltipTrigger render={<span className={cn("inline-flex", className)} />}>{children}</TooltipTrigger>
            <TooltipPopup>{hint}</TooltipPopup>
        </Tooltip>
    );
}
