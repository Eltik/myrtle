import type { messages as heroMessages } from "#/components/operators/detail/impl/components/OperatorHero.messages";
import { useOperatorFactLabel } from "#/components/operators/OperatorFacts";
import { useOperatorName } from "#/hooks/use-operator-name";
import { useFormatters, useT } from "#/lib/i18n";
import type { TypedT } from "#/lib/i18n/messages";
import { archetypeLabel, getAvatarById, nationLabel, professionLabel, rarityToNumber } from "#/lib/utils";
import type { OperatorChange } from "#/types/generated/OperatorChange";
import type { IOperatorIndexEntry } from "#/types/operators";
import type { IOperatorOwnershipInfo } from "../types";
import { useCnChangeSummary } from "./CnUpdateBadge";
import type { messages as cnMessages } from "./CnUpdateBadge.messages";
import { CampIcon, ClassIcon } from "./Icons";
import type { messages as cardListMessages } from "./OperatorCardList.messages";
import type { messages } from "./OperatorPreview.messages";
import styles from "./OperatorPreview.module.css";
import type { messages as ownershipMessages } from "./OwnershipBadge.messages";

type PreviewT = TypedT<typeof messages & typeof cardListMessages & typeof heroMessages & typeof ownershipMessages & typeof cnMessages>;

interface IOperatorPreviewProps {
    operator: IOperatorIndexEntry;
    /** The game-data server the record comes from, for its avatar; omitted for this site's own. */
    server?: string;
    /** The community counts the card's pill abbreviates, stated here in full: a native title is
     *  unreachable on touch and by keyboard. */
    ownership?: IOperatorOwnershipInfo | null;
    /** What CN changed, which the card's Updated pill otherwise only carries in its title. */
    cnChanges?: readonly OperatorChange[] | null;
}

export function OperatorPreview({ operator, server, ownership, cnChanges }: IOperatorPreviewProps) {
    const t: PreviewT = useT("operators");
    const f = useFormatters();
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
                    <div className={styles.rarity} role="img" aria-label={t("card.rarityStars", { rarity })}>
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
                        <span className={styles.v}>{operator.position === "RANGED" ? t("hero.position.ranged") : operator.position === "MELEE" ? t("hero.position.melee") : operator.position}</span>
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

            {(ownership || (cnChanges && cnChanges.length > 0)) && (
                <div className={styles.facts}>
                    {ownership && ownership.pct != null && <p>{t("ownership.owned.title", { owners: f.number(ownership.owners), exact: (ownership.pct * 100).toFixed(2) })}</p>}
                    {ownership && ownership.e2Pct != null && <p>{t("ownership.e2.title", { e2Owners: f.number(ownership.e2Owners), owners: f.number(ownership.owners), exact: (ownership.e2Pct * 100).toFixed(2) })}</p>}
                    {cnChanges && cnChanges.length > 0 && <CnChangeLine changes={cnChanges} />}
                </div>
            )}
        </div>
    );
}

function CnChangeLine({ changes }: { changes: readonly OperatorChange[] }) {
    const t: PreviewT = useT("operators");
    const summary = useCnChangeSummary(changes);
    return <p>{t("cnUpdates.badge.title", { changes: summary })}</p>;
}
