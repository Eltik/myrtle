import { PillButton } from "frontend";
import { ArrowLeftIcon, ChevronLeftIcon, ChevronRightIcon, EyeOffIcon, FastForwardIcon, HistoryIcon, MaximizeIcon, PlayIcon, SettingsIcon } from "lucide-react";
import type { ReactNode } from "react";

// One control in the reader's frosted pills. Under 1024 px (which includes this
// capture) it is icon-only and its label moves into a tooltip; at 1024 and up
// it carries the label beside the icon. The pill glass and the dark scene
// behind it are the parent's, so each story rebuilds that frame.

const BG = "https://api.myrtle.moe/api/assets/textures/avg/bg/avg_bkg_h1_bg_ce_0/bg_ceo.png";
const noop = () => {};

function Scene({ children }: { children: ReactNode }) {
    return (
        <div className="relative h-48 w-full overflow-hidden rounded-lg bg-black">
            <img src={BG} alt="" className="absolute inset-0 size-full object-cover" />
            <div className="absolute inset-x-2 top-2 flex items-start justify-between gap-1.5">{children}</div>
        </div>
    );
}

function Pill({ children }: { children: ReactNode }) {
    return <div className="flex items-center gap-0.5 rounded-lg bg-black/45 p-1 backdrop-blur-sm">{children}</div>;
}

// The scene pill as the reader builds it: back, settings, log, hide, and the
// previous/next story chevrons.
export const ScenePill = () => (
    <Scene>
        <Pill>
            <PillButton compact label="Library" onClick={noop}>
                <ArrowLeftIcon />
            </PillButton>
            <PillButton compact label="Settings" onClick={noop}>
                <SettingsIcon />
            </PillButton>
            <PillButton compact label="Log" onClick={noop}>
                <HistoryIcon />
            </PillButton>
            <PillButton compact label="Hide" onClick={noop}>
                <EyeOffIcon />
            </PillButton>
            <PillButton compact iconOnly label="Previous story" onClick={noop}>
                <ChevronLeftIcon />
            </PillButton>
            <PillButton compact iconOnly label="Next story" onClick={noop}>
                <ChevronRightIcon />
            </PillButton>
        </Pill>
    </Scene>
);

// The states side by side: idle, active (AUTO running tints the control),
// and disabled (the dead chevron at a chapter's end, at 35% opacity).
export const States = () => (
    <Scene>
        <Pill>
            <PillButton compact label="Auto" state="Off" onClick={noop}>
                <PlayIcon />
            </PillButton>
            <PillButton compact label="Auto" state="On" active onClick={noop}>
                <PlayIcon />
            </PillButton>
            <PillButton compact label="Skip" onClick={noop}>
                <FastForwardIcon />
            </PillButton>
            <PillButton compact label="Skip" disabled onClick={noop}>
                <FastForwardIcon />
            </PillButton>
            <PillButton compact iconOnly label="Next story" disabled onClick={noop}>
                <ChevronRightIcon />
            </PillButton>
        </Pill>
    </Scene>
);

// The wide register (non-compact). The label span is `hidden lg:inline`, so it
// only shows at 1024 px and up; below that it is the same icon with a title.
export const Wide = () => (
    <Scene>
        <span />
        <Pill>
            <PillButton compact={false} label="Auto" state="Off" onClick={noop}>
                <PlayIcon />
            </PillButton>
            <PillButton compact={false} label="Skip" onClick={noop}>
                <FastForwardIcon />
            </PillButton>
            <PillButton compact={false} label="Fullscreen" onClick={noop}>
                <MaximizeIcon />
            </PillButton>
        </Pill>
    </Scene>
);
