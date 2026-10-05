import { useState } from "react";
import { OperatorAvatar } from "#/components/ui/operator-avatar";
import { env } from "#/env";
import { type ArtFit, entityArtFit, entityIconURL, type ITierEntity, kindArtFit, type TierEntityKind } from "#/lib/api/tier-entities";
import { useGamedataServer } from "#/lib/i18n";
import { cn } from "#/lib/utils";
import { KIND_DEFINITIONS, kindDefinition, NEUTRAL_ACCENT } from "./kinds";

// The face of a tier list tile, shared by the board, the editor, the drag
// ghost and the hover previews. What differs per kind comes from
// `KIND_DEFINITIONS`. An operator renders through `OperatorAvatar`, untouched
// by kinds; a placement the served data does not know renders its raw id.

/** Width over height of a wide tile: an event's banner. */
export const WIDE_TILE_ASPECT = 2.1;

/** The tile's accent: the bottom bar and the hover border. Only operators and skins (rarity) and enemies (rank) carry a colour of their own. */
export function entityAccent(entity: ITierEntity): string {
    if (!entity.resolved) return NEUTRAL_ACCENT;
    return kindDefinition(entity.kind).accent?.(entity) ?? NEUTRAL_ACCENT;
}

/** A wide kind's tile spans two square slots of the same grid; every other kind is one square. */
export function entityShape(entity: ITierEntity): "square" | "wide" {
    return entity.resolved && KIND_DEFINITIONS[entity.kind].shape === "wide" ? "wide" : "square";
}

/** The attributes that mark a non-operator tile for the grid's kind and wide-shape rules. None on an operator, whose tile predates kinds, or on an unresolved placement. */
export function kindTileAttributes(entity: ITierEntity): { "data-kind"?: TierEntityKind; "data-shape"?: "square" | "wide" } {
    if (!entity.resolved || entity.kind === "operator") return {};
    return { "data-kind": entity.kind, "data-shape": entityShape(entity) };
}

/** The image classes of each art fit (see {@link entityArtFit}). */
export const FIT_CLASS: Record<ArtFit, string> = {
    cover: "object-cover",
    object: "object-contain p-[6%]",
    glyph: "object-contain p-[16%]",
};

/** Up to two letters for a tile with no art: the initials of the first two words, or the first two letters of a single word. */
export function entityInitials(name: string): string {
    const words = name
        .replace(/[^\p{L}\p{N}\s]/gu, " ")
        .split(/\s+/)
        .filter(Boolean);
    const [first, second] = words;
    if (!first) return name.trim().charAt(0).toUpperCase() || "?";
    if (!second) return Array.from(first).slice(0, 2).join("").toUpperCase();
    return `${Array.from(first)[0] ?? ""}${Array.from(second)[0] ?? ""}`.toUpperCase();
}

/** An entity's icon as a URL on this deployment, read from `server` when given, else the reader's game server. */
function useIconURL(icon: string | null, server?: string): string | null {
    const readerServer = useGamedataServer();
    if (!icon) return null;
    return entityIconURL(icon, env.VITE_BACKEND_URL ?? "", server ?? readerServer);
}

interface IEntityAvatarProps {
    entity: ITierEntity;
    /**
     * `chip` (default) fits any sized wrapper, falling back to initials.
     * `tile` is a board or pool tile: a wide kind without art spells its name across the tile instead.
     */
    face?: "chip" | "tile";
    /** `dark` when the wrapper is dark in both themes (the drag ghost), so a glyph is never inverted onto it. */
    tone?: "theme" | "dark";
    /** The server to read the art from when it is not the reader's, e.g. `cn` for an operator only CN has released. */
    server?: string;
}

/** The tile's face. Drop it inside a sized wrapper, like {@link OperatorAvatar}. */
export function EntityAvatar({ entity, face = "chip", tone = "theme", server }: IEntityAvatarProps) {
    if (entity.resolved && entity.kind === "operator") return <OperatorAvatar charId={entity.id} name={entity.name} server={server} />;
    if (!entity.resolved) return <span className="line-clamp-3 break-all px-1 text-center font-mono text-[9px] leading-tight opacity-80">{entity.id}</span>;
    return <EntityIcon kind={entity.kind} name={entity.name} icon={entity.icon} face={face} tone={tone} fit={entityArtFit(entity)} server={server} />;
}

interface IEntityIconProps {
    kind: Exclude<TierEntityKind, "operator">;
    name: string;
    /** The server-neutral API path the backend resolved, or `null` when the extract has no art. */
    icon: string | null;
    face?: "chip" | "tile";
    tone?: "theme" | "dark";
    /** How the art sits in the tile; the kind's own fit when absent. */
    fit?: ArtFit;
    /** The server to read the art from; the reader's when absent. */
    server?: string;
}

/** A non-operator's art from its resolved icon path, or a typeset stand-in when it has none or the image fails. Used where only the summary fields travel (card previews). */
export function EntityIcon({ kind, name, icon, face = "chip", tone = "theme", fit = kindArtFit(kind), server }: IEntityIconProps) {
    const src = useIconURL(icon, server);
    const [failed, setFailed] = useState(false);
    if (!src || failed) {
        if (face === "tile" && KIND_DEFINITIONS[kind].shape === "wide") {
            return <span className="line-clamp-3 px-2 text-center font-sans font-semibold text-[11px] leading-tight tracking-tight opacity-90">{name}</span>;
        }
        return (
            <span className="select-none font-bold font-sans tracking-tight opacity-85" aria-hidden="true">
                {entityInitials(name)}
            </span>
        );
    }
    return <img src={src} alt="" aria-hidden="true" loading="lazy" decoding="async" draggable={false} onDragStart={(e) => e.preventDefault()} onError={() => setFailed(true)} className={cn("block h-full w-full rounded-[inherit]", FIT_CLASS[fit], fit === "glyph" && tone === "theme" && "icon-theme-aware")} />;
}
