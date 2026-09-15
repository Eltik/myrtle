import { DEFAULT_OG_TAGS } from "../presets";
import { AccentStrip, BrandRow, FG, FG_06, FG_08, FG_55, FootRow } from "./Frame";

export interface IDefaultOgData {
    title: string;
    subtitle?: string;
    /** A chip id from `DEFAULT_OG_TAGS`, not a label - see `tagLabels`. */
    activeTag?: string;
    /**
     * Chip id -> the label to draw, for the active locale. Satori renders this
     * on the server with no React tree, so there is no context to read a
     * catalog from: the handler resolves the five labels and passes them in.
     * Absent, the ids double as their own English labels.
     */
    tagLabels?: Record<string, string>;
}

export function DefaultTemplate(data: IDefaultOgData) {
    const { title, subtitle, activeTag = "Home", tagLabels } = data;

    return (
        <div
            style={{
                width: 1200,
                height: 630,
                display: "flex",
                flexDirection: "column",
                background: `radial-gradient(ellipse 60% 50% at 50% 50%, rgba(96,215,207,0.08), transparent 70%), radial-gradient(ellipse 80% 60% at 80% 20%, rgba(236,111,93,0.10), transparent 70%), linear-gradient(135deg, #0d1419 0%, #131c24 100%)`,
                color: FG,
                fontFamily: "Inter",
                position: "relative",
                padding: "56px 64px",
            }}
        >
            <BrandRow kicker="ARKNIGHTS · COMPANION" />

            <div style={{ display: "flex", flexDirection: "column", flex: 1, justifyContent: "center" }}>
                <div
                    style={{
                        display: "flex",
                        fontSize: 96,
                        fontWeight: 700,
                        lineHeight: 0.95,
                        letterSpacing: "-0.028em",
                        color: FG,
                        maxWidth: 980,
                    }}
                >
                    {title}
                </div>
                {subtitle ? <div style={{ display: "flex", fontSize: 28, color: FG_55, marginTop: 24, maxWidth: 880, lineHeight: 1.4 }}>{subtitle}</div> : null}

                <div style={{ display: "flex", gap: 10, marginTop: 36 }}>
                    {DEFAULT_OG_TAGS.map((t) => {
                        const active = t === activeTag;
                        return (
                            <div
                                key={t}
                                style={{
                                    display: "flex",
                                    fontFamily: "Geist Mono",
                                    fontSize: 13,
                                    letterSpacing: "0.18em",
                                    textTransform: "uppercase",
                                    color: active ? "#ec6f5d" : FG_55,
                                    background: active ? "rgba(236,111,93,0.08)" : FG_06,
                                    border: `1px solid ${active ? "rgba(236,111,93,0.4)" : FG_08}`,
                                    padding: "8px 14px",
                                    borderRadius: 8,
                                }}
                            >
                                {tagLabels?.[t] ?? t}
                            </div>
                        );
                    })}
                </div>
            </div>

            <FootRow />
            <AccentStrip />
        </div>
    );
}
