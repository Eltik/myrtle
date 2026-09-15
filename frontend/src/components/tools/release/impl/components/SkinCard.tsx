import { Search } from "lucide-react";
import type * as React from "react";
import { asset } from "#/components/operators/detail/impl/assets";
import { ClassIcon } from "#/components/operators/list/impl/components/Icons";
import { useT } from "#/lib/i18n";
import type { TypedT } from "#/lib/i18n/messages";
import { cn, parseOperatorName, rarityToNumber } from "#/lib/utils";
import type { SkinPrice } from "#/types/generated/SkinPrice";
import { useAutoTranslate } from "../autoTranslate";
import type { IPlanSkin } from "../plan";
import type { messages } from "./SkinCard.messages";
import styles from "./SkinCard.module.css";
import { useSkinPopup } from "./SkinPopup";
import { type OperatorLookup, resolveName, useArt } from "./shared";

type CardT = TypedT<typeof messages>;

const CARD_SHEET = asset("/textures/arts/ui/[uc]classicgachapool/classic_gacha%230.png");
const CARD_CROP = { x: 19, y: 389, w: 276, h: 608, sheet: 1024 };
const CARD_WIDTH = 177;
export const FALLBACK_COLOR = "#4b5563";

/** The obtain methods the game data spells out, mapped to a badge-sized label. */
const OBTAIN_SHORT_KEYS: Record<string, keyof typeof messages & string> = {
    "Event Reward": "release.card.obtain.eventReward",
    "Event Gift": "release.card.obtain.eventGift",
    "Integrated Strategies": "release.card.obtain.is",
    "Reclamation Algorithm": "release.card.obtain.ra",
    "Quest Reward": "release.card.obtain.quest",
    "Obtain from Special Pack": "release.card.obtain.pack",
};

function obtainShort(obtain: string, t: CardT): string {
    const key = OBTAIN_SHORT_KEYS[obtain];
    if (key) return t(key);
    return /code/i.test(obtain) ? t("release.card.obtain.code") : obtain.split(/[\s,、]/)[0].slice(0, 6);
}

function priceTitle(price: SkinPrice, t: CardT): string {
    return price.store ? t("release.card.price.store", { price: price.price }) : t("release.card.price.none", { obtain: price.obtain || t("release.card.price.notSold") });
}

export function stopsOf(colors: string[]): string {
    return colors.length === 1 ? `${colors[0]}, ${colors[0]}` : colors.join(", ");
}

export function cardVars(colors: string[], width = CARD_WIDTH): React.CSSProperties {
    const k = width / CARD_CROP.w;
    const palette = colors.length > 0 ? colors : [FALLBACK_COLOR];
    return {
        "--planner-stops": stopsOf(palette),
        "--planner-c0": palette[0],
        "--planner-sheet": `url("${CARD_SHEET}")`,
        "--planner-sheet-size": `${CARD_CROP.sheet * k}px`,
        "--planner-sheet-pos": `${-CARD_CROP.x * k}px ${-CARD_CROP.y * k}px`,
    } as React.CSSProperties;
}

export function SkinCard({ skin, on, lookup, onPick }: { skin: IPlanSkin; on: boolean; lookup: OperatorLookup; onPick: (skin: IPlanSkin, on: boolean) => void }): React.ReactElement {
    const t: CardT = useT("tools");
    const autoOn = useAutoTranslate();
    const art = useArt(skin.portraitPath);
    const openPopup = useSkinPopup();
    const entry = lookup.get(skin.charId);
    const opName = entry ? parseOperatorName(entry.name).displayName : (skin.charName?.text ?? skin.charId);
    const rarity = entry ? rarityToNumber(entry.rarity) : 0;
    const skinName = resolveName(skin.skinName, skin.skinNameEn, skin.skinNameAuto, autoOn).text;
    return (
        <div className={cn(styles.card, on && styles.selected)} style={cardVars(skin.colors)}>
            <button
                type="button"
                aria-pressed={on}
                aria-label={on ? t("release.card.remove", { operator: opName, skin: skinName }) : t("release.card.add", { operator: opName, skin: skinName })}
                title={t("release.card.title", { operator: opName, skin: skinName })}
                onClick={() => onPick(skin, !on)}
                className="absolute inset-0 z-2 cursor-pointer bg-transparent"
            />
            {art.src ? <img src={art.src} alt="" loading="lazy" onError={art.onError} className={styles.portrait} /> : <span className={cn(styles.portrait, "flex items-center justify-center font-bold text-[40px] text-white/30")}>{opName.charAt(0)}</span>}
            <span className={styles.shine} />
            <span className={styles.price} title={priceTitle(skin.price, t)}>
                <span className={cn(styles.priceNumber, !skin.price.store && styles.priceLabel)}>{skin.price.store ? skin.price.price : obtainShort(skin.price.obtain, t)}</span>
                {entry && (
                    <span className={styles.priceClass}>
                        <ClassIcon profession={entry.profession} size={28} />
                    </span>
                )}
            </span>
            {rarity > 0 && <span className={styles.stars}>{"★".repeat(rarity)}</span>}
            <button
                type="button"
                aria-label={t("release.card.inspect", { skin: skinName })}
                className={styles.inspect}
                onClick={(e) => {
                    e.stopPropagation();
                    openPopup({ charId: skin.charId, skinId: skin.skinId, skinName: skin.skinName, charName: skin.charName });
                }}
            >
                <Search className="size-5" />
            </button>
            <span className={styles.overlay}>{t("release.card.selected")}</span>
            <span className={styles.strip} style={{ fontSize: opName.length > 14 ? 11 : opName.length > 10 ? 13 : 16 }}>
                {opName}
            </span>
        </div>
    );
}
