import { AttackType } from "../../../../../types/impl/lib/impl/dps-calculator";
import { OperatorProfession } from "../../../../../types/impl/lib/impl/local/impl/gamedata/impl/operators";

export default class Stats {
    public name: string;
    public def: number;
    public res: number;

    public operatorClass: OperatorProfession | null;
    public operatorArchetype: string | null;
    public hp: number | null;
    public atk: number | null;
    public dpCost: number | null;
    public deployTime: number | null;
    public blockCnt: number | null;
    public attackSpeed: number | null;
    public attackType: AttackType | null;

    constructor(
        name: string,
        def: number,
        res: number,
        operatorClass: OperatorProfession | null,
        operatorArchetype: string | null,
        hp: number | null,
        atk: number | null,
        dpCost: number | null,
        deployTime: number | null,
        blockCnt: number | null,
        attackSpeed: number | null,
        attackType: AttackType | null,
    ) {
        this.name = name;
        this.def = def;
        this.res = res;
        this.operatorClass = operatorClass;
        this.operatorArchetype = operatorArchetype;
        this.hp = hp;
        this.atk = atk;
        this.dpCost = dpCost;
        this.deployTime = deployTime;
        this.blockCnt = blockCnt;
        this.attackSpeed = attackSpeed;
        this.attackType = attackType;
    }
}
