import { DetailRow } from "frontend";

// One labelled row of the skin detail panel's description list: a small mono
// uppercase label over the value. SkinDetailContent stacks them in a
// `<dl className="flex flex-col gap-3 text-sm">`. Values are Thorns's
// "Blade-cleaved Tides" (char_293_thorns@boc#8) from `/api/skins/index`.

/** The detail panel's list, as SkinDetailContent sets it. */
export const DetailList = () => (
    <div className="w-full max-w-sm p-5">
        <dl className="flex flex-col gap-3 text-sm">
            <DetailRow label="Obtain">Store</DetailRow>
            <DetailRow label="Usage">One of Thorns's outfits for crucial moments.</DetailRow>
            <DetailRow label="Description">'Once more has the tide returned to Iberia, and it follows the point of my sword. Do you hear the command of the waves as they crash upon the shore? They call upon Iberia to set sail once more.'</DetailRow>
            <DetailRow label="Released">November 1, 2024</DetailRow>
        </dl>
    </div>
);

/** A free outfit: the price reads in green, with its note. */
export const FreePrice = () => (
    <div className="w-full max-w-sm p-5">
        <dl className="flex flex-col gap-3 text-sm">
            <DetailRow label="Price">
                <span className="font-semibold text-emerald-600 dark:text-emerald-400">Free</span>
                <span className="ml-2 text-muted-foreground text-xs">- event reward</span>
            </DetailRow>
        </dl>
    </div>
);

/** The quoted dialog line and the art credit. */
export const DialogAndCredits = () => (
    <div className="w-full max-w-sm p-5">
        <dl className="flex flex-col gap-3 text-sm">
            <DetailRow label="Dialog">
                <q className="italic">Bloodline of Combat Collection/Blade-cleaved Tides. Thorns bares his blade, and Iberia's fleet sets sail after a century of silence.</q>
            </DetailRow>
            <DetailRow label="Credits">
                <span>Art: Studio Montagne</span>
            </DetailRow>
        </dl>
    </div>
);
