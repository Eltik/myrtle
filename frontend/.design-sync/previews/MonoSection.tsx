import { MonoSection } from "frontend";

export function NoteFieldHeadings() {
    return (
        <div className="flex max-w-xl flex-col gap-4">
            <div className="flex flex-col gap-1.5">
                <MonoSection>Summary</MonoSection>
                <p className="text-[13px] leading-[1.55]">Premier Arts damage dealer — his S3 ignores DEF entirely, so he scales with ASPD buffs rather than ATK.</p>
            </div>
            <div className="flex flex-col gap-1.5">
                <MonoSection>Preview</MonoSection>
                <p className="text-[13px] text-muted-foreground leading-[1.55]">Rendered exactly as it appears on the public operator page.</p>
            </div>
        </div>
    );
}

export function PanelColumnLabels() {
    return (
        <div className="grid max-w-xl grid-cols-3 gap-4">
            <div className="flex flex-col gap-1.5">
                <MonoSection>Pros</MonoSection>
                <p className="text-[13px] leading-[1.5]">True damage, ignores DEF</p>
            </div>
            <div className="flex flex-col gap-1.5">
                <MonoSection>Cons</MonoSection>
                <p className="text-[13px] leading-[1.5]">Very high DP cost</p>
            </div>
            <div className="flex flex-col gap-1.5">
                <MonoSection>Trivia</MonoSection>
                <p className="text-[13px] leading-[1.5]">Voiced by Yuuichi Nakamura</p>
            </div>
        </div>
    );
}

export function AboveDataBlock() {
    return (
        <div className="max-w-md rounded-2xl border border-border bg-card p-4">
            <MonoSection>Audit · last write</MonoSection>
            <div className="mt-2 font-mono text-[12.5px] tabular-nums">2024-05-15T09:41:22Z</div>
            <div className="mt-1 text-[12.5px] text-muted-foreground">Kyostinv edited `pros` on Mlynar</div>
        </div>
    );
}
