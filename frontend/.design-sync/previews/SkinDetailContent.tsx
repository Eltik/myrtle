import { Badge, Dialog, SkinDetailContent } from "frontend";
import { type ReactNode, useEffect } from "react";

// The skin detail popup's content (a DialogContent): the outfit's art on the
// left, and on the right the wearer's avatar, outfit and series names, then a
// description list of price, obtain, usage, description, dialog, credits and
// release date. The release tool's SkinPopup and the skin dialogs mount it in a
// Dialog. Entries are `/api/skins/index` rows; dynamic art stays off (no
// DynamicArtProvider), so the static art shows.

const THORNS = {"skinId": "char_293_thorns@boc#8", "charId": "char_293_thorns", "displaySkin": {"skinName": "Blade-cleaved Tides", "skinGroupId": "2024#boc", "skinGroupName": "Bloodline of Combat/VIII", "skinGroupSortIndex": 177, "displayTagId": null, "getTime": 1730394000, "sortId": 326, "description": "'Once more has the tide returned to Iberia, and it follows the point of my sword. Do you hear the command of the waves as they crash upon the shore? They call upon Iberia to set sail once more.'", "content": "<color name=#ffffff>Bloodline of Combat Collection/Blade-cleaved Tides. Thorns bares his blade, and Iberia's fleet sets sail after a century of silence. A golden secret has emerged in lawless lands, and seas of sand and salt can no longer conceal its radiance.</color>", "dialog": "Bloodline of Combat Collection/Blade-cleaved Tides. Thorns bares his blade, and Iberia's fleet sets sail after a century of silence. A golden secret has emerged in lawless lands, and seas of sand and salt can no longer conceal its radiance.", "usage": "One of Thorns's outfits for crucial moments.", "obtainApproach": "Store", "designerList": null, "drawerList": ["Studio Montagne"]}, "obtainChannel": "store"};
const AMBRIEL = {"skinId": "char_302_glaze@summer#11", "charId": "char_302_glaze", "displaySkin": {"skinName": "Holiday HD29", "skinGroupId": "2023#summer", "skinGroupName": "Coral Coast/XI", "skinGroupSortIndex": 115, "displayTagId": "Event Reward", "getTime": 1709035200, "sortId": 213, "description": "She should have made proper plans before setting off on her trip. Now, the blistering heat is right by her side while the shade is far away. She can only take in the sun while she figures something out.", "content": "MARTHE [Coral Coast] Holiday Series HD29. Comfortable, relaxed, breathable, and lightweight. Allows you to enjoy a perfect beach experience in absolute relaxation.", "dialog": "MARTHE [Coral Coast] Holiday Series HD29. Comfortable, relaxed, breathable, and lightweight. Allows you to enjoy a perfect beach experience in absolute relaxation.", "usage": "One of Ambriel's summer outfits.", "obtainApproach": "Event Gift", "designerList": null, "drawerList": ["水滴鱼"]}, "obtainChannel": "event"};

const API = "https://api.myrtle.moe/api";

/** An open dialog over a full-viewport stage; the first-control focus ring is blurred. */
const Stage = ({ children }: { children: ReactNode }) => {
    useEffect(() => {
        const ids = [60, 180, 400].map((ms) => setTimeout(() => (document.activeElement as HTMLElement | null)?.blur(), ms));
        return () => ids.forEach(clearTimeout);
    }, []);
    return (
        <div className="min-h-dvh">
            <Dialog open>{children}</Dialog>
        </div>
    );
};

/** A store outfit with no price on record (the price row drops), and its art credit. */
export const StoreOutfit = () => (
    <Stage>
        <SkinDetailContent skin={THORNS} opName="Thorns" skinName="Blade-cleaved Tides" avatarURL={`${API}/avatar/char_293_thorns_boc%238`} server="en" />
    </Stage>
);

/** An event reward: a free price and a corner badge over the art. */
export const EventReward = () => (
    <Stage>
        <SkinDetailContent
            skin={AMBRIEL}
            opName="Ambriel"
            skinName="Holiday HD29"
            avatarURL={`${API}/avatar/char_302_glaze_summer%2311`}
            server="en"
            price={{ kind: "free", label: "Free", tooltip: "Event reward" }}
            corner={<Badge variant="secondary">Event Reward</Badge>}
        />
    </Stage>
);
