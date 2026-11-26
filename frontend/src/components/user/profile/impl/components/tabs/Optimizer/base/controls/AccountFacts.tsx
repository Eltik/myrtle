import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from "#/components/ui/select";
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
                        <Select onValueChange={(next: string | null) => api.setTrainingClass(next)} value={api.trainingClass ?? null}>
                            <SelectTrigger aria-label={t("profile.base.facts.training.aria")} className="w-auto min-w-28" size="sm">
                                <SelectValue>{(value: string | null) => value ?? t("profile.base.facts.training.nobody")}</SelectValue>
                            </SelectTrigger>
                            <SelectContent>
                                <SelectItem value={null}>{t("profile.base.facts.training.nobody")}</SelectItem>
                                {/* The class names are both the API value and game vocabulary, so they stay as-is. */}
                                {["Vanguard", "Guard", "Defender", "Sniper", "Caster", "Medic", "Supporter", "Specialist"].map((c) => (
                                    <SelectItem key={c} value={c}>
                                        {c}
                                    </SelectItem>
                                ))}
                            </SelectContent>
                        </Select>
                        {api.factsSaved !== null && <span className={api.factsSaved ? "text-[10px] text-success-foreground" : "text-[10px] text-muted-foreground"}>{api.factsSaved ? t("profile.base.facts.saved") : t("profile.base.facts.whatIf")}</span>}
                    </div>
                )}
            />
            <TooltipPopup className="max-w-64">{t("profile.base.facts.recruitSlots.tooltip")}</TooltipPopup>
        </Tooltip>
    );
}
