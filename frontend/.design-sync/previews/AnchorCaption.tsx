import { AnchorCaption, ResolutionBadge } from "frontend";

// "with <event>": names the event a skin or banner arrives alongside.

// Official EN name.
export const OfficialName = () => (
    <div className="p-4 font-sans text-[11px] text-muted-foreground">
        <AnchorCaption anchor={{ cnId: "act1vhalfidle", nameCn: "次生预案", cnStart: 1758787200, cnEnd: 1760558399, offsetSecs: 0, nameEn: "Rebuilding Mandate", nameEnAuto: null }} />
    </div>
);

// Auto-translated name.
export const AutoName = () => (
    <div className="p-4 font-sans text-[11px] text-muted-foreground">
        <AnchorCaption anchor={{ cnId: "act42sre", nameCn: "众生行记·复刻", cnStart: 1779436800, cnEnd: 1780257599, offsetSecs: 0, nameEn: null, nameEnAuto: { text: "The Masses' Travels", source: "memory" } }} />
    </div>
);

// No EN name at all: the CN text, marked zh-CN.
export const ChineseOnly = () => (
    <div className="p-4 font-sans text-[11px] text-muted-foreground">
        <AnchorCaption anchor={{ cnId: "act50side", nameCn: "泡影苍霆", cnStart: 1780268400, cnEnd: 1781467199, offsetSecs: 0, nameEn: null, nameEnAuto: null }} />
    </div>
);

// Where it lives: the caption line under a ResolutionBadge.
export const UnderBadge = () => (
    <div className="flex w-full max-w-md justify-end p-4">
        <ResolutionBadge
            resolution={{ status: "estimated", enStart: 1793086200, lo: 1792945800, hi: 1793450700 }}
            today={new Date("2026-09-01T12:00:00+09:00")}
            caption={<AnchorCaption anchor={{ cnId: "act42sre", nameCn: "众生行记·复刻", cnStart: 1779436800, cnEnd: 1780257599, offsetSecs: 0, nameEn: null, nameEnAuto: { text: "The Masses' Travels", source: "memory" } }} />}
        />
    </div>
);
