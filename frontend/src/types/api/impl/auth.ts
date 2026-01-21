// Authentication API response types

export interface SendCodeResponse {
    success: boolean;
    error?: string;
}

export interface LoginResponse {
    success: boolean;
    error?: string;
}
