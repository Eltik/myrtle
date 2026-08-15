import { UserCard } from "frontend";

const DOCTORS = [
    { uid: "10148372", nickname: "Kal'tsit", level: 120, avatar_id: "char_003_kalts", server: "en", grade: "SS+", total_score: 184_920, operator_count: 289, skin_count: 214 },
    { uid: "20419865", nickname: "Amiya", level: 118, avatar_id: "char_002_amiya", server: "en", grade: "SS", total_score: 162_540, operator_count: 271, skin_count: 168 },
    { uid: "31820447", nickname: "Doctor Rhine", level: 104, avatar_id: "char_263_skadi", server: "cn", grade: "S", total_score: 121_308, operator_count: 244, skin_count: 96 },
    { uid: "40277519", nickname: "Chernobog", level: 92, avatar_id: "char_1028_texas2", server: "jp", grade: "A", total_score: 88_760, operator_count: 201, skin_count: 54 },
];

export const TopRanked = () => (
    <div className="w-full max-w-sm">
        <UserCard user={DOCTORS[0]} />
    </div>
);

export const SkinAvatarAndServer = () => (
    <div className="w-full max-w-sm">
        <UserCard
            user={{
                uid: "18820913",
                nickname: "Ursus Student",
                level: 111,
                avatar_id: "char_1012_skadi2@boc#4",
                server: "cn",
                grade: "SS",
                total_score: 149_112,
                operator_count: 258,
                skin_count: 133,
            }}
        />
    </div>
);

export const UnnamedDoctor = () => (
    <div className="w-full max-w-sm">
        <UserCard
            user={{
                uid: "50931204",
                nickname: null,
                level: 34,
                avatar_id: null,
                server: "en",
                grade: null,
                total_score: 4_180,
                operator_count: 46,
                skin_count: null,
            }}
        />
    </div>
);

export const ResultsGrid = () => (
    <div className="grid w-full grid-cols-1 gap-3 sm:grid-cols-2">
        {DOCTORS.map((user) => (
            <UserCard key={`${user.uid}-${user.server}`} user={user} />
        ))}
    </div>
);
