import { lazy, Suspense, useState } from "react";
import type { ExportDialog, IExportDialogProps } from "./ExportDialog";

// `lazy` drops the dialog's type parameter, so the generic signature is restored by cast.
const LazyExportDialog = lazy(() => import("./ExportDialog").then((m) => ({ default: m.ExportDialog }))) as unknown as typeof ExportDialog;

/** Loads the dialog on its first open and keeps it mounted after, so the close animation and
 *  the dialog's own state across reopens behave as before while the exporters stay out of the
 *  list chunks. */
export function DynamicExportDialog<T>(props: IExportDialogProps<T>) {
    const [opened, setOpened] = useState(props.open);
    if (props.open && !opened) setOpened(true);
    if (!opened) return null;
    return (
        <Suspense fallback={null}>
            <LazyExportDialog {...props} />
        </Suspense>
    );
}
