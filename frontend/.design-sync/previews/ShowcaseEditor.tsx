import { ShowcaseEditor } from "frontend";

// ShowcaseEditor is the owner's editor for the profile Showcase tab: the
// blocks in order (favourites, grid, tier list, plan), each a row with its
// type icon, name, move up/down and remove; a favourites block opens to its
// title and its picks; a gone block carries the Removed badge and is dropped
// on save; "Add a block" chooses a type. Seeded from the served ShowcaseView.
// Favourite entities are live EN catalogue rows
// (`/api/tier-lists/catalogue/<kind>`). The bundle has no signed-in user and
// its server functions are stubbed to fail, so a grid/tier-list block's
// title and the plan block's operator stay unresolved here.

const VIEW = {
    "blocks": [
        {
            "block": {
                "type": "favourites",
                "entity_kind": "operator",
                "ids": [
                    "char_003_kalts",
                    "char_1012_skadi2",
                    "char_4064_mlynar",
                    "char_293_thorns",
                    "char_291_aglina"
                ],
                "title": "Day-one carries"
            },
            "removed": false,
            "entities": [
                {
                    "id": "char_003_kalts",
                    "entity": {
                        "kind": "operator",
                        "id": "char_003_kalts",
                        "name": "Kal'tsit",
                        "icon": "/avatar/char_003_kalts",
                        "href": "/operators/char_003_kalts",
                        "facets": {
                            "appellation": " ",
                            "nation_id": "rhodes",
                            "nation_name": "Rhodes Island",
                            "position": "RANGED",
                            "profession": "MEDIC",
                            "profession_name": "Medic",
                            "rarity": "6",
                            "sub_profession_id": "physician",
                            "sub_profession_name": "Medic"
                        }
                    },
                    "entity_server": null
                },
                {
                    "id": "char_1012_skadi2",
                    "entity": {
                        "kind": "operator",
                        "id": "char_1012_skadi2",
                        "name": "Skadi the Corrupting Heart",
                        "icon": "/avatar/char_1012_skadi2",
                        "href": "/operators/char_1012_skadi2",
                        "facets": {
                            "appellation": " ",
                            "nation_id": "egir",
                            "nation_name": "Ægir",
                            "position": "RANGED",
                            "profession": "SUPPORT",
                            "profession_name": "Supporter",
                            "rarity": "6",
                            "sub_profession_id": "bard",
                            "sub_profession_name": "Bard"
                        }
                    },
                    "entity_server": null
                },
                {
                    "id": "char_4064_mlynar",
                    "entity": {
                        "kind": "operator",
                        "id": "char_4064_mlynar",
                        "name": "Młynar",
                        "icon": "/avatar/char_4064_mlynar",
                        "href": "/operators/char_4064_mlynar",
                        "facets": {
                            "appellation": " ",
                            "nation_id": "kazimierz",
                            "nation_name": "Kazimierz",
                            "position": "MELEE",
                            "profession": "WARRIOR",
                            "profession_name": "Guard",
                            "rarity": "6",
                            "sub_profession_id": "librator",
                            "sub_profession_name": "Liberator"
                        }
                    },
                    "entity_server": null
                },
                {
                    "id": "char_293_thorns",
                    "entity": {
                        "kind": "operator",
                        "id": "char_293_thorns",
                        "name": "Thorns",
                        "icon": "/avatar/char_293_thorns",
                        "href": "/operators/char_293_thorns",
                        "facets": {
                            "appellation": " ",
                            "nation_id": "iberia",
                            "nation_name": "Iberia",
                            "position": "MELEE",
                            "profession": "WARRIOR",
                            "profession_name": "Guard",
                            "rarity": "6",
                            "sub_profession_id": "lord",
                            "sub_profession_name": "Lord"
                        }
                    },
                    "entity_server": null
                },
                {
                    "id": "char_291_aglina",
                    "entity": {
                        "kind": "operator",
                        "id": "char_291_aglina",
                        "name": "Angelina",
                        "icon": "/avatar/char_291_aglina",
                        "href": "/operators/char_291_aglina",
                        "facets": {
                            "appellation": " ",
                            "nation_id": "siracusa",
                            "nation_name": "Siracusa",
                            "position": "RANGED",
                            "profession": "SUPPORT",
                            "profession_name": "Supporter",
                            "rarity": "6",
                            "sub_profession_id": "slower",
                            "sub_profession_name": "Decel Binder"
                        }
                    },
                    "entity_server": null
                }
            ]
        },
        {
            "block": {
                "type": "favourites",
                "entity_kind": "event",
                "ids": [
                    "act18d3",
                    "act31side"
                ]
            },
            "removed": false,
            "entities": [
                {
                    "id": "act18d3",
                    "entity": {
                        "kind": "event",
                        "id": "act18d3",
                        "name": "Under Tides",
                        "icon": "/event-image/act18d3",
                        "href": null,
                        "facets": {
                            "display_type": "BRANCHLINE",
                            "start_time": "1634828400",
                            "type": "TYPE_ACT9D0"
                        }
                    },
                    "entity_server": null
                },
                {
                    "id": "act31side",
                    "entity": {
                        "kind": "event",
                        "id": "act31side",
                        "name": "Here A People Sows",
                        "icon": "/event-image/act31side",
                        "href": null,
                        "facets": {
                            "display_type": "SIDESTORY",
                            "start_time": "1722427200",
                            "type": "TYPE_ACT9D0"
                        }
                    },
                    "entity_server": null
                }
            ]
        },
        {
            "block": {
                "type": "tier_list",
                "slug": "is5-core-picks"
            },
            "removed": true,
            "entities": []
        },
        {
            "block": {
                "type": "plan",
                "id": "plan-mlynar",
                "operator_id": "char_4064_mlynar"
            },
            "removed": false,
            "entities": []
        }
    ]
};

const noop = () => undefined;

/** Four blocks: two favourites, a deleted tier list, a plan. */
export const FourBlocks = () => (
    <div style={{ width: 860, minHeight: 640 }} className="relative">
        <ShowcaseEditor uid="18220561" view={VIEW as never} plansHidden={false} onClose={noop} />
    </div>
);

/** An empty showcase: the editor's starting state. */
export const Empty = () => (
    <div style={{ width: 860, minHeight: 400 }} className="relative">
        <ShowcaseEditor uid="18220561" view={{ blocks: [] }} plansHidden={false} onClose={noop} />
    </div>
);

/** Plans tab hidden: the plan block is flagged as not shown to visitors. */
export const PlansHidden = () => (
    <div style={{ width: 860, minHeight: 640 }} className="relative">
        <ShowcaseEditor uid="18220561" view={{ blocks: [VIEW.blocks[3], VIEW.blocks[0]] } as never} plansHidden onClose={noop} />
    </div>
);
