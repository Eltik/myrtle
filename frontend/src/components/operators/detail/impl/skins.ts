import type { IDisplaySkin, ISkin } from "#/lib/api/skins";
import type { TypedT } from "#/lib/i18n/messages";
import { operatorElite0, operatorElite2, skinTexture, skinThumbnail } from "./assets";
import type { messages as skinMessages } from "./skins.messages";

export interface IUISkin {
    id: string;
    name: string;
    /** Heading shown above the skin name in the side card. */
    kicker: string;
    /** Sub text shown below the skin name in the side card. */
    sub: string;
    image: string;
    thumbnail: string;
    isDefault: boolean;
    displaySkin?: IDisplaySkin;
    /** Only in CN's data so far; its art is served from the CN tree and its text is CN's. */
    fromCn?: boolean;
}

interface IBuildArgs {
    skinsFromBackend: ISkin[];
    operatorId: string;
    operatorSkin: string | null;
    operatorPortrait: string | null;
    phasesLength: number;
    artistFallback?: string;
    /** True when this operator is an alternate form (e.g. Amiya Guard/Medic).
     *  Branches unlock at E2 and have no E0/E1 art, so the default placeholder
     *  is replaced with the E2 art and the "Evolved" entry is suppressed. */
    isBranchForm?: boolean;
    server?: "en" | "cn";
    /** CN's skins for this operator; any this server lacks follow its own, marked {@link IUISkin.fromCn}. */
    cnSkins?: ISkin[];
    /** Resolves the stand-in copy this builder supplies; see `skins.messages.ts`. */
    t: TypedT<typeof skinMessages>;
}

export function buildOperatorSkinList({ skinsFromBackend, operatorId, operatorSkin, operatorPortrait, phasesLength, artistFallback, isBranchForm, server, cnSkins, t }: IBuildArgs): IUISkin[] {
    const skins: IUISkin[] = [];

    const e0 = operatorElite0(operatorId, operatorSkin, operatorPortrait, server);
    const e2 = operatorElite2(operatorId, operatorSkin, operatorPortrait, server);

    if (isBranchForm) {
        skins.push({
            id: `${operatorId}_default`,
            name: t("skins.default.name"),
            kicker: t("skins.kicker.elite2"),
            sub: artistFallback ? t("skins.artist", { artist: artistFallback }) : t("skins.unlockedByDefault"),
            image: e2,
            thumbnail: e2,
            isDefault: true,
        });
    } else {
        skins.push({
            id: `${operatorId}_default`,
            name: t("skins.default.name"),
            kicker: phasesLength > 2 ? t("skins.kicker.elite0or1") : t("skins.kicker.elite0"),
            sub: artistFallback ? t("skins.artist", { artist: artistFallback }) : t("skins.unlockedByDefault"),
            image: e0,
            thumbnail: e0,
            isDefault: true,
        });

        if (phasesLength > 2) {
            skins.push({
                id: `${operatorId}_e2`,
                name: t("skins.evolvedArt.name"),
                kicker: t("skins.kicker.elite2"),
                sub: artistFallback ? t("skins.artist", { artist: artistFallback }) : t("skins.elite2Promotion"),
                image: e2,
                thumbnail: e2,
                isDefault: false,
            });
        }
    }

    const own = new Set<string>();
    const pushSkin = (skin: ISkin, skinServer: "en" | "cn" | undefined, fromCn: boolean) => {
        const id = skin.skinId;
        const isDefault = !id?.includes("@") && (id?.endsWith("#1") || id?.endsWith("#2"));
        if (!id || isDefault) return;

        const display = skin.displaySkin;
        skins.push({
            id,
            name: display?.skinName ?? t("skins.fallback.name"),
            kicker: display?.skinGroupName ?? t("skins.fallback.kicker"),
            sub: artistFallback ? t("skins.artist", { artist: artistFallback }) : (display?.obtainApproach ?? t("skins.fallback.sub")),
            image: skinTexture(operatorId, id, skinServer),
            thumbnail: skinThumbnail(operatorId, id, skinServer),
            isDefault: false,
            displaySkin: display,
            ...(fromCn ? { fromCn } : {}),
        });
    };

    for (const skin of skinsFromBackend) {
        if (skin.skinId) own.add(skin.skinId);
        pushSkin(skin, server, false);
    }
    for (const skin of cnSkins ?? []) {
        if (skin.skinId && !own.has(skin.skinId)) pushSkin(skin, "cn", true);
    }

    return skins;
}

export function chibiSkinKey(skinId: string): string {
    if (!skinId) return "default";
    if (skinId.endsWith("_default") || skinId.endsWith("_e2")) return "default";
    const at = skinId.indexOf("@");
    return at !== -1 ? skinId.slice(at + 1) : "default";
}
