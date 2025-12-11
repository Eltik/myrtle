import { type ServerResponse } from "http";
import { env } from "~/env.js";
import type { ChibiOperatorList, Chibis, ChibisSimplified } from "~/types/impl/api/impl/chibis";

export default async function handler(request: Request, response: ServerResponse) {
    // In the backend implementation, you'll need to modify the processCharsForFrontend function
    // to include the animationTypes property in the returned data.
    // The response should contain information about which animation types (front, back, dorm)
    // are available for each skin.

    // Determine if we're in development mode
    const isDevelopment = env.NODE_ENV === "development";

    // Set cache control header based on environment
    const headers: Record<string, string> = {
        "Content-Type": "application/json",
    };

    // In development, explicitly disable caching
    if (isDevelopment) {
        headers["Cache-Control"] = "no-store, max-age=0";
        console.log("Development mode: Chibis cache disabled");
    }

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

    // Set response headers, including cache control
    response.writeHead(200, headers);
    response.write(JSON.stringify(data));
    response.end();
}

interface Request {
    body: {
        id?: string;
        format?: "full" | "simplified" | "operatorList";
    };
}
