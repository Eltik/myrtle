/**
 * The reader's font choice, and the custom face a reader uploads.
 *
 * A font overrides the reading-style preset's FAMILY and nothing else: the
 * preset keeps its size multiplier, line height and tracking, so "Large print"
 * with a custom face is still large print. `preset` is the default and the kill
 * switch, and it returns the preset's own family string unchanged.
 *
 * The uploaded bytes go to IndexedDB, not localStorage. A `.ttf` off this
 * machine is 300 KB to 30 MB, which a 5 MB localStorage quota cannot hold, and
 * a quota failure there is silent. IndexedDB keeps one record under one key,
 * so an upload REPLACES the previous face rather than accumulating.
 *
 * The `FontFace` registration does not survive a reload, so the reader
 * re-registers on every mount from the stored bytes. Every call here resolves
 * rather than throws: a blocked or absent IndexedDB (private window, cleared
 * site data, SSR) degrades to "no custom font" and the preset family stands.
 */

export const STORY_FONTS = ["preset", "sans", "dyslexic", "heading", "display", "mono", "cjk", "terraAegir", "terraSami", "terraSarkaz", "system", "custom"] as const;
export type StoryFont = (typeof STORY_FONTS)[number];

/** The family name the uploaded face is registered under. */
export const CUSTOM_FONT_FAMILY = "MyrtleStoryCustomFont";

/**
 * The platform UI stack, which is deliberately NOT one of the site's tokens:
 * "System" means whatever the reader's OS ships, so the page's own webfonts
 * must not appear in it.
 */
export const SYSTEM_FONT_STACK = 'system-ui, -apple-system, "Segoe UI", Roboto, "Helvetica Neue", Arial, sans-serif';

/**
 * Terra's own scripts, shipped in `public/terra-fonts` and declared as
 * `@font-face` in `reader.css`, so they are fetched when the reader mounts and
 * never on a page that has no dialogue on it.
 *
 * They are DISPLAY faces with a partial character set, so each one is followed
 * by the reading style's own family: a glyph the face does not carry falls to
 * the font the reader chose, not to a browser serif.
 */
const TERRA_FAMILY: Partial<Record<StoryFont, string>> = {
    terraAegir: '"Terra Aegir"',
    terraSami: '"Terra Sami"',
    terraSarkaz: '"Terra Sarkaz"',
};

/**
 * The reader's own default face, and the family every other choice falls back
 * to: a Terra display face for the glyphs it does not carry, an uploaded face
 * that failed to register, and the `preset` token itself.
 */
export const PRESET_FONT_FAMILY = "var(--font-sans)";

/**
 * OpenDyslexic, shipped in `public/opendyslexic` and declared as `@font-face`
 * in `reader.css` at 400 and 700, under the SIL Open Font License 1.1. It is
 * what "Dyslexia friendly" writes, and it is also a font a reader can pick on
 * its own without the preset's size and tracking coming with it.
 */
const DYSLEXIC_FAMILY = '"OpenDyslexic"';

/** The site's own families, by token. `preset` and `custom` are resolved separately. */
const TOKEN_FAMILY: Partial<Record<StoryFont, string>> = {
    sans: "var(--font-sans)",
    // The fallback behind it is not decoration: OpenDyslexic carries Latin,
    // and a CJK or Cyrillic line drops to the reader's default face rather
    // than to a browser serif.
    dyslexic: `${DYSLEXIC_FAMILY}, ${PRESET_FONT_FAMILY}`,
    heading: "var(--font-heading)",
    display: "var(--font-display)",
    mono: "var(--font-mono)",
    cjk: "var(--font-cjk)",
    system: SYSTEM_FONT_STACK,
};

export const FONT_FILE_EXTENSIONS = [".ttf", ".otf", ".woff", ".woff2"] as const;
/** The `accept` attribute of the file input. */
export const FONT_FILE_ACCEPT = FONT_FILE_EXTENSIONS.join(",");
/** 32 MB. The largest CJK `.ttf` on a stock macOS is under 30 MB. */
export const MAX_FONT_BYTES = 32 * 1024 * 1024;

/** True when the name ends in one of the four accepted extensions, case-insensitive. */
export function isFontFileName(name: string): boolean {
    const lower = name.toLowerCase();
    return FONT_FILE_EXTENSIONS.some((ext) => lower.endsWith(ext));
}

/**
 * The CSS `font-family` for a choice. `presetFamily` is the reading style's
 * own family, which is both the `preset` answer and the FALLBACK behind a
 * custom face that has not loaded (or failed to).
 */
export function fontFamilyFor(font: StoryFont, presetFamily: string, customLoaded: boolean): string {
    if (font === "custom") return customLoaded ? `"${CUSTOM_FONT_FAMILY}", ${presetFamily}` : presetFamily;
    const terra = TERRA_FAMILY[font];
    if (terra) return `${terra}, ${presetFamily}`;
    return TOKEN_FAMILY[font] ?? presetFamily;
}

const DB_NAME = "myrtle.story";
const DB_VERSION = 1;
const STORE = "fonts";
const RECORD_KEY = "custom";

export interface StoredFont {
    name: string;
    data: ArrayBuffer;
}

function openDb(): Promise<IDBDatabase | null> {
    return new Promise((resolve) => {
        if (typeof indexedDB === "undefined") return resolve(null);
        let request: IDBOpenDBRequest;
        try {
            request = indexedDB.open(DB_NAME, DB_VERSION);
        } catch {
            return resolve(null);
        }
        request.onupgradeneeded = () => {
            const db = request.result;
            if (!db.objectStoreNames.contains(STORE)) db.createObjectStore(STORE);
        };
        request.onsuccess = () => resolve(request.result);
        request.onerror = () => resolve(null);
        request.onblocked = () => resolve(null);
    });
}

function withStore<T>(mode: IDBTransactionMode, run: (store: IDBObjectStore) => IDBRequest<T>): Promise<T | null> {
    return openDb().then(
        (db) =>
            new Promise<T | null>((resolve) => {
                if (!db) return resolve(null);
                try {
                    const tx = db.transaction(STORE, mode);
                    const request = run(tx.objectStore(STORE));
                    request.onsuccess = () => resolve(request.result ?? null);
                    request.onerror = () => resolve(null);
                    tx.oncomplete = () => db.close();
                } catch {
                    resolve(null);
                }
            }),
    );
}

export async function saveCustomFont(name: string, data: ArrayBuffer): Promise<boolean> {
    const record: StoredFont = { name, data };
    const done = await withStore<IDBValidKey>("readwrite", (store) => store.put(record, RECORD_KEY));
    return done !== null;
}

export async function loadCustomFont(): Promise<StoredFont | null> {
    const record = await withStore<StoredFont>("readonly", (store) => store.get(RECORD_KEY));
    if (!record || typeof record.name !== "string" || !(record.data instanceof ArrayBuffer)) return null;
    return record;
}

export async function clearCustomFont(): Promise<void> {
    await withStore<undefined>("readwrite", (store) => store.delete(RECORD_KEY));
    removeRegistration();
}

let registered: FontFace | null = null;

function removeRegistration(): void {
    if (registered && typeof document !== "undefined") {
        try {
            document.fonts.delete(registered);
        } catch {
            // The set is immutable in some embedders; the family simply stays resolvable.
        }
    }
    registered = null;
}

/**
 * Register the stored bytes as `CUSTOM_FONT_FAMILY`. Returns false when the
 * bytes are not a font the browser accepts, which is the one failure the UI
 * reports: everything else degrades to the preset family silently.
 */
export async function registerCustomFont(data: ArrayBuffer): Promise<boolean> {
    if (typeof document === "undefined" || typeof FontFace === "undefined") return false;
    removeRegistration();
    try {
        const face = new FontFace(CUSTOM_FONT_FAMILY, data);
        await face.load();
        document.fonts.add(face);
        registered = face;
        return true;
    } catch {
        return false;
    }
}
