import { useEffect, useRef } from "react";
import { VersionsBar } from "frontend";

const version = (v: number, publishedAt: string, changelog: string | null) => ({
    id: `ver-${v}`,
    tierListId: "tl-global-meta",
    version: v,
    snapshot: { tiers: [] },
    changelog,
    publishedBy: "u-myrtle",
    publishedAt,
});

const versions = [
    version(4, "2024-05-14T08:30:00.000Z", "Moved **Logos** into S+ after the CN Risk 18 clears landed. Degenbrecher swaps places with Surtr; her module stage 2 is now the assumed baseline."),
    version(3, "2024-04-26T19:05:00.000Z", "Added the B tier for budget clears and moved Myrtle, Texas and Ptilopsis into it."),
    version(2, "2024-03-19T11:20:00.000Z", "Post-banner pass: Wiš'adel enters at S+, Rosmontis drops to A."),
    version(1, "2024-02-04T09:00:00.000Z", null),
];

export const LatestLive = () => <VersionsBar slug="global-6-star-meta" versions={versions} selectedVersion={null} isLatestView />;

export const ViewingSnapshot = () => <VersionsBar slug="global-6-star-meta" versions={versions} selectedVersion={versions[2]} isLatestView={false} />;

export const SnapshotWithoutChangelog = () => <VersionsBar slug="global-6-star-meta" versions={versions} selectedVersion={versions[3]} isLatestView={false} />;

// The version menu owns its own open state, so a static story would only ever
// show the closed bar. Click the trigger after the first paint (Base UI wires it
// up then) to photograph the picker the way a reader actually meets it.
function OpenedPicker() {
    const ref = useRef<HTMLDivElement | null>(null);

    useEffect(() => {
        const outer = requestAnimationFrame(() => {
            requestAnimationFrame(() => {
                ref.current?.querySelector<HTMLElement>('button[aria-label^="Select version"]')?.click();
            });
        });
        return () => cancelAnimationFrame(outer);
    }, []);

    return (
        <div ref={ref} className="pb-16">
            <VersionsBar slug="global-6-star-meta" versions={versions} selectedVersion={null} isLatestView />
        </div>
    );
}

export const VersionPickerOpen = () => <OpenedPicker />;
