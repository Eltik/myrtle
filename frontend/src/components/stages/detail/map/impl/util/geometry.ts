export const TILE_SIZE = 48;
export const SPACING = 2;

export interface IViewBox {
    width: number;
    height: number;
}

export function viewBox(width: number, height: number, tileSize = TILE_SIZE, spacing = SPACING): IViewBox {
    return {
        width: width * tileSize + (width - 1) * spacing,
        height: height * tileSize + (height - 1) * spacing,
    };
}

export interface ICenterArgs {
    row: number;
    col: number;
    reachOffset?: { x: number; y: number } | null;
}

export function tileCenter({ row, col, reachOffset }: ICenterArgs, tileSize = TILE_SIZE, spacing = SPACING): { x: number; y: number } {
    let x = tileSize / 2 + col * (tileSize + spacing);
    let y = tileSize / 2 + row * (tileSize + spacing);
    if (reachOffset) {
        x += reachOffset.x * (tileSize + spacing);
        y += reachOffset.y * (tileSize + spacing);
    }
    return { x, y };
}
