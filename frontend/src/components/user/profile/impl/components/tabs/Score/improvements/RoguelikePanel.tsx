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
                <ProgressLine label={t("score.improvements.roguelike.endings")} current={theme.endings.current} max={theme.endings.max} accent={accent} />
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
