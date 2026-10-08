import { useOperatorFactLabel } from "#/components/operators/OperatorFacts";
import { type ITierEntity, isEntityOfKind, type TierEntityKind } from "#/lib/api/tier-entities";
import { type IKindFacet, type IKindPool, kindDefinition, useKindT } from "../kinds";

/** One filter row of the pool, over any entity: one of another kind never matches a selection. */
export interface IPoolFacet extends Omit<IKindFacet<TierEntityKind>, "valueOf"> {
    valueOf: (entity: ITierEntity) => string | null;
}

/** What the pool frame (`KindPool`) reads: the kind's pool definition, widened to take any entity. */
export interface IPoolKind extends Omit<IKindPool<TierEntityKind>, "facets" | "searchTexts" | "compare"> {
    facets: IPoolFacet[];
    searchTexts: (entity: ITierEntity) => (string | null)[];
    compare?: (a: ITierEntity, b: ITierEntity) => number;
}

/**
 * The pool configuration for `kind`, in the reader's language, from its
 * `KIND_DEFINITIONS` entry. `entities` is the kind's catalogue; an entity of
 * another kind, which a catalogue never holds, matches no filter and is
 * searched by its name alone.
 */
export function usePoolKind(kind: TierEntityKind, entities: readonly ITierEntity[] = []): IPoolKind {
    const t = useKindT();
    const fact = useOperatorFactLabel();
    const ofKind = (entity: ITierEntity) => isEntityOfKind(entity, kind);
    const pool = kindDefinition(kind).pool(t, entities.filter(ofKind), { race: fact.race });
    const { compare } = pool;
    return {
        ...pool,
        facets: pool.facets.map((facet) => ({ ...facet, valueOf: (entity) => (ofKind(entity) ? facet.valueOf(entity) : null) })),
        searchTexts: (entity) => (ofKind(entity) ? pool.searchTexts(entity) : [entity.name]),
        compare: compare && ((a, b) => (ofKind(a) && ofKind(b) ? compare(a, b) : 0)),
    };
}
