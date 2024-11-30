/**
 * @author All credit to https://github.com/Awedtan/HellaBot/
 * @param text
 * @returns string
 */

export function removeStyleTags(text: string): string {
    return text.replace(/<.[a-z]{2,5}?\.[^<]+>|<\/[^<]*>|<color=[^>]+>/g, "");
}
