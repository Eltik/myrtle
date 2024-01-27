import { type ServerResponse } from "http";
import { env } from "~/env.mjs";
import { type PlayerData, type Server } from "~/types/types";

export default async function handler(request: Request, response: ServerResponse) {
    const data = (await (
        await fetch(`${env.BACKEND_URL}/search`, {
            method: "POST",
            headers: {
                "Content-Type": "application/json",
            },
            body: JSON.stringify({
                nickname: request.body.nickname,
                nicknumber: request.body.nicknumber,
                server: request.body.server,
            }),
        })
    ).json()) as PlayerData[];

    response.writeHead(200, { "Content-Type": "application/json" });
    response.write(JSON.stringify(data));
    response.end();
}

interface Request {
    body: {
        nickname: string;
        nicknumber?: string;
        server?: Server;
    };
}
