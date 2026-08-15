import { PublishingPanel } from "frontend";
import { type ReactNode, useEffect, useRef } from "react";

/** `ITierListFlair[]` — codes match the app's own FLAIR_ACCENT map. */
const FLAIRS = [
    { id: 1, code: "meta", label: "Meta", color: "#dc4d56", displayOrder: 0, isActive: true },
    { id: 2, code: "endgame", label: "Endgame", color: "#e0603c", displayOrder: 1, isActive: true },
    { id: 3, code: "roguelike", label: "Integrated Strategies", color: "#8b6ad6", displayOrder: 2, isActive: true },
    { id: 4, code: "event", label: "Contingency Contract", color: "#7a5cd0", displayOrder: 3, isActive: true },
    { id: 5, code: "beginner", label: "Newcomer", color: "#4f9d69", displayOrder: 4, isActive: true },
    { id: 6, code: "niche", label: "Niche picks", color: "#c9a227", displayOrder: 5, isActive: true },
];

const META = { id: 1, label: "Meta", color: "#dc4d56" };

const noop = () => {};

/** The editor's right-hand rail is a 320px column. */
const Rail = ({ children }: { children: ReactNode }) => <div className="w-80">{children}</div>;

/** Opens an overlay whose trigger is internal — deferred two frames so Base UI has wired the button. */
const ClickOnMount = ({ children }: { children: ReactNode }) => {
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
        <div ref={ref} className="relative min-h-[520px] w-80">
            {children}
        </div>
    );
};

export const Listed = () => (
    <Rail>
        <PublishingPanel flair={META} flairOptions={FLAIRS} isListed canPublish publishingDisabledReason={null} settingFlair={false} settingVisibility={false} onSetFlair={noop} onSetVisibility={noop} onOpenPublishDialog={noop} />
    </Rail>
);

export const HiddenFromBrowse = () => (
    <Rail>
        <PublishingPanel
            flair={null}
            flairOptions={FLAIRS}
            isListed={false}
            canPublish
            publishingDisabledReason={null}
            settingFlair={false}
            settingVisibility={false}
            onSetFlair={noop}
            onSetVisibility={noop}
            onOpenPublishDialog={noop}
        />
    </Rail>
);

export const BlockedByUnsavedChanges = () => (
    <Rail>
        <PublishingPanel
            flair={{ id: 3, label: "Integrated Strategies", color: "#8b6ad6" }}
            flairOptions={FLAIRS}
            isListed
            canPublish={false}
            publishingDisabledReason="Save your changes before publishing a version."
            settingFlair={false}
            settingVisibility={false}
            onSetFlair={noop}
            onSetVisibility={noop}
            onOpenPublishDialog={noop}
        />
    </Rail>
);

export const FlairMenuOpen = () => (
    <ClickOnMount>
        <PublishingPanel flair={META} flairOptions={FLAIRS} isListed canPublish publishingDisabledReason={null} settingFlair={false} settingVisibility={false} onSetFlair={noop} onSetVisibility={noop} onOpenPublishDialog={noop} />
    </ClickOnMount>
);
