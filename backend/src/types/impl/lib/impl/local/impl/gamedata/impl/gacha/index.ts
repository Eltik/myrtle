export enum GachaRuleType {
    NORMAL = "NORMAL",
    LIMITED = "LIMITED",
    LINKAGE = "LINKAGE",
    ATTAIN = "ATTAIN",
    CLASSIC = "CLASSIC",
    SINGLE = "SINGLE",
    FESCLASSIC = "FESCLASSIC",
    CLASSIC_ATTAIN = "CLASSIC_ATTAIN",
    SPECIAL = "SPECIAL",
}

export enum PotentialMaterialType {
    LGG_SHD = "LGG_SHD",
    HGG_SHD = "HGG_SHD",
}

export enum ClassicPotentialMaterialType {
    CLASSIC_SHD = "CLASSIC_SHD",
}

export interface GachaPoolClient {
    gachaPoolId: string;
    gachaIndex: number;
    openTime: number;
    endTime: number;
    gachaPoolName: string;
    gachaPoolSummary: string;
    gachaPoolDetail: string;
    guarantee5Avail: number;
    guarantee5Count: number;
    LMTGSID: string | null;
    CDPrimColor: string | null;
    CDSecColor: string | null;
    freeBackColor: string | null;
    gachaRuleType: GachaRuleType;
    dynMeta: any | null; // Type this more strictly if the structure is known (seems complex based on limited view)
    linkageRuleId: string | null;
    linkageParam: any | null; // Type this more strictly if the structure is known
    limitParam: any | null; // Type this more strictly if the structure is known
}

export interface NewbeeGachaPoolClient {
    gachaPoolId: string;
    gachaIndex: number;
    gachaPoolName: string;
    gachaPoolDetail: string;
    gachaPrice: number;
    gachaTimes: number;
    gachaOffset: string; // Represented as string in JSON e.g., "0.35"
}

export interface GachaTag {
    tagId: number;
    tagName: string;
    tagGroup: number;
    tagCat?: string;
}

export interface RecruitTimeTableEntry {
    timeLength: number;
    recruitPrice: number;
}

export interface RecruitConstants {
    tagPriceList: Record<string, number>;
    maxRecruitTime: number;
}

export interface RecruitPool {
    recruitTimeTable: RecruitTimeTableEntry[];
    recruitConstants: RecruitConstants;
}

export interface PotentialMaterialItem {
    id: string;
    count: number;
    type: PotentialMaterialType;
}

export interface PotentialMaterialConverter {
    items: Record<string, PotentialMaterialItem>;
}

export interface ClassicPotentialMaterialItem {
    id: string; // e.g., "classic_normal_ticket"
    count: number;
    type: ClassicPotentialMaterialType;
}

export interface ClassicPotentialMaterialConverter {
    items: Record<string, ClassicPotentialMaterialItem>;
}

export interface RecruitRarityTableEntry {
    rarityStart: number;
    rarityEnd: number;
}

export interface CarouselEntry {
    poolId: string;
    index: number;
    startTime: number;
    endTime: number;
    spriteId: string;
}

export interface FreeGachaEntry {
    poolId: string;
    openTime: number;
    endTime: number;
    freeCount: number;
}

export interface LimitTenGachaItemEntry {
    itemId: string;
    endTime: number;
}

export interface LinkageTenGachaItemEntry {
    itemId: string;
    endTime: number;
    gachaPoolId: string;
}

export interface NormalGachaItemEntry {
    itemId: string;
    endTime: number;
    gachaPoolId: string;
    isTen: boolean;
}

export interface FesGachaPoolRelateItemEntry {
    rarityRank5ItemId: string;
    rarityRank6ItemId: string;
}

export interface GachaTable {
    gachaPoolClient: GachaPoolClient[];
    newbeeGachaPoolClient: NewbeeGachaPoolClient[];
    // The following keys were either empty or not present in the provided JSON.
    // Define specific types if their structure is known and they can contain data.
    termGachaPoolClient: any[];
    limitGachaPoolClient: any[];
    freeGachaPoolClient: any[]; // Note: This seems distinct from 'freeGacha'
    linkageGachaPoolClient: any[];
    spGachaPoolClient: any[];
    classicGachaPoolClient: any[];
    fesClassicGachaPoolClient: any[];
    jointGachaPoolClient: any[];
    attractionGachaPoolClient: any[];
    specialRecruitPool: any[];
    // --- End of potentially empty arrays ---
    gachaTags: GachaTag[];
    recruitPool: RecruitPool;
    potentialMaterialConverter: PotentialMaterialConverter;
    classicPotentialMaterialConverter: ClassicPotentialMaterialConverter;
    recruitRarityTable: Record<string, RecruitRarityTableEntry>;
    specialTagRarityTable: Record<string, number[]>;
    recruitDetail: string;
    showGachaLogEntry: boolean;
    carousel: CarouselEntry[];
    freeGacha: FreeGachaEntry[]; // Note: This seems distinct from 'freeGachaPoolClient'
    limitTenGachaItem: LimitTenGachaItemEntry[];
    linkageTenGachaItem: LinkageTenGachaItemEntry[];
    normalGachaItem: NormalGachaItemEntry[];
    fesGachaPoolRelateItem: Record<string, FesGachaPoolRelateItemEntry>;
    dicRecruit6StarHint: Record<string, string>;
    specialGachaPercentDict: Record<string, number>;
    gachaGroups: Record<string, any>; // Structure still unknown, left as Record<string, any>
}
