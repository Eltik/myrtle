import type { IImprovementsResponse } from "#/lib/api/user";
import { EmptyHint, PANEL_PADDING, ProgressLine, SectionHeader } from "./shared";

interface IProps {
    improvements: IImprovementsResponse;
    accent: string;
}

export function SandboxPanel({ improvements, accent }: IProps) {
    const s = improvements.sandbox;
    const hasAnyData = s.achievements.current > 0 || s.nodes.current > 0 || s.tech.current > 0 || s.quests.current > 0;

    return (
        <div className={`${PANEL_PADDING} flex flex-col gap-3`}>
            <SectionHeader title="Reclamation Algorithm" accent={accent} />
            {!hasAnyData && <EmptyHint>No RA progress detected. Start RA in Operation Originium Dust to begin tracking.</EmptyHint>}
            <div className="grid gap-2.5 sm:grid-cols-2">
                <ProgressLine label="Achievements" current={s.achievements.current} max={s.achievements.max} accent={accent} />
                <ProgressLine label="Map nodes" current={s.nodes.current} max={s.nodes.max} accent={accent} />
                <ProgressLine label="Tech unlocks" current={s.tech.current} max={s.tech.max} accent={accent} />
                <ProgressLine label="Story Acts" current={s.quests.current} max={s.quests.max} accent={accent} />
            </div>
        </div>
    );
}
