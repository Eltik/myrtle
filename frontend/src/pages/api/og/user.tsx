import { ImageResponse } from "@vercel/og";
import type { NextRequest } from "next/server";

export const config = {
    runtime: "edge",
};

// Site color scheme (dark mode) - converted from oklch to hex
const COLORS = {
    background: "#121214",
    card: "#1c1c1f",
    foreground: "#f5f5f7",
    primary: "#e07355",
    border: "#2d2d32",
    muted: "#9a9aa0",
    secondary: "#252528",
    gold: "#FFD440",
};

// Rarity colors
const RARITY_COLORS: Record<number, string> = {
    1: "#9e9e9e",
    2: "#dce537",
    3: "#00b2eb",
    4: "#dbb1db",
    5: "#ffe65c",
    6: "#ff7e29",
};

function getBaseUrl(req: NextRequest): string {
    const host = req.headers.get("host");
    if (host?.includes("localhost")) {
        return `http://${host}`;
    }
    return "https://myrtle.moe";
}

function getCharartUrl(baseUrl: string, charId: string, skinId: string, evolvePhase: number): string {
    if (skinId.includes("@")) {
        const normalizedSkinId = skinId.replaceAll("@", "_").replaceAll("#", "%23");
        return `${baseUrl}/api/cdn/upk/skinpack/${charId}/${normalizedSkinId}.png`;
    }
    if (!skinId || skinId.endsWith("#1")) {
        const suffix = evolvePhase >= 2 ? "_2" : "_1";
        return `${baseUrl}/api/cdn/upk/chararts/${charId}/${charId}${suffix}.png`;
    }
    const normalizedSkinId = skinId.replaceAll("@", "_").replaceAll("#", "_");
    return `${baseUrl}/api/cdn/upk/chararts/${charId}/${normalizedSkinId}.png`;
}

function getAvatarUrl(baseUrl: string, charId: string, skinId: string | null): string {
    // No skin provided - use charId (backend resolves the correct file)
    if (!skinId) {
        return `${baseUrl}/api/cdn/avatar/${charId}`;
    }

    // Custom skins with @: Replace @ with _, encode # as %23
    if (skinId.includes("@")) {
        const normalizedSkinId = skinId.replaceAll("@", "_").replaceAll("#", "%23");
        return `${baseUrl}/api/cdn/avatar/${normalizedSkinId}`;
    }

    // Default skin (ends with #1 or _1): Use just charId (backend resolves correct file)
    if (skinId.endsWith("#1") || skinId.endsWith("_1")) {
        return `${baseUrl}/api/cdn/avatar/${charId}`;
    }

    // E2 skin (ends with #2 or _2): Normalize and use
    if (skinId.endsWith("#2") || skinId.endsWith("_2")) {
        const normalizedSkinId = skinId.replaceAll("#", "_");
        return `${baseUrl}/api/cdn/avatar/${normalizedSkinId}`;
    }

    // Other skins: Replace # with _
    const normalizedSkinId = skinId.replaceAll("#", "_");
    return `${baseUrl}/api/cdn/avatar/${normalizedSkinId}`;
}

// Icon URL patterns from /operators page analysis
function getEliteIconUrl(baseUrl: string, phase: number): string {
    return `${baseUrl}/api/cdn/upk/arts/elite_hub/elite_${phase}.png`;
}

function getPotentialIconUrl(baseUrl: string, rank: number): string {
    return `${baseUrl}/api/cdn/upk/arts/potential_hub/potential_${rank}.png`;
}

function getMasteryIconUrl(baseUrl: string, level: number): string {
    return `${baseUrl}/api/cdn/upk/arts/specialized_hub/specialized_${level}.png`;
}

function getModuleIconUrl(baseUrl: string, uniEquipIcon: string): string {
    return `${baseUrl}/api/cdn/upk/spritepack/ui_equip_big_img_hub_0/${uniEquipIcon}.png`;
}

function getSkillIconUrl(baseUrl: string, skillId: string): string {
    return `${baseUrl}/api/cdn/upk/spritepack/skill_icons_0/skill_icon_${skillId}.png`;
}

function parseRarity(rarity: string): number {
    const match = rarity.match(/TIER_(\d)/);
    return match?.[1] ? Number.parseInt(match[1], 10) : 1;
}

async function fetchImageAsBase64(url: string): Promise<string | null> {
    try {
        const response = await fetch(url);
        if (!response.ok) return null;
        const buffer = await response.arrayBuffer();
        return `data:image/png;base64,${Buffer.from(buffer).toString("base64")}`;
    } catch {
        return null;
    }
}

interface SkillData {
    skillId: string;
    specializeLevel: number; // mastery (0-3)
    skillLevel: number; // main skill level (1-7)
    iconBase64: string | null;
    masteryIconBase64: string | null;
}

interface ModuleData {
    uniEquipId: string;
    uniEquipIcon: string;
    level: number; // 0 = not unlocked, 1-3 = upgrade level
    locked: boolean; // true if locked (locked === 1 from API)
    iconBase64: string | null;
}

interface SupportUnit {
    name: string;
    level: number;
    potential: number;
    avatarBase64: string | null;
    rarity: number;
    evolvePhase: number;
    // Pre-fetched icon base64
    eliteIconBase64: string | null;
    potentialIconBase64: string | null;
    // All skills for the operator
    skills: SkillData[];
    // All available modules for the operator (up to 3)
    modules: ModuleData[];
}

async function generateFallbackImage() {
    return new ImageResponse(
        <div
            style={{
                height: "100%",
                width: "100%",
                display: "flex",
                flexDirection: "column",
                alignItems: "center",
                justifyContent: "center",
                backgroundColor: COLORS.background,
            }}
        >
            <span style={{ fontSize: "48px", fontWeight: "bold", color: COLORS.foreground }}>myrtle.moe</span>
            <span style={{ fontSize: "24px", color: COLORS.muted, marginTop: "16px" }}>Arknights Profile</span>
        </div>,
        { width: 1200, height: 630 },
    );
}

export default async function handler(req: NextRequest) {
    const { searchParams } = new URL(req.url);
    const userId = searchParams.get("id");
    const baseUrl = getBaseUrl(req);

    // Load fonts
    let fontData: ArrayBuffer | undefined;
    let fontBoldData: ArrayBuffer | undefined;

    const fetchFont = async (urls: string[]): Promise<ArrayBuffer | undefined> => {
        for (const url of urls) {
            try {
                const res = await fetch(url, { cache: "force-cache" });
                if (!res.ok) continue;
                const buffer = await res.arrayBuffer();
                const view = new Uint8Array(buffer.slice(0, 4));
                const signature = String.fromCharCode(...view);
                if (signature.startsWith("<!DO") || signature.startsWith("<htm")) continue;
                return buffer;
            } catch {
                // Try next URL
            }
        }
        return undefined;
    };

    try {
        const regularUrls = ["https://github.com/rsms/inter/raw/master/docs/font-files/Inter-Regular.woff", "https://raw.githubusercontent.com/rsms/inter/master/docs/font-files/Inter-Regular.woff"];
        const boldUrls = ["https://github.com/rsms/inter/raw/master/docs/font-files/Inter-Bold.woff", "https://raw.githubusercontent.com/rsms/inter/master/docs/font-files/Inter-Bold.woff"];
        [fontData, fontBoldData] = await Promise.all([fetchFont(regularUrls), fetchFont(boldUrls)]);
    } catch {
        // Continue without custom font
    }

    if (!userId) {
        return generateFallbackImage();
    }

    try {
        const backendUrl = process.env.BACKEND_URL;
        const response = await fetch(`${backendUrl}/get-user?uid=${userId}`, {
            headers: { "Content-Type": "application/json" },
        });

        if (!response.ok) {
            return generateFallbackImage();
        }

        const userData = await response.json();
        const { data } = userData;

        if (!data?.status) {
            return generateFallbackImage();
        }

        const nickName = data.status.nickName || "Unknown";
        const level = data.status.level || 1;
        const secretary = data.status.secretary;
        const secretarySkinId = data.status.secretarySkinId;

        const chars = data.troop?.chars || {};
        const charsArray = Object.values(chars) as Array<{
            instId: number;
            charId: string;
            evolvePhase: number;
            skin: string;
            level: number;
            potentialRank: number;
            skills: Array<{
                specializeLevel: number;
                skillId: string;
                static?: { iconId?: string; skillId?: string; image?: string };
            }>;
            currentEquip: string | null;
            equip: Record<string, { level: number; locked?: number }>;
            static?: {
                name: string;
                rarity: string;
                modules?: Array<{ uniEquipId: string; typeName1: string; uniEquipIcon?: string; image?: string }>;
            };
        }>;

        const secretaryChar = charsArray.find((c) => c.charId === secretary);
        const evolvePhase = secretaryChar?.evolvePhase ?? 2;

        // Fetch artwork
        const artworkUrl = getCharartUrl(baseUrl, secretary, secretarySkinId, evolvePhase);
        const artworkBase64 = await fetchImageAsBase64(artworkUrl);

        // Get support units with icons
        const assistCharList = data.social?.assistCharList || [];
        const supportUnits: SupportUnit[] = await Promise.all(
            assistCharList
                .filter((assist: { charInstId: number } | null) => assist !== null)
                .slice(0, 3)
                .map(async (assist: { charInstId: number; skillIndex: number }) => {
                    const charData = charsArray.find((c) => c.instId === assist.charInstId) || (chars[assist.charInstId.toString()] as (typeof charsArray)[0]);
                    if (!charData) return null;

                    const charId = charData.charId;
                    const name = charData.static?.name || charId.replace("char_", "").split("_").pop() || "Unknown";
                    const rarity = parseRarity(charData.static?.rarity || "TIER_1");
                    const charLevel = charData.level || 1;
                    const potentialRank = charData.potentialRank || 0; // 0-indexed for URL (0-5)
                    const potential = potentialRank + 1; // 1-indexed for display (1-6)
                    const charEvolvePhase = charData.evolvePhase || 0;
                    const mainSkillLvl = (charData as { mainSkillLvl?: number }).mainSkillLvl || 7;

                    // Get all available modules (excluding ORIGINAL type)
                    const availableModules = charData.static?.modules?.filter((m) => m.typeName1 !== "ORIGINAL") || [];

                    // Fetch base images in parallel
                    const [avatarBase64, eliteIconBase64, potentialIconBase64] = await Promise.all([
                        fetchImageAsBase64(getAvatarUrl(baseUrl, charId, charData.skin)),
                        charEvolvePhase >= 2 ? fetchImageAsBase64(getEliteIconUrl(baseUrl, 2)) : charEvolvePhase === 1 ? fetchImageAsBase64(getEliteIconUrl(baseUrl, 1)) : Promise.resolve(null),
                        fetchImageAsBase64(getPotentialIconUrl(baseUrl, potentialRank)), // Use 0-indexed for URL
                    ]);

                    // Fetch all skill icons and mastery icons
                    const skillsData: SkillData[] = await Promise.all(
                        (charData.skills || []).map(async (skill) => {
                            // Use iconId from static data, fallback to skillId
                            const skillIconId = skill.static?.iconId ?? skill.static?.skillId ?? skill.skillId;
                            const skillImagePath = skill.static?.image;

                            // Use image field as primary source, fallback to skill icon URL
                            const iconUrl = skillImagePath ? `${baseUrl}/api/cdn${skillImagePath}` : getSkillIconUrl(baseUrl, skillIconId);

                            const [iconBase64, masteryIconBase64] = await Promise.all([fetchImageAsBase64(iconUrl), skill.specializeLevel > 0 ? fetchImageAsBase64(getMasteryIconUrl(baseUrl, skill.specializeLevel)) : Promise.resolve(null)]);

                            return {
                                skillId: skill.skillId,
                                specializeLevel: skill.specializeLevel || 0,
                                skillLevel: mainSkillLvl,
                                iconBase64,
                                masteryIconBase64,
                            };
                        }),
                    );

                    // Fetch all available modules (keep level 0 for placeholder display)
                    const modulesData: ModuleData[] = await Promise.all(
                        availableModules.slice(0, 3).map(async (module) => {
                            const equipData = charData.equip?.[module.uniEquipId];
                            const level = equipData?.level ?? 0;
                            const locked = equipData?.locked === 1;
                            const isUnlocked = level > 0 && !locked;

                            // Only fetch icon if module is actually unlocked
                            let iconBase64: string | null = null;
                            if (isUnlocked) {
                                const iconUrl = module.image ? `${baseUrl}/api/cdn${module.image}` : module.uniEquipIcon ? getModuleIconUrl(baseUrl, module.uniEquipIcon) : null;
                                iconBase64 = iconUrl ? await fetchImageAsBase64(iconUrl) : null;
                            }

                            return {
                                uniEquipId: module.uniEquipId,
                                uniEquipIcon: module.uniEquipIcon || "",
                                level,
                                locked,
                                iconBase64,
                            };
                        }),
                    );

                    return {
                        name,
                        level: charLevel,
                        potential,
                        avatarBase64,
                        rarity,
                        evolvePhase: charEvolvePhase,
                        eliteIconBase64,
                        potentialIconBase64,
                        skills: skillsData,
                        modules: modulesData,
                    };
                }),
        );

        const validSupportUnits = supportUnits.filter((u): u is SupportUnit => u !== null);

        // Calculate rarity distribution
        const rarityCount: Record<number, number> = { 1: 0, 2: 0, 3: 0, 4: 0, 5: 0, 6: 0 };
        for (const char of charsArray) {
            const rarity = parseRarity((char as { static?: { rarity: string } }).static?.rarity || "TIER_1");
            if (rarityCount[rarity] !== undefined) {
                rarityCount[rarity]++;
            }
        }
        const totalChars = Object.values(rarityCount).reduce((a, b) => a + b, 0);

        // Build fonts
        type FontWeight = 100 | 200 | 300 | 400 | 500 | 600 | 700 | 800 | 900;
        const fonts: { name: string; data: ArrayBuffer; weight: FontWeight; style: "normal" }[] = [];
        if (fontData) fonts.push({ name: "Inter", data: fontData, weight: 400 as FontWeight, style: "normal" });
        if (fontBoldData) fonts.push({ name: "Inter", data: fontBoldData, weight: 700 as FontWeight, style: "normal" });

        return new ImageResponse(
            <div
                style={{
                    height: "100%",
                    width: "100%",
                    display: "flex",
                    backgroundColor: COLORS.background,
                    fontFamily: "Inter, sans-serif",
                    position: "relative",
                }}
            >
                {/* Left side - Full artwork */}
                <div style={{ width: "60%", height: "100%", display: "flex", position: "relative", overflow: "hidden" }}>
                    {artworkBase64 && (
                        // biome-ignore lint/performance/noImgElement: Satori requires native HTML
                        <img
                            alt="Secretary"
                            src={artworkBase64}
                            style={{
                                position: "absolute",
                                top: "50%",
                                left: "45%",
                                transform: "translate(-50%, -50%)",
                                height: "140%",
                                objectFit: "contain",
                            }}
                        />
                    )}

                    {/* Gradient overlays */}
                    <div style={{ position: "absolute", bottom: 0, left: 0, right: 0, height: "180px", background: `linear-gradient(transparent, ${COLORS.background})`, display: "flex" }} />
                    <div style={{ position: "absolute", top: 0, right: 0, bottom: 0, width: "150px", background: `linear-gradient(to left, ${COLORS.background}, transparent)`, display: "flex" }} />

                    {/* Username bottom-left */}
                    <div style={{ position: "absolute", bottom: "24px", left: "24px", display: "flex", flexDirection: "column", alignItems: "flex-start", gap: "2px" }}>
                        <span style={{ fontSize: "56px", fontWeight: 700, color: COLORS.foreground, textShadow: "2px 2px 12px rgba(0,0,0,0.9)" }}>{nickName}</span>
                        <span style={{ fontSize: "22px", fontWeight: 500, color: COLORS.muted, letterSpacing: "0.5px", marginLeft: "2px" }}>Level {level}</span>
                    </div>
                </div>

                {/* Right side - Support units (full height) */}
                <div
                    style={{
                        width: "40%",
                        height: "100%",
                        backgroundColor: COLORS.card,
                        display: "flex",
                        flexDirection: "column",
                        borderLeft: `1px solid ${COLORS.border}`,
                    }}
                >
                    {/* Header with branding */}
                    <div style={{ padding: "12px 16px", borderBottom: `1px solid ${COLORS.border}`, display: "flex", alignItems: "center", justifyContent: "space-between" }}>
                        <span style={{ fontSize: "12px", fontWeight: 700, color: COLORS.muted, textTransform: "uppercase", letterSpacing: "0.5px" }}>Support Units</span>
                        <span style={{ fontSize: "11px", color: COLORS.muted }}>myrtle.moe</span>
                    </div>

                    {/* Support units - full height */}
                    <div style={{ display: "flex", flexDirection: "column", flex: 1 }}>
                        {validSupportUnits.map((unit, index) => (
                            <div
                                key={unit.name}
                                style={{
                                    display: "flex",
                                    flex: 1,
                                    padding: "12px 14px",
                                    borderBottom: index < validSupportUnits.length - 1 ? `1px solid ${COLORS.border}` : "none",
                                    gap: "10px",
                                }}
                            >
                                {/* Left: Avatar with potential overlay */}
                                <div
                                    style={{
                                        position: "relative",
                                        width: "100px",
                                        height: "100px",
                                        borderRadius: "8px",
                                        overflow: "hidden",
                                        backgroundColor: COLORS.secondary,
                                        border: `3px solid ${RARITY_COLORS[unit.rarity] || COLORS.border}`,
                                        flexShrink: 0,
                                        display: "flex",
                                    }}
                                >
                                    {unit.avatarBase64 ? (
                                        // biome-ignore lint/performance/noImgElement: Satori requires native HTML
                                        <img alt={unit.name} src={unit.avatarBase64} style={{ width: "100%", height: "100%", objectFit: "cover" }} />
                                    ) : (
                                        <div style={{ display: "flex", alignItems: "center", justifyContent: "center", width: "100%", height: "100%" }}>
                                            <span style={{ fontSize: "24px", color: COLORS.muted }}>?</span>
                                        </div>
                                    )}
                                    {/* Potential overlay in bottom-left */}
                                    {unit.potentialIconBase64 && (
                                        <div style={{ position: "absolute", bottom: "2px", left: "2px", display: "flex" }}>
                                            {/* biome-ignore lint/performance/noImgElement: Satori requires native HTML */}
                                            <img alt={`P${unit.potential}`} height={28} src={unit.potentialIconBase64} style={{ objectFit: "contain" }} width={28} />
                                        </div>
                                    )}
                                </div>

                                {/* Center: Elite icon + Level circle (stacked) */}
                                <div style={{ display: "flex", flexDirection: "column", alignItems: "center", gap: "4px", flexShrink: 0 }}>
                                    {/* Elite icon - LARGER */}
                                    {unit.eliteIconBase64 ? (
                                        // biome-ignore lint/performance/noImgElement: Satori requires native HTML
                                        <img alt={`E${unit.evolvePhase}`} height={36} src={unit.eliteIconBase64} style={{ objectFit: "contain" }} width={36} />
                                    ) : (
                                        <div style={{ width: "36px", height: "36px", display: "flex" }} />
                                    )}
                                    {/* Level circle - LARGER with less padding */}
                                    <div
                                        style={{
                                            width: "48px",
                                            height: "48px",
                                            borderRadius: "50%",
                                            border: `2px solid ${COLORS.gold}`,
                                            display: "flex",
                                            alignItems: "center",
                                            justifyContent: "center",
                                            backgroundColor: COLORS.background,
                                        }}
                                    >
                                        <span style={{ fontSize: "20px", fontWeight: 700, color: COLORS.gold }}>{unit.level}</span>
                                    </div>
                                </div>

                                {/* Right: Name + 3x2 Skills/Modules grid */}
                                <div style={{ display: "flex", flexDirection: "column", flex: 1, minWidth: 0, gap: "4px" }}>
                                    {/* Name */}
                                    <span style={{ fontSize: "14px", fontWeight: 700, color: COLORS.foreground, overflow: "hidden", textOverflow: "ellipsis", whiteSpace: "nowrap" }}>{unit.name}</span>

                                    {/* 3x2 Grid: Skills (row 1) + Modules (row 2) */}
                                    <div style={{ display: "flex", flexDirection: "column", gap: "3px" }}>
                                        {/* Row 1: Skills */}
                                        <div style={{ display: "flex", gap: "3px" }}>
                                            {[0, 1, 2].map((idx) => {
                                                const skill = unit.skills[idx];
                                                return (
                                                    <div
                                                        key={`skill-${idx}`}
                                                        style={{
                                                            display: "flex",
                                                            flexDirection: "column",
                                                            alignItems: "center",
                                                            gap: "2px",
                                                            backgroundColor: skill ? COLORS.secondary : COLORS.background,
                                                            borderRadius: "4px",
                                                            padding: "4px",
                                                            width: "48px",
                                                            opacity: skill ? 1 : 0.3,
                                                        }}
                                                    >
                                                        {/* Skill icon */}
                                                        {skill?.iconBase64 ? (
                                                            // biome-ignore lint/performance/noImgElement: Satori requires native HTML
                                                            <img alt={`Skill ${idx + 1}`} height={36} src={skill.iconBase64} style={{ objectFit: "contain", borderRadius: "4px" }} width={36} />
                                                        ) : (
                                                            <div style={{ width: "36px", height: "36px", backgroundColor: COLORS.muted, borderRadius: "4px", display: "flex", alignItems: "center", justifyContent: "center" }}>
                                                                <span style={{ fontSize: "12px", color: COLORS.background }}>{idx + 1}</span>
                                                            </div>
                                                        )}
                                                        {/* Mastery icon or level text */}
                                                        {skill?.specializeLevel && skill.specializeLevel > 0 && skill.masteryIconBase64 ? (
                                                            // biome-ignore lint/performance/noImgElement: Satori requires native HTML
                                                            <img alt={`M${skill.specializeLevel}`} height={22} src={skill.masteryIconBase64} style={{ objectFit: "contain" }} width={22} />
                                                        ) : skill ? (
                                                            <span style={{ fontSize: "10px", fontWeight: 600, color: COLORS.muted }}>Lv.{skill.skillLevel}</span>
                                                        ) : (
                                                            <span style={{ fontSize: "10px", color: COLORS.muted }}>---</span>
                                                        )}
                                                    </div>
                                                );
                                            })}
                                        </div>

                                        {/* Row 2: Modules - always show 3 slots */}
                                        <div style={{ display: "flex", gap: "3px" }}>
                                            {[0, 1, 2].map((idx) => {
                                                const module = unit.modules[idx];
                                                const isUnlocked = module && module.level > 0 && !module.locked;
                                                return (
                                                    <div
                                                        key={`module-${idx}`}
                                                        style={{
                                                            display: "flex",
                                                            flexDirection: "column",
                                                            alignItems: "center",
                                                            gap: "2px",
                                                            backgroundColor: isUnlocked ? COLORS.secondary : COLORS.background,
                                                            borderRadius: "4px",
                                                            padding: "4px",
                                                            width: "48px",
                                                            opacity: isUnlocked ? 1 : 0.3,
                                                        }}
                                                    >
                                                        {/* Module icon - only show if unlocked */}
                                                        {isUnlocked && module.iconBase64 ? (
                                                            // biome-ignore lint/performance/noImgElement: Satori requires native HTML
                                                            <img alt={`Module ${idx + 1}`} height={36} src={module.iconBase64} style={{ objectFit: "contain", borderRadius: "4px" }} width={36} />
                                                        ) : (
                                                            <div style={{ width: "36px", height: "36px", backgroundColor: COLORS.muted, borderRadius: "4px", display: "flex", alignItems: "center", justifyContent: "center" }}>
                                                                <span style={{ fontSize: "10px", color: COLORS.background }}>MOD</span>
                                                            </div>
                                                        )}
                                                        {/* Module level or placeholder */}
                                                        {isUnlocked ? <span style={{ fontSize: "10px", fontWeight: 600, color: COLORS.foreground }}>Lv.{module.level}</span> : <span style={{ fontSize: "10px", color: COLORS.muted }}>---</span>}
                                                    </div>
                                                );
                                            })}
                                        </div>
                                    </div>
                                </div>
                            </div>
                        ))}

                        {validSupportUnits.length === 0 && <div style={{ display: "flex", alignItems: "center", justifyContent: "center", flex: 1, color: COLORS.muted, fontSize: "14px" }}>No support units set</div>}
                    </div>
                </div>

                {/* Bottom rarity bar */}
                <div style={{ position: "absolute", bottom: 0, left: 0, right: 0, height: "6px", display: "flex" }}>
                    {totalChars > 0 &&
                        [6, 5, 4, 3, 2, 1].map((rarity) => {
                            const count = rarityCount[rarity] ?? 0;
                            if (count === 0) return null;
                            const widthPercent = (count / totalChars) * 100;
                            return <div key={rarity} style={{ width: `${widthPercent}%`, height: "100%", backgroundColor: RARITY_COLORS[rarity] }} />;
                        })}
                </div>
            </div>,
            {
                width: 1200,
                height: 630,
                fonts: fonts.length > 0 ? fonts : undefined,
                headers: { "Cache-Control": "public, max-age=86400, s-maxage=86400" },
            },
        );
    } catch (error) {
        console.error("OG image generation error:", error);
        return generateFallbackImage();
    }
}
