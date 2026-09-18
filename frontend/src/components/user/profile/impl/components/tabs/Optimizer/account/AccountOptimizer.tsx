import { useQuery } from "@tanstack/react-query";
import { Calculator } from "lucide-react";
import { useState } from "react";
import { Button } from "#/components/ui/button";
import { OperatorAvatar } from "#/components/ui/operator-avatar";
import { maxLevelCostQueryOptions } from "#/lib/api/user";
import { useFormatters, useT } from "#/lib/i18n";
import type { TypedT } from "#/lib/i18n/messages";
import { Headline } from "../base/controls/Headline";
import type { IOptimizerProps } from "../optimizers";
import type { messages } from "./AccountOptimizer.messages";

/** Rows shown before the list is expanded. */
const PREVIEW_ROWS = 12;

/**
 * The Account Optimizer's first card: what it takes to bring every owned
 * operator to its level cap. The figure is computed on demand (a button), not
 * on tab open, so browsing a profile never pays for it.
 */
export function AccountOptimizer({ uid }: IOptimizerProps) {
    const t: TypedT<typeof messages> = useT("user");
    const f = useFormatters();
    const [requested, setRequested] = useState(false);
    const [showAll, setShowAll] = useState(false);
    const query = useQuery({ ...maxLevelCostQueryOptions(uid), enabled: requested });
    const data = query.data;
    const rows = data ? (showAll ? data.operators : data.operators.slice(0, PREVIEW_ROWS)) : [];

    const run = () => {
        if (requested) {
            void query.refetch();
        } else {
            setRequested(true);
        }
    };

    return (
        <div className="flex flex-col gap-3">
            <div className="flex flex-wrap items-end justify-between gap-3 rounded-xl border border-border bg-card px-4 py-3">
                <div className="flex max-w-xl flex-col gap-0.5">
                    <p className="font-medium text-[13px] text-foreground">{t("profile.account.maxLevel.title")}</p>
                    <p className="text-[12px] text-muted-foreground">{t("profile.account.maxLevel.blurb")}</p>
                </div>
                <Button disabled={query.isFetching} onClick={run} size="sm">
                    <Calculator />
                    {query.isFetching ? t("profile.account.maxLevel.calculating") : data ? t("profile.account.maxLevel.recalculate") : t("profile.account.maxLevel.calculate")}
                </Button>
            </div>

            {query.isError && <p className="text-[12px] text-destructive">{t("profile.account.maxLevel.error")}</p>}

            {data && (
                <div className="flex flex-wrap gap-6 rounded-xl border border-border bg-card px-4 py-3">
                    <Headline label={t("profile.account.maxLevel.remaining")} value={`${data.operators_remaining} / ${data.operators_total}`} />
                    <Headline hint={t("profile.account.maxLevel.owned", { owned: f.number(data.exp_owned) })} label={t("profile.account.maxLevel.expNeeded")} value={f.number(data.exp_needed)} />
                    <Headline hint={t("profile.account.maxLevel.lmdSplit", { levels: f.number(data.level_lmd_needed), promotions: f.number(data.promotion_lmd_needed) })} label={t("profile.account.maxLevel.lmdNeeded")} value={f.number(data.lmd_needed)} />
                    <Headline label={t("profile.account.maxLevel.expMissing")} value={f.number(data.exp_missing)} />
                    <Headline hint={t("profile.account.maxLevel.owned", { owned: f.number(data.lmd_owned) })} label={t("profile.account.maxLevel.lmdMissing")} value={f.number(data.lmd_missing)} />
                </div>
            )}

            {data && data.operators_remaining === 0 && <p className="text-[12.5px] text-muted-foreground">{t("profile.account.maxLevel.allMaxed")}</p>}

            {data && data.operators_remaining > 0 && (
                <div className="overflow-x-auto rounded-xl border border-border bg-card">
                    <table className="w-full text-[12.5px]">
                        <thead>
                            <tr className="text-[10px] text-muted-foreground uppercase tracking-wider">
                                <th className="px-3 py-2 text-left font-medium">{t("profile.account.maxLevel.col.operator")}</th>
                                <th className="px-3 py-2 text-left font-medium">{t("profile.account.maxLevel.col.now")}</th>
                                <th className="px-3 py-2 text-left font-medium">{t("profile.account.maxLevel.col.target")}</th>
                                <th className="px-3 py-2 text-right font-medium">{t("profile.account.maxLevel.col.exp")}</th>
                                <th className="px-3 py-2 text-right font-medium">{t("profile.account.maxLevel.col.lmd")}</th>
                            </tr>
                        </thead>
                        <tbody>
                            {rows.map((op) => (
                                <tr className="border-border border-t" key={op.operator_id}>
                                    <td className="px-3 py-1.5">
                                        <span className="flex items-center gap-2">
                                            <OperatorAvatar charId={op.operator_id} className="size-7 shrink-0 overflow-hidden rounded-md" name={op.name} />
                                            <span className="truncate">{op.name}</span>
                                            <span className="text-[11px] text-muted-foreground">{"★".repeat(op.rarity)}</span>
                                        </span>
                                    </td>
                                    <td className="px-3 py-1.5 font-mono tabular-nums">{t("profile.account.maxLevel.elite", { elite: op.elite, level: op.level })}</td>
                                    <td className="px-3 py-1.5 font-mono tabular-nums">{t("profile.account.maxLevel.elite", { elite: op.target_elite, level: op.target_level })}</td>
                                    <td className="px-3 py-1.5 text-right font-mono tabular-nums">{f.number(op.exp)}</td>
                                    <td className="px-3 py-1.5 text-right font-mono tabular-nums">{f.number(op.level_lmd + op.promotion_lmd)}</td>
                                </tr>
                            ))}
                        </tbody>
                    </table>
                    {data.operators.length > PREVIEW_ROWS && (
                        <div className="border-border border-t px-3 py-2">
                            <Button onClick={() => setShowAll((v) => !v)} size="sm" variant="ghost">
                                {showAll ? t("profile.account.maxLevel.showFewer") : t("profile.account.maxLevel.showAll", { count: data.operators.length })}
                            </Button>
                        </div>
                    )}
                </div>
            )}

            <div className="rounded-xl border border-border border-dashed p-6 text-center">
                <p className="text-[12.5px] text-muted-foreground">{t("profile.account.rest")}</p>
            </div>
        </div>
    );
}
