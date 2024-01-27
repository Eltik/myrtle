import { type ServerResponse } from "http";
import { env } from "~/env.mjs";
import { type SearchResponse, type Server } from "~/types/types";

export default async function handler(request: Request, response: ServerResponse) {
    const data = (await (
        await fetch(`${env.BACKEND_URL}/search-players`, {
            method: "POST",
            headers: {
                "Content-Type": "application/json",
            },
            body: JSON.stringify({
                nickname: request.body.nickname,
                nicknumber: request.body.nicknumber,
                limit: request.body.limit ?? 10,
                
                uid: request.body.uid,
                secret: request.body.secret,
                seqnum: request.body.seqnum,
                server: request.body.server,
            }),
        })
    ).json()) as SearchResponse[];

    response.writeHead(200, { "Content-Type": "application/json" });
    response.write(JSON.stringify(data));
    response.end();
}

interface Request {
    body: {
        nickname: string;
        nicknumber: string;
        limit?: number;

        uid: string;
        secret: string;
        seqnum: number;
        server: Server;
    };
}
