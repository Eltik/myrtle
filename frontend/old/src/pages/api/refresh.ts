import type { ServerResponse } from "node:http";
import { env } from "~/env.js";
import type { AKServer } from "~/types/impl/api";
import type { RefreshResponse } from "~/types/impl/api/impl/refresh";

export default async function handler(request: Request, response: ServerResponse) {
    const data = (await (
        await fetch(`${env.BACKEND_URL}/refresh`, {
            method: "POST",
            headers: {
                "Content-Type": "application/json",
            },
            body: JSON.stringify({
                uid: request.body.uid,
                secret: request.body.secret,
                seqnum: request.body.seqnum,
                server: request.body.server,
            }),
        })
    ).json()) as RefreshResponse;

    response.writeHead(200, { "Content-Type": "application/json" });
    response.write(JSON.stringify(data));
    response.end();
}

interface Request {
    body: {
        uid: string;
        secret: string;
        seqnum: number;
        server: AKServer;
    };
}
