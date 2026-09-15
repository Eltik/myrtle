import { ToggleGroup, ToggleGroupItem } from "#/components/ui/toggle-group";
import { Tooltip, TooltipPopup, TooltipTrigger } from "#/components/ui/tooltip";
import { useT } from "#/lib/i18n";
import type { TypedT } from "#/lib/i18n/messages";
import { useBaseOptimizer } from "../base-context";
import type { messages } from "./AccountFacts.messages";

/**
 * Account facts the sync cannot read, declared once and folded into every
 * score. Today that is one fact: recruit slots purchased beyond the initial
 * one, which prices per-slot HR skills (Lin's Meritocracy) that otherwise
 * honestly count as zero.
 */
export function AccountFacts() {
    const t: TypedT<typeof messages> = useT("user");
    const api = useBaseOptimizer();

    return (
        <Tooltip>
            <TooltipTrigger
                render={(props) => (
                    <div {...props} className="flex items-center gap-2">
                        <span className="text-[12px] text-muted-foreground">{t("profile.base.facts.recruitSlots")}</span>
                        <ToggleGroup
                            aria-label={t("profile.base.facts.recruitSlots.aria")}
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
                        <span className="text-[12px] text-muted-foreground">{t("profile.base.facts.training")}</span>
                        <select aria-label={t("profile.base.facts.training.aria")} className="rounded-md border border-border bg-transparent px-1.5 py-1 text-[11px]" onChange={(e) => api.setTrainingClass(e.target.value === "" ? null : e.target.value)} value={api.trainingClass ?? ""}>
                            <option value="">{t("profile.base.facts.training.nobody")}</option>
                            {/* The class names are both the API value and game vocabulary, so they stay as-is. */}
                            {["Vanguard", "Guard", "Defender", "Sniper", "Caster", "Medic", "Supporter", "Specialist"].map((c) => (
                                <option key={c} value={c}>
                                    {c}
                                </option>
                            ))}
                        </select>
                        {api.factsSaved !== null && <span className={api.factsSaved ? "text-[10px] text-emerald-400" : "text-[10px] text-muted-foreground"}>{api.factsSaved ? t("profile.base.facts.saved") : t("profile.base.facts.whatIf")}</span>}
                    </div>
                )}
            />
            <TooltipPopup className="max-w-64">{t("profile.base.facts.recruitSlots.tooltip")}</TooltipPopup>
        </Tooltip>
    );
}
