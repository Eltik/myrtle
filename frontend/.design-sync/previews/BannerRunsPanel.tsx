import { useEffect, useRef } from "react";
import { BannerRunsPanel } from "frontend";

const secs = (y: number, m: number, d: number) => Math.floor(Date.UTC(y, m - 1, d) / 1000);

const INDEX: Array<[string, string]> = [
    ["char_245_cello", "Virtuosa"],
    ["char_2013_cerber", "Ceobe"],
    ["char_358_lisa", "Suzuran"],
    ["char_1016_agoat2", "Eyjafjalla the Hvít Aska"],
    ["char_222_bpipe", "Bagpipe"],
    ["char_136_hsguma", "Hoshiguma"],
    ["char_112_siege", "Siege"],
    ["char_134_ifrit", "Ifrit"],
    ["char_202_demkni", "Saria"],
    ["char_263_skadi", "Skadi"],
    ["char_010_chen", "Ch'en"],
    ["char_340_shwaz", "Schwarz"],
    ["char_188_helage", "Hellagur"],
    ["char_248_mgllan", "Magallan"],
    ["char_213_mostma", "Mostima"],
    ["char_225_haak", "Aak"],
    ["char_250_phatom", "Phantom"],
    ["char_197_poca", "Rosa"],
];
const OPERATORS_BY_ID = new Map(INDEX.map(([id, name]) => [id, { id, name, profession: "WARRIOR" }]));

type Row = [string, string, string, number[], number[], string[], string];

// Real EN banner identities. The run windows are set so the mix a doctor sees
// mid-rotation (limited + collab + kernel + two standard pools) is live on the
// capture clock's 2024-05-15.
const ROWS: Row[] = [
    ["LIMITED_EN_27_0_3", "By My Will", "LIMITED", [2024, 5, 2], [2024, 5, 16], ["char_245_cello"], "Ends at 03:59, 5/16"],
    ["LINKAGE_EN_29_0_7", "Storm, Reinforce, Missions Cycle", "LINKAGE", [2024, 5, 9], [2024, 5, 23], [], "Ends May 23 03:59"],
    ["CLASSIC_EN_27_0_2", "Rare Operators useful in all kinds of stages", "CLASSIC", [2024, 5, 7], [2024, 5, 21], ["char_2013_cerber", "char_358_lisa"], "Ends May 21 03:59"],
    ["NORM_EN_27_0_4", "Rare Operators useful in all kinds of stages", "NORMAL", [2024, 5, 10], [2024, 5, 24], [], "Ends at 03:59, 5/24"],
    ["SINGLE_EN_27_0_1", "From Gleams And Smoke I Emerge", "SINGLE", [2024, 5, 3], [2024, 5, 17], [], "Ends at 03:59, 5/17"],
    ["FESCLASSIC_EN_27_0_3", "Rare Operators useful in all kinds of stages", "FESCLASSIC", [2024, 5, 21], [2024, 6, 4], ["char_112_siege", "char_134_ifrit", "char_202_demkni", "char_263_skadi", "char_010_chen", "char_340_shwaz", "char_188_helage", "char_248_mgllan", "char_213_mostma", "char_225_haak", "char_250_phatom", "char_197_poca"], "Ends June 4 03:59"],
    ["CLASSIC_EN_27_0_1", "Rare Operators useful in all kinds of stages", "CLASSIC", [2024, 4, 23], [2024, 5, 7], ["char_222_bpipe", "char_136_hsguma"], "Ends May 7 03:59"],
    ["NORM_EN_27_0_2", "Rare Operators useful in all kinds of stages", "NORMAL", [2024, 4, 26], [2024, 5, 10], [], "Ends at 03:59, 5/10"],
    ["LIMITED_EN_25_0_5", "Cloudtop Lucid Dreams", "LIMITED", [2024, 1, 16], [2024, 1, 30], ["char_1016_agoat2"], "Ends January 30 03:59"],
];

const BANNERS = ROWS.map(([gachaPoolId, gachaPoolName, gachaRuleType, o, e, featured6, gachaPoolSummary], gachaIndex) => ({
    gachaPoolId,
    gachaPoolName,
    gachaRuleType,
    gachaIndex,
    openTime: secs(o[0], o[1], o[2]),
    endTime: secs(e[0], e[1], e[2]),
    gachaPoolSummary,
    gachaPoolDetail: null,
    guarantee5Avail: 1,
    guarantee5Count: 10,
    guaranteeName: null,
    featured6,
    featured5: [],
}));

const STATS = new Map(
    (
        [
            ["LIMITED_EN_27_0_3", 24812, 702, 2280, 431],
            ["CLASSIC_EN_27_0_2", 9140, 264, 851, 288],
            ["NORM_EN_27_0_4", 6038, 171, 559, 246],
            ["SINGLE_EN_27_0_1", 11402, 322, 1063, 302],
            ["CLASSIC_EN_27_0_1", 7705, 219, 706, 271],
            ["NORM_EN_27_0_2", 5120, 144, 474, 219],
            ["LIMITED_EN_25_0_5", 18664, 528, 1719, 388],
        ] as Array<[string, number, number, number, number]>
    ).map(([poolId, pullCount, sixStarCount, fiveStarCount, userCount]) => [poolId, { poolId, pullCount, sixStarCount, fiveStarCount, userCount }]),
);

/** Presses one of the panel's own segmented buttons after first paint. */
const PressFilter = ({ label, children }: { label: string; children: React.ReactNode }) => {
    const ref = useRef<HTMLDivElement>(null);
    useEffect(() => {
        let inner = 0;
        const outer = requestAnimationFrame(() => {
            inner = requestAnimationFrame(() => {
                const btn = [...(ref.current?.querySelectorAll("button") ?? [])].find((b) => b.textContent?.trim() === label);
                (btn as HTMLButtonElement | undefined)?.click();
            });
        });
        return () => {
            cancelAnimationFrame(outer);
            cancelAnimationFrame(inner);
        };
    }, [label]);
    return <div ref={ref}>{children}</div>;
};

/** Default filter is "Active": the five pools a doctor can actually pull on today. */
export const ActiveRuns = () => <BannerRunsPanel banners={BANNERS} operatorsById={OPERATORS_BY_ID} statsById={STATS} isLoading={false} />;

/** Every run in the 180-day window: active first, then upcoming, then most-recently-ended. */
export const AllRuns = () => (
    <PressFilter label="All">
        <BannerRunsPanel banners={BANNERS} operatorsById={OPERATORS_BY_ID} statsById={STATS} isLoading={false} />
    </PressFilter>
);

/** Between rotations: nothing is live, so the Active filter lands on its empty branch. */
export const NoActiveRuns = () => <BannerRunsPanel banners={BANNERS.filter((b) => b.endTime < secs(2024, 5, 15) || b.openTime > secs(2024, 5, 15))} operatorsById={OPERATORS_BY_ID} statsById={STATS} isLoading={false} />;

export const Loading = () => <BannerRunsPanel banners={[]} operatorsById={OPERATORS_BY_ID} statsById={STATS} isLoading={true} />;
