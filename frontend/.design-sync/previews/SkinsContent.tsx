import { SkinsContent } from "frontend";

// Fields lifted verbatim from GET https://api.myrtle.moe/api/operators/<id>.
// The alternate-outfit catalog comes from a `createServerFn` fetcher that is
// stubbed in the preview bundle, so the list is the two entries
// `buildOperatorSkinList` derives from the operator itself: the E0/E1 art and the
// Elite 2 promotion art.
const MLYNAR = {
    id: "char_4064_mlynar",
    name: "Młynar",
    server: "en",
    rarity: "TIER_6",
    profession: "WARRIOR",
    subProfessionId: "librator",
    artists: ["竜崎いち"],
    portrait: "/portraits/char_4064_mlynar_2.png",
    skin: "/textures/chararts/char_4064_mlynar/char_4064_mlynar_2.png",
    phases: [{ phase: "PHASE_0" }, { phase: "PHASE_1" }, { phase: "PHASE_2" }],
};

const KROOS = {
    id: "char_124_kroos",
    name: "Kroos",
    server: "en",
    rarity: "TIER_3",
    profession: "SNIPER",
    subProfessionId: "fastshot",
    artists: ["下野宏铭"],
    portrait: "/portraits/char_124_kroos_1.png",
    skin: "/textures/chararts/char_124_kroos/char_124_kroos_1.png",
    phases: [{ phase: "PHASE_0" }, { phase: "PHASE_1" }],
};

const AMIYA_GUARD = {
    id: "char_1001_amiya2",
    name: "Amiya",
    server: "en",
    rarity: "TIER_5",
    profession: "WARRIOR",
    subProfessionId: "musha",
    tmplDefault: "char_002_amiya",
    artists: ["三目YYB", "唯@W"],
    portrait: "/portraits/char_1001_amiya2_2.png",
    skin: "/textures/chararts/char_1001_amiya2/char_1001_amiya2_2.png",
    phases: [{ phase: "PHASE_0" }, { phase: "PHASE_1" }, { phase: "PHASE_2" }],
};

export const Outfits = () => (
    <div className="w-full max-w-lg rounded-xl border border-border bg-card">
        <SkinsContent operator={MLYNAR} />
    </div>
);

export const EliteZeroOnly = () => (
    <div className="w-full max-w-lg rounded-xl border border-border bg-card">
        <SkinsContent operator={KROOS} />
    </div>
);

export const AlternateForm = () => (
    <div className="w-full max-w-lg rounded-xl border border-border bg-card">
        <SkinsContent operator={AMIYA_GUARD} />
    </div>
);
