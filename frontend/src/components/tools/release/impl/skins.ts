import type { AutoName } from "#/types/generated/AutoName";
import type { EventAnchor } from "#/types/generated/EventAnchor";
import type { LagModel } from "#/types/generated/LagModel";
import type { NewSkin } from "#/types/generated/NewSkin";
import type { Resolution } from "#/types/generated/Resolution";
import { sortKey } from "./helpers";

export interface INewSkinGroup {
    skinGroupId: string;
    skinGroupName: string;
    skinGroupNameAuto: AutoName | null;
    skins: NewSkin[];
    resolution: Resolution;
    cnGetTime: number;
    anchor: EventAnchor | null;
}

export function groupNewSkins(skins: NewSkin[], model: LagModel | null): INewSkinGroup[] {
    const groups = new Map<string, INewSkinGroup>();
    for (const s of skins) {
        const existing = groups.get(s.skinGroupId);
        if (existing) {
            existing.skins.push(s);
            if (sortKey(s.resolution, s.cnGetTime, model) < sortKey(existing.resolution, existing.cnGetTime, model)) {
                existing.resolution = s.resolution;
                existing.anchor = s.anchor;
            }
            if (!existing.skinGroupNameAuto && s.skinGroupNameAuto) existing.skinGroupNameAuto = s.skinGroupNameAuto;
            existing.cnGetTime = Math.min(existing.cnGetTime, s.cnGetTime);
        } else {
            groups.set(s.skinGroupId, { skinGroupId: s.skinGroupId, skinGroupName: s.skinGroupName, skinGroupNameAuto: s.skinGroupNameAuto, skins: [s], resolution: s.resolution, cnGetTime: s.cnGetTime, anchor: s.anchor });
        }
    }
    return [...groups.values()];
}
