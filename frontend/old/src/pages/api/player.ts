import type { ServerResponse } from "node:http";
import { env } from "~/env.js";
import type { AKServer } from "~/types/impl/api";
import type { PlayerResponse } from "~/types/impl/api/impl/player";

export default async function handler(request: Request, response: ServerResponse) {
    const data = (await (
        await fetch(`${env.BACKEND_URL}/player`, {
            method: "POST",
            headers: {
                "Content-Type": "application/json",
            },
            body: JSON.stringify({
                uid: request.body.uid,
                server: request.body.server,
            }),
        })
    ).json()) as PlayerResponse;

    response.writeHead(200, { "Content-Type": "application/json" });
    response.write(JSON.stringify(data));
    response.end();
}

interface Request {
    body: {
        uid: string;
        server: AKServer;
    };
}
