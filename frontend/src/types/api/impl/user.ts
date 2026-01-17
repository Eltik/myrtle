// User types - Converted from backend Rust types
// Includes types from ak-roster (https://github.com/neeia/ak-roster)

import type { ActivityMetrics, EngagementMetrics } from "./leaderboard";
import type { CompletionSummary, RoguelikeScore, RoguelikeThemeScore, ScoreBreakdown } from "./score";

// Server type aliases
export type ApiServer = "en" | "jp" | "cn" | "kr";
export type YostarServer = "en" | "jp" | "kr";
export type AKServer = "en" | "jp" | "kr" | "cn" | "bili" | "tw";
export type ArknightsServer = AKServer; // Alias for compatibility
export type Distributor = "yostar" | "hypergryph" | "bilibili";

// Channel IDs for different distributors
export const channelIds: { [distributor in Distributor]: string } = {
    hypergryph: "1",
    bilibili: "2",
    yostar: "3",
};

// Yostar API domains by server region
export const yostarDomains: Record<YostarServer, string> = {
    en: "https://en-sdk-api.yostarplat.com",
    jp: "https://jp-sdk-api.yostarplat.com",
    kr: "https://jp-sdk-api.yostarplat.com",
};

// Network configuration URLs for all servers
export const networkConfigUrls: { [server in AKServer]: string } = {
    en: "https://ak-conf.arknights.global/config/prod/official/network_config",
    jp: "https://ak-conf.arknights.jp/config/prod/official/network_config",
    kr: "https://ak-conf.arknights.kr/config/prod/official/network_config",
    cn: "https://ak-conf.hypergryph.com/config/prod/official/network_config",
    bili: "https://ak-conf.hypergryph.com/config/prod/b/network_config",
    tw: "https://ak-conf.txwy.tw/config/prod/official/network_config",
};

// Yostar authentication response
export interface YostarAuthData {
    result: number;
    yostar_uid: string;
    yostar_token: string;
    yostar_account: string;
}

// Yostar token response
export interface YostarToken {
    result: number;
    uid: string;
    token: string;
}

// Access token response
export interface AccessToken {
    result: number;
    accessToken: string;
}

// U8 token response
export interface U8Token {
    result: number;
    uid: string;
    token: string;
}

// Login secret response
export interface LoginSecret {
    result: number;
    uid: string;
    secret: string;
}

// Game version information
export interface VersionInfo {
    resVersion: string;
    clientVersion: string;
}

// Device/authentication token data
export interface TokenData {
    deviceId: string;
    token: YostarToken;
}

// Database model for stored user (wrapper around game data)
export interface StoredUser {
    id: string; // UUID
    uid: string;
    server: string;
    data: User; // The actual game User data
    settings: Record<string, unknown>;
    role: string;
    score: StoredUserScore | null; // Score data calculated by backend
    createdAt: string; // ISO date string
    updatedAt: string; // ISO date string
}

// Score data stored in database (matches backend UserScore)
export interface StoredUserScore {
    totalScore: number;
    operatorScore: number;
    stageScore: number;
    roguelikeScore: number;
    sandboxScore: number;
    medalScore: number;
    baseScore: number;
    grade: StoredUserGrade;
    breakdown: ScoreBreakdown;
    completionSummary: CompletionSummary;
    // Additional detailed fields available but not commonly used in UI
    operatorScores?: unknown[];
    zoneScores?: unknown[];
    roguelikeThemeScores?: RoguelikeThemeScore[];
    sandboxAreaScores?: unknown[];
    medalCategoryScores?: unknown[];
    roguelikeDetails?: RoguelikeScore;
    sandboxDetails?: unknown;
    medalDetails?: unknown;
    baseDetails?: unknown;
}

// Grade information (matches backend UserGrade)
export interface StoredUserGrade {
    grade: "S" | "A" | "B" | "C" | "D" | "F";
    compositeScore: number;
    accountAgeDays: number;
    normalizedScore: number;
    activityMetrics: ActivityMetrics;
    engagementMetrics: EngagementMetrics;
    percentileEstimate: number;
    calculatedAt: number;
}

/** Response from account/syncData endpoint */
export interface UserResponse {
    result: number;
    ts: number;
    user: User | null;
}

/** Main user data structure */
export interface User {
    dungeon: Dungeon;
    activity: unknown; // Complex, varies by event
    status: UserStatus;
    troop: Troop;
    npcAudio: Record<string, NpcAudioInfo>;
    recruit: Recruit;
    pushFlags: PushFlags;
    equipment: Equipment;
    skin: UserSkin;
    shop: Shop;
    mission: UserMission;
    social: Social;
    building: Building;
    dexNav: DexNav;
    crisis: Crisis;
    crisisV2?: unknown; // From ak-roster
    tshop: Record<string, TShopEntry>;
    gacha: Gacha;
    backflow: Backflow;
    mainline: Mainline;
    avatar: Record<string, AvatarEntry>;
    background: Background;
    homeTheme: HomeTheme;
    rlv2: Rlv2;
    deepSea: DeepSea;
    tower: Tower;
    siracusaMap: SiracusaMap;
    storyreview: StoryReview;
    medal: Medal;
    nameCardStyle?: unknown; // From ak-roster
    aprilFool: Record<string, AprilFoolEntry>;
    retro: Retro;
    charm: Charm;
    carousel: Carousel;
    consumable: Record<string, Record<string, ConsumableEntry>>;
    event: Event;
    collectionReward: CollectionReward;
    checkIn: CheckIn;
    car: Car;
    openServer: OpenServer;
    campaignsV2: CampaignsV2;
    inventory: Record<string, InventoryItem>;
    limitedBuff: LimitedBuff;
    ticket: unknown;
    // Additional fields from ak-roster
    tokenData?: TokenData;
    sandboxPerm?: unknown;
    trainingGround?: unknown;
    checkMeta?: unknown;
    share?: unknown;
    roguelike?: unknown;
}

export interface Dungeon {
    stages: Record<string, StageData>;
    cowLevel: Record<string, CowLevelEntry>;
    mainlineBannedStages: unknown[];
}

export interface StageData {
    stageId: string;
    completeTimes: number;
    startTimes: number;
    practiceTimes: number;
    state: number;
    hasBattleReplay: number;
    noCostCnt: number;
}

export interface CowLevelEntry {
    id: string;
    type: string;
    val: string[];
    fts: number;
    rts: number;
}

export interface UserStatus {
    nickName: string;
    nickNumber: string;
    level: number;
    exp: number;
    socialPoint: number;
    gachaTicket: number;
    classicGachaTicket: number;
    tenGachaTicket: number;
    classicTenGachaTicket: number;
    instantFinishTicket: number;
    hggShard: number;
    lggShard: number;
    classicShard: number;
    recruitLicense: number;
    progress: number;
    buyApRemainTimes: number;
    apLimitUpFlag: number;
    uid: string;
    flags: Record<string, number>;
    ap: number;
    maxAp: number;
    payDiamond: number;
    freeDiamond: number;
    diamondShard: number;
    gold: number;
    practiceTicket: number;
    lastRefreshTs: number;
    lastApAddTime: number;
    mainStageProgress: string;
    registerTs: number;
    lastOnlineTs: number;
    serverName: string;
    avatarId: string;
    avatar: Avatar;
    resume: string;
    friendNumLimit: number;
    monthlySubscriptionStartTime: number;
    monthlySubscriptionEndTime: number;
    tipMonthlyCardExpireTs: number;
    secretary: string;
    secretarySkinId: string;
    globalVoiceLan?: string; // From ak-roster - global voice language setting
}

export interface Avatar {
    type: string | null;
    id: string | null;
}

export interface Troop {
    curCharInstId: number;
    curSquadCount: number;
    squads: Record<string, Squad>;
    chars: Record<string, CharacterData>;
    charGroup: Record<string, CharGroup>;
    charMission: Record<string, Record<string, number>>;
    addon?: unknown; // From ak-roster
}

export interface Squad {
    squadId: string;
    name: string;
    slots: (SquadSlot | null)[];
}

export interface SquadSlot {
    charInstId: number;
    skillIndex: number;
    currentEquip: string | null;
}

export interface CharacterData {
    instId: number;
    charId: string;
    favorPoint: number;
    potentialRank: number;
    mainSkillLvl: number;
    skin: string;
    level: number;
    exp: number;
    evolvePhase: number;
    defaultSkillIndex: number;
    gainTime: number;
    currentTmpl: string | null;
    tmpl: Record<string, CharacterTemplate> | null;
    skills: CharacterSkill[];
    voiceLan: string;
    currentEquip: string | null;
    equip: Record<string, EquipData>;
    starMark?: number; // From ak-roster - star mark/favorite indicator
    static?: CharacterStatic | null; // Added by formatUser - operator metadata
}

export interface CharacterTemplate {
    equip: Record<string, EquipData>;
    skills: CharacterSkill[];
    skinId: string;
    currentEquip: string;
    defaultSkillIndex: number;
}

export interface CharacterSkill {
    skillId: string;
    unlock: number;
    state: number;
    specializeLevel: number;
    completeUpgradeTime: number;
    static?: UserSkillStatic | null; // Added by formatUser - skill metadata
}

export interface EquipData {
    hide: number;
    locked: number;
    level: number;
}

export interface CharGroup {
    favorPoint: number;
}

export interface NpcAudioInfo {
    npcShowAudioInfoFlag: string;
}

export interface Recruit {
    normal: RecruitNormal;
}

export interface RecruitNormal {
    slots: Record<string, RecruitSlot>;
}

export interface RecruitSlot {
    state: number;
    tags: number[];
    selectTags: SelectTag[];
    startTs: number;
    durationInSec: number;
    maxFinishTs: number;
    realFinishTs: number;
}

export interface SelectTag {
    tagId: number;
    pick: number;
}

export interface PushFlags {
    hasGifts: number;
    hasFriendRequest: number;
    hasClues: number;
    hasFreeLevelGp: number;
    status: number;
}

export interface Equipment {
    missions: Record<string, EquipmentMission>;
}

export interface EquipmentMission {
    value: number;
    target: number;
}

export interface UserSkin {
    characterSkins: Record<string, number>; // Record<skin_id, 1> - the number is always 1
    skinTs: Record<string, number>; // Record<skin_id, epoch_timestamp>
}

export interface Shop {
    LS: ShopLS;
    HS: ShopHS;
    ES: ShopES;
    CASH: ShopCash;
    GP: ShopGP;
    FURNI: ShopFurni;
    SOCIAL: ShopSocial;
    CLASSIC: ShopClassic;
}

export interface ShopLS {
    curShopId: string;
    curGroupId: string;
    info: ShopInfo[];
}

export interface ShopHS {
    curShopId: string;
    info: ShopInfo[];
    progressInfo: ShopProgressInfo;
}

export interface ShopES {
    curShopId: string;
    info: ShopInfo[];
}

export interface ShopCash {
    info: ShopInfo[];
}

export interface ShopGP {
    oneTime: ShopInfoContainer;
    level: ShopInfoContainer;
    weekly: ShopGroupInfoContainer;
    monthly: ShopGroupInfoContainer;
    choose: ShopInfoContainer;
    backflow: ShopInfoContainer;
}

export interface ShopInfoContainer {
    info: ShopInfo[];
}

export interface ShopGroupInfoContainer {
    curGroupId: string;
    info: ShopInfo[];
}

export interface ShopFurni {
    info: ShopInfo[];
}

export interface ShopSocial {
    curShopId: string;
    info: ShopInfo[];
}

export interface ShopClassic {
    info: ShopInfo[];
    progressInfo: ShopProgressInfo;
}

export interface ShopInfo {
    id: string;
    count: number;
}

export interface UserMission {
    missions: UserMissionCategories;
    missionRewards: UserMissionRewards;
    missionGroups: Record<string, number>;
}

export interface UserMissionCategories {
    OPENSERVER: Record<string, UserMissionData>;
    DAILY: Record<string, UserMissionData>;
    WEEKLY: Record<string, UserMissionData>;
    GUIDE: Record<string, UserMissionData>;
    MAIN: Record<string, UserMissionData>;
    ACTIVITY: Record<string, UserMissionData>;
    SUB: Record<string, UserMissionData>;
}

export interface UserMissionData {
    state: number;
    progress: UserMissionProgress[];
}

export interface UserMissionProgress {
    target: number;
    value: number;
}

export interface UserMissionRewards {
    dailyPoint: number;
    weeklyPoint: number;
    rewards: UserMissionRewardCategories;
}

export interface UserMissionRewardCategories {
    DAILY: Record<string, number>;
    WEEKLY: Record<string, number>;
}

export interface Social {
    assistCharList: (AssistChar | null)[];
    yesterdayReward: YesterdayReward;
    yCrisisSs: string; // Fixed typo from yCrissSs
    yCrisisV2Ss?: unknown; // From ak-roster
    medalBoard?: MedalBoard; // From ak-roster
}

export interface MedalBoard {
    type: string;
    custom: string;
    template: unknown;
    templateMedalList: unknown;
}

export interface AssistChar {
    charInstId: number;
    skillIndex: number;
    currentEquip: string | null;
}

export interface YesterdayReward {
    canReceive: number;
    assistAmount: number;
    comfortAmount: number;
    first: number;
}

export interface Building {
    status: BuildingStatus;
    chars: Record<string, BuildingChar>;
    roomSlots: Record<string, RoomSlot>;
    rooms: BuildingRooms;
    furniture: Record<string, FurnitureEntry>;
    assist: number[];
    diyPresentSolutions: unknown;
    solution: BuildingSolution;
}

export interface BuildingStatus {
    labor: Labor;
    workshop: WorkshopStatus;
}

export interface Labor {
    buffSpeed: number;
    processPoint: number;
    value: number;
    lastUpdateTime: number;
    maxValue: number;
}

export interface WorkshopStatus {
    bonusActive: number;
    bonus: unknown;
}

export interface BuildingChar {
    charId: string;
    lastApAddTime: number;
    ap: number;
    roomSlotId: string;
    index: number;
    changeScale: number;
    bubble: Bubble;
    workTime: number;
}

export interface Bubble {
    normal: BubbleData;
    assist: BubbleData;
}

export interface BubbleData {
    add: number;
    ts: number;
}

export interface RoomSlot {
    level: number;
    state: number;
    roomId: string;
    charInstIds: number[];
    completeConstructTime: number;
}

export interface BuildingRooms {
    CONTROL: Record<string, ControlRoom>;
    ELEVATOR: Record<string, Record<string, unknown>>; // Usually empty
    POWER: Record<string, PowerRoom>;
    MANUFACTURE: Record<string, ManufactureRoom>;
    TRADING: Record<string, TradingRoom>;
    DORMITORY: Record<string, DormitoryRoom>;
    CORRIDOR: Record<string, Record<string, unknown>>; // Usually empty
    WORKSHOP: Record<string, WorkshopRoom>;
}

export interface FurnitureEntry {
    count: number;
    inUse: number;
}

export interface BuildingSolution {
    furnitureTs: Record<string, number>;
}

export interface DexNav {
    character: Record<string, DexNavCharacter>;
    formula: DexNavFormula;
    teamV2: Record<string, Record<string, number>>;
    enemy: DexNavEnemy;
}

export interface DexNavCharacter {
    charInstId: number;
    count: number;
    classicCount: number;
}

export interface DexNavFormula {
    shop: Record<string, number>;
    manufacture: Record<string, number>;
    workshop: Record<string, number>;
}

export interface DexNavEnemy {
    enemies: Record<string, number>;
    stage: Record<string, number>;
}

export interface Crisis {
    current: string;
    lst: number;
    nst: number;
    map: Record<string, CrisisMap>;
    shop: CrisisShop;
    training: CrisisTraining;
    season: unknown;
    box: unknown[];
}

export interface CrisisMap {
    rank: number;
    confirmed: number;
}

export interface CrisisShop {
    coin: number;
    info: ShopInfo[];
    progressInfo: ShopProgressInfo;
}

export interface CrisisTraining {
    currentStage: string[];
    stage: Record<string, CrisisTrainingStage>;
    nst: number;
}

export interface CrisisTrainingStage {
    point: number;
}

export interface TShopEntry {
    coin: number;
    info: ShopInfo[];
    progressInfo: ShopProgressInfo;
}

export interface Gacha {
    newbee: GachaNewbee;
    normal: Record<string, GachaNormal>;
    attain: unknown;
    limit: Record<string, GachaLimit>;
    single: Record<string, GachaSingle>;
    fesClassic: unknown;
}

export interface GachaNewbee {
    openFlag: number;
    cnt: number;
    poolId: string;
}

export interface GachaNormal {
    cnt: number;
    maxCnt: number;
    rarity: number;
    avail: boolean;
}

export interface GachaLimit {
    leastFree: number;
}

export interface GachaSingle {
    singleEnsureCnt: number;
    singleEnsureUse: boolean;
    singleEnsureChar: string;
}

export interface Backflow {
    open: boolean;
    current: unknown | null;
    currentV2: unknown | null;
}

export interface Mainline {
    record: unknown;
    cache: unknown[];
    version: number;
    additionalMission: unknown;
}

export interface AvatarEntry {
    ts: number;
    src: string;
}

export interface Background {
    selected: string;
    bgs: Record<string, BackgroundEntry>;
}

export interface HomeTheme {
    selected: string;
    themes: Record<string, HomeThemeEntry>;
}

export interface HomeThemeEntry {
    unlock: number;
}

export interface Rlv2 {
    current: Rlv2Current | null;
    outer: Record<string, Rlv2Outer>;
}

export interface Rlv2Current {
    player: Rlv2Player;
    map: { zones: Record<string, unknown> };
    troop: Rlv2Troop;
    inventory: Rlv2Inventory;
    game: Rlv2Game;
    buff: Rlv2Buff;
    record: { brief: unknown | null };
    module: unknown[];
}

export interface Rlv2Player {
    state: string;
    property: {
        exp: number;
        level: number;
        maxLevel?: number;
        hp: { current: number; max: number };
        gold: number;
        shield: number;
        capacity: number;
        population: { cost: number; max: number };
        conPerfectBattle: number;
    };
    cursor: { zone: number; position: unknown | null };
    trace: unknown[];
    pending: unknown[];
    status: { bankPut: number };
    toEnding: string;
    chgEnding: boolean;
}

export interface Rlv2Troop {
    chars: Record<string, unknown>;
    expedition: unknown[];
    expeditionReturn: unknown | null;
    hasExpeditionReturn: boolean;
    expeditionDetails?: Record<string, unknown>;
}

export interface Rlv2Inventory {
    relic: Record<string, unknown>;
    recruit: Record<string, unknown>;
    trap: unknown | null;
    consumable: Record<string, unknown>;
    exploreTool?: Record<string, unknown>;
}

export interface Rlv2Game {
    mode: string;
    predefined: unknown | null;
    theme: string;
    outer: { support: boolean };
    start: number;
    modeGrade: number;
    equivalentGrade: number;
}

export interface Rlv2Buff {
    tmpHP: number;
    capsule: unknown | null;
    squadBuff: unknown[];
}

export interface Rlv2Outer {
    bank: Rlv2Bank;
    bp: { point: number; reward: Record<string, unknown> };
    buff: Rlv2OuterBuff;
    collect: Rlv2Collect;
    mission: Rlv2Mission;
    challenge: { reward: Record<string, unknown>; grade: Record<string, unknown>; highScore?: Record<string, number> };
    monthTeam: { valid: string[]; reward: Record<string, unknown>; mission: Record<string, unknown> };
    record: Rlv2Record;
    activity?: Record<string, unknown>;
}

export interface Rlv2Bank {
    show: boolean;
    current: number;
    record: number;
    reward: Record<string, unknown>;
    totalPut: number;
}

export interface Rlv2OuterBuff {
    pointOwned: number;
    pointCost: number;
    unlocked: Record<string, unknown>;
    score: number;
}

export interface Rlv2Collect {
    relic: Record<string, Rlv2CollectItem>;
    capsule: Record<string, Rlv2CollectItem>;
    activeTool: Record<string, Rlv2CollectItem>;
    bgm: Record<string, number>;
    pic: Record<string, number>;
    chat: Record<string, number>;
    band: Record<string, Rlv2CollectItem>;
    buff: Record<string, Rlv2CollectItem>;
    endBook: Record<string, unknown>;
    mode: Record<string, Record<string, Rlv2CollectItem>>;
    recruitSet: Record<string, Rlv2CollectItem>;
    modeGrade: Record<string, Record<string, Rlv2CollectItem>>;
}

export interface Rlv2CollectItem {
    state: number;
    progress: number[] | null;
}

export interface Rlv2Mission {
    updateId: string;
    refresh: number;
    list: unknown[];
}

export interface Rlv2Record {
    last: number;
    modeCnt: Record<string, unknown>;
    endingCnt: Record<string, unknown>;
    stageCnt: Record<string, unknown>;
    bandCnt: Record<string, unknown>;
}

export interface DeepSea {
    places: Record<string, number>;
    nodes: Record<string, number>;
    choices: Record<string, number[]>;
    events: unknown;
    treasures: unknown;
    stories: unknown;
    techTrees: unknown;
    logs: unknown;
}

export interface Tower {
    current: TowerCurrent | null;
    outer: TowerOuter;
    season: TowerSeason;
}

export interface TowerCurrent {
    status: TowerStatus;
    layer: unknown[];
    cards: Record<string, unknown>;
    godCard: { id: string; subGodCardId: string };
    halftime: { count: number; candidate: unknown[]; canGiveUp: boolean };
    trap: unknown[];
    reward?: { high: number; low: number };
}

export interface TowerStatus {
    state: string;
    tower: string;
    coord: number;
    tactical: Record<string, string>; // CASTER, MEDIC, PIONEER, etc.
    strategy: string;
    start: number;
    isHard: boolean;
}

export interface TowerOuter {
    training: Record<string, unknown>;
    towers: Record<string, unknown>;
    hasTowerPass: number;
    pickedGodCard: Record<string, unknown>;
    tactical: Record<string, string>;
    strategy: string;
}

export interface TowerSeason {
    id: string;
    finishTs: number;
    missions: Record<string, TowerMission>;
    passWithGodCard: Record<string, unknown>;
    slots: Record<string, unknown>;
    period: TowerPeriod;
}

export interface TowerMission {
    value: number;
    target: number;
    hasRecv: boolean;
}

export interface TowerPeriod {
    termTs: number;
    items: Record<string, unknown>;
    cur: number;
    len: number;
}

export interface SiracusaMap {
    select: unknown | null;
    card: unknown;
    opera: SiracusaOpera;
    area: Record<string, number>;
}

export interface SiracusaOpera {
    total: number;
    show: string | null;
    release: Record<string, number>;
    like: Record<string, string>; // Map of char_card_X -> opera_X
}

export interface StoryReview {
    groups: Record<string, StoryReviewGroup>;
    tags: unknown;
}

export interface StoryReviewGroup {
    rts: number;
    stories: StoryReviewStory[];
}

export interface Medal {
    medals: Record<string, MedalEntry>;
    custom: MedalCustom;
}

export interface MedalEntry {
    id: string;
    val: [number, number][]; // Array of [index, value] tuples, e.g. [[0, 7], [1, 30]]
    fts: number; // First timestamp (-1 if not obtained)
    rts: number; // Recent timestamp (-1 if not used)
}

export interface MedalCustom {
    currentIndex: string | null;
    customs: Record<string, unknown>;
}

export interface AprilFoolEntry {
    stages: unknown;
    liveEndings: unknown | null;
    cameraLv: number | null;
    fans: number | null;
    posts: number | null;
    missions: Record<string, AprilFoolMission> | null;
}

export interface AprilFoolMission {
    value: number;
    target: number;
    finished: boolean;
    hasRecv: boolean;
}

export interface Retro {
    coin: number;
    supplement: number;
    block: Record<string, RetroBlock>;
    trail: Record<string, Record<string, number>>;
    lst: number;
    nst: number;
    rewardPerm: unknown[];
}

export interface RetroBlock {
    locked: number;
    open: number;
}

export interface Charm {
    charms: Record<string, number>;
    squad: unknown[];
}

export interface Carousel {
    furnitureShop: CarouselFurnitureShop;
}

export interface CarouselFurnitureShop {
    goods: unknown;
    groups: unknown;
}

export interface ConsumableEntry {
    ts: number;
    count: number;
}

export interface Event {
    building: number;
}

export interface CollectionReward {
    team: unknown;
}

export interface CheckIn {
    canCheckIn: number;
    checkInGroupId: string;
    checkInRewardIndex: number;
    checkInHistory: number[];
    newbiePackage: NewbiePackage;
}

export interface NewbiePackage {
    open: boolean;
    groupId: string;
    finish: number;
    stopSale: number;
    checkInHistory: number[];
}

export interface Car {
    battleCar: Record<string, unknown>;
    exhibitionCar: Record<string, unknown>;
    accessories: Record<string, CarAccessory>;
}

export interface OpenServer {
    checkIn: OpenServerCheckIn;
    chainLogin: ChainLogin;
}

export interface OpenServerCheckIn {
    isAvailable: boolean;
    history: number[];
}

export interface ChainLogin {
    isAvailable: boolean;
    nowIndex: number;
    history: number[];
    finalReward: number;
}

export interface CampaignsV2 {
    campaignCurrentFee: number;
    campaignTotalFee: number;
    open: CampaignsOpen;
    instances: Record<string, CampaignInstance>;
    missions: Record<string, number>;
    lastRefreshTs: number;
    sweepMaxKills: unknown;
}

export interface CampaignsOpen {
    permanent: string[];
    rotate: string;
    rGroup: string;
    training: string[];
    tGroup: string;
    tAllOpen: unknown | null;
}

export interface CampaignInstance {
    maxKills: number;
    rewardStatus: number[];
}

export interface LimitedBuff {
    dailyUsage: Record<string, unknown>;
    inventory: Record<string, unknown>;
}

// ==================== Inventory Types ====================

export interface InventoryItem {
    amount: number;
    buildingProductList?: BuildingProductEntry[];
    classifyType?: string;
    description?: string;
    hideInItemGet?: boolean;
    iconId?: string;
    itemId: string;
    itemType?: string;
    name?: string;
    obtainApproach?: string | null;
    overrideBkg?: string | null;
    rarity?: string;
    sortId?: number;
    stackIconId?: string;
    stageDropList?: StageDropEntry[];
    usage?: string;
    voucherRelateList?: unknown[] | null;
    image?: string; // Added by backend formatting
}

export interface BuildingProductEntry {
    formulaId: string;
    roomType: string;
}

export interface StageDropEntry {
    occPer: string;
    stageId: string;
}

// ==================== Background Types ====================

export interface BackgroundEntry {
    unlock?: number; // Unix timestamp when unlocked
    conditions?: Record<string, BackgroundCondition>;
}

export interface BackgroundCondition {
    t: number; // Condition type
    v: number; // Condition value
}

// ==================== Shop Progress Types ====================

export interface ShopProgressInfo {
    [id: string]: ShopProgressEntry;
}

export interface ShopProgressEntry {
    count: number;
    order: number;
}

// ==================== Building Room Types ====================

export interface ControlRoom {
    apCost: number;
    buff: ControlRoomBuff;
    lastUpdateTime: number;
    presetQueue: unknown[];
}

export interface ControlRoomBuff {
    apCost: Record<string, number>;
    dormitory: { recover: number; tagRecover: Record<string, number> };
    global: { apCost: number; roomCnt: Record<string, number> };
    hire: { apCost: number; spUp: { base: number; up: number } };
    manufacture: { apCost: number; roomSpeed: Record<string, number>; sSpeed: number; speed: number };
    meeting: { apCost: number; clue: number; notOwned: number; sSpeed: number; speedUp: number; weight: Record<string, number> };
    point: Record<string, number>;
    power: { apCost: number };
    trading: { apCost: number; charLimit: Record<string, number>; charSpeed: Record<string, number>; roomLimit: Record<string, number>; roomSpeed: Record<string, number>; sSpeed: number; speed: number };
    training: { speed: number };
}

export interface TradingRoom {
    buff?: TradingRoomBuff;
    state?: number;
    lastUpdateTime?: number;
    strategy?: string; // e.g. "O_GOLD"
    stockLimit?: number;
    apCost?: number;
    stock?: TradingStock[];
    next?: TradingNext;
    completeWorkTime?: number;
    display?: RoomDisplay;
}

export interface TradingRoomBuff {
    speed: number;
    limit: number;
    apCost: { all: number; single: unknown; self: unknown };
    rate: unknown;
    tgw: unknown[];
    point: unknown;
    manuLines: unknown;
    orderWtBuff: unknown[];
    orderBuff: unknown[];
    violatedInfo: { orderChecker: unknown[]; cntBuff: unknown[] };
}

export interface TradingStock {
    instId?: number;
    type?: string; // e.g. "MATERIAL"
    delivery?: TradingDelivery[];
    gain?: { type?: string; id?: string; count?: number };
    buff?: unknown[];
}

export interface TradingDelivery {
    type: string;
    id: string;
    count: number;
}

export interface TradingNext {
    order: number;
    processPoint: number;
    maxPoint: number;
    speed: number;
}

export interface ManufactureRoom {
    buff?: ManufactureRoomBuff;
    state?: number;
    formulaId?: string;
    remainSolutionCnt?: number;
    outputSolutionCnt?: number;
    lastUpdateTime?: number;
    saveTime?: number;
    tailTime?: number;
    apCost?: number;
    completeWorkTime?: number;
    capacity?: number;
    processPoint?: number;
    display?: RoomDisplay;
}

export interface ManufactureRoomBuff {
    apCost: { self: unknown };
    speed: number;
    capacity: number;
    sSpeed: number;
    tSpeed: unknown;
    cSpeed: number;
    capFrom: unknown;
    maxSpeed: number;
    point: unknown;
    flag: unknown;
    skillExtend: unknown;
}

export interface RoomDisplay {
    base: number;
    buff: number;
}

export interface DormitoryRoom {
    buff?: DormitoryRoomBuff;
    comfort?: number;
    diySolution?: DormitorySolution | null;
}

export interface DormitoryRoomBuff {
    apCost: {
        all: number;
        single: { target: unknown | null; value: number };
        self: unknown;
        exclude: unknown;
    };
    point: unknown;
}

export interface DormitorySolution {
    wallPaper?: string;
    floor?: string;
    carpet?: DormitoryCarpet;
    furnitures?: DormitoryFurniture[];
}

export interface DormitoryCarpet {
    id?: string;
    coordinate?: { x: number; y: number };
}

export interface DormitoryFurniture {
    id: string;
    coordinate: { x: number; y: number };
}

export interface PowerRoom {
    buff?: PowerRoomBuff;
    state?: number;
}

export interface PowerRoomBuff {
    global: { roomCnt: unknown };
    manufacture: { charSpeed: unknown };
    laborSpeed: number;
    apCost: { self: unknown };
}

export interface WorkshopRoom {
    buff?: WorkshopRoomBuff;
    state?: number;
    lastUpdateTime?: number;
    statistic?: { noAddition: number };
}

export interface WorkshopRoomBuff {
    rate: { all: number };
    cost: { type: string; limit: number; reduction: number };
    costRe: { type: string; from: number; change: number };
    frate: unknown[];
    recovery: { type: string; pace: number; recover: number };
    goldFree: unknown;
    costForce: { type: string; cost: number };
    fFix: { asRarity: unknown };
    activeBonus: unknown;
    apRate: unknown;
    costDevide: { type: string; limit: number; denominator: number };
}

// ==================== Car Types ====================

export interface CarAccessory {
    id: string;
    num: number;
}

// ==================== Story Review Types ====================

export interface StoryReviewStory {
    id: string;
    rc: number; // Read count
    uts: number; // Unix timestamp
}

// ==================== Character Static Types ====================
// These types represent the static operator data added by backend formatUser
// Prefixed with "User" to avoid conflicts with game data table types in operator.ts, skill.ts, etc.

/** Static operator data added by backend formatUser */
export interface CharacterStatic {
    id: string;
    name: string;
    appellation?: string;
    description?: string;
    rarity: string; // "TIER_1" through "TIER_6"
    profession: string; // "CASTER", "GUARD", etc.
    subProfessionId?: string;
    position: string; // "RANGED" or "MELEE"
    nationId?: string | null;
    groupId?: string | null;
    teamId?: string | null;
    displayNumber?: string;
    portrait?: string;
    skin?: string;
    tagList?: string[];
    isSpChar?: boolean;
    isNotObtainable?: boolean;
    itemUsage?: string;
    itemDesc?: string;
    itemObtainApproach?: string;
    maxPotentialLevel?: number;
    canUseGeneralPotentialItem?: boolean;
    canUseActivityPotentialItem?: boolean;
    potentialItemId?: string | null;
    classicPotentialItemId?: string | null;
    activityPotentialItemId?: string | null;
    displayTokenDict?: Record<string, unknown> | null;
    trait?: UserCharacterTrait | null;
    phases?: UserCharacterPhase[];
    skills?: UserCharacterStaticSkill[];
    talents?: UserCharacterTalent[];
    potentialRanks?: UserPotentialRank[];
    favorKeyFrames?: UserFavorKeyFrame[];
    allSkillLevelUp?: UserSkillLevelUpCost[];
    modules?: UserCharacterModule[];
    handbook?: UserCharacterHandbook;
    profile?: UserCharacterProfile;
    trust?: number;
    artists?: string[];
}

export interface UserCharacterTrait {
    candidates?: UserTraitCandidate[];
}

export interface UserTraitCandidate {
    additionalDescription?: string | null;
    blackboard?: UserBlackboardEntry[];
    overrideDescripton?: string | null; // Note: typo in game data
    prefabKey?: string | null;
    rangeId?: string | null;
    requiredPotentialRank?: number;
    unlockCondition?: UserUnlockCondition;
}

export interface UserBlackboardEntry {
    key: string;
    value: number;
    valueStr?: string | null;
}

export interface UserUnlockCondition {
    Level: number;
    Phase: string; // "PHASE_0", "PHASE_1", "PHASE_2"
}

export interface UserCharacterPhase {
    AttributesKeyFrames?: UserAttributeKeyFrame[];
    CharacterPrefabKey?: string;
    EvolveCost?: UserItemCost[] | null;
    MaxLevel: number;
    RangeId?: string;
}

export interface UserAttributeKeyFrame {
    Data: UserCharacterAttributes;
    Level: number;
}

export interface UserCharacterAttributes {
    Atk: number;
    AttackSpeed: number;
    BaseAttackTime: number;
    BlockCnt: number;
    Cost: number;
    Def: number;
    HpRecoveryPerSec: number;
    MagicResistance: number;
    MassLevel: number;
    MaxDeckStackCnt: number;
    MaxDeployCount: number;
    MaxHp: number;
    MoveSpeed: number;
    RespawnTime: number;
    SpRecoveryPerSec: number;
    StunImmune: boolean;
    SilenceImmune: boolean;
    SleepImmune: boolean;
    FrozenImmune: boolean;
    LevitateImmune: boolean;
    DisarmedCombatImmune: boolean;
    TauntLevel: number;
}

export interface UserItemCost {
    Count: number;
    IconId?: string;
    Id: string;
    Image?: string;
    Type_: string;
}

export interface UserCharacterStaticSkill {
    skillId: string;
    overridePrefabKey?: string | null;
    overrideTokenKey?: string | null;
    levelUpCostCond?: UserSkillLevelUpCondition[];
    static?: UserSkillStatic;
}

export interface UserSkillLevelUpCondition {
    LevelUpCost?: UserItemCost[];
    LevelUpTime?: number;
    UnlockCond?: UserUnlockCondition;
}

export interface UserCharacterTalent {
    Candidates?: UserTalentCandidate[];
}

export interface UserTalentCandidate {
    BlackBoard?: UserBlackboardEntry[];
    Description?: string;
    Name?: string;
    PrefabKey?: string | null;
    RangeId?: string | null;
    RequiredPotentialRank?: number;
    TalentIndex?: number;
    UnlockCondition?: UserUnlockCondition;
}

export interface UserPotentialRank {
    Buff?: UserPotentialBuff | null;
    Description?: string;
    Type_?: number;
}

export interface UserPotentialBuff {
    Attributes?: UserPotentialBuffAttributes;
}

export interface UserPotentialBuffAttributes {
    AbnormalFlags?: unknown | null;
    AbnormalImmunes?: unknown | null;
    AbnormalAntis?: unknown | null;
    AbnormalCombos?: unknown | null;
    AbnormalComboImmunes?: unknown | null;
    AttributeModifiers?: UserAttributeModifier[];
}

export interface UserAttributeModifier {
    AttributeType: string; // "MAX_HP", "ATK", "DEF", "MAGIC_RESISTANCE", "COST", "ATTACK_SPEED", "RESPAWN_TIME", etc.
    FormulaItem: number;
    Value: number;
    FrameIndex: number;
}

export interface UserFavorKeyFrame {
    Data: UserFavorData;
    Level: number;
}

export interface UserFavorData {
    Atk: number;
    Def: number;
    MaxHp: number;
}

export interface UserSkillLevelUpCost {
    LvlUpCost?: UserItemCost[];
    UnlockCond?: UserUnlockCondition;
}

export interface UserCharacterModule {
    id: string;
    charId: string;
    uniEquipId: string;
    uniEquipName?: string;
    uniEquipIcon?: string;
    uniEquipDesc?: string;
    uniEquipGetTime?: number;
    charEquipOrder?: number;
    tmplId?: string | null;
    type: string;
    typeName1?: string;
    typeName2?: string;
    typeIcon?: string;
    equipShiningColor?: string;
    showEvolvePhase?: string;
    unlockEvolvePhase?: string;
    showLevel?: number;
    unlockLevel?: number;
    unlockFavorPoint?: number;
    image?: string;
    data?: UserModuleData;
    missionList?: UserModuleMission[];
    itemCost?: Record<string, UserItemCost[]>;
}

export interface UserModuleData {
    phases?: UserModulePhase[];
}

export interface UserModulePhase {
    attributeBlackboard?: UserBlackboardEntry[];
    equipLevel?: number;
    parts?: UserModulePart[];
    tokenAttributeBlackboard?: Record<string, UserBlackboardEntry[]>;
}

export interface UserModulePart {
    isToken?: boolean;
    overrideTraitDataBundle?: UserModuleTraitBundle;
    addOrOverrideTalentDataBundle?: UserModuleTalentBundle;
    target?: string;
}

export interface UserModuleTraitBundle {
    candidates?: UserTraitCandidate[] | null;
}

export interface UserModuleTalentBundle {
    candidates?: UserTalentCandidate[] | null;
}

export interface UserModuleMission {
    desc?: string;
    jumpStageId?: string | null;
    template?: string;
    uniEquipId?: string;
    uniEquipMissionId?: string;
    uniEquipMissionSort?: number;
}

export interface UserCharacterHandbook {
    charID: string;
    infoName?: string;
    isLimited?: boolean;
    storyTextAudio?: UserStoryTextAudio[];
    handbookAvgList?: UserHandbookAvg[];
}

export interface UserStoryTextAudio {
    stories?: UserHandbookStory[];
    storyTitle?: string;
    unLockorNot?: boolean;
}

export interface UserHandbookStory {
    storyText?: string;
    unLockParam?: string;
    unLockType?: number;
}

export interface UserHandbookAvg {
    avgList?: UserAvgEntry[];
    storySetId?: string;
    storySetName?: string;
    sortId?: number;
}

export interface UserAvgEntry {
    avgTag?: string;
    storyId?: string;
    storyIntro?: string;
    storyName?: string;
    storyTxt?: string;
}

export interface UserCharacterProfile {
    basicInfo?: UserBasicInfo;
    physicalExam?: UserPhysicalExam;
}

export interface UserBasicInfo {
    codeName?: string;
    combatExperience?: string;
    dateOfBirth?: string;
    gender?: string;
    height?: string;
    infectionStatus?: string;
    placeOfBirth?: string;
    race?: string;
}

export interface UserPhysicalExam {
    cellOriginiumAssimilation?: string;
    originiumArtsAssimilation?: string;
    physicalStrength?: string;
    tacticalAcumen?: string;
    combatSkill?: string;
    mobility?: string;
    bloodOriginiumCrystalDensity?: string;
}

// ==================== Skill Static Types ====================

/** Static skill data added by backend formatUser */
export interface UserSkillStatic {
    skillId: string;
    name?: string;
    description?: string;
    duration?: number;
    hidden?: boolean;
    iconId?: string | null;
    image?: string;
    spData?: UserSkillSpData;
    levels?: UserSkillLevel[]; // Full skill level data when available
}

export interface UserSkillLevel {
    name: string;
    rangeId?: string | null;
    description: string;
    skillType: string | number;
    durationType?: string | null;
    duration: number;
    spData: UserSkillSpData;
    prefabId?: string;
    blackboard: UserSkillBlackboard[];
}

export interface UserSkillBlackboard {
    key: string;
    value: number;
    valueStr?: string | null;
}

export interface UserSkillSpData {
    increment?: number;
    initSp?: number;
    levelUpCost?: unknown[] | null;
    maxChargeTime?: number;
    spCost?: number;
    spType?: string | number; // "INCREASE_WITH_TIME", "INCREASE_WHEN_ATTACK", etc.
}
