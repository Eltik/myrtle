import styles from "./Board.module.css";

interface IProps {
    max: number;
    current: number;
}

export const LevelPips = ({ max, current }: IProps) => (
    <span className={styles["riic-tile-level-pips"]}>
        {Array.from({ length: max }, (_, i) => (
            // biome-ignore lint/suspicious/noArrayIndexKey: pips are positional, there is no other identity
            <span key={i} className={styles["riic-tile-level-pip"]} data-on={i < current} />
        ))}
    </span>
);
