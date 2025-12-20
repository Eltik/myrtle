// User types - Converted from backend Rust types

export type AKServer = "en" | "jp" | "kr" | "cn" | "bili" | "tw";

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
    inventory: Record<string, unknown>;
    limitedBuff: LimitedBuff;
    ticket: unknown;
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
    static: unknown | null; // Added by formatUser
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
    static: unknown | null; // Added by formatUser
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
    missions: unknown;
}

export interface UserSkin {
    characterSkins: unknown;
    skinTs: unknown;
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
    progressInfo: unknown;
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
    progressInfo: unknown;
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
    yCrissSs: string;
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
    CONTROL: Record<string, unknown>;
    ELEVATOR: Record<string, unknown>;
    POWER: Record<string, unknown>;
    MANUFACTURE: Record<string, unknown>;
    TRADING: Record<string, unknown>;
    DORMITORY: Record<string, unknown>;
    CORRIDOR: Record<string, unknown>;
    WORKSHOP: Record<string, unknown>;
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
    info: unknown[];
    progressInfo: unknown;
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
    info: unknown[];
    progressInfo: unknown;
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
    bgs: Record<string, unknown>;
}

export interface HomeTheme {
    selected: string;
    themes: Record<string, HomeThemeEntry>;
}

export interface HomeThemeEntry {
    unlock: number;
}

export interface Rlv2 {
    current: unknown;
    outer: Record<string, unknown>;
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
    current: unknown;
    outer: unknown;
    season: unknown;
}

export interface SiracusaMap {
    select: unknown | null;
    card: unknown;
    opera: SiracusaOpera;
    area: Record<string, number>;
}

export interface SiracusaOpera {
    total: number;
    show: unknown | null;
    release: Record<string, number>;
    like: unknown;
}

export interface StoryReview {
    groups: Record<string, StoryReviewGroup>;
    tags: unknown;
}

export interface StoryReviewGroup {
    rts: number;
    stories: unknown[];
}

export interface Medal {
    medals: Record<string, MedalEntry>;
    custom: MedalCustom;
}

export interface MedalEntry {
    id: string;
    val: unknown; // Can be nested arrays like [[23,30]]
    fts: number;
    rts: number;
}

export interface MedalCustom {
    currentIndex: unknown | null;
    customs: unknown;
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
    battleCar: Record<string, unknown | null>;
    exhibitionCar: Record<string, unknown | null>;
    accessories: Record<string, unknown | null>;
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
    dailyUsage: unknown;
    inventory: unknown;
}
