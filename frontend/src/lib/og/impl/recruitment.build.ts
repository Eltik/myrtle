/**
 * The recruitment card's data, built from the calculator. Run in two places
 * from the same `/operators/recruitment` document: by the tool route's
 * `head()` to hash the og:image URL, and by the `recruitment` OG handler to
 * draw it, so the hash and the picture read the same best combination.
 */

import { calculateResults, guaranteedFloorRarity } from "#/components/tools/recruitment/impl/calculator";
import { HIDDEN_TAG_IDS } from "#/components/tools/recruitment/impl/constants";
import { DEFAULT_SETTINGS } from "#/components/tools/recruitment/impl/useRecruitOptions";
import type { GamedataServer } from "#/lib/api/gamedata";
import type { IRecruitmentData } from "#/lib/api/recruitment";
import { type IRecruitmentOgData, type IRecruitmentOgTag, normalizeRecruitmentTagIds, RECRUITMENT_OG_TILES } from "./recruitment";

/**
 * The card for `tagIds` against `data`, or `null` when none of the ids is a
 * tag the server offers (an unknown id, or one the tool hides, is dropped).
 * The best combination is the calculator's first result under its default
 * settings: its ranking already puts the highest guaranteed floor first and
 * breaks ties its own way.
 */
export function buildRecruitmentOgData(tagIds: readonly number[], data: IRecruitmentData, server: GamedataServer): IRecruitmentOgData | null {
    const names = new Map(data.tags.filter((t) => !HIDDEN_TAG_IDS.has(t.tagId)).map((t) => [t.tagId, t.tagName] as const));
    const tags: IRecruitmentOgTag[] = normalizeRecruitmentTagIds(tagIds).flatMap((id) => {
        const name = names.get(id);
        return name ? [{ id, name }] : [];
    });
    if (tags.length === 0) return null;

    const top = calculateResults(tags, data.operators, DEFAULT_SETTINGS)[0];
    return {
        server,
        tags,
        best: top
            ? {
                  tagIds: top.tags,
                  floor: guaranteedFloorRarity(top),
                  operators: top.operators.slice(0, RECRUITMENT_OG_TILES).map((op) => ({ id: op.id, name: op.name, rarity: op.rarity })),
                  total: top.operators.length,
              }
            : null,
    };
}
