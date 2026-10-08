import { useQuery } from "@tanstack/react-query";
import { Link } from "@tanstack/react-router";
import { ArrowUpRightIcon, LockIcon } from "lucide-react";
import { createContext, type ReactNode, useContext, useMemo, useState } from "react";
import { EntityAvatar } from "#/components/tier-lists/entities";
import { useEntityLabels } from "#/components/tier-lists/kinds";
import { Button } from "#/components/ui/button";
import { Dialog, DialogContent, DialogDescription, DialogFooter, DialogHeader, DialogPanel, DialogTitle } from "#/components/ui/dialog";
import { OperatorAvatar } from "#/components/ui/operator-avatar";
import { Skeleton } from "#/components/ui/skeleton";
import { Switch } from "#/components/ui/switch";
import { operatorQueryOptions, operatorsIndexQueryOptions } from "#/lib/api/operators";
import type { ITierEntityOf, ITierOperator } from "#/lib/api/tier-entities";
import { upcomingQueryOptions } from "#/lib/api/upcoming";
import { type IRosterEntry, userRosterOperatorQueryOptions } from "#/lib/api/user";
import { useGamedataServer, useT } from "#/lib/i18n";
import type { TypedT } from "#/lib/i18n/messages";
import { cn } from "#/lib/utils";
import type { IOperatorIndexEntry } from "#/types/operators";
import { toOwnedEntry } from "../Roster/helpers";
import { OperatorDialog } from "../Roster/OperatorDialog";
import type { messages } from "./FavouriteDialogs.messages";
import { factionMembers, type RosterAccess, rosterById } from "./favourites";
import { OperatorFace } from "./OperatorFace";

type FavT = TypedT<typeof messages>;

/** Whose showcase this is, for the dialogs its tiles open. Absent (the editor's preview), a tile opens nothing. */
export interface IShowcasePlayer {
    uid: string;
    /** The player's name as the profile shows it. */
    name: string;
    access: RosterAccess;
}

export const ShowcasePlayerContext = createContext<IShowcasePlayer | null>(null);

export function useShowcasePlayer(): IShowcasePlayer | null {
    return useContext(ShowcasePlayerContext);
}

/**
 * A dialog whose body mounts on first open and stays mounted, so its queries
 * start only once asked for and the close animation still has content.
 */
function LazyDialog({ open, onOpenChange, children }: { open: boolean; onOpenChange: (open: boolean) => void; children: ReactNode }) {
    const [mounted, setMounted] = useState(open);
    if (open && !mounted) setMounted(true);
    return (
        <Dialog open={open} onOpenChange={onOpenChange}>
            {mounted && children}
        </Dialog>
    );
}

// ---------------------------------------------------------------------------
// Operator: the player's build

export interface IOperatorTarget {
    id: string;
    name: string;
    rarity: number;
    /** Where its game data comes from when not the reader's server (a CN-only operator). */
    server?: string;
}

export function OperatorFavouriteDialog({ operator, open, onOpenChange }: { operator: IOperatorTarget; open: boolean; onOpenChange: (open: boolean) => void }) {
    return (
        <LazyDialog open={open} onOpenChange={onOpenChange}>
            <OperatorFavouriteBody operator={operator} />
        </LazyDialog>
    );
}

/** The player's roster row for `id`: the row, `null` when they do not own it, `pending`, or `private` when the viewer may not read the roster. */
function useRosterRow(player: IShowcasePlayer, id: string): IRosterEntry | null | "pending" | "private" {
    const { access } = player;
    // Only while the profile's own roster request is still out: the one row is a smaller ask than waiting for all of them.
    // A 403 here throws, which reads as private; the dialog shows that instead of an error.
    const single = useQuery({ ...userRosterOperatorQueryOptions(player.uid, id), enabled: access.state === "loading", retry: false });
    if (access.state === "private") return "private";
    if (access.state === "ready") return access.roster.find((r) => r.operator_id === id) ?? null;
    if (single.isError) return "private";
    if (single.isPending) return "pending";
    return single.data ?? null;
}

function OperatorFavouriteBody({ operator }: { operator: IOperatorTarget }) {
    const t: FavT = useT("user");
    const player = useShowcasePlayer();
    if (!player) return null;
    return <OperatorFavouriteRow player={player} operator={operator} t={t} />;
}

function OperatorFavouriteRow({ player, operator, t }: { player: IShowcasePlayer; operator: IOperatorTarget; t: FavT }) {
    const row = useRosterRow(player, operator.id);
    if (row === "pending") return <BuildSkeleton name={operator.name} />;
    if (row === "private" || row === null) {
        return (
            <OperatorNote operator={operator} icon={row === "private" ? <LockIcon aria-hidden="true" className="size-3.5 shrink-0" /> : null}>
                {row === "private" ? t("profile.showcase.operator.private", { player: player.name }) : t("profile.showcase.operator.notOwned", { player: player.name })}
            </OperatorNote>
        );
    }
    return <OperatorBuild operator={operator} row={row} />;
}

/** The roster's own operator dialog, fed the one operator's table entry instead of the 23.9 MB table the Roster tab loads. */
function OperatorBuild({ operator, row }: { operator: IOperatorTarget; row: IRosterEntry }) {
    const readerServer = useGamedataServer();
    const server = operator.server ?? readerServer;
    const detail = useQuery(operatorQueryOptions(operator.id, server));
    const index = useQuery(operatorsIndexQueryOptions(server));
    if (detail.isPending || index.isPending) return <BuildSkeleton name={operator.name} />;
    const meta = index.data?.find((o) => o.id === operator.id) ?? null;
    const entry = toOwnedEntry(row, meta, detail.data ?? null);
    // With no index row the builder falls back to the id; the favourite's own name and rarity are better.
    return <OperatorDialog entry={meta ? entry : { ...entry, name: operator.name, rarity: operator.rarity }} />;
}

function BuildSkeleton({ name }: { name: string }) {
    const t: FavT = useT("user");
    return (
        <DialogContent className="max-h-[90vh] max-w-2xl overflow-hidden p-0">
            <DialogTitle className="sr-only">{name}</DialogTitle>
            <output aria-label={t("profile.showcase.operator.loading", { name })} className="flex flex-col gap-4 p-6">
                <Skeleton className="h-64 w-full rounded-xl" />
                <Skeleton className="h-5 w-1/2" />
                <Skeleton className="h-24 w-full" />
            </output>
        </DialogContent>
    );
}

/** The small dialog for an operator whose build is not shown: its avatar, its name, why, and a way to its page. */
function OperatorNote({ operator, icon, children }: { operator: IOperatorTarget; icon: ReactNode; children: ReactNode }) {
    const t: FavT = useT("user");
    return (
        <DialogContent className="max-w-sm">
            <DialogHeader className="flex-row items-center gap-4">
                <span aria-hidden="true" className="relative size-16 shrink-0 overflow-hidden rounded-xl border border-border bg-muted">
                    <OperatorAvatar charId={operator.id} name={operator.name} server={operator.server} />
                </span>
                <div className="flex min-w-0 flex-col gap-1.5 pr-6">
                    <DialogTitle className="truncate text-lg">{operator.name}</DialogTitle>
                    <DialogDescription className="flex items-center gap-1.5">
                        {icon}
                        <span>{children}</span>
                    </DialogDescription>
                </div>
            </DialogHeader>
            <DialogFooter variant="bare">
                <Button variant="outline" size="sm" render={<Link to="/operators/$id" params={{ id: operator.id }} />}>
                    {t("profile.showcase.operator.openPage")}
                    <ArrowUpRightIcon aria-hidden="true" />
                </Button>
            </DialogFooter>
        </DialogContent>
    );
}

// ---------------------------------------------------------------------------
// Faction: every operator in it

export function FactionFavouriteDialog({ faction, open, onOpenChange }: { faction: ITierEntityOf<"faction">; open: boolean; onOpenChange: (open: boolean) => void }) {
    return (
        <LazyDialog open={open} onOpenChange={onOpenChange}>
            <FactionBody faction={faction} />
        </LazyDialog>
    );
}

function FactionBody({ faction }: { faction: ITierEntityOf<"faction"> }) {
    const t: FavT = useT("user");
    const labels = useEntityLabels();
    const player = useShowcasePlayer();
    const server = useGamedataServer();
    const index = useQuery(operatorsIndexQueryOptions(server));
    // What /operators lists under Upcoming: out on CN, not yet on the reader's server. Labelled, not hidden.
    const upcoming = useQuery(upcomingQueryOptions(server));
    const [ownedOnly, setOwnedOnly] = useState(false);
    const [building, setBuilding] = useState<IOperatorIndexEntry | null>(null);

    const ref = { id: faction.id, powerLevel: faction.powerLevel };
    // biome-ignore lint/correctness/useExhaustiveDependencies: the faction's id and level are the inputs, not the object
    const members = useMemo(() => factionMembers(index.data ?? [], ref), [index.data, faction.id, faction.powerLevel]);
    // biome-ignore lint/correctness/useExhaustiveDependencies: as above
    const upcomingMembers = useMemo(() => factionMembers(upcoming.data ?? [], ref), [upcoming.data, faction.id, faction.powerLevel]);
    const access = player?.access ?? { state: "private" as const };
    const owned = useMemo(() => (access.state === "ready" ? rosterById(access.roster) : null), [access]);
    const ownedCount = owned ? members.filter((m) => owned.has(m.id)).length : 0;
    const shown = owned && ownedOnly ? members.filter((m) => owned.has(m.id)) : members;
    const level = labels.detail(faction)[0];
    const playerName = player?.name ?? "";

    return (
        <DialogContent className="max-h-[90vh] max-w-3xl">
            <DialogHeader className="flex-row items-center gap-4">
                <span aria-hidden="true" className="relative flex size-14 shrink-0 items-center justify-center overflow-hidden rounded-xl border border-white/10 bg-[linear-gradient(to_bottom,oklch(0.27_0.005_285),oklch(0.15_0.004_285))] p-1.5 text-white">
                    <EntityAvatar entity={faction} face="tile" tone="dark" />
                </span>
                <div className="flex min-w-0 flex-col gap-1 pr-6">
                    {level && <span className="font-mono text-[11px] text-muted-foreground uppercase tracking-widest">{level}</span>}
                    <DialogTitle className="truncate text-lg">{faction.name}</DialogTitle>
                    <DialogDescription className="tabular-nums">
                        {index.isPending ? " " : t("profile.showcase.faction.count", { count: members.length })}
                        {owned && !index.isPending && <span> · {t("profile.showcase.faction.ownedCount", { owned: ownedCount, count: members.length })}</span>}
                    </DialogDescription>
                </div>
            </DialogHeader>
            <DialogPanel className="flex flex-col gap-4">
                {owned ? (
                    // biome-ignore lint/a11y/noLabelWithoutControl: the Switch renders the control inside the label
                    <label className="flex w-fit cursor-pointer items-center gap-2 text-sm">
                        <Switch checked={ownedOnly} onCheckedChange={setOwnedOnly} />
                        {t("profile.showcase.faction.ownedOnly")}
                    </label>
                ) : (
                    access.state === "private" &&
                    player && (
                        <p className="m-0 flex items-center gap-1.5 text-muted-foreground text-xs">
                            <LockIcon aria-hidden="true" className="size-3.5 shrink-0" />
                            {t("profile.showcase.faction.private", { player: playerName })}
                        </p>
                    )
                )}
                {index.isPending ? (
                    <div className={GRID}>
                        {Array.from({ length: 10 }, (_, i) => (
                            // biome-ignore lint/suspicious/noArrayIndexKey: placeholders have no identity
                            <Skeleton key={i} className="aspect-2/3 rounded-md" />
                        ))}
                    </div>
                ) : shown.length === 0 ? (
                    <p className="m-0 py-6 text-center text-muted-foreground text-sm">{members.length === 0 ? t("profile.showcase.faction.empty") : t("profile.showcase.faction.emptyOwned", { player: playerName })}</p>
                ) : (
                    <ul aria-label={t("profile.showcase.faction.gridLabel", { name: faction.name })} className={cn(GRID, "m-0 list-none p-0")}>
                        {shown.map((op) => (
                            <li key={op.id}>
                                <MemberCard op={op} owned={owned?.has(op.id) ?? false} playerName={playerName} onBuild={() => setBuilding(op)} t={t} />
                            </li>
                        ))}
                    </ul>
                )}
                {!ownedOnly && upcomingMembers.length > 0 && (
                    <section className="flex flex-col gap-2">
                        <h3 className="m-0 font-mono text-[11px] text-muted-foreground uppercase tracking-widest">{t("profile.showcase.faction.upcoming", { count: upcomingMembers.length })}</h3>
                        <ul className={cn(GRID, "m-0 list-none p-0")}>
                            {upcomingMembers.map((op) => (
                                <li key={op.id}>
                                    <Link to="/operators/$id" params={{ id: op.id }} aria-label={t("profile.showcase.faction.upcomingAria", { name: op.name })} className={cn(CARD_CLASS, "opacity-80 hover:opacity-100")}>
                                        <OperatorFace entity={op} server="cn" />
                                    </Link>
                                </li>
                            ))}
                        </ul>
                    </section>
                )}
            </DialogPanel>
            {building && <OperatorFavouriteDialog operator={{ id: building.id, name: building.name, rarity: building.rarity }} open onOpenChange={(o) => !o && setBuilding(null)} />}
        </DialogContent>
    );
}

const GRID = "grid grid-cols-3 gap-2 sm:grid-cols-5 sm:gap-2.5";

/** The showcase operator tile's frame, at the grid's width. */
const CARD_CLASS = "group relative block aspect-2/3 w-full cursor-pointer overflow-hidden rounded-md border border-muted/50 bg-card text-left text-foreground no-underline outline-none transition-[border-radius,border-color] duration-150 hover:rounded-lg hover:border-muted focus-visible:ring-2 focus-visible:ring-ring";

function MemberCard({ op, owned, playerName, onBuild, t }: { op: IOperatorIndexEntry; owned: boolean; playerName: string; onBuild: () => void; t: FavT }) {
    const badge = owned ? (
        <span className="absolute top-1 left-1 z-20 rounded-sm bg-primary px-1 py-px font-medium font-mono text-[9px] text-primary-foreground uppercase leading-tight tracking-wide">{t("profile.showcase.faction.owned")}</span>
    ) : op.isNotObtainable ? (
        <span className="absolute top-1 left-1 z-20 rounded-sm bg-background/80 px-1 py-px font-mono text-[9px] text-muted-foreground uppercase leading-tight tracking-wide">{t("profile.showcase.faction.unobtainable")}</span>
    ) : null;
    if (owned) {
        return (
            <button type="button" onClick={onBuild} aria-label={t("profile.showcase.faction.cardBuild", { name: op.name, player: playerName })} className={cn(CARD_CLASS, "border-primary/50")}>
                {badge}
                <OperatorFace entity={op} />
            </button>
        );
    }
    return (
        <Link to="/operators/$id" params={{ id: op.id }} aria-label={t("profile.showcase.faction.cardPage", { name: op.name })} className={CARD_CLASS}>
            {badge}
            <OperatorFace entity={op} />
        </Link>
    );
}

/** A favourite operator as the dialog reads it. */
export function operatorTarget(entity: ITierOperator, server: string | undefined): IOperatorTarget {
    return { id: entity.id, name: entity.name, rarity: entity.rarity, server };
}
