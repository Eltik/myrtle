import { useT } from "#/lib/i18n";
import type { TypedT } from "#/lib/i18n/messages";
import type { messages } from "./Home.messages";
import { truncateNames } from "./queue";

export type HomeT = TypedT<typeof messages>;

/** "Includes 3 six-stars: A, B and C." for the staff queue row. */
export function sixStarsSub(t: HomeT, names: readonly string[]): string {
    if (names.length === 0) return t("home.notes.sub.none");
    const { shown, rest } = truncateNames(names);
    return rest > 0 ? t("home.notes.sub.sixStarsMore", { count: names.length, names: shown.join(", "), rest }) : t("home.notes.sub.sixStars", { count: names.length, names: shown.join(", ") });
}

/** "Includes A, B and C." for the editor's notes card. */
export function namesSub(t: HomeT, names: readonly string[]): string {
    if (names.length === 0) return t("home.notes.sub.none");
    const { shown, rest } = truncateNames(names);
    return rest > 0 ? t("home.notes.sub.namesMore", { names: shown.join(", "), rest }) : t("home.notes.sub.names", { names: shown.join(", ") });
}

export function useNoteFieldLabel(): (field: string) => string {
    const t: HomeT = useT("admin");
    return (field) => {
        switch (field) {
            case "summary":
                return t("home.field.summary");
            case "pros":
                return t("home.field.pros");
            case "cons":
                return t("home.field.cons");
            case "notes":
                return t("home.field.notes");
            case "trivia":
                return t("home.field.trivia");
            case "tags":
                return t("home.field.tags");
            default:
                return field.charAt(0).toUpperCase() + field.slice(1);
        }
    };
}
