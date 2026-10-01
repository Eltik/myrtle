import type { ReactNode } from "react";
import { ClassIcon } from "#/components/operators/list/impl/components/Icons";
import type { ITierEntity, TierEntityKind } from "#/lib/api/tier-entities";
import { useT } from "#/lib/i18n";
import type { TypedT } from "#/lib/i18n/messages";
import { formatProfession } from "#/lib/utils";
import type { OperatorProfession } from "#/types/operators";
import { useEntityLabels } from "../entities";
import type { messages } from "./KindPool.messages";

// What each kind's pool offers: its labels, the filters over its facets, what
// the search box matches and the order tiles come in. The pool frame
// (`KindPool`) is the same for every kind; a new kind is one arm here.

export interface IPoolFacetOption {
    value: string;
    /** Visible text, or the tooltip and accessible name when `icon` is set. */
    label: string;
    /** Accessible name when it should differ from `label` (`6 star` for `6★`). */
    ariaLabel?: string;
    icon?: ReactNode;
}

export interface IPoolFacet {
    id: string;
    /** The filter row's label, rendered uppercase. */
    label: string;
    /** Accessible name of the row of toggles. */
    groupLabel: string;
    /** `mono` for short tabular values (stars), `text` for words, `icon` for glyph buttons with tooltips. */
    variant: "mono" | "text" | "icon";
    options: IPoolFacetOption[];
    /** The entity's value for this facet; an entity with none never matches a selection. */
    valueOf: (entity: ITierEntity) => string | null;
}

export interface IPoolKind {
    kicker: string;
    dialogTitle: string;
    searchLabel: string;
    gridLabel: string;
    dropArea: string;
    facets: IPoolFacet[];
    /** Strings the search box matches, already in display form. */
    searchTexts: (entity: ITierEntity) => (string | null)[];
    /** Pool order. Absent: the catalogue's own order, which the backend sets per kind. */
    compare?: (a: ITierEntity, b: ITierEntity) => number;
}

const CLASS_OPTIONS: OperatorProfession[] = ["PIONEER", "WARRIOR", "TANK", "SNIPER", "CASTER", "MEDIC", "SUPPORT", "SPECIAL"];
const RARITY_OPTIONS = [6, 5, 4, 3, 2, 1] as const;

/** The pool configuration for `kind`, in the reader's language. */
export function usePoolKind(kind: TierEntityKind): IPoolKind {
    const t: TypedT<typeof messages> = useT("tierLists");
    const labels = useEntityLabels();

    const classFacet = (read: (e: ITierEntity) => string | null): IPoolFacet => ({
        id: "class",
        label: t("edit.pool.class"),
        groupLabel: t("edit.pool.class.group"),
        variant: "icon",
        options: CLASS_OPTIONS.map((c) => ({ value: c, label: formatProfession(c), icon: <ClassIcon profession={c} size={16} /> })),
        valueOf: read,
    });
    const nameOnly = (e: ITierEntity) => [e.name];
    const anyOfKind = { dropArea: t("edit.pool.dropArea.any") };

    switch (kind) {
        case "operator":
            return {
                kicker: t("edit.pool.kicker"),
                dialogTitle: t("edit.pool.dialogTitle"),
                searchLabel: t("edit.pool.search.label"),
                gridLabel: t("edit.pool.gridLabel"),
                dropArea: t("edit.pool.dropArea"),
                facets: [
                    {
                        id: "rarity",
                        label: t("edit.pool.rarity"),
                        groupLabel: t("edit.pool.rarity.group"),
                        variant: "mono",
                        options: RARITY_OPTIONS.map((r) => ({ value: String(r), label: `${r}★`, ariaLabel: t("edit.pool.rarity.option", { rarity: r }) })),
                        valueOf: (e) => (e.resolved && e.kind === "operator" ? String(e.rarity) : null),
                    },
                    classFacet((e) => (e.resolved && e.kind === "operator" ? e.profession : null)),
                ],
                searchTexts: (e) => [e.name, e.resolved && e.kind === "operator" ? e.appellation : null],
                compare: (a, b) => {
                    const ra = a.resolved && a.kind === "operator" ? a.rarity : 0;
                    const rb = b.resolved && b.kind === "operator" ? b.rarity : 0;
                    if (ra !== rb) return rb - ra;
                    return a.name.localeCompare(b.name);
                },
            };
        case "class":
            return { kicker: t("edit.pool.kicker.class"), dialogTitle: t("edit.pool.kicker.class"), searchLabel: t("edit.pool.search.class"), gridLabel: t("edit.pool.grid.class"), ...anyOfKind, facets: [], searchTexts: nameOnly };
        case "subclass":
            return {
                kicker: t("edit.pool.kicker.subclass"),
                dialogTitle: t("edit.pool.kicker.subclass"),
                searchLabel: t("edit.pool.search.subclass"),
                gridLabel: t("edit.pool.grid.subclass"),
                ...anyOfKind,
                facets: [classFacet((e) => (e.resolved && e.kind === "subclass" ? e.profession : null))],
                searchTexts: nameOnly,
            };
        case "faction":
            return {
                kicker: t("edit.pool.kicker.faction"),
                dialogTitle: t("edit.pool.kicker.faction"),
                searchLabel: t("edit.pool.search.faction"),
                gridLabel: t("edit.pool.grid.faction"),
                ...anyOfKind,
                facets: [
                    {
                        id: "level",
                        label: t("edit.pool.level"),
                        groupLabel: t("edit.pool.level.group"),
                        variant: "text",
                        options: (["nation", "group", "team"] as const).map((v) => ({ value: v, label: labels.factionLevel[v] })),
                        valueOf: (e) => (e.resolved && e.kind === "faction" ? e.powerLevel : null),
                    },
                ],
                searchTexts: nameOnly,
            };
        case "enemy":
            return {
                kicker: t("edit.pool.kicker.enemy"),
                dialogTitle: t("edit.pool.kicker.enemy"),
                searchLabel: t("edit.pool.search.enemy"),
                gridLabel: t("edit.pool.grid.enemy"),
                ...anyOfKind,
                facets: [
                    {
                        id: "rank",
                        label: t("edit.pool.rank"),
                        groupLabel: t("edit.pool.rank.group"),
                        variant: "text",
                        options: (["NORMAL", "ELITE", "BOSS"] as const).map((v) => ({ value: v, label: labels.enemyLevel[v] })),
                        valueOf: (e) => (e.resolved && e.kind === "enemy" ? e.level : null),
                    },
                ],
                searchTexts: (e) => [e.name, e.resolved && e.kind === "enemy" ? e.index : null],
            };
        case "event":
            return {
                kicker: t("edit.pool.kicker.event"),
                dialogTitle: t("edit.pool.kicker.event"),
                searchLabel: t("edit.pool.search.event"),
                gridLabel: t("edit.pool.grid.event"),
                ...anyOfKind,
                facets: [
                    {
                        id: "type",
                        label: t("edit.pool.type"),
                        groupLabel: t("edit.pool.type.group"),
                        variant: "text",
                        options: ["SIDESTORY", "MINISTORY", "BRANCHLINE", "NONE"].map((v) => ({ value: v, label: labels.eventType(v) })),
                        valueOf: (e) => (e.resolved && e.kind === "event" ? e.displayType : null),
                    },
                    {
                        id: "edition",
                        label: t("edit.pool.edition"),
                        groupLabel: t("edit.pool.edition.group"),
                        variant: "text",
                        options: [
                            { value: "original", label: t("edit.pool.edition.original") },
                            { value: "rerun", label: labels.rerun },
                        ],
                        valueOf: (e) => (e.resolved && e.kind === "event" ? (e.rerun ? "rerun" : "original") : null),
                    },
                ],
                searchTexts: nameOnly,
            };
        case "stronghold_bond":
            return {
                kicker: t("edit.pool.kicker.stronghold_bond"),
                dialogTitle: t("edit.pool.kicker.stronghold_bond"),
                searchLabel: t("edit.pool.search.stronghold_bond"),
                gridLabel: t("edit.pool.grid.stronghold_bond"),
                ...anyOfKind,
                facets: [
                    {
                        id: "type",
                        label: t("edit.pool.type"),
                        groupLabel: t("edit.pool.type.group"),
                        variant: "text",
                        options: (["season", "regular"] as const).map((v) => ({ value: v, label: labels.bondType[v] })),
                        valueOf: (e) => (e.resolved && e.kind === "stronghold_bond" ? e.bondType : null),
                    },
                ],
                searchTexts: nameOnly,
            };
    }
}
