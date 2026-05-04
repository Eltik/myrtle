import { RARITY_COLORS } from "#/components/operators/list/impl/constants";
import type { OperatorProfession, OperatorRarity } from "#/types/operators";
import { Frame } from "./Frame";

export interface OperatorOgData {
    name: string;
    appellation: string;
    profession: OperatorProfession;
    rarity: OperatorRarity;
}

export function OperatorTemplate(data: OperatorOgData) {
    const accent = RARITY_COLORS[data.rarity] ?? "#5B8DEF";
    return (
        <Frame accent={accent}>
            <div style={{ display: "flex", flexDirection: "column", gap: 18 }}>
                <div style={{ display: "flex", alignItems: "center", gap: 16 }}>
                    <div style={{ display: "flex", padding: "6px 14px", borderRadius: 999, background: accent, color: "#0F1729", fontSize: 22, fontWeight: 700 }}>{data.rarity}★</div>
                    <div style={{ fontSize: 26, opacity: 0.7 }}>{data.profession}</div>
                </div>
                <div style={{ fontSize: 96, fontWeight: 700, lineHeight: 1.05 }}>{data.name}</div>
                {data.appellation ? <div style={{ fontSize: 32, opacity: 0.7 }}>{data.appellation}</div> : null}
            </div>
        </Frame>
    );
}
