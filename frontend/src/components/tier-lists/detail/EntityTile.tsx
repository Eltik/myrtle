import { Link } from "@tanstack/react-router";
import { HoverCard, HoverCardContent, HoverCardTrigger } from "#/components/ui/preview-card";
import { type ITierEntity, isOperatorEntity } from "#/lib/api/tier-entities";
import { stripMarkdown } from "#/lib/markdown";
import { EntityAvatar, entityAccent, entityShape, kindTileAttributes } from "../entities";
import { entityPage, useEntityLabels } from "../kinds";
import { operatorPlacementNote } from "../shared";
import { OperatorTile } from "./OperatorTile";
import styles from "./TierListDetail.module.css";
import previewStyles from "./TierOperatorPreview.module.css";

/**
 * One placement on the board. An operator keeps its own tile; every other kind
 * gets the same tile with its own art, a hover card naming it, and a link when
 * the site has a page for it. A placement the served data does not know shows
 * its raw id.
 */
export function EntityTile({ entity }: { entity: ITierEntity }) {
    const labels = useEntityLabels();
    if (isOperatorEntity(entity)) return <OperatorTile operator={entity} />;
    const accent = entityAccent(entity);
    if (!entity.resolved) {
        return (
            <span className={styles.opTile} style={{ ["--rarity-color" as string]: accent }} role="img" aria-label={entity.id} title={entity.id}>
                <EntityAvatar entity={entity} />
                <span className={styles.opRarity} aria-hidden="true" />
            </span>
        );
    }

    const tileProps = {
        className: styles.opTile,
        style: { ["--rarity-color" as string]: accent },
        ...kindTileAttributes(entity),
        "aria-label": labels.tileLabel(entity),
    };
    const face = (
        <>
            <EntityAvatar entity={entity} face="tile" />
            <span className={styles.opRarity} aria-hidden="true" />
        </>
    );
    const page = entityPage(entity);
    const trigger =
        page !== null ? (
            <Link to={page} params={{ id: entity.id }} {...tileProps}>
                {face}
            </Link>
        ) : (
            // biome-ignore lint/a11y/noNoninteractiveTabindex: focusable so a keyboard reader can open the hover card naming the tile
            <span role="img" tabIndex={0} {...tileProps}>
                {face}
            </span>
        );

    return (
        <HoverCard>
            <HoverCardTrigger render={trigger} />
            <HoverCardContent className="w-max max-w-[calc(100vw-2rem)] p-0" sideOffset={6}>
                <EntityPreview entity={entity} linked={page !== null} />
            </HoverCardContent>
        </HoverCard>
    );
}

/** The hover card of a non-operator tile: its art, its kind, its name, what kind of thing it is, and the author's note. */
export function EntityPreview({ entity, linked }: { entity: ITierEntity; linked: boolean }) {
    const labels = useEntityLabels();
    const detail = labels.detail(entity);
    const note = operatorPlacementNote(entity);
    const wide = entityShape(entity) === "wide";

    return (
        <article className={previewStyles.preview} style={{ ["--c" as string]: entityAccent(entity) }}>
            {wide && entity.icon ? (
                <div className="-mx-3.5 -mt-3.5 aspect-[2.1/1] overflow-hidden border-border border-b bg-muted" aria-hidden="true">
                    <EntityAvatar entity={entity} />
                </div>
            ) : null}
            <div className={previewStyles.head}>
                {wide && entity.icon ? null : (
                    <div className={previewStyles.avatar} aria-hidden="true">
                        <EntityAvatar entity={entity} />
                    </div>
                )}
                <div className={previewStyles.title}>
                    {entity.resolved && <div className="font-bold font-mono text-[9.5px] text-muted-foreground uppercase leading-none tracking-[0.14em]">{labels.singular(entity.kind)}</div>}
                    <div className={previewStyles.name}>{entity.name}</div>
                    {detail.length > 0 && <div className={previewStyles.subtitle}>{detail.join(" · ")}</div>}
                </div>
            </div>

            {note && (
                <p className={previewStyles.notes} style={{ display: "-webkit-box", WebkitBoxOrient: "vertical", WebkitLineClamp: 5, overflow: "hidden" }}>
                    “{stripMarkdown(note)}”
                </p>
            )}

            {linked && <p className={previewStyles.hint}>{labels.openPage}</p>}
        </article>
    );
}
