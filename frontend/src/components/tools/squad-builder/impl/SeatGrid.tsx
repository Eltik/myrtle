import { useQuery } from "@tanstack/react-query";
import { ArrowRightLeft, Crown, FlaskConical, Handshake, MoreHorizontal, Plus, Trash2 } from "lucide-react";
import * as React from "react";
import { createPortal } from "react-dom";
import { ClassIcon } from "#/components/operators/list/impl/components/Icons";
import { cardVars } from "#/components/tools/release/impl/components/SkinCard";
import { eliteIcon, moduleIconURL, potentialIcon, skillIconURL, specializedIcon } from "#/components/tools/shared/icons";
import { Button } from "#/components/ui/button";
import { Dialog, DialogDescription, DialogHeader, DialogPanel, DialogPopup, DialogTitle } from "#/components/ui/dialog";
import { Menu, MenuItem, MenuPopup, MenuSeparator, MenuTrigger } from "#/components/ui/menu";
import { Skeleton } from "#/components/ui/skeleton";
import { operatorQueryOptions } from "#/lib/api/operators";
import { useGamedataServer, useT } from "#/lib/i18n";
import type { TypedT } from "#/lib/i18n/messages";
import { cn, getAvatarById, getPortraitById, parseOperatorName, rarityStarColor, subProfessionLabel } from "#/lib/utils";
import type { SquadMember } from "#/types/generated/SquadMember";
import type { IOperatorIndexEntry } from "#/types/operators";
import { type IResolvedSeat, SQUAD_SIZE, SUPPORT_SEAT } from "../model";
import { useSeatDrag } from "../useSeatDrag";
import styles from "./SeatCard.module.css";
import type { messages } from "./SeatGrid.messages";

type GridT = TypedT<typeof messages>;

const SEAT_INDEXES: readonly number[] = Array.from({ length: SQUAD_SIZE }, (_, i) => i);
/** Column counts from `SQUAD_SIZE`: compact puts every seat and the support in one row; wide puts the seats in two rows with the support in the last column. */
const GRID_COLUMNS = { "--seat-cols-compact": SQUAD_SIZE + 1, "--seat-cols": Math.ceil(SQUAD_SIZE / 2) + 1 } as React.CSSProperties;

/** A minimized tile's height floor, so a row of empty tiles stands as tall as a filled one. */
const COMPACT_MIN_H = "min-h-14";

/**
 * The width `cardVars()` scales the card's pool sheet to before the grid has
 * been measured: the Release Planner's own card width, so the first paint is
 * the planner's card and the measured one only rescales the sheet.
 */
const UNMEASURED_CARD_W = 140;

export interface ISeatGridProps {
    members: ReadonlyArray<SquadMember | null>;
    seats: ReadonlyArray<IResolvedSeat | null>;
    supportMember: SquadMember | null;
    support: IResolvedSeat | null;
    operators: ReadonlyMap<string, IOperatorIndexEntry>;
    compact: boolean;
    readOnly: boolean;
    /** The seat whose editor or picker is open; `SUPPORT_SEAT` for the support. */
    focus: number | null;
    /** A seat the analysis panels point at. */
    highlight: number | null;
    onFocus: (seat: number) => void;
    onSwap: (a: number, b: number) => void;
    onRemove: (seat: number) => void;
    onOpenDps: (seat: number) => void;
}

/**
 * The width of one grid column, read off the first cell and kept current by
 * a ResizeObserver. The card's CSS derives every measurement from this one
 * number the way the planner's card does, so a 105px phone column and a
 * 190px desktop column draw the same card at two sizes.
 */
function useColumnWidth(ref: React.RefObject<HTMLUListElement | null>, enabled: boolean): number {
    const [width, setWidth] = React.useState(0);
    React.useEffect(() => {
        if (!enabled) return;
        const cell = ref.current?.querySelector("li");
        if (!cell) return;
        const observer = new ResizeObserver((entries) => {
            const next = entries[0]?.contentRect.width ?? 0;
            if (next > 0) setWidth(next);
        });
        observer.observe(cell);
        return () => observer.disconnect();
    }, [ref, enabled]);
    return width > 0 ? width : UNMEASURED_CARD_W;
}

/**
 * The `SQUAD_SIZE` seats and the support in ONE grid. On `lg` and up the
 * support takes the last column beside the two rows of seats, set off by a rule drawn in
 * the gutter, with its caption directly under the card in the same cell;
 * below `lg` the support flows after the last seat like any other card, so
 * nothing ever sits beside a narrow grid. Seven columns need `lg`: at `md` a
 * column is 99px and the badges go below their floors. Only the twelve seats
 * carry `data-seat`, so a drag never lands on the support.
 */
export function SeatGrid(props: ISeatGridProps): React.ReactElement {
    const t: GridT = useT("tools");
    const { members, seats, supportMember, support, operators, compact, readOnly, focus, highlight, onFocus, onSwap, onRemove, onOpenDps } = props;
    const { drag, onPointerDown, consumeClick } = useSeatDrag(onSwap, !readOnly);
    const [moving, setMoving] = React.useState<number | null>(null);
    const listRef = React.useRef<HTMLUListElement | null>(null);
    const width = useColumnWidth(listRef, !compact);
    const movingOp = moving !== null ? operators.get(members[moving]?.char_id ?? "") : undefined;
    const ghostOp = drag ? members[drag.from] : null;
    const supportOp = supportMember ? operators.get(supportMember.char_id) : undefined;

    return (
        <div>
            <ul ref={listRef} aria-label={t("squad.grid.aria")} style={GRID_COLUMNS} className={cn("grid gap-2 sm:gap-3", compact ? "grid-cols-6 md:grid-cols-[repeat(var(--seat-cols-compact),minmax(0,1fr))]" : "grid-cols-3 sm:grid-cols-4 lg:grid-cols-[repeat(var(--seat-cols),minmax(0,1fr))]")}>
                {SEAT_INDEXES.map((i) => {
                    const member = members[i] ?? null;
                    return (
                        <li key={`seat-${i}`} data-seat={i} className="min-w-0">
                            <SeatCard
                                seat={i}
                                member={member}
                                resolved={seats[i] ?? null}
                                op={member ? operators.get(member.char_id) : undefined}
                                width={width}
                                compact={compact}
                                readOnly={readOnly}
                                focused={focus === i}
                                highlighted={highlight === i}
                                dropTarget={drag !== null && drag.over === i && drag.from !== i}
                                dragging={drag?.from === i}
                                onPointerDown={member ? (e) => onPointerDown(e, i) : undefined}
                                onClick={() => {
                                    if (consumeClick()) return;
                                    onFocus(i);
                                }}
                                onMakeLeader={() => onSwap(i, 0)}
                                onMove={() => setMoving(i)}
                                onRemove={() => onRemove(i)}
                                onOpenDps={() => onOpenDps(i)}
                                t={t}
                            />
                        </li>
                    );
                })}
                <li
                    className={cn(
                        "relative flex min-w-0 flex-col gap-2",
                        compact ? "md:before:absolute md:before:inset-y-0 md:before:-left-1.5 md:before:w-px md:before:bg-border" : "lg:col-start-(--seat-cols) lg:row-span-2 lg:row-start-1 lg:before:absolute lg:before:inset-y-0 lg:before:-left-1.5 lg:before:w-px lg:before:bg-border",
                    )}
                >
                    <SeatCard
                        seat={SUPPORT_SEAT}
                        member={supportMember}
                        resolved={support}
                        op={supportOp}
                        width={width}
                        compact={compact}
                        readOnly={readOnly}
                        focused={focus === SUPPORT_SEAT}
                        highlighted={highlight === SUPPORT_SEAT}
                        dropTarget={false}
                        dragging={false}
                        onClick={() => onFocus(SUPPORT_SEAT)}
                        onRemove={() => onRemove(SUPPORT_SEAT)}
                        onOpenDps={() => onOpenDps(SUPPORT_SEAT)}
                        t={t}
                    />
                    {!compact && <p className="text-[10.5px] text-muted-foreground leading-snug">{t("squad.grid.supportCaption")}</p>}
                </li>
            </ul>
            {!readOnly && !compact && <p className="mt-2 text-[11.5px] text-muted-foreground">{t("squad.grid.dragHint")}</p>}

            <Dialog open={moving !== null} onOpenChange={(open) => !open && setMoving(null)}>
                <DialogPopup className="max-w-sm">
                    <DialogHeader>
                        <DialogTitle>{t("squad.grid.moveTitle", { name: movingOp?.name ?? "" })}</DialogTitle>
                        <DialogDescription>{t("squad.grid.moveDesc")}</DialogDescription>
                    </DialogHeader>
                    <DialogPanel>
                        <div className="grid grid-cols-4 gap-2">
                            {SEAT_INDEXES.map((i) => (
                                <Button
                                    key={`move-${i}`}
                                    variant={i === moving ? "secondary" : "outline"}
                                    disabled={i === moving}
                                    onClick={() => {
                                        if (moving !== null) onSwap(moving, i);
                                        setMoving(null);
                                    }}
                                >
                                    {t("squad.grid.moveSeat", { seat: i + 1 })}
                                </Button>
                            ))}
                        </div>
                    </DialogPanel>
                </DialogPopup>
            </Dialog>

            {drag && ghostOp && typeof document !== "undefined"
                ? createPortal(
                      <div aria-hidden="true" className="pointer-events-none fixed z-80 size-16 -translate-x-1/2 -translate-y-1/2 rotate-3 overflow-hidden rounded-lg border-2 border-primary bg-card shadow-xl" style={{ left: drag.x, top: drag.y }}>
                          <img alt="" src={getAvatarById(ghostOp.char_id)} className="size-full object-cover" />
                      </div>,
                      document.body,
                  )
                : null}
        </div>
    );
}

interface ISeatCardProps {
    /** 0 to 11, or `SUPPORT_SEAT`. */
    seat: number;
    member: SquadMember | null;
    resolved: IResolvedSeat | null;
    op: IOperatorIndexEntry | undefined;
    /** The grid column's width in px; the card scales from it. */
    width: number;
    compact: boolean;
    readOnly: boolean;
    focused: boolean;
    highlighted: boolean;
    dropTarget: boolean;
    dragging: boolean;
    onPointerDown?: (e: React.PointerEvent<HTMLElement>) => void;
    onClick: () => void;
    onMakeLeader?: () => void;
    onMove?: () => void;
    onRemove: () => void;
    onOpenDps: () => void;
    t: GridT;
}

function ringClass(focused: boolean, highlighted: boolean, dropTarget: boolean): string {
    if (dropTarget) return "border-primary ring-2 ring-primary ring-offset-2 ring-offset-background";
    if (focused) return "border-primary ring-2 ring-primary/40";
    if (highlighted) return "ring-2 ring-info/70";
    return "";
}

/** The leader's and the support's tag; a plain seat has none. */
function RoleTag({ seat, t }: { seat: number; t: GridT }): React.ReactElement | null {
    if (seat === SUPPORT_SEAT) {
        return (
            <span className={cn(styles.role, styles.roleSupport)}>
                <Handshake aria-hidden="true" />
                {t("squad.grid.support")}
            </span>
        );
    }
    if (seat === 0) {
        return (
            <span className={cn(styles.role, styles.roleLeader)}>
                <Crown aria-hidden="true" />
                {t("squad.grid.leader")}
            </span>
        );
    }
    return null;
}

function SeatCard(props: ISeatCardProps): React.ReactElement {
    const { seat, member, resolved, op, width, compact, readOnly, focused, highlighted, dropTarget, dragging, onPointerDown, onClick, onMakeLeader, onMove, onRemove, onOpenDps, t } = props;
    const [menuOpen, setMenuOpen] = React.useState(false);
    const [artFailed, setArtFailed] = React.useState(false);
    const isSupport = seat === SUPPORT_SEAT;

    if (!member) {
        return (
            <button
                type="button"
                onClick={onClick}
                disabled={readOnly}
                aria-label={isSupport ? t("squad.grid.supportAdd") : t("squad.grid.addAria", { seat: seat + 1 })}
                className={cn(
                    "group/empty relative flex w-full cursor-pointer flex-col overflow-hidden border border-border border-dashed bg-card/40 text-muted-foreground outline-none transition-colors hover:border-primary/60 hover:bg-card/70 hover:text-foreground focus-visible:ring-2 focus-visible:ring-ring disabled:cursor-default disabled:opacity-60",
                    compact ? cn("aspect-square rounded-lg", COMPACT_MIN_H) : cn(styles.empty, "rounded-sm"),
                    ringClass(focused, highlighted, dropTarget),
                )}
            >
                {!compact && !isSupport && seat !== 0 && <span className="absolute top-1.5 left-2 font-mono text-[10px] text-muted-foreground/70 tabular-nums">{t("squad.grid.seatTag", { seat: seat + 1 })}</span>}
                {!compact && seat === 0 && (
                    <span className="absolute top-1.5 left-2 flex items-center gap-1 font-semibold text-[10px] text-primary uppercase tracking-wide">
                        <Crown aria-hidden="true" className="size-3" />
                        {t("squad.grid.leader")}
                    </span>
                )}
                {!compact && isSupport && (
                    <span className="absolute top-1.5 left-2 flex items-center gap-1 font-semibold text-[10px] text-info uppercase tracking-wide">
                        <Handshake aria-hidden="true" className="size-3" />
                        {t("squad.grid.support")}
                    </span>
                )}
                <span className="flex flex-1 flex-col items-center justify-center gap-2 px-2">
                    <span className={cn("flex items-center justify-center rounded-full border border-muted-foreground/40 border-dashed transition-colors group-hover/empty:border-primary/60", compact ? "size-6" : "size-10")}>
                        <Plus className={compact ? "size-3.5" : "size-5"} aria-hidden="true" />
                    </span>
                    {!compact && <span className="text-center font-semibold text-[11px] uppercase leading-tight tracking-wide">{isSupport ? t("squad.grid.supportAdd") : t("squad.grid.add")}</span>}
                    {compact && isSupport && <Handshake aria-hidden="true" className="size-3 text-info" />}
                    {compact && seat === 0 && <Crown aria-hidden="true" className="size-3 text-primary" />}
                </span>
            </button>
        );
    }

    if (!op || !resolved) {
        return <Skeleton className={cn("w-full", compact ? cn("aspect-square rounded-lg", COMPACT_MIN_H) : cn(styles.empty, "rounded-sm"))} />;
    }

    const { build } = resolved;
    const color = rarityStarColor(op.rarity);
    const displayName = parseOperatorName(op.name).displayName;
    const ariaLabel = isSupport ? t("squad.grid.supportAria", { name: op.name }) : t("squad.grid.seatAria", { seat: seat + 1, name: op.name });
    const eliteLabel = t("squad.grid.eliteAria", { phase: build.phase });
    const potentialLabel = t("squad.grid.potentialAria", { rank: build.potential + 1 });
    const onContextMenu = (e: React.MouseEvent) => {
        if (readOnly) return;
        e.preventDefault();
        setMenuOpen(true);
    };

    const card = compact ? (
        <button
            type="button"
            onPointerDown={onPointerDown}
            onClick={onClick}
            onContextMenu={onContextMenu}
            aria-label={ariaLabel}
            title={`${op.name}, ${t("squad.grid.level", { level: build.level })}`}
            className={cn(
                "relative block aspect-square w-full cursor-pointer touch-manipulation select-none overflow-hidden rounded-lg border border-border bg-muted outline-none transition-[border-color,box-shadow,opacity] [-webkit-touch-callout:none] hover:border-primary/50 focus-visible:ring-2 focus-visible:ring-ring",
                COMPACT_MIN_H,
                ringClass(focused, highlighted, dropTarget),
                dragging && "opacity-40",
            )}
            style={{ boxShadow: `inset 0 -3px 0 ${color}` }}
        >
            <img alt="" draggable={false} src={getAvatarById(op.id)} className="size-full object-cover" loading="lazy" decoding="async" />
            <span className="absolute inset-x-0 bottom-[3px] flex items-center justify-center gap-0.5 bg-background/80 px-0.5 font-mono text-[10px] text-foreground tabular-nums leading-4">
                {/* Below lg a minimized tile is 46px wide (768px, 13 columns): the number alone fits, the icon does not. */}
                <img alt={eliteLabel} src={eliteIcon(build.phase)} className="icon-theme-aware hidden size-3 object-contain lg:block" />
                <span className="truncate">{t("squad.grid.level", { level: build.level })}</span>
            </span>
            {seat === 0 && <Crown aria-label={t("squad.grid.leader")} className="absolute top-0.5 left-0.5 size-3.5 rounded-sm bg-background/80 p-px text-primary" />}
            {isSupport && <Handshake aria-label={t("squad.grid.support")} className="absolute top-0.5 left-0.5 size-3.5 rounded-sm bg-background/80 p-px text-info" />}
        </button>
    ) : (
        <button
            type="button"
            onPointerDown={onPointerDown}
            onClick={onClick}
            onContextMenu={onContextMenu}
            aria-label={ariaLabel}
            title={`${op.name}, ${subProfessionLabel(op)}`}
            data-focused={focused || undefined}
            data-highlight={(highlighted && !focused) || undefined}
            data-drop={dropTarget || undefined}
            data-dragging={dragging || undefined}
            className={cn(styles.card, !readOnly && !isSupport && styles.draggable)}
        >
            <span className={styles.art}>
                {artFailed ? (
                    <span aria-hidden="true" className={styles.initial}>
                        {displayName.charAt(0)}
                    </span>
                ) : (
                    <img alt="" draggable={false} src={getPortraitById(op.id)} onError={() => setArtFailed(true)} className={styles.portrait} loading="lazy" decoding="async" />
                )}
                <span className={styles.shine} />
                {/* The planner's corner block, carrying the build: level, class, promotion. */}
                <span className={styles.corner}>
                    <span className={styles.level} title={t("squad.grid.level", { level: build.level })}>
                        {build.level}
                    </span>
                    <span className={styles.classBox} title={subProfessionLabel(op)}>
                        <ClassIcon profession={op.profession} size={26} />
                    </span>
                    <span className={styles.eliteBox} title={eliteLabel}>
                        <img alt={eliteLabel} src={eliteIcon(build.phase)} />
                    </span>
                </span>
                <span role="img" aria-label={t("squad.grid.rarityAria", { rarity: op.rarity })} className={styles.stars} style={{ "--star-color": rarityStarColor(op.rarity) } as React.CSSProperties}>
                    {"★".repeat(op.rarity)}
                </span>
                <RoleTag seat={seat} t={t} />
                <span className={styles.band}>
                    <SeatLoadout resolved={resolved} op={op} t={t} />
                    <span className={styles.potential} title={potentialLabel}>
                        <img alt={potentialLabel} src={potentialIcon(build.potential)} />
                    </span>
                </span>
            </span>
            <span className={styles.strip}>
                {!isSupport && <span className={styles.seatNo}>{seat + 1}</span>}
                <span className={styles.name}>{displayName}</span>
            </span>
        </button>
    );

    return (
        // The planner's card variables sit on the wrapper so the card and the menu square over it scale together.
        <div className="group/seat relative" style={compact ? undefined : cardVars([color], width)}>
            {card}
            {!readOnly && (
                <Menu open={menuOpen} onOpenChange={setMenuOpen}>
                    <MenuTrigger
                        aria-label={t("squad.grid.menu")}
                        className={cn(
                            compact
                                ? "absolute top-0.5 right-0.5 flex size-5 cursor-pointer items-center justify-center rounded-md bg-background/80 text-muted-foreground transition-[opacity,color] hover:text-foreground focus-visible:opacity-100 focus-visible:ring-2 focus-visible:ring-ring data-popup-open:text-foreground data-popup-open:opacity-100 sm:opacity-0 sm:group-hover/seat:opacity-100"
                                : cn(styles.menu, "sm:opacity-70 sm:group-hover/seat:opacity-100"),
                        )}
                    >
                        <MoreHorizontal className={compact ? "size-4" : undefined} aria-hidden="true" />
                    </MenuTrigger>
                    <MenuPopup align="end">
                        {!isSupport && onMakeLeader && (
                            <MenuItem disabled={seat === 0} onClick={onMakeLeader}>
                                <Crown />
                                {t("squad.grid.makeLeader")}
                            </MenuItem>
                        )}
                        {!isSupport && onMove && (
                            <MenuItem onClick={onMove}>
                                <ArrowRightLeft />
                                {t("squad.grid.move")}
                            </MenuItem>
                        )}
                        <MenuItem onClick={onOpenDps}>
                            <FlaskConical />
                            {t("squad.grid.openDps")}
                        </MenuItem>
                        <MenuSeparator />
                        <MenuItem variant="destructive" onClick={onRemove}>
                            <Trash2 />
                            {t("squad.grid.remove")}
                        </MenuItem>
                    </MenuPopup>
                </Menu>
            )}
        </div>
    );
}

/**
 * The chosen skill and the module as two tiles in the card's build band. The
 * skill tile wears the mastery glyph, or the skill rank as a numeral below
 * mastery; the module tile wears its rank. The icons need the operator
 * detail, which the seat editor shares through the query cache; until it
 * lands the tiles show `S3` and the module's type label from the index.
 */
function SeatLoadout({ resolved, op, t }: { resolved: IResolvedSeat; op: IOperatorIndexEntry; t: GridT }): React.ReactElement {
    const server = useGamedataServer();
    const { data: detail } = useQuery(operatorQueryOptions(resolved.member.char_id, server));
    const { member, build } = resolved;
    // A 1 or 2 star operator has no skill at all (`skillCount` 0), which is not the same as a skill whose icon has not loaded.
    const hasSkill = op.skillCount > 0;
    const skill = hasSkill ? detail?.skills?.[member.skill_index] : undefined;
    const skillShort = t("squad.grid.skillShort", { skill: member.skill_index + 1 });
    const skillName = skill?.static?.levels?.[0]?.name ?? t("squad.grid.skillAria", { skill: member.skill_index + 1 });
    const skillRank = build.mastery > 0 ? t("squad.grid.masteryAria", { mastery: build.mastery }) : t("squad.grid.skillLevelAria", { level: build.skill_level });
    const indexModule = member.module_id ? op.modules.find((m) => m.uniEquipId === member.module_id) : undefined;
    const moduleLabel = indexModule ? [indexModule.typeName1, indexModule.typeName2].filter(Boolean).join("-") : "";
    const mod = member.module_id ? detail?.modules?.find((m) => m.uniEquipId === member.module_id) : undefined;
    const moduleRank = t("squad.grid.moduleRank", { rank: build.module_level });

    return (
        <>
            {hasSkill ? (
                <span className={styles.tile} title={`${skillName}, ${skillRank}`}>
                    {skill ? <img alt={skillName} src={skillIconURL(skill)} loading="lazy" /> : <span className={styles.tileText}>{skillShort}</span>}
                    {build.mastery > 0 ? (
                        <img alt={skillRank} src={specializedIcon(build.mastery)} className={styles.glyph} />
                    ) : (
                        <span role="img" aria-label={skillRank} className={styles.rank}>
                            {build.skill_level}
                        </span>
                    )}
                </span>
            ) : (
                <span role="img" aria-label={t("squad.grid.noSkill")} title={t("squad.grid.noSkill")} className={cn(styles.tile, styles.tileEmpty)} />
            )}
            {member.module_id ? (
                <span className={styles.tile} title={moduleLabel ? `${moduleLabel}, ${moduleRank}` : moduleRank}>
                    {mod ? <img alt={moduleLabel || mod.uniEquipName} src={moduleIconURL(mod)} loading="lazy" /> : <span className={styles.tileText}>{moduleLabel}</span>}
                    {build.module_level > 0 && (
                        <span role="img" aria-label={moduleRank} className={styles.rank}>
                            {build.module_level}
                        </span>
                    )}
                </span>
            ) : (
                <span role="img" aria-label={t("squad.grid.noModule")} title={t("squad.grid.noModule")} className={cn(styles.tile, styles.tileEmpty)} />
            )}
        </>
    );
}
