import type { AKServer } from "~/types/api";

export const SERVER_OPTIONS: { value: AKServer; label: string }[] = [
    { value: "en", label: "Global (EN)" },
    { value: "jp", label: "Japan (JP)" },
    { value: "kr", label: "Korea (KR)" },
    { value: "cn", label: "China (CN)" },
    { value: "bili", label: "Bilibili" },
    { value: "tw", label: "Taiwan (TW)" },
];
