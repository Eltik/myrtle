/**
 * THE FONT SECTION, which owns the UPLOADED FACE and is the reason it is its
 * own file.
 *
 * Every other row in the dialog writes one field and is done. This one carries
 * a second store: the bytes live in IndexedDB, a `FontFace` has to be
 * registered with the document before any row can be drawn in it, and neither
 * survives a reload. So the section holds `customLoaded` and re-reads IndexedDB
 * on every open, because the stored NAME is the source of truth for the row's
 * label and the settings copy is only a cache: a font cleared from another tab
 * would otherwise leave the name behind.
 *
 * Every row is drawn IN the family it names, resolved through `fontFamilyFor`,
 * the one function the reader resolves its own dialogue with, so the row and
 * the line it changes can never disagree.
 */
import { TrashIcon, TypeIcon } from "lucide-react";
import type React from "react";
import { useEffect, useId, useRef, useState } from "react";
import { Button } from "#/components/ui/button";
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from "#/components/ui/select";
import { useT } from "#/lib/i18n";
import type { TypedT } from "#/lib/i18n/messages";
import type { StorySettings } from "#/lib/story/settings";
import { clearCustomFont, FONT_FILE_ACCEPT, fontFamilyFor, isFontFileName, loadCustomFont, MAX_FONT_BYTES, PRESET_FONT_FAMILY, registerCustomFont, STORY_FONTS, type StoryFont, saveCustomFont } from "./fonts";
import type { messages as fontMessages } from "./fonts.messages";
import type { messages } from "./reader.messages";
import { Row, TOUCH_BUTTON } from "./SettingsRow";

/**
 * A font's name, drawn in the font. The three Terra scripts are the exception
 * and they have to be: a row that says "Terra Aegir" in Terra Aegir is three
 * alien glyphs and no name at all, so those keep the interface face and carry a
 * SAMPLE of the script beside them instead.
 */
function FontName({ font, label, family }: { font: StoryFont; label: string; family: string }): React.ReactElement {
    if (!font.startsWith("terra")) return <span style={{ fontFamily: family }}>{label}</span>;
    return (
        <span className="inline-flex items-baseline gap-2">
            <span>{label}</span>
            <span aria-hidden className="text-muted-foreground" style={{ fontFamily: family }}>
                Terra
            </span>
        </span>
    );
}

/**
 * `useT` is bound HERE rather than taken as a prop, and that is not a style
 * choice: `scripts/i18n-extract.mjs` binds a `t` to its namespace by the
 * `useT(...)` call in the SAME file, so a component handed its translator
 * through a prop takes every key it uses out of the catalogue's reach. Passing
 * it down measured 54 keys newly unused and 59 call sites lost.
 */
export interface ISettingsFontProps {
    /** The dialog's own open state: the IndexedDB read is re-run on every open. */
    open: boolean;
    settings: StorySettings;
    /** A write that does NOT re-derive the reading style, for the cached file name. */
    onChange: (next: StorySettings) => void;
    /** A write that re-derives the reading style, for the font itself. */
    apply: (next: StorySettings) => void;
}

export function SettingsFont({ open, settings, onChange, apply }: ISettingsFontProps): React.ReactElement {
    const t: TypedT<typeof messages> = useT("story");
    const tf: TypedT<typeof fontMessages> = useT("story");
    const fontFileId = useId();
    const fontRef = useRef<HTMLInputElement>(null);
    const [fontNotice, setFontNotice] = useState<string | null>(null);
    // Whether the stored face is registered RIGHT NOW, which is what lets the
    // dropdown print the custom row in the reader's own font.
    const [customLoaded, setCustomLoaded] = useState(false);

    // biome-ignore lint/correctness/useExhaustiveDependencies: `settings` is what this effect WRITES; listing it would re-run the read on its own write.
    useEffect(() => {
        if (!open) return;
        void loadCustomFont().then(async (stored) => {
            const name = stored?.name ?? "";
            if (name !== settings.customFontName) onChange({ ...settings, customFontName: name });
            // Registered even when it is not the CHOSEN font: the dropdown
            // renders every row in the face it names, and an unregistered
            // family would silently draw that one row in the fallback.
            setCustomLoaded(stored ? await registerCustomFont(stored.data) : false);
        });
    }, [open]);

    const familyOf = (font: StoryFont): string => fontFamilyFor(font, PRESET_FONT_FAMILY, customLoaded);
    /** The custom row prints its file name once there is one, which is the only label a reader recognises. */
    const fontLabel = (font: StoryFont): string => (font === "custom" && settings.customFontName ? settings.customFontName : tf(`settings.font.${font}`));

    const uploadFont = async (file: File | undefined) => {
        if (!file) return;
        setFontNotice(null);
        if (!isFontFileName(file.name)) return setFontNotice(t("settings.font.rejected"));
        if (file.size > MAX_FONT_BYTES) return setFontNotice(t("settings.font.tooBig", { mb: Math.round(MAX_FONT_BYTES / (1024 * 1024)) }));
        const data = await file.arrayBuffer();
        // Register BEFORE storing: bytes the browser will not parse are not a
        // font, and storing them would leave the reader loading a dead record
        // on every mount.
        if (!(await registerCustomFont(data))) return setFontNotice(t("settings.font.rejected"));
        setCustomLoaded(true);
        await saveCustomFont(file.name, data);
        apply({ ...settings, font: "custom", customFontName: file.name });
    };

    const removeFont = async () => {
        await clearCustomFont();
        setCustomLoaded(false);
        setFontNotice(null);
        apply({ ...settings, font: settings.font === "custom" ? "preset" : settings.font, customFontName: "" });
    };

    return (
        <>
            <Row label={t("settings.font")} hint={t("settings.fontHint")}>
                <div className="flex w-full flex-col gap-2">
                    <Select value={settings.font} onValueChange={(v: string | null) => v && apply({ ...settings, font: v as StoryFont })}>
                        <SelectTrigger className="max-sm:h-11">
                            <SelectValue>{() => <FontName font={settings.font} label={fontLabel(settings.font)} family={familyOf(settings.font)} />}</SelectValue>
                        </SelectTrigger>
                        <SelectContent>
                            {STORY_FONTS.map((f) => (
                                <SelectItem key={f} value={f} data-story-font-option={f}>
                                    <FontName font={f} label={fontLabel(f)} family={familyOf(f)} />
                                </SelectItem>
                            ))}
                        </SelectContent>
                    </Select>
                    {/* One line on the choice itself, because "Site serif" says
                        what the token is called and not what it is for. */}
                    <p className="text-muted-foreground text-xs">{tf(`settings.fontNote.${settings.font}`)}</p>
                    {/* The Terra faces are fan work under two different
                        licences; the attribution itself is one line in the site footer. */}
                    {settings.font.startsWith("terra") ? <p className="text-muted-foreground text-xs">{t("settings.font.terraCredit")}</p> : null}
                    {/* The preview is the sample AT THE READING SIZE: a face
                        that looks fine at 12 px is the wrong thing to judge a
                        130% reading style by. */}
                    <p className="rounded-md border bg-muted/40 px-3 py-2 leading-snug" data-story-font-preview style={{ fontFamily: familyOf(settings.font), fontSize: `${settings.textSize / 100}rem` }}>
                        {t("settings.font.previewText")}
                    </p>
                </div>
            </Row>
            {settings.font === "custom" ? (
                <div className="-mt-2 flex flex-wrap items-center gap-2 sm:ps-[calc(10rem+0.75rem)]">
                    <Button variant="outline" size="sm" className={TOUCH_BUTTON} onClick={() => fontRef.current?.click()}>
                        <TypeIcon /> {t("settings.font.upload")}
                    </Button>
                    <input id={fontFileId} ref={fontRef} type="file" accept={FONT_FILE_ACCEPT} className="sr-only" data-story-font-input onChange={(e) => void uploadFont(e.target.files?.[0])} />
                    <span className="min-w-0 truncate text-muted-foreground text-xs">{settings.customFontName || t("settings.font.none")}</span>
                    {settings.customFontName ? (
                        <Button variant="ghost" size="sm" className={TOUCH_BUTTON} onClick={() => void removeFont()}>
                            <TrashIcon /> {t("settings.font.remove")}
                        </Button>
                    ) : null}
                    <output className="basis-full text-destructive text-xs" aria-live="polite">
                        {fontNotice}
                    </output>
                </div>
            ) : null}
        </>
    );
}
