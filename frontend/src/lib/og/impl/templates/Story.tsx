import type { ReactNode } from "react";
import type { IStoryOgData } from "../story";
import { AccentStrip, BrandRow, FG, FG_06, FG_08, FG_45, FG_55, FG_70, FootRow, siteHost } from "./Frame";

/** Han, kana and hangul set about twice as wide as Latin at the same size, so they count double when the title picks its size. */
const WIDE_GLYPH = /[ᄀ-ᇿ⺀-鿿가-힯豈-﫿＀-￯]/;

function visualLength(text: string): number {
    let n = 0;
    for (const ch of text) n += WIDE_GLYPH.test(ch) ? 1.9 : 1;
    return n;
}

/** Step the title down for longer names so it holds the 640px column in at most two lines. */
function nameFontSize(name: string): number {
    const len = visualLength(name);
    if (len <= 14) return 84;
    if (len <= 22) return 68;
    if (len <= 32) return 56;
    return 46;
}

function Chip({ children, color, border, background }: { children: ReactNode; color: string; border: string; background: string }) {
    return (
        <div
            style={{
                display: "flex",
                alignItems: "center",
                gap: 8,
                fontFamily: "Geist Mono",
                fontSize: 13,
                letterSpacing: "0.14em",
                textTransform: "uppercase",
                color,
                background,
                border: `1px solid ${border}`,
                padding: "7px 12px",
                borderRadius: 999,
                lineHeight: 1,
            }}
        >
            {children}
        </div>
    );
}

/**
 * How far a plate is drawn past its box on every side. The 17 mainline key
 * visuals are posters with a white keyline baked into their edge, which a
 * cover-fit at 1.76x draws as a 1px rule down the middle of the card; 24px
 * crops it, and costs an event plate 3.2% of its width.
 */
const PLATE_BLEED = 24;

/** The visual side. A plate (key visual or cover) is cover-fit to the right 760px; an operator is contained and top-anchored, the way the operator card draws one; no art draws the watermark. */
function Art({ data }: { data: IStoryOgData }) {
    const { artURL, artKind, artPosition, accent, watermark } = data;
    if (artURL && artKind === "operator") {
        return (
            <div style={{ position: "absolute", right: -40, top: -30, width: 760, height: 760, display: "flex" }}>
                <div style={{ position: "absolute", left: 0, top: 0, width: 760, height: 760, display: "flex", background: `radial-gradient(circle at 50% 40%, ${accent}2e, transparent 55%)` }} />
                <img alt="" src={artURL} width={760} height={760} style={{ width: 760, height: 760, objectFit: "contain", objectPosition: "center top" }} />
            </div>
        );
    }
    if (artURL) {
        return (
            <div style={{ position: "absolute", left: 440, top: 0, width: 760, height: 630, display: "flex", overflow: "hidden" }}>
                <img alt="" src={artURL} width={760 + 2 * PLATE_BLEED} height={630 + 2 * PLATE_BLEED} style={{ position: "absolute", left: -PLATE_BLEED, top: -PLATE_BLEED, width: 760 + 2 * PLATE_BLEED, height: 630 + 2 * PLATE_BLEED, objectFit: "cover", objectPosition: artPosition }} />
            </div>
        );
    }
    return (
        <div
            style={{
                position: "absolute",
                right: 56,
                top: 0,
                bottom: 0,
                display: "flex",
                alignItems: "center",
                fontFamily: "Geist Mono",
                fontWeight: 700,
                fontSize: 200,
                letterSpacing: "-0.04em",
                color: "rgba(255,255,255,0.045)",
                lineHeight: 1,
            }}
        >
            {watermark}
        </div>
    );
}

export function StoryTemplate(data: IStoryOgData) {
    const { name, groupName, categoryLabel, chapterTag, code, phase, cutsceneLabel, noScriptLabel, stats, accent } = data;
    const hasChips = Boolean(code || phase || cutsceneLabel || noScriptLabel);

    return (
        <div
            style={{
                width: 1200,
                height: 630,
                display: "flex",
                background: `radial-gradient(ellipse 70% 60% at 82% 35%, ${accent}1f, transparent 65%), linear-gradient(135deg, #0e141a 0%, #15202a 60%, #0c1218 100%)`,
                color: FG,
                fontFamily: "Inter",
                position: "relative",
                overflow: "hidden",
            }}
        >
            <Art data={data} />
            <div
                style={{
                    position: "absolute",
                    left: 0,
                    top: 0,
                    width: 1200,
                    height: 630,
                    display: "flex",
                    background: "linear-gradient(90deg, #0d1216 0%, rgba(13,18,22,0.96) 36%, rgba(13,18,22,0.6) 55%, rgba(13,18,22,0.1) 78%, transparent 100%), linear-gradient(0deg, #0d1216 0%, transparent 28%)",
                }}
            />
            <div
                style={{
                    position: "absolute",
                    left: 0,
                    top: 0,
                    width: 760,
                    height: 630,
                    padding: "56px 64px",
                    display: "flex",
                    flexDirection: "column",
                }}
            >
                <BrandRow kicker="STORY · COMPANION" />
                <div style={{ display: "flex", flexDirection: "column", flex: 1, justifyContent: "center", marginTop: 24 }}>
                    {hasChips ? (
                        <div style={{ display: "flex", alignItems: "center", gap: 12, marginBottom: 22 }}>
                            {code ? (
                                <div
                                    style={{
                                        display: "flex",
                                        padding: "8px 16px",
                                        borderRadius: 8,
                                        background: accent,
                                        color: "#0b1116",
                                        fontFamily: "Geist Mono",
                                        fontWeight: 700,
                                        fontSize: 26,
                                        letterSpacing: "0.02em",
                                        lineHeight: 1,
                                    }}
                                >
                                    {code}
                                </div>
                            ) : null}
                            {phase ? (
                                <Chip color={FG_70} border="rgba(255,255,255,0.16)" background={FG_06}>
                                    {phase}
                                </Chip>
                            ) : null}
                            {cutsceneLabel ? (
                                <Chip color="#f0c9a3" border="rgba(236,111,93,0.45)" background="rgba(236,111,93,0.12)">
                                    {/* biome-ignore lint/a11y/noSvgWithoutTitle: Satori renders <title> as visible text */}
                                    <svg width="11" height="12" viewBox="0 0 11 12" fill="#f0c9a3">
                                        <path d="M0 0.8v10.4c0 .62.68 1 1.2.66l8.9-5.2a.77.77 0 0 0 0-1.32L1.2.14C.68-.2 0 .18 0 .8z" />
                                    </svg>
                                    <div style={{ display: "flex" }}>{cutsceneLabel}</div>
                                </Chip>
                            ) : null}
                            {noScriptLabel ? (
                                <Chip color={FG_55} border="rgba(255,255,255,0.14)" background="transparent">
                                    {noScriptLabel}
                                </Chip>
                            ) : null}
                        </div>
                    ) : null}
                    <div
                        style={{
                            display: "flex",
                            fontSize: nameFontSize(name),
                            fontWeight: 700,
                            lineHeight: 1.02,
                            letterSpacing: "-0.025em",
                            color: FG,
                            maxWidth: 640,
                        }}
                    >
                        {name}
                    </div>
                    <div
                        style={{
                            display: "flex",
                            fontSize: 28,
                            fontWeight: 400,
                            lineHeight: 1.25,
                            color: FG_70,
                            marginTop: 16,
                            maxWidth: 620,
                        }}
                    >
                        {groupName}
                    </div>
                    <div
                        style={{
                            display: "flex",
                            gap: 10,
                            alignItems: "center",
                            fontFamily: "Geist Mono",
                            fontSize: 14,
                            letterSpacing: "0.14em",
                            textTransform: "uppercase",
                            color: FG_55,
                            marginTop: 14,
                        }}
                    >
                        <div style={{ display: "flex" }}>{categoryLabel}</div>
                        {chapterTag ? <div style={{ display: "flex", color: FG_45 }}>·</div> : null}
                        {chapterTag ? <div style={{ display: "flex", color: FG_70 }}>{chapterTag}</div> : null}
                    </div>
                    {stats.length > 0 ? (
                        <div
                            style={{
                                display: "flex",
                                marginTop: 30,
                                background: FG_06,
                                border: `1px solid ${FG_08}`,
                                borderRadius: 12,
                                overflow: "hidden",
                                width: 150 * stats.length + 60,
                                maxWidth: 600,
                            }}
                        >
                            {stats.map((s, i) => (
                                <div
                                    key={s.label}
                                    style={{
                                        flex: 1,
                                        display: "flex",
                                        flexDirection: "column",
                                        gap: 6,
                                        padding: "14px 18px",
                                        borderLeft: i === 0 ? "none" : `1px solid ${FG_08}`,
                                    }}
                                >
                                    <div style={{ display: "flex", fontFamily: "Geist Mono", fontSize: 10, letterSpacing: "0.22em", textTransform: "uppercase", color: FG_55 }}>{s.label}</div>
                                    <div style={{ display: "flex", fontSize: 24, fontWeight: 700, color: FG, lineHeight: 1, letterSpacing: "-0.01em" }}>{s.value}</div>
                                </div>
                            ))}
                        </div>
                    ) : null}
                </div>
                <div style={{ display: "flex", height: 1, background: `linear-gradient(90deg, ${accent}80, transparent 70%)` }} />
            </div>
            <FootRow path={`${siteHost()} / stories`} />
            <AccentStrip color={accent} />
        </div>
    );
}
