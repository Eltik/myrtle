import type { ITierListDetail } from "#/lib/api/tier-lists";
import styles from "./TierListDetail.module.css";
import { TierRow } from "./TierRow";

interface ITierListBoardProps {
    detail: ITierListDetail;
}

export function TierListBoard({ detail }: ITierListBoardProps) {
    if (detail.tiers.length === 0) {
        return (
            <div className="rounded-2xl border border-border border-dashed bg-muted/20 px-5 py-12 text-center">
                <p className="m-0 font-medium font-sans text-foreground text-sm">This list has no tiers yet.</p>
                <p className="mt-1 font-sans text-[12.5px] text-muted-foreground">The author hasn't published any tiers for this list.</p>
            </div>
        );
    }

    const totalOps = detail.tiers.reduce((acc, t) => acc + t.operators.length, 0);

    return (
        <div className="space-y-3">
            <ul className={styles.tierBoard} aria-label={`Tier list ${detail.title}`}>
                {detail.tiers.map((tier, i) => (
                    <TierRow key={tier.id} tier={tier} index={i} />
                ))}
            </ul>
            <p className="px-1 font-mono text-[10.5px] text-muted-foreground uppercase tracking-wider">
                <span className="text-foreground tabular-nums">{detail.tiers.length}</span> tier{detail.tiers.length === 1 ? "" : "s"} · <span className="text-foreground tabular-nums">{totalOps}</span> operator{totalOps === 1 ? "" : "s"} placed
            </p>
        </div>
    );
}
