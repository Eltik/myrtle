import { TierSettingsDialog } from "frontend";
import type { ReactNode } from "react";

/** `IEditTier` — operator ids verified against https://api.myrtle.moe/api/operators/index. */
const S_TIER = {
    id: "tier-s",
    name: "S",
    color: "#dc4d56",
    description: "Warps a map on its own. An operator only lands here if a squad without them has to change plan, not just play slower.",
    operatorIds: ["char_1035_wisdel", "char_4064_mlynar", "char_1028_texas2", "char_4087_ines", "char_350_surtr", "char_4116_blkkgt", "char_2012_typhon"],
};

const NEW_TIER = { id: "tier-d", name: "D", color: "#4f9d69", description: "", operatorIds: [] };

const ONLY_TIER = { id: "tier-1", name: "Pick-One", color: "#8b6ad6", description: "One slot, six candidates. Ranked for CC#12 Daybreak risk 18.", operatorIds: [] };

const noop = () => {};

/** Full-viewport stage: the popup is `position: fixed` against the story root, and this dialog is tall enough to be cropped by a shorter one. */
const Stage = ({ children }: { children: ReactNode }) => <div className="min-h-dvh">{children}</div>;

export const PopulatedTier = () => (
    <Stage>
        <TierSettingsDialog tier={S_TIER} canDelete onClose={noop} onSave={noop} onDelete={noop} onClear={noop} />
    </Stage>
);

export const EmptyTier = () => (
    <Stage>
        <TierSettingsDialog tier={NEW_TIER} canDelete onClose={noop} onSave={noop} onDelete={noop} onClear={noop} />
    </Stage>
);

export const LastRemainingTier = () => (
    <Stage>
        <TierSettingsDialog tier={ONLY_TIER} canDelete={false} onClose={noop} onSave={noop} onDelete={noop} onClear={noop} />
    </Stage>
);
