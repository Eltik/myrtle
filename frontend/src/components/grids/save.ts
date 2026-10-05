import { useMutation, useQueryClient } from "@tanstack/react-query";
import { type Dispatch, useRef, useState } from "react";
import { useErrorMessage } from "#/components/ui/error-message";
import { toastManager } from "#/components/ui/toast";
import { gridQueryOptions, updateGridFn } from "#/lib/api/grids";
import { useGamedataServer, useT } from "#/lib/i18n";
import type { TypedT } from "#/lib/i18n/messages";
import type { messages } from "./GridEditor.messages";
import { afterSave, type GridEditAction, gridToState, type IGridEditState, isGridDirty, toGridInput } from "./state";

interface IUseGridSaveArgs {
    slug: string;
    viewerId: string | null;
    /** The loaded grid as editor state: the first baseline. */
    initial: IGridEditState;
    state: IGridEditState;
    dispatch: Dispatch<GridEditAction>;
}

/**
 * The editor's save: the baseline `state` is dirty against, the PUT, and the
 * reconciliation of its response with edits made while it was in flight
 * (`afterSave`). A grid is saved whole, so there is one mutation.
 */
export function useGridSave({ slug, viewerId, initial, state, dispatch }: IUseGridSaveArgs) {
    const t: TypedT<typeof messages> = useT("grids");
    const describeError = useErrorMessage();
    const queryClient = useQueryClient();
    const server = useGamedataServer();
    const [original, setOriginal] = useState<IGridEditState>(initial);
    const [saveError, setSaveError] = useState<string | null>(null);

    // The state a save's response is compared against: edits made while it was in flight must survive it.
    const stateRef = useRef(state);
    stateRef.current = state;

    const mutation = useMutation({
        mutationFn: (snapshot: IGridEditState) => updateGridFn({ data: { slug, input: toGridInput(snapshot), server } }),
        onMutate: () => setSaveError(null),
        onSuccess: (saved, sent) => {
            queryClient.setQueryData(gridQueryOptions(slug, viewerId, server).queryKey, saved);
            void queryClient.invalidateQueries({ queryKey: ["grids", "browse"] });
            void queryClient.invalidateQueries({ queryKey: ["grids", "mine"] });
            const next = afterSave(sent, stateRef.current, gridToState(saved));
            setOriginal(next.original);
            if (next.state !== stateRef.current) dispatch({ type: "load", state: next.state });
            toastManager.add({ id: `grid-save-${Date.now()}`, title: t("edit.toast.savedTitle"), description: t("edit.toast.savedBody"), type: "success" });
        },
        onError: (err: unknown) => {
            const message = describeError(err);
            setSaveError(message);
            toastManager.add({ id: `grid-save-err-${Date.now()}`, title: t("edit.toast.saveFailedTitle"), description: message, type: "error" });
        },
    });

    return {
        dirty: isGridDirty(original, state),
        saving: mutation.isPending,
        saveError,
        save: (snapshot: IGridEditState) => mutation.mutate(snapshot),
        /** Back to the last saved state; the save error goes with the edits. */
        discard: () => {
            dispatch({ type: "load", state: original });
            setSaveError(null);
        },
    };
}
