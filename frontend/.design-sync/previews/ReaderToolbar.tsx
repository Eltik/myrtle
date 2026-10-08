import { ReaderToolbar } from "frontend";
import type { ComponentProps, ReactNode } from "react";

// The two frosted pills over the stage: scene controls on the left (library,
// settings, log, hide, previous/next story, collapse) and playback on the
// right (auto, speed, skip, fullscreen, mute, collapse). The toolbar is
// `absolute` over the stage, so each story sits it in the reader's 16:9 box
// over a real scene. Under 1024 px (this capture) the pills are icon-only.

const BG = "https://api.myrtle.moe/api/assets/textures/avg/bg/avg_bkg_h1_bg_ce_0/bg_ceo.png";
const noop = () => {};

type Props = ComponentProps<typeof ReaderToolbar>;

const base: Props = {
    compact: true,
    shown: true,
    barRef: null as unknown as Props["barRef"],
    onPointerEnter: noop,
    onPointerLeave: noop,
    // The route hands the toolbar its own link; an anchor is the same element.
    backLink: <a href="/stories" aria-label="Library" />,
    hasPrevious: true,
    hasNext: true,
    onPrevious: noop,
    onNext: noop,
    onSettings: noop,
    onLog: noop,
    onExport: noop,
    theater: false,
    onTheater: noop,
    onHideToolbar: noop,
    autoPlay: false,
    onAutoPlay: noop,
    animateRatio: 1,
    onSpeed: noop,
    skipping: false,
    onSkip: noop,
    fullscreen: false,
    onFullscreen: noop,
    muted: false,
    onMute: noop,
    musicVolume: 0.7,
    sfxVolume: 0.85,
    onVolume: noop,
    volumeOpen: false,
    onVolumeOpenChange: noop,
};

function StageBox({ children }: { children: ReactNode }) {
    return (
        <div className="relative aspect-video w-full overflow-hidden rounded-lg bg-black">
            <img src={BG} alt="" className="absolute inset-0 size-full object-cover" />
            {children}
        </div>
    );
}

// Mid-story at rest: every control idle, both chapter neighbours available.
export const Default = () => (
    <StageBox>
        <ReaderToolbar {...base} />
    </StageBox>
);

// Auto-play running, muted and in fullscreen: the active controls tint white.
export const AutoPlayingMuted = () => (
    <StageBox>
        <ReaderToolbar {...base} autoPlay muted fullscreen />
    </StageBox>
);

// The chapter's last story on its end card: Next story is dead (kept in place
// so the pill does not change width) and there is nothing left to skip.
export const ChapterEnd = () => (
    <StageBox>
        <ReaderToolbar {...base} hasNext={false} skipDisabled />
    </StageBox>
);

// The wide register: at 1024 px and up the pills carry their labels and the
// 1x/2x speed toggle appears. Below 1024 this renders like Default.
export const Wide = () => (
    <StageBox>
        <ReaderToolbar {...base} compact={false} animateRatio={2} />
    </StageBox>
);

