import { checkSpecs } from "./check-specs";

export function checkResetAttack(
    key: string,
    blackboard: {
        key: string;
        value: number;
        valueStr?: string | null;
    }[],
    options: any,
) {
    if (String(checkSpecs(key, "reset_attack")) === "false") return false;
    else if (checkSpecs(key, "overdrive") && !options.overdrive_mode) return false;
    else
        return (
            checkSpecs(key, "reset_attack") ||
            (
                blackboard as unknown as {
                    base_attack_time: number;
                }
            )["base_attack_time"] ||
            (
                blackboard as unknown as {
                    ["attack@max_target"]: number;
                }
            )["attack@max_target"] ||
            (
                blackboard as unknown as {
                    ["max_target"]: number;
                }
            )["max_target"]
        );
}
