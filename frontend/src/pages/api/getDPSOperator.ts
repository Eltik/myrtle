import { type ServerResponse } from "http";
import { env } from "~/env.js";
import type { DPSOperatorResponse } from "~/types/impl/api/impl/dps-calculator";

export default async function handler(request: Request, response: ServerResponse) {
    const data = (await (
        await fetch(`${env.BACKEND_URL}/dps-operator`, {
            method: "POST",
            headers: {
                "Content-Type": "application/json",
            },
            body: JSON.stringify({
                id: request.body.id,
            }),
        })
    ).json()) as DPSOperatorResponse;

    response.writeHead(200, { "Content-Type": "application/json" });
    response.write(JSON.stringify(data));
    response.end();
}

interface Request {
    body: {
        id: string;
    };
}
