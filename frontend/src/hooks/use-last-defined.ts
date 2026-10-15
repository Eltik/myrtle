import { useRef } from "react";

/** Keeps showing the last non-null value, e.g. while a dialog or sheet animates closed. */
export function useLastDefined<T>(value: T | null): T | null {
    const last = useRef(value);
    if (value !== null) last.current = value;
    return value ?? last.current;
}
