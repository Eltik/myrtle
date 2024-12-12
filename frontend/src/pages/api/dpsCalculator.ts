import { type ServerResponse } from "http";
import { env } from "~/env.js";
import type { DPSCalculatorResponse, OperatorParams } from "~/types/impl/api/impl/dps-calculator";

export default async function handler(request: Request, response: ServerResponse) {
    const data = (await (
        await fetch(`${env.BACKEND_URL}/dps-calculator`, {
            method: "POST",
            headers: {
                "Content-Type": "application/json",
            },
            body: JSON.stringify({
                id: request.body.id,
                params: request.body.params,
                enemy: request.body.enemy,
            }),
        })
    ).json()) as DPSCalculatorResponse;

    response.writeHead(200, { "Content-Type": "application/json" });
    response.write(JSON.stringify(data));
    response.end();
}

interface Request {
    body: {
        id: string;
        params?: OperatorParams;
        enemy?: {
            defense?: number;
            res?: number;
        };
    };
}
