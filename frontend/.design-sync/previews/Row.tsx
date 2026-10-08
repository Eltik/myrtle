import { Input, Row, Slider, Switch } from "frontend";
import type { ReactNode } from "react";

// One row of the reader's Settings sheet: a 10rem label column, the control,
// and a right-aligned mono value column; a hint, when given, runs full width
// underneath. Under 640 px the label and value share a line above a full-width
// control. Each story sits the rows in the sheet's own panel.

function Sheet({ children }: { children: ReactNode }) {
    return <div className="flex w-full max-w-xl flex-col gap-4 rounded-2xl border bg-popover p-6 text-popover-foreground shadow-lg">{children}</div>;
}

// Slider rows with their printed values, the commonest row in the sheet.
export const SliderRows = () => (
    <Sheet>
        <Row label="Text speed" value="45 chars/s">
            <Slider min={10} max={120} step={5} defaultValue={[45]} />
        </Row>
        <Row label="Text size" value="115%">
            <Slider min={70} max={200} step={5} defaultValue={[115]} />
        </Row>
        <Row label="Line height" value="1.65">
            <Slider min={1.2} max={2.2} step={0.05} defaultValue={[1.65]} />
        </Row>
    </Sheet>
);

// A switch with a hint: no value column, the hint wraps under the row.
export const SwitchWithHint = () => (
    <Sheet>
        <Row label="Light text box" hint="Only the reading surface changes; the site theme stays as you set it.">
            <Switch defaultChecked />
        </Row>
        <Row label="Letterbox like the game" hint="The game draws every scene into a 16:9 box and fills the rest of the screen with black. Off, the reader continues the background across it, blurred and dimmed.">
            <Switch />
        </Row>
    </Sheet>
);

// A value and a hint together: the value stays on the row, the hint drops below.
export const ValueAndHint = () => (
    <Sheet>
        <Row label="Playback speed" value="1.5×" hint="Multiplies every fade, tween and hold the script asks for. 0 plays each step instantly.">
            <Slider min={0} max={3} step={0.1} defaultValue={[1.5]} />
        </Row>
        <Row label="Doctor's name" hint="Used wherever a story addresses you by name. Up to 24 characters; an empty field reads as Doctor.">
            <Input placeholder="Doctor" defaultValue="" />
        </Row>
    </Sheet>
);
