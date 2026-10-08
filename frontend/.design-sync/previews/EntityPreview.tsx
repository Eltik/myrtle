import { EntityPreview } from "frontend";
import type { ReactNode } from "react";

// The hover card of a non-operator tier-list tile: its art, its kind, its name,
// a detail line (what kind of thing it is), the author's note, and an "open
// page" hint when the site links it. A wide kind (an event) leads with its
// banner. Rendered here inside the popup surface HoverCardContent gives it.
// Entities are live `/api/tier-lists/<slug>` placements mapped through `toTierEntity`.

const THORNS_SKIN = {"key":"skin:char_293_thorns@boc#8","id":"char_293_thorns@boc#8","name":"Blade-cleaved Tides","icon":"/avatar/char_293_thorns_boc%238","href":"/operators/char_293_thorns","facets":{"brand":"Bloodline of Combat","char_id":"char_293_thorns","operator":"Thorns","profession":"WARRIOR","rarity":"6"},"subOrder":0,"description":null,"updatedAt":"2024-05-14T09:00:00.000Z","kind":"skin","resolved":true,"charId":"char_293_thorns","operatorName":"Thorns","rarity":6,"profession":"WARRIOR","brand":"Bloodline of Combat"} as const;
const CENTURION = {"key":"enemy:enemy_1501_demonk","id":"enemy_1501_demonk","name":"Sarkaz Centurion","icon":"/enemy-icon/enemy_1501_demonk","href":"/enemies/enemy_1501_demonk","facets":{"enemy_index":"SC","enemy_level":"BOSS"},"subOrder":0,"description":null,"updatedAt":"2024-05-14T09:00:00.000Z","kind":"enemy","resolved":true,"level":"BOSS","index":"SC"} as const;
const SHATTERPOINT = {"key":"main_story:main_10","id":"main_10","name":"Shatterpoint","icon":"/assets/textures/spritepack/mixstory_kv_sprites_1/kv_shatterpoint.png","href":null,"facets":{"act":"2","act_name":"Shadow of A Dying Sun","episode":"10"},"subOrder":0,"description":null,"updatedAt":"2024-05-14T09:00:00.000Z","kind":"main_story","resolved":true,"episode":10,"act":2,"actName":"Shadow of A Dying Sun"} as const;
const RHODES = {"key":"faction:rhodes","id":"rhodes","name":"Rhodes Island","icon":"/assets/textures/spritepack/ui_camp_logo_0/logo_rhodes.png","href":null,"facets":{"power_level":"nation"},"subOrder":0,"description":null,"updatedAt":"2024-05-14T09:00:00.000Z","kind":"faction","resolved":true,"powerLevel":"nation"} as const;
const PALE_SEA = {"key":"event:act39side","id":"act39side","name":"Exodus from the Pale Sea","icon":"/event-image/act39side","href":null,"facets":{"display_type":"SIDESTORY","start_time":"1749124800","type":"TYPE_ACT9D0"},"subOrder":0,"description":null,"updatedAt":"2024-05-14T09:00:00.000Z","kind":"event","resolved":true,"displayType":"SIDESTORY","startTime":1749124800,"rerun":false} as const;

const CENTURION_NOTED = { ...CENTURION, description: "Hits like a truck and walks straight through a single blocker. Bring a second defender or a reliable sleep, or lose the lane by wave 3." };

/** The hover card's popup surface. */
const Popup = ({ children }: { children: ReactNode }) => (
    <div className="p-6">
        <div className="w-max overflow-hidden rounded-lg border border-border bg-popover text-popover-foreground shadow-lg">{children}</div>
    </div>
);

/** A boss enemy with an author's note and a link to its handbook page. */
export const EnemyWithNote = () => (
    <Popup>
        <EntityPreview entity={CENTURION_NOTED} linked />
    </Popup>
);

/** A skin: the wearer and brand in the detail line, linked to the operator page. */
export const Skin = () => (
    <Popup>
        <EntityPreview entity={THORNS_SKIN} linked />
    </Popup>
);

/** An event leads with its banner; events have no page, so no hint. */
export const EventBanner = () => (
    <Popup>
        <EntityPreview entity={PALE_SEA} linked={false} />
    </Popup>
);

/** A faction glyph and a main story episode, unlinked. */
export const FactionAndStory = () => (
    <div className="flex flex-wrap items-start gap-4">
        <Popup>
            <EntityPreview entity={RHODES} linked={false} />
        </Popup>
        <Popup>
            <EntityPreview entity={SHATTERPOINT} linked={false} />
        </Popup>
    </div>
);
