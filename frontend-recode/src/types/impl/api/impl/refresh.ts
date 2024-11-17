import type { User } from "..";

export type RefreshResponse = User &{
    error?: string;
    message?: string;
}