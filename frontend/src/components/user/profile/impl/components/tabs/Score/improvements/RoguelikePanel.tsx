import type { IImprovementsResponse, IRoguelikeThemeImprovement } from "#/lib/api/user";
import { EmptyHint, PANEL_PADDING, ProgressLine, SectionHeader } from "./shared";

interface IProps {
    improvements: IImprovementsResponse;
    accent: string;
}

export function RoguelikePanel({ improvements, accent }: IProps) {
    const themes = improvements.roguelike;
    if (themes.length === 0) {
        return (
            <div className={PANEL_PADDING}>
                <EmptyHint>No roguelike themes loaded for this account.</EmptyHint>
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
    const collectibleTotal = theme.collectibles.relics.max + theme.collectibles.capsules.max + theme.collectibles.bands.max;
    const collectibleCurrent = theme.collectibles.relics.current + theme.collectibles.capsules.current + theme.collectibles.bands.current;
    return (
        <div className="flex flex-col gap-3">
            <SectionHeader title={theme.theme_name} count={theme.theme_id.toUpperCase()} accent={accent} />
            <div className="grid gap-2.5 sm:grid-cols-2">
                <ProgressLine label="Endings" current={theme.endings.current} max={theme.endings.max} accent={accent} />
                <ProgressLine label="BP levels" current={theme.bp.current} max={theme.bp.max} accent={accent} />
                {theme.difficulty.max > 0 && <ProgressLine label="Difficulty" current={Math.max(0, theme.difficulty.highest_cleared)} max={theme.difficulty.max} accent={accent} />}
                {theme.challenges.max > 0 && <ProgressLine label="Challenges" current={theme.challenges.current} max={theme.challenges.max} accent={accent} />}
                <ProgressLine label="Collectibles" current={collectibleCurrent} max={collectibleTotal} accent={accent} />
            </div>
            <div className="flex flex-wrap items-center gap-x-3 gap-y-1 font-mono text-[10px] text-muted-foreground">
                {theme.collectibles.relics.max > 0 && (
                    <span>
                        Relics{" "}
                        <span className="tabular-nums text-foreground/80">
                            {theme.collectibles.relics.current} / {theme.collectibles.relics.max}
                        </span>
                    </span>
                )}
                {theme.collectibles.capsules.max > 0 && (
                    <span>
                        Capsules{" "}
                        <span className="tabular-nums text-foreground/80">
                            {theme.collectibles.capsules.current} / {theme.collectibles.capsules.max}
                        </span>
                    </span>
                )}
                {theme.collectibles.bands.max > 0 && (
                    <span>
                        Bands{" "}
                        <span className="tabular-nums text-foreground/80">
                            {theme.collectibles.bands.current} / {theme.collectibles.bands.max}
                        </span>
                    </span>
                )}
            </div>
        </div>
    );
}
