import { Birthdays } from "frontend";

// The whole birthday tool page. It takes the operator index and parses
// `profile.basicInfo.dateOfBirth` itself, so the fixture carries the real
// handbook strings. The harness clock is fixed to 2024-05-15, which is why
// Tequila shows up in the "Today" callout.
type Row = [string, string, number, string, string, string];

const toOperator = ([id, name, rarity, profession, nationId, dateOfBirth]: Row) => ({
    id,
    name,
    rarity: `TIER_${rarity}`,
    profession,
    nationId,
    profile: { basicInfo: { dateOfBirth } },
});

const OPERATORS: Row[] = [
    ["char_1026_gvial2", "Gavial the Invincible", 6, "WARRIOR", "rhodes", "May 1"],
    ["char_193_frostl", "Frostleaf", 4, "WARRIOR", "rhodes", "May 1"],
    ["char_126_shotst", "Meteor", 4, "SNIPER", "kazimierz", "May 1"],
    ["char_301_cutter", "Cutter", 4, "WARRIOR", "columbia", "May 2"],
    ["char_343_tknogi", "Tsukinogi", 5, "SUPPORT", "higashi", "May 3"],
    ["char_179_cgbird", "Nightingale", 6, "MEDIC", "rhodes", "May 4"],
    ["char_458_rfrost", "Frost", 5, "SPECIAL", "lungmen", "May 4"],
    ["char_188_helage", "Hellagur", 6, "WARRIOR", "ursus", "May 5"],
    ["char_117_myrrh", "Myrrh", 4, "MEDIC", "rhodes", "May 5"],
    ["char_493_firwhl", "Firewhistle", 5, "TANK", "rim", "May 6"],
    ["char_4048_doroth", "Dorothy", 6, "SPECIAL", "columbia", "May 8"],
    ["char_401_elysm", "Elysium", 5, "PIONEER", "rhodes", "May 8"],
    ["char_346_aosta", "Aosta", 5, "SNIPER", "siracusa", "May 9"],
    ["char_484_robrta", "Roberta", 4, "SUPPORT", "columbia", "May 10"],
    ["char_297_hamoni", "Harmonie", 5, "CASTER", "victoria", "May 13"],
    ["char_291_aglina", "Angelina", 6, "SUPPORT", "siracusa", "May 14"],
    ["char_4082_qiubai", "Qiubai", 6, "WARRIOR", "yan", "May 14"],
    ["char_1029_yato2", "Kirin R Yato", 6, "SPECIAL", "rhodes", "May 14"],
    ["char_154_morgan", "Morgan", 5, "WARRIOR", "victoria", "May 14"],
    ["char_502_nblade", "Yato", 2, "PIONEER", "rhodes", "May 14"],
    ["char_486_takila", "Tequila", 5, "WARRIOR", "bolivar", "May 15"],
    ["char_1031_slent2", "Silence the Paradigmatic", 6, "SUPPORT", "columbia", "May 18"],
    ["char_107_liskam", "Liskarm", 5, "TANK", "columbia", "May 18"],
    ["char_199_yak", "Matterhorn", 4, "TANK", "kjerag", "May 19"],
    ["char_283_midn", "Midnight", 3, "WARRIOR", "rhodes", "May 20"],
    ["char_4193_lemuen", "Lemuen", 6, "SNIPER", "laterano", "May 21"],
    ["char_4138_narant", "Narantuya", 6, "SNIPER", "sargon", "May 26"],
    ["char_130_doberm", "Dobermann", 4, "WARRIOR", "rhodes", "May 27"],
    ["char_4148_philae", "Philae", 5, "TANK", "sargon", "May 29"],
    ["char_211_adnach", "Adnachiel", 3, "SNIPER", "rhodes", "May 30"],
    ["char_479_sleach", "Saileach", 6, "PIONEER", "victoria", "May 31"],
    ["char_1028_texas2", "Texas the Omertosa", 6, "SPECIAL", "lungmen", "Jun. 1"],
    ["char_102_texas", "Texas", 5, "PIONEER", "lungmen", "Jun. 1"],
    ["char_4116_blkkgt", "Degenbrecher", 6, "WARRIOR", "kjerag", "Jun. 6"],
    ["char_4042_lumen", "Lumen", 6, "MEDIC", "iberia", "Jun. 12"],
    ["char_010_chen", "Ch'en", 6, "WARRIOR", "lungmen", "Jul. 7"],
    ["char_112_siege", "Siege", 6, "PIONEER", "victoria", "Jul. 10"],
    ["char_003_kalts", "Kal'tsit", 6, "MEDIC", "rhodes", "Undisclosed"],
];

export const MonthCalendar = () => <Birthdays operators={OPERATORS.map(toOperator)} />;

export const NoOperatorsLoaded = () => <Birthdays operators={[]} />;
