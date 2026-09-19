import { useLocaleSwitch } from "#/components/LocaleSwitcher";
import type { messages as localeSwitcherMessages } from "#/components/LocaleSwitcher.messages";
import { Button } from "#/components/ui/button";
import { DropdownMenu, DropdownMenuContent, DropdownMenuGroup, DropdownMenuLabel, DropdownMenuRadioGroup, DropdownMenuRadioItem, DropdownMenuTrigger } from "#/components/ui/menu";
import { useI18n, useT } from "#/lib/i18n";
import type { TypedT } from "#/lib/i18n/messages";
import type { messages } from "./LanguageToggle.messages";

/**
 * The language control in the header.
 *
 * The active language's code in the top bar, not a row inside the appearance
 * popover: language is the one setting people actively hunt for, and burying it
 * behind a theme icon tests whether the visitor can guess our information
 * architecture. It was a globe, which announces that language lives here but
 * not which language you are reading; the code does both for the same width.
 *
 * Visible at every width. The bar is busy on a phone, but a control nobody can
 * find is worse than a tight row, and the hamburger drawer carries the same
 * choice as a second path.
 *
 * The room for it comes from the wordmark, which `Header` hides below `sm` -
 * see the comment there. If you add another button to this cluster, take the
 * space from something else rather than from this one.
 *
 * Renders nothing below two enabled locales, so a single-language deployment
 * pays no pixels for it.
 */
export default function LanguageToggle(): React.ReactElement | null {
    const t: TypedT<typeof messages> = useT("nav");
    const tCommon: TypedT<typeof localeSwitcherMessages> = useT("common");
    const { locale, available } = useI18n();
    const switchLocale = useLocaleSwitch();

    if (available.length < 2) return null;

    const active = available.find((entry) => entry.code === locale);
    const triggerLabel = t("languageToggle.trigger", { language: active?.nativeName ?? locale });
    // `zh-CN` reads as `ZH` here; the full tag is in the label and the tooltip.
    const shortCode = locale.split("-")[0];

    return (
        <DropdownMenu>
            <DropdownMenuTrigger
                render={
                    <Button variant="ghost" size="icon" aria-label={t("languageToggle.triggerAria", { label: triggerLabel })} title={triggerLabel}>
                        {/* The code, not a globe: the globe says "language lives here",
                            which you only need once, while the code also says WHICH
                            language you are reading, which was the actual request. It
                            costs about the same width as the icon did. Not a flag: a
                            language is not a country, and Русский, 日本語 and Tagalog
                            have no single correct one. */}
                        <span className="font-mono font-semibold text-[11px] uppercase leading-none tracking-[0.04em]">{shortCode}</span>
                    </Button>
                }
            />
            <DropdownMenuContent align="end" className="w-44">
                {/* `DropdownMenuLabel` is a GROUP label in this primitive set,
                    so it must sit inside a group or base-ui throws for a
                    missing MenuGroupRootContext. */}
                <DropdownMenuGroup>
                    <DropdownMenuLabel>{tCommon("localeSwitcher.language")}</DropdownMenuLabel>
                </DropdownMenuGroup>
                <DropdownMenuRadioGroup
                    value={locale}
                    onValueChange={(next) => {
                        if (next && next !== locale) switchLocale(next);
                    }}
                >
                    {available.map((entry) => (
                        <DropdownMenuRadioItem key={entry.code} value={entry.code} lang={entry.code} className="cursor-pointer">
                            {entry.nativeName}
                        </DropdownMenuRadioItem>
                    ))}
                </DropdownMenuRadioGroup>
            </DropdownMenuContent>
        </DropdownMenu>
    );
}
