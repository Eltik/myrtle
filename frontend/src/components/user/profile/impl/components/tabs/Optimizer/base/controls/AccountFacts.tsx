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
                        <span className="text-[12px] text-muted-foreground">Training</span>
                        <select aria-label="Class currently training" className="rounded-md border border-border bg-transparent px-1.5 py-1 text-[11px]" onChange={(e) => api.setTrainingClass(e.target.value === "" ? null : e.target.value)} value={api.trainingClass ?? ""}>
                            <option value="">Nobody</option>
                            {["Vanguard", "Guard", "Defender", "Sniper", "Caster", "Medic", "Supporter", "Specialist"].map((c) => (
                                <option key={c} value={c}>
                                    {c}
                                </option>
                            ))}
                        </select>
                        {api.factsSaved !== null && <span className={api.factsSaved ? "text-[10px] text-emerald-400" : "text-[10px] text-muted-foreground"}>{api.factsSaved ? "saved" : "what-if"}</span>}
                    </div>
                )}
            />
            <TooltipPopup className="max-w-64">Recruitment slots you have purchased beyond the first - account state the sync cannot read. Declaring it prices skills like Lin&rsquo;s Meritocracy (+10% HR speed per slot); left at +0 they honestly count as zero.</TooltipPopup>
        </Tooltip>
    );
}
