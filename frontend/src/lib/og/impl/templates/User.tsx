import { Frame } from "./Frame";

export interface UserOgData {
    nickname: string;
    level: number | null;
    grade: string | null;
    totalScore: number | null;
}

export function UserTemplate(data: UserOgData) {
    return (
        <Frame accent="#10B981">
            <div style={{ display: "flex", flexDirection: "column", gap: 16 }}>
                <div style={{ fontSize: 28, opacity: 0.7 }}>Doctor</div>
                <div style={{ fontSize: 96, fontWeight: 700, lineHeight: 1.05 }}>{data.nickname}</div>
                <div style={{ display: "flex", gap: 36, marginTop: 12 }}>
                    {data.level != null && <Stat label="Level" value={String(data.level)} />}
                    {data.grade && <Stat label="Grade" value={data.grade} />}
                    {data.totalScore != null && <Stat label="Score" value={data.totalScore.toLocaleString("en-US")} />}
                </div>
            </div>
        </Frame>
    );
}

function Stat({ label, value }: { label: string; value: string }) {
    return (
        <div style={{ display: "flex", flexDirection: "column", gap: 4 }}>
            <div style={{ fontSize: 18, opacity: 0.55, letterSpacing: 1 }}>{label.toUpperCase()}</div>
            <div style={{ fontSize: 40, fontWeight: 700 }}>{value}</div>
        </div>
    );
}
