import type { IBlackboard } from "#/types/operators";

const DESCRIPTION_COLORS = {
    valueUp: "#d597da",
    valueDown: "#ff847d",
    reminder: "#da9a46",
    keyword: "#49b3ff",
    potential: "#49b3ff",
    skillTooltip: "#49b3ff",
};

function colorForTag(tag: string): string {
    if (tag === "@ba.vup") return DESCRIPTION_COLORS.valueUp;
    if (tag === "@ba.vdown") return DESCRIPTION_COLORS.valueDown;
    if (tag === "@ba.rem") return DESCRIPTION_COLORS.reminder;
    if (tag === "@ba.kw") return DESCRIPTION_COLORS.keyword;
    if (tag === "@ba.talpu") return DESCRIPTION_COLORS.potential;
    if (tag.startsWith("$")) return DESCRIPTION_COLORS.skillTooltip;
    return DESCRIPTION_COLORS.keyword;
}

const tagRegex = /<((?:@ba\.[a-z]+|\$[^>]+))>([\s\S]*?)<\/>/g;
const interpolationRegex = /-?\{-?([^}:]+)(?::([^}]+))?\}/g;

export function descriptionToHtml(description: string, blackboard: IBlackboard[] | { key: string; value: number }[] = []): string {
    if (!description) return "";

    const interp = new Map<string, number>();
    for (const b of blackboard) {
        if (b?.key) interp.set(b.key.toLowerCase(), b.value);
    }

    let html = description.replace(/&/g, "&amp;");

    html = html.replace(tagRegex, (_match, tag: string, content: string) => {
        return `<span style="color:${colorForTag(tag)}">${content}</span>`;
    });

    html = html.replace(interpolationRegex, (_match, rawKey: string, format?: string) => {
        const key = rawKey.toLowerCase();
        const value = interp.get(key);
        if (value === undefined) return `[${key}]`;
        if (!format) return `${value}`;
        if (format === "0%") return `${Math.round(value * 100)}%`;
        if (format === "0.0") return value.toFixed(1);
        return `${value}`;
    });

    html = html.replace(/\n/g, "<br/>");
    return html;
}
