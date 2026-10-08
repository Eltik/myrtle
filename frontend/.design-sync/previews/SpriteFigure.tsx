import { SpriteFigure } from "frontend";

// One story-sprite expression drawn through the reader's own body + face
// composite (or the backend's composed thumb), inside a window cropped to the
// head. The square plate is sized in percent, so one markup serves a 150 px
// card and a 600 px preview. Variant is Kal'tsit's most-used expression from
// the live `/api/story/sprites` index. Crops are the gallery's own constants.

const KALTSIT_1 = {"key":"#1$1","bodyUrl":"/textures/avg/characters/avg_003_kalts_1/avg_003_kalts_1$1.png","faceUrl":"/textures/avg/characters/avg_003_kalts_1/1$1.png","facePos":{"x":551.0,"y":62.0,"w":136.0,"h":181.0},"bodySize":{"w":1280.0,"h":1280.0},"plate":{"x":0.0,"y":180.0,"w":890.0,"h":890.0},"wholeBody":false,"uses":600};
/** `CARD_CROP`: the grid card's 3:4 head window. */
const CARD_CROP = { zoom: 1.7, aspect: 4 / 3, anchorY: 0.34 };
/** `CELL_CROP`: the expression sheet's square face window. */
const CELL_CROP = { zoom: 3.6, aspect: 1, anchorY: 0.45 };

/** The card crop, composed live from body plate + face patch. */
export const CardCrop = () => (
    <div className="w-44 p-4">
        <SpriteFigure variant={KALTSIT_1} name="avg_003_kalts_1" crop={CARD_CROP} className="aspect-3/4 w-full rounded-lg bg-secondary/55" />
    </div>
);

/** The sheet cell's tighter square face crop. */
export const FaceCrop = () => (
    <div className="w-36 p-4">
        <SpriteFigure variant={KALTSIT_1} name="avg_003_kalts_1" crop={CELL_CROP} className="aspect-square w-full rounded-lg bg-secondary/55" />
    </div>
);

/** No crop: the whole plate, as the sheet's large preview draws it. */
export const WholePlate = () => (
    <div className="w-80 p-4">
        <SpriteFigure variant={KALTSIT_1} name="avg_003_kalts_1" crop={null} className="aspect-square w-full rounded-lg bg-secondary/55" />
    </div>
);
