import { useQueryClient } from "@tanstack/react-query";
import { Pencil, Plus, Trash } from "lucide-react";
import * as React from "react";

import { Button } from "#/components/ui/button";
import { Combobox, ComboboxEmpty, ComboboxInput, ComboboxList, ComboboxPopup, ComboboxPrimitive } from "#/components/ui/combobox";
import { Input } from "#/components/ui/input";
import { deleteGroupFn, upsertGroupFn } from "#/lib/api/planner";
import { useT } from "#/lib/i18n";
import type { TypedT } from "#/lib/i18n/messages";
import { compactForSearch } from "#/lib/search/fuzzy";
import { cn } from "#/lib/utils";
import type { messages } from "./OperatorPlannerDialog.messages";
import { PLANS_QUERY_PREFIX } from "./planTargets";

interface IPlanGroupsFieldProps {
    /** Every group the player has, in the API's order. */
    groupNames: string[];
    selectedGroups: string[];
    onSelectedGroupsChange: React.Dispatch<React.SetStateAction<string[]>>;
}

/**
 * The plan's group multi-select. Groups are created, renamed and deleted from
 * inside the popup; each is saved straight away (independent of the plan's
 * Save button) and the plan's own selection follows the new name.
 *
 * The search, open and create-row state is all transient popup state: closing
 * the popup clears it, and the field unmounts with the rest of the target form
 * when the dialog closes.
 */
export function PlanGroupsField({ groupNames, selectedGroups, onSelectedGroupsChange }: IPlanGroupsFieldProps): React.ReactElement {
    const t: TypedT<typeof messages> = useT("tools");
    const queryClient = useQueryClient();
    const [searchQuery, setSearchQuery] = React.useState<string>("");
    const [isOpen, setIsOpen] = React.useState<boolean>(false);
    const [isCreating, setIsCreating] = React.useState<boolean>(false);
    const [newGroupName, setNewGroupName] = React.useState<string>("");

    const filteredGroupNames = React.useMemo(() => {
        if (!searchQuery.trim()) return groupNames;
        const query = compactForSearch(searchQuery);
        return groupNames.filter((name) => compactForSearch(name).includes(query));
    }, [groupNames, searchQuery]);

    const cancelCreate = () => {
        setIsCreating(false);
        setNewGroupName("");
    };

    const invalidatePlans = () => queryClient.invalidateQueries({ queryKey: PLANS_QUERY_PREFIX });

    const handleCreate = async () => {
        const name = newGroupName.trim();
        if (!name) return;
        try {
            await upsertGroupFn({ data: { name } });
            onSelectedGroupsChange((prev) => (prev.includes(name) ? prev : [...prev, name]));
            cancelCreate();
            invalidatePlans();
        } catch (err) {
            console.error(err);
        }
    };

    const handleRename = async (oldName: string) => {
        const newName = window.prompt(t("planner.dialog.groups.renamePrompt"), oldName);
        if (newName === null) return;
        const trimmed = newName.trim();
        if (!trimmed || trimmed === oldName) return;
        try {
            await upsertGroupFn({ data: { oldName, name: trimmed } });
            onSelectedGroupsChange((prev) => prev.map((g) => (g === oldName ? trimmed : g)));
            invalidatePlans();
        } catch (err) {
            console.error(err);
        }
    };

    const handleDelete = async (name: string) => {
        if (!window.confirm(t("planner.dialog.groups.deleteConfirm", { name }))) return;
        try {
            await deleteGroupFn({ data: { name } });
            onSelectedGroupsChange((prev) => prev.filter((g) => g !== name));
            invalidatePlans();
        } catch (err) {
            console.error(err);
        }
    };

    /** The row's own buttons act on the group; they must not also toggle it in the selection. */
    const rowAction = (action: () => void) => (e: React.MouseEvent) => {
        e.stopPropagation();
        e.preventDefault();
        action();
    };

    return (
        <div className="space-y-2">
            <label className="block font-medium text-[13px] text-muted-foreground leading-none" htmlFor="group-selector">
                {t("planner.dialog.groups")}
            </label>
            <Combobox<string, true>
                multiple
                items={filteredGroupNames}
                value={selectedGroups}
                onValueChange={onSelectedGroupsChange}
                filter={null}
                open={isOpen}
                onOpenChange={(open) => {
                    setIsOpen(open);
                    if (!open) {
                        setSearchQuery("");
                        cancelCreate();
                    }
                }}
                // Closed, the input shows the selection; open, it is the search box.
                inputValue={isOpen ? searchQuery : selectedGroups.join(", ")}
                onInputValueChange={(value) => {
                    if (isOpen) setSearchQuery(value);
                }}
                itemToStringLabel={(g) => g ?? ""}
                itemToStringValue={(g) => g ?? ""}
            >
                <ComboboxInput id="group-selector" placeholder={t("planner.dialog.groups.placeholder")} className="truncate text-ellipsis" />
                <ComboboxPopup className="max-w-100">
                    {isCreating ? (
                        <div className="flex items-center gap-2 border-border border-b p-2">
                            <Input
                                value={newGroupName}
                                onChange={(e) => setNewGroupName(e.target.value)}
                                placeholder={t("planner.dialog.groups.newName")}
                                className="h-8 flex-1 text-xs"
                                autoFocus
                                onKeyDown={(e) => {
                                    if (e.key === "Enter") {
                                        e.preventDefault();
                                        handleCreate();
                                    } else if (e.key === "Escape") {
                                        e.preventDefault();
                                        cancelCreate();
                                    }
                                }}
                            />
                            <Button size="sm" className="h-8 px-2 text-xs" onClick={handleCreate}>
                                {t("planner.dialog.groups.create")}
                            </Button>
                            <Button variant="outline" size="sm" className="h-8 px-2 text-xs" onClick={cancelCreate}>
                                {t("planner.dialog.groups.cancel")}
                            </Button>
                        </div>
                    ) : (
                        <div className="border-border border-b p-1">
                            <button type="button" onClick={() => setIsCreating(true)} className="flex w-full items-center gap-2 rounded-md px-2 py-1.5 text-left font-medium text-primary text-xs hover:bg-primary/10 hover:text-primary">
                                <Plus className="h-3.5 w-3.5" />
                                {t("planner.dialog.groups.createNew")}
                            </button>
                        </div>
                    )}
                    <ComboboxEmpty>{t("planner.dialog.groups.none")}</ComboboxEmpty>
                    <ComboboxList>
                        {(name: string) => {
                            const isSelected = selectedGroups.includes(name);
                            return (
                                <ComboboxPrimitive.Item
                                    key={name}
                                    value={name}
                                    className="flex min-h-8 cursor-default items-center justify-between gap-2 rounded-sm px-2 py-1 text-sm outline-none data-disabled:pointer-events-none data-highlighted:bg-accent data-highlighted:text-accent-foreground data-disabled:opacity-64 sm:min-h-7"
                                >
                                    <div className="flex items-center gap-2">
                                        <span className={cn("flex size-4.5 shrink-0 items-center justify-center rounded-sm border border-input transition-colors sm:size-4", isSelected ? "border-primary bg-primary text-primary-foreground" : "border-input bg-background")}>
                                            {isSelected && (
                                                <svg aria-hidden="true" className="size-3 sm:size-2.5" fill="none" height="24" stroke="currentColor" strokeLinecap="round" strokeLinejoin="round" strokeWidth="3" viewBox="0 0 24 24" width="24" xmlns="http://www.w3.org/2000/svg">
                                                    <path d="M5.252 12.7 10.2 18.63 18.748 5.37" />
                                                </svg>
                                            )}
                                        </span>
                                        <span>{name}</span>
                                    </div>
                                    <div className="flex items-center gap-1">
                                        <button type="button" onClick={rowAction(() => handleRename(name))} className="rounded p-1 text-muted-foreground hover:bg-muted hover:text-foreground">
                                            <Pencil className="h-3.5 w-3.5" />
                                        </button>
                                        <button type="button" onClick={rowAction(() => handleDelete(name))} className="rounded p-1 text-muted-foreground hover:bg-muted hover:text-destructive">
                                            <Trash className="h-3.5 w-3.5" />
                                        </button>
                                    </div>
                                </ComboboxPrimitive.Item>
                            );
                        }}
                    </ComboboxList>
                </ComboboxPopup>
            </Combobox>
        </div>
    );
}
