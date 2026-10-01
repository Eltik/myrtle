import { useRef } from "react";
import { useFittedOpCount } from "#/hooks/use-fitted-op-count";
import { CardEntityAvatar, cardEntityKey } from "./CardEntityAvatar";
import type { IThumbRow } from "./shared";

interface IThumbTierRowProps {
    row: IThumbRow;
    /** The card's CSS module; it must define `tierRow`, `tierPill`, `tierOps`, `op` and `opOverflow`. */
    styles: CSSModuleClasses;
    title: string;
}

/**
 * One tier row of a card thumbnail: the tier pill, as many operator tiles as
 * the row's measured width holds, and a "+N" badge for the rest.
 */
export function ThumbTierRow({ row, styles, title }: IThumbTierRowProps) {
    const opsRef = useRef<HTMLDivElement>(null);
    const visibleCount = useFittedOpCount(opsRef, row.operators.length, row.fallbackVisible);
    const visible = row.operators.slice(0, visibleCount);
    const overflow = row.operators.length - visible.length;

    return (
        <div className={styles.tierRow} style={{ ["--row-color" as string]: row.color }}>
            <span className={styles.tierPill} title={title}>
                {row.name}
            </span>
            <div ref={opsRef} className={styles.tierOps}>
                {visible.map((op) => (
                    <span key={cardEntityKey(op)} className={styles.op} title={op.name}>
                        <CardEntityAvatar op={op} />
                    </span>
                ))}
                {overflow > 0 && <span className={styles.opOverflow}>+{overflow}</span>}
            </div>
        </div>
    );
}
