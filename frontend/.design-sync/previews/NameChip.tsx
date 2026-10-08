import { NameChip } from "frontend";
import { type ReactNode, useEffect, useRef } from "react";

// One name under "Named in scripts" on a story character's sheet: a chip with
// the name's line count that opens what the census knows about it (count and
// share, the stories, example lines) in a popover. `primary` is the folder's
// main name; `stray` is a cut name in the muted dashed form. Data is the live
// `/api/story/sprites/avg_1032_excu2_1` response (Executor the Ex Foedere's
// sprite, named "Federico" in most scripts).

const FEDERICO = {"name":"Federico","count":1097.0,"stories":[{"id":"act29side_level_act29side_st02","name":"Intermezzo 'Unfinished Fugue'","code":"ZT-ST-2","tag":"Interlude","lines":42.0},{"id":"act26side_level_act26side_03_beg","name":"No Empty Hands","code":"HE-3","tag":"Before Operation","lines":39.0},{"id":"act26side_level_act26side_05_beg","name":"God in His Temple","code":"HE-5","tag":"Before Operation","lines":39.0},{"id":"act29side_level_act29side_01_end","name":"Chorale 'Lied des klaren Himmels'","code":"ZT-1","tag":"After Operation","lines":39.0},{"id":"act26side_level_act26side_06_beg","name":"Serenade","code":"HE-6","tag":"Before Operation","lines":36.0}],"moreStories":47,"examples":[{"storyId":"act26side_level_act26side_st01","storyName":"Crown Offering","code":"HE-ST-1","tag":"Interlude","line":174,"text":"Alright."},{"storyId":"act26side_level_act26side_01_beg","storyName":"Home Sweet Home","code":"HE-1","tag":"Before Operation","line":187,"text":"See for yourself."},{"storyId":"act26side_level_act26side_01_end","storyName":"Home Sweet Home","code":"HE-1","tag":"After Operation","line":174,"text":"I did not."}]};
const EXECUTOR = {"name":"Executor","count":116.0,"stories":[{"id":"story_excu2_set_1_story_1","name":"Azure","tag":"Interlude","lines":116.0}],"moreStories":0,"examples":[{"storyId":"story_excu2_set_1_story_1","storyName":"Azure","tag":"Interlude","line":12,"text":"...Target has been placed into custody and the lost medical supplies retrieved. One operator suffered a mild ankle sprain during the operation."},{"storyId":"story_excu2_set_1_story_1","storyName":"Azure","tag":"Interlude","line":13,"text":"That is all I have to report about the mission. I have been asked to communicate my superiors' gratitude for Rhodes Island's support of Lateran citizens abroad."},{"storyId":"story_excu2_set_1_story_1","storyName":"Azure","tag":"Interlude","line":14,"text":"The Notarial Hall would be honored to welcome you while Rhodes Island is stationed near Laterano."}]};
const STRAY = {"name":"Arturia","count":20.0,"stories":[{"id":"act29side_level_act29side_04_end","name":"Rhapsody 'Longing' ","code":"ZT-4","tag":"After Operation","lines":19.0},{"id":"act29side_level_act29side_10_end","name":"Messe 'König'","code":"ZT-10","tag":"After Operation","lines":1.0}],"moreStories":0,"examples":[{"storyId":"act29side_level_act29side_04_end","storyName":"Rhapsody 'Longing' ","code":"ZT-4","tag":"After Operation","line":35,"text":"If I told you... I never cared who they were, and had no interest in Leithanien's political turmoil, would you believe me?"},{"storyId":"act29side_level_act29side_10_end","storyName":"Messe 'König'","code":"ZT-10","tag":"After Operation","line":410,"text":"No."},{"storyId":"act29side_level_act29side_04_end","storyName":"Rhapsody 'Longing' ","code":"ZT-4","tag":"After Operation","line":39,"text":"Oh, Federico. You really do know me better than anyone. Everyone else's minds are tainted with thoughts, while you... boring as you are, you and I are alike."}]};
/** The folder's named lines: the share's denominator. */
const TOTAL = 1278.5;

/** Clicks the first button inside two frames after mount (an effect alone is dropped), then blurs it so the trigger shows no focus ring. */
const OpenOnMount = ({ children }: { children: ReactNode }) => {
    const ref = useRef<HTMLDivElement>(null);
    useEffect(() => {
        const ids: ReturnType<typeof setTimeout>[] = [];
        requestAnimationFrame(() =>
            requestAnimationFrame(() => {
                ref.current?.querySelector("button")?.click();
                for (const ms of [60, 180, 400]) ids.push(setTimeout(() => (document.activeElement as HTMLElement | null)?.blur(), ms));
            }),
        );
        return () => ids.forEach(clearTimeout);
    }, []);
    return (
        <div ref={ref} className="min-h-dvh p-4">
            {children}
        </div>
    );
};

const Row = ({ children }: { children: ReactNode }) => <div className="flex flex-wrap items-baseline gap-1.5 p-4">{children}</div>;

/** The folder's main name and an alias, side by side as the sheet lists them. */
export const PrimaryAndAlias = () => (
    <Row>
        <NameChip name="Federico" count={FEDERICO.count} primary detail={FEDERICO} total={TOTAL} owner="Federico" />
        <NameChip name="Executor" count={EXECUTOR.count} primary={false} detail={EXECUTOR} total={TOTAL} owner="Federico" />
    </Row>
);

/** A stray name: the muted dashed chip the stray list shows. */
export const Stray = () => (
    <Row>
        <NameChip name={STRAY.name} count={STRAY.count} primary={false} stray detail={STRAY} total={TOTAL} owner="Federico" />
    </Row>
);

/** The main name's popover open: count and share, top stories with "+N more", example lines. */
export const DetailOpen = () => (
    <OpenOnMount>
        <NameChip name="Federico" count={FEDERICO.count} primary detail={FEDERICO} total={TOTAL} owner="Federico" />
    </OpenOnMount>
);
