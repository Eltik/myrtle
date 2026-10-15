import { TextBox } from "frontend";
import type { ReactNode } from "react";
import { DEFAULT_SETTINGS } from "../../src/lib/story/settings";

// The dialogue box: a frosted panel at the foot of the stage, the speaker's
// name on a plate over its top-left edge (a stable hue per name), the line in
// the reading face, and a bouncing chevron once the line is fully typed.
// `completeSignal={1}` jumps each reveal to the end so the capture shows whole
// lines. It positions itself absolutely against the stage, and its margins
// come from `reader.css` under `[data-story-reader]`, so each story builds that
// stage over a real 15-8 background.

const BG = "https://api.myrtle.moe/api/assets/textures/avg/bg/avg_bkg_h1_bg_in_0/bg_infirmary.png";
const noop = () => {};
type Settings = typeof DEFAULT_SETTINGS;

function StageBox({ children }: { children: ReactNode }) {
    return (
        <div data-story-reader className="relative aspect-video w-full overflow-hidden rounded-lg bg-black [container-type:size]">
            <img src={BG} alt="" className="absolute inset-0 size-full object-cover" />
            {children}
        </div>
    );
}

const Box = ({ speaker, text, narration = false, settings = {} }: { speaker?: string; text: string; narration?: boolean; settings?: Partial<Settings> }) => (
    <StageBox>
        <TextBox
            speaker={speaker}
            text={text}
            isNarration={narration}
            revealKey={1}
            armed
            settings={{ ...DEFAULT_SETTINGS, ...settings }}
            onRevealDone={noop}
            completeSignal={1}
            continueLabel="Continue"
            hidden={false}
            customFontLoaded={false}
            onPositionChange={noop}
            dragLabel="Move the text box"
        />
    </StageBox>
);

// A spoken line with its speaker plate.
export const Dialogue = () => <Box speaker="Amiya" text="Doctor! I'm so relieved... You're finally awake." />;

// A long line: the box grows to fit, capped at 40% of the stage.
export const LongLine = () => <Box speaker="Kal'tsit" text="Diagnosing thought isn't quite as simple as diagnosing biological tissue, though. Further analysis will be required. Initial testing showed no obvious signs of damage for either of you." />;

// Narration: no plate, the line in italics.
export const Narration = () => <Box narration text="You see your reflection in the window, almost as though you were still standing with Kal'tsit." />;

// The light reading surface: a paper box with dark text and the darkened hue.
export const LightBox = () => <Box speaker="Amiya" text="Don't go off on your own, Doctor. We need to stay in the emergency observation area that Dr. Kal'tsit set up." settings={{ lightBox: true }} />;

// The box moved to the top centre preset (Settings, Box position).
export const RaisedToTop = () => <Box speaker="Kal'tsit" text="Welcome back, Doctor." settings={{ boxX: 0, boxY: 1 }} />;
