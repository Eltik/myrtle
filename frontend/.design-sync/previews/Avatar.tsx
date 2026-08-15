import { Avatar, AvatarFallback, AvatarImage, Badge, Card } from "frontend";

const avatar = (id: string) => `https://api.myrtle.moe/api/avatar/${id}`;

export const Basic = () => (
    <div className="flex items-center gap-3">
        <Avatar className="size-10">
            <AvatarImage alt="Mlynar" src={avatar("char_4064_mlynar")} />
            <AvatarFallback>ML</AvatarFallback>
        </Avatar>
        <div className="min-w-0">
            <p className="font-semibold text-foreground text-sm leading-snug">Mlynar</p>
            <p className="font-mono text-[11px] text-muted-foreground tabular-nums leading-none">E2 90 · Pot 3</p>
        </div>
    </div>
);

export const Sizes = () => (
    <div className="flex items-end gap-4">
        {(["size-6", "size-8", "size-10", "size-14"] as const).map((size, i) => (
            <div className="flex flex-col items-center gap-1.5" key={size}>
                <Avatar className={size}>
                    <AvatarImage alt="Operator avatar" src={avatar(["char_002_amiya", "char_102_texas", "char_263_skadi", "char_249_mlyss"][i] as string)} />
                    <AvatarFallback>{["AM", "TX", "SK", "MU"][i]}</AvatarFallback>
                </Avatar>
                <span className="font-mono text-[10px] text-muted-foreground">{size.replace("size-", "")}</span>
            </div>
        ))}
    </div>
);

export const SquadStack = () => (
    <div className="flex items-center gap-3">
        <div className="flex">
            {["char_4064_mlynar", "char_263_skadi", "char_180_amgoat", "char_102_texas", "char_249_mlyss"].map((id, i) => (
                <Avatar className={i === 0 ? "size-9 ring-2 ring-background" : "-ml-1 size-9 ring-2 ring-background"} key={id}>
                    <AvatarImage alt={id} src={avatar(id)} />
                    <AvatarFallback className="text-[10px]">OP</AvatarFallback>
                </Avatar>
            ))}
        </div>
        <span className="text-muted-foreground text-sm">+7 more in this squad</span>
    </div>
);

export const InUserCard = () => (
    <Card className="max-w-sm">
        <div className="flex items-center gap-3.5 px-4 py-3.5">
            <Avatar className="size-14 shrink-0 rounded-xl border border-border">
                <AvatarImage alt="Dr. Vector" src={avatar("char_1028_texas2")} />
                <AvatarFallback className="rounded-xl text-sm">VE</AvatarFallback>
            </Avatar>
            <div className="min-w-0 flex-1">
                <div className="flex items-center gap-1.5">
                    <span className="truncate font-semibold text-[14px] text-foreground leading-snug">Dr. Vector</span>
                    <Badge className="font-mono" size="sm" variant="outline">
                        A2
                    </Badge>
                </div>
                <div className="mt-1 flex flex-wrap items-center gap-x-2 gap-y-0.5">
                    <span className="font-mono text-[11px] text-muted-foreground tabular-nums leading-none">10000241</span>
                    <Badge className="font-mono uppercase" size="sm" variant="secondary">
                        EN
                    </Badge>
                </div>
            </div>
        </div>
    </Card>
);
