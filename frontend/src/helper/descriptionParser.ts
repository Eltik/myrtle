import XRegExp, { type MatchRecursiveValueNameMatch } from "xregexp";

/**
 * @author All credit to https://github.com/iansjk/sanity-gone/blob/main/src/utils/description-parser.ts
 */

import type { InterpolatedValue } from "~/types/impl/frontend/impl/operators";

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

export const descriptionToHtml = (description: string, interpolation: InterpolatedValue[]): string => {
    let htmlDescription = description.slice();
    let recursiveMatch: MatchRecursiveValueNameMatch[] | null = null;
    let match: RegExpMatchArray | null = null;
    do {
        recursiveMatch = XRegExp.matchRecursive(htmlDescription, descriptionTagLeftDelim, descriptionTagRightDelim, "g", {
            valueNames: ["between", "tagName", "tagContent", "closingTag"],
        });

        if ((recursiveMatch ?? []).length > 0) {
            let resultingString = "";
            for (const match of recursiveMatch) {
                if (match.name === "between") {
                    resultingString += match.value;
                } else if (match.name === "tagName") {
                    const tagName = match.value.slice(1, -1);
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
                                color = "skill-tooltip";
                                break;
                            }
                            console.warn(`Unrecognized tag: ${tagName}`);
                            break;
                    }
                    resultingString += `<span style="${color}">`;
                } else if (match.name === "tagContent") {
                    resultingString += match.value;
                } else if (match.name === "closingTag") {
                    resultingString += "</span>";
                }
            }

            htmlDescription = resultingString;
        }
    } while (recursiveMatch.length > 0);

    // replace any newlines with <br> tags to get past HTML whitespace collapsing
    htmlDescription = htmlDescription
        .replace(/\n/g, "<br>")
        .replace(/<\/br>/g, "<br>")
        .replace(/<(?!\/?span)(?!br)([^>]+)>/g, "&lt;$1&gt;");

    do {
        match = descriptionInterpolationRegex.exec(htmlDescription);
        if (match?.groups) {
            const key = match.groups.interpolationKey?.toLowerCase();
            const value = interpolation.find((value) => value.key?.toLowerCase() === key)?.value;
            if (!value) {
                throw new Error(`Couldn't find matching interpolation key: ${key}`);
            }

            let interpolated = "";
            const { formatString } = match.groups;
            if (typeof formatString === "undefined") {
                interpolated = `${value}`;
            } else if (formatString === "0%") {
                // convert to percentage and suffix with "%"
                interpolated = `${Math.round(value * 100)}%`;
            } else if (formatString === "0.0") {
                // return as-is to one-decimal place
                interpolated = `${value.toFixed(1)}`;
            } else {
                console.warn(`Unrecognized format string: ${match.groups.formatString}`);
            }
            htmlDescription = htmlDescription.replace(descriptionInterpolationRegex, interpolated);
        }
    } while (match);

    return htmlDescription;
};
