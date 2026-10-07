import { useQuery } from "@tanstack/react-query";
import { Link } from "@tanstack/react-router";
import { ArrowUpRightIcon, EyeOffIcon, HeartIcon, LayoutGridIcon, ListOrderedIcon, PencilIcon, RotateCwIcon, SparklesIcon, TargetIcon } from "lucide-react";
import { type CSSProperties, lazy, type ReactNode, Suspense, useLayoutEffect, useMemo, useRef, useState } from "react";
import { compactGridSize } from "#/components/grids/compact";
import { GridBoard } from "#/components/grids/GridBoard";
import { gridToState } from "#/components/grids/state";
import { CampIcon, ClassIcon } from "#/components/operators/list/impl/components/Icons";
import { RARITY_BLUR_COLORS, RARITY_COLORS } from "#/components/operators/list/impl/constants";
import { TierListBoard } from "#/components/tier-lists/detail/TierListBoard";
import { EntityAvatar, entityAccent, entityShape } from "#/components/tier-lists/entities";
import { entityPage, useEntityLabels } from "#/components/tier-lists/kinds";
import { Button } from "#/components/ui/button";
import { Skeleton } from "#/components/ui/skeleton";
import { useAuth } from "#/hooks/use-auth";
import { gridQueryOptions } from "#/lib/api/grids";
import { publicPlansQueryOptions } from "#/lib/api/planner";
import { type ITierOperator, isEntityOfKind } from "#/lib/api/tier-entities";
import { tierListDetailQueryOptions } from "#/lib/api/tier-lists";
import type { IRosterEntry } from "#/lib/api/user";
import { userShowcaseQueryOptions } from "#/lib/api/user";
import { useGamedataServer, useT } from "#/lib/i18n";
import type { TypedT } from "#/lib/i18n/messages";
import { cn, getPortraitById, rarityToNumber } from "#/lib/utils";
import type { ShowcaseBlockView } from "#/types/generated/ShowcaseBlockView";
import type { IUserProfile } from "#/types/user";
import { draftFromView, type IShowcaseEntity, type ShowcaseDraftBlock, shownBlocks } from "../../../showcase";
import { PlanCard } from "../Plans/PlansTab";
import { CARD_PADDING, Kicker, StatCard } from "../Stats/primitives";
import { blockFit, showcaseRows } from "./rows";
import type { messages } from "./ShowcaseTab.messages";

// The editor (and the entity picker it opens) loads only when the owner edits.
const ShowcaseEditor = lazy(() => import("./ShowcaseEditor").then((m) => ({ default: m.ShowcaseEditor })));

/** Each block type's accent: the hairline across the card's top edge. */
export const BLOCK_ACCENT = {
    favourites: "var(--primary)",
    grid: "oklch(0.70 0.14 200)",
    tier_list: "oklch(0.74 0.17 75)",
    plan: "oklch(0.70 0.15 162)",
} as const;

/** A block's slot that hands its height to the card inside, so blocks side by side end on one line. */
const FILL_SLOT = "flex flex-col *:flex-1";

/** Tallest a tier list block's board shows before it fades out over a link to the full list. */
const TIER_PEEK_MAX_HEIGHT = "max-h-[560px]";

interface IShowcaseTabProps {
    uid: string;
    profile: IUserProfile;
    isOwner: boolean;
    roster: IRosterEntry[];
}

export function ShowcaseTab({ uid, profile, isOwner, roster }: IShowcaseTabProps) {
    const t: TypedT<typeof messages> = useT("user");
    const server = useGamedataServer();
    const { user } = useAuth();
    const query = useQuery(userShowcaseQueryOptions(uid, user?.id ?? null, server));
    const [editing, setEditing] = useState(false);
    const layout = profile.profile_layout ?? null;
    const hiddenTab = (id: string) => layout?.tabs.some((tab) => tab.id === id && !tab.visible) ?? false;

    if (query.isPending) return <ShowcaseSkeleton />;
    if (query.isError) {
        return (
            <div className="flex flex-col items-center gap-3 rounded-3xl border border-border bg-card px-8 py-12 text-center">
                <p className="text-muted-foreground text-sm">{t("profile.showcase.error")}</p>
                <Button type="button" variant="outline" size="sm" onClick={() => void query.refetch()}>
                    <RotateCwIcon />
                    {t("profile.showcase.retry")}
                </Button>
            </div>
        );
    }

    const view = query.data;
    if (editing && isOwner) {
        return (
            <Suspense fallback={<ShowcaseSkeleton />}>
                <ShowcaseEditor uid={uid} view={view} plansHidden={hiddenTab("plans")} onClose={() => setEditing(false)} />
            </Suspense>
        );
    }

    const blocks = shownBlocks(view, isOwner);
    if (blocks.length === 0) {
        // A visitor is never shown an empty showcase (the tab is hidden), so this is the owner's.
        if (!isOwner) return null;
        return <ShowcaseEmpty onStart={() => setEditing(true)} />;
    }

    const card = (i: number) => {
        const block = blocks[i];
        if (!block) return null;
        return <ShowcaseBlockCard view={block} uid={uid} isOwner={isOwner} roster={roster} plansHidden={hiddenTab("plans")} />;
    };
    const rows = showcaseRows(blocks.map((b) => blockFit({ type: b.block.type, removed: b.removed })));

    return (
        <section aria-label={t("profile.showcase.aria")} className="flex flex-col gap-4">
            {isOwner && (
                <div className="flex flex-col-reverse gap-2 sm:flex-row sm:items-center sm:justify-between">
                    {hiddenTab("showcase") ? (
                        <p className="m-0 flex items-center gap-2 text-muted-foreground text-sm">
                            <EyeOffIcon aria-hidden="true" className="size-4 shrink-0" />
                            {t("profile.showcase.tabHidden")}
                        </p>
                    ) : (
                        <span aria-hidden="true" className="hidden sm:block" />
                    )}
                    <Button type="button" variant="ghost" size="sm" className="self-end text-muted-foreground hover:text-foreground sm:self-auto" onClick={() => setEditing(true)}>
                        <PencilIcon />
                        {t("profile.showcase.edit")}
                    </Button>
                </div>
            )}
            {rows.map((row) =>
                // Keyed by the row's first block: a block's place is its identity (two favourites blocks may share a kind and title).
                row.kind === "full" ? (
                    <div key={row.index} className="min-w-0">
                        {card(row.index)}
                    </div>
                ) : row.kind === "anchor" ? (
                    // Side by side, the shorter side's cards grow to the taller side's height.
                    <div key={row.index} className="flex flex-col gap-4 lg:flex-row">
                        <div className={cn(FILL_SLOT, "min-w-0 max-w-full lg:shrink-0")}>{card(row.index)}</div>
                        {row.lane.length > 0 && (
                            <div className="flex min-w-0 flex-1 flex-col gap-4">
                                {row.lane.map((i) => (
                                    <div key={i} className={cn(FILL_SLOT, "min-w-0 flex-1")}>
                                        {card(i)}
                                    </div>
                                ))}
                            </div>
                        )}
                    </div>
                ) : (
                    <div key={row.items[0]} className="flex flex-wrap gap-4">
                        {row.items.map((i) => (
                            <div key={i} className={cn(FILL_SLOT, "min-w-0", flowItemClass(blocks[i]))}>
                                {card(i)}
                            </div>
                        ))}
                    </div>
                ),
            )}
        </section>
    );
}

/** A flowing block's share of its row: favourites grow into the room left, a plan stays a card's width. */
function flowItemClass(view: ShowcaseBlockView | undefined): string {
    if (view?.block.type === "plan" || view?.removed) return "flex-[1_1_20rem] sm:max-w-[34rem]";
    return "flex-[999_1_20rem]";
}

function ShowcaseSkeleton() {
    return (
        <div className="grid grid-cols-1 gap-4 lg:grid-cols-2">
            <Skeleton className="h-[44rem] rounded-2xl lg:col-span-1" />
            <div className="flex flex-col gap-4">
                <Skeleton className="h-80 rounded-2xl" />
                <Skeleton className="h-72 rounded-2xl" />
            </div>
        </div>
    );
}

function ShowcaseEmpty({ onStart }: { onStart: () => void }) {
    const t: TypedT<typeof messages> = useT("user");
    return (
        <div className="relative flex flex-col items-center justify-center gap-3 overflow-hidden rounded-3xl border border-border border-dashed bg-card px-6 py-14 text-center sm:px-8 sm:py-16">
            <div aria-hidden="true" className="pointer-events-none absolute inset-x-0 top-0 h-px bg-[linear-gradient(to_right,transparent,color-mix(in_oklch,var(--primary)_70%,transparent),transparent)]" />
            <span aria-hidden="true" className="flex size-11 items-center justify-center rounded-2xl border border-border bg-muted/50 text-primary">
                <SparklesIcon className="size-5" />
            </span>
            <span className="font-mono text-[11px] text-muted-foreground uppercase tracking-widest">{t("profile.showcase.empty.kicker")}</span>
            <h3 className="font-semibold text-lg tracking-tight">{t("profile.showcase.empty.title")}</h3>
            <p className="max-w-md text-muted-foreground text-sm">{t("profile.showcase.empty.desc")}</p>
            <Button type="button" className="mt-2" onClick={onStart}>
                <PencilIcon />
                {t("profile.showcase.empty.cta")}
            </Button>
        </div>
    );
}

interface IShowcaseBlockCardProps {
    view: ShowcaseBlockView;
    uid: string;
    isOwner: boolean;
    roster: IRosterEntry[];
    plansHidden: boolean;
}

/** One block as visitors see it. Its width comes from the row it sits in (see `rows.ts`). */
function ShowcaseBlockCard({ view, uid, isOwner, roster, plansHidden }: IShowcaseBlockCardProps) {
    // The same working-copy shape the editor uses, so both render a block's favourites alike.
    const draft = useMemo(() => draftFromView({ blocks: [view] })[0], [view]);
    if (!draft) return null;
    if (draft.removed) return <RemovedBlock type={draft.type} />;
    if (draft.type === "favourites") return <FavouritesBlock block={draft} />;
    if (draft.type === "grid") return <GridBlock slug={draft.slug} isOwner={isOwner} />;
    if (draft.type === "tier_list") return <TierListBlock slug={draft.slug} isOwner={isOwner} />;
    return <PlanBlock id={draft.id} uid={uid} roster={roster} isOwner={isOwner} plansHidden={plansHidden} />;
}

/** The chip that marks a gone block or entity. Only the owner is ever shown one. */
export function RemovedBadge() {
    const t: TypedT<typeof messages> = useT("user");
    return <span className="shrink-0 rounded-full bg-destructive/10 px-2 py-0.5 font-medium font-mono text-[11px] text-destructive leading-none">{t("profile.showcase.removed.badge")}</span>;
}

function RemovedBlock({ type }: { type: ShowcaseDraftBlock["type"] }) {
    const t: TypedT<typeof messages> = useT("user");
    const note = {
        favourites: t("profile.showcase.removed.favourites"),
        grid: t("profile.showcase.removed.grid"),
        tier_list: t("profile.showcase.removed.tierList"),
        plan: t("profile.showcase.removed.plan"),
    }[type];
    return (
        <div className="flex h-full items-start gap-3 rounded-2xl border border-border border-dashed bg-muted/20 px-4 py-3">
            <RemovedBadge />
            <p className="m-0 text-muted-foreground text-sm">{note}</p>
        </div>
    );
}

/** A block's header row: the kicker, an optional title and a trailing slot. */
function BlockHeader({ icon, kicker, title, end }: { icon: typeof HeartIcon; kicker: string; title?: string | null; end?: ReactNode }) {
    return (
        <div className="flex items-start justify-between gap-3">
            <div className="flex min-w-0 flex-col gap-1">
                <Kicker icon={icon} label={kicker} />
                {title && <h3 className="truncate font-semibold text-base text-foreground leading-tight tracking-tight">{title}</h3>}
            </div>
            {end}
        </div>
    );
}

/** The header's link to a grid's or tier list's own page. */
function OpenLink({ to, slug, title }: { to: "/grids/$slug" | "/tier-lists/$id"; slug: string; title: string }) {
    const t: TypedT<typeof messages> = useT("user");
    const className = "-mr-1.5 inline-flex shrink-0 items-center gap-1 rounded-md px-1.5 py-1 font-medium text-muted-foreground text-xs no-underline transition-colors hover:bg-accent hover:text-foreground";
    const body = (
        <>
            {t("profile.showcase.block.open")}
            <ArrowUpRightIcon aria-hidden="true" className="size-3.5" />
        </>
    );
    return to === "/grids/$slug" ? (
        <Link to={to} params={{ slug }} aria-label={t("profile.showcase.block.openAria", { title })} className={className}>
            {body}
        </Link>
    ) : (
        <Link to={to} params={{ id: slug }} aria-label={t("profile.showcase.block.openAria", { title })} className={className}>
            {body}
        </Link>
    );
}

function FavouritesBlock({ block }: { block: Extract<ShowcaseDraftBlock, { type: "favourites" }> }) {
    const t: TypedT<typeof messages> = useT("user");
    const labels = useEntityLabels();
    const title = block.title.trim();
    return (
        <StatCard color={BLOCK_ACCENT.favourites}>
            <div className={cn(CARD_PADDING, "flex flex-col gap-4")}>
                <BlockHeader icon={HeartIcon} kicker={labels.plural(block.kind)} title={title || null} end={<span className="shrink-0 text-muted-foreground text-xs tabular-nums">{t("profile.showcase.block.favourites.count", { count: block.entities.length })}</span>} />
                <FavouriteTiles entities={block.entities} />
            </div>
        </StatCard>
    );
}

/**
 * The tiles of a favourites block, in a row that wraps. Operators stand as
 * the 2 : 3 card of the operators page; every other kind keeps its own art's shape
 * (square, or a wide event banner), at one shared height.
 */
export function FavouriteTiles({ entities }: { entities: readonly IShowcaseEntity[] }) {
    return (
        <ul className="m-0 flex list-none flex-wrap gap-2.5 p-0 sm:gap-3">
            {entities.map((e) => (
                <FavouriteTile key={e.id} item={e} />
            ))}
        </ul>
    );
}

/** One tile's frame. Dark in both themes, as the art behind it is drawn for. */
const TILE_CLASS =
    "group relative flex items-center justify-center overflow-hidden rounded-xl border border-white/10 bg-[linear-gradient(to_bottom,color-mix(in_oklch,var(--tile-accent)_26%,oklch(0.24_0.005_285)),oklch(0.15_0.004_285))] text-lg text-white no-underline outline-none transition-[border-color,transform,box-shadow] duration-150 hover:-translate-y-0.5 hover:border-(--tile-accent) hover:shadow-[0_6px_18px_-6px_color-mix(in_oklch,var(--tile-accent)_45%,transparent)] focus-visible:ring-2 focus-visible:ring-ring";

/** An operator's card, drawn as the operators page draws it: no tinted backdrop, the portrait on the plain card. */
const OPERATOR_TILE_CLASS = "group relative block overflow-hidden rounded-md border border-muted/50 bg-card text-foreground no-underline outline-none transition-[border-radius,border-color] duration-150 hover:rounded-lg hover:border-muted focus-visible:ring-2 focus-visible:ring-ring";

/** Each shape's size. A portrait is the operators page's 2 : 3 card; a wide banner is 2.1 squares wide. */
const TILE_SIZE = {
    portrait: "w-[88px] aspect-2/3 sm:w-[104px] lg:w-[120px]",
    square: "size-[88px] sm:size-[112px] lg:size-[136px]",
    wide: "h-[88px] w-[185px] sm:h-[112px] sm:w-[235px] lg:h-[136px] lg:w-[286px]",
} as const;

function FavouriteTile({ item }: { item: IShowcaseEntity }) {
    const t: TypedT<typeof messages> = useT("user");
    const labels = useEntityLabels();
    const entity = item.entity;
    if (!entity) {
        const label = t("profile.showcase.removed.entity", { id: item.id });
        return (
            <li>
                <span role="img" aria-label={label} title={label} className={cn(TILE_CLASS, TILE_SIZE.square, "border-border border-dashed bg-muted/30 bg-none text-muted-foreground hover:translate-y-0")}>
                    <span className="line-clamp-3 break-all px-1 text-center font-mono text-[9px] leading-tight">{item.id}</span>
                </span>
            </li>
        );
    }
    const server = item.server ?? undefined;
    const operator = isEntityOfKind(entity, "operator") ? entity : null;
    const label = labels.tileLabel(entity);
    const page = entityPage(entity);
    const face = operator ? <OperatorFace entity={operator} server={server} /> : <EntityFace entity={entity} server={server} />;
    const props = {
        "aria-label": label,
        title: label,
        className: operator ? cn(OPERATOR_TILE_CLASS, TILE_SIZE.portrait) : cn(TILE_CLASS, TILE_SIZE[entityShape(entity)]),
        style: { "--tile-accent": entityAccent(entity) } as CSSProperties,
    };
    return (
        <li>
            {page !== null ? (
                <Link to={page} params={{ id: entity.id }} {...props}>
                    {face}
                </Link>
            ) : (
                // biome-ignore lint/a11y/noNoninteractiveTabindex: focusable so a keyboard reader can bring up the name, as a hover does
                <span role="img" tabIndex={0} {...props}>
                    {face}
                </span>
            )}
        </li>
    );
}

/** Any other kind's face: its own art, the accent line, and the name on hover. */
function EntityFace({ entity, server }: { entity: NonNullable<IShowcaseEntity["entity"]>; server?: string }) {
    return (
        <>
            <EntityAvatar entity={entity} face="tile" tone="dark" server={server} />
            <span aria-hidden="true" className="absolute inset-x-0 bottom-0 h-0.5 bg-(--tile-accent)" />
            <span
                aria-hidden="true"
                className="pointer-events-none absolute inset-x-0 bottom-0 bg-[linear-gradient(to_top,oklch(0_0_0/0.85),oklch(0_0_0/0.55)_60%,transparent)] px-1.5 pt-5 pb-2 text-center font-medium font-sans text-[11px] text-white leading-tight opacity-0 transition-opacity duration-150 group-hover:opacity-100 group-focus-visible:opacity-100"
            >
                <span className="line-clamp-2">{entity.name}</span>
            </span>
        </>
    );
}

/** The operators page's card in small: the faction mark faint behind the portrait, the name and class on a frosted bar, the rarity line under it. */
function OperatorFace({ entity, server }: { entity: ITierOperator; server?: string }) {
    const rarity = rarityToNumber(entity.rarity);
    return (
        <>
            {entity.nationId && <CampIcon groupId={entity.nationId} size={240} className="pointer-events-none absolute -top-3 -left-8 max-w-none opacity-5 transition-opacity group-hover:opacity-10" />}
            <PortraitArt entity={entity} server={server} />
            <span aria-hidden="true" className="absolute inset-x-0 bottom-0 z-10 flex h-9 items-end gap-1 bg-background/80 px-1.5 pb-1.5 backdrop-blur-sm">
                <span className="line-clamp-2 min-w-0 flex-1 font-bold text-[10px] uppercase leading-tight opacity-70 transition-opacity group-hover:opacity-100 sm:text-[11px]">{entity.name}</span>
                <ClassIcon profession={entity.profession} size={160} className="size-4 shrink-0" />
            </span>
            <span aria-hidden="true" className="absolute inset-x-0 bottom-0 z-10 h-0.5" style={{ backgroundColor: RARITY_COLORS[rarity] }} />
            <span aria-hidden="true" className="absolute inset-x-0 -bottom-0.5 z-10 h-1 blur-sm" style={{ backgroundColor: RARITY_BLUR_COLORS[rarity] }} />
        </>
    );
}

/** An operator's portrait (180 x 360), falling back to the square avatar where the extract has no portrait. */
function PortraitArt({ entity, server }: { entity: ITierOperator; server?: string }) {
    const [failed, setFailed] = useState(false);
    if (failed) return <EntityAvatar entity={entity} face="tile" tone="dark" server={server} />;
    return <img src={getPortraitById(entity.id, server)} alt="" aria-hidden="true" loading="lazy" decoding="async" draggable={false} onError={() => setFailed(true)} className="absolute inset-0 block h-full w-full object-contain transition-transform duration-150 group-hover:scale-105" />;
}

function GridBlock({ slug, isOwner }: { slug: string; isOwner: boolean }) {
    const t: TypedT<typeof messages> = useT("user");
    const server = useGamedataServer();
    const { user } = useAuth();
    const { data: grid, isPending } = useQuery(gridQueryOptions(slug, user?.id ?? null, server));
    const cells = useMemo(() => (grid ? gridToState(grid).cells : []), [grid]);
    if (isPending) return <Skeleton className="h-[44rem] w-full rounded-2xl lg:w-[30rem]" />;
    if (!grid) return isOwner ? <RemovedBlock type="grid" /> : null;
    const description = grid.description?.trim();
    // One surface: the card is the board's panel (the compact board draws no panel of its own), and at lg
    // the card is exactly the board's width plus its padding, so nothing sits empty beside it.
    const cardWidth = `calc(${compactGridSize(grid.rows, grid.cols).boardMaxPx}px + 2rem + 2px)`;
    return (
        <div className={cn(FILL_SLOT, "lg:w-(--card-w) lg:max-w-full")} style={{ "--card-w": cardWidth } as CSSProperties}>
            <StatCard color={BLOCK_ACCENT.grid}>
                <div className="flex flex-col gap-3 p-3 sm:p-4">
                    <BlockHeader icon={LayoutGridIcon} kicker={t("profile.showcase.block.grid")} title={grid.title} end={<OpenLink to="/grids/$slug" slug={slug} title={grid.title} />} />
                    {description && <p className="m-0 line-clamp-3 whitespace-pre-line text-muted-foreground text-sm">{description}</p>}
                    <GridBoard title={grid.title} rows={grid.rows} cols={grid.cols} cells={cells} size="compact" />
                </div>
            </StatCard>
        </div>
    );
}

function TierListBlock({ slug, isOwner }: { slug: string; isOwner: boolean }) {
    const t: TypedT<typeof messages> = useT("user");
    const server = useGamedataServer();
    const { data: detail, isPending } = useQuery(tierListDetailQueryOptions(slug, server));
    if (isPending) return <Skeleton className="h-72 rounded-2xl" />;
    if (!detail) return isOwner ? <RemovedBlock type="tier_list" /> : null;
    return (
        <StatCard color={BLOCK_ACCENT.tier_list}>
            <div className={CARD_PADDING}>
                <BlockHeader icon={ListOrderedIcon} kicker={t("profile.showcase.block.tierList")} title={detail.title} end={<OpenLink to="/tier-lists/$id" slug={slug} title={detail.title} />} />
            </div>
            <TierListPeek slug={slug}>
                <TierListBoard detail={detail} />
            </TierListPeek>
        </StatCard>
    );
}

/**
 * A tier list cut to its top tiers. At card width a real list runs 931-949 px
 * (6 tiers / 127 operators, 10 tiers / 61), which used to scroll inside a
 * 640 px card; a scroll box inside a scrolling page is the thing that read
 * wrong. Past the cap the board fades out over a link to the full list.
 */
function TierListPeek({ slug, children }: { slug: string; children: ReactNode }) {
    const t: TypedT<typeof messages> = useT("user");
    const ref = useRef<HTMLDivElement>(null);
    const [cut, setCut] = useState(false);
    useLayoutEffect(() => {
        const el = ref.current;
        if (!el) return;
        const check = () => setCut(el.scrollHeight > el.clientHeight + 1);
        check();
        const observer = new ResizeObserver(check);
        observer.observe(el);
        if (el.firstElementChild) observer.observe(el.firstElementChild);
        return () => observer.disconnect();
    }, []);
    return (
        <div className="relative px-3 pb-4 sm:px-5 sm:pb-5">
            <div ref={ref} className={cn(TIER_PEEK_MAX_HEIGHT, "overflow-hidden")}>
                {children}
            </div>
            {cut && (
                <div className="relative -mt-16 flex h-16 items-end justify-center bg-[linear-gradient(to_bottom,transparent,var(--card)_80%)]">
                    <Link to="/tier-lists/$id" params={{ id: slug }} className="inline-flex items-center gap-1 rounded-md border border-border bg-card px-3 py-1.5 font-medium text-foreground text-xs no-underline shadow-sm transition-colors hover:bg-accent">
                        {t("profile.showcase.block.tierList.viewAll")}
                        <ArrowUpRightIcon aria-hidden="true" className="size-3.5" />
                    </Link>
                </div>
            )}
        </div>
    );
}

function PlanBlock({ id, uid, roster, isOwner, plansHidden }: { id: string; uid: string; roster: IRosterEntry[]; isOwner: boolean; plansHidden: boolean }) {
    const t: TypedT<typeof messages> = useT("user");
    const { data: plans, isPending } = useQuery(publicPlansQueryOptions(uid));
    if (isPending) return <Skeleton className="h-40 rounded-2xl" />;
    const plan = plans?.find((p) => p.id === id);
    if (!plan) return isOwner ? <RemovedBlock type="plan" /> : null;
    // One card like every other block: the kicker inside, the plan's body drawn on the card itself.
    return (
        <StatCard color={BLOCK_ACCENT.plan}>
            <div className={cn(CARD_PADDING, "flex flex-col gap-3.5")}>
                <BlockHeader
                    icon={TargetIcon}
                    kicker={t("profile.showcase.block.plan")}
                    end={
                        isOwner &&
                        plansHidden && (
                            <span className="inline-flex shrink-0 items-center gap-1 rounded-full bg-muted px-2 py-0.5 font-medium text-[11px] text-muted-foreground leading-none">
                                <EyeOffIcon aria-hidden="true" className="size-3" />
                                {t("profile.showcase.block.plansHidden")}
                            </span>
                        )
                    }
                />
                <PlanCard p={plan} roster={roster} variant="embedded" />
            </div>
        </StatCard>
    );
}
