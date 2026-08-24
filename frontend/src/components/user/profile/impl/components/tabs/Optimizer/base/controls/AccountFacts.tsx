import { ToggleGroup, ToggleGroupItem } from "#/components/ui/toggle-group";
import { Tooltip, TooltipPopup, TooltipTrigger } from "#/components/ui/tooltip";
import { useBaseOptimizer } from "../base-context";

/**
 * Account facts the sync cannot read, declared once and folded into every
 * score. Today that is one fact: recruit slots purchased beyond the initial
 * one, which prices per-slot HR skills (Lin's Meritocracy) that otherwise
 * honestly count as zero.
 */
export function AccountFacts() {
    const api = useBaseOptimizer();

    return (
        <Tooltip>
            <TooltipTrigger
                render={(props) => (
                    <div {...props} className="flex items-center gap-2">
                        <span className="text-[12px] text-muted-foreground">Recruit slots</span>
                        <ToggleGroup
                            aria-label="Recruit slots purchased beyond the first"
                            onValueChange={(next: string[]) => {
                                if (next[0] !== undefined) api.setOpenRecruitSlots(Number(next[0]));
                            }}
                            value={[String(api.openRecruitSlots)]}
                        >
                            {[0, 1, 2, 3].map((n) => (
                                <ToggleGroupItem key={n} size="sm" value={String(n)}>
                                    {n === 0 ? "+0" : `+${n}`}
                                </ToggleGroupItem>
                            ))}
                        </ToggleGroup>
                    </div>
                )}
            />
            <TooltipPopup className="max-w-64">Recruitment slots you have purchased beyond the first - account state the sync cannot read. Declaring it prices skills like Lin&rsquo;s Meritocracy (+10% HR speed per slot); left at +0 they honestly count as zero.</TooltipPopup>
        </Tooltip>
    );
}
