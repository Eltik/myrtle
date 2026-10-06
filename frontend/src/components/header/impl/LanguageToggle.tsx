import { useGamedataServerChoice, useGamedataServerOptions, useGamedataServerSwitch } from "#/components/GamedataServerSwitcher";
import type { messages as gamedataServerMessages } from "#/components/GamedataServerSwitcher.messages";
import { LocaleOptionLabel, useLocaleSwitch } from "#/components/LocaleSwitcher";
import type { messages as localeSwitcherMessages } from "#/components/LocaleSwitcher.messages";
import { Button } from "#/components/ui/button";
import { DropdownMenu, DropdownMenuContent, DropdownMenuGroup, DropdownMenuLabel, DropdownMenuRadioGroup, DropdownMenuRadioItem, DropdownMenuSeparator, DropdownMenuTrigger } from "#/components/ui/menu";
import { useI18n, useT } from "#/lib/i18n";
import type { TypedT } from "#/lib/i18n/messages";
import type { messages } from "./LanguageToggle.messages";

/**
 * The language control in the header.
 *
 * The active language's code in the top bar, not a row inside the appearance
 * popover: language is the one setting people actively hunt for, and burying it
 * behind a theme icon tests whether the visitor can guess our information
 * architecture.
 *
 * Visible at every width. The bar is busy on a phone, but a control nobody can
 * find is worse than a tight row, and the hamburger drawer carries the same
 * choice as a second path.
 *
 * The room for it comes from the wordmark, which `Header` hides below `sm` -
 * see the comment there. If you add another button to this cluster, take the
 * space from something else rather than from this one.
 *
 * The game-text picker shares this menu because it defaults to the language:
 * the two read as one decision ("what am I reading") with an override.
 *
 * Renders nothing when there is neither a second locale nor a second loaded
 * game-data server, so a single-language, single-server deployment pays no
 * pixels for it.
 */
export default function LanguageToggle(): React.ReactElement | null {
    const t: TypedT<typeof messages> = useT("nav");
    const tCommon: TypedT<typeof localeSwitcherMessages & typeof gamedataServerMessages> = useT("common");
    const { locale, available } = useI18n();
    const switchLocale = useLocaleSwitch();
    const serverOptions = useGamedataServerOptions();
    const serverChoice = useGamedataServerChoice();
    const switchServer = useGamedataServerSwitch();

    const hasLocales = available.length >= 2;
    if (!hasLocales && serverOptions.length === 0) return null;

    const active = available.find((entry) => entry.code === locale);
    const triggerLabel = t("languageToggle.trigger", { language: active?.nativeName ?? locale });
    // `zh-CN` reads as `ZH` here; the full tag is in the label and the tooltip.
    const shortCode = locale.split("-")[0];

    return (
        <DropdownMenu>
            <DropdownMenuTrigger
                render={
                    <Button variant="ghost" size="icon" aria-label={t("languageToggle.triggerAria", { label: triggerLabel })} title={triggerLabel}>
                        {/* The code, not a globe: a globe only says "language lives
                            here", which you need once, while the code also says WHICH
                            language you are reading, for about the same width. Not a
                            flag: a language is not a country, and Русский, 日本語 and
                            Tagalog have no single correct one. */}
                        <span className="font-mono font-semibold text-[11px] uppercase leading-none tracking-[0.04em]">{shortCode}</span>
                    </Button>
                }
            />
            <DropdownMenuContent align="end" className="w-52">
                {/* `DropdownMenuLabel` is a GROUP label in this primitive set,
                    so it must sit inside a group or base-ui throws for a
                    missing MenuGroupRootContext. */}
                {hasLocales ? (
                    <>
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
                                    <LocaleOptionLabel entry={entry} />
                                </DropdownMenuRadioItem>
                            ))}
                        </DropdownMenuRadioGroup>
                    </>
                ) : null}
                {serverOptions.length > 0 ? (
                    <>
                        {hasLocales ? <DropdownMenuSeparator /> : null}
                        <DropdownMenuGroup>
                            <DropdownMenuLabel>{tCommon("gamedataServer.label")}</DropdownMenuLabel>
                        </DropdownMenuGroup>
                        <DropdownMenuRadioGroup
                            value={serverChoice}
                            onValueChange={(next) => {
                                if (next && next !== serverChoice) switchServer(next);
                            }}
                        >
                            {serverOptions.map((option) => (
                                <DropdownMenuRadioItem key={option.value} value={option.value} className="cursor-pointer">
                                    {option.label}
                                </DropdownMenuRadioItem>
                            ))}
                        </DropdownMenuRadioGroup>
                    </>
                ) : null}
            </DropdownMenuContent>
        </DropdownMenu>
    );
}
