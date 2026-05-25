import { CampIcon, ClassIcon } from "#/components/operators/list/impl/components/Icons";
import type { ITierOperator } from "#/lib/api/tier-lists";
import { stripMarkdown } from "#/lib/markdown";
import { formatNationId, formatProfession, formatSubProfession, getAvatarById, parseOperatorName } from "#/lib/utils";
import { operatorPlacementNote } from "../shared";
import styles from "./TierOperatorPreview.module.css";

interface ITierOperatorPreviewProps {
    operator: ITierOperator;
}

export function TierOperatorPreview({ operator }: ITierOperatorPreviewProps) {
    const { displayName, subtitle } = parseOperatorName(operator.name);
    const note = operatorPlacementNote(operator);
    const archetype = formatSubProfession(operator.subProfessionId).replace(formatProfession(operator.profession), "").trim();
    const factionId = operator.nationId && operator.nationId.length > 0 ? operator.nationId : "rhodes";
    const positionLabel = operator.position === "RANGED" ? "Ranged" : operator.position === "MELEE" ? "Melee" : null;
    const initial = operator.name.charAt(0).toUpperCase();

    return (
        <article className={styles.preview} data-rarity={operator.rarity}>
            <div className={styles.head}>
                <div className={styles.avatar} aria-hidden="true">
                    {operator.id ? <img className={styles.avatarImg} src={getAvatarById(operator.id)} alt="" /> : <span className={styles.initial}>{initial}</span>}
                    <CampIcon groupId={factionId} size={48} className={styles.faction} />
                </div>
                <div className={styles.title}>
                    <div className={styles.name}>{displayName}</div>
                    {subtitle && <div className={styles.subtitle}>{subtitle}</div>}
                    <div className={styles.rarity} role="img" aria-label={`${operator.rarity}-star`}>
                        {Array.from({ length: operator.rarity }, (_, i) => (
                            // biome-ignore lint/suspicious/noArrayIndexKey: decorative stars
                            <span key={`s-${i}`} aria-hidden="true">
                                ★
                            </span>
                        ))}
                    </div>
                </div>
            </div>

            <div className={styles.row}>
                <ClassIcon profession={operator.profession} size={16} />
                <span className={styles.class}>{formatProfession(operator.profession)}</span>
                {archetype && (
                    <>
                        <span className={styles.dot} aria-hidden="true">
                            ·
                        </span>
                        <span className={styles.archetype}>{archetype}</span>
                    </>
                )}
            </div>

            {(operator.nationId || positionLabel) && (
                <div className={styles.meta}>
                    {operator.nationId && (
                        <span className={styles.metaItem}>
                            <span className={styles.k}>Nation</span>
                            <span className={styles.v}>{formatNationId(operator.nationId)}</span>
                        </span>
                    )}
                    {positionLabel && (
                        <span className={styles.metaItem}>
                            <span className={styles.k}>Position</span>
                            <span className={styles.v}>{positionLabel}</span>
                        </span>
                    )}
                </div>
            )}

            {note && (
                <p className={styles.notes} style={{ display: "-webkit-box", WebkitBoxOrient: "vertical", WebkitLineClamp: 5, overflow: "hidden" }}>
                    “{stripMarkdown(note)}”
                </p>
            )}

            <p className={styles.hint}>Click to view operator</p>
        </article>
    );
}
