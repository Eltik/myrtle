import { Skill } from "../../../../../types/impl/lib/impl/local/impl/gamedata/impl/skills";

export default class Skills {
    public name: string;
    public skillData: Skill;

    // Attacks_Per_Second
    public attackSpeedModifiers: number[];
    public attackIntervalModifiers: number[];

    // Final_Attack
    public baseAttackModifiers: number[];
    public attackMultiplierModifiers: number[];
    public soraBuff: number[]; // We love Sora

    // Physical_Damage
    public flatDefModifiers: number[];
    public scalingDefModifiers: number[];
    public physTakenModifiers: number[];
    public extraPhysDamageDone: number[];

    // Arts_Damage
    public flatResModifiers: number[];
    public scalingResModifiers: number[];
    public artsTakenModifiers: number[];
    public extraArtsDamageDone: number[];

    // True_Damage
    public extraTrueDamageDone: number[];

    constructor(
        name: string,
        skillData: Skill,
        modifiers?: {
            attackSpeedModifiers?: number[];
            attackIntervalModifiers?: number[];
            baseAttackModifiers?: number[];
            attackMultiplierModifiers?: number[];
            soraBuff?: number[];
            flatDefModifiers?: number[];
            scalingDefModifiers?: number[];
            physTakenModifiers?: number[];
            extraPhysDamageDone?: number[];
            flatResModifiers?: number[];
            scalingResModifiers?: number[];
            artsTakenModifiers?: number[];
            extraArtsDamageDone?: number[];
            extraTrueDamageDone?: number[];
        },
    ) {
        this.name = name;
        this.skillData = skillData;

        this.attackSpeedModifiers = modifiers?.attackSpeedModifiers ?? [];
        this.attackIntervalModifiers = modifiers?.attackIntervalModifiers ?? [];
        this.baseAttackModifiers = modifiers?.baseAttackModifiers ?? [];
        this.attackMultiplierModifiers = modifiers?.attackMultiplierModifiers ?? [];
        this.soraBuff = modifiers?.soraBuff ?? [];
        this.flatDefModifiers = modifiers?.flatDefModifiers ?? [];
        this.scalingDefModifiers = modifiers?.scalingDefModifiers ?? [];
        this.physTakenModifiers = modifiers?.physTakenModifiers ?? [];
        this.extraPhysDamageDone = modifiers?.extraPhysDamageDone ?? [];
        this.flatResModifiers = modifiers?.flatResModifiers ?? [];
        this.scalingResModifiers = modifiers?.scalingResModifiers ?? [];
        this.artsTakenModifiers = modifiers?.artsTakenModifiers ?? [];
        this.extraArtsDamageDone = modifiers?.extraArtsDamageDone ?? [];
        this.extraTrueDamageDone = modifiers?.extraTrueDamageDone ?? [];
    }
}
