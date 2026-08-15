import { DownloadButton } from "frontend";
import { type ReactNode, useEffect, useRef } from "react";

// The export control in the chibi viewer's toolbar (ported from
// ChibiViewer.tsx, which renders it beside the view/animation selects). It
// turns into an inline progress strip while a clip is being encoded, so the
// stories sweep that state machine: ready -> settings open -> recording ->
// disabled.
//
// `animationBounds` is a measured Spine local-bounds box; the resolution
// labels in the settings popover are derived from it.
const MLYNAR_IDLE_BOUNDS = { x: -134, y: -281, width: 268, height: 302 };

const noop = () => {};

// The settings popover owns its `open` state internally, so the story clicks
// the trigger on mount. Base UI wires the trigger after the first paint, so the
// click has to wait two frames.
const AutoOpen = ({ children }: { children: ReactNode }) => {
    const ref = useRef<HTMLDivElement>(null);
    useEffect(() => {
        let inner = 0;
        const outer = requestAnimationFrame(() => {
            inner = requestAnimationFrame(() => ref.current?.querySelector("button")?.click());
        });
        return () => {
            cancelAnimationFrame(outer);
            cancelAnimationFrame(inner);
        };
    }, []);
    return (
        <div className="relative min-h-[520px] w-full" ref={ref}>
            {children}
        </div>
    );
};

// The toolbar strip the button actually lives in.
const Toolbar = ({ children }: { children: ReactNode }) => (
    <div className="w-fit rounded-lg border border-border bg-card/30 p-3">
        <h4 className="mb-2 font-medium text-foreground">Chibi Preview</h4>
        <div className="flex flex-wrap items-center gap-2">{children}</div>
    </div>
);

export const Ready = () => (
    <Toolbar>
        <DownloadButton animationBounds={MLYNAR_IDLE_BOUNDS} disabled={false} isRecording={false} onCancel={noop} onDownload={noop} progress={0} />
    </Toolbar>
);

export const Disabled = () => (
    <Toolbar>
        <DownloadButton animationBounds={null} disabled isRecording={false} onCancel={noop} onDownload={noop} progress={0} />
    </Toolbar>
);

export const Recording = () => (
    <Toolbar>
        <DownloadButton animationBounds={MLYNAR_IDLE_BOUNDS} disabled={false} isRecording onCancel={noop} onDownload={noop} progress={62} />
    </Toolbar>
);

export const ExportSettings = () => (
    <AutoOpen>
        <Toolbar>
            <DownloadButton animationBounds={MLYNAR_IDLE_BOUNDS} disabled={false} isRecording={false} onCancel={noop} onDownload={noop} progress={0} />
        </Toolbar>
    </AutoOpen>
);
