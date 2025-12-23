export type AKServer = "en" | "jp" | "kr" | "cn" | "bili" | "tw";

export interface LoginProps {
    variant?: "default" | "header";
}

export interface SendCodeResponse {
    success: boolean;
    error?: string;
}

export interface LoginResponse {
    success: boolean;
    error?: string;
}
