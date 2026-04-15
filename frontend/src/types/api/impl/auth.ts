// Authentication API types for v3 backend

export interface SendCodeResponse {
    status: string;
}

export interface LoginResponse {
    token: string;
    uid: string;
    server: string;
}

export interface VerifyResponse {
    valid: boolean;
    userId: string;
    uid: string;
    server: string;
}
