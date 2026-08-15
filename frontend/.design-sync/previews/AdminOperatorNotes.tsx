import { AdminOperatorNotes } from "frontend";

// The screen joins the operator index against the notes table; both are stubbed
// server functions in a preview, so it settles on the empty-collection branch —
// filter chips with zero counts, sort menu, and the "No operator notes yet."
// placeholder.
export function NoNotesLoaded() {
    return <AdminOperatorNotes />;
}
