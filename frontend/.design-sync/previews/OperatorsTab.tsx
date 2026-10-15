import { OperatorsTab } from "frontend";
import type { LibRecord } from "../../src/components/story/library/impl/derive";
import type { StoryProgress } from "../../src/lib/story/progress";

// OperatorsTab is the Archives' operator-record shelf: a search box, "Hide
// finished", the read count, and a grid of operator cards (avatar, rarity
// stars, class, record count and how many are read, a check once finished).
// A card opens the operator's dialog. Inside the browse page's records
// section it runs with `controls={false}` and pages behind "Show more".
// Fixtures: live EN index operator records, ids verified by charId.

const RECORDS: LibRecord[] = [{"charId":"char_003_kalts","name":"Kal'tsit","rarity":6,"profession":"MEDIC","professionName":"Medic","avatarUrl":"/textures/spritepack/ui_char_avatar_7/char_003_kalts.png","wordCount":3109,"illustrationCount":4,"spriteCount":5,"stories":[{"id":"story_kalts_set_1_story_1","name":"End of a Long Journey","sort":101,"groupId":"story_kalts_set_1","hasScript":true,"wordCount":3109,"hasVideo":false,"requiredStages":[]}]},{"charId":"char_010_chen","name":"Ch'en","rarity":6,"profession":"WARRIOR","professionName":"Guard","avatarUrl":"/textures/spritepack/ui_char_avatar_7/char_010_chen.png","wordCount":4377,"illustrationCount":10,"spriteCount":13,"stories":[{"id":"story_chen_set_1_story_1","name":"In One Effort","sort":101,"groupId":"story_chen_set_1","hasScript":true,"wordCount":4377,"hasVideo":false,"requiredStages":[]}]},{"charId":"char_102_texas","name":"Texas","rarity":5,"profession":"PIONEER","professionName":"Vanguard","avatarUrl":"/textures/spritepack/ui_char_avatar_7/char_102_texas.png","wordCount":2614,"illustrationCount":5,"spriteCount":5,"stories":[{"id":"story_texas_set_1_story_1","name":"Guaranteed Success","sort":101,"groupId":"story_texas_set_1","hasScript":true,"wordCount":2614,"hasVideo":false,"requiredStages":[]}]},{"charId":"char_103_angel","name":"Exusiai","rarity":6,"profession":"SNIPER","professionName":"Sniper","avatarUrl":"/textures/spritepack/ui_char_avatar_7/char_103_angel.png","wordCount":5854,"illustrationCount":8,"spriteCount":13,"stories":[{"id":"story_angel_set_1_story_1","name":"Incoming Mail","sort":101,"groupId":"story_angel_set_1","hasScript":true,"wordCount":2571,"hasVideo":false,"requiredStages":[]},{"id":"story_angel_set_2_story_1","name":"Constants Within Changes","sort":201,"groupId":"story_angel_set_2","hasScript":true,"wordCount":3283,"hasVideo":false,"requiredStages":[]}]},{"charId":"char_112_siege","name":"Siege","rarity":6,"profession":"PIONEER","professionName":"Vanguard","avatarUrl":"/textures/spritepack/ui_char_avatar_7/char_112_siege.png","wordCount":2871,"illustrationCount":5,"spriteCount":8,"stories":[{"id":"story_siege_set_1_story_1","name":"Miraged Sun","sort":101,"groupId":"story_siege_set_1","hasScript":true,"wordCount":2871,"hasVideo":false,"requiredStages":[]}]},{"charId":"char_134_ifrit","name":"Ifrit","rarity":6,"profession":"CASTER","professionName":"Caster","avatarUrl":"/textures/spritepack/ui_char_avatar_8/char_134_ifrit.png","wordCount":2876,"illustrationCount":6,"spriteCount":8,"stories":[{"id":"story_ifrit_set_1_story_1","name":"Team Captain","sort":101,"groupId":"story_ifrit_set_1","hasScript":true,"wordCount":2876,"hasVideo":false,"requiredStages":[]}]},{"charId":"char_017_huang","name":"Blaze","rarity":6,"profession":"WARRIOR","professionName":"Guard","avatarUrl":"/textures/spritepack/ui_char_avatar_7/char_017_huang.png","wordCount":2953,"illustrationCount":3,"spriteCount":10,"stories":[{"id":"story_huang_set_1_story_1","name":"Will You?","sort":101,"groupId":"story_huang_set_1","hasScript":true,"wordCount":2953,"hasVideo":false,"requiredStages":[]}]},{"charId":"char_1013_chen2","name":"Ch'en the Holungday","rarity":6,"profession":"SNIPER","professionName":"Sniper","avatarUrl":"/textures/spritepack/ui_char_avatar_7/char_1013_chen2.png","wordCount":6028,"illustrationCount":9,"spriteCount":11,"stories":[{"id":"story_chen2_set_1_story_1","name":"Belonging","sort":101,"groupId":"story_chen2_set_1","hasScript":true,"wordCount":2860,"hasVideo":false,"requiredStages":[]},{"id":"story_chen2_set_2_story_1","name":"Pathways","sort":201,"groupId":"story_chen2_set_2","hasScript":true,"wordCount":3168,"hasVideo":false,"requiredStages":[]}]},{"charId":"char_101_sora","name":"Sora","rarity":5,"profession":"SUPPORT","professionName":"Supporter","avatarUrl":"/textures/spritepack/ui_char_avatar_7/char_101_sora.png","wordCount":5561,"illustrationCount":10,"spriteCount":11,"stories":[{"id":"story_sora_set_1_story_1","name":"Distance","sort":101,"groupId":"story_sora_set_1","hasScript":true,"wordCount":1782,"hasVideo":false,"requiredStages":[]},{"id":"story_sora_set_2_story_1","name":"To the Stage!","sort":201,"groupId":"story_sora_set_2","hasScript":true,"wordCount":3779,"hasVideo":false,"requiredStages":[]}]},{"charId":"char_108_silent","name":"Silence","rarity":5,"profession":"MEDIC","professionName":"Medic","avatarUrl":"/textures/spritepack/ui_char_avatar_7/char_108_silent.png","wordCount":2854,"illustrationCount":4,"spriteCount":7,"stories":[{"id":"story_silent_set_1_story_1","name":"New to the Workforce","sort":101,"groupId":"story_silent_set_1","hasScript":true,"wordCount":2854,"hasVideo":false,"requiredStages":[]}]}];

const AT = 1715600000000;
/** Kal'tsit, Texas and both of Exusiai's records read; Ch'en read in the game. */
const PROGRESS: StoryProgress = {
    v: 2,
    read: Object.fromEntries([RECORDS[0], RECORDS[2], RECORDS[3]].flatMap((r) => r.stories.map((s) => [s.id, AT]))),
    pos: {},
};
const GAME_READ = new Set(RECORDS[1].stories.map((s) => s.id));

/** The tab with its controls: ten operators, four finished (Ch'en by the game's verdict). */
export const WithControls = () => (
    <div style={{ width: 860 }}>
        <OperatorsTab records={RECORDS} progress={PROGRESS} gameRead={GAME_READ} />
    </div>
);

/** Embedded in the browse page's records section: no controls of its own. */
export const Embedded = () => (
    <div style={{ width: 860 }}>
        <OperatorsTab records={RECORDS.slice(0, 5)} progress={PROGRESS} gameRead={GAME_READ} controls={false} />
    </div>
);

/** A search that matches nobody. */
export const Empty = () => (
    <div style={{ width: 860 }}>
        <OperatorsTab records={[]} progress={PROGRESS} gameRead={GAME_READ} />
    </div>
);
