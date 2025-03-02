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

/**
 * Preprocesses the description to balance any unbalanced tags
 * @param description The description to preprocess
 * @returns The preprocessed description with balanced tags
 */
const preprocessDescription = (description: string): string => {
    // Count opening and closing tags
    const openingTagsMatch = description.match(new RegExp(descriptionTagLeftDelim, "g")) ?? [];
    const closingTagsMatch = description.match(new RegExp(descriptionTagRightDelim, "g")) ?? [];
    
    let processedDescription = description;
    
    // If there are more opening tags than closing tags, add closing tags at the end
    if (openingTagsMatch.length > closingTagsMatch.length) {
        const missingClosingTags = openingTagsMatch.length - closingTagsMatch.length;
        for (let i = 0; i < missingClosingTags; i++) {
            processedDescription += "</>";
        }
    }
    
    // Handle the case where there might be a closing tag without an opening tag
    // by simply removing those closing tags
    if (closingTagsMatch.length > openingTagsMatch.length) {
        // This is a more complex case. For simplicity, we'll escape all tags
        // to prevent parsing errors
        processedDescription = description
            .replace(/<(?:@ba.|\\$)[^>]+>/g, (match) => `&lt;${match.slice(1, -1)}&gt;`)
            .replace(/<\/>/g, "&lt;/&gt;");
    }
    
    return processedDescription;
};

export const descriptionToHtml = (description: string, interpolation: InterpolatedValue[]): string => {
    // Preprocess the description to balance tags
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
    } catch (error) {
        // Handle unbalanced delimiters by escaping all tags
        console.warn("Error parsing description tags:", error);
        // Escape all tags to prevent HTML injection and return the original text
        return description
            .replace(/</g, "&lt;")
            .replace(/>/g, "&gt;")
            .replace(/\n/g, "<br>");
    }

    // replace any newlines with <br> tags to get past HTML whitespace collapsing
    htmlDescription = htmlDescription
        .replace(/\n/g, "<br>")
        .replace(/<\/br>/g, "<br>")
        .replace(/<(?!\/?span)(?!br)([^>]+)>/g, "&lt;$1&gt;");

    try {
        do {
            match = descriptionInterpolationRegex.exec(htmlDescription);
            if (match?.groups) {
                const key = match.groups.interpolationKey?.toLowerCase();
                const value = interpolation.find((value) => value.key?.toLowerCase() === key)?.value;
                if (!value) {
                    console.warn(`Couldn't find matching interpolation key: ${key}`);
                    // Replace with placeholder instead of throwing error
                    htmlDescription = htmlDescription.replace(descriptionInterpolationRegex, `[${key}]`);
                    continue;
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
                    interpolated = `${value}`;
                }
                htmlDescription = htmlDescription.replace(descriptionInterpolationRegex, interpolated);
            }
        } while (match);
    } catch (error) {
        console.warn("Error processing interpolation:", error);
        // If interpolation fails, return the HTML description without interpolation
    }

    return htmlDescription;
};
