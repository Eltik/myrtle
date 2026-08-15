import { MyTierLists } from "frontend";
import { type ReactNode, useEffect, useRef } from "react";

const noop = () => {};

/**
 * `MyTierLists` reads `myTierListsDetailedQueryOptions(authed)`. `useAuth` is stubbed
 * signed-out in previews, so the query stays disabled and the container resolves to its
 * empty-workshop branch — hero, toolbar and the create-first-list call to action.
 */
const search = { initialSort: "recent", initialType: "all", initialView: "grid", initialQuery: "", onPersistSearch: noop } as const;

/**
 * Opens the create dialog, whose open state lives inside the container. Two frames —
 * Base UI wires triggers after first paint. Two more frames then `blur()`: the dialog
 * auto-focuses the empty "Name" field and the brand-red focus ring around an empty
 * required input reads as a validation error. The dialog stays open through the blur.
 */
const ClickOnMount = ({ label, children }: { label: string; children: ReactNode }) => {
    const ref = useRef<HTMLDivElement>(null);
    useEffect(() => {
        let f2 = 0;
        let f3 = 0;
        let f4 = 0;
        const f1 = requestAnimationFrame(() => {
            f2 = requestAnimationFrame(() => {
                const buttons = Array.from(ref.current?.querySelectorAll("button") ?? []);
                buttons.find((b) => (b.textContent ?? "").includes(label))?.click();
                f3 = requestAnimationFrame(() => {
                    f4 = requestAnimationFrame(() => (document.activeElement as HTMLElement | null)?.blur());
                });
            });
        });
        return () => {
            cancelAnimationFrame(f1);
            cancelAnimationFrame(f2);
            cancelAnimationFrame(f3);
            cancelAnimationFrame(f4);
        };
    }, [label]);
    return (
        <div ref={ref} className="relative min-h-[520px]">
            {children}
        </div>
    );
};

export const EmptyWorkshop = () => <MyTierLists {...search} />;

export const CreateDialogOpen = () => (
    <ClickOnMount label="Create your first list">
        <MyTierLists {...search} />
    </ClickOnMount>
);
