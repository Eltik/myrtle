import { InfoHint } from "#/components/ui/info-hint";
import type { IImprovementsResponse, IRoguelikeThemeImprovement } from "#/lib/api/user";
import { useT } from "#/lib/i18n";
import type { TypedT } from "#/lib/i18n/messages";
import { cn } from "#/lib/utils";
import type { messages } from "./RoguelikePanel.messages";
import { EmptyHint, PANEL_PADDING, ProgressLine, SectionHeader, TEXT_BADGE } from "./shared";

interface IProps {
    improvements: IImprovementsResponse;
    accent: string;
}

export function RoguelikePanel({ improvements, accent }: IProps) {
    const t: TypedT<typeof messages> = useT("user");
    const themes = improvements.roguelike;
    if (themes.length === 0) {
        return (
            <div className={PANEL_PADDING}>
                <EmptyHint>{t("score.improvements.roguelike.empty")}</EmptyHint>
            </div>
        );
    }

    return (
        <div className={`${PANEL_PADDING} flex flex-col gap-5`}>
            {themes.map((theme) => (
                <ThemeBlock key={theme.theme_id} theme={theme} accent={accent} />
            ))}
        </div>
    );
}

function ThemeBlock({ theme, accent }: { theme: IRoguelikeThemeImprovement; accent: string }) {
    const t: TypedT<typeof messages> = useT("user");
    const collectibleTotal = theme.collectibles.relics.max + theme.collectibles.capsules.max + theme.collectibles.bands.max;
    const collectibleCurrent = theme.collectibles.relics.current + theme.collectibles.capsules.current + theme.collectibles.bands.current;
    return (
        <div className="flex flex-col gap-3">
            <SectionHeader title={theme.theme_name} count={theme.theme_id.toUpperCase()} accent={accent} />
            <div className="grid gap-2.5 sm:grid-cols-2">
                <ProgressLine label={t("score.improvements.roguelike.endings")} current={theme.endings.current} max={theme.endings.max} accent={accent} hint={<EndingsHint theme={theme} />} />
                <ProgressLine label={t("score.improvements.roguelike.bp")} current={theme.bp.current} max={theme.bp.max} accent={accent} />
                {theme.difficulty.max > 0 && <ProgressLine label={t("score.improvements.roguelike.difficulty")} current={Math.max(0, theme.difficulty.highest_cleared)} max={theme.difficulty.max} accent={accent} />}
                {theme.challenges.max > 0 && <ProgressLine label={t("score.improvements.roguelike.challenges")} current={theme.challenges.current} max={theme.challenges.max} accent={accent} />}
                <ProgressLine label={t("score.improvements.roguelike.collectibles")} current={collectibleCurrent} max={collectibleTotal} accent={accent} />
            </div>
            <div className={cn("flex flex-wrap items-center gap-x-3 gap-y-1 text-muted-foreground", TEXT_BADGE)}>
                {theme.collectibles.relics.max > 0 && (
                    <span>
                        {t("score.improvements.roguelike.relics")}{" "}
                        <span className="text-foreground/80">
                            {theme.collectibles.relics.current} / {theme.collectibles.relics.max}
                        </span>
                    </span>
                )}
                {theme.collectibles.capsules.max > 0 && (
                    <span>
                        {t("score.improvements.roguelike.capsules")}{" "}
                        <span className="text-foreground/80">
                            {theme.collectibles.capsules.current} / {theme.collectibles.capsules.max}
                        </span>
                    </span>
                )}
                {theme.collectibles.bands.max > 0 && (
                    <span>
                        {t("score.improvements.roguelike.bands")}{" "}
                        <span className="text-foreground/80">
                            {theme.collectibles.bands.current} / {theme.collectibles.bands.max}
                        </span>
                    </span>
                )}
            </div>
        </div>
    );
}

/**
 * The endings the count includes beyond the regular runs, or `null` for no note.
 *
 * The count is the game data's ending list, unchanged on purpose: `rogue_3`
 * lists 5 where players count 4 in game, and the fifth is real game data, so
 * the page explains it rather than hardcoding 4. Driven by `special_endings`,
 * never by a theme id, so a future theme with such an ending gets the note
 * with no change here. A MISSING field (a backend from before it existed, or a
 * theme with none, which the backend omits) is today's behaviour: no note.
 */
export function endingsNote(theme: IRoguelikeThemeImprovement): { endings: NonNullable<IRoguelikeThemeImprovement["special_endings"]>; regular: number; total: number } | null {
    if (theme.special_endings === undefined || theme.special_endings.length === 0) return null;
    return { endings: theme.special_endings, regular: Math.max(0, theme.endings.max - theme.special_endings.length), total: theme.endings.max };
}

function EndingsHint({ theme }: { theme: IRoguelikeThemeImprovement }) {
    const t: TypedT<typeof messages> = useT("user");
    const note = endingsNote(theme);
    if (note === null) return null;
    return (
        <InfoHint label={t("score.improvements.roguelike.endings.special.label")}>
            {note.endings.map((ending) => (ending.mode_name === undefined ? t("score.improvements.roguelike.endings.special.itemNoMode", { name: ending.name }) : t("score.improvements.roguelike.endings.special.item", { name: ending.name, mode: ending.mode_name }))).join(" ")}{" "}
            {t("score.improvements.roguelike.endings.special.count", { special: note.endings.length, regular: note.regular, total: note.total })}
        </InfoHint>
    );
}
