import { Badge } from "~/components/ui/shadcn/badge";
import { Card, CardContent, CardHeader, CardTitle } from "~/components/ui/shadcn/card";
import { Progress } from "~/components/ui/shadcn/progress";

interface RoomCardProps {
    title: string;
    level: number;
    details: {
        label: string;
        value: string;
    }[];
    efficiency: number;
}

export function RoomCard({ title, level, details, efficiency }: RoomCardProps) {
    return (
        <Card>
            <CardHeader className="pb-2">
                <CardTitle className="flex items-center justify-between text-sm">
                    <span>{title}</span>
                    <Badge variant="outline">Lv.{level}</Badge>
                </CardTitle>
            </CardHeader>
            <CardContent className="space-y-2">
                {details.map((detail) => (
                    <div className="flex items-center justify-between text-sm" key={detail.label}>
                        <span className="text-muted-foreground">{detail.label}</span>
                        <span className="max-w-[120px] truncate font-medium">{detail.value}</span>
                    </div>
                ))}
                <div className="space-y-1">
                    <div className="flex items-center justify-between text-sm">
                        <span className="text-muted-foreground">Efficiency</span>
                        <span className="font-medium">{efficiency}%</span>
                    </div>
                    <Progress className="h-1.5" value={Math.min(efficiency, 200) / 2} />
                </div>
            </CardContent>
        </Card>
    );
}
