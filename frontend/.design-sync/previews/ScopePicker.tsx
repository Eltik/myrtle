import { ScopePicker } from "frontend";
import { useState } from "react";

// ScopePicker is the user search's "Owns every" scope: eight class icons in a
// row (tooltip names), then, once a class is chosen, its archetypes as chips
// with "Whole class" first. A class press selects the whole class (pressed
// again it clears); an archetype chip narrows to it. Names come from the
// operator index; the fixture is one real obtainable operator per archetype
// from `/api/operators/index` (71 archetypes), which is all the picker reads.

type Scope = { kind: "class"; profession: string } | { kind: "sub"; subProfessionId: string };

const OPERATORS = [
    {"id": "char_225_haak", "name": "Aak", "appellation": " ", "rarity": 6, "profession": "SPECIAL", "subProfessionId": "geek", "position": "RANGED", "nationId": "lungmen", "isNotObtainable": false, "professionName": "Specialist", "subProfessionName": "Geek"},
    {"id": "char_291_aglina", "name": "Angelina", "appellation": " ", "rarity": 6, "profession": "SUPPORT", "subProfessionId": "slower", "position": "RANGED", "nationId": "siracusa", "isNotObtainable": false, "professionName": "Supporter", "subProfessionName": "Decel Binder"},
    {"id": "char_332_archet", "name": "Archetto", "appellation": " ", "rarity": 6, "profession": "SNIPER", "subProfessionId": "fastshot", "position": "RANGED", "nationId": "laterano", "isNotObtainable": false, "professionName": "Sniper", "subProfessionName": "Marksman"},
    {"id": "char_4132_ascln", "name": "Ascalon", "appellation": " ", "rarity": 6, "profession": "SPECIAL", "subProfessionId": "stalker", "position": "MELEE", "nationId": "rhodes", "isNotObtainable": false, "professionName": "Specialist", "subProfessionName": "Ambusher"},
    {"id": "char_222_bpipe", "name": "Bagpipe", "appellation": " ", "rarity": 6, "profession": "PIONEER", "subProfessionId": "charger", "position": "MELEE", "nationId": "victoria", "isNotObtainable": false, "professionName": "Vanguard", "subProfessionName": "Charger"},
    {"id": "char_4037_demetr", "name": "Bellone", "appellation": " ", "rarity": 6, "profession": "WARRIOR", "subProfessionId": "fighter", "position": "MELEE", "nationId": "siracusa", "isNotObtainable": false, "professionName": "Guard", "subProfessionName": "Fighter"},
    {"id": "char_017_huang", "name": "Blaze", "appellation": " ", "rarity": 6, "profession": "WARRIOR", "subProfessionId": "centurion", "position": "MELEE", "nationId": "rhodes", "isNotObtainable": false, "professionName": "Guard", "subProfessionName": "Centurion"},
    {"id": "char_1040_blaze2", "name": "Blaze the Igniting Spark", "appellation": " ", "rarity": 6, "profession": "CASTER", "subProfessionId": "primcaster", "position": "RANGED", "nationId": "rhodes", "isNotObtainable": false, "professionName": "Caster", "subProfessionName": "Primal Caster"},
    {"id": "char_423_blemsh", "name": "Blemishine", "appellation": " ", "rarity": 6, "profession": "TANK", "subProfessionId": "guardian", "position": "MELEE", "nationId": "kazimierz", "isNotObtainable": false, "professionName": "Defender", "subProfessionName": "Guardian"},
    {"id": "char_426_billro", "name": "Carnelian", "appellation": " ", "rarity": 6, "profession": "CASTER", "subProfessionId": "phalanx", "position": "RANGED", "nationId": "leithanien", "isNotObtainable": false, "professionName": "Caster", "subProfessionName": "Phalanx Caster"},
    {"id": "char_2013_cerber", "name": "Ceobe", "appellation": " ", "rarity": 6, "profession": "CASTER", "subProfessionId": "corecaster", "position": "RANGED", "nationId": "rhodes", "isNotObtainable": false, "professionName": "Caster", "subProfessionName": "Core Caster"},
    {"id": "char_010_chen", "name": "Ch'en", "appellation": " ", "rarity": 6, "profession": "WARRIOR", "subProfessionId": "sword", "position": "MELEE", "nationId": "lungmen", "isNotObtainable": false, "professionName": "Guard", "subProfessionName": "Swordmaster"},
    {"id": "char_1050_chen3", "name": "Ch'en the Dawnstreak", "appellation": " ", "rarity": 6, "profession": "WARRIOR", "subProfessionId": "artsfghter", "position": "MELEE", "nationId": "yan", "isNotObtainable": false, "professionName": "Guard", "subProfessionName": "Arts Fighter"},
    {"id": "char_1013_chen2", "name": "Ch'en the Holungday", "appellation": " ", "rarity": 6, "profession": "SNIPER", "subProfessionId": "reaperrange", "position": "RANGED", "nationId": "rhodes", "isNotObtainable": false, "professionName": "Sniper", "subProfessionName": "Spreadshooter"},
    {"id": "char_4134_cetsyr", "name": "Civilight Eterna", "appellation": " ", "rarity": 6, "profession": "SUPPORT", "subProfessionId": "bard", "position": "RANGED", "nationId": "rhodes", "isNotObtainable": false, "professionName": "Supporter", "subProfessionName": "Bard"},
    {"id": "char_1502_crosly", "name": "Crownslayer", "appellation": " ", "rarity": 6, "profession": "SPECIAL", "subProfessionId": "executor", "position": "MELEE", "nationId": "rhodes", "isNotObtainable": false, "professionName": "Specialist", "subProfessionName": "Executor"},
    {"id": "char_4048_doroth", "name": "Dorothy", "appellation": " ", "rarity": 6, "profession": "SPECIAL", "subProfessionId": "traper", "position": "RANGED", "nationId": "columbia", "isNotObtainable": false, "professionName": "Specialist", "subProfessionName": "Trapmaster"},
    {"id": "char_2015_dusk", "name": "Dusk", "appellation": " ", "rarity": 6, "profession": "CASTER", "subProfessionId": "splashcaster", "position": "RANGED", "nationId": "yan", "isNotObtainable": false, "professionName": "Caster", "subProfessionName": "Splash Caster"},
    {"id": "char_4046_ebnhlz", "name": "Ebenholz", "appellation": " ", "rarity": 6, "profession": "CASTER", "subProfessionId": "mystic", "position": "RANGED", "nationId": "leithanien", "isNotObtainable": false, "professionName": "Caster", "subProfessionName": "Mystic Caster"},
    {"id": "char_4010_etlchi", "name": "Entelechia", "appellation": " ", "rarity": 6, "profession": "WARRIOR", "subProfessionId": "reaper", "position": "MELEE", "nationId": "rhodes", "isNotObtainable": false, "professionName": "Guard", "subProfessionName": "Reaper"},
    {"id": "char_416_zumama", "name": "Eunectes", "appellation": " ", "rarity": 6, "profession": "TANK", "subProfessionId": "duelist", "position": "MELEE", "nationId": "sargon", "isNotObtainable": false, "professionName": "Defender", "subProfessionName": "Duelist"},
    {"id": "char_1016_agoat2", "name": "Eyjafjalla the Hvít Aska", "appellation": " ", "rarity": 6, "profession": "MEDIC", "subProfessionId": "wandermedic", "position": "RANGED", "nationId": "leithanien", "isNotObtainable": false, "professionName": "Medic", "subProfessionName": "Wandering Medic"},
    {"id": "char_430_fartth", "name": "Fartooth", "appellation": " ", "rarity": 6, "profession": "SNIPER", "subProfessionId": "longrange", "position": "RANGED", "nationId": "kazimierz", "isNotObtainable": false, "professionName": "Sniper", "subProfessionName": "Deadeye"},
    {"id": "char_300_phenxi", "name": "Fiammetta", "appellation": " ", "rarity": 6, "profession": "SNIPER", "subProfessionId": "aoesniper", "position": "RANGED", "nationId": "laterano", "isNotObtainable": false, "professionName": "Sniper", "subProfessionName": "Artilleryman"},
    {"id": "char_420_flamtl", "name": "Flametail", "appellation": " ", "rarity": 6, "profession": "PIONEER", "subProfessionId": "pioneer", "position": "MELEE", "nationId": "kazimierz", "isNotObtainable": false, "professionName": "Vanguard", "subProfessionName": "Pioneer"},
    {"id": "char_474_glady", "name": "Gladiia", "appellation": " ", "rarity": 6, "profession": "SPECIAL", "subProfessionId": "hookmaster", "position": "MELEE", "nationId": "egir", "isNotObtainable": false, "professionName": "Specialist", "subProfessionName": "Hookmaster"},
    {"id": "char_206_gnosis", "name": "Gnosis", "appellation": " ", "rarity": 6, "profession": "SUPPORT", "subProfessionId": "underminer", "position": "RANGED", "nationId": "kjerag", "isNotObtainable": false, "professionName": "Supporter", "subProfessionName": "Hexer"},
    {"id": "char_377_gdglow", "name": "Goldenglow", "appellation": " ", "rarity": 6, "profession": "CASTER", "subProfessionId": "funnel", "position": "RANGED", "nationId": "victoria", "isNotObtainable": false, "professionName": "Caster", "subProfessionName": "Mech-accord Caster"},
    {"id": "char_4202_haruka", "name": "Haruka", "appellation": " ", "rarity": 6, "profession": "SUPPORT", "subProfessionId": "blessing", "position": "RANGED", "nationId": "higashi", "isNotObtainable": false, "professionName": "Supporter", "subProfessionName": "Abjurer"},
    {"id": "char_188_helage", "name": "Hellagur", "appellation": " ", "rarity": 6, "profession": "WARRIOR", "subProfessionId": "musha", "position": "MELEE", "nationId": "ursus", "isNotObtainable": false, "professionName": "Guard", "subProfessionName": "Soloblade"},
    {"id": "char_4088_hodrer", "name": "Hoederer", "appellation": " ", "rarity": 6, "profession": "WARRIOR", "subProfessionId": "crusher", "position": "MELEE", "nationId": "", "isNotObtainable": false, "professionName": "Guard", "subProfessionName": "Crusher"},
    {"id": "char_4039_horn", "name": "Horn", "appellation": " ", "rarity": 6, "profession": "TANK", "subProfessionId": "fortress", "position": "MELEE", "nationId": "victoria", "isNotObtainable": false, "professionName": "Defender", "subProfessionName": "Fortress"},
    {"id": "char_136_hsguma", "name": "Hoshiguma", "appellation": " ", "rarity": 6, "profession": "TANK", "subProfessionId": "protector", "position": "MELEE", "nationId": "lungmen", "isNotObtainable": false, "professionName": "Defender", "subProfessionName": "Protector"},
    {"id": "char_1044_hsgma2", "name": "Hoshiguma the Breacher", "appellation": " ", "rarity": 6, "profession": "TANK", "subProfessionId": "artsprotector", "position": "MELEE", "nationId": "lungmen", "isNotObtainable": false, "professionName": "Defender", "subProfessionName": "Arts Protector"},
    {"id": "char_134_ifrit", "name": "Ifrit", "appellation": " ", "rarity": 6, "profession": "CASTER", "subProfessionId": "blastcaster", "position": "RANGED", "nationId": "columbia", "isNotObtainable": false, "professionName": "Caster", "subProfessionName": "Blast Caster"},
    {"id": "char_4087_ines", "name": "Ines", "appellation": " ", "rarity": 6, "profession": "PIONEER", "subProfessionId": "agent", "position": "MELEE", "nationId": "", "isNotObtainable": false, "professionName": "Vanguard", "subProfessionName": "Agent"},
    {"id": "char_1034_jesca2", "name": "Jessica the Liberated", "appellation": " ", "rarity": 6, "profession": "TANK", "subProfessionId": "shotprotector", "position": "MELEE", "nationId": "columbia", "isNotObtainable": false, "professionName": "Defender", "subProfessionName": "Sentry Protector"},
    {"id": "char_003_kalts", "name": "Kal'tsit", "appellation": " ", "rarity": 6, "profession": "MEDIC", "subProfessionId": "physician", "position": "RANGED", "nationId": "rhodes", "isNotObtainable": false, "professionName": "Medic", "subProfessionName": "Medic"},
    {"id": "char_322_lmlee", "name": "Lee", "appellation": " ", "rarity": 6, "profession": "SPECIAL", "subProfessionId": "merchant", "position": "MELEE", "nationId": "lungmen", "isNotObtainable": false, "professionName": "Specialist", "subProfessionName": "Merchant"},
    {"id": "char_1043_leizi2", "name": "Leizi the Thunderbringer", "appellation": " ", "rarity": 6, "profession": "WARRIOR", "subProfessionId": "librator", "position": "MELEE", "nationId": "yan", "isNotObtainable": false, "professionName": "Guard", "subProfessionName": "Liberator"},
    {"id": "char_4011_lessng", "name": "Lessing", "appellation": " ", "rarity": 6, "profession": "WARRIOR", "subProfessionId": "fearless", "position": "MELEE", "nationId": "leithanien", "isNotObtainable": false, "professionName": "Guard", "subProfessionName": "Dreadnought"},
    {"id": "char_2023_ling", "name": "Ling", "appellation": " ", "rarity": 6, "profession": "SUPPORT", "subProfessionId": "summoner", "position": "RANGED", "nationId": "yan", "isNotObtainable": false, "professionName": "Supporter", "subProfessionName": "Summoner"},
    {"id": "char_4042_lumen", "name": "Lumen", "appellation": " ", "rarity": 6, "profession": "MEDIC", "subProfessionId": "healer", "position": "RANGED", "nationId": "iberia", "isNotObtainable": false, "professionName": "Medic", "subProfessionName": "Therapist"},
    {"id": "char_4179_monstr", "name": "Mon3tr", "appellation": " ", "rarity": 6, "profession": "MEDIC", "subProfessionId": "chainhealer", "position": "RANGED", "nationId": "rhodes", "isNotObtainable": false, "professionName": "Medic", "subProfessionName": "Chain Medic"},
    {"id": "char_311_mudrok", "name": "Mudrock", "appellation": " ", "rarity": 6, "profession": "TANK", "subProfessionId": "unyield", "position": "MELEE", "nationId": "rhodes", "isNotObtainable": false, "professionName": "Defender", "subProfessionName": "Juggernaut"},
    {"id": "char_249_mlyss", "name": "Muelsyse", "appellation": " ", "rarity": 6, "profession": "PIONEER", "subProfessionId": "tactician", "position": "RANGED", "nationId": "columbia", "isNotObtainable": false, "professionName": "Vanguard", "subProfessionName": "Tactician"},
    {"id": "char_4138_narant", "name": "Narantuya", "appellation": " ", "rarity": 6, "profession": "SNIPER", "subProfessionId": "loopshooter", "position": "RANGED", "nationId": "sargon", "isNotObtainable": false, "professionName": "Sniper", "subProfessionName": "Loopshooter"},
    {"id": "char_4212_nasti", "name": "Nasti", "appellation": " ", "rarity": 6, "profession": "SUPPORT", "subProfessionId": "craftsman", "position": "MELEE", "nationId": "columbia", "isNotObtainable": false, "professionName": "Supporter", "subProfessionName": "Artificer"},
    {"id": "char_450_necras", "name": "Necrass", "appellation": " ", "rarity": 6, "profession": "CASTER", "subProfessionId": "soulcaster", "position": "RANGED", "nationId": "victoria", "isNotObtainable": false, "professionName": "Caster", "subProfessionName": "Shaper Caster"},
    {"id": "char_179_cgbird", "name": "Nightingale", "appellation": " ", "rarity": 6, "profession": "MEDIC", "subProfessionId": "ringhealer", "position": "RANGED", "nationId": "", "isNotObtainable": false, "professionName": "Medic", "subProfessionName": "Multi-target Medic"},
    {"id": "char_485_pallas", "name": "Pallas", "appellation": " ", "rarity": 6, "profession": "WARRIOR", "subProfessionId": "instructor", "position": "MELEE", "nationId": "minos", "isNotObtainable": false, "professionName": "Guard", "subProfessionName": "Instructor"},
    {"id": "char_472_pasngr", "name": "Passenger", "appellation": " ", "rarity": 6, "profession": "CASTER", "subProfessionId": "chain", "position": "RANGED", "nationId": "sargon", "isNotObtainable": false, "professionName": "Caster", "subProfessionName": "Chain Caster"},
    {"id": "char_4058_pepe", "name": "Pepe", "appellation": " ", "rarity": 6, "profession": "WARRIOR", "subProfessionId": "hammer", "position": "MELEE", "nationId": "sargon", "isNotObtainable": false, "professionName": "Guard", "subProfessionName": "Earthshaker"},
    {"id": "char_4055_bgsnow", "name": "Pozëmka", "appellation": " ", "rarity": 6, "profession": "SNIPER", "subProfessionId": "closerange", "position": "RANGED", "nationId": "rhodes", "isNotObtainable": false, "professionName": "Sniper", "subProfessionName": "Heavyshooter"},
    {"id": "char_4082_qiubai", "name": "Qiubai", "appellation": " ", "rarity": 6, "profession": "WARRIOR", "subProfessionId": "lord", "position": "MELEE", "nationId": "yan", "isNotObtainable": false, "professionName": "Guard", "subProfessionName": "Lord"},
    {"id": "char_4117_ray", "name": "Ray", "appellation": " ", "rarity": 6, "profession": "SNIPER", "subProfessionId": "hunter", "position": "RANGED", "nationId": "rim", "isNotObtainable": false, "professionName": "Sniper", "subProfessionName": "Hunter"},
    {"id": "char_1020_reed2", "name": "Reed the Flame Shadow", "appellation": " ", "rarity": 6, "profession": "MEDIC", "subProfessionId": "incantationmedic", "position": "RANGED", "nationId": "victoria", "isNotObtainable": false, "professionName": "Medic", "subProfessionName": "Incantation Medic"},
    {"id": "char_197_poca", "name": "Rosa", "appellation": "Роса", "rarity": 6, "profession": "SNIPER", "subProfessionId": "siegesniper", "position": "RANGED", "nationId": "ursus", "isNotObtainable": false, "professionName": "Sniper", "subProfessionName": "Besieger"},
    {"id": "char_391_rosmon", "name": "Rosmontis", "appellation": " ", "rarity": 6, "profession": "SNIPER", "subProfessionId": "bombarder", "position": "RANGED", "nationId": "rhodes", "isNotObtainable": false, "professionName": "Sniper", "subProfessionName": "Flinger"},
    {"id": "char_479_sleach", "name": "Saileach", "appellation": " ", "rarity": 6, "profession": "PIONEER", "subProfessionId": "bearer", "position": "MELEE", "nationId": "victoria", "isNotObtainable": false, "professionName": "Vanguard", "subProfessionName": "Standard Bearer"},
    {"id": "char_1045_svash2", "name": "SilverAsh the Reignfrost", "appellation": " ", "rarity": 6, "profession": "PIONEER", "subProfessionId": "counsellor", "position": "MELEE", "nationId": "kjerag", "isNotObtainable": false, "professionName": "Vanguard", "subProfessionName": "Strategist"},
    {"id": "char_1023_ghost2", "name": "Specter the Unchained", "appellation": " ", "rarity": 6, "profession": "SPECIAL", "subProfessionId": "dollkeeper", "position": "MELEE", "nationId": "egir", "isNotObtainable": false, "professionName": "Specialist", "subProfessionName": "Dollkeeper"},
    {"id": "char_1039_thorn2", "name": "Thorns the Lodestar", "appellation": " ", "rarity": 6, "profession": "SPECIAL", "subProfessionId": "alchemist", "position": "RANGED", "nationId": "iberia", "isNotObtainable": false, "professionName": "Specialist", "subProfessionName": "Alchemist"},
    {"id": "char_1042_phatm2", "name": "Tragodia", "appellation": " ", "rarity": 6, "profession": "SUPPORT", "subProfessionId": "ritualist", "position": "RANGED", "nationId": "victoria", "isNotObtainable": false, "professionName": "Supporter", "subProfessionName": "Ritualist"},
    {"id": "char_400_weedy", "name": "Weedy", "appellation": " ", "rarity": 6, "profession": "SPECIAL", "subProfessionId": "pusher", "position": "MELEE", "nationId": "rhodes", "isNotObtainable": false, "professionName": "Specialist", "subProfessionName": "Push Stroker"},
    {"id": "char_2026_yu", "name": "Yu", "appellation": " ", "rarity": 6, "profession": "TANK", "subProfessionId": "primprotector", "position": "MELEE", "nationId": "yan", "isNotObtainable": false, "professionName": "Defender", "subProfessionName": "Primal Protector"},
    {"id": "char_4187_graceb", "name": "Gracebearer", "appellation": " ", "rarity": 5, "profession": "WARRIOR", "subProfessionId": "primguard", "position": "MELEE", "nationId": "bolivar", "isNotObtainable": false, "professionName": "Guard", "subProfessionName": "Primal Guard"},
    {"id": "char_394_hadiya", "name": "Hadiya", "appellation": " ", "rarity": 5, "profession": "WARRIOR", "subProfessionId": "mercenary", "position": "MELEE", "nationId": "sargon", "isNotObtainable": false, "professionName": "Guard", "subProfessionName": "Mercenary"},
    {"id": "char_4213_skybx", "name": "Skybox", "appellation": " ", "rarity": 5, "profession": "SNIPER", "subProfessionId": "skybreaker", "position": "RANGED", "nationId": "columbia", "isNotObtainable": false, "professionName": "Sniper", "subProfessionName": "Skybreaker"},
    {"id": "char_4222_taraxa", "name": "Taraxacum", "appellation": " ", "rarity": 5, "profession": "MEDIC", "subProfessionId": "watchman", "position": "RANGED", "nationId": "yan", "isNotObtainable": false, "professionName": "Medic", "subProfessionName": "Watchman"},
    {"id": "char_4191_tippi", "name": "Tippi", "appellation": " ", "rarity": 5, "profession": "SPECIAL", "subProfessionId": "skywalker", "position": "MELEE", "nationId": "columbia", "isNotObtainable": false, "professionName": "Specialist", "subProfessionName": "Skyranger"},
];

function Stage({ initial, operators = OPERATORS, loading = false }: { initial: Scope | null; operators?: typeof OPERATORS; loading?: boolean }) {
    const [scope, setScope] = useState<Scope | null>(initial);
    return (
        <div style={{ width: 360 }} className="rounded-xl border bg-popover p-3">
            <ScopePicker scope={scope} onChange={setScope} operators={(loading ? undefined : operators) as never} />
        </div>
    );
}

/** Nothing set: the eight classes alone. */
export const NotSet = () => <Stage initial={null} />;

/** A whole class chosen (Guard): its archetype chips, "Whole class" checked. */
export const WholeClass = () => <Stage initial={{ kind: "class", profession: "WARRIOR" }} />;

/** One archetype chosen (Bard): its class lit, the chip checked. */
export const Archetype = () => <Stage initial={{ kind: "sub", subProfessionId: "bard" }} />;

/** A class chosen before the operator index loads. */
export const IndexLoading = () => <Stage initial={{ kind: "class", profession: "CASTER" }} loading />;
