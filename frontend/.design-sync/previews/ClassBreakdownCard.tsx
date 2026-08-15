import { ClassBreakdownCard } from "frontend";

interface ISub {
    subProfessionId: string;
    displayName: string;
    owned: number;
    total: number;
    percentage: number;
}

interface IProfession {
    profession: string;
    displayName: string;
    owned: number;
    total: number;
    percentage: number;
    subProfessions: ISub[];
}

const sub = (subProfessionId: string, displayName: string, owned: number, total: number): ISub => ({
    subProfessionId,
    displayName,
    owned,
    total,
    percentage: total > 0 ? (owned / total) * 100 : 0,
});

const prof = (profession: string, displayName: string, owned: number, total: number, subProfessions: ISub[]): IProfession => ({
    profession,
    displayName,
    owned,
    total,
    percentage: total > 0 ? (owned / total) * 100 : 0,
    subProfessions,
});

const FULL_ROSTER: IProfession[] = [
    prof("PIONEER", "Vanguard", 24, 29, [sub("pioneer", "Pioneer Vanguard", 6, 7), sub("charger", "Charger Vanguard", 5, 6), sub("tactician", "Tactician Vanguard", 5, 6), sub("bearer", "Standard Bearer Vanguard", 5, 6), sub("agent", "Agent Vanguard", 3, 4)]),
    prof("WARRIOR", "Guard", 47, 61, [sub("musha", "Musha Guard", 7, 9), sub("centurion", "Centurion Guard", 8, 10), sub("fighter", "Brawler Guard", 7, 9), sub("instructor", "Instructor Guard", 5, 7), sub("artsfghter", "Arts Fighter Guard", 9, 12), sub("sword", "Swordmaster Guard", 11, 14)]),
    prof("TANK", "Defender", 26, 33, [sub("protector", "Protector Defender", 9, 11), sub("guardian", "Guardian Defender", 6, 8), sub("fortress", "Fortress Defender", 5, 6), sub("duelist", "Duelist Defender", 6, 8)]),
    prof("SNIPER", "Sniper", 33, 43, [sub("fastshot", "Marksman Sniper", 10, 13), sub("aoesniper", "Artilleryman Sniper", 6, 8), sub("closerange", "Heavyshooter Sniper", 5, 7), sub("siegesniper", "Deadeye Sniper", 6, 7), sub("longrange", "Besieger Sniper", 6, 8)]),
    prof("CASTER", "Caster", 34, 44, [sub("corecaster", "Core Caster", 9, 11), sub("splashcaster", "Splash Caster", 7, 9), sub("funnel", "Mech-Accord Caster", 4, 5), sub("phalanx", "Phalanx Caster", 5, 7), sub("mystic", "Mystic Caster", 4, 6), sub("chain", "Chain Caster", 5, 6)]),
    prof("MEDIC", "Medic", 25, 32, [sub("physician", "Medic", 9, 11), sub("ringhealer", "Multi-target Medic", 6, 8), sub("healer", "Therapist Medic", 5, 6), sub("incantationmedic", "Incantation Medic", 5, 7)]),
    prof("SUPPORT", "Supporter", 20, 27, [sub("bard", "Bard Supporter", 4, 5), sub("craftsman", "Artificer Supporter", 3, 4), sub("blessing", "Abjurer Supporter", 4, 6), sub("summoner", "Summoner Supporter", 5, 6), sub("slower", "Decel Binder Supporter", 4, 6)]),
    prof("SPECIAL", "Specialist", 22, 30, [sub("pusher", "Push Stroker Specialist", 5, 6), sub("stalker", "Ambusher Specialist", 4, 5), sub("geek", "Geek Specialist", 3, 4), sub("merchant", "Merchant Specialist", 3, 5), sub("hookmaster", "Hookmaster Specialist", 3, 5), sub("executor", "Executor Specialist", 4, 5)]),
];

const EARLY_ROSTER: IProfession[] = FULL_ROSTER.map((p) => ({
    ...p,
    owned: Math.round(p.total * 0.16),
    percentage: 16,
    subProfessions: p.subProfessions.map((s) => ({ ...s, owned: Math.min(1, s.owned), percentage: (Math.min(1, s.owned) / s.total) * 100 })),
}));

export const FullRoster = () => (
    <div className="w-full max-w-3xl">
        <ClassBreakdownCard professions={FULL_ROSTER} />
    </div>
);

export const EarlyAccount = () => (
    <div className="w-full max-w-3xl">
        <ClassBreakdownCard professions={EARLY_ROSTER} />
    </div>
);

export const FrontlineOnly = () => (
    <div className="w-full max-w-3xl">
        <ClassBreakdownCard professions={FULL_ROSTER.slice(0, 4)} />
    </div>
);
