import { LocaleOptionLabel } from "frontend";
import type { ReactNode } from "react";

// A locale's native name with how much of the site it translates, for the menus
// that have room for both (the header's language menu, Settings > Language).
// The percentage is formatted in the reader's locale; the source locale carries
// no completion and shows none.

/** A menu-width column, as the header's language menu lays its radio items out. */
const Menu = ({ children }: { children: ReactNode }) => (
    <div className="p-4">
        <div className="flex w-56 flex-col gap-0.5 rounded-lg border border-border bg-popover p-1 text-popover-foreground text-sm shadow-lg">{children}</div>
    </div>
);
const Item = ({ children }: { children: ReactNode }) => <div className="rounded-sm px-2 py-1.5">{children}</div>;

/** The language menu's options: the source locale, then translations by completion. */
export const LanguageMenu = () => (
    <Menu>
        <Item><LocaleOptionLabel entry={{ code: "en", nativeName: "English" }} /></Item>
        <Item><LocaleOptionLabel entry={{ code: "ja", nativeName: "日本語", completion: 0.92 }} /></Item>
        <Item><LocaleOptionLabel entry={{ code: "zh-CN", nativeName: "简体中文", completion: 0.87 }} /></Item>
        <Item><LocaleOptionLabel entry={{ code: "ko", nativeName: "한국어", completion: 0.64 }} /></Item>
        <Item><LocaleOptionLabel entry={{ code: "fr", nativeName: "Français", completion: 0.45 }} /></Item>
    </Menu>
);

/** One partly translated locale. */
export const PartialTranslation = () => (
    <Menu>
        <Item><LocaleOptionLabel entry={{ code: "de", nativeName: "Deutsch", completion: 0.31 }} /></Item>
    </Menu>
);

/** The source locale: no percentage. */
export const SourceLocale = () => (
    <Menu>
        <Item><LocaleOptionLabel entry={{ code: "en", nativeName: "English" }} /></Item>
    </Menu>
);
