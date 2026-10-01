import type { IOperator } from "#/components/home/impl/data";
import { OperatorAvatar } from "#/components/ui/operator-avatar";
import { entityKey } from "#/lib/api/tier-entities";
import { EntityIcon } from "./entities";

/** One card-preview tile's face: an operator's avatar, any other kind's own icon. Drop it inside a sized wrapper. */
export function CardEntityAvatar({ op }: { op: IOperator }) {
    if (op.kind) return <EntityIcon kind={op.kind} name={op.name} icon={op.icon ?? null} fit={op.fit} />;
    return <OperatorAvatar charId={op.id} name={op.name} />;
}

/** A card preview tile's React key: ids of two kinds may coincide. */
export function cardEntityKey(op: IOperator): string {
    return entityKey(op.kind ?? "operator", op.id);
}
