import { TimingPanel } from "frontend";

const DAY_MS = 86_400_000;
const utc = (y: number, m: number, d: number) => Date.UTC(y, m - 1, d);
const secs = (y: number, m: number, d: number) => Math.floor(utc(y, m, d) / 1000);

// 121 days of community pulls ending on the capture clock's "today" (2024-05-15).
const FIRST_PULL_AT = secs(2024, 1, 16);
const DAYS = 121;

// Banner-open days: every EN headhunting rotation that started inside the window.
const OPEN_DAYS = new Set(["2024-01-16", "2024-01-19", "2024-01-30", "2024-02-13", "2024-02-27", "2024-03-12", "2024-03-26", "2024-04-09", "2024-04-23", "2024-04-30", "2024-05-07", "2024-05-10"]);

let seed = 20240515;
const rnd = () => {
    seed = (seed * 1103515245 + 12345) % 2147483648;
    return seed / 2147483648;
};

const BY_DATE = Array.from({ length: DAYS }, (_, i) => {
    const date = new Date(utc(2024, 1, 16) + i * DAY_MS).toISOString().slice(0, 10);
    const base = 360 + Math.round(rnd() * 280);
    const spike = OPEN_DAYS.has(date) ? 2400 + Math.round(rnd() * 2100) : 0;
    return { date, pullCount: base + spike };
});

// Real /gacha/stats/enhanced hour + day-of-week distributions.
const BY_HOUR = [3662, 2765, 3136, 2672, 2800, 5168, 3814, 4455, 3682, 3622, 2848, 15490, 9127, 6061, 6493, 61601, 18379, 12962, 8856, 6220, 4002, 4892, 4367, 3596].map((pullCount, hour) => ({ hour, pullCount, percentage: pullCount / 200670 }));

const BY_DOW = [
    ["Sunday", 11494],
    ["Monday", 15602],
    ["Tuesday", 35352],
    ["Wednesday", 17451],
    ["Thursday", 74815],
    ["Friday", 32213],
    ["Saturday", 13743],
].map(([dayName, pullCount], day) => ({ day, dayName: dayName as string, pullCount: pullCount as number, percentage: (pullCount as number) / 200670 }));

const TIMING = { byHour: BY_HOUR, byDayOfWeek: BY_DOW, byDate: BY_DATE };

// Real EN banner runs across the same window, keyed the way /static/banners serves them.
const BANNERS = (
    [
        ["CLASSIC_EN_25_0_2", "Rare Operators useful in all kinds of stages", "CLASSIC", [2024, 1, 16], [2024, 1, 30]],
        ["LIMITED_EN_25_0_5", "Cloudtop Lucid Dreams", "LIMITED", [2024, 1, 16], [2024, 1, 30]],
        ["NORM_EN_25_0_6", "Rare Operators useful in all kinds of stages", "NORMAL", [2024, 1, 19], [2024, 2, 2]],
        ["CLASSIC_EN_25_0_3", "Rare Operators useful in all kinds of stages", "CLASSIC", [2024, 1, 30], [2024, 2, 13]],
        ["NORM_EN_25_0_7", "Rare Operators useful in all kinds of stages", "NORMAL", [2024, 2, 2], [2024, 2, 16]],
        ["NORM_EN_25_0_8", "Joint Operation", "NORMAL", [2024, 2, 12], [2024, 2, 26]],
        ["FESCLASSIC_EN_25_0_4", "Rare Operators useful in all kinds of stages", "FESCLASSIC", [2024, 2, 13], [2024, 2, 27]],
        ["NORM_EN_25_0_9", "Rare Operators useful in all kinds of stages", "NORMAL", [2024, 2, 16], [2024, 3, 1]],
        ["CLASSIC_EN_25_0_5", "Rare Operators useful in all kinds of stages", "CLASSIC", [2024, 2, 27], [2024, 3, 12]],
        ["SINGLE_EN_26_0_1", "The Woe-cleansing Hurricane", "SINGLE", [2024, 2, 27], [2024, 3, 12]],
        ["NORM_EN_26_0_2", "Rare Operators useful in all kinds of stages", "NORMAL", [2024, 3, 1], [2024, 3, 15]],
        ["SINGLE_EN_26_0_3", "Pathfinder of Sands", "SINGLE", [2024, 3, 5], [2024, 3, 19]],
        ["CLASSIC_EN_26_0_1", "Rare Operators useful in all kinds of stages", "CLASSIC", [2024, 3, 12], [2024, 3, 26]],
        ["NORM_EN_26_0_4", "Rare Operators useful in all kinds of stages", "NORMAL", [2024, 3, 15], [2024, 3, 29]],
        ["SINGLE_EN_26_0_5", "Clank Liberty", "SINGLE", [2024, 3, 19], [2024, 4, 2]],
        ["CLASSIC_EN_26_0_2", "Rare Operators useful in all kinds of stages", "CLASSIC", [2024, 3, 26], [2024, 4, 9]],
        ["NORM_EN_26_0_6", "Rare Operators useful in all kinds of stages", "NORMAL", [2024, 3, 29], [2024, 4, 12]],
        ["NORM_EN_26_0_7", "The Front That Was", "NORMAL", [2024, 4, 2], [2024, 4, 16]],
        ["CLASSIC_EN_26_0_3", "Rare Operators useful in all kinds of stages", "CLASSIC", [2024, 4, 9], [2024, 4, 23]],
        ["NORM_EN_26_0_8", "Rare Operators useful in all kinds of stages", "NORMAL", [2024, 4, 12], [2024, 4, 26]],
        ["SINGLE_EN_27_0_1", "From Gleams And Smoke I Emerge", "SINGLE", [2024, 4, 16], [2024, 4, 30]],
        ["CLASSIC_EN_27_0_1", "Rare Operators useful in all kinds of stages", "CLASSIC", [2024, 4, 23], [2024, 5, 7]],
        ["NORM_EN_27_0_2", "Rare Operators useful in all kinds of stages", "NORMAL", [2024, 4, 26], [2024, 5, 10]],
        ["LIMITED_EN_27_0_3", "By My Will", "LIMITED", [2024, 4, 30], [2024, 5, 14]],
        ["CLASSIC_EN_27_0_2", "Rare Operators useful in all kinds of stages", "CLASSIC", [2024, 5, 7], [2024, 5, 21]],
        ["NORM_EN_27_0_4", "Rare Operators useful in all kinds of stages", "NORMAL", [2024, 5, 10], [2024, 5, 24]],
    ] as Array<[string, string, string, number[], number[]]>
).map(([gachaPoolId, gachaPoolName, gachaRuleType, o, e], gachaIndex) => ({
    gachaPoolId,
    gachaPoolName,
    gachaRuleType,
    gachaIndex,
    openTime: secs(o[0], o[1], o[2]),
    endTime: secs(e[0], e[1], e[2]),
    gachaPoolSummary: "-",
    gachaPoolDetail: null,
    guarantee5Avail: 1,
    guarantee5Count: 10,
    featured6: [],
    featured5: [],
}));

export const Default = () => <TimingPanel timing={TIMING} firstPullAt={FIRST_PULL_AT} banners={BANNERS} />;

/** Same series with no static banner metadata loaded: the chart keeps its shape, the run tracks drop out. */
export const WithoutBannerTracks = () => <TimingPanel timing={TIMING} firstPullAt={FIRST_PULL_AT} banners={[]} />;

/** The backend can answer without the per-date breakdown; only the hour and weekday panels render. */
export const HourAndWeekdayOnly = () => <TimingPanel timing={{ byHour: BY_HOUR, byDayOfWeek: BY_DOW }} firstPullAt={FIRST_PULL_AT} banners={BANNERS} />;
