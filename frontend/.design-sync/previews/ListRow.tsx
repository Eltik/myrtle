import { CnName, ListRow, OpRefList, ResolutionBadge, RowImage, Tag } from "frontend";

// One row of a release list: thumbnail, the name block, and the EN-date badge on
// the right. Fixtures are live /release rows; `today` is pinned to 2026-09-01.
const API = "https://api.myrtle.moe/api";
const TODAY = new Date("2026-09-01T12:00:00+09:00");
const entry = (id: string, name: string, rarity: number, profession: string, subProfessionId: string, position: string, nationId: string) =>
    [id, { id, name, appellation: " ", rarity, profession, subProfessionId, position, tagList: [], nationId, isNotObtainable: false, groupId: null, teamId: null }] as const;
const LOOKUP = new Map<string, never>([entry("char_4065_judge", "Penance", 6, "TANK", "unyield", "MELEE", "siracusa"), entry("char_4121_zuole", "Zuo Le", 6, "WARRIOR", "musha", "MELEE", "yan")] as never);

// Events: 16:9 art, two rows.
export const Events = () => (
    <div className="w-full max-w-3xl p-4">
        <ListRow visual={<RowImage src={`${API}/en/event-image/act51side`} alt="People, A People" onError={() => {}} />} badge={<ResolutionBadge resolution={{ status: "confirmed", enId: "act51side", enStart: 1789570800, enEnd: 1790765999 }} today={TODAY} />}>
            <CnName cn="人们，我们" en="People, A People" primaryClassName="font-sans font-semibold text-[14px] text-foreground">
                <Tag>Side story</Tag>
            </CnName>
        </ListRow>
        <ListRow visual={<RowImage src={`${API}/cn/event-image/act4mainss`} alt="Critical Phase Transition" onError={() => {}} />} badge={<ResolutionBadge resolution={{ status: "estimated", enStart: 1791239400, lo: 1791099000, hi: 1791603900 }} today={TODAY} />}>
            <CnName cn="相变临界" auto={{ text: "Critical Phase Transition", source: "appellation" }} primaryClassName="font-sans font-semibold text-[14px] text-foreground">
                <Tag>Main story</Tag>
            </CnName>
        </ListRow>
    </div>
);

// A banner: wide 5:2 art and the featured operators.
export const Banner = () => (
    <div className="w-full max-w-3xl p-4">
        <ListRow wide visual={<RowImage wide src={`${API}/cn/banner-image/DOUBLE_77_0_5`} alt="Rare Operators useful in all kinds of stages" onError={() => {}} />} badge={<ResolutionBadge resolution={{ status: "independent" }} today={TODAY} />}>
            <CnName cn="适合多种场合的强力干员" auto={{ text: "Rare Operators useful in all kinds of stages", source: "memory" }} primaryClassName="font-sans font-semibold text-[14px] text-foreground">
                <Tag>Double rate-up</Tag>
            </CnName>
            <OpRefList ids={["char_4065_judge", "char_4121_zuole"]} lookup={LOOKUP} />
        </ListRow>
    </div>
);

// No visual: the name and badge columns only.
export const TextOnly = () => (
    <div className="w-full max-w-3xl p-4">
        <ListRow visual={null} badge={<ResolutionBadge resolution={{ status: "estimated", enStart: 1770751800, lo: 1770611400, hi: 1771116300 }} today={new Date("2026-01-20T12:00:00+09:00")} />}>
            <CnName cn="焰烬曙明完结庆祝" primaryClassName="font-sans font-semibold text-[14px] text-foreground">
                <Tag>Sign-in</Tag>
            </CnName>
        </ListRow>
    </div>
);
