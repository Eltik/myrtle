import { CircleCheck, TriangleAlert } from "lucide-react";
import { Badge } from "#/components/ui/badge";

interface IProps {
    verdict: string;
    depletedCount: number;
    dormOverflow: number;
    horizonHours: number;
}

export function SustainabilityBadge({ verdict, depletedCount, dormOverflow, horizonHours }: IProps) {
    const holds = verdict === "holds_up";
    const days = Math.round(horizonHours / 24);

    return (
        <div className="flex items-center gap-2">
            <Badge className="gap-1" variant={holds ? "secondary" : "destructive"}>
                {holds ? <CircleCheck className="h-3 w-3" /> : <TriangleAlert className="h-3 w-3" />}
                {holds ? "Holds up" : "Depletes"}
            </Badge>
            <p className="text-[11px] text-muted-foreground">
                {holds ? `Nobody runs dry over ${days} simulated day${days === 1 ? "" : "s"}.` : `${depletedCount} operator${depletedCount === 1 ? "" : "s"} hit zero morale within ${days} day${days === 1 ? "" : "s"}.`}
                {dormOverflow > 0 ? ` Dorms are ${dormOverflow} bed${dormOverflow === 1 ? "" : "s"} short at peak.` : ""}
            </p>
        </div>
    );
}
