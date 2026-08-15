import { EventDetailDialog } from "frontend";

/** One `IEventVM` off `buildStageTree(stageIndex)` — Episode 4, Burning Run. */
const BURNING_RUN = {
    zoneId: "main_4",
    title: "Burning Run",
    kicker: null,
    banner: "textures/spritepack/ui_home_act_banner_zone_0/main_4.png",
    codeRange: "4-1 ~ SW-EV-4",
    statusLabel: "Main Story",
    bossCount: 2,
    stageCount: 12,
    order: 4,
    tone: "var(--primary)",
    group: "story",
    stages: [
        {
            stageId: "main_04-01",
            badge: "4-1",
            title: "Consolation",
            apCost: 18,
            boss: false,
            hasChallenge: true,
            canView: true,
            difficulty: "NORMAL",
            preview: "textures/arts/ui/stage_mappreview_h2_main_04_0/main_04-01.png",
        },
        {
            stageId: "main_04-02",
            badge: "4-2",
            title: "A Walk in the Rain",
            apCost: 18,
            boss: false,
            hasChallenge: true,
            canView: true,
            difficulty: "NORMAL",
            preview: "textures/arts/ui/stage_mappreview_h2_main_04_0/main_04-02.png",
        },
        {
            stageId: "main_04-03",
            badge: "4-3",
            title: "Artificial Cold",
            apCost: 18,
            boss: false,
            hasChallenge: true,
            canView: true,
            difficulty: "NORMAL",
            preview: "textures/arts/ui/stage_mappreview_h2_main_04_0/main_04-03.png",
        },
        {
            stageId: "main_04-04",
            badge: "4-4",
            title: "Nerves of Steel",
            apCost: 18,
            boss: true,
            hasChallenge: true,
            canView: true,
            difficulty: "NORMAL",
            preview: "textures/arts/ui/stage_mappreview_h2_main_04_0/main_04-04.png",
        },
        {
            stageId: "main_04-05",
            badge: "4-5",
            title: "Bureaucracy",
            apCost: 18,
            boss: false,
            hasChallenge: true,
            canView: true,
            difficulty: "NORMAL",
            preview: "textures/arts/ui/stage_mappreview_h2_main_04_0/main_04-05.png",
        },
        {
            stageId: "main_04-06",
            badge: "4-6",
            title: "Ignorance",
            apCost: 18,
            boss: false,
            hasChallenge: true,
            canView: true,
            difficulty: "NORMAL",
            preview: "textures/arts/ui/stage_mappreview_h2_main_04_0/main_04-06.png",
        },
        {
            stageId: "main_04-07",
            badge: "4-7",
            title: "Mutual Benefits",
            apCost: 18,
            boss: false,
            hasChallenge: true,
            canView: true,
            difficulty: "NORMAL",
            preview: "textures/arts/ui/stage_mappreview_h2_main_04_0/main_04-07.png",
        },
        {
            stageId: "main_04-08",
            badge: "4-8",
            title: "Acute Stress Disorder",
            apCost: 21,
            boss: false,
            hasChallenge: true,
            canView: true,
            difficulty: "NORMAL",
            preview: "textures/arts/ui/stage_mappreview_h2_main_04_0/main_04-08.png",
        },
        {
            stageId: "main_04-09",
            badge: "4-9",
            title: "Bad to the Bone",
            apCost: 21,
            boss: false,
            hasChallenge: true,
            canView: true,
            difficulty: "NORMAL",
            preview: "textures/arts/ui/stage_mappreview_h2_main_04_0/main_04-09.png",
        },
        {
            stageId: "main_04-10",
            badge: "4-10",
            title: "Extinguished Flames",
            apCost: 21,
            boss: true,
            hasChallenge: true,
            canView: true,
            difficulty: "NORMAL",
            preview: "textures/arts/ui/stage_mappreview_h2_main_04_0/main_04-10.png",
        },
        {
            stageId: "sub_04-1-1",
            badge: "S4-1",
            title: "Cluster-1",
            apCost: 18,
            boss: false,
            hasChallenge: false,
            canView: true,
            difficulty: "NORMAL",
            preview: "textures/arts/ui/stage_mappreview_h2_sub_04_0/sub_04-1-1.png",
        },
        {
            stageId: "sub_04-1-2",
            badge: "S4-2",
            title: "Cluster-2",
            apCost: 18,
            boss: false,
            hasChallenge: false,
            canView: true,
            difficulty: "NORMAL",
            preview: "textures/arts/ui/stage_mappreview_h2_sub_04_0/sub_04-1-2.png",
        },
    ],
};

/** A bossless side event — Crossing (`act20mini`). */
const CROSSING = {
    zoneId: "act20mini",
    title: "Crossing",
    kicker: null,
    banner: "textures/spritepack/ui_zone_home_theme_act20mini/act20mini.png",
    codeRange: "CG-1 ~ CG-S-2",
    statusLabel: "Events",
    bossCount: 0,
    stageCount: 10,
    order: 1786622400,
    tone: "#f7a452",
    group: "events",
    stages: [
        {
            stageId: "act20mini_01",
            badge: "CG-1",
            title: "Beyond the Applause",
            apCost: 9,
            boss: false,
            hasChallenge: true,
            canView: true,
            difficulty: "NORMAL",
            preview: "textures/arts/ui/stage_mappreview_h2_act20mini_1_0/act20mini_01.png",
        },
        {
            stageId: "act20mini_02",
            badge: "CG-2",
            title: "Bubbles in the Wineglass",
            apCost: 9,
            boss: false,
            hasChallenge: true,
            canView: true,
            difficulty: "NORMAL",
            preview: "textures/arts/ui/stage_mappreview_h2_act20mini_1_0/act20mini_02.png",
        },
        {
            stageId: "act20mini_03",
            badge: "CG-3",
            title: "The Art of Contract",
            apCost: 12,
            boss: false,
            hasChallenge: true,
            canView: true,
            difficulty: "NORMAL",
            preview: "textures/arts/ui/stage_mappreview_h2_act20mini_1_0/act20mini_03.png",
        },
        {
            stageId: "act20mini_04",
            badge: "CG-4",
            title: "Waking From Inebriation",
            apCost: 12,
            boss: false,
            hasChallenge: true,
            canView: true,
            difficulty: "NORMAL",
            preview: "textures/arts/ui/stage_mappreview_h2_act20mini_1_0/act20mini_04.png",
        },
        {
            stageId: "act20mini_05",
            badge: "CG-5",
            title: "Reap What You Sow",
            apCost: 15,
            boss: false,
            hasChallenge: true,
            canView: true,
            difficulty: "NORMAL",
            preview: "textures/arts/ui/stage_mappreview_h2_act20mini_1_0/act20mini_05.png",
        },
        {
            stageId: "act20mini_06",
            badge: "CG-6",
            title: "Set Price",
            apCost: 15,
            boss: false,
            hasChallenge: true,
            canView: true,
            difficulty: "NORMAL",
            preview: "textures/arts/ui/stage_mappreview_h2_act20mini_1_0/act20mini_06.png",
        },
        {
            stageId: "act20mini_07",
            badge: "CG-7",
            title: "Light Luggage",
            apCost: 21,
            boss: false,
            hasChallenge: true,
            canView: true,
            difficulty: "NORMAL",
            preview: "textures/arts/ui/stage_mappreview_h2_act20mini_1_0/act20mini_07.png",
        },
        {
            stageId: "act20mini_08",
            badge: "CG-8",
            title: "Entry Permit",
            apCost: 21,
            boss: false,
            hasChallenge: true,
            canView: true,
            difficulty: "NORMAL",
            preview: "textures/arts/ui/stage_mappreview_h2_act20mini_1_0/act20mini_08.png",
        },
        {
            stageId: "act20mini_s01",
            badge: "CG-S-1",
            title: "New Winds",
            apCost: 0,
            boss: false,
            hasChallenge: false,
            canView: true,
            difficulty: "NORMAL",
            preview: "textures/arts/ui/stage_mappreview_h2_act20mini_a_0/act20mini_s01.png",
        },
        {
            stageId: "act20mini_s02",
            badge: "CG-S-2",
            title: "The Bestower of Trust",
            apCost: 0,
            boss: false,
            hasChallenge: false,
            canView: true,
            difficulty: "NORMAL",
            preview: "textures/arts/ui/stage_mappreview_h2_act20mini_a_0/act20mini_s02.png",
        },
    ],
};

const noop = () => {};

/** A story chapter: banner art, category blurb, totals and six field previews. */
export const ChapterOverview = () => (
    <div className="min-h-[520px]">
        <EventDetailDialog event={BURNING_RUN} onClose={noop} onBrowse={noop} />
    </div>
);

/** A limited-time event with no boss operation: the skull line goes, Bosses reads "-". */
export const WithoutBosses = () => (
    <div className="min-h-[520px]">
        <EventDetailDialog event={CROSSING} onClose={noop} onBrowse={noop} />
    </div>
);

/** Procedural modes ship no map art at all: banner and previews fall back to the group wash. */
export const WithoutArtwork = () => (
    <div className="min-h-[520px]">
        <EventDetailDialog
            event={{ ...CROSSING, zoneId: "rogue_5", title: "Sui's Garden of Grotesqueries", group: "is", tone: "#b78fe6", statusLabel: "Integrated Strategies", codeRange: "1-1_dlc1 ~ Stasis", banner: null, stages: CROSSING.stages.map((s) => ({ ...s, preview: null })) }}
            onClose={noop}
            onBrowse={noop}
        />
    </div>
);
