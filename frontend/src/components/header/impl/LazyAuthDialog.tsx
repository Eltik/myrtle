import { cloneElement, lazy, Suspense, useEffect, useState } from "react";
import { authActions } from "#/lib/auth/store";

// The sign-in dialog is about 26 kB of the entry chunk that most visitors never open, so it loads
// on demand: the first hover or focus of a trigger starts the fetch, and the dialog mounts the
// first time it opens (then stays mounted, so its close animation still plays).
const loadAuthDialog = () => import("./AuthDialog");
const AuthDialog = lazy(() => loadAuthDialog().then((m) => ({ default: m.AuthDialog })));

/** Starts fetching the dialog's chunk. Safe to call repeatedly; a failed preload is ignored, and opening loads it again. */
export function preloadAuthDialog(): void {
    loadAuthDialog().catch(() => undefined);
}

interface ITriggerProps {
    onClick?: (event: React.MouseEvent<HTMLElement>) => void;
    onPointerEnter?: (event: React.PointerEvent<HTMLElement>) => void;
    onFocus?: (event: React.FocusEvent<HTMLElement>) => void;
}

interface ILazyAuthDialogProps {
    trigger?: React.ReactElement<ITriggerProps>;
    onOpenChange?: (open: boolean) => void;
    open?: boolean;
}

/** `AuthDialog` with the same props, loaded on first use. */
export function LazyAuthDialog({ trigger, onOpenChange, open: openProp }: ILazyAuthDialogProps): React.ReactElement | null {
    const [internalOpen, setInternalOpen] = useState(false);
    const isOpen = openProp ?? internalOpen;
    const setOpen = onOpenChange ?? setInternalOpen;
    const [mounted, setMounted] = useState(false);
    if (isOpen && !mounted) setMounted(true);

    // `AuthDialog` applies the remembered "save credentials" answer on mount; doing it here keeps
    // that at page load rather than at first open.
    useEffect(() => {
        authActions.hydrateSaveCredentials();
    }, []);

    if (!mounted) {
        if (!trigger) return null;
        return cloneElement(trigger, {
            onClick: () => setOpen(true),
            onPointerEnter: preloadAuthDialog,
            onFocus: preloadAuthDialog,
        });
    }
    return (
        <Suspense fallback={trigger ?? null}>
            <AuthDialog trigger={trigger} open={isOpen} onOpenChange={setOpen} />
        </Suspense>
    );
}
