import type { ReactNode } from "react";
import { OG_CONFIG } from "../config";

// Shared for every OG image
export function Frame({ children, accent = "#5B8DEF" }: { children: ReactNode; accent?: string }) {
    return (
        <div
            style={{
                width: "100%",
                height: "100%",
                display: "flex",
                flexDirection: "column",
                background: "linear-gradient(135deg, #0F1729 0%, #1B2541 100%)",
                color: "#F5F7FB",
                padding: 64,
                fontFamily: "Inter",
            }}
        >
            <div style={{ display: "flex", alignItems: "center", gap: 12 }}>
                <div style={{ width: 14, height: 14, borderRadius: 999, background: accent }} />
                <div style={{ fontSize: 22, fontWeight: 700, letterSpacing: 1, opacity: 0.85 }}>{OG_CONFIG.siteName.toUpperCase()}</div>
            </div>
            <div style={{ flex: 1, display: "flex", flexDirection: "column", justifyContent: "center" }}>{children}</div>
            <div style={{ display: "flex", justifyContent: "space-between", alignItems: "center", fontSize: 20, opacity: 0.55 }}>
                <div>{OG_CONFIG.siteUrl.replace(/^https?:\/\//, "")}</div>
                <div>{`v${OG_CONFIG.designVersion}`}</div>
            </div>
        </div>
    );
}
