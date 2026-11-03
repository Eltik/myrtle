import type * as React from "react";
import { FilterChip } from "#/components/ui/filter-chip";
import { Kicker } from "#/components/ui/kicker";
import { useT } from "#/lib/i18n";
import type { TypedT } from "#/lib/i18n/messages";
import { rarityInk } from "#/lib/utils";
import { SENIOR_OPERATOR_TAG_ID, TAG_GROUP_LABEL_KEYS, TAG_GROUP_ORDER, TOP_OPERATOR_TAG_ID } from "../constants";
import type { messages as recruitConstantsMessages } from "../constants.messages";
import type { IRecruitmentTag, TagType } from "../types";

interface ITagSelectorProps {
    groups: Record<TagType, IRecruitmentTag[]>;
    selectedTagIds: ReadonlySet<number>;
    onToggle: (id: number) => void;
    maxReached: boolean;
}

/** The two qualification tags that guarantee a rarity carry that rarity's tone; every other tag is untoned. */
const TAG_RARITY: Partial<Record<number, number>> = { [TOP_OPERATOR_TAG_ID]: 6, [SENIOR_OPERATOR_TAG_ID]: 5 };

export function TagSelector({ groups, selectedTagIds, onToggle, maxReached }: ITagSelectorProps): React.ReactElement {
    const t: TypedT<typeof recruitConstantsMessages> = useT("tools");
    return (
        <div className="flex flex-col gap-4">
            {TAG_GROUP_ORDER.map((type) => {
                const tagsForGroup = groups[type];
                if (tagsForGroup.length === 0) return null;
                return (
                    <section key={type}>
                        <h3 className="mb-2">
                            <Kicker>{t(TAG_GROUP_LABEL_KEYS[type])}</Kicker>
                        </h3>
                        <div className="flex flex-wrap gap-1.5">
                            {tagsForGroup.map((tag) => {
                                const selected = selectedTagIds.has(tag.id);
                                const rarity = TAG_RARITY[tag.id];
                                return <FilterChip key={tag.id} active={selected} className="shadow-none before:hidden" disabled={!selected && maxReached} label={tag.name} onSelect={() => onToggle(tag.id)} tone={rarity ? `var(--rarity-${rarity})` : undefined} toneInk={rarity ? rarityInk(rarity) : undefined} />;
                            })}
                        </div>
                    </section>
                );
            })}
        </div>
    );
}
