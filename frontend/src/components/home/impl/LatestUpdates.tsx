import { Link } from "@tanstack/react-router";
import { formatNoteDate } from "#/components/changelog/release-note-shared";
import { RELEASE_NOTES } from "#/content/changelog/entries";
import type { messages as entryMessages } from "#/content/changelog/entries.messages";
import { useLocale, useT } from "#/lib/i18n";
import type { TypedT } from "#/lib/i18n/messages";
import { stripMarkdown } from "#/lib/markdown";
import styles from "./Digest.module.css";
import type { messages } from "./LatestUpdates.messages";
import { splitNoteTitle } from "./notes";
import shared from "./shared.module.css";

/** How many release notes the home page lists. */
const NOTE_LIMIT = 4;

export default function LatestUpdates() {
    const t: TypedT<typeof messages> = useT("home");
    const tn: TypedT<typeof entryMessages> = useT("changelog");
    const locale = useLocale();

    return (
        <div className={styles.panel}>
            <div className={styles.head}>
                <div className={styles.headText}>
                    <h2 className={styles.title}>{t("updates.title")}</h2>
                </div>
                <Link to="/changelog" className={shared.moreLink}>
                    {t("updates.seeAll")}
                </Link>
            </div>
            <div className={styles.list}>
                {RELEASE_NOTES.slice(0, NOTE_LIMIT).map((note) => {
                    const { kind, title } = splitNoteTitle(tn(note.titleKey));
                    return (
                        <Link key={note.id} to={note.href ?? "/changelog"} className={styles.update}>
                            <span className={styles.updateMeta}>
                                {kind && <span className="kicker">{kind}</span>}
                                <span className={styles.updateDate}>{formatNoteDate(note.date, locale, { month: "short", day: "numeric" })}</span>
                            </span>
                            <span className={styles.updateTitle}>{title}</span>
                            <span className={styles.updateLead}>{stripMarkdown(tn(note.leadKey))}</span>
                        </Link>
                    );
                })}
            </div>
        </div>
    );
}
