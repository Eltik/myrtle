import { type ServerResponse } from "http";
import { env } from "~/env.mjs";
import { type PlayerData, type Server } from "~/types/types";

export default async function handler(request: Request, response: ServerResponse) {
    const data = (await (
        await fetch(`${env.BACKEND_URL}/refresh`, {
            method: "POST",
            headers: {
                "Content-Type": "application/json",
            },
            body: JSON.stringify({
                uid: request.body.uid,
                email: request.body.email,
                secret: request.body.secret,
                seqnum: request.body.seqnum,
                server: request.body.server,
            }),
        })
    ).json()) as PlayerData;

    response.writeHead(200, { "Content-Type": "application/json" });
    response.write(JSON.stringify(data));
    response.end();
}

interface Request {
    body: {
        uid: string;
        email: string;
        secret: string;
        seqnum: number;
        server: Server;
    };
}
