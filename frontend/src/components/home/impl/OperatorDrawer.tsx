import { Badge } from "#/components/ui/badge";
import { Button } from "#/components/ui/button";
import { Kicker } from "#/components/ui/kicker";
import { OperatorAvatar } from "#/components/ui/operator-avatar";
import { Sheet, SheetContent } from "#/components/ui/sheet";
import { ROLE_DRAWER_GRADIENT, type Role } from "#/lib/role-styles";
import type { Operator } from "./data";

export default function OperatorDrawer({ op, onClose }: { op: Operator; onClose: () => void }) {
    return (
        <Sheet open onOpenChange={(open) => !open && onClose()}>
            <SheetContent className="flex flex-col gap-4 p-6">
                <Kicker>Operator</Kicker>
                <div className="flex h-[120px] w-full flex-col items-center justify-center gap-2 rounded-xl" style={{ background: ROLE_DRAWER_GRADIENT[op.role as Role] }}>
                    <span className="inline-flex h-[84px] w-[84px] items-center justify-center overflow-hidden rounded-2xl border border-border/70 bg-card/55 font-sans text-[32px] font-bold tracking-tight leading-none text-foreground">
                        <OperatorAvatar charId={op.id} name={op.name} />
                    </span>
                    <span className="font-sans text-sm font-medium leading-none text-primary tracking-wider">{"★".repeat(op.rarity)}</span>
                </div>
                <h2 className="m-0 font-sans text-[22px] font-bold leading-[1.1] tracking-tight text-foreground">{op.name}</h2>
                <div className="flex flex-wrap gap-1.5">
                    <Badge>{op.role}</Badge>
                    <Badge variant="secondary">{op.arch}</Badge>
                    {op.owned ? (
                        <Badge variant="success">
                            E{op.e} · Lv{op.lvl}
                        </Badge>
                    ) : (
                        <Badge variant="outline">Not owned</Badge>
                    )}
                </div>
                <div className="grid grid-cols-4 gap-2.5">
                    {[
                        { k: "HP", v: "2,020" },
                        { k: "ATK", v: "510" },
                        { k: "DEF", v: "225" },
                        { k: "RES", v: "0" },
                    ].map((s) => (
                        <div key={s.k} className="flex flex-col gap-1 rounded-lg border border-border bg-muted p-2.5">
                            <div className="font-mono text-[10px] font-medium uppercase leading-none tracking-widest text-muted-foreground">{s.k}</div>
                            <div className="font-sans text-lg font-bold leading-none tracking-tight text-foreground">{s.v}</div>
                        </div>
                    ))}
                </div>
                <div className="flex gap-2.5">
                    <Button variant="default">View skills</Button>
                    <Button variant="outline">Add to team</Button>
                </div>
            </SheetContent>
        </Sheet>
    );
}
