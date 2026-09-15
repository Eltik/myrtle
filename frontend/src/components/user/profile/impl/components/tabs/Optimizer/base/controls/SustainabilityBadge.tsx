import { CircleCheck, TriangleAlert } from "lucide-react";
import { Badge } from "#/components/ui/badge";
import { useT } from "#/lib/i18n";
import type { TypedT } from "#/lib/i18n/messages";
import type { messages } from "./SustainabilityBadge.messages";

interface IProps {
    verdict: string;
    depletedCount: number;
    dormOverflow: number;
    horizonHours: number;
}

export function SustainabilityBadge({ verdict, depletedCount, dormOverflow, horizonHours }: IProps) {
    const t: TypedT<typeof messages> = useT("user");
    const holds = verdict === "holds_up";
    const days = Math.round(horizonHours / 24);

    return (
        <div className="flex items-center gap-2">
            <Badge className="gap-1" variant={holds ? "secondary" : "destructive"}>
                {holds ? <CircleCheck className="h-3 w-3" /> : <TriangleAlert className="h-3 w-3" />}
                {holds ? t("profile.base.sustain.holds") : t("profile.base.sustain.depletes")}
            </Badge>
            <p className="text-[11px] text-muted-foreground">
                {holds ? t("profile.base.sustain.holdsDetail", { days }) : t("profile.base.sustain.depletesDetail", { count: depletedCount, days })}
                {dormOverflow > 0 ? t("profile.base.sustain.dormOverflow", { beds: dormOverflow }) : ""}
            </p>
        </div>
    );
}
