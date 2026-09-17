/**
 * Which name an operator is shown under.
 *
 * Every operator record carries two names from `character_table`: `name`, in
 * the server's own language, and `appellation`, the Latin-script name the
 * game itself prints under the portrait. On the EN server the two are the
 * same string for all but four operators (the Ursus crew, whose appellation
 * is Cyrillic). On CN, `name` is Han and `appellation` is the name the
 * operator will most likely carry when EN releases it: across the 1140
 * operators both servers have, the CN appellation equals the EN name for
 * 1115 (measured 2026-09-17 against the two trees under `assets/output`).
 *
 * The site reads the EN tree, so a Han `name` reaches the page only for the
 * CN-only ("upcoming") operators. When the Latin-names preference is on,
 * those render under the appellation; everything else is untouched.
 */

export interface INamedOperator {
    name: string;
    appellation?: string | null;
}

// Han, kana and hangul: the scripts a `name` is written in when it comes from
// the CN, JP or KR tree. Cyrillic is deliberately absent: an Ursus
// appellation such as `Зима` is still the game's own Latin-column value and
// there is nothing better to swap it for.
const CJK_RE = /[\p{Script=Han}\p{Script=Hiragana}\p{Script=Katakana}\p{Script=Hangul}]/u;

export function hasCjk(text: string): boolean {
    return CJK_RE.test(text);
}

/**
 * The name to render. Returns `appellation` only when the preference is on,
 * `name` contains a CJK script, and the appellation is non-blank; otherwise
 * `name` exactly as stored, so the preference is inert for every EN operator.
 */
export function operatorDisplayName(op: INamedOperator, latinNames: boolean): string {
    if (!latinNames) return op.name;
    const appellation = op.appellation?.trim() ?? "";
    if (appellation.length === 0 || !hasCjk(op.name)) return op.name;
    return appellation;
}
