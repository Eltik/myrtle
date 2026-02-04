import { RARITY_COLORS } from "../../constants";

interface RarityStarsProps {
    rarity: number;
    className?: string;
    starClassName?: string;
}

export function RarityStars({ rarity, className, starClassName }: RarityStarsProps) {
    const color = RARITY_COLORS[rarity] ?? "#ffffff";

    return (
        <div className={className}>
            {Array.from({ length: rarity }).map((_, i) => (
                // biome-ignore lint/suspicious/noArrayIndexKey: Static array of stars based on rarity count, order never changes
                <span className={starClassName} key={i} style={{ color }}>
                    ★
                </span>
            ))}
        </div>
    );
}
