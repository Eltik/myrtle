import XRegExp, { type MatchRecursiveValueNameMatch } from "xregexp";

/**
 * Description parser for Arknights operator skill/talent descriptions
 * Credit: https://github.com/iansjk/sanity-gone/blob/main/src/utils/description-parser.ts
 */

interface InterpolatedValue {
    key: string;
    value: number;
    valueStr?: string | null;
}

const descriptionTagLeftDelim = "<(?:@ba.|\\$)[^>]+>";
const descriptionTagRightDelim = "</>";

const descriptionInterpolationRegex = /-?{-?(?<interpolationKey>[^}:]+)(?::(?<formatString>[^}]+))?}/;

export const DESCRIPTION_COLORS = {
    valueUp: "#49b3ff",
    valueDown: "#ff5e5e",
    reminder: "#ffcf53",
    potential: "#49b3ff",
    keyword: "#49b3ff",
};

/**
 * Preprocesses the description to balance any unbalanced tags
 */
const preprocessDescription = (description: string): string => {
    const openingTagsMatch = description.match(new RegExp(descriptionTagLeftDelim, "g")) ?? [];
    const closingTagsMatch = description.match(new RegExp(descriptionTagRightDelim, "g")) ?? [];

    let processedDescription = description;

    if (openingTagsMatch.length > closingTagsMatch.length) {
        const missingClosingTags = openingTagsMatch.length - closingTagsMatch.length;
        for (let i = 0; i < missingClosingTags; i++) {
            processedDescription += "</>";
        }
    }

    if (closingTagsMatch.length > openingTagsMatch.length) {
        processedDescription = description.replace(/<(?:@ba.|\\$)[^>]+>/g, (match) => `&lt;${match.slice(1, -1)}&gt;`).replace(/<\/>/g, "&lt;/&gt;");
    }

    return processedDescription;
};

export const descriptionToHtml = (description: string, interpolation: InterpolatedValue[]): string => {
    let htmlDescription = preprocessDescription(description.slice());
    let recursiveMatch: MatchRecursiveValueNameMatch[] | null = null;
    let match: RegExpMatchArray | null = null;

    try {
        do {
            recursiveMatch = XRegExp.matchRecursive(htmlDescription, descriptionTagLeftDelim, descriptionTagRightDelim, "g", {
                valueNames: ["between", "tagName", "tagContent", "closingTag"],
            });

            if ((recursiveMatch ?? []).length > 0) {
                let resultingString = "";
                for (const currentMatch of recursiveMatch) {
                    if (currentMatch.name === "between") {
                        resultingString += currentMatch.value;
                    } else if (currentMatch.name === "tagName") {
                        const tagName = currentMatch.value.slice(1, -1);
                        let color = "";
                        switch (tagName) {
                            case "@ba.vup":
                                color = `color: ${DESCRIPTION_COLORS.valueUp};`;
                                break;
                            case "@ba.vdown":
                                color = `color: ${DESCRIPTION_COLORS.valueDown};`;
                                break;
                            case "@ba.rem":
                                color = `color: ${DESCRIPTION_COLORS.reminder};`;
                                break;
                            case "@ba.kw":
                                color = `color: ${DESCRIPTION_COLORS.keyword};`;
                                break;
                            case "@ba.talpu":
                                color = `color: ${DESCRIPTION_COLORS.potential};`;
                                break;
                            default:
                                if (tagName?.startsWith("$")) {
                                    color = `color: ${DESCRIPTION_COLORS.keyword};`;
                                    break;
                                }
                                break;
                        }
                        resultingString += `<span style="${color}">`;
                    } else if (currentMatch.name === "tagContent") {
                        resultingString += currentMatch.value;
                    } else if (currentMatch.name === "closingTag") {
                        resultingString += "</span>";
                    }
                }

                htmlDescription = resultingString;
            }
        } while (recursiveMatch.length > 0);
    } catch (error) {
        console.warn("Error parsing description tags:", error);
        return description.replace(/</g, "&lt;").replace(/>/g, "&gt;").replace(/\n/g, "<br>");
    }

    htmlDescription = htmlDescription
        .replace(/\n/g, "<br>")
        .replace(/<\/br>/g, "<br>")
        .replace(/<(?!\/?span)(?!br)([^>]+)>/g, "&lt;$1&gt;");

    try {
        do {
            match = descriptionInterpolationRegex.exec(htmlDescription);
            if (match?.groups) {
                const key = match.groups.interpolationKey?.toLowerCase();
                const value = interpolation.find((v) => v.key?.toLowerCase() === key)?.value;
                if (value === undefined) {
                    htmlDescription = htmlDescription.replace(descriptionInterpolationRegex, `[${key}]`);
                    continue;
                }

                let interpolated = "";
                const { formatString } = match.groups;
                if (typeof formatString === "undefined") {
                    interpolated = `${value}`;
                } else if (formatString === "0%") {
                    interpolated = `${Math.round(value * 100)}%`;
                } else if (formatString === "0.0") {
                    interpolated = `${value.toFixed(1)}`;
                } else {
                    interpolated = `${value}`;
                }
                htmlDescription = htmlDescription.replace(descriptionInterpolationRegex, interpolated);
            }
        } while (match);
    } catch (error) {
        console.warn("Error processing interpolation:", error);
    }

    return htmlDescription;
};
