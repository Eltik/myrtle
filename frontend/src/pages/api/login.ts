import { type ServerResponse } from "http";
import { env } from "~/env.js";
import type { AKServer } from "~/types/impl/api";
import type { LoginResponse } from "~/types/impl/api/impl/login";

export default async function handler(request: Request, response: ServerResponse) {
    const data = (await (
        await fetch(`${env.BACKEND_URL}/login`, {
            method: "POST",
            headers: {
                "Content-Type": "application/json",
            },
            body: JSON.stringify({
                email: request.body.email,
                code: request.body.code,
                server: request.body.server,
            }),
        })
    ).json()) as LoginResponse;

    response.writeHead(200, { "Content-Type": "application/json" });
    response.write(JSON.stringify(data));
    response.end();
}

interface Request {
    body: {
        email: string;
        code: number;
        server: AKServer;
    };
}
