import { ALL_ENEMIES } from "../..";
import emitter, { Events } from "../../../../../../../events";
import { enemies } from "../../../../../local/impl/gamedata";
import { getAll } from "../../../../../local/impl/gamedata/impl/enemies";
import EnemyUnit from "../../../classes/enemy-unit";
import { enemy_1000_gopro } from "./impl/enemy_1000_gopro";
import { enemy_1000_gopro_2 } from "./impl/enemy_1000_gopro_2";
import { enemy_1000_gopro_3 } from "./impl/enemy_1000_gopro_3";
import { enemy_1001_slime_2 } from "./impl/enemy_1001_slime_2";
import { enemy_1001_slime_3 } from "./impl/enemy_1001_slime_3";
import { enemy_1002_nsabr } from "./impl/enemy_1002_nsabr";
import { enemy_1003_ncbow } from "./impl/enemy_1003_ncbow";
import { enemy_1003_ncbow_2 } from "./impl/enemy_1003_ncbow_2";
import { enemy_1004_mslime } from "./impl/enemy_1004_mslime";
import { enemy_1004_mslime_2 } from "./impl/enemy_1004_mslime_2";
import { enemy_1005_yokai } from "./impl/enemy_1005_yokai";
import { enemy_1007_slime } from "./impl/enemy_1007_slime";
import { enemy_1008_ghost } from "./impl/enemy_1008_ghost";
import { enemy_1009_lurker } from "./impl/enemy_1009_lurker";
import { enemy_1011_wizard } from "./impl/enemy_1011_wizard";
import { enemy_1011_wizard_2 } from "./impl/enemy_1011_wizard_2";
import { enemy_1013_airdrp } from "./impl/enemy_1013_airdrp";
import { enemy_1013_airdrp_2 } from "./impl/enemy_1013_airdrp_2";
import { enemy_1014_rogue } from "./impl/enemy_1014_rogue";
import { enemy_1014_rogue_2 } from "./impl/enemy_1014_rogue_2";
import { enemy_1015_litamr } from "./impl/enemy_1015_litamr";
import { enemy_1015_litamr_2 } from "./impl/enemy_1015_litamr_2";
import { enemy_1016_diaman } from "./impl/enemy_1016_diaman";
import { enemy_1017_defdrn } from "./impl/enemy_1017_defdrn";
import { enemy_1019_jshoot } from "./impl/enemy_1019_jshoot";
import { enemy_1019_jshoot_2 } from "./impl/enemy_1019_jshoot_2";
import { enemy_1020_obsv } from "./impl/enemy_1020_obsv";
import { enemy_1021_bslime } from "./impl/enemy_1021_bslime";
import { enemy_1021_bslime_2 } from "./impl/enemy_1021_bslime_2";
import { enemy_1023_jmage } from "./impl/enemy_1023_jmage";
import { enemy_1023_jmage_2 } from "./impl/enemy_1023_jmage_2";
import { enemy_1024_mortar } from "./impl/enemy_1024_mortar";
import { enemy_1024_mortar_2 } from "./impl/enemy_1024_mortar_2";
import { enemy_1026_aghost } from "./impl/enemy_1026_aghost";
import { enemy_1027_mob } from "./impl/enemy_1027_mob";
import { enemy_1027_mob_2 } from "./impl/enemy_1027_mob_2";
import { enemy_1028_mocock } from "./impl/enemy_1028_mocock";
import { enemy_1028_mocock_2 } from "./impl/enemy_1028_mocock_2";
import { enemy_1029_shdsbr } from "./impl/enemy_1029_shdsbr";
import { enemy_1029_shdsbr_2 } from "./impl/enemy_1029_shdsbr_2";

// NOT FINISHED NEED TO DO enemy_1030_wteeth and onward
async function fetchEnemyData(enemyId: string) {
    const enemyData = await enemies(enemyId);
    if (!enemyData) throw new Error(`Enemy data not found for ${enemyId}`);

    let stats = null;

    switch (enemyId) {
        case "enemy_1007_slime":
            stats = enemy_1007_slime(enemyData);
            break;
        case "enemy_1001_slime_2":
            stats = enemy_1001_slime_2(enemyData);
            break;
        case "enemy_1001_slime_3":
            stats = enemy_1001_slime_3(enemyData);
            break;
        case "enemy_1004_mslime":
            stats = enemy_1004_mslime(enemyData);
            break;
        case "enemy_1004_mslime_2":
            stats = enemy_1004_mslime_2(enemyData);
            break;
        case "enemy_1021_bslime":
            stats = enemy_1021_bslime(enemyData);
            break;
        case "enemy_1021_bslime_2":
            stats = enemy_1021_bslime_2(enemyData);
            break;
        case "enemy_1000_gopro":
            stats = enemy_1000_gopro(enemyData);
            break;
        case "enemy_1000_gopro_2":
            stats = enemy_1000_gopro_2(enemyData);
            break;
        case "enemy_1000_gopro_3":
            stats = enemy_1000_gopro_3(enemyData);
            break;
        case "enemy_1002_nsabr":
            stats = enemy_1002_nsabr(enemyData);
            break;
        case "enemy_1003_ncbow":
            stats = enemy_1003_ncbow(enemyData);
            break;
        case "enemy_1003_ncbow_2":
            stats = enemy_1003_ncbow_2(enemyData);
            break;
        case "enemy_1011_wizard":
            stats = enemy_1011_wizard(enemyData);
            break;
        case "enemy_1011_wizard_2":
            stats = enemy_1011_wizard_2(enemyData);
            break;
        case "enemy_1013_airdrp":
            stats = enemy_1013_airdrp(enemyData);
            break;
        case "enemy_1013_airdrp_2":
            stats = enemy_1013_airdrp_2(enemyData);
            break;
        case "enemy_1024_mortar":
            stats = enemy_1024_mortar(enemyData);
            break;
        case "enemy_1024_mortar_2":
            stats = enemy_1024_mortar_2(enemyData);
            break;
        case "enemy_1014_rogue":
            stats = enemy_1014_rogue(enemyData);
            break;
        case "enemy_1014_rogue_2":
            stats = enemy_1014_rogue_2(enemyData);
            break;
        case "enemy_1015_litamr":
            stats = enemy_1015_litamr(enemyData);
            break;
        case "enemy_1015_litamr_2":
            stats = enemy_1015_litamr_2(enemyData);
            break;
        case "enemy_1016_diaman":
            stats = enemy_1016_diaman(enemyData);
            break;
        case "enemy_1009_lurker":
            stats = enemy_1009_lurker(enemyData);
            break;
        case "enemy_1008_ghost":
            stats = enemy_1008_ghost(enemyData);
            break;
        case "enemy_1005_yokai":
            stats = enemy_1005_yokai(enemyData);
            break;
        case "enemy_1017_defdrn":
            stats = enemy_1017_defdrn(enemyData);
            break;
        case "enemy_1019_jshoot":
            stats = enemy_1019_jshoot(enemyData);
            break;
        case "enemy_1019_jshoot_2":
            stats = enemy_1019_jshoot_2(enemyData);
            break;
        case "enemy_1023_jmage":
            stats = enemy_1023_jmage(enemyData);
            break;
        case "enemy_1023_jmage_2":
            stats = enemy_1023_jmage_2(enemyData);
            break;
        case "enemy_1020_obsv":
            stats = enemy_1020_obsv(enemyData);
            break;
        case "enemy_1026_aghost":
            stats = enemy_1026_aghost(enemyData);
            break;
        case "enemy_1027_mob":
            stats = enemy_1027_mob(enemyData);
            break;
        case "enemy_1027_mob_2":
            stats = enemy_1027_mob_2(enemyData);
            break;
        case "enemy_1028_mocock":
            stats = enemy_1028_mocock(enemyData);
            break;
        case "enemy_1028_mocock_2":
            stats = enemy_1028_mocock_2(enemyData);
            break;
        case "enemy_1029_shdsbr":
            stats = enemy_1029_shdsbr(enemyData);
            break;
        case "enemy_1029_shdsbr_2":
            stats = enemy_1029_shdsbr_2(enemyData);
            break;
        default:
            throw new Error(`Unknown enemy type for ${enemyId}`);
    }

    if (!stats) throw new Error(`Stats not found for ${enemyId}`);

    return new EnemyUnit(enemyData, stats, []);
}

export default async function fetchNormalEnemies() {
    const enemyIds = Object.values(getAll().enemyData)
        .filter((enemy) => enemy.enemyLevel === "NORMAL")
        .filter((enemy) => enemy)
        .map((enemy) => enemy.enemyId);
    const enemyData = await Promise.all(enemyIds.map(fetchEnemyData));

    await emitter.emit(Events.DPS_ENEMY_CLASS_FETCHED, {
        name: "Normal",
    });

    ALL_ENEMIES.push(...enemyData);
}
