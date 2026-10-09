import { useRouterState } from "@tanstack/react-router";
import { useT } from "#/lib/i18n";
import type { TypedT } from "#/lib/i18n/messages";
import { type IAdminAccess, useAdminAccess } from "./access";
import { useInboxQueueCount, useNotesMissingCount, usePeopleWarnedCount, useTranslationsTodoCount } from "./counts";
import { type AdminSectionId, type AdminSectionPath, SECTION_PATHS, sectionForPath } from "./model";
import type { messages } from "./nav.messages";

export type AdminCountVariant = "default" | "warning" | "secondary";

export interface IAdminNavSection {
    id: AdminSectionId;
    to: AdminSectionPath;
    label: string;
    /** `undefined` or 0 hides the badge. */
    count: number | undefined;
    countVariant: AdminCountVariant;
}

export interface IAdminNav {
    /** The sections this user may open, in bar order. */
    sections: IAdminNavSection[];
    /** The section the current URL is in (regardless of whether it is listed yet). */
    current: AdminSectionId;
    /** Label for `current`, even while its tab is hidden behind a loading grant. */
    currentLabel: string;
    access: IAdminAccess;
}

export function useRoleLabel(): (role: string | null | undefined) => string {
    const t: TypedT<typeof messages> = useT("admin");
    return (role) => {
        switch (role) {
            case "super_admin":
                return t("shell.role.superAdmin");
            case "tier_list_admin":
                return t("shell.role.tierListAdmin");
            case "tier_list_editor":
                return t("shell.role.tierListEditor");
            case "translator":
                return t("shell.role.translator");
            default:
                return t("shell.role.user");
        }
    };
}

/**
 * The admin bar's model: the visible sections for the signed-in user with
 * their labels and badge counts, plus the role model they were derived from.
 */
export function useAdminNav(): IAdminNav {
    const t: TypedT<typeof messages> = useT("admin");
    const access = useAdminAccess();
    const pathname = useRouterState({ select: (s) => s.location.pathname });
    const current = sectionForPath(pathname);

    const inboxCount = useInboxQueueCount(access);
    const warnedCount = usePeopleWarnedCount(access);
    const notesCount = useNotesMissingCount(access);
    const translationsCount = useTranslationsTodoCount(access);

    const { staff, canAssign, can } = access;
    const all: IAdminNavSection[] = [
        { id: "home", to: SECTION_PATHS.home, label: staff ? t("shell.nav.inbox") : t("shell.nav.myWork"), count: staff ? inboxCount : undefined, countVariant: "default" },
        { id: "users", to: SECTION_PATHS.users, label: t("shell.nav.people"), count: canAssign ? warnedCount : undefined, countVariant: "warning" },
        { id: "tierlists", to: SECTION_PATHS.tierlists, label: staff ? t("shell.nav.tierLists") : t("shell.nav.myTierLists"), count: undefined, countVariant: "secondary" },
        { id: "notes", to: SECTION_PATHS.notes, label: t("shell.nav.notes"), count: notesCount, countVariant: "secondary" },
        { id: "translations", to: SECTION_PATHS.translations, label: t("shell.nav.translations"), count: translationsCount, countVariant: "secondary" },
        { id: "system", to: SECTION_PATHS.system, label: t("shell.nav.system"), count: undefined, countVariant: "secondary" },
    ];

    return {
        sections: all.filter((s) => can[s.id]),
        current,
        currentLabel: all.find((s) => s.id === current)?.label ?? all[0].label,
        access,
    };
}
