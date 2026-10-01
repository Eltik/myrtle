import type { IOperator } from "#/components/home/impl/data";
import { OperatorAvatar } from "#/components/ui/operator-avatar";
import { EntityIcon } from "./entities";

/** One card-preview tile's face: an operator's avatar exactly as before, any other kind's own icon. Drop it inside a sized wrapper. */
export function CardEntityAvatar({ op }: { op: IOperator }) {
    if (op.kind) return <EntityIcon kind={op.kind} name={op.name} icon={op.icon ?? null} />;
    return <OperatorAvatar charId={op.id} name={op.name} />;
}

/** A card preview tile's React key: ids of two kinds may coincide. */
export function cardEntityKey(op: IOperator): string {
    return `${op.kind ?? "operator"}:${op.id}`;
}
