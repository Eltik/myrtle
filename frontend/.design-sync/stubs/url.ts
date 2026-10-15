// Design-bundle stand-in for Node's `url` polyfill (~110 KB with its qs /
// object-inspect / get-intrinsic chain). Only `@pixi/utils` imports it, to expose
// `utils.url.{parse,format,resolve}`, which Pixi itself deprecated in 7.3 in favour of
// the native URL API; nothing in this app calls them. Native-URL equivalents keep
// the shape. The bundle must stay under the 12 MiB upload cap.
export function resolve(from: string, to: string): string {
    return new URL(to, new URL(from, "resolve://")).href.replace(/^resolve:\/\//, "");
}

export function parse(href: string) {
    const u = new URL(href, "resolve://");
    return { href, protocol: u.protocol, host: u.host, hostname: u.hostname, port: u.port, pathname: u.pathname, search: u.search, hash: u.hash };
}

export function format(obj: { href?: string } | string): string {
    return typeof obj === "string" ? obj : (obj.href ?? "");
}

export default { resolve, parse, format };
