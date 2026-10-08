import { ActiveFilters, CategoryChips } from "frontend";
import { useState } from "react";

// ActiveFilters closes the gallery's results header while anything filters:
// the chosen story as a removable primary-tinted chip (book icon, name, x),
// then a ghost "Clear all". With only a category or a query filtering, the
// story chip is absent and Clear all stands alone. ArtBrowser renders it
// after the result count, so the stories compose it in that row.

function ResultsRow({ initialGroup, count }: { initialGroup: string | null; count: number }) {
    const [group, setGroup] = useState<string | null>(initialGroup);
    return (
        <div style={{ width: 640 }} className="flex flex-wrap items-center gap-2">
            <span className="font-mono text-muted-foreground text-xs tabular-nums">{count} pictures</span>
            <ActiveFilters groupName={group} onRemoveGroup={() => setGroup(null)} onClear={() => setGroup(null)} />
        </div>
    );
}

/** A story chosen: its chip, then Clear all. */
export const StoryChosen = () => <ResultsRow initialGroup="Under Tides" count={19} />;

/** A long story name truncates inside the chip's 16rem cap. */
export const LongStoryName = () => <ResultsRow initialGroup="Operation Lucent Arrowhead" count={11} />;

/** Only a category filters: Clear all alone, beside the chips that are on. */
export const CategoryOnly = () => (
    <div style={{ width: 640 }} className="flex flex-col gap-2">
        <CategoryChips
            options={[
                { id: "main", count: 358 },
                { id: "side", count: 855 },
                { id: "vignette", count: 43 },
            ]}
            chosen={["vignette"]}
            onToggle={() => undefined}
        />
        <div className="flex items-center gap-2">
            <span className="font-mono text-muted-foreground text-xs tabular-nums">43 pictures</span>
            <ActiveFilters groupName={null} onRemoveGroup={() => undefined} onClear={() => undefined} />
        </div>
    </div>
);
