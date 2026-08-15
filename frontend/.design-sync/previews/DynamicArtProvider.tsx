import { Badge, Card, DynamicArtOverlay, DynamicArtProvider } from "frontend";

/**
 * DynamicArtProvider is headless: it resolves the `dyn_illust` spine set for an
 * operator's equipped skin and hands it to every DynamicArtOverlay below it.
 * The chibi catalog is a stubbed server call in previews, so no dynamic set
 * resolves and each overlay renders nothing — which is exactly the static-art
 * fallback these surfaces show in the app when an operator has no dynamic
 * illustration or the "animate dynamic art" preference is off.
 */
const ART = "https://api.myrtle.moe/api/assets/textures/chararts";
const SKINPACK = "https://api.myrtle.moe/api/assets/textures/skinpack";

interface IHero {
    operatorCode: string;
    skinId: string | null;
    name: string;
    src: string;
    profession: string;
    elite: string;
}

const ROSTER: IHero[] = [
    { operatorCode: "char_4064_mlynar", skinId: null, name: "Mlynar", src: `${ART}/char_4064_mlynar/char_4064_mlynar_2.png`, profession: "Guard", elite: "E2 90" },
    { operatorCode: "char_263_skadi", skinId: null, name: "Skadi", src: `${ART}/char_263_skadi/char_263_skadi_2.png`, profession: "Guard", elite: "E2 90" },
];

function RosterHero({ hero, gated }: { hero: IHero; gated?: boolean }) {
    return (
        <Card className="flex w-full flex-col gap-0 overflow-hidden border-2 border-muted/30 py-0 pb-1">
            <div className="relative aspect-5/4 w-full overflow-hidden">
                <img alt={hero.name} className="h-full w-full object-cover object-top" decoding="async" src={hero.src} />
                <DynamicArtOverlay fit={{ mode: "contain", align: "center" }} operatorCode={hero.operatorCode} skinId={hero.skinId} viewportGated={gated} />
                <div className="absolute inset-0 bg-linear-to-t from-black/80 via-black/20 to-transparent" />
                <div className="absolute right-0 bottom-0 left-0 p-4">
                    <h3 className="mt-2 max-w-3/4 text-left font-bold text-white text-xl">{hero.name}</h3>
                    <div className="flex items-center justify-between">
                        <span className="text-sm text-white">{hero.profession}</span>
                        <span className="font-mono text-white/80 text-xs">{hero.elite}</span>
                    </div>
                </div>
            </div>
        </Card>
    );
}

export const RosterGrid = () => (
    <DynamicArtProvider server="en">
        <div className="grid w-full max-w-lg grid-cols-2 gap-3">
            {ROSTER.map((hero) => (
                <RosterHero gated hero={hero} key={hero.operatorCode} />
            ))}
        </div>
    </DynamicArtProvider>
);

export const OperatorDetailPanel = () => (
    <DynamicArtProvider server="en">
        <Card className="w-full max-w-md overflow-hidden py-0">
            <div className="relative aspect-5/4 w-full overflow-hidden bg-muted/30">
                <img alt="Mlynar" className="h-full w-full object-contain object-top" decoding="async" src={`${ART}/char_4064_mlynar/char_4064_mlynar_2.png`} />
                <DynamicArtOverlay fit={{ mode: "contain", align: "top" }} operatorCode="char_4064_mlynar" skinId={null} />
            </div>
            <div className="flex items-center justify-between px-4 py-3">
                <div className="flex flex-col gap-1">
                    <span className="font-semibold text-sm">Mlynar</span>
                    <span className="font-mono text-[11px] text-muted-foreground">char_4064_mlynar</span>
                </div>
                <Badge variant="secondary">E2 90 · Pot 6</Badge>
            </div>
        </Card>
    </DynamicArtProvider>
);

export const CnServerSkin = () => (
    <DynamicArtProvider server="cn">
        <Card className="w-full max-w-md overflow-hidden py-0">
            <div className="relative aspect-5/4 w-full overflow-hidden bg-muted/30">
                <img alt="Skadi the Corrupting Heart — Bloodline of Combat" className="h-full w-full object-contain object-top" decoding="async" src={`${SKINPACK}/char_1012_skadi2/char_1012_skadi2_boc%234.png`} />
                <DynamicArtOverlay fit={{ mode: "contain", align: "bottom" }} operatorCode="char_1012_skadi2" skinId="char_1012_skadi2@boc#4" />
            </div>
            <div className="flex items-center justify-between px-4 py-3">
                <div className="flex flex-col gap-1">
                    <span className="font-semibold text-sm">Skadi the Corrupting Heart</span>
                    <span className="font-mono text-[11px] text-muted-foreground">Bloodline of Combat</span>
                </div>
                <Badge variant="outline">CN</Badge>
            </div>
        </Card>
    </DynamicArtProvider>
);
