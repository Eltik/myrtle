/**
 * Release-note titles lead with their kind ("New: ...", "Improved: ..."). The
 * panel shows that kind as a tag and the rest as the title. A title without a
 * short prefix is shown whole, untagged.
 */
export function splitNoteTitle(title: string): { kind: string | null; title: string } {
    const match = /^\s*([^:：]{1,20})[:：]\s*(\S[\s\S]*)$/.exec(title);
    return match ? { kind: match[1].trim(), title: match[2] } : { kind: null, title };
}
