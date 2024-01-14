import { type ServerResponse } from "http";
import { env } from "~/env.mjs";
import { type Server, type CodeResponse } from "~/types/types";

export default async function handler(request: Request, response: ServerResponse) {
    const data = await (
        await fetch(`${env.BACKEND_URL}/send-code`, {
            method: "POST",
            headers: {
                "Content-Type": "application/json",
            },
            body: JSON.stringify({
                email: request.body.email,
                server: request.body.server
            }),
        })
    ).json() as CodeResponse;

    response.writeHead(200, { "Content-Type": "application/json" });
    response.write(JSON.stringify(data));
    response.end();
}

interface Request {
    body: {
        email: string;
        server: Server;
    };
}
