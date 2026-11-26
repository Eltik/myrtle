import { useRef } from "react";
import { useFittedOpCount } from "#/hooks/use-fitted-op-count";
import { CardEntityAvatar, cardEntityKey } from "./CardEntityAvatar";
import { readableTextColor } from "./detail/contrast";
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
    const pillText = readableTextColor(row.color);

    return (
        <div
            className={styles.tierRow}
            style={{
                ["--row-color" as string]: row.color,
                ["--tier-fg" as string]: pillText,
                ["--tier-shadow" as string]: pillText === "white" ? "0 1px 0 oklch(0 0 0 / 0.25)" : "0 1px 0 oklch(1 0 0 / 0.5)",
            }}
        >
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
