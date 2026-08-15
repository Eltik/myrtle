import { MyToolbar } from "frontend";
import { type ReactNode, useEffect, useRef } from "react";

const noop = () => {};

/** Opens a Base UI menu whose trigger is internal — deferred two frames so the trigger is wired. */
const ClickOnMount = ({ label, children }: { label: string; children: ReactNode }) => {
    const ref = useRef<HTMLDivElement>(null);
    useEffect(() => {
        let inner = 0;
        const outer = requestAnimationFrame(() => {
            inner = requestAnimationFrame(() => {
                const buttons = Array.from(ref.current?.querySelectorAll("button") ?? []);
                buttons.find((b) => (b.textContent ?? "").includes(label))?.click();
            });
        });
        return () => {
            cancelAnimationFrame(outer);
            cancelAnimationFrame(inner);
        };
    }, [label]);
    return (
        <div ref={ref} className="relative min-h-[520px]">
            {children}
        </div>
    );
};

export const Default = () => <MyToolbar sort="recent" type="all" view="grid" query="" resultCount={7} totalCount={7} hasOfficial onSortChange={noop} onTypeChange={noop} onViewChange={noop} onQueryChange={noop} />;

export const FilteredSearch = () => <MyToolbar sort="views" type="community" view="grid" query="endgame" resultCount={2} totalCount={7} hasOfficial onSortChange={noop} onTypeChange={noop} onViewChange={noop} onQueryChange={noop} />;

export const ListViewNoOfficial = () => <MyToolbar sort="alpha" type="all" view="list" query="" resultCount={4} totalCount={4} hasOfficial={false} onSortChange={noop} onTypeChange={noop} onViewChange={noop} onQueryChange={noop} />;

export const SortMenuOpen = () => (
    <ClickOnMount label="Recently updated">
        <MyToolbar sort="recent" type="all" view="grid" query="" resultCount={7} totalCount={7} hasOfficial onSortChange={noop} onTypeChange={noop} onViewChange={noop} onQueryChange={noop} />
    </ClickOnMount>
);
