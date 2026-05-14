import { Skeleton } from "#/components/ui/skeleton";
import type { IImprovementsResponse } from "#/lib/api/user";
import type { ISubscore } from "../helpers";
import { BasePanel } from "./BasePanel";
import { MedalPanel } from "./MedalPanel";
import { OperatorPanel } from "./OperatorPanel";
import { RoguelikePanel } from "./RoguelikePanel";
import { SandboxPanel } from "./SandboxPanel";
import { StagePanel } from "./StagePanel";
import { PANEL_PADDING } from "./shared";

interface IProps {
    sub: ISubscore;
    improvements: IImprovementsResponse | null | undefined;
    isLoading: boolean;
}

/**
 * Dispatches to the right detail panel based on which subscore card the user
 * expanded. Renders a skeleton while improvements load (the query is shared
 * across all cards so the wait cost is paid once).
 */
export function ImprovementsPanel({ sub, improvements, isLoading }: IProps) {
    if (isLoading && !improvements) return <PanelSkeleton />;
    if (!improvements) {
        return (
            <div className={PANEL_PADDING}>
                <p className="rounded-md border border-dashed border-border/40 bg-muted/15 px-3 py-2 text-center text-[11px] text-muted-foreground">Improvements unavailable. Profile may be private or not yet synced.</p>
            </div>
        );
    }
    switch (sub.key) {
        case "operator_score":
            return <OperatorPanel improvements={improvements} accent={sub.color} />;
        case "base_score":
            return <BasePanel improvements={improvements} accent={sub.color} />;
        case "stage_score":
            return <StagePanel improvements={improvements} accent={sub.color} />;
        case "roguelike_score":
            return <RoguelikePanel improvements={improvements} accent={sub.color} />;
        case "sandbox_score":
            return <SandboxPanel improvements={improvements} accent={sub.color} />;
        case "medal_score":
            return <MedalPanel improvements={improvements} accent={sub.color} />;
        default:
            return null;
    }
}

function PanelSkeleton() {
    return (
        <div className={`${PANEL_PADDING} flex flex-col gap-3`}>
            <Skeleton className="h-3 w-32" />
            <Skeleton className="h-2 w-full" />
            <div className="grid gap-2 sm:grid-cols-2">
                <Skeleton className="h-10 rounded-md" />
                <Skeleton className="h-10 rounded-md" />
                <Skeleton className="h-10 rounded-md" />
                <Skeleton className="h-10 rounded-md" />
            </div>
        </div>
    );
}
