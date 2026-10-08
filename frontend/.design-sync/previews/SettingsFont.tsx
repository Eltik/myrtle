import { SettingsFont } from "frontend";
import type { ReactNode } from "react";
import { DEFAULT_SETTINGS } from "../../src/lib/story/settings";

// The Font row of the reader's Settings sheet: the face picker (each option
// drawn in its own face), a one-line note on the choice, and a sample at the
// reading size. The three Terra scripts keep the interface face for their name
// and show a glyph sample beside it, plus a credit line. "Custom" adds the
// upload row. `open={false}` skips the IndexedDB read of a stored face.

const noop = () => {};
type Settings = typeof DEFAULT_SETTINGS;

function Sheet({ children }: { children: ReactNode }) {
    return <div className="flex w-full max-w-xl flex-col gap-4 rounded-2xl border bg-popover p-6 text-popover-foreground shadow-lg">{children}</div>;
}

const Font = (patch: Partial<Settings>) => (
    <Sheet>
        <SettingsFont open={false} settings={{ ...DEFAULT_SETTINGS, ...patch }} onChange={noop} apply={noop} />
    </Sheet>
);

// The default: the reading style's own serif.
export const Preset = () => Font({});

// OpenDyslexic at a 130% reading size, the accessibility face.
export const Dyslexic = () => Font({ font: "dyslexic", textSize: 130 });

// A Terra script: name in the interface face, the glyph sample and the credit.
export const TerraSarkaz = () => Font({ font: "terraSarkaz" });

// Custom with a stored file name: the upload row and Remove appear.
export const CustomUploaded = () => Font({ font: "custom", customFontName: "Noto Serif Display.ttf" });
