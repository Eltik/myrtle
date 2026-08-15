import { EmptyHint, SectionHeader } from "frontend";
import type { ReactNode } from "react";

const STAGE = "oklch(0.62 0.20 255)";
const MEDAL = "oklch(0.62 0.22 295)";
const SANDBOX = "oklch(0.70 0.14 200)";

const Panel = ({ children }: { children: ReactNode }) => <div className="flex w-full max-w-xl flex-col gap-5 rounded-xl border border-border bg-card px-4 pt-4 pb-4 sm:px-5 sm:pb-5">{children}</div>;

const Kicker = ({ children }: { children: ReactNode }) => <span className="font-mono font-semibold text-[10.5px] text-muted-foreground uppercase tracking-[0.12em]">{children}</span>;

export const NothingLeftToDo = () => (
    <Panel>
        <div className="flex flex-col gap-3">
            <SectionHeader accent={MEDAL} count="0 missing" title="Medals" />
            <EmptyHint>You've earned every medal that's currently reachable. Nice work.</EmptyHint>
        </div>
    </Panel>
);

export const NoDataYet = () => (
    <Panel>
        <div className="flex flex-col gap-3">
            <SectionHeader accent={SANDBOX} title="Reclamation Algorithm" />
            <EmptyHint>No RA progress detected. Start RA in Operation Originium Dust to begin tracking.</EmptyHint>
        </div>
    </Panel>
);

export const InsideStageLists = () => (
    <Panel>
        <div className="flex flex-col gap-3">
            <SectionHeader accent={STAGE} count="1,431 / 1,431 3★" title="Permanent" />
            <div className="flex flex-col gap-1.5">
                <Kicker>Missing</Kicker>
                <EmptyHint>All stages cleared.</EmptyHint>
            </div>
            <div className="flex flex-col gap-1.5">
                <Kicker>Cleared, not 3★</Kicker>
                <EmptyHint>Every clear is 3★.</EmptyHint>
            </div>
            <div className="flex flex-col gap-1.5">
                <Kicker>Annihilation</Kicker>
                <EmptyHint>Every currently-playable Annihilation map is maxed.</EmptyHint>
            </div>
        </div>
    </Panel>
);
