import { Frame } from "./Frame";

export interface DefaultOgData {
    title: string;
    subtitle?: string;
}

export function DefaultTemplate(data: DefaultOgData) {
    return (
        <Frame>
            <div style={{ display: "flex", flexDirection: "column", gap: 18 }}>
                <div style={{ fontSize: 96, fontWeight: 700, lineHeight: 1.05 }}>{data.title}</div>
                {data.subtitle ? <div style={{ fontSize: 32, opacity: 0.7, maxWidth: 900 }}>{data.subtitle}</div> : null}
            </div>
        </Frame>
    );
}
