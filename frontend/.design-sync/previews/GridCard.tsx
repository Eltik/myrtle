import { Button, DeleteGridButton, GridCard } from "frontend";
import { EyeOffIcon, PencilIcon } from "lucide-react";
import type { ReactNode } from "react";

// One grid in the browse gallery and on My grids: the board in miniature (every
// cell in the grid's shape, each pick's art), title, size, owner, fork count and
// the allowed types. Summaries are live `/api/grids` responses (owner names other
// than Eltik's replaced); the icons are real `/avatar/...` paths, and the last
// cell of "About Me" is a CN-only operator fetched from the `cn` server.

/** "About Me", 6 x 6, 36 picks. */
const ABOUT_ME = {"slug":"about-me-i95xd5","title":"About Me","rows":6,"cols":6,"owner":{"id":"4a919d84-3b2d-427f-bf86-5399fd35776d","name":"Eltik"},"fork_count":7,"is_listed":true,"entity_kinds":["operator","skin","story_sprite"],"updated_at":"2024-05-13T21:40:00Z","preview":[{"kind":"operator","icon":"/avatar/char_4193_lemuen","server":null},{"kind":"operator","icon":"/avatar/char_348_ceylon","server":null},{"kind":"operator","icon":"/avatar/char_4122_grabds","server":null},{"kind":"operator","icon":"/avatar/char_1028_texas2","server":null},{"kind":"operator","icon":"/avatar/char_249_mlyss","server":null},{"kind":"operator","icon":"/avatar/char_1032_excu2","server":null},{"kind":"operator","icon":"/avatar/char_4202_haruka","server":null},{"kind":"operator","icon":"/avatar/char_4194_rmixer","server":null},{"kind":"operator","icon":"/avatar/char_245_cello","server":null},{"kind":"operator","icon":"/avatar/char_498_inside","server":null},{"kind":"operator","icon":"/avatar/char_421_crow","server":null},{"kind":"skin","icon":"/avatar/char_300_phenxi_boc%239","server":null},{"kind":"operator","icon":"/avatar/char_4015_spuria","server":null},{"kind":"operator","icon":"/avatar/char_264_f12yin","server":null},{"kind":"operator","icon":"/avatar/char_362_saga","server":null},{"kind":"operator","icon":"/avatar/char_4141_marcil","server":null},{"kind":"operator","icon":"/avatar/char_213_mostma","server":null},{"kind":"operator","icon":"/avatar/char_377_gdglow","server":null},{"kind":"operator","icon":"/avatar/char_4188_confes","server":null},{"kind":"operator","icon":"/avatar/char_192_falco","server":null},{"kind":"operator","icon":"/avatar/char_302_glaze","server":null},{"kind":"operator","icon":"/avatar/char_4187_graceb","server":null},{"kind":"operator","icon":"/avatar/char_332_archet","server":null},{"kind":"operator","icon":"/avatar/char_1041_angel2","server":null},{"kind":"operator","icon":"/avatar/char_222_bpipe","server":null},{"kind":"operator","icon":"/avatar/char_4182_oblvns","server":null},{"kind":"operator","icon":"/avatar/char_2025_shu","server":null},{"kind":"operator","icon":"/avatar/char_4163_rosesa","server":null},{"kind":"operator","icon":"/avatar/char_113_cqbw","server":null},{"kind":"operator","icon":"/avatar/char_4146_nymph","server":null},{"kind":"operator","icon":"/avatar/char_4091_ulika","server":null},{"kind":"operator","icon":"/avatar/char_4036_forcer","server":null},{"kind":"story_sprite","icon":"/story-sprite-thumb/avg_npc_1793","server":null},{"kind":"story_sprite","icon":"/story-sprite-thumb/avg_npc_1781","server":null},{"kind":"operator","icon":"/avatar/char_4193_lemuen","server":null},{"kind":"operator","icon":"/avatar/char_1015_aglna2","server":"cn"}]};

/** "A 3 panel comic", 1 x 3: two Jessicas and an outfit. */
const COMIC = {"slug":"a-3-panel-comic-5htphz","title":"A 3 panel comic","rows":1,"cols":3,"owner":{"id":"u-5htphz","name":"Closure"},"fork_count":4,"is_listed":true,"entity_kinds":["operator","skin"],"updated_at":"2024-05-13T21:40:00Z","preview":[{"kind":"operator","icon":"/avatar/char_235_jesica","server":null},{"kind":"operator","icon":"/avatar/char_1034_jesca2","server":null},{"kind":"skin","icon":"/avatar/char_1034_jesca2_cfa%231","server":null}]};

/** "Skills", a 3 x 3 template: labels only, no picks yet. */
const SKILLS = {"slug":"skills-0f6nbd","title":"Skills","rows":3,"cols":3,"owner":{"id":"u-0f6nbd","name":"Ptilopsis"},"fork_count":6,"is_listed":true,"entity_kinds":["skill"],"updated_at":"2024-05-13T21:40:00Z","preview":[null,null,null,null,null,null,null,null,null]};

/** A 7 x 6 with eight allowed types, unlisted (the My grids badge). */
const ABOUT_ME_WIDE = {"slug":"about-me-4eqi31","title":"About Me","rows":7,"cols":6,"owner":{"id":"u-4eqi31","name":"Thornsfan"},"fork_count":1,"is_listed":false,"entity_kinds":["operator","subclass","event","faction","stronghold_bond","skin","integrated_strategies","story_sprite"],"updated_at":"2024-05-13T21:40:00Z","preview":[{"kind":"operator","icon":"/avatar/char_293_thorns","server":null},{"kind":"operator","icon":"/avatar/char_401_elysm","server":null},{"kind":"operator","icon":"/avatar/char_455_nothin","server":null},{"kind":"operator","icon":"/avatar/char_151_myrtle","server":null},{"kind":"operator","icon":"/avatar/char_4204_mantra","server":null},{"kind":"operator","icon":"/avatar/char_1039_thorn2","server":null},{"kind":"operator","icon":"/avatar/char_4163_rosesa","server":null},{"kind":"operator","icon":"/avatar/char_322_lmlee","server":null},{"kind":"operator","icon":"/avatar/char_4064_mlynar","server":null},{"kind":"operator","icon":"/avatar/char_1039_thorn2","server":null},{"kind":"operator","icon":"/avatar/char_1039_thorn2","server":null},{"kind":"skin","icon":"/avatar/char_293_thorns_boc%238","server":null},{"kind":"operator","icon":"/avatar/char_293_thorns","server":null},{"kind":"operator","icon":"/avatar/char_172_svrash","server":null},{"kind":"operator","icon":"/avatar/char_1034_jesca2","server":null},{"kind":"operator","icon":"/avatar/char_4195_radian","server":null},{"kind":"operator","icon":"/avatar/char_4042_lumen","server":null},{"kind":"operator","icon":"/avatar/char_1034_jesca2","server":null},{"kind":"operator","icon":"/avatar/char_4000_jnight","server":null},{"kind":"operator","icon":"/avatar/char_124_kroos","server":null},{"kind":"operator","icon":"/avatar/char_235_jesica","server":null},{"kind":"operator","icon":"/avatar/char_1021_kroos2","server":null},{"kind":"operator","icon":"/avatar/char_1039_thorn2","server":null},{"kind":"operator","icon":"/avatar/char_1039_thorn2","server":null},{"kind":"operator","icon":"/avatar/char_420_flamtl","server":null},{"kind":"operator","icon":"/avatar/char_4145_ulpia","server":null},{"kind":"operator","icon":"/avatar/char_1034_jesca2","server":null},{"kind":"operator","icon":"/avatar/char_108_silent","server":null},{"kind":"operator","icon":"/avatar/char_235_jesica","server":null},{"kind":"operator","icon":"/avatar/char_4204_mantra","server":null},{"kind":"operator","icon":"/avatar/char_1031_slent2","server":null},{"kind":"operator","icon":"/avatar/char_1039_thorn2","server":null},{"kind":"story_sprite","icon":"/story-sprite-thumb/avg_npc_1581","server":null},{"kind":"story_sprite","icon":"/story-sprite-thumb/avg_npc_625","server":null},{"kind":"operator","icon":"/avatar/char_293_thorns","server":null},{"kind":"operator","icon":"/avatar/char_4229_aphris","server":"cn"},{"kind":"operator","icon":"/avatar/char_4009_irene","server":null},{"kind":"operator","icon":"/avatar/char_474_glady","server":null},{"kind":"event","icon":"/event-image/act39side","server":null},{"kind":"integrated_strategies","icon":"/assets/textures/spritepack/ui_zone_home_theme_rogue_5_entry_display_1/rogue_5_entry_display_1.png","server":null},{"kind":"event","icon":"/event-image/act2autochess","server":null},{"kind":"operator","icon":"/avatar/char_140_whitew","server":null}]};

const Slot = ({ children }: { children: ReactNode }) => <div className="w-80 p-4">{children}</div>;

export const Popular = () => (
    <Slot>
        <GridCard grid={ABOUT_ME} />
    </Slot>
);

/** A one-row board fills the thumbnail's width. */
export const OneRow = () => (
    <Slot>
        <GridCard grid={COMIC} />
    </Slot>
);

/** A template with no picks: the board's empty grey in every cell. */
export const EmptyTemplate = () => (
    <Slot>
        <GridCard grid={SKILLS} />
    </Slot>
);

/** As My grids draws it: the unlisted badge beside the size, edit and delete under the stats, and the kind chips capped at three. */
export const MyGridsCard = () => (
    <Slot>
        <GridCard
            grid={ABOUT_ME_WIDE}
            badge={
                <span className="inline-flex items-center gap-1 rounded-sm bg-muted px-1.5 py-0.5 font-mono text-[10px] text-muted-foreground uppercase tracking-wider">
                    <EyeOffIcon className="h-3 w-3" aria-hidden="true" />
                    Unlisted
                </span>
            }
            actions={
                <>
                    <Button render={<a href="/grids/about-me-4eqi31/edit" />} variant="outline" size="sm" className="flex-1">
                        <PencilIcon />
                        Edit
                    </Button>
                    <DeleteGridButton grid={ABOUT_ME_WIDE} />
                </>
            }
        />
    </Slot>
);
