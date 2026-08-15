import { Badge, Card, CardContent, CardDescription, CardHeader, CardKV, CardTitle, HCode } from "frontend";

export function RuntimeProbe() {
    return (
        <Card className="max-w-md">
            <CardHeader>
                <CardTitle className="text-sm">Runtime probe</CardTitle>
                <CardDescription className="text-xs">
                    Live data from <HCode>GET /health</HCode>.
                </CardDescription>
            </CardHeader>
            <CardContent className="pt-0">
                <CardKV k="Cache backend" v="redis" />
                <CardKV k="Cache status" v="connected" />
                <CardKV k="Database status" v="connected" />
                <CardKV k="Service status" v="ok" />
                <CardKV k="Probe timestamp" v="2024-05-15T12:00:04Z" />
            </CardContent>
        </Card>
    );
}

export function SignedInSession() {
    return (
        <Card className="max-w-md">
            <CardHeader>
                <CardTitle className="text-sm">Signed-in User</CardTitle>
                <CardDescription className="text-xs">Your current session.</CardDescription>
            </CardHeader>
            <CardContent className="pt-0">
                <CardKV k="Nickname" v="Kyostinv" />
                <CardKV k="UID" v="10289471" />
                <CardKV k="Server" v="EN" />
                <CardKV
                    k="Role"
                    v={
                        <>
                            super_admin
                            <Badge variant="success" className="ml-2">
                                super
                            </Badge>
                        </>
                    }
                />
                <CardKV k="Total score" v="8,412" />
            </CardContent>
        </Card>
    );
}

export function GameDataSnapshot() {
    return (
        <Card className="max-w-md">
            <CardHeader>
                <CardTitle className="text-sm">Loaded game data</CardTitle>
                <CardDescription className="text-xs">
                    Resident in <HCode>Arc&lt;GameData&gt;</HCode>.
                </CardDescription>
            </CardHeader>
            <CardContent className="pt-0">
                <CardKV k="Operators" v="438" />
                <CardKV k="Skills" v="1,207" />
                <CardKV k="Modules" v="612" />
                <CardKV k="Stages" v="4,812" />
                <CardKV k="Enemies" v="1,163" />
            </CardContent>
        </Card>
    );
}
