import { useState } from "react";
import { OperatorAvatar } from "#/components/ui/operator-avatar";
import { env } from "#/env";
import { DEFAULT_GAMEDATA_SERVER } from "#/lib/api/gamedata";
import { entityIconURL, type ITierEntity, type TierEntityKind } from "#/lib/api/tier-entities";
import { useGamedataServer, useT } from "#/lib/i18n";
import type { TypedT } from "#/lib/i18n/messages";
import { cn, formatProfession, RARITY_HEX_MUTED } from "#/lib/utils";
import type { messages } from "./entities.messages";

// Per-kind look of a tier list tile. Every tile (board, editor, drag ghost,
// hover previews) asks here instead of reading operator fields, so a new kind
// is one arm in each function. An operator renders exactly as it did before
// kinds existed; a placement the served data does not know renders its raw id.

const NEUTRAL_ACCENT = RARITY_HEX_MUTED[1] as string;

/** The tile's accent: the bottom bar and the hover border. Only operators (rarity) and enemies (rank) carry a colour of their own. */
export function entityAccent(entity: ITierEntity): string {
    if (!entity.resolved) return NEUTRAL_ACCENT;
    switch (entity.kind) {
        case "operator":
            return RARITY_HEX_MUTED[entity.rarity] ?? NEUTRAL_ACCENT;
        case "enemy":
            if (entity.level === "BOSS") return "#dc4d56";
            if (entity.level === "ELITE") return RARITY_HEX_MUTED[5] ?? NEUTRAL_ACCENT;
            return NEUTRAL_ACCENT;
        default:
            return NEUTRAL_ACCENT;
    }
}

/** An event's art is a banner, so its tile spans two square slots of the same grid; every other kind is one square. */
export function entityShape(entity: ITierEntity): "square" | "wide" {
    return entity.resolved && entity.kind === "event" ? "wide" : "square";
}

/** Kinds whose art is a white glyph on transparency: drawn inset and inverted for the light theme. The rest fill the tile. */
const GLYPH_KINDS: ReadonlySet<TierEntityKind> = new Set(["class", "subclass", "faction", "stronghold_bond"]);

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

/** An entity's icon as a URL on this deployment, read from the reader's game server. */
function useIconURL(icon: string | null): string | null {
    const server = useGamedataServer();
    if (!icon) return null;
    return entityIconURL(icon, env.VITE_BACKEND_URL ?? "", server === DEFAULT_GAMEDATA_SERVER ? undefined : server);
}

interface IEntityAvatarProps {
    entity: ITierEntity;
    /**
     * `chip` (default) fits any sized wrapper, falling back to initials.
     * `tile` is a board or pool tile: an event without art spells its name across the wide tile instead.
     */
    face?: "chip" | "tile";
    /** `dark` when the wrapper is dark in both themes (the drag ghost), so a glyph is never inverted onto it. */
    tone?: "theme" | "dark";
}

/** The tile's face. Drop it inside a sized wrapper, like {@link OperatorAvatar}. */
export function EntityAvatar({ entity, face = "chip", tone = "theme" }: IEntityAvatarProps) {
    if (entity.resolved && entity.kind === "operator") return <OperatorAvatar charId={entity.id} name={entity.name} />;
    if (!entity.resolved) return <span className="line-clamp-3 break-all px-1 text-center font-mono text-[9px] leading-tight opacity-80">{entity.id}</span>;
    return <EntityIcon kind={entity.kind} name={entity.name} icon={entity.icon} face={face} tone={tone} />;
}

interface IEntityIconProps {
    kind: Exclude<TierEntityKind, "operator">;
    name: string;
    /** The server-neutral API path the backend resolved, or `null` when the extract has no art. */
    icon: string | null;
    face?: "chip" | "tile";
    tone?: "theme" | "dark";
}

/** A non-operator's art from its resolved icon path, or a typeset stand-in when it has none or the image fails. Used where only the summary fields travel (card previews). */
export function EntityIcon({ kind, name, icon, face = "chip", tone = "theme" }: IEntityIconProps) {
    const src = useIconURL(icon);
    const [failed, setFailed] = useState(false);
    if (!src || failed) {
        if (face === "tile" && kind === "event") {
            return <span className="line-clamp-3 px-2 text-center font-sans font-semibold text-[11px] leading-tight tracking-tight opacity-90">{name}</span>;
        }
        return (
            <span className="select-none font-bold font-sans tracking-tight opacity-85" aria-hidden="true">
                {entityInitials(name)}
            </span>
        );
    }
    const glyph = GLYPH_KINDS.has(kind);
    return (
        <img
            src={src}
            alt=""
            aria-hidden="true"
            loading="lazy"
            decoding="async"
            draggable={false}
            onDragStart={(e) => e.preventDefault()}
            onError={() => setFailed(true)}
            className={cn("block h-full w-full rounded-[inherit]", glyph ? cn("object-contain p-[16%]", tone === "theme" && "icon-theme-aware") : "object-cover")}
        />
    );
}

/** The translated labels every kind's tile, hover card, pool and settings read. */
export function useEntityLabels() {
    const t: TypedT<typeof messages> = useT("tierLists");
    const kinds: Record<TierEntityKind, string> = {
        operator: t("entity.kinds.operator"),
        class: t("entity.kinds.class"),
        subclass: t("entity.kinds.subclass"),
        faction: t("entity.kinds.faction"),
        enemy: t("entity.kinds.enemy"),
        event: t("entity.kinds.event"),
        stronghold_bond: t("entity.kinds.stronghold_bond"),
    };
    const kind: Record<TierEntityKind, string> = {
        operator: t("entity.kind.operator"),
        class: t("entity.kind.class"),
        subclass: t("entity.kind.subclass"),
        faction: t("entity.kind.faction"),
        enemy: t("entity.kind.enemy"),
        event: t("entity.kind.event"),
        stronghold_bond: t("entity.kind.stronghold_bond"),
    };
    const enemyLevel = { NORMAL: t("entity.enemyLevel.normal"), ELITE: t("entity.enemyLevel.elite"), BOSS: t("entity.enemyLevel.boss") };
    const factionLevel = { nation: t("entity.factionLevel.nation"), group: t("entity.factionLevel.group"), team: t("entity.factionLevel.team") };
    const bondType = { season: t("entity.bondType.season"), regular: t("entity.bondType.regular") };
    const eventTypes: Record<string, string> = {
        SIDESTORY: t("entity.eventType.SIDESTORY"),
        MINISTORY: t("entity.eventType.MINISTORY"),
        BRANCHLINE: t("entity.eventType.BRANCHLINE"),
        NONE: t("entity.eventType.NONE"),
    };
    const eventType = (displayType: string) => eventTypes[displayType] ?? eventTypes.NONE ?? displayType;
    const rerun = t("entity.event.rerun");

    /** One line of what kind of thing this is, e.g. `Elite · B1` or `Side Story · Rerun`. Empty for an unresolved placement. */
    const detail = (entity: ITierEntity): string[] => {
        if (!entity.resolved) return [];
        switch (entity.kind) {
            case "operator":
                return [formatProfession(entity.profession)];
            case "class":
                return [];
            case "subclass":
                return [formatProfession(entity.profession)];
            case "enemy":
                return [enemyLevel[entity.level], ...(entity.index ? [entity.index] : [])];
            case "event":
                return [eventType(entity.displayType), ...(entity.rerun ? [rerun] : [])];
            case "faction":
                return [factionLevel[entity.powerLevel]];
            case "stronghold_bond":
                return [bondType[entity.bondType]];
        }
    };

    const tileLabel = (entity: ITierEntity) => (entity.resolved ? t("entity.tile.label", { name: entity.name, kind: kind[entity.kind] }) : entity.id);

    return { kinds, kind, enemyLevel, factionLevel, bondType, eventType, rerun, detail, tileLabel, openPage: t("entity.preview.open") };
}

export type EntityLabels = ReturnType<typeof useEntityLabels>;
