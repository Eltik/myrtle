import { type ServerResponse } from "http";
import { env } from "~/env.js";
import type { AKServer } from "~/types/impl/api";
import type { SendCodeResponse } from "~/types/impl/api/impl/send-code";

export default async function handler(request: Request, response: ServerResponse) {
    const data = (await (
        await fetch(`${env.BACKEND_URL}/send-code`, {
            method: "POST",
            headers: {
                "Content-Type": "application/json",
            },
            body: JSON.stringify({
                email: request.body.email,
                server: request.body.server,
            }),
        })
    ).json()) as SendCodeResponse;

    response.writeHead(200, { "Content-Type": "application/json" });
    response.write(JSON.stringify(data));
    response.end();
};

interface Request {
    body: {
        email: string;
        server: AKServer;
    };
}
