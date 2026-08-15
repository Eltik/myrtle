import { useEffect, useRef } from "react";
import { EnemyLocationFilter } from "frontend";

// Tree shape produced by `buildLocationTree` from the backend's enemy-stage
// index. These are the real zones Originium Slug (B1) shows up in.
const TREE = [
    {
        key: "story",
        label: "Main Story",
        zones: [
            {
                token: "main_0",
                label: "Evil Time Part 1",
                stages: [
                    { token: "level_main_00-01", label: "0-1" },
                    { token: "level_main_00-02", label: "0-2" },
                    { token: "level_main_00-10", label: "0-10" },
                    { token: "level_main_00-11", label: "0-11" },
                ],
            },
            {
                token: "main_1",
                label: "Evil Time Part 2",
                stages: [
                    { token: "level_main_01-02", label: "1-2" },
                    { token: "level_main_01-03", label: "1-3" },
                    { token: "level_main_01-08", label: "1-8" },
                    { token: "level_main_01-10", label: "1-10" },
                ],
            },
            {
                token: "main_14",
                label: "Absolved Will Be the Seekers",
                stages: [
                    { token: "level_main_14-01", label: "14-2" },
                    { token: "level_tough_14-01", label: "14-2 (Adverse)" },
                ],
            },
        ],
    },
    {
        key: "events",
        label: "Events",
        zones: [
            {
                token: "act13d2",
                label: "Grani and the Knights' Treasure - Rerun",
                stages: [
                    { token: "level_a001_02", label: "GT-2" },
                    { token: "level_a001_ex01", label: "GT-EX-1" },
                ],
            },
            {
                token: "act1autochess",
                label: "Stronghold Protocol: Alliance",
                stages: [
                    { token: "level_act1autochess_01", label: "01" },
                    { token: "level_act1autochess_tr01", label: "tr01" },
                    { token: "level_act1autochess_tr02", label: "tr02" },
                ],
            },
        ],
    },
    {
        key: "is",
        label: "Integrated Strategies",
        zones: [
            {
                token: "rogue_1",
                label: "Phantom & Crimson Solitaire",
                stages: [
                    { token: "level_rogue1_1-1", label: "A Date With Slugs" },
                    { token: "level_rogue1_1-4", label: "Accident" },
                ],
            },
            {
                token: "rogue_2",
                label: "Mizuki & Caerula Arbor",
                stages: [
                    { token: "level_rogue2_1-1", label: "Cistern" },
                    { token: "level_rogue2_1-2", label: "Insect Infestation" },
                ],
            },
        ],
    },
    {
        key: "supplies",
        label: "Supplies",
        zones: [
            { token: "weekly_5", label: "Tough Siege", stages: [{ token: "level_weekly_toxic_1", label: "AP-1" }] },
            {
                token: "weekly_9",
                label: "Cargo Escort",
                stages: [
                    { token: "level_weekly_melee_1", label: "CE-1" },
                    { token: "level_weekly_melee_2", label: "CE-2" },
                ],
            },
        ],
    },
];

const noop = () => undefined;

// The popover owns its own open state and exposes no `open` prop, so the open
// story clicks the trigger after Base UI has wired it up (two frames after
// mount — firing from the effect body is dropped).
function AutoOpen({ children }: { children: React.ReactNode }) {
    const ref = useRef<HTMLDivElement>(null);
    useEffect(() => {
        let raf2 = 0;
        const raf1 = requestAnimationFrame(() => {
            raf2 = requestAnimationFrame(() => {
                ref.current?.querySelector<HTMLElement>('[aria-label="Filter by where enemies appear"]')?.click();
            });
        });
        return () => {
            cancelAnimationFrame(raf1);
            cancelAnimationFrame(raf2);
        };
    }, []);
    return <div ref={ref}>{children}</div>;
}

export const ClosedTrigger = () => <EnemyLocationFilter tree={TREE} selected={[]} onChange={noop} />;

export const WithSelections = () => <EnemyLocationFilter tree={TREE} selected={["main_0", "level_main_14-01", "level_rogue2_1-2", "weekly_9"]} onChange={noop} />;

export const OpenTree = () => (
    <div className="flex min-h-[520px] items-start">
        <AutoOpen>
            <EnemyLocationFilter tree={TREE} selected={[]} onChange={noop} />
        </AutoOpen>
    </div>
);

export const OpenWithSelections = () => (
    <div className="flex min-h-[520px] items-start">
        <AutoOpen>
            <EnemyLocationFilter tree={TREE} selected={["main_0", "level_main_01-03"]} onChange={noop} />
        </AutoOpen>
    </div>
);
