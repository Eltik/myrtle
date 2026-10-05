import { useQuery, useQueryClient } from "@tanstack/react-query";
import { Trash } from "lucide-react";
import * as React from "react";

import { Button } from "#/components/ui/button";
import { useErrorMessage } from "#/components/ui/error-message";
import { Input } from "#/components/ui/input";
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from "#/components/ui/select";
import { deletePlanPresetFn, type IPresetTarget, PLAN_PRESETS_QUERY_KEY, planPresetsQueryOptions, upsertPlanPresetFn } from "#/lib/api/planner";
import { useT } from "#/lib/i18n";
import type { TypedT } from "#/lib/i18n/messages";
import type { messages } from "./BulkPlanDialog.messages";
import { normalizePresetTarget } from "./bulkTargets";

interface IPresetRowProps {
    target: IPresetTarget;
    onLoad: (target: IPresetTarget) => void;
}

/** Load, save and delete named presets. Each action is saved straight away, independent of the dialog's Save. */
export function PresetRow({ target, onLoad }: IPresetRowProps): React.ReactElement {
    const t: TypedT<typeof messages> = useT("tools");
    const errorMessage = useErrorMessage();
    const queryClient = useQueryClient();
    const { data: presets = [] } = useQuery(planPresetsQueryOptions());
    const [chosen, setChosen] = React.useState<string | null>(null);
    const [name, setName] = React.useState<string>("");
    const [isBusy, setIsBusy] = React.useState<boolean>(false);
    const [error, setError] = React.useState<string | null>(null);

    const sorted = React.useMemo(() => [...presets].sort((a, b) => a.name.localeCompare(b.name)), [presets]);
    const trimmed = name.trim();
    const isReplace = sorted.some((p) => p.name === trimmed);

    const run = async (action: () => Promise<unknown>) => {
        setIsBusy(true);
        setError(null);
        try {
            await action();
            await queryClient.invalidateQueries({ queryKey: PLAN_PRESETS_QUERY_KEY });
            return true;
        } catch (err) {
            setError(errorMessage(err));
            return false;
        } finally {
            setIsBusy(false);
        }
    };

    const handleChoose = (value: string | null) => {
        setChosen(value);
        const preset = sorted.find((p) => p.name === value);
        if (!preset) return;
        onLoad(normalizePresetTarget(preset.target));
        setName(preset.name);
    };

    const handleSave = async () => {
        if (!trimmed) return;
        if (await run(() => upsertPlanPresetFn({ data: { name: trimmed, target } }))) setChosen(trimmed);
    };

    const handleDelete = async () => {
        if (!chosen) return;
        const deleted = chosen;
        if (await run(() => deletePlanPresetFn({ data: deleted }))) {
            setChosen(null);
            if (name.trim() === deleted) setName("");
        }
    };

    return (
        <div className="space-y-3 rounded-xl border border-border bg-card/40 p-4">
            <span className="font-semibold text-muted-foreground text-xs uppercase tracking-wider">{t("planner.bulk.preset")}</span>
            <div className="flex items-center gap-2">
                <Select value={chosen} onValueChange={(v: string | null) => handleChoose(v)}>
                    <SelectTrigger className="min-w-0 flex-1" aria-label={t("planner.bulk.preset")}>
                        <SelectValue placeholder={t("planner.bulk.presetPlaceholder")}>{(value: string | null) => value ?? t("planner.bulk.presetPlaceholder")}</SelectValue>
                    </SelectTrigger>
                    <SelectContent>
                        {sorted.length === 0 ? (
                            <p className="px-2 py-1.5 text-muted-foreground text-xs">{t("planner.bulk.presetNone")}</p>
                        ) : (
                            sorted.map((preset) => (
                                <SelectItem key={preset.id} value={preset.name}>
                                    {preset.name}
                                </SelectItem>
                            ))
                        )}
                    </SelectContent>
                </Select>
                <Button variant="outline" size="icon" onClick={handleDelete} disabled={!chosen || isBusy} aria-label={t("planner.bulk.presetDelete")} title={t("planner.bulk.presetDelete")}>
                    <Trash className="size-4" />
                </Button>
            </div>
            <div className="flex items-center gap-2">
                <Input
                    value={name}
                    onChange={(e) => setName(e.target.value)}
                    placeholder={t("planner.bulk.presetName")}
                    maxLength={100}
                    className="min-w-0 flex-1"
                    onKeyDown={(e) => {
                        if (e.key === "Enter") {
                            e.preventDefault();
                            handleSave();
                        }
                    }}
                />
                <Button variant="outline" onClick={handleSave} disabled={!trimmed || isBusy}>
                    {isReplace ? t("planner.bulk.presetReplace") : t("planner.bulk.presetSave")}
                </Button>
            </div>
            {error && (
                <p role="alert" className="text-destructive-foreground text-xs">
                    {error}
                </p>
            )}
        </div>
    );
}
