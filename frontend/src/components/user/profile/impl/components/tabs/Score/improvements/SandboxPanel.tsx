import type { IImprovementsResponse, ISandboxCategory, ISandboxTopic } from "#/lib/api/user";
import { useFormatters, useT } from "#/lib/i18n";
import type { TypedT } from "#/lib/i18n/messages";
import { cn } from "#/lib/utils";
import type { messages } from "./SandboxPanel.messages";
import { EmptyHint, PANEL_PADDING, Pill, SectionHeader, TEXT_BADGE } from "./shared";

interface IProps {
    improvements: IImprovementsResponse;
    accent: string;
}

export function SandboxPanel({ improvements, accent }: IProps) {
    const t: TypedT<typeof messages> = useT("user");
    const s = improvements.sandbox;
    const hasAnyData = s.total > 0 || s.topics.some((topic) => topic.categories.some((c) => c.parts.some((p) => p.current > 0)));

    return (
        <div className={`${PANEL_PADDING} flex flex-col gap-5`}>
            {!hasAnyData && <EmptyHint>{t("score.improvements.sandbox.empty")}</EmptyHint>}
            {s.topics.map((topic) => (
                <TopicBlock key={topic.topic_id} topic={topic} shared={s.topics.length > 1} accent={accent} />
            ))}
        </div>
    );
}

function TopicBlock({ topic, shared, accent }: { topic: ISandboxTopic; shared: boolean; accent: string }) {
    const t: TypedT<typeof messages> = useT("user");
    const pct = Math.round(Math.min(1, Math.max(0, topic.total)) * 100);
    const count = shared ? t("score.improvements.sandbox.topicShare", { pct, share: Math.round(topic.weight * 100) }) : `${pct}%`;
    return (
        <div className="flex flex-col gap-3">
            {/* "Reclamation Algorithm" is the game mode's own name and stays as game vocabulary; it titles a topic the game data names no further. */}
            <SectionHeader title={topic.name || "Reclamation Algorithm"} count={count} accent={accent} />
            <div className="grid gap-x-5 gap-y-3 sm:grid-cols-2">
                {topic.categories.map((category) => (
                    <CategoryRow key={category.key} category={category} accent={accent} />
                ))}
            </div>
        </div>
    );
}

function CategoryRow({ category, accent }: { category: ISandboxCategory; accent: string }) {
    const t: TypedT<typeof messages> = useT("user");
    const f = useFormatters();
    const pct = Math.min(100, Math.max(0, category.score * 100));
    const weightPct = Math.round(category.weight * 100);
    return (
        <div className="flex flex-col gap-1.5">
            <div className="flex items-center justify-between gap-2">
                <span className="flex items-center gap-1.5">
                    <span className="text-[11px] text-foreground/85">{category.label}</span>
                    <Pill>{t("score.improvements.sandbox.weight", { pct: weightPct })}</Pill>
                </span>
                <span className={cn(TEXT_BADGE, "text-foreground/85")}>{pct.toFixed(0)}%</span>
            </div>
            <div className="h-1 w-full overflow-hidden rounded-full bg-muted/40">
                <div
                    className="h-full rounded-full transition-[width] duration-500 ease-out"
                    style={{
                        width: `${pct}%`,
                        background: `linear-gradient(to right, color-mix(in oklch, ${accent} 55%, transparent), ${accent})`,
                    }}
                />
            </div>
            <div className="flex flex-wrap gap-x-3 gap-y-0.5">
                {category.parts.map((p) => (
                    <span key={p.label} className="text-[10px] text-muted-foreground">
                        {p.label}{" "}
                        <span className="text-foreground/70 tabular-nums">
                            {f.number(p.current)}/{f.number(p.max)}
                        </span>
                    </span>
                ))}
            </div>
        </div>
    );
}
