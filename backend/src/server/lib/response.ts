export const createResponse = (data: unknown, status: number = 200, headers: { [key: string]: string } = {}) => {
    return new Response(data as null | undefined, {
        status,
        headers: {
            "Content-Type": "application/json",
            "Access-Control-Allow-Origin": "*",
            "Access-Control-Allow-Methods": "GET, POST, OPTIONS",
            "Access-Control-Max-Age": "2592000",
            "Access-Control-Allow-Headers": "*",
            ...headers,
        },
    });
};
