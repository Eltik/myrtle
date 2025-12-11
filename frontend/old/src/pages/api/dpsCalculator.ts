import type { ServerResponse } from "node:http";
import { env } from "~/env.js";
import type { DPSCalculatorResponse, OperatorParams } from "~/types/impl/api/impl/dps-calculator";

export default async function handler(request: Request, response: ServerResponse) {
    const data = (await (
        await fetch(`${env.BACKEND_URL}/dps-calculator`, {
            method: "POST",
            headers: {
                "Content-Type": "application/json",
            },
            body: JSON.stringify(request.body),
        })
    ).json()) as DPSCalculatorResponse;

    response.writeHead(200, { "Content-Type": "application/json" });
    response.write(JSON.stringify(data));
    response.end();
}

interface Request {
    body: {
        method: "operator" | "dps";
        id?: string;
        params?: OperatorParams;
        range?: {
            minDef?: number;
            maxDef?: number;
            minRes?: number;
            maxRes?: number;
        };
    };
}
