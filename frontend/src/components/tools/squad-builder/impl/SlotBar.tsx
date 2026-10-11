import { Link } from "@tanstack/react-router";
import type * as React from "react";
import { Badge } from "#/components/ui/badge";
import { Button } from "#/components/ui/button";
import { Skeleton } from "#/components/ui/skeleton";
import { useFormatters, useT } from "#/lib/i18n";
import type { TypedT } from "#/lib/i18n/messages";
import { cn, getAvatarById } from "#/lib/utils";
import type { GameSquad } from "#/types/generated/GameSquad";
import type { SavedSquad } from "#/types/generated/SavedSquad";
import { SLOT_LIMIT, SQUAD_SIZE } from "../model";
import type { messages } from "./SlotBar.messages";

type SlotT = TypedT<typeof messages>;

const SLOT_NUMBERS: readonly number[] = Array.from({ length: SLOT_LIMIT }, (_, i) => i + 1);

export interface ISlotBarProps {
    signedIn: boolean;
    isLoading: boolean;
    saved: ReadonlyArray<SavedSquad | null>;
    synced: readonly GameSquad[];
    activeSlot: number | null;
    dirty: boolean;
    onLoadSlot: (slot: number) => void;
    onLoadGame: (squad: GameSquad) => void;
    onSignIn: () => void;
}

export function SlotBar({ signedIn, isLoading, saved, synced, activeSlot, dirty, onLoadSlot, onLoadGame, onSignIn }: ISlotBarProps): React.ReactElement {
    const t: SlotT = useT("tools");
    const fmt = useFormatters();
    const latestSync = synced.reduce<string | null>((acc, s) => (!acc || s.synced_at > acc ? s.synced_at : acc), null);

    return (
        <div className={cn("grid gap-4", signedIn && "lg:grid-cols-[minmax(0,3fr)_minmax(0,2fr)]")}>
            <section className="rounded-xl border border-border bg-card p-3">
                <h2 className="mb-2 font-semibold text-[13px]">{t("squad.slots.title")}</h2>
                {!signedIn ? (
                    <div className="flex flex-wrap items-center gap-3 rounded-lg border border-border border-dashed px-3 py-3">
                        <p className="flex-1 text-[12.5px] text-muted-foreground">{t("squad.slots.signIn")}</p>
                        <Button size="sm" variant="outline" onClick={onSignIn}>
                            {t("squad.slots.signInButton")}
                        </Button>
                    </div>
                ) : isLoading ? (
                    <div className="grid grid-cols-2 gap-2 sm:grid-cols-4">
                        {SLOT_NUMBERS.map((slot) => (
                            <Skeleton key={`slot-skel-${slot}`} className="h-[68px] rounded-lg" />
                        ))}
                    </div>
                ) : (
                    <ul className="grid grid-cols-2 gap-2 sm:grid-cols-4">
                        {Array.from({ length: SLOT_LIMIT }, (_, i) => {
                            const slot = i + 1;
                            const squad = saved[i] ?? null;
                            const active = activeSlot === slot;
                            const count = squad ? squad.members.filter((m) => m !== null).length : 0;
                            return (
                                <li key={`slot-${slot}`}>
                                    <button
                                        type="button"
                                        onClick={() => onLoadSlot(slot)}
                                        aria-label={t("squad.slots.loadAria", { slot })}
                                        aria-current={active ? "true" : undefined}
                                        className={cn(
                                            "flex h-full w-full cursor-pointer items-start gap-2 rounded-lg border bg-background/40 p-2 text-left outline-none transition-colors focus-visible:ring-2 focus-visible:ring-ring",
                                            active ? "border-primary ring-1 ring-primary/30" : "border-border hover:border-primary/50",
                                            !squad && "border-dashed",
                                        )}
                                    >
                                        <span className={cn("flex size-6 shrink-0 items-center justify-center rounded-md font-mono font-semibold text-[11px]", active ? "bg-primary text-primary-foreground" : "bg-muted text-muted-foreground")}>{slot}</span>
                                        <span className="min-w-0 flex-1">
                                            {squad ? (
                                                <>
                                                    <span className="block truncate font-medium text-[12.5px] leading-tight">{squad.name.trim() || t("squad.slots.untitled", { slot })}</span>
                                                    <span className="block font-mono text-[10.5px] text-muted-foreground">{t("squad.slots.count", { count, total: SQUAD_SIZE })}</span>
                                                    <span className="block truncate text-[10.5px] text-muted-foreground">{t("squad.slots.saved", { date: fmt.date(squad.updated_at) })}</span>
                                                </>
                                            ) : (
                                                <>
                                                    <span className="block font-medium text-[12.5px] text-muted-foreground leading-tight">{t("squad.slots.empty")}</span>
                                                    <span className="block text-[10.5px] text-muted-foreground">{t("squad.slots.emptyDesc")}</span>
                                                </>
                                            )}
                                            {active && (
                                                <Badge variant={dirty ? "warning" : "secondary"} className="mt-1">
                                                    {dirty ? t("squad.slots.unsaved") : t("squad.slots.active")}
                                                </Badge>
                                            )}
                                        </span>
                                    </button>
                                </li>
                            );
                        })}
                    </ul>
                )}
            </section>

            {signedIn && (
                <section className="rounded-xl border border-border bg-card p-3">
                    <div className="mb-2 flex items-baseline justify-between gap-2">
                        <h2 className="font-semibold text-[13px]">{t("squad.slots.game")}</h2>
                        {latestSync && <span className="truncate text-[10.5px] text-muted-foreground">{t("squad.slots.gameSynced", { date: fmt.date(latestSync) })}</span>}
                    </div>
                    {synced.length === 0 ? (
                        <div className="flex flex-wrap items-center gap-3 rounded-lg border border-border border-dashed px-3 py-3">
                            <p className="flex-1 text-[12.5px] text-muted-foreground">{t("squad.slots.gameEmpty")}</p>
                            <Button size="sm" variant="outline" render={<Link to="/settings" />}>
                                {t("squad.slots.gameEmptyLink")}
                            </Button>
                        </div>
                    ) : (
                        <>
                            <p className="mb-2 text-[11.5px] text-muted-foreground">{t("squad.slots.gameDesc")}</p>
                            <ul className="grid grid-cols-2 gap-2">
                                {synced.map((sq) => {
                                    const name = sq.name.trim() || t("squad.slots.gameUntitled", { index: sq.index + 1 });
                                    const ids = sq.slots.filter((s) => s !== null).map((s) => s.char_id);
                                    return (
                                        <li key={`game-${sq.index}`}>
                                            <button
                                                type="button"
                                                onClick={() => onLoadGame(sq)}
                                                aria-label={t("squad.slots.gameAria", { name })}
                                                className="flex w-full cursor-pointer flex-col gap-1.5 rounded-lg border border-border bg-background/40 p-2 text-left outline-none transition-colors hover:border-primary/50 focus-visible:ring-2 focus-visible:ring-ring"
                                            >
                                                <span className="flex items-baseline justify-between gap-2">
                                                    <span className="truncate font-medium text-[12.5px]">{name}</span>
                                                    <span className="shrink-0 font-mono text-[10.5px] text-muted-foreground">{t("squad.slots.count", { count: ids.length, total: SQUAD_SIZE })}</span>
                                                </span>
                                                <span className="flex -space-x-1.5 overflow-hidden">
                                                    {ids.slice(0, 8).map((id) => (
                                                        <img key={id} alt="" src={getAvatarById(id)} className="size-6 rounded-full border border-card bg-muted object-cover" loading="lazy" />
                                                    ))}
                                                </span>
                                            </button>
                                        </li>
                                    );
                                })}
                            </ul>
                        </>
                    )}
                </section>
            )}
        </div>
    );
}
