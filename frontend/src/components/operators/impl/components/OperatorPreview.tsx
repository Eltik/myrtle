import { formatNationId, formatProfession, formatSubProfession, getAvatarById, rarityToNumber } from "#/lib/utils";
import type { IOperatorListItem } from "#/types/operators";
import { CampIcon, ClassIcon } from "./Icons";
import styles from "./OperatorPreview.module.css";

interface IOperatorPreviewProps {
    operator: IOperatorListItem;
}

export function OperatorPreview({ operator }: IOperatorPreviewProps) {
    const rarity = rarityToNumber(operator.rarity);
    const initial = operator.name.charAt(0).toUpperCase();
    const nationLabel = operator.nationId ? formatNationId(operator.nationId) : null;
    const archetype = formatSubProfession(operator.subProfessionId).replace(formatProfession(operator.profession), "");
    const logoId = operator.nationId && operator.nationId.length > 0 ? operator.nationId : operator.teamId && operator.teamId.length > 0 ? operator.teamId : operator.groupId && operator.groupId.length > 0 ? operator.groupId : "rhodes";
    const gender = operator.profile?.basicInfo.gender;
    const race = operator.profile?.basicInfo.race;

    return (
        <div className={styles.opPreview} data-rarity={rarity}>
            <div className={styles.head}>
                <div className={styles.avatar} aria-hidden="true">
                    {operator.id ? <img className={styles.avatarImg} src={getAvatarById(operator.id)} alt="" /> : <span className={styles.initial}>{initial}</span>}
                    <CampIcon groupId={logoId} size={60} className={styles.faction} />
                </div>
                <div className={styles.title}>
                    <div className={styles.name}>{operator.name}</div>
                    <div className={styles.rarity}>
                        {Array.from({ length: rarity }, (_, i) => (
                            // biome-ignore lint/suspicious/noArrayIndexKey: decorative stars with fixed count
                            <span key={`s${operator.id}-${i}`} aria-hidden="true">
                                ★
                            </span>
                        ))}
                    </div>
                    {nationLabel && <div className={styles.nation}>{nationLabel}</div>}
                </div>
            </div>

            <div className={styles.row}>
                <ClassIcon profession={operator.profession} size={16} />
                <span className={styles.class}>{formatProfession(operator.profession)}</span>
                <span className={styles.dot} aria-hidden="true">
                    ·
                </span>
                <span className={styles.archetype}>{archetype}</span>
            </div>

            {operator.position && (
                <div className={styles.meta}>
                    <span className={styles.metaItem}>
                        <span className={styles.k}>Position</span>
                        <span className={styles.v}>{operator.position.charAt(0) + operator.position.slice(1).toLowerCase()}</span>
                    </span>
                    {race && (
                        <span className={styles.metaItem}>
                            <span className={styles.k}>Race</span>
                            <span className={styles.v}>{race}</span>
                        </span>
                    )}
                    {gender && (
                        <span className={styles.metaItem}>
                            <span className={styles.k}>Gender</span>
                            <span className={styles.v}>{gender}</span>
                        </span>
                    )}
                </div>
            )}
        </div>
    );
}
