import type { ManufactureRoom, TradingRoom } from "~/types/api/impl/user";

export function calculateEfficiency(display: TradingRoom["display"] | ManufactureRoom["display"]): number {
    return display ? display.base + display.buff : 100;
}

export function formatTradingStrategy(strategy: string | undefined): string {
    return strategy === "O_GOLD" ? "LMD" : "Orundum";
}
