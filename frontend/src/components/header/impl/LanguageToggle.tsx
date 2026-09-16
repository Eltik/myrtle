import { GlobeIcon } from "lucide-react";

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
 * A globe in the top bar, not a row inside the appearance popover: language is
 * the one setting people actively hunt for, and a globe is the affordance they
 * hunt for it with. Burying it behind a theme icon tests whether the visitor
 * can guess our information architecture.
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

    return (
        <DropdownMenu>
            <DropdownMenuTrigger
                render={
                    <Button variant="ghost" size="icon" aria-label={t("languageToggle.triggerAria", { label: triggerLabel })} title={triggerLabel}>
                        <GlobeIcon className="h-4 w-4" />
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
