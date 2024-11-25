import { type ServerResponse } from "http";
import type Module from "module";
import { env } from "~/env.js";
import type { Item } from "~/types/impl/api/static/material";
import type { ModuleData, Modules } from "~/types/impl/api/static/modules";
import type { Operator } from "~/types/impl/api/static/operator";
import type { Ranges } from "~/types/impl/api/static/ranges";
import type { Skill } from "~/types/impl/api/static/skills";

export default async function handler(request: Request, response: ServerResponse) {
    switch (request.body.type) {
        case "materials":
            const materials = (await (
                await fetch(`${env.BACKEND_URL}/static`, {
                    method: "POST",
                    headers: {
                        "Content-Type": "application/json",
                    },
                    body: JSON.stringify({
                        type: request.body.type,
                        id: request.body.id,
                    }),
                })
            ).json()) as {
                materials: Item[];
            };

            response.writeHead(200, { "Content-Type": "application/json" });
            response.write(
                JSON.stringify({
                    data: materials.materials,
                }),
            );
            return response.end();
        case "modules":
            const modules = (await (
                await fetch(`${env.BACKEND_URL}/static`, {
                    method: "POST",
                    headers: {
                        "Content-Type": "application/json",
                    },
                    body: JSON.stringify({
                        type: request.body.type,
                        id: request.body.id,
                        method: request.body.method,
                    }),
                })
            ).json()) as
                | {
                      modules: Modules;
                  }
                | {
                      details: ModuleData;
                  }
                | {
                      modules: Module[];
                  }
                | {
                      modules: Module;
                  };

            response.writeHead(200, { "Content-Type": "application/json" });
            response.write(JSON.stringify(modules));
            return response.end();
        case "operators":
            const operators = (await (
                await fetch(`${env.BACKEND_URL}/static`, {
                    method: "POST",
                    headers: {
                        "Content-Type": "application/json",
                    },
                    body: JSON.stringify({
                        type: request.body.type,
                        id: request.body.id,
                    }),
                })
            ).json()) as {
                operators: Operator[];
            };

            response.writeHead(200, { "Content-Type": "application/json" });
            response.write(
                JSON.stringify({
                    data: operators.operators,
                }),
            );
            return response.end();
        case "ranges":
            const ranges = (await (
                await fetch(`${env.BACKEND_URL}/static`, {
                    method: "POST",
                    headers: {
                        "Content-Type": "application/json",
                    },
                    body: JSON.stringify({
                        type: request.body.type,
                        id: request.body.id,
                    }),
                })
            ).json()) as {
                range: Ranges;
            };

            response.writeHead(200, { "Content-Type": "application/json" });
            response.write(
                JSON.stringify({
                    data: ranges.range,
                }),
            );
            return response.end();
        case "skills":
            const skills = (await (
                await fetch(`${env.BACKEND_URL}/static`, {
                    method: "POST",
                    headers: {
                        "Content-Type": "application/json",
                    },
                    body: JSON.stringify({
                        type: request.body.type,
                        id: request.body.id,
                    }),
                })
            ).json()) as {
                skills: Skill[];
            };

            response.writeHead(200, { "Content-Type": "application/json" });
            response.write(
                JSON.stringify({
                    data: skills.skills,
                }),
            );
            return response.end();
        case "trust":
            const trust = (await (
                await fetch(`${env.BACKEND_URL}/static`, {
                    method: "POST",
                    headers: {
                        "Content-Type": "application/json",
                    },
                    body: JSON.stringify({
                        type: request.body.type,
                        id: request.body.id,
                        trust: request.body.trust,
                    }),
                })
            ).json()) as {
                trust: number | null;
            };

            response.writeHead(200, { "Content-Type": "application/json" });
            response.write(
                JSON.stringify({
                    data: trust.trust,
                }),
            );
            return response.end();
        default:
            response.writeHead(400, { "Content-Type": "application/json" });
            response.write(
                JSON.stringify({
                    error: "Invalid type.",
                }),
            );
            return response.end();
    }
}

interface Request {
    body: {
        type: "materials" | "modules" | "operators" | "ranges" | "skills" | "trust";
        id?: string;
        method?: string;
        trust?: number;
    };
}
