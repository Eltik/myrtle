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
 * The active language's code in the top bar, not a row in the appearance
 * popover: language is the one setting people hunt for. Visible at every
 * width; the hamburger drawer carries the same choice as a second path. The
 * room comes from the wordmark, which `Header` hides below `sm`, so a new
 * button in this cluster takes its space from something else.
 *
 * The game-text picker shares this menu because it defaults to the language:
 * one decision ("what am I reading") with an override. Renders nothing with
 * neither a second locale nor a second loaded game-data server.
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
                        {/* The code, not a globe: it also says WHICH language, at the
                            same width. Not a flag: a language is not a country. */}
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
