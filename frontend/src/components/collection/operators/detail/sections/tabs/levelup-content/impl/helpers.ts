// Helper function to format attribute keys to readable names
export function formatStatKey(key: string): string {
    const statKeyMap: Record<string, string> = {
        atk: "ATK",
        def: "DEF",
        max_hp: "Max HP",
        attack_speed: "ASPD",
        magic_resistance: "RES",
        cost: "DP Cost",
        respawn_time: "Redeploy",
        block_cnt: "Block",
        hp_recovery_per_sec: "HP Regen",
        sp_recovery_per_sec: "SP Regen",
        base_attack_time: "Attack Interval",
    };
    return statKeyMap[key.toLowerCase()] ?? key.replace(/_/g, " ").replace(/\b\w/g, (c) => c.toUpperCase());
}

// Helper function to format stat values (percentages vs flat values)
export function formatStatValue(value: number): string {
    // Values less than 2 and greater than -2 are typically percentages (e.g., 0.15 = 15%)
    if (Math.abs(value) < 2 && value !== 0 && !Number.isInteger(value)) {
        const percentage = Math.round(value * 100);
        return `${percentage >= 0 ? "+" : ""}${percentage}%`;
    }
    return `${value >= 0 ? "+" : ""}${value}`;
}
