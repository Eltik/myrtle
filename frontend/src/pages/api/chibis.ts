import { type ServerResponse } from "http";
import { env } from "~/env.js";
import type { ChibiOperatorList, Chibis, ChibisSimplified } from "~/types/impl/api/impl/chibis";

export default async function handler(request: Request, response: ServerResponse) {
    // In the backend implementation, you'll need to modify the processCharsForFrontend function
    // to include the animationTypes property in the returned data.
    // The response should contain information about which animation types (front, back, dorm)
    // are available for each skin.

    const data = (await (
        await fetch(`${env.BACKEND_URL}/chibis`, {
            method: "POST",
            headers: {
                "Content-Type": "application/json",
            },
            body: JSON.stringify({
                id: request.body.id,
                format: request.body.format,
            }),
        })
    ).json()) as ChibisSimplified | ChibiOperatorList | Chibis;

    response.writeHead(200, { "Content-Type": "application/json" });
    response.write(JSON.stringify(data));
    response.end();
}

interface Request {
    body: {
        id?: string;
        format?: "full" | "simplified" | "operatorList";
    };
}
