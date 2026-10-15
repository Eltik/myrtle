import { ReadMark } from "frontend";

// ReadMark is the ticket's bookmark tab at legend size: amber for a part-read
// chapter, green for a finished one. It lives inside the browse toolbar's
// read-state pills, which is where the ticket's colour code is spelled out.
// "Any" and "Unread" render nothing (an unread ticket wears no tab).

const PILL = "flex h-7 shrink-0 items-center justify-center gap-1.5 rounded-md px-2.5 font-sans font-semibold text-[11.5px]";

/** The read-state segmented control as the toolbar draws it, "In progress" selected. */
export const InReadFilter = () => (
    <div className="flex w-fit gap-0.5 rounded-[9px] border border-border bg-secondary/45 p-0.75">
        <span className={`${PILL} text-muted-foreground`}>
            <ReadMark state="any" />
            Any
        </span>
        <span className={`${PILL} text-muted-foreground`}>
            <ReadMark state="unread" />
            Unread
        </span>
        <span className={`${PILL} bg-background text-foreground shadow-sm/5`}>
            <ReadMark state="progress" />
            In progress
        </span>
        <span className={`${PILL} text-muted-foreground`}>
            <ReadMark state="done" />
            Finished
        </span>
    </div>
);

/** The two marks on their own: part-read and finished. */
export const States = () => (
    <div className="flex items-center gap-6 font-sans text-[12px] text-muted-foreground">
        <span className="flex items-center gap-2">
            <ReadMark state="progress" />
            In progress
        </span>
        <span className="flex items-center gap-2">
            <ReadMark state="done" />
            Finished
        </span>
    </div>
);
