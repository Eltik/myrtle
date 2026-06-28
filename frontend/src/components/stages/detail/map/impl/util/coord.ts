const LETTERS = Array.from({ length: 26 }, (_, i) => String.fromCharCode(65 + i));
const PREFIX = [" ", ...LETTERS];

const COLUMN_SEQUENCE: string[] = ([] as string[]).concat(...Array.from({ length: LETTERS.length + 1 }, (_, n) => LETTERS.map((l) => `${PREFIX[n]}${l}`.trim())));

export interface ICoordArgs {
    row: number;
    col: number;
    height: number;
    override?: string | null;
}

export function coordLabel({ row, col, height, override }: ICoordArgs): string {
    if (override === "number") return `${col},${row}`;
    if (override === "maa") return `${col},${height - row - 1}`;
    return `${COLUMN_SEQUENCE[row]}${col + 1}`;
}
