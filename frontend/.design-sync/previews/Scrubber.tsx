import { Scrubber } from "frontend";
import { type ReactNode, useEffect, useRef } from "react";

// The progress hairline along the stage's top edge: a 3 px track with the read
// share in brand red. Hovering it thickens the track, shows a knob at the
// current line and a tooltip naming the line under the pointer ("Line n of N",
// the speaker in their hue, the first words); a click jumps there. It is
// `absolute` on the stage, so each story sits it over a real 15-8 scene.

const BG = "https://api.myrtle.moe/api/assets/textures/avg/bg/avg_bkg_h1_bg_in_0/bg_infirmary.png";
const noop = () => {};
type Summary = Parameters<typeof Scrubber>[0]["summaries"][number];

const LINES: [string | undefined, string][] = [
    ["???", "......"],
    ["???", "Doctor..."],
    ["Amiya", "Doctor! I'm so relieved..."],
    ["Amiya", "You're finally awake."],
    ["Amiya", "The evacuation hasn't been smooth. Our neural link was abruptly severed..."],
    ["Amiya", "I just woke up myself."],
    ["Amiya", "Don't worry about me. I'm doing much better, thanks to Theresa's protection."],
    ["Amiya", "You should get a full check-up from Dr. Kal'tsit."],
    ["Approaching Voice", "I started preparing cognition baseline tests as soon as I confirmed..."],
    ["Kal'tsit", "Diagnosing thought isn't quite as simple as diagnosing biological tissue..."],
    ["Kal'tsit", "Welcome back, Doctor."],
    [undefined, "You see your reflection in the window, almost as though you were still standing with Kal'tsit."],
];
const SUMMARIES: Summary[] = LINES.map(([speaker, preview], haltIndex) => ({ haltIndex, kind: "line", speaker, preview }) as Summary);

function StageBox({ children, hoverAt }: { children: ReactNode; hoverAt?: number }) {
    const ref = useRef<HTMLDivElement>(null);
    useEffect(() => {
        if (hoverAt === undefined) return;
        // The tooltip follows the pointer; fire one move over the track after
        // two frames, at `hoverAt` of its width.
        let f2 = 0;
        const f1 = requestAnimationFrame(() => {
            f2 = requestAnimationFrame(() => {
                const el = ref.current?.querySelector<HTMLElement>("[data-story-scrubber]");
                if (!el) return;
                const r = el.getBoundingClientRect();
                el.dispatchEvent(new PointerEvent("pointermove", { bubbles: true, clientX: r.left + r.width * hoverAt, clientY: r.top + 2, pointerType: "mouse" }));
            });
        });
        return () => {
            cancelAnimationFrame(f1);
            cancelAnimationFrame(f2);
        };
    }, [hoverAt]);
    return (
        <div ref={ref} className="relative aspect-video w-full overflow-hidden rounded-lg bg-black">
            <img src={BG} alt="" className="absolute inset-0 size-full object-cover" />
            {children}
        </div>
    );
}

const bar = (progress: number) => <Scrubber shown summaries={SUMMARIES} totalHalts={LINES.length} progress={progress} barRect={() => undefined} onPointerEnter={noop} onPointerLeave={noop} onJump={noop} />;

// At rest a third of the way through: only the hairline.
export const Resting = () => <StageBox>{bar(0.33)}</StageBox>;

// Hovering ahead of the current line: the tooltip names Kal'tsit's line.
export const HoverTooltip = () => <StageBox hoverAt={0.8}>{bar(0.33)}</StageBox>;

// Near the end of the story.
export const NearEnd = () => <StageBox>{bar(0.92)}</StageBox>;
