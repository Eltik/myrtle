import type { IOptimizerDef } from "./optimizers";

export function ComingSoon({ def }: { def: IOptimizerDef }) {
    return (
        <div className="rounded-xl border border-border border-dashed p-10 text-center">
            <p className="font-medium text-[13px] text-foreground">{def.label} is not built yet.</p>
            <p className="mx-auto mt-1 max-w-md text-[12.5px] text-muted-foreground">{def.blurb}</p>
        </div>
    );
}
