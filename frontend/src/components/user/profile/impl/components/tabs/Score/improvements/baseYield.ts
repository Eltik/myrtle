import type { IBaseAssignment, IRoomAssignment } from "#/lib/api/user";

/** Compact number: 25700 → "25.7k", 1_240_000 → "1.24M". */
export function compactNum(n: number): string {
    const abs = Math.abs(n);
    if (abs >= 1_000_000) return `${(n / 1_000_000).toFixed(2).replace(/\.?0+$/, "")}M`;
    if (abs >= 1_000) return `${(n / 1_000).toFixed(1).replace(/\.0$/, "")}k`;
    return Math.round(n).toString();
}

/** Signed compact number for deltas: "+25.7k", "-1.2k", "+0". */
export function signedCompact(n: number): string {
    const sign = n > 0 ? "+" : n < 0 ? "−" : "";
    return `${sign}${compactNum(Math.abs(n))}`;
}

export interface RoomYieldInfo {
    /** Resource short label, e.g. "LMD", "gold", "EXP". */
    unit: "LMD" | "gold" | "EXP" | null;
    /** Per-day amount in that unit. */
    amount: number;
}

/** Which resource a room produces, and how much per day. */
export function roomYield(room: IRoomAssignment): RoomYieldInfo {
    if (room.yield_gold_per_day > 0) return { unit: "gold", amount: room.yield_gold_per_day };
    if (room.yield_exp_per_day > 0) return { unit: "EXP", amount: room.yield_exp_per_day };
    if (room.yield_lmd_per_day > 0) return { unit: "LMD", amount: room.yield_lmd_per_day };
    return { unit: null, amount: 0 };
}

/** Display label for a room's yield, e.g. "39 gold/day" or "25.7k LMD/day*". */
export function roomYieldLabel(room: IRoomAssignment): string | null {
    const y = roomYield(room);
    if (!y.unit) return null;
    // Trading posts show *potential* LMD (depends on gold supply).
    const star = y.unit === "LMD" ? "*" : "";
    return `${compactNum(y.amount)} ${y.unit}/day${star}`;
}

export interface AssignmentTotals {
    lmd: number;
    exp: number;
    /** Combined LMD-equivalent (EXP folded at 1:1). */
    value: number;
}

export function assignmentTotals(asn: IBaseAssignment): AssignmentTotals {
    return { lmd: asn.yield_lmd_per_day, exp: asn.yield_exp_per_day, value: asn.yield_total_value };
}
