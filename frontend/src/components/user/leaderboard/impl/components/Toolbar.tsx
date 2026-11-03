import { Search, TrendingUp, X } from "lucide-react";
import type React from "react";
import { Button } from "#/components/ui/button";
import { InputGroup, InputGroupAddon, InputGroupInput } from "#/components/ui/input-group";
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from "#/components/ui/select";
import { Toggle } from "#/components/ui/toggle";
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
    catalogError: boolean;
    onRetryCatalog: () => void;
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

/** One control height for the row: 44px under a coarse pointer, 36px on narrow screens, the site's 32px on desktop. */
const CONTROL_HEIGHT = "h-9 pointer-coarse:h-11 sm:h-8";
/** A kit Select trigger sized to the row; the label inside is plain muted sans, the value foreground. */
const FILTER = cn(CONTROL_HEIGHT, "w-full min-w-0 justify-start sm:w-auto");

export function Toolbar({ ranking, onRanking, catalog, materials, catalogError, onRetryCatalog, scope, onScope, server, onServer, interval, onInterval, movementOnly, onMovementOnly, query, onQuery }: IToolbarProps) {
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
                <RankByPicker ranking={ranking} onRanking={onRanking} catalog={catalog} materials={materials} catalogError={catalogError} onRetryCatalog={onRetryCatalog} variant="toolbar" className="w-full sm:w-auto sm:max-w-80" />
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

            {/* Below sm a two-column grid of equal-height controls: wrapped as a flex row, the
                segment, the two menus and the switch broke into ragged rows of mixed heights. */}
            <div className="grid grid-cols-2 gap-2 sm:flex sm:flex-wrap sm:items-center">
                <Segmented
                    label={t("leaderboard.toolbar.scope.label")}
                    value={scope}
                    onChange={onScope}
                    options={[
                        { value: "global", label: t("leaderboard.toolbar.scope.global") },
                        { value: "friends", label: t("leaderboard.toolbar.scope.friends"), disabled: true, hint: t("leaderboard.toolbar.comingSoon") },
                    ]}
                />
                <Select value={server} onValueChange={(next: ServerCode | "All" | null) => next && onServer(next)}>
                    <SelectTrigger className={FILTER} aria-label={t("leaderboard.toolbar.serverLabel")}>
                        <span className="text-muted-foreground">{t("leaderboard.toolbar.server")}</span>
                        <SelectValue className="font-semibold">{(value: ServerCode | "All") => (value === "All" ? t("leaderboard.toolbar.all") : value)}</SelectValue>
                    </SelectTrigger>
                    <SelectContent>
                        <SelectItem value="All">{t("leaderboard.toolbar.allServers")}</SelectItem>
                        {SERVERS.map((code) => (
                            <SelectItem key={code} value={code}>
                                {code}
                            </SelectItem>
                        ))}
                    </SelectContent>
                </Select>
                <Select value={interval} onValueChange={(next: LeaderboardInterval | null) => next && onInterval(next)} disabled={!movementAvailable}>
                    <HintWrap hint={movementHint} className="w-full sm:w-auto">
                        <SelectTrigger className={FILTER} aria-label={t("leaderboard.toolbar.intervalLabel")}>
                            <span className="text-muted-foreground">{t("leaderboard.toolbar.interval")}</span>
                            <SelectValue className="font-semibold tabular-nums">{() => t(intervalShortKey)}</SelectValue>
                        </SelectTrigger>
                    </HintWrap>
                    <SelectContent>
                        {INTERVALS.map((opt) => (
                            <SelectItem key={opt.value} value={opt.value}>
                                {t(opt.labelKey)}
                            </SelectItem>
                        ))}
                    </SelectContent>
                </Select>
                <HintWrap hint={movementHint ?? (movementOnly ? t("leaderboard.toolbar.movementOnly.on") : t("leaderboard.toolbar.movementOnly.off"))} className="col-span-2 w-full sm:w-auto">
                    <Toggle variant="outline" pressed={movementOnly && movementAvailable} disabled={!movementAvailable} onPressedChange={(next) => onMovementOnly(next)} className={cn(CONTROL_HEIGHT, "w-full gap-1.5 px-2.5 sm:w-auto")}>
                        <TrendingUp className="size-3.5" aria-hidden />
                        <span>{t("leaderboard.toolbar.movementOnly")}</span>
                    </Toggle>
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
        <ToggleGroup aria-label={label} variant="outline" value={[value]} onValueChange={(next) => next[0] && onChange(next[0] as T)} className="col-span-2 grid w-full auto-cols-fr grid-flow-col sm:flex sm:w-fit">
            {options.map((opt) => (
                <HintWrap key={opt.value} hint={opt.hint} className="w-full sm:w-auto">
                    <ToggleGroupItem value={opt.value} disabled={opt.disabled} className={cn(CONTROL_HEIGHT, "w-full px-3 sm:w-auto")}>
                        {opt.label}
                    </ToggleGroupItem>
                </HintWrap>
            ))}
        </ToggleGroup>
    );
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
