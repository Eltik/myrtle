export function Headline({ label, value, hint }: { label: string; value: string; hint?: string }) {
    return (
        <div className="flex flex-col gap-0.5">
            <span className="text-[10px] text-muted-foreground uppercase tracking-wider">{label}</span>
            <span className="font-mono font-semibold text-[15px] tabular-nums">{value}</span>
            {hint && <span className="text-[10px] text-muted-foreground">{hint}</span>}
        </div>
    );
}
