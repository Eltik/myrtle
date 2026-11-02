import { useOperatorFactLabel } from "#/components/operators/OperatorFacts";
import { useOperatorName } from "#/hooks/use-operator-name";
import { useT } from "#/lib/i18n";
import type { TypedT } from "#/lib/i18n/messages";
import { archetypeLabel, getAvatarById, nationLabel, professionLabel, rarityToNumber } from "#/lib/utils";
import type { IOperatorIndexEntry } from "#/types/operators";
import { CampIcon, ClassIcon } from "./Icons";
import type { messages } from "./OperatorPreview.messages";
import styles from "./OperatorPreview.module.css";

interface IOperatorPreviewProps {
    operator: IOperatorIndexEntry;
    /** The game-data server the record comes from, for its avatar; omitted for this site's own. */
    server?: string;
}

export function OperatorPreview({ operator, server }: IOperatorPreviewProps) {
    const t: TypedT<typeof messages> = useT("operators");
    const fact = useOperatorFactLabel();
    const name = useOperatorName()(operator);
    const rarity = rarityToNumber(operator.rarity);
    const initial = name.charAt(0).toUpperCase();
    const nation = nationLabel(operator);
    const archetype = archetypeLabel(operator);
    const logoId = operator.nationId && operator.nationId.length > 0 ? operator.nationId : operator.teamId && operator.teamId.length > 0 ? operator.teamId : operator.groupId && operator.groupId.length > 0 ? operator.groupId : "rhodes";
    const gender = operator.gender;
    const race = operator.race;

    return (
        <div className={styles.opPreview} data-rarity={rarity}>
            <div className={styles.head}>
                <div className={styles.avatar} aria-hidden="true">
                    {operator.id ? <img className={styles.avatarImg} src={getAvatarById(operator.id, server)} alt="" /> : <span className={styles.initial}>{initial}</span>}
                    <CampIcon groupId={logoId} size={60} className={styles.faction} />
                </div>
                <div className={styles.title}>
                    <div className={styles.name}>{name}</div>
                    <div className={styles.rarity}>
                        {Array.from({ length: rarity }, (_, i) => (
                            // biome-ignore lint/suspicious/noArrayIndexKey: decorative stars with fixed count
                            <span key={`s${operator.id}-${i}`} aria-hidden="true">
                                ★
                            </span>
                        ))}
                    </div>
                    {nation && <div className={styles.nation}>{nation}</div>}
                </div>
            </div>

            <div className={styles.row}>
                <ClassIcon profession={operator.profession} size={16} />
                <span className={styles.class}>{professionLabel(operator)}</span>
                <span className={styles.dot} aria-hidden="true">
                    ·
                </span>
                <span className={styles.archetype}>{archetype}</span>
            </div>

            {operator.position && (
                <div className={styles.meta}>
                    <span className={styles.metaItem}>
                        <span className={styles.k}>{t("preview.position")}</span>
                        <span className={styles.v}>{operator.position.charAt(0) + operator.position.slice(1).toLowerCase()}</span>
                    </span>
                    {race && (
                        <span className={styles.metaItem}>
                            <span className={styles.k}>{t("preview.race")}</span>
                            <span className={styles.v}>{fact.race(race)}</span>
                        </span>
                    )}
                    {gender && (
                        <span className={styles.metaItem}>
                            <span className={styles.k}>{t("preview.gender")}</span>
                            <span className={styles.v}>{fact.gender(gender)}</span>
                        </span>
                    )}
                </div>
            )}
        </div>
    );
}
