import { Avatar, AvatarFallback, AvatarImage, Card } from "frontend";
import { UserIcon } from "lucide-react";

const avatar = (id: string) => `https://api.myrtle.moe/api/avatar/${id}`;

export const Initials = () => (
    <div className="flex items-center gap-3">
        {[
            { initials: "VE", label: "Dr. Vector" },
            { initials: "KA", label: "Dr. Kal" },
            { initials: "SI", label: "Dr. Silence" },
            { initials: "RH", label: "Dr. Rhine" },
        ].map((doctor) => (
            <div className="flex flex-col items-center gap-1.5" key={doctor.initials}>
                <Avatar className="size-10">
                    <AvatarFallback>{doctor.initials}</AvatarFallback>
                </Avatar>
                <span className="text-[10px] text-muted-foreground">{doctor.label}</span>
            </div>
        ))}
    </div>
);

export const IconFallback = () => (
    <div className="flex items-center gap-3">
        <Avatar className="size-10">
            <AvatarFallback>
                <UserIcon className="size-4 text-muted-foreground" />
            </AvatarFallback>
        </Avatar>
        <div>
            <p className="font-medium text-foreground text-sm">Anonymous doctor</p>
            <p className="text-muted-foreground text-xs">No profile synced yet</p>
        </div>
    </div>
);

export const Shapes = () => (
    <div className="flex items-center gap-4">
        <Avatar className="size-12">
            <AvatarFallback className="font-semibold text-sm">TX</AvatarFallback>
        </Avatar>
        <Avatar className="size-12 rounded-xl border border-border">
            <AvatarFallback className="rounded-xl font-semibold text-sm">SK</AvatarFallback>
        </Avatar>
        <Avatar className="size-12 rounded-none">
            <AvatarFallback className="rounded-none font-semibold text-sm">ML</AvatarFallback>
        </Avatar>
        <Avatar className="size-12">
            <AvatarFallback className="bg-primary font-semibold text-primary-foreground text-sm">MU</AvatarFallback>
        </Avatar>
    </div>
);

export const MixedRoster = () => (
    <Card className="max-w-sm">
        <div className="flex flex-col gap-1 p-3">
            {[
                { id: "char_4064_mlynar", initials: "ML", name: "Mlynar", owned: true },
                { id: "char_263_skadi", initials: "SK", name: "Skadi", owned: true },
                { id: null, initials: "VG", name: "Virtuosa", owned: false },
                { id: null, initials: "CE", name: "Cement", owned: false },
            ].map((op) => (
                <div className="flex items-center gap-3 rounded-md px-2 py-1.5" key={op.name}>
                    <Avatar className="size-8">
                        {op.id ? <AvatarImage alt={op.name} src={avatar(op.id)} /> : null}
                        <AvatarFallback className="text-[10px]">{op.initials}</AvatarFallback>
                    </Avatar>
                    <span className="flex-1 font-medium text-foreground text-sm">{op.name}</span>
                    <span className="font-mono text-[11px] text-muted-foreground">{op.owned ? "E2 90" : "not owned"}</span>
                </div>
            ))}
        </div>
    </Card>
);
